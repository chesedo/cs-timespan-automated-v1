#!/usr/bin/env python3
"""Compare a ported Rust crate against its upstream C# source/tests and file a
GitHub issue for each behavioral gap found.

The scan/verify system prompts are NOT hardcoded here — they're loaded from
this plugin's drift-scan.md / drift-verify.md agent definitions (copied into
this repo at bootstrap time under prompts/), so the interactive Claude Code
agents and this unattended CI script share one source of truth for the
actual review logic. This script only adds the strict raw-JSON-only calling
convention those agent files don't need for interactive use.

Requires ANTHROPIC_API_KEY and GH_TOKEN in the environment, and the gh CLI.
Requires drift_config.json (see drift_config.json.example) alongside this
script: the upstream repo/branch once, the Rust files to audit, and the
C# source paths (relative to that repo) to compare against.

Pass --dry-run to run the full scan/verify pipeline without filing anything —
confirmed gaps are printed instead of created as GitHub issues.
"""

import http.client
import json
import os
import re
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request
from typing import Callable, TypeVar

T = TypeVar("T")

SCRIPT_DIR = os.path.dirname(__file__)
CONFIG_FILE = os.path.join(SCRIPT_DIR, "drift_config.json")
PROMPTS_DIR = os.path.join(SCRIPT_DIR, "prompts")
IGNORE_FILE = os.path.join(SCRIPT_DIR, "drift_ignore.md")

MODEL = "claude-sonnet-5"
MAX_TOKENS = 32000
THINKING_EFFORT = "high"
MAX_ISSUES_PER_RUN = 15
DRY_RUN = "--dry-run" in sys.argv

# Wrapped around the loaded agent-file bodies. The agent files describe *what*
# to look for and why; this is the *calling convention* needed only because
# this script parses the raw response as JSON with no human or Claude Code
# harness in the loop. Stated at both the start and end of the prompt, not
# just once — a single mid-prompt mention of "JSON only" is easy for the
# model to satisfy loosely (e.g. wrapping the JSON in a code fence) when nothing
# reinforces it right before generation begins.
STRICT_JSON_PREAMBLE = """\
CRITICAL OUTPUT RULE, read this first: your reply is fed straight into a JSON \
parser, never read by a person. The first character you output must be the \
opening bracket/brace of the JSON value described below, and the last must be \
its matching close. No ``` or ```json code fence, no prose before or after, no \
step-by-step trace written outside the JSON. Any reasoning you want to keep \
goes inside the JSON's own text fields — writing it as a preamble breaks the \
parser and silently discards the finding, so don't do it, even out of habit.

"""

STRICT_JSON_POSTAMBLE = """

Reminder: raw JSON only, no ``` fence, no prose before or after. Begin your
reply with the opening bracket/brace right now.
"""


def strip_frontmatter(md: str) -> str:
    """Drop the --- ... --- YAML frontmatter block from an agent .md file,
    keeping only the system-prompt body."""
    match = re.match(r"^---\n.*?\n---\n", md, re.DOTALL)
    return md[match.end():] if match else md


def load_prompt(filename: str) -> str:
    path = os.path.join(PROMPTS_DIR, filename)
    try:
        with open(path, encoding="utf-8") as f:
            body = strip_frontmatter(f.read())
    except FileNotFoundError:
        print(
            f"Expected an agent prompt copy at {path}, but it doesn't exist.\n"
            f"This script reuses the csharp-porting-toolkit plugin's own "
            f"drift-scan.md/drift-verify.md agent definitions as its prompts, "
            f"rather than duplicating them — the bootstrap-port skill copies "
            f"those two files into .github/scripts/prompts/ when it scaffolds "
            f"a repo. If this repo predates that step, copy them in manually:\n"
            f"  cp <plugin>/agents/drift-scan.md <plugin>/agents/drift-verify.md "
            f"{PROMPTS_DIR}/",
            file=sys.stderr,
        )
        sys.exit(1)
    return STRICT_JSON_PREAMBLE + body + STRICT_JSON_POSTAMBLE


def load_config() -> dict:
    try:
        with open(CONFIG_FILE, encoding="utf-8") as f:
            config = json.load(f)
    except FileNotFoundError:
        print(
            f"{CONFIG_FILE} not found. Copy drift_config.json.example to "
            f"drift_config.json alongside this script and fill in "
            f"csharp_source_repo/rust_files/csharp_sources/label for this "
            f"project.",
            file=sys.stderr,
        )
        sys.exit(1)
    for key in ("rust_files", "csharp_sources", "label", "csharp_source_repo"):
        if key not in config:
            print(f"drift_config.json is missing required key: {key}", file=sys.stderr)
            sys.exit(1)
    config.setdefault("csharp_source_branch", "main")
    return config


def csharp_source_url(config: dict, path: str) -> str:
    """Build a raw.githubusercontent.com URL from the config's single
    repo/branch plus a path relative to it, so the repo/branch aren't
    repeated in every csharp_sources entry."""
    repo = config["csharp_source_repo"]
    branch = config["csharp_source_branch"]
    return f"https://raw.githubusercontent.com/{repo}/{branch}/{path}"


RETRYABLE_ERRORS = (
    urllib.error.URLError, ConnectionError, TimeoutError, socket.timeout,
    http.client.IncompleteRead,
)


def with_retries(fn: Callable[[], T], *, attempts: int = 3, base_delay: float = 5) -> T:
    """Retry fn() on transient network errors, since raw.githubusercontent.com and
    the Anthropic API both occasionally drop a connection or 404 for a moment."""
    for attempt in range(1, attempts + 1):
        try:
            return fn()
        except RETRYABLE_ERRORS as e:
            if attempt == attempts:
                raise
            delay = base_delay * attempt
            print(f"Attempt {attempt}/{attempts} failed ({e}); retrying in {delay}s ...", file=sys.stderr)
            time.sleep(delay)
    raise AssertionError("unreachable")


def fetch(url: str) -> str:
    print(f"Fetching {url} ...", file=sys.stderr)

    def do() -> str:
        with urllib.request.urlopen(url, timeout=30) as resp:
            return resp.read().decode("utf-8")

    return with_retries(do)


def read_local(path: str) -> str:
    with open(path, encoding="utf-8") as f:
        return f.read()


def existing_drift_titles(label: str) -> set[str]:
    print("Fetching already-filed drift issues ...", file=sys.stderr)
    out = subprocess.run(
        [
            "gh", "issue", "list", "--label", label,
            "--state", "all", "--limit", "200", "--json", "title",
        ],
        capture_output=True, text=True, check=True,
    )
    return {item["title"] for item in json.loads(out.stdout)}


def call_claude(system_prompt: str, user_prompt: str) -> str:
    # Streamed: with max_tokens=32000, a non-streaming call can take several
    # minutes to generate before Anthropic sends a single byte back, and an
    # idle connection that long gets killed by an intermediate proxy. Streaming
    # sends periodic SSE events (including keep-alive pings) the whole time.
    # Thinking is enabled rather than disabled: without it, the model has no
    # outlet for genuinely hard multi-step reasoning and ends up writing that
    # reasoning into the visible answer as prose before the JSON, breaking
    # the parser.
    body = json.dumps({
        "model": MODEL,
        "max_tokens": MAX_TOKENS,
        "system": system_prompt,
        "messages": [{"role": "user", "content": user_prompt}],
        "stream": True,
        "thinking": {"type": "adaptive"},
        "output_config": {"effort": THINKING_EFFORT},
    }).encode("utf-8")
    req = urllib.request.Request(
        "https://api.anthropic.com/v1/messages",
        data=body,
        headers={
            "content-type": "application/json",
            "x-api-key": os.environ["ANTHROPIC_API_KEY"],
            "anthropic-version": "2023-06-01",
        },
    )

    def do() -> tuple[str, str | None]:
        text_parts: list[str] = []
        stop_reason = None
        block_types: list[str] = []
        with urllib.request.urlopen(req, timeout=600) as resp:
            for raw_line in resp:
                line = raw_line.decode("utf-8").strip()
                if not line.startswith("data: "):
                    continue
                try:
                    event = json.loads(line[len("data: "):])
                except json.JSONDecodeError as e:
                    # A malformed data line means the stream was cut off
                    # mid-event (e.g. a dropped connection) rather than a
                    # genuinely bad payload from the API - treat it the same
                    # as the other transient network failures so it retries.
                    raise ConnectionError(f"malformed SSE event: {e}") from e
                event_type = event.get("type")
                if event_type == "content_block_start":
                    block_types.append(event["content_block"]["type"])
                elif event_type == "content_block_delta" and event["delta"].get("type") == "text_delta":
                    text_parts.append(event["delta"]["text"])
                elif event_type == "message_delta":
                    stop_reason = event["delta"].get("stop_reason")
                elif event_type == "error":
                    raise RuntimeError(event["error"].get("message", "unknown streaming error"))
                elif event_type == "message_stop":
                    break
        text = "".join(text_parts)
        if not text.strip() and block_types:
            print(f"Debug: stream produced no text; content block types seen: {block_types}", file=sys.stderr)
        return text, stop_reason

    try:
        text, stop_reason = with_retries(do)
    except (TimeoutError, socket.timeout):
        print("Anthropic API call timed out (600s) after retries. Try again.", file=sys.stderr)
        sys.exit(1)
    except urllib.error.HTTPError as e:
        print(f"Anthropic API returned HTTP {e.code}: {e.read().decode('utf-8')}", file=sys.stderr)
        sys.exit(1)
    except (urllib.error.URLError, ConnectionError) as e:
        print(f"Anthropic API request failed after retries: {e}", file=sys.stderr)
        sys.exit(1)
    except RuntimeError as e:
        print(f"Anthropic API returned a streaming error: {e}", file=sys.stderr)
        sys.exit(1)
    if stop_reason == "max_tokens":
        print(
            f"Warning: response was cut off (hit max_tokens={MAX_TOKENS}). The "
            f"JSON below is likely incomplete — consider raising MAX_TOKENS.",
            file=sys.stderr,
        )
    if not text.strip():
        print("Warning: no text content in Claude's streamed response.", file=sys.stderr)
    return text


def parse_json_response(raw: str, label: str) -> object:
    try:
        return json.loads(raw.strip())
    except json.JSONDecodeError:
        print(f"Claude did not return valid JSON ({label}). Raw response was:", file=sys.stderr)
        print("--- start raw response ---", file=sys.stderr)
        print(raw if raw.strip() else "(empty)", file=sys.stderr)
        print("--- end raw response ---", file=sys.stderr)
        sys.exit(1)


def fail_schema(label: str, data: object) -> None:
    print(f"Claude's response for {label} didn't match the expected schema:", file=sys.stderr)
    print(json.dumps(data), file=sys.stderr)
    sys.exit(1)


def scan_candidates(
    rust_blob: str, csharp_blob: str, known: set[str], ignore_notes: str
) -> list[dict]:
    print("Running scan pass (calling Claude) ...", file=sys.stderr)
    system_prompt = load_prompt("drift-scan.md")
    user_prompt = (
        f"Already-filed gaps (do not repeat these, by title):\n"
        f"{json.dumps(sorted(known))}\n\n"
        f"Accepted-as-intentional divergences (do NOT list candidates matching "
        f"these, even under a different title):\n{ignore_notes}\n\n"
        f"--- RUST SOURCE ---\n{rust_blob}\n\n"
        f"--- C# SOURCE/TESTS ---\n{csharp_blob}"
    )
    raw = call_claude(system_prompt, user_prompt)
    data = parse_json_response(raw, "scan")
    if not isinstance(data, list) or not all(
        isinstance(item, dict) and isinstance(item.get("id"), str) for item in data
    ):
        fail_schema("scan (expected a JSON array of drift-scan candidate objects)", data)
    return data


def verify_candidate(candidate: dict, ignore_notes: str) -> dict:
    system_prompt = load_prompt("drift-verify.md")
    # Exactly the candidate JSON, nothing else — no other candidates, no scan
    # reasoning, no hint about which candidates a prior pass already trusted.
    # This is what makes verification independent rather than a rubber stamp.
    user_prompt = (
        f"Candidate to verify:\n{json.dumps(candidate)}\n\n"
        f"Accepted-as-intentional divergences (reject the candidate if it "
        f"matches one of these, even under different wording):\n{ignore_notes}"
    )
    raw = call_claude(system_prompt, user_prompt)
    label = f"verify: {candidate.get('id', '?')}"
    data = parse_json_response(raw, label)
    if not isinstance(data, dict) or data.get("verdict") not in ("CONFIRMED", "STALE", "FALSE"):
        fail_schema(f"{label} (expected an object with verdict CONFIRMED/STALE/FALSE)", data)
    if data["verdict"] == "CONFIRMED" and not (
        isinstance(data.get("title"), str) and isinstance(data.get("body"), str)
    ):
        fail_schema(f"{label} (verdict=CONFIRMED requires string 'title'/'body')", data)
    return data


def main() -> None:
    config = load_config()
    label = config["label"]

    print("Reading local Rust source ...", file=sys.stderr)
    rust_blob = "\n\n".join(
        f"// ===== {path} =====\n{read_local(path)}" for path in config["rust_files"]
    )
    csharp_blob = "\n\n".join(
        f"// ===== {name} =====\n{fetch(csharp_source_url(config, path))}"
        for name, path in config["csharp_sources"].items()
    )
    known = existing_drift_titles(label)
    ignore_notes = read_local(IGNORE_FILE) if os.path.exists(IGNORE_FILE) else ""

    candidates = scan_candidates(rust_blob, csharp_blob, known, ignore_notes)
    if not candidates:
        print("No candidates found.")
        return
    print(f"Scan found {len(candidates)} candidate(s).", file=sys.stderr)

    filed = 0
    for i, candidate in enumerate(candidates, start=1):
        if filed >= MAX_ISSUES_PER_RUN:
            print(f"Filed {MAX_ISSUES_PER_RUN} issues, stopping for this run.", file=sys.stderr)
            break

        cid = candidate.get("id", f"candidate-{i}")
        print(f"[{i}/{len(candidates)}] {cid}", file=sys.stderr)
        # Each candidate is verified in its own isolated call — no shared
        # context with the scan pass or with any other candidate — so an
        # earlier finding can't bias whether this one gets confirmed.
        verdict = verify_candidate(candidate, ignore_notes)

        if verdict["verdict"] != "CONFIRMED":
            print(f"{verdict['verdict']}: {cid} ({verdict.get('reason', 'no reason given')})")
            continue

        title = verdict["title"].strip()
        if title in known:
            continue
        body = verdict["body"] + "\n\n---\n_Filed automatically by the C# drift-check workflow._"

        if DRY_RUN:
            print(f"[dry run] Would file: {title}\n{body}\n")
        else:
            subprocess.run(
                ["gh", "issue", "create", "--label", label, "--title", title, "--body", body],
                check=True,
            )
            print(f"Filed: {title}")
        # Scan is deliberately over-inclusive, so two candidates in the same
        # run can independently verify to the same title — record it in
        # `known` immediately so a later candidate in this same loop doesn't
        # file a duplicate (existing_drift_titles() only covers prior runs).
        known.add(title)
        filed += 1

    if filed == 0:
        print("No drift confirmed after verification.")


if __name__ == "__main__":
    main()

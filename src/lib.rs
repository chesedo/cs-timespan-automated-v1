#![doc = include_str!("../README.md")]

mod error;
mod time_span;
mod time_span_builder;
mod time_span_format_custom;
mod time_span_parse;
mod time_span_parse_constant;
mod time_span_parse_exact;

pub use error::TimeSpanError;
pub use time_span::{TimeSpan, TimeSpanStyles};
pub use time_span_builder::TimeSpanBuilder;

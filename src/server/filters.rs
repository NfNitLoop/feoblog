///! Filters for askama.

use askama::Result;

use crate::markdown::ToHTML;
use crate::backend::Timestamp;

pub(crate) fn markdown(s: &str) -> Result<String> {
    Ok(s.md_to_html())
}


// Seems filters always accept by reference:
pub(crate) fn with_offset(utc_ms: &i64, offset_mins: &i32) -> Result<String> {
    let timestamp = Timestamp{
        unix_utc_ms: *utc_ms,
    };
    Ok(
        timestamp.format_with_offset(*offset_mins as i16)
    )
}
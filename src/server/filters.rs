///! Filters for askama.

use askama::Result;

use crate::markdown::ToHTML;

pub(crate) fn markdown(s: &str) -> Result<String> {
    Ok(s.md_to_html())
}
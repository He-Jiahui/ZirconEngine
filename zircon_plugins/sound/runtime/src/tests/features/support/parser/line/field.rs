#[path = "../../../../support/parser/line/field/bracketed.rs"]
mod bracketed;
#[path = "../../../../support/parser/line/field/quoted.rs"]
mod quoted;
#[path = "field/raw.rs"]
mod raw;

pub(super) use bracketed::bracketed_value;
pub(super) use quoted::quoted_value;
pub(super) use raw::raw_value;

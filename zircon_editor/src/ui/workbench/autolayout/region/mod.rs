#[path = "document_region.rs"]
mod document_region;
#[path = "tool_region/mod.rs"]
mod tool_region;

pub(super) use document_region::build_document_region_state;
pub(super) use tool_region::build_tool_region_state;

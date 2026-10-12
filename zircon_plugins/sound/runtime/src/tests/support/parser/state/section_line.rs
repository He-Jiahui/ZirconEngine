#[path = "../../../state/section_line/support_dependency.rs"]
mod dependency;
#[path = "../../../state/section_line/support_entry.rs"]
mod entry;
#[path = "../../../state/section_line/support_feature.rs"]
mod feature;
#[path = "../../../state/section_line/support_module.rs"]
mod module;

pub(super) use entry::parse_section_line;

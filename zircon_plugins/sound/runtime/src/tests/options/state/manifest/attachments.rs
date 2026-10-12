#[path = "../attachments/apply.rs"]
mod apply;
#[path = "../attachments/enum_values.rs"]
mod enum_values;
#[path = "../../../state/attachments/required_capability.rs"]
mod required_capability;

pub(super) use apply::apply_option_manifest_attachments;

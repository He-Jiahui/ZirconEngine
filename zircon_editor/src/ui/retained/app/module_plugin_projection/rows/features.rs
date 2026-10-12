#[path = "../../rows/features/action.rs"]
mod action;
#[path = "../../rows/features/summary.rs"]
mod summary;

pub(in crate::ui::retained_host::app::module_plugin_projection) use action::module_plugin_feature_action;
pub(in crate::ui::retained_host::app::module_plugin_projection) use summary::module_plugin_optional_feature_summary;

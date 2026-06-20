mod features;
mod labels;
mod manifest;

pub(super) use features::{module_plugin_feature_action, module_plugin_optional_feature_summary};
pub(super) use labels::{
    module_plugin_action_id, module_plugin_primary_action, packaging_label, target_mode_label,
};
pub(super) use manifest::fallback_project_manifest;

#[cfg(test)]
mod tests;

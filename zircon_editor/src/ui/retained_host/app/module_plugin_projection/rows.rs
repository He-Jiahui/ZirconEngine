mod features;
mod labels;

pub(super) use features::{module_plugin_feature_action, module_plugin_optional_feature_summary};
pub(super) use labels::{
    module_plugin_action_id, module_plugin_primary_action, packaging_label, target_mode_label,
};

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use super::FeatureDefinition;

#[derive(Clone, Debug)]
pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) struct FeatureDefinitionMap {
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) definitions:
        HashMap<String, FeatureDefinition>,
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) diagnostics: Vec<String>,
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) definition_order: Vec<String>,
}

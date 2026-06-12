use crate::{plugin::ExportPackagingStrategy, plugin::PluginMaturity, RuntimePluginId};

use super::super::RuntimePluginDescriptor;

impl RuntimePluginDescriptor {
    pub fn new(
        package_id: impl Into<String>,
        display_name: impl Into<String>,
        runtime_id: RuntimePluginId,
        crate_name: impl Into<String>,
    ) -> Self {
        Self {
            package_id: package_id.into(),
            display_name: display_name.into(),
            category: "runtime".to_string(),
            runtime_id,
            crate_name: crate_name.into(),
            enabled_by_default: true,
            required_by_default: false,
            target_modes: Vec::new(),
            capabilities: Vec::new(),
            system_sets: Vec::new(),
            system_anchors: Vec::new(),
            capability_statuses: Vec::new(),
            maturity: PluginMaturity::default(),
            optional_features: Vec::new(),
            default_packaging: vec![
                ExportPackagingStrategy::SourceTemplate,
                ExportPackagingStrategy::LibraryEmbed,
            ],
        }
    }
}

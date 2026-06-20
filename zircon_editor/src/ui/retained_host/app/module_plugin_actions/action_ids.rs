mod parser;

pub(super) use parser::parse_module_plugin_action;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum ModulePluginAction<'a> {
    SetEnabled {
        plugin_id: &'a str,
        enabled: bool,
    },
    CyclePackaging {
        plugin_id: &'a str,
    },
    CycleTargetModes {
        plugin_id: &'a str,
    },
    SetFeatureEnabled {
        plugin_id: &'a str,
        feature_id: &'a str,
        enabled: bool,
    },
    EnableFeatureDependencies {
        plugin_id: &'a str,
        feature_id: &'a str,
    },
    Unload {
        plugin_id: &'a str,
    },
    HotReload {
        plugin_id: &'a str,
    },
}

#[cfg(test)]
mod tests;

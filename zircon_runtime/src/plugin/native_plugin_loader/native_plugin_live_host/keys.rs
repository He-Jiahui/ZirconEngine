use crate::plugin::PluginModuleKind;

pub(super) fn live_key(module_kind: PluginModuleKind, plugin_id: &str) -> String {
    format!("{}{plugin_id}", live_key_prefix(module_kind))
}

pub(super) fn live_key_prefix(module_kind: PluginModuleKind) -> &'static str {
    match module_kind {
        PluginModuleKind::Runtime => "runtime:",
        PluginModuleKind::Editor => "editor:",
        PluginModuleKind::Native => "native:",
        PluginModuleKind::Vm => "vm:",
    }
}

pub(super) fn module_kind_label(module_kind: PluginModuleKind) -> &'static str {
    match module_kind {
        PluginModuleKind::Runtime => "runtime",
        PluginModuleKind::Editor => "editor",
        PluginModuleKind::Native => "native",
        PluginModuleKind::Vm => "vm",
    }
}

pub(super) fn module_kind_article_label(module_kind: PluginModuleKind) -> &'static str {
    match module_kind {
        PluginModuleKind::Runtime => "a runtime",
        PluginModuleKind::Editor => "an editor",
        PluginModuleKind::Native => "a native",
        PluginModuleKind::Vm => "a vm",
    }
}

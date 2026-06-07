use super::super::super::StaticModule;

// Preserves the static plugin.toml scanner's table-boundary behavior for module rows.
#[derive(Default)]
pub(in super::super) struct ModuleContributionParserState {
    pub(in super::super) modules: Vec<StaticModule>,
    pub(in super::super) current_name: Option<String>,
    pub(in super::super) current_kind: Option<zircon_runtime::plugin::PluginModuleKind>,
    pub(in super::super) current_crate_name: Option<String>,
    pub(in super::super) current_target_modes: Vec<zircon_runtime::RuntimeTargetMode>,
    pub(in super::super) current_capabilities: Vec<String>,
    pub(in super::super) inside_module: bool,
}

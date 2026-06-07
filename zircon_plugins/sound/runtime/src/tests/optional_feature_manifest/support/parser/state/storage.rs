use super::super::super::types::{PendingOptionalFeatureManifest, StaticOptionalFeatureManifest};
use super::super::section::OptionalFeatureSection;

// Keeps the hand-rolled static TOML scanner's pending rows coherent across table changes.
#[derive(Default)]
pub(in super::super) struct OptionalFeatureParserState {
    pub(in super::super) features: Vec<StaticOptionalFeatureManifest>,
    pub(in super::super) current_feature: Option<PendingOptionalFeatureManifest>,
    pub(in super::super) current_dependency_plugin_id: Option<String>,
    pub(in super::super) current_dependency_capability: Option<String>,
    pub(in super::super) current_dependency_primary: Option<bool>,
    pub(in super::super) current_module_name: Option<String>,
    pub(in super::super) current_module_kind: Option<zircon_runtime::plugin::PluginModuleKind>,
    pub(in super::super) current_module_crate_name: Option<String>,
    pub(in super::super) current_module_target_modes: Vec<zircon_runtime::RuntimeTargetMode>,
    pub(in super::super) current_module_capabilities: Vec<String>,
    pub(in super::super) section: OptionalFeatureSection,
}

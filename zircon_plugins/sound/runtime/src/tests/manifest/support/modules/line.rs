#[path = "../../../support/modules/line/capabilities.rs"]
mod capabilities;
#[path = "line/dispatch.rs"]
mod dispatch;
#[path = "line/identity.rs"]
mod identity;
#[path = "line/kind.rs"]
mod kind;
#[path = "line/targets.rs"]
mod targets;

pub(super) fn parse_module_contribution_line(
    line: &str,
    name: &mut Option<String>,
    kind: &mut Option<zircon_runtime::plugin::PluginModuleKind>,
    crate_name: &mut Option<String>,
    target_modes: &mut Vec<zircon_runtime::core::framework::platform::RuntimeTargetMode>,
    capabilities: &mut Vec<String>,
) {
    dispatch::parse_module_contribution_line(
        line,
        name,
        kind,
        crate_name,
        target_modes,
        capabilities,
    );
}

use super::{capabilities, identity, kind, targets};

pub(super) fn parse_module_line(
    line: &str,
    name: &mut Option<String>,
    kind: &mut Option<zircon_runtime::plugin::PluginModuleKind>,
    crate_name: &mut Option<String>,
    target_modes: &mut Vec<zircon_runtime::builtin::RuntimeTargetMode>,
    capabilities: &mut Vec<String>,
) {
    if identity::parse_module_identity_line(line, name, crate_name) {
        return;
    }

    if kind::parse_module_kind_line(line, kind) {
        return;
    }

    if targets::parse_module_target_modes_line(line, target_modes) {
        return;
    }

    if capabilities::parse_module_capabilities_line(line, capabilities) {
        return;
    }
}

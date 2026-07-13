pub(in super::super::super) fn parse_optional_feature_module_line(
    line: &str,
    name: &mut Option<String>,
    kind: &mut Option<zircon_runtime::plugin::PluginModuleKind>,
    crate_name: &mut Option<String>,
    target_modes: &mut Vec<zircon_runtime::core::framework::platform::RuntimeTargetMode>,
    capabilities: &mut Vec<String>,
) {
    super::dispatch::parse_module_line(line, name, kind, crate_name, target_modes, capabilities);
}

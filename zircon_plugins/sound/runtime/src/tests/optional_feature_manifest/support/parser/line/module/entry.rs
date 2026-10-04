// 模块字段仅在所属区段汇入当前待提交记录，避免静态对照把跨表同名字段视为同一声明。
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

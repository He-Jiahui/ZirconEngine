pub(super) fn set_module_kind(
    kind: &mut Option<zircon_runtime::plugin::PluginModuleKind>,
    value: zircon_runtime::plugin::PluginModuleKind,
) {
    *kind = Some(value);
}

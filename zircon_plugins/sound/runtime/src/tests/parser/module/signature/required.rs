pub(super) fn take_required_module_kind(
    kind: &mut Option<zircon_runtime::plugin::PluginModuleKind>,
) -> zircon_runtime::plugin::PluginModuleKind {
    kind.take()
        .expect("optional feature module should declare kind")
}

pub(super) fn take_required_crate_name(crate_name: &mut Option<String>) -> String {
    crate_name
        .take()
        .expect("optional feature module should declare crate_name")
}

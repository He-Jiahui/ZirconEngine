pub(super) fn take_required_module_kind(
    value: &mut Option<zircon_runtime::plugin::PluginModuleKind>,
) -> zircon_runtime::plugin::PluginModuleKind {
    take_required_module_field(value, "sound module should declare kind")
}

pub(super) fn take_required_module_crate_name(value: &mut Option<String>) -> String {
    take_required_module_field(value, "sound module should declare crate_name")
}

fn take_required_module_field<T>(value: &mut Option<T>, missing_message: &'static str) -> T {
    value.take().expect(missing_message)
}

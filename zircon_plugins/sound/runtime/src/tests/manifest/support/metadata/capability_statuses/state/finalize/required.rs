pub(super) fn take_required_capability_status(
    value: &mut Option<zircon_runtime::plugin::CapabilityStatus>,
) -> zircon_runtime::plugin::CapabilityStatus {
    take_required_capability_status_field(value, "sound capability status should declare status")
}

fn take_required_capability_status_field<T>(
    value: &mut Option<T>,
    missing_message: &'static str,
) -> T {
    value.take().expect(missing_message)
}

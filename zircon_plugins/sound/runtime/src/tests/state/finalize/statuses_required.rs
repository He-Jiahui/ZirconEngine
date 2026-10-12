// 已识别的能力行必须声明状态；缺失时让测试失败，不把静态发布元数据默认为其他状态。
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

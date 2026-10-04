// 静态贡献行的必填项缺失即失败；测试不能为发布清单暗中补默认值。
pub(super) fn take_required_event_catalog_version(value: &mut Option<u32>) -> u32 {
    take_required_event_catalog_field(value, "sound event catalog should declare version")
}

fn take_required_event_catalog_field<T>(value: &mut Option<T>, missing_message: &'static str) -> T {
    value.take().expect(missing_message)
}

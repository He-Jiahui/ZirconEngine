// 静态贡献行的必填项缺失即失败；测试不能为发布清单暗中补默认值。
pub(super) fn take_required_dependency_required(value: &mut Option<bool>) -> bool {
    take_required_dependency_field(value, "sound dependency should declare required")
}

fn take_required_dependency_field<T>(value: &mut Option<T>, missing_message: &'static str) -> T {
    value.take().expect(missing_message)
}

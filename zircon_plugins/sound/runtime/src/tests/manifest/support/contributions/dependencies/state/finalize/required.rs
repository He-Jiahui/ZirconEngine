pub(super) fn take_required_dependency_required(value: &mut Option<bool>) -> bool {
    take_required_dependency_field(value, "sound dependency should declare required")
}

fn take_required_dependency_field<T>(value: &mut Option<T>, missing_message: &'static str) -> T {
    value.take().expect(missing_message)
}

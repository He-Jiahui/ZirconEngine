pub(super) fn set_dependency_capability(capability: &mut Option<String>, value: String) {
    *capability = Some(value);
}

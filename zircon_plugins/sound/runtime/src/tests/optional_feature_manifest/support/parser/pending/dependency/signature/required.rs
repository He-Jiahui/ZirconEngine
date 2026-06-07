pub(super) fn take_required_capability(capability: &mut Option<String>) -> String {
    capability
        .take()
        .expect("optional feature dependency should declare capability")
}

pub(super) fn take_required_primary(primary: &mut Option<bool>) -> bool {
    primary
        .take()
        .expect("optional feature dependency should declare primary")
}

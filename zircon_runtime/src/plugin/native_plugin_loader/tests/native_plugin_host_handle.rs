use super::NativePluginHostHandle;

#[test]
fn weak_host_handle_does_not_extend_backend_lifetime() {
    let host = NativePluginHostHandle::default();
    let weak = host.downgrade();

    assert!(weak.upgrade().is_some());
    drop(host);

    assert!(weak.upgrade().is_none());
}

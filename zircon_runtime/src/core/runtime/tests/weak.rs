#[test]
fn core_weak_upgrade_uses_direct_upgrade_branch() {
    let weak_source = include_str!("../weak.rs");

    assert!(weak_source.contains("let Some(inner) = self.inner.upgrade() else"));
    assert!(weak_source.contains("return None;"));
    assert!(weak_source.contains("Some(CoreHandle { inner })"));
    assert!(!weak_source.contains(".upgrade().map(|inner| CoreHandle { inner })"));
}

#[test]
fn compute_texture_view_uses_compiled_access_views_without_legacy_fallback() {
    let source = include_str!("../texture_view.rs");
    let view_resolver = source
        .splitn(2, "pub(super) fn resolve_compute_texture_view(")
        .nth(1)
        .expect("compute texture view resolver source");
    let view_resolver = view_resolver
        .splitn(2, "pub(super) fn resolve_compute_texture_desc(")
        .next()
        .expect("compute texture view resolver body");

    assert!(view_resolver.contains("transient_texture_view_for_access"));
    assert!(view_resolver.contains("external_texture_view_for_access"));
    assert!(!view_resolver.contains("texture_view_with_full_mip_fallback"));
    assert!(!view_resolver.contains("require_owned_texture_mip_view"));
}

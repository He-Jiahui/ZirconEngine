use super::*;

#[test]
fn host_invalidation_paint_only_does_not_require_slow_path() {
    let mask = HostInvalidationMask::PAINT_ONLY
        .union(HostInvalidationMask::POINTER_HOVER)
        .union(HostInvalidationMask::VIEWPORT_IMAGE);
    let mut root = HostInvalidationRoot::default();
    root.invalidate(mask);

    assert!(!mask.requires_host_recompute());
    assert!(root.take_recompute_reasons().is_empty());
    assert_eq!(root.paint_only_requests, 1);
}

#[test]
fn host_invalidation_layout_implies_presentation_slow_path() {
    let mask = HostInvalidationMask::LAYOUT;

    assert!(mask.requires_layout());
    assert!(mask.requires_presentation());
    assert!(mask.requires_host_recompute());
}

#[test]
fn host_invalidation_render_is_separate_from_presentation() {
    let mask = HostInvalidationMask::RENDER;

    assert!(mask.requires_render());
    assert!(!mask.requires_layout());
    assert!(!mask.requires_presentation());
    assert!(mask.requires_host_recompute());
}

#[test]
fn host_invalidation_counts_and_drains_recompute_reasons() {
    let mut root = HostInvalidationRoot::default();
    root.invalidate(HostInvalidationMask::LAYOUT);
    root.invalidate(HostInvalidationMask::RENDER);

    let render = root.consume_recompute_reasons(HostInvalidationMask::RENDER);
    assert_eq!(render, HostInvalidationMask::RENDER);

    let remaining = root.take_recompute_reasons();
    assert!(remaining.contains(HostInvalidationMask::LAYOUT));
    assert!(!remaining.contains(HostInvalidationMask::RENDER));
    assert_eq!(root.total_requests, 2);
    assert_eq!(root.layout_requests, 1);
    assert_eq!(root.render_requests, 1);
}

#[test]
fn host_invalidation_diagnostics_snapshot_exposes_paint_only_count() {
    let mut root = HostInvalidationRoot::default();
    root.invalidate(HostInvalidationMask::VIEWPORT_IMAGE.union(HostInvalidationMask::PAINT_ONLY));
    root.record_slow_path_rebuild();
    root.record_render_rebuild();

    let diagnostics = root.diagnostics_snapshot();

    assert_eq!(diagnostics.slow_path_rebuild_count, 1);
    assert_eq!(diagnostics.render_rebuild_count, 1);
    assert_eq!(diagnostics.paint_only_request_count, 1);
}

#[test]
fn host_invalidation_paint_only_does_not_request_layout_or_presentation_recompute() {
    let mut root = HostInvalidationRoot::default();
    root.invalidate(
        HostInvalidationMask::PAINT_ONLY
            .union(HostInvalidationMask::POINTER_HOVER)
            .union(HostInvalidationMask::VIEWPORT_IMAGE),
    );

    let diagnostics = root.diagnostics_snapshot();
    assert_eq!(diagnostics.paint_only_request_count, 1);
    assert_eq!(root.layout_requests, 0);
    assert_eq!(root.presentation_requests, 0);
    assert_eq!(root.render_requests, 0);
    assert!(root.take_recompute_reasons().is_empty());
}

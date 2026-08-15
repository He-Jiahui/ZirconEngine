use super::*;
use crate::ui::retained_host::HostShellContentScope;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::view::ViewInstanceId;

#[test]
fn host_invalidation_coalesces_recompute_reasons_per_view_scope() {
    let view = ViewInstanceId::new("ui-asset-editor#main");
    let mut root = HostInvalidationRoot::default();
    root.invalidate_view(view.clone(), HostInvalidationMask::PRESENTATION_DATA);
    root.invalidate_view(view.clone(), HostInvalidationMask::HIT_TEST);

    let transaction = root.take_recompute_transaction();

    assert_eq!(transaction.scope_count(), 1);
    assert_eq!(
        transaction.reasons_for(&HostInvalidationScope::View(view)),
        Some(HostInvalidationMask::PRESENTATION_DATA.union(HostInvalidationMask::HIT_TEST))
    );
}

#[test]
fn host_invalidation_paint_only_does_not_require_slow_path() {
    let mask = HostInvalidationMask::PAINT_ONLY
        .union(HostInvalidationMask::POINTER_HOVER)
        .union(HostInvalidationMask::VIEWPORT_IMAGE);
    let mut root = HostInvalidationRoot::default();
    root.invalidate(mask);

    assert!(!mask.requires_host_recompute());
    assert!(root.take_recompute_transaction().reasons().is_empty());
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
fn shell_content_requires_presentation_without_requiring_layout() {
    let mask = HostInvalidationMask::SHELL_CONTENT;

    assert!(!mask.requires_layout());
    assert!(mask.requires_presentation());
    assert!(mask.requires_host_recompute());
    assert_eq!(mask.summary(), "shell-content");
}

#[test]
fn host_invalidation_render_is_separate_from_presentation() {
    let mask = HostInvalidationMask::RENDER;

    assert!(mask.requires_render());
    assert!(!mask.requires_layout());
    assert!(!mask.requires_presentation());
    assert!(!mask.requires_host_recompute());
}

#[test]
fn host_invalidation_counts_and_drains_recompute_reasons() {
    let mut root = HostInvalidationRoot::default();
    root.invalidate(HostInvalidationMask::LAYOUT);
    root.invalidate(HostInvalidationMask::RENDER);

    let render = root.consume_recompute_reasons(HostInvalidationMask::RENDER);
    assert_eq!(render, HostInvalidationMask::NONE);

    let remaining = root.take_recompute_transaction().reasons();
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
    assert!(root.take_recompute_transaction().reasons().is_empty());
}

#[test]
fn host_invalidation_default_scope_remains_global() {
    let mut root = HostInvalidationRoot::default();
    root.invalidate(HostInvalidationMask::PRESENTATION_DATA);

    let transaction = root.take_recompute_transaction();

    assert_eq!(transaction.scope_count(), 1);
    assert_eq!(
        transaction.reasons_for(&HostInvalidationScope::All),
        Some(HostInvalidationMask::PRESENTATION_DATA)
    );
}

#[test]
fn host_invalidation_exposes_view_only_presentation_scopes() {
    let first = ViewInstanceId::new("ui-asset-editor#first");
    let second = ViewInstanceId::new("ui-asset-editor#second");
    let mut root = HostInvalidationRoot::default();
    root.invalidate_view(first.clone(), HostInvalidationMask::PRESENTATION_DATA);
    root.invalidate_view(second.clone(), HostInvalidationMask::PRESENTATION_DATA);

    let transaction = root.take_recompute_transaction();

    assert_eq!(
        transaction.presentation_only_view_ids(),
        Some(vec![first, second])
    );
}

#[test]
fn host_invalidation_rejects_global_or_non_presentation_scoped_fast_paths() {
    let view = ViewInstanceId::new("ui-asset-editor#main");
    let mut root = HostInvalidationRoot::default();
    root.invalidate_view(
        view.clone(),
        HostInvalidationMask::PRESENTATION_DATA.union(HostInvalidationMask::HIT_TEST),
    );
    assert!(root.has_pending_presentation_recompute());
    assert_eq!(
        root.take_recompute_transaction()
            .presentation_only_view_ids(),
        None
    );

    root.invalidate(HostInvalidationMask::PRESENTATION_DATA);
    assert_eq!(
        root.take_recompute_transaction()
            .presentation_only_view_ids(),
        None
    );
}

#[test]
fn shell_content_scope_coalesces_reasons_without_losing_the_target() {
    let scope = HostShellContentScope::new(
        ActivityDrawerSlot::LeftBottom,
        ViewInstanceId::new("editor.module_plugins#main"),
    );
    let mut root = HostInvalidationRoot::default();
    root.invalidate_shell_content(scope.clone(), HostInvalidationMask::SHELL_CONTENT);
    root.invalidate_shell_content(scope.clone(), HostInvalidationMask::PRESENTATION_DATA);

    let transaction = root.take_recompute_transaction();

    assert_eq!(transaction.scope_count(), 1);
    assert_eq!(transaction.shell_content_scope(), Some(scope));
}

#[test]
fn multiple_shell_content_targets_reject_the_single_dock_fast_path() {
    let mut root = HostInvalidationRoot::default();
    root.invalidate_shell_content(
        HostShellContentScope::new(
            ActivityDrawerSlot::LeftTop,
            ViewInstanceId::new("editor.hierarchy#main"),
        ),
        HostInvalidationMask::SHELL_CONTENT,
    );
    root.invalidate_shell_content(
        HostShellContentScope::new(
            ActivityDrawerSlot::RightTop,
            ViewInstanceId::new("editor.inspector#main"),
        ),
        HostInvalidationMask::SHELL_CONTENT,
    );

    let transaction = root.take_recompute_transaction();

    assert_eq!(transaction.scope_count(), 2);
    assert_eq!(transaction.shell_content_scope(), None);
}

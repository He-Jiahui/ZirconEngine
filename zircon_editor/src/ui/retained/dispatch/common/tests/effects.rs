use super::merge_effects;
use crate::ui::retained_host::event_bridge::{HostShellContentScope, UiHostEventEffects};
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::view::ViewInstanceId;

#[test]
fn merging_opened_view_visibility_keeps_each_actual_instance_request() {
    let mut target = UiHostEventEffects::default();
    target
        .opened_view_visibility_requests
        .push(ViewInstanceId::new("editor.inspector#1"));
    let mut source = UiHostEventEffects::default();
    source
        .opened_view_visibility_requests
        .push(ViewInstanceId::new("editor.hierarchy#1"));
    merge_effects(&mut target, source);
    assert_eq!(
        target.opened_view_visibility_requests,
        vec![
            ViewInstanceId::new("editor.inspector#1"),
            ViewInstanceId::new("editor.hierarchy#1"),
        ]
    );
    assert!(!target.is_viewport_resize_recompute_compatible());
}

#[test]
fn merging_distinct_shell_content_scopes_disables_the_single_target_scope() {
    let mut first = UiHostEventEffects::default();
    first.reuse_layout_for_shell_content(HostShellContentScope::new(
        ActivityDrawerSlot::LeftTop,
        ViewInstanceId::new("editor.hierarchy#main"),
    ));
    let mut second = UiHostEventEffects::default();
    second.reuse_layout_for_shell_content(HostShellContentScope::new(
        ActivityDrawerSlot::RightTop,
        ViewInstanceId::new("editor.inspector#main"),
    ));

    merge_effects(&mut first, second);

    assert_eq!(first.shell_content_scope(), None);
}

#[test]
fn merging_unscoped_presentation_disables_the_shell_content_scope() {
    let mut scoped = UiHostEventEffects::default();
    scoped.reuse_layout_for_shell_content(HostShellContentScope::new(
        ActivityDrawerSlot::LeftTop,
        ViewInstanceId::new("editor.hierarchy#main"),
    ));
    let mut global = UiHostEventEffects::default();
    global.request_presentation();

    merge_effects(&mut scoped, global);

    assert_eq!(scoped.shell_content_scope(), None);
}

#[test]
fn merging_paint_only_preserves_the_shell_content_scope() {
    let scope = HostShellContentScope::new(
        ActivityDrawerSlot::LeftTop,
        ViewInstanceId::new("editor.hierarchy#main"),
    );
    let mut scoped = UiHostEventEffects::default();
    scoped.reuse_layout_for_shell_content(scope.clone());
    let mut paint_only = UiHostEventEffects::default();
    paint_only.request_paint_only();

    merge_effects(&mut scoped, paint_only);

    assert_eq!(scoped.shell_content_scope(), Some(scope));
}

use std::collections::BTreeMap;

use super::{active_shell_content_scope_for_pane, validate_shell_content_layout_transition};
use crate::ui::retained_host::HostShellContentScope;
use crate::ui::workbench::layout::{
    ActivityDrawerMode, ActivityDrawerSlot, LayoutCommand, LayoutManager, WorkbenchLayout,
};
use crate::ui::workbench::model::ToolWindowStackModel;
use crate::ui::workbench::view::ViewInstanceId;

fn tool_window_stack(
    slot: ActivityDrawerSlot,
    active_tab: Option<ViewInstanceId>,
    visible: bool,
) -> ToolWindowStackModel {
    ToolWindowStackModel {
        slot,
        mode: ActivityDrawerMode::Pinned,
        visible,
        tabs: Vec::new(),
        active_tab,
    }
}

fn drawer_switch_fixture() -> (WorkbenchLayout, HostShellContentScope) {
    let mut layout = WorkbenchLayout::default();
    let first = ViewInstanceId::new("editor.hierarchy#committed");
    let second = ViewInstanceId::new("editor.project#committed");
    let drawer = layout
        .active_activity_window_mut()
        .expect("active activity window")
        .activity_drawers
        .get_mut(&ActivityDrawerSlot::LeftTop)
        .expect("left drawer");
    drawer.tab_stack.tabs = vec![first.clone(), second.clone()];
    drawer.tab_stack.active_tab = Some(first.clone());
    drawer.active_view = Some(first);
    drawer.mode = ActivityDrawerMode::Pinned;
    (
        layout,
        HostShellContentScope::new(ActivityDrawerSlot::LeftTop, second),
    )
}

fn switched_layout(previous: &WorkbenchLayout, scope: &HostShellContentScope) -> WorkbenchLayout {
    let mut next = previous.clone();
    LayoutManager::default()
        .apply(
            &mut next,
            LayoutCommand::ActivateDrawerTab {
                slot: scope.slot,
                instance_id: scope.instance_id.clone(),
            },
        )
        .expect("switch drawer tab");
    next
}

#[test]
fn accepts_an_exact_tab_switch_inside_the_committed_region() {
    let (previous, scope) = drawer_switch_fixture();
    let next = switched_layout(&previous, &scope);

    assert!(validate_shell_content_layout_transition(
        &previous, &next, &scope
    ));
}

#[test]
fn accepts_content_only_update_for_the_current_active_drawer_pane() {
    let (layout, _) = drawer_switch_fixture();
    let scope = HostShellContentScope::new(
        ActivityDrawerSlot::LeftTop,
        ViewInstanceId::new("editor.hierarchy#committed"),
    );

    assert!(validate_shell_content_layout_transition(
        &layout, &layout, &scope
    ));
}

#[test]
fn rejects_a_region_switch_when_the_mounted_extent_changes() {
    let (previous, scope) = drawer_switch_fixture();
    let mut next = switched_layout(&previous, &scope);
    next.active_activity_window_mut()
        .expect("active activity window")
        .activity_drawers
        .get_mut(&ActivityDrawerSlot::LeftTop)
        .expect("left drawer")
        .extent += 24.0;

    assert!(!validate_shell_content_layout_transition(
        &previous, &next, &scope
    ));
}

#[test]
fn rejects_any_non_target_layout_change_in_the_same_transaction() {
    let (previous, scope) = drawer_switch_fixture();
    let mut next = switched_layout(&previous, &scope);
    next.active_main_page = crate::ui::workbench::layout::MainPageId::new("unexpected");

    assert!(!validate_shell_content_layout_transition(
        &previous, &next, &scope
    ));
}

#[test]
fn active_visible_drawer_pane_resolves_to_shell_content_scope() {
    let instance_id = ViewInstanceId::new("plugin.rows#active");
    let tool_windows = BTreeMap::from([(
        ActivityDrawerSlot::RightTop,
        tool_window_stack(
            ActivityDrawerSlot::RightTop,
            Some(instance_id.clone()),
            true,
        ),
    )]);

    assert_eq!(
        active_shell_content_scope_for_pane(&tool_windows, &instance_id.0),
        Some(HostShellContentScope::new(
            ActivityDrawerSlot::RightTop,
            instance_id
        ))
    );
}

#[test]
fn inactive_or_hidden_drawer_pane_rejects_shell_content_scope() {
    let target = ViewInstanceId::new("plugin.rows#target");
    let other = ViewInstanceId::new("plugin.rows#other");
    let inactive = BTreeMap::from([(
        ActivityDrawerSlot::RightTop,
        tool_window_stack(ActivityDrawerSlot::RightTop, Some(other), true),
    )]);
    let hidden = BTreeMap::from([(
        ActivityDrawerSlot::RightTop,
        tool_window_stack(ActivityDrawerSlot::RightTop, Some(target.clone()), false),
    )]);

    assert_eq!(
        active_shell_content_scope_for_pane(&inactive, &target.0),
        None
    );
    assert_eq!(
        active_shell_content_scope_for_pane(&hidden, &target.0),
        None
    );
}

#[test]
fn duplicate_active_pane_identity_rejects_ambiguous_shell_content_scope() {
    let target = ViewInstanceId::new("plugin.rows#duplicate");
    let tool_windows = BTreeMap::from([
        (
            ActivityDrawerSlot::LeftTop,
            tool_window_stack(ActivityDrawerSlot::LeftTop, Some(target.clone()), true),
        ),
        (
            ActivityDrawerSlot::RightTop,
            tool_window_stack(ActivityDrawerSlot::RightTop, Some(target.clone()), true),
        ),
    ]);

    assert_eq!(
        active_shell_content_scope_for_pane(&tool_windows, &target.0),
        None
    );
}

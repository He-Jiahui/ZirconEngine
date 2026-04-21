use super::super::support::*;
use crate::core::editor_event::{
    ActivityDrawerMode as EventActivityDrawerMode, ActivityDrawerSlot as EventActivityDrawerSlot,
    LayoutCommand as EventLayoutCommand, MainPageId as EventMainPageId,
    SplitAxis as EventSplitAxis, SplitPlacement as EventSplitPlacement, ViewHost as EventViewHost,
    ViewInstanceId as EventViewInstanceId, WorkspaceTarget as EventWorkspaceTarget,
};

#[test]
fn tab_drop_dispatch_attaches_view_and_reopens_target_drawer_from_normalized_route() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_tab_drop_dispatch");
    dispatch_layout_command(
        &harness.runtime,
        LayoutCommand::SetDrawerMode {
            slot: ActivityDrawerSlot::RightTop,
            mode: ActivityDrawerMode::Collapsed,
        },
    )
    .unwrap();

    let effects = dispatch_tab_drop(
        &harness.runtime,
        "editor.project#1",
        &ResolvedWorkbenchTabDropRoute {
            target_group: WorkbenchDragTargetGroup::Right,
            target_label: "right tool stack",
            target: ResolvedWorkbenchTabDropTarget::Attach(ResolvedTabDrop {
                host: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
                anchor: None,
            }),
        },
    )
    .unwrap();

    let journal = harness.runtime.journal();
    let events: Vec<_> = journal
        .records()
        .iter()
        .rev()
        .take(2)
        .map(|record| record.event.clone())
        .collect();
    assert_eq!(
        events.into_iter().rev().collect::<Vec<_>>(),
        vec![
            EditorEvent::Layout(EventLayoutCommand::AttachView {
                instance_id: EventViewInstanceId::new("editor.project#1"),
                target: EventViewHost::Drawer(EventActivityDrawerSlot::RightTop),
                anchor: None,
            }),
            EditorEvent::Layout(EventLayoutCommand::SetDrawerMode {
                slot: EventActivityDrawerSlot::RightTop,
                mode: EventActivityDrawerMode::Pinned,
            }),
        ]
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn tab_drop_dispatch_does_not_reopen_drawer_when_target_is_already_visible() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_tab_drop_attach_visible");

    let baseline_records = harness.runtime.journal().records().len();
    let effects = dispatch_tab_drop(
        &harness.runtime,
        "editor.project#1",
        &ResolvedWorkbenchTabDropRoute {
            target_group: WorkbenchDragTargetGroup::Right,
            target_label: "right tool stack",
            target: ResolvedWorkbenchTabDropTarget::Attach(ResolvedTabDrop {
                host: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
                anchor: None,
            }),
        },
    )
    .unwrap();

    let journal = harness.runtime.journal();
    let new_records = &journal.records()[baseline_records..];
    assert_eq!(new_records.len(), 1);
    assert_eq!(
        new_records[0].event,
        EditorEvent::Layout(EventLayoutCommand::AttachView {
            instance_id: EventViewInstanceId::new("editor.project#1"),
            target: EventViewHost::Drawer(EventActivityDrawerSlot::RightTop),
            anchor: None,
        })
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn tab_drop_dispatch_preserves_auto_hide_drawer_mode() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_tab_drop_attach_autohide");
    dispatch_layout_command(
        &harness.runtime,
        LayoutCommand::SetDrawerMode {
            slot: ActivityDrawerSlot::RightTop,
            mode: ActivityDrawerMode::AutoHide,
        },
    )
    .unwrap();

    let baseline_records = harness.runtime.journal().records().len();
    let effects = dispatch_tab_drop(
        &harness.runtime,
        "editor.project#1",
        &ResolvedWorkbenchTabDropRoute {
            target_group: WorkbenchDragTargetGroup::Right,
            target_label: "right tool stack",
            target: ResolvedWorkbenchTabDropTarget::Attach(ResolvedTabDrop {
                host: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
                anchor: None,
            }),
        },
    )
    .unwrap();

    let journal = harness.runtime.journal();
    let new_records = &journal.records()[baseline_records..];
    assert_eq!(new_records.len(), 1);
    assert_eq!(
        new_records[0].event,
        EditorEvent::Layout(EventLayoutCommand::AttachView {
            instance_id: EventViewInstanceId::new("editor.project#1"),
            target: EventViewHost::Drawer(EventActivityDrawerSlot::RightTop),
            anchor: None,
        })
    );
    assert_eq!(
        harness
            .runtime
            .current_layout()
            .drawers
            .get(&ActivityDrawerSlot::RightTop)
            .map(|drawer| drawer.mode),
        Some(ActivityDrawerMode::AutoHide)
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn tab_drop_dispatch_creates_split_for_document_edge_route() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_tab_drop_split_dispatch");

    let effects = dispatch_tab_drop(
        &harness.runtime,
        "editor.project#1",
        &ResolvedWorkbenchTabDropRoute {
            target_group: WorkbenchDragTargetGroup::Document,
            target_label: "Split Document Left",
            target: ResolvedWorkbenchTabDropTarget::Split {
                workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
                path: vec![],
                axis: SplitAxis::Horizontal,
                placement: SplitPlacement::Before,
            },
        },
    )
    .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Layout(EventLayoutCommand::CreateSplit {
            workspace: EventWorkspaceTarget::MainPage(EventMainPageId::workbench()),
            path: vec![],
            axis: EventSplitAxis::Horizontal,
            placement: EventSplitPlacement::Before,
            new_instance: EventViewInstanceId::new("editor.project#1"),
        })
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}

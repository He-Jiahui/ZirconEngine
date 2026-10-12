use std::collections::BTreeMap;

use super::{
    runtime_diagnostics_refresh_target_for_drawers, runtime_diagnostics_refresh_target_for_parts,
    RuntimeDiagnosticsRefreshTarget,
};
use crate::ui::retained_host::HostShellContentScope;
use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::layout::{
    ActivityDrawerMode, ActivityDrawerSlot, MainPageId, WorkspaceTarget,
};
use crate::ui::workbench::model::{
    DocumentTabModel, FloatingWindowModel, PaneTabModel, ToolWindowStackModel,
};
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};

fn diagnostic_stack(
    slot: ActivityDrawerSlot,
    instance_id: &str,
    content_kind: ViewContentKind,
) -> ToolWindowStackModel {
    let instance_id = ViewInstanceId::new(instance_id);
    ToolWindowStackModel {
        slot,
        mode: ActivityDrawerMode::Pinned,
        visible: true,
        tabs: vec![PaneTabModel {
            instance_id: instance_id.clone(),
            descriptor_id: ViewDescriptorId::new("diagnostics"),
            title: "Diagnostics".to_owned(),
            icon_key: String::new(),
            content_kind,
            active: true,
            closeable: true,
            empty_state: None,
        }],
        active_tab: Some(instance_id),
    }
}

fn diagnostic_document(instance_id: &str) -> DocumentTabModel {
    DocumentTabModel {
        workspace: WorkspaceTarget::MainPage(MainPageId::new("main")),
        workspace_path: Vec::new(),
        instance_id: ViewInstanceId::new(instance_id),
        descriptor_id: ViewDescriptorId::new("diagnostics"),
        title: "Diagnostics".to_owned(),
        icon_key: String::new(),
        content_kind: ViewContentKind::RuntimeDiagnostics,
        active: true,
        closeable: true,
        empty_state: None,
    }
}

#[test]
fn unique_active_diagnostic_drawer_uses_shell_content_refresh() {
    let instance_id = ViewInstanceId::new("runtime-diagnostics#drawer");
    let tool_windows = BTreeMap::from([(
        ActivityDrawerSlot::RightTop,
        diagnostic_stack(
            ActivityDrawerSlot::RightTop,
            &instance_id.0,
            ViewContentKind::RuntimeDiagnostics,
        ),
    )]);

    assert_eq!(
        runtime_diagnostics_refresh_target_for_drawers(false, &tool_windows),
        RuntimeDiagnosticsRefreshTarget::ShellContent(HostShellContentScope::new(
            ActivityDrawerSlot::RightTop,
            instance_id,
        ))
    );
}

#[test]
fn performance_timeline_drawer_uses_the_same_local_refresh_contract() {
    let instance_id = ViewInstanceId::new("performance-timeline#drawer");
    let tool_windows = BTreeMap::from([(
        ActivityDrawerSlot::Bottom,
        diagnostic_stack(
            ActivityDrawerSlot::Bottom,
            &instance_id.0,
            ViewContentKind::PerformanceTimeline,
        ),
    )]);

    assert_eq!(
        runtime_diagnostics_refresh_target_for_drawers(false, &tool_windows),
        RuntimeDiagnosticsRefreshTarget::ShellContent(HostShellContentScope::new(
            ActivityDrawerSlot::Bottom,
            instance_id,
        ))
    );
}

#[test]
fn document_or_floating_diagnostics_require_full_presentation() {
    let tool_windows = BTreeMap::new();
    let document = diagnostic_document("runtime-diagnostics#document");

    assert_eq!(
        runtime_diagnostics_refresh_target_for_parts(
            std::slice::from_ref(&document),
            &[],
            &tool_windows,
        ),
        RuntimeDiagnosticsRefreshTarget::FullPresentation
    );

    let floating = FloatingWindowModel {
        window_id: MainPageId::new("floating"),
        title: "Diagnostics".to_owned(),
        requested_frame: ShellFrame::default(),
        focused_view: Some(document.instance_id.clone()),
        tabs: vec![document],
    };
    assert_eq!(
        runtime_diagnostics_refresh_target_for_parts(
            &[],
            std::slice::from_ref(&floating),
            &tool_windows,
        ),
        RuntimeDiagnosticsRefreshTarget::FullPresentation
    );
}

#[test]
fn multiple_or_inconsistent_drawer_targets_require_full_presentation() {
    let mut inconsistent = diagnostic_stack(
        ActivityDrawerSlot::LeftTop,
        "runtime-diagnostics#inconsistent",
        ViewContentKind::RuntimeDiagnostics,
    );
    inconsistent.active_tab = Some(ViewInstanceId::new("another-tab"));
    assert_eq!(
        runtime_diagnostics_refresh_target_for_drawers(
            false,
            &BTreeMap::from([(ActivityDrawerSlot::LeftTop, inconsistent)]),
        ),
        RuntimeDiagnosticsRefreshTarget::FullPresentation
    );

    let multiple = BTreeMap::from([
        (
            ActivityDrawerSlot::LeftTop,
            diagnostic_stack(
                ActivityDrawerSlot::LeftTop,
                "runtime-diagnostics#left",
                ViewContentKind::RuntimeDiagnostics,
            ),
        ),
        (
            ActivityDrawerSlot::RightTop,
            diagnostic_stack(
                ActivityDrawerSlot::RightTop,
                "performance-timeline#right",
                ViewContentKind::PerformanceTimeline,
            ),
        ),
    ]);
    assert_eq!(
        runtime_diagnostics_refresh_target_for_drawers(false, &multiple),
        RuntimeDiagnosticsRefreshTarget::FullPresentation
    );
}

#[test]
fn hidden_or_absent_diagnostics_do_not_schedule_presentation() {
    let mut hidden = diagnostic_stack(
        ActivityDrawerSlot::RightBottom,
        "runtime-diagnostics#hidden",
        ViewContentKind::RuntimeDiagnostics,
    );
    hidden.visible = false;

    assert_eq!(
        runtime_diagnostics_refresh_target_for_drawers(
            false,
            &BTreeMap::from([(ActivityDrawerSlot::RightBottom, hidden)]),
        ),
        RuntimeDiagnosticsRefreshTarget::None
    );
    assert_eq!(
        runtime_diagnostics_refresh_target_for_drawers(false, &BTreeMap::new()),
        RuntimeDiagnosticsRefreshTarget::None
    );
}

#[test]
fn pending_refresh_keeps_diagnostics_payload_collection_enabled() {
    assert!(!RuntimeDiagnosticsRefreshTarget::None.should_collect_payload());
    assert!(RuntimeDiagnosticsRefreshTarget::Pending.should_collect_payload());
    assert!(RuntimeDiagnosticsRefreshTarget::FullPresentation.should_collect_payload());
}

use super::*;

#[test]
fn inline_rename_focus_only_replaces_the_matching_row_text() {
    let focus = HostTextInputFocusData {
        control_id: HIERARCHY_INLINE_RENAME_CONTROL_ID.into(),
        dispatch_kind: "hierarchy_inline_rename:7".into(),
        value_text: "Renamed".into(),
        ..HostTextInputFocusData::default()
    };
    let selected = SceneNodeData {
        id: "7".into(),
        selected: true,
        ..SceneNodeData::default()
    };
    let other_selected = SceneNodeData {
        id: "8".into(),
        selected: true,
        ..SceneNodeData::default()
    };

    assert_eq!(
        inline_hierarchy_rename_value(&selected, Some(&focus)),
        Some("Renamed")
    );
    assert_eq!(
        inline_hierarchy_rename_value(&other_selected, Some(&focus)),
        None
    );
}

#[test]
fn visible_row_range_is_bounded_by_the_clipped_viewport() {
    let viewport = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 240.0,
        height: 100.0,
    };
    let metrics = hierarchy_row_metrics_from_host_metrics(
        crate::ui::retained_host::host_contract::paint_theme::METRICS,
    );
    let range = visible_hierarchy_row_range(&viewport, &viewport, 560.0, 10_000, metrics);
    let maximum_rows =
        (viewport.height / (metrics.row_height + metrics.row_gap)).ceil() as usize + 2;

    assert!(range.start > 0);
    assert!(range.end < 10_000);
    assert!(range.len() <= maximum_rows);
}

#[test]
fn visible_row_range_is_empty_for_an_empty_clip() {
    let viewport = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 240.0,
        height: 100.0,
    };
    let empty_clip = FrameRect {
        height: 0.0,
        ..viewport.clone()
    };

    assert!(visible_hierarchy_row_range(
        &viewport,
        &empty_clip,
        0.0,
        10_000,
        hierarchy_row_metrics_from_host_metrics(
            crate::ui::retained_host::host_contract::paint_theme::METRICS,
        ),
    )
    .is_empty());
}

#[test]
fn actual_native_hierarchy_pointer_owner_controls_selection_pixels_until_focus_leaves() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    use crate::ui::retained_host::host_contract::native_pointer::{
        routing::{PanePointerRoute, PanePointerTarget},
        NativePointerButtonState,
    };
    use zircon_runtime_interface::ui::{dispatch::UiInputModifiers, surface::UiPointerButton};
    let ui = crate::ui::retained_host::UiHostWindow::new().unwrap();
    let viewport = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 240.0,
        height: 90.0,
    };
    let presentation =
        crate::ui::retained_host::host_contract::data::HostWindowPresentationData::default();
    let metrics = hierarchy_row_metrics_from_host_metrics(
        crate::ui::retained_host::host_contract::paint_theme::METRICS,
    );
    let selected = SceneNodeData {
        id: "7".into(),
        name: "Node".into(),
        selected: true,
        ..Default::default()
    };
    let paint = || {
        let mut frame = HostRgbaFrame::filled(240, 90, [0, 0, 0, 255]);
        let interaction = ui.get_pane_interaction_state();
        draw_hierarchy_row(
            &mut frame,
            &viewport,
            &viewport,
            0,
            0.0,
            metrics,
            &selected,
            &interaction,
            None,
        );
        let rect = hierarchy_row_frame(&viewport, 0, 0.0, metrics);
        let x = (rect.right() - 4.0) as usize;
        let y = (rect.y + rect.height / 2.0) as usize;
        let index = (y * 240 + x) * 4;
        frame.bytes[index..index + 4].to_vec()
    };
    let dispatch = |target| {
        crate::ui::retained_host::host_contract::native_pointer::dispatch_pane_button(
            &ui,
            &presentation,
            PanePointerRoute::new(target, &viewport, 10.0, 10.0),
            NativePointerButtonState::Pressed,
            UiPointerButton::Primary,
            0,
            UiInputModifiers::default(),
            None,
        )
    };
    assert_eq!(paint(), [64, 87, 111, 255]);
    dispatch(PanePointerTarget::Hierarchy);
    assert_eq!(
        ui.get_pane_interaction_state().focused_template_control_id,
        "HierarchyListPanel"
    );
    assert_eq!(paint(), [0, 112, 224, 255]);
    dispatch(PanePointerTarget::Inspector);
    assert!(selected.selected);
    assert_eq!(paint(), [64, 87, 111, 255]);
    dispatch(PanePointerTarget::Hierarchy);
    ui.dispatch_native_focus_lost_event_for_test();
    assert_eq!(paint(), [64, 87, 111, 255]);
}

#[test]
fn actual_native_draw_receipt_keeps_data_identity_global_frames_and_visible_commands() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let scope=crate::ui::retained_host::host_contract::paint_template_nodes::PaintEvidenceScope::begin_for_normal_present(&root).unwrap();
    let viewport = FrameRect {
        x: 140.0,
        y: 90.0,
        width: 240.0,
        height: 80.0,
    };
    let metrics = hierarchy_row_metrics_from_host_metrics(
        crate::ui::retained_host::host_contract::paint_theme::METRICS,
    );
    let data = SceneNodeData {
        id: "runtime/17".into(),
        name: "Actual row".into(),
        depth: 1,
        ..Default::default()
    };
    let mut frame = HostRgbaFrame::recording_only(500, 360);
    draw_hierarchy_row(
        &mut frame,
        &viewport,
        &viewport,
        1,
        0.0,
        metrics,
        &data,
        &Default::default(),
        None,
    );
    draw_hierarchy_row(
        &mut frame,
        &viewport,
        &viewport,
        100,
        0.0,
        metrics,
        &data,
        &Default::default(),
        None,
    );
    let commands = frame.into_recorded_commands();
    let receipt = scope.finish().unwrap();
    let rows = receipt["nativeHierarchyDraws"].as_array().unwrap();
    assert_eq!(
        rows.len(),
        1,
        "clipped rows are never manufactured from a prototype"
    );
    let row = &rows[0];
    assert_eq!(row["dataIdentity"]["nodeId"], data.id.as_str());
    let actual = hierarchy_row_frame(&viewport, 1, 0.0, metrics);
    assert_eq!(row["bounds"]["x"].as_f64(), Some(f64::from(actual.x)));
    assert_eq!(row["bounds"]["y"].as_f64(), Some(f64::from(actual.y)));
    assert_eq!(row["clip"]["x"], 140.0);
    assert_eq!(row["commands"].as_array().unwrap().len(), commands.len());
    assert!(commands.iter().any(|command|matches!(&command.kind,crate::ui::retained_host::host_contract::paint_frame::HostRecordedPaintKind::Text{text,..} if text==data.name.as_str())));
}

use super::*;
use std::sync::Arc;
use zircon_runtime_interface::ui::{
    dispatch::{
        UiImeInputEvent, UiImeInputEventKind, UiInputEvent, UiInputEventMetadata,
        UiKeyboardInputEvent, UiKeyboardInputState, UiPointerEvent, UiPointerInputEvent,
    },
    layout::UiPoint,
    surface::{UiPointerButton, UiPointerEventKind, UiSurfaceFrame},
    widget::UiWidgetEvent,
};

const RETAINED_SAMPLES: &[&str] = &[
    "WorkbenchRailButton",
    "WorkbenchIcon",
    "WorkbenchAlert",
    "WorkbenchCommandPalette",
    "WorkbenchConfirmDialog",
    "WorkbenchContextMenu",
    "WorkbenchDialog",
    "WorkbenchDragOverlay",
    "WorkbenchDropdownPopup",
    "WorkbenchNotificationCenter",
    "WorkbenchPopupMenu",
    "WorkbenchSkeleton",
    "WorkbenchToast",
    "WorkbenchTooltip",
    "WorkbenchNumberField",
    "WorkbenchRadio",
    "WorkbenchRangeSlider",
    "WorkbenchSegmentedControl",
    "WorkbenchSlider",
    "WorkbenchTabStrip",
    "WorkbenchToggle",
];

fn open_ui_asset() -> (ChildWindowHostHarness, ViewInstanceId) {
    let harness = ChildWindowHostHarness::new("ui_asset_builtin_palette");
    harness.activate_workbench_page();
    let instance = {
        let host = harness.host.borrow();
        let manager = host
            .editor_manager
            .enter()
            .expect("actual manager admission");
        manager
            .open_ui_asset_editor(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("assets/ui/editor/ui_asset_editor.zui"),
                None,
            )
            .expect("open current authored UI asset without source copies")
    };
    {
        let mut host = harness.host.borrow_mut();
        host.mark_layout_dirty();
        host.refresh_ui();
        host.recompute_if_dirty();
    }
    (harness, instance)
}

fn pane(
    h: &ChildWindowHostHarness,
    id: &ViewInstanceId,
) -> crate::ui::asset_editor::UiAssetEditorPanePresentation {
    let host = h.host.borrow();
    let manager = host.editor_manager.enter().expect("manager admission");
    manager
        .ui_asset_editor_pane_presentation(id)
        .expect("current session pane publication")
}

fn pointer(kind: UiPointerEventKind, point: UiPoint) -> UiPointerInputEvent {
    UiPointerInputEvent {
        metadata: UiInputEventMetadata::default(),
        event: UiPointerEvent::new(kind, point).with_button(UiPointerButton::Primary),
        precise_scroll: None,
    }
}

fn click_control(h: &ChildWindowHostHarness, id: &ViewInstanceId, control: &str) {
    let point = h
        .root_ui
        .ui_asset_palette_control_point_for_test(&id.0, control);
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Down, point));
    h.host.borrow_mut().recompute_if_dirty();
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Up, point));
    h.host.borrow_mut().recompute_if_dirty();
}

fn select(h: &ChildWindowHostHarness, id: &ViewInstanceId, component: &str) -> usize {
    let initial = pane(h, id);
    let index = initial
        .palette_items
        .iter()
        .position(|label| label == &format!("Builtin / {component}"))
        .expect("retained catalog sample must be a genuine palette choice");
    select_index(h, id, index);
    assert_eq!(pane(h, id).palette_selected_index, index as i32);
    index
}

fn select_index(h: &ChildWindowHostHarness, id: &ViewInstanceId, index: usize) {
    let count = pane(h, id).palette_items.len();
    for _ in 0..=count / 4 {
        let start = pane(h, id).palette_visible_start;
        if index >= start && index < start + 4 {
            break;
        }
        click_control(
            h,
            id,
            if index < start {
                "PalettePreviousPage"
            } else {
                "PaletteNextPage"
            },
        );
    }
    click_control(h, id, &format!("UiAssetPaletteEntry{index}"));
}

fn published(h: &ChildWindowHostHarness, id: &ViewInstanceId) -> Arc<UiSurfaceFrame> {
    let model = pane(h, id)
        .preview_surface_frame
        .expect("actual PreviewHost publication");
    let published = h.root_ui.ui_asset_preview_source_for_test(&id.0);
    assert!(
        Arc::ptr_eq(&model, &published),
        "the current Session frame is actually published to its existing canvas"
    );
    published
}

fn root_point(source: &UiSurfaceFrame, component: &str) -> UiPoint {
    let root = source
        .arranged_tree
        .nodes
        .iter()
        .find(|node| node.control_id.as_deref() == Some(format!("{component}Root").as_str()))
        .expect("authored component root is live");
    UiPoint::new(
        root.frame.x + root.frame.width * 0.5,
        root.frame.y + root.frame.height * 0.5,
    )
}

fn callback(
    h: &ChildWindowHostHarness,
    id: &ViewInstanceId,
    source: Arc<UiSurfaceFrame>,
    event: UiInputEvent,
) {
    pane_surface_host(&h.root_ui).invoke_retained_pane_input(
        id.0.as_str().into(),
        source,
        zircon_runtime_interface::ui::window::UiWindowInputPumpEvent::Input(event),
    );
}

fn apply_layout(h: &ChildWindowHostHarness, command: LayoutCommand) {
    let mut host = h.host.borrow_mut();
    let effects = callback_dispatch::dispatch_layout_command(&host.runtime, command)
        .expect("actual layout command admission");
    host.apply_dispatch_effects(effects);
    host.refresh_ui();
    host.recompute_if_dirty();
}

fn has_preview_composition(source: &UiSurfaceFrame) -> bool {
    source.render_extract.list.commands.iter().any(|command| {
        command
            .text_layout
            .as_ref()
            .and_then(|layout| layout.editable.as_ref())
            .is_some_and(|editable| editable.composition.is_some())
    })
}

impl UiHostWindow {
    /// Caller holds the existing environment lock while this real consumer executes.
    pub(crate) fn exercise_builtin_ui_asset_preview_consumers_for_test(
    ) -> Vec<zircon_runtime::ui::v2::UiV2SourceFileReceipt> {
        let (h, id) = open_ui_asset();
        let descriptors = crate::ui::template_runtime::builtin::builtin_component_descriptors()
            .expect("current approved catalog");
        let mut seen = std::collections::BTreeSet::new();
        let mut consumed_sources = Vec::new();
        let mut previous_source: Option<Arc<UiSurfaceFrame>> = None;
        for component in RETAINED_SAMPLES {
            select(&h, &id, component);
            let selected_source = published(&h, &id);
            if let Some(retired) = previous_source.take() {
                assert!(!Arc::ptr_eq(&retired, &selected_source));
                callback(
                    &h,
                    &id,
                    retired,
                    UiInputEvent::Pointer(pointer(
                        UiPointerEventKind::Move,
                        UiPoint::new(0.0, 0.0),
                    )),
                );
                assert!(
                    Arc::ptr_eq(&published(&h, &id), &selected_source),
                    "switching must reject retired input without replacing the new live source"
                );
                assert!(pane(&h, &id).preview_input_result.is_none());
            }
            let data = pane(&h, &id);
            let source = data.preview_surface_frame.as_ref().expect("live preview");
            let descriptor = descriptors
                .iter()
                .find(|descriptor| descriptor.component_id == *component)
                .expect("existing descriptor");
            assert_eq!(
                descriptor.tier,
                crate::ui::template::EditorComponentTier::Primitive
            );
            let physical = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("assets")
                .join(descriptor.document_id.strip_prefix("res://").unwrap());
            let original = zircon_runtime::ui::v2::UiV2PrototypeStoreFileCache::new()
                .load_store([physical.clone()])
                .expect("current original source closure");
            assert_eq!(data.preview_source_receipts, original.source_receipts);
            assert!(data
                .preview_source_receipts
                .iter()
                .any(|receipt| receipt.asset_id == descriptor.document_id
                    && receipt.physical_path
                        == physical.canonicalize().expect("physical builtin source")
                    && receipt.sha256.len() == 64));
            let records = h.root_ui.ui_asset_preview_paint_sources_for_test();
            assert!(
                records
                    .iter()
                    .any(|(frame, _, _)| Arc::ptr_eq(frame, source)),
                "{component} must reach the real window painter with its exact PreviewHost frame"
            );
            let point = root_point(source, component);
            let root = source
                .arranged_tree
                .nodes
                .iter()
                .find(|node| {
                    node.control_id.as_deref() == Some(format!("{component}Root").as_str())
                })
                .unwrap();
            let hit = zircon_runtime::ui::surface::hit_test_surface_frame(source, point);
            if root.supports_pointer() {
                assert_eq!(
                    hit.top_hit,
                    Some(root.node_id),
                    "{component} needs a genuine Runtime hit"
                );
            }
            let point = h.root_ui.ui_asset_preview_point_for_test(&id.0, point);
            h.root_ui
                .dispatch_ui_asset_normalized_pointer_for_test(pointer(
                    UiPointerEventKind::Move,
                    point,
                ));
            let after = pane(&h, &id);
            let result = after
                .preview_input_result
                .as_ref()
                .expect("Window input reached the actual session Surface");
            assert!(matches!(&result.event, UiInputEvent::Pointer(_)));
            if !root.supports_pointer() {
                assert!(
                    result.widget_events.is_empty(),
                    "passive samples cannot manufacture activation"
                );
            }
            assert!(!after.source_dirty);
            let receipt = original
                .source_receipts
                .iter()
                .find(|receipt| receipt.asset_id == descriptor.document_id)
                .unwrap();
            println!(
                "UI_ASSET_BUILTIN_CONSUMER {}",
                serde_json::json!({
                    "component": component, "documentId": descriptor.document_id,
                    "physicalSource": receipt.physical_path, "sourceSha256": receipt.sha256,
                    "sourceRoot": original.root_document.components[*component].root,
                    "treeId": source.tree_id, "frameGeneration": source.generation,
                    "painted": true, "typedInput": "PointerMove", "interactive": root.supports_pointer(),
                })
            );
            consumed_sources.extend(after.preview_source_receipts.clone());
            previous_source = Some(published(&h, &id));
            seen.insert(*component);
        }
        assert_eq!(seen.len(), 21);
        assert!(seen.contains("WorkbenchDropdownPopup"));
        let before = pane(&h, &id);
        let selected = before.palette_selected_index;
        let frame = before.preview_surface_frame.unwrap();
        pane_surface_host(&h.root_ui).invoke_ui_asset_collection_event(
            id.0.as_str().into(),
            "palette".into(),
            "selected".into(),
            i32::MAX,
            None,
            Default::default(),
        );
        let after = pane(&h, &id);
        assert_eq!(after.palette_selected_index, selected);
        assert!(
            Arc::ptr_eq(&after.preview_surface_frame.unwrap(), &frame),
            "invalid public palette selection cannot partially replace the live source"
        );

        select_index(&h, &id, 0);
        let authoring = pane(&h, &id);
        assert!(
            authoring.preview_source_receipts.is_empty(),
            "returning to authoring retires the builtin source closure"
        );
        assert!(!authoring.source_dirty);
        let restored = published(&h, &id);
        callback(
            &h,
            &id,
            previous_source.expect("final actual sample source"),
            UiInputEvent::Pointer(pointer(UiPointerEventKind::Move, UiPoint::new(0.0, 0.0))),
        );
        assert!(
            Arc::ptr_eq(&published(&h, &id), &restored),
            "retired builtin input cannot mutate the authoring Surface"
        );
        assert!(!consumed_sources.is_empty());
        consumed_sources
    }
}

#[test]
fn builtin_palette_samples_publish_real_source_paint_hits_and_typed_input() {
    let _guard = lock_env();
    UiHostWindow::exercise_builtin_ui_asset_preview_consumers_for_test();
}

#[test]
fn builtin_preview_capture_accepts_lag_up_and_rejects_retired_frames() {
    let _guard = lock_env();
    let (h, id) = open_ui_asset();
    let selected = select(&h, &id, "WorkbenchToggle");
    let before = published(&h, &id);
    let point = root_point(&before, "WorkbenchToggle");
    let native_point = h.root_ui.ui_asset_preview_point_for_test(&id.0, point);
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(
            UiPointerEventKind::Down,
            native_point,
        ));
    let pressed = pane(&h, &id);
    let down = pressed
        .preview_input_result
        .clone()
        .expect("actual normalized Down");
    assert!(
        pressed
            .preview_surface_frame
            .as_ref()
            .unwrap()
            .focus_state
            .pressed
            .is_some()
            || pressed
                .preview_surface_frame
                .as_ref()
                .unwrap()
                .focus_state
                .captured
                .is_some()
    );
    let UiInputEvent::Pointer(input) = &down.event else {
        panic!("actual pointer event");
    };
    let mut up = input.clone();
    up.event =
        UiPointerEvent::new(UiPointerEventKind::Up, point).with_button(UiPointerButton::Primary);
    callback(
        &h,
        &id,
        Arc::clone(&before),
        UiInputEvent::Pointer(up.clone()),
    );
    let after = pane(&h, &id);
    let released = after
        .preview_input_result
        .as_ref()
        .expect("valid lag Up is consumed");
    assert!(
        matches!(&released.event, UiInputEvent::Pointer(pointer) if pointer.event.kind == UiPointerEventKind::Up)
    );
    assert!(
        down.widget_events
            .iter()
            .chain(released.widget_events.iter())
            .any(|event| matches!(
                event,
                UiWidgetEvent::ValueChange {
                    value: zircon_runtime_interface::ui::component::UiValue::Bool(true),
                    ..
                }
            )),
        "Toggle must emit its declared typed value feedback"
    );
    assert!(after
        .preview_surface_frame
        .as_ref()
        .unwrap()
        .focus_state
        .pressed
        .is_none());
    let current = after.preview_surface_frame.as_ref().unwrap().clone();
    callback(
        &h,
        &id,
        Arc::clone(&before),
        UiInputEvent::Pointer(input.clone()),
    );
    assert!(
        Arc::ptr_eq(&published(&h, &id), &current),
        "stale Down cannot overwrite live state"
    );
    assert_eq!(pane(&h, &id).palette_selected_index, selected as i32);
    select(&h, &id, "WorkbenchNumberField");
    select(&h, &id, "WorkbenchToggle");
    let replacement = published(&h, &id);
    assert_eq!(before.tree_id, replacement.tree_id);
    assert_eq!(before.arranged_tree.roots, replacement.arranged_tree.roots);
    assert!(!Arc::ptr_eq(&before, &replacement));
    callback(&h, &id, before, UiInputEvent::Pointer(up));
    assert!(Arc::ptr_eq(&published(&h, &id), &replacement));
    assert!(pane(&h, &id).preview_input_result.is_none());
    select_index(&h, &id, 0);
    let authoring = pane(&h, &id);
    assert!(authoring.preview_source_receipts.is_empty());
    assert!(!authoring.source_dirty);
    let restored = authoring.preview_surface_frame.unwrap();
    callback(&h, &id, replacement, UiInputEvent::Pointer(input.clone()));
    assert!(Arc::ptr_eq(&published(&h, &id), &restored));
}

#[test]
fn builtin_preview_normalized_keyboard_ime_and_cross_pane_focus_use_one_surface() {
    let _guard = lock_env();
    let (h, id) = open_ui_asset();
    select(&h, &id, "WorkbenchNumberField");
    let source = published(&h, &id);
    let point = h
        .root_ui
        .ui_asset_preview_point_for_test(&id.0, root_point(&source, "WorkbenchNumberField"));
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Down, point));
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Up, point));
    assert!(h.root_ui.retained_pane_accepts_text_input());
    let mut metadata = UiInputEventMetadata::default();
    metadata.modifiers.control = true;
    h.root_ui
        .dispatch_retained_pane_platform_input(UiInputEvent::Keyboard(UiKeyboardInputEvent {
            metadata,
            state: UiKeyboardInputState::Pressed,
            key_code: 0,
            scan_code: None,
            physical_key: "KeyA".into(),
            logical_key: "a".into(),
            text: None,
        }))
        .expect("normalized text selection enters the existing Surface");
    for (kind, text) in [
        (UiImeInputEventKind::Preedit, "7"),
        (UiImeInputEventKind::Commit, "7"),
    ] {
        h.root_ui
            .dispatch_retained_pane_platform_input(UiInputEvent::Ime(UiImeInputEvent {
                metadata: UiInputEventMetadata::default(),
                kind,
                text: text.into(),
                cursor_range: None,
                preedit_clauses: Vec::new(),
                delete_surrounding: None,
            }))
            .expect("actual IME event is forwarded without collapsing preedit into legacy text");
        let data = pane(&h, &id);
        assert!(matches!(&data.preview_input_result.as_ref().unwrap().event,
            UiInputEvent::Ime(input) if input.kind == kind));
        assert!(!data.source_dirty);
    }
    let committed = pane(&h, &id).preview_input_result.unwrap();
    assert!(committed
        .widget_events
        .iter()
        .any(|event| matches!(event, UiWidgetEvent::TextEditChange { .. })));
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(
            UiPointerEventKind::Down,
            UiPoint::new(1.0, 1.0),
        ));
    assert!(!h.root_ui.retained_pane_accepts_text_input());
    let after_leave = pane(&h, &id);
    let last_result = after_leave.preview_input_result.unwrap();
    assert!(h
        .root_ui
        .dispatch_retained_pane_platform_input(UiInputEvent::Ime(UiImeInputEvent {
            metadata: UiInputEventMetadata::default(),
            kind: UiImeInputEventKind::Commit,
            text: "9".into(),
            cursor_range: None,
            preedit_clauses: Vec::new(),
            delete_surrounding: None,
        }))
        .is_none());
    assert!(Arc::ptr_eq(
        &pane(&h, &id).preview_input_result.unwrap(),
        &last_result
    ));
    assert!(!pane(&h, &id).source_dirty);
    assert!(
        published(&h, &id).focus_state.focused.is_none(),
        "cross-pane input releases the Surface's actual text focus"
    );
    let source = published(&h, &id);
    let point = h
        .root_ui
        .ui_asset_preview_point_for_test(&id.0, root_point(&source, "WorkbenchNumberField"));
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Down, point));
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Up, point));
    h.root_ui
        .dispatch_retained_pane_platform_input(UiInputEvent::Ime(UiImeInputEvent {
            metadata: UiInputEventMetadata::default(),
            kind: UiImeInputEventKind::Preedit,
            text: "8".into(),
            cursor_range: None,
            preedit_clauses: Vec::new(),
            delete_surrounding: None,
        }))
        .expect("real composition before native focus loss");
    h.root_ui.dispatch_ui_asset_native_focus_lost_for_test();
    let lost = published(&h, &id);
    assert_eq!(lost.window_state.focused, Some(false));
    assert!(lost.focus_state.focused.is_none());
    assert!(lost.focus_state.captured.is_none() && lost.focus_state.pressed.is_none());
    assert!(!h.root_ui.retained_pane_accepts_text_input());
    let result = pane(&h, &id).preview_input_result.unwrap();
    assert!(h
        .root_ui
        .dispatch_retained_pane_platform_input(UiInputEvent::Ime(UiImeInputEvent {
            metadata: UiInputEventMetadata::default(),
            kind: UiImeInputEventKind::Commit,
            text: "8".into(),
            cursor_range: None,
            preedit_clauses: Vec::new(),
            delete_surrounding: None,
        }))
        .is_none());
    assert!(Arc::ptr_eq(
        &pane(&h, &id).preview_input_result.unwrap(),
        &result
    ));

    // Move the actual view, then hide its real drawer while Runtime owns a press and IME.
    apply_layout(
        &h,
        LayoutCommand::MoveView {
            instance_id: id.clone(),
            target: crate::ui::workbench::view::ViewHost::Drawer(ActivityDrawerSlot::Bottom),
        },
    );
    h.activate_drawer_tab(ActivityDrawerSlot::Bottom, &id.0);
    apply_layout(
        &h,
        LayoutCommand::SetDrawerExtent {
            slot: ActivityDrawerSlot::Bottom,
            extent: 600.0,
        },
    );
    let before_hidden = published(&h, &id);
    let local = root_point(&before_hidden, "WorkbenchNumberField");
    let native = h.root_ui.ui_asset_preview_point_for_test(&id.0, local);
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Down, native));
    let held = pane(&h, &id);
    let held_result = held
        .preview_input_result
        .as_ref()
        .expect("real retained press")
        .clone();
    let held_source = held.preview_surface_frame.as_ref().unwrap().clone();
    assert!(held_source.focus_state.focused.is_some());
    assert!(
        held_source.focus_state.captured.is_some() || held_source.focus_state.pressed.is_some()
    );
    let UiInputEvent::Pointer(held_pointer) = &held_result.event else {
        panic!("actual held pointer");
    };
    let mut old_up = held_pointer.clone();
    old_up.event =
        UiPointerEvent::new(UiPointerEventKind::Up, local).with_button(UiPointerButton::Primary);
    h.root_ui
        .dispatch_retained_pane_platform_input(UiInputEvent::Ime(UiImeInputEvent {
            metadata: UiInputEventMetadata::default(),
            kind: UiImeInputEventKind::Preedit,
            text: "4".into(),
            cursor_range: None,
            preedit_clauses: Vec::new(),
            delete_surrounding: None,
        }))
        .expect("actual composition before drawer collapse");
    assert!(has_preview_composition(&published(&h, &id)));
    apply_layout(
        &h,
        LayoutCommand::SetDrawerMode {
            slot: ActivityDrawerSlot::Bottom,
            mode: ActivityDrawerMode::Collapsed,
        },
    );
    assert!(!h.root_ui.retained_pane_input_source_visible(&id.0));
    assert!(!h.root_ui.retained_pane_accepts_text_input());
    let hidden = pane(&h, &id);
    let hidden_source = hidden.preview_surface_frame.as_ref().unwrap().clone();
    let hidden_result = hidden.preview_input_result.as_ref().unwrap().clone();
    assert_eq!(hidden_source.window_state.focused, Some(false));
    assert!(hidden_source.focus_state.focused.is_none());
    assert!(
        hidden_source.focus_state.captured.is_none() && hidden_source.focus_state.pressed.is_none()
    );
    assert!(!has_preview_composition(&hidden_source));
    assert!(!h
        .root_ui
        .ui_asset_preview_paint_sources_for_test()
        .iter()
        .any(|(source, _, _)| Arc::ptr_eq(source, &hidden_source)));
    assert!(h
        .root_ui
        .dispatch_retained_pane_platform_input(UiInputEvent::Pointer(old_up.clone()))
        .is_none());
    for input in [
        UiInputEvent::Keyboard(UiKeyboardInputEvent {
            metadata: UiInputEventMetadata::default(),
            state: UiKeyboardInputState::Pressed,
            key_code: 0,
            scan_code: None,
            physical_key: "KeyA".into(),
            logical_key: "a".into(),
            text: None,
        }),
        UiInputEvent::Ime(UiImeInputEvent {
            metadata: UiInputEventMetadata::default(),
            kind: UiImeInputEventKind::Commit,
            text: "4".into(),
            cursor_range: None,
            preedit_clauses: Vec::new(),
            delete_surrounding: None,
        }),
    ] {
        assert!(h
            .root_ui
            .dispatch_retained_pane_platform_input(input)
            .is_none());
    }
    callback(
        &h,
        &id,
        held_source.clone(),
        UiInputEvent::Pointer(old_up.clone()),
    );
    callback(
        &h,
        &id,
        hidden_source.clone(),
        UiInputEvent::Pointer(held_pointer.clone()),
    );
    assert!(Arc::ptr_eq(
        &pane(&h, &id).preview_surface_frame.unwrap(),
        &hidden_source
    ));
    assert!(Arc::ptr_eq(
        &pane(&h, &id).preview_input_result.unwrap(),
        &hidden_result
    ));
    h.activate_drawer_tab(ActivityDrawerSlot::Bottom, &id.0);
    let reopened = published(&h, &id);
    assert!(reopened.focus_state.focused.is_none());
    assert!(reopened.focus_state.captured.is_none() && reopened.focus_state.pressed.is_none());
    assert!(!has_preview_composition(&reopened));
    callback(&h, &id, before_hidden, UiInputEvent::Pointer(old_up));
    assert!(
        Arc::ptr_eq(&published(&h, &id), &reopened),
        "reopening cannot revive the retired press"
    );
    let native = h
        .root_ui
        .ui_asset_preview_point_for_test(&id.0, root_point(&reopened, "WorkbenchNumberField"));
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Down, native));
    assert!(published(&h, &id).focus_state.focused.is_some());
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Up, native));
    assert!(h.root_ui.retained_pane_accepts_text_input());
    assert!(!pane(&h, &id).source_dirty);

    // The public UiAssetEditor can keep two different production documents visible.
    apply_layout(
        &h,
        LayoutCommand::SetDrawerExtent {
            slot: ActivityDrawerSlot::Bottom,
            extent: 300.0,
        },
    );
    let second = {
        let host = h.host.borrow();
        let manager = host.editor_manager.enter().expect("manager admission");
        manager
            .open_ui_asset_editor(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("assets/ui/editor/component_showcase.zui"),
                None,
            )
            .expect("second existing production document")
    };
    assert_ne!(second, id);
    {
        let mut host = h.host.borrow_mut();
        host.mark_layout_dirty();
        host.refresh_ui();
        host.recompute_if_dirty();
    }
    select(&h, &second, "WorkbenchToggle");
    assert!(h.root_ui.retained_pane_input_source_visible(&id.0));
    assert!(h.root_ui.retained_pane_input_source_visible(&second.0));
    let old = published(&h, &id);
    let native = h
        .root_ui
        .ui_asset_preview_point_for_test(&id.0, root_point(&old, "WorkbenchNumberField"));
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Down, native));
    h.root_ui
        .dispatch_retained_pane_platform_input(UiInputEvent::Ime(UiImeInputEvent {
            metadata: UiInputEventMetadata::default(),
            kind: UiImeInputEventKind::Preedit,
            text: "5".into(),
            cursor_range: None,
            preedit_clauses: Vec::new(),
            delete_surrounding: None,
        }))
        .expect("old visible pane owns a real composition");
    let old_active = published(&h, &id);
    assert!(has_preview_composition(&old_active));
    assert!(old_active.focus_state.focused.is_some());
    assert!(old_active.focus_state.captured.is_some() || old_active.focus_state.pressed.is_some());
    let new_source = published(&h, &second);
    let records = h.root_ui.ui_asset_preview_paint_sources_for_test();
    for source in [&old_active, &new_source] {
        assert!(
            records
                .iter()
                .any(|(painted, _, _)| Arc::ptr_eq(painted, source)),
            "both actual source Surfaces must be visible before switching input"
        );
    }
    let new_local = root_point(&new_source, "WorkbenchToggle");
    let new_native = h
        .root_ui
        .ui_asset_preview_point_for_test(&second.0, new_local);
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(
            UiPointerEventKind::Down,
            new_native,
        ));
    let prior = published(&h, &id);
    assert!(
        h.root_ui.retained_pane_input_source_visible(&id.0),
        "old pane remains visible"
    );
    assert_eq!(prior.window_state.focused, Some(false));
    assert!(prior.focus_state.focused.is_none());
    assert!(prior.focus_state.captured.is_none() && prior.focus_state.pressed.is_none());
    assert!(!has_preview_composition(&prior));
    let down = pane(&h, &second).preview_input_result.unwrap();
    assert!(
        matches!(&down.event, UiInputEvent::Pointer(pointer) if pointer.event.kind == UiPointerEventKind::Down)
    );
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Up, new_native));
    let up = pane(&h, &second).preview_input_result.unwrap();
    assert!(down
        .widget_events
        .iter()
        .chain(up.widget_events.iter())
        .any(|event| matches!(
            event,
            UiWidgetEvent::ValueChange {
                value: zircon_runtime_interface::ui::component::UiValue::Bool(true),
                ..
            }
        )));
    assert!(!h.root_ui.retained_pane_accepts_text_input());
    assert!(!pane(&h, &id).source_dirty && !pane(&h, &second).source_dirty);
}

#[test]
fn builtin_popup_instance_defaults_and_range_capture_use_declared_runtime_behavior() {
    let _guard = lock_env();
    let (h, id) = open_ui_asset();
    select(&h, &id, "WorkbenchDropdownPopup");
    let initial = published(&h, &id);
    assert!(h
        .root_ui
        .ui_asset_preview_paint_sources_for_test()
        .iter()
        .any(|(source, _, _)| Arc::ptr_eq(source, &initial)));
    let point = h
        .root_ui
        .ui_asset_preview_point_for_test(&id.0, root_point(&initial, "WorkbenchDropdownPopup"));
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Down, point));
    h.root_ui
        .dispatch_retained_pane_platform_input(UiInputEvent::Keyboard(UiKeyboardInputEvent {
            metadata: UiInputEventMetadata::default(),
            state: UiKeyboardInputState::Pressed,
            key_code: 0,
            scan_code: None,
            physical_key: "Escape".into(),
            logical_key: "Escape".into(),
            text: None,
        }))
        .expect("popup Escape uses the current preview's typed input path");
    let dismissed = pane(&h, &id);
    let result = dismissed.preview_input_result.unwrap();
    assert!(
        result
            .widget_events
            .iter()
            .any(|event| matches!(event, UiWidgetEvent::OpenChanged { open: false, .. })),
        "catalog defaults do not bypass later actual popup dismissal"
    );
    assert!(!dismissed.source_dirty);
    select(&h, &id, "WorkbenchDragOverlay");
    let drag = published(&h, &id);
    let point = h
        .root_ui
        .ui_asset_preview_point_for_test(&id.0, root_point(&drag, "WorkbenchDragOverlay"));
    h.root_ui
        .dispatch_retained_pane_platform_input(UiInputEvent::DragDrop(
            zircon_runtime_interface::ui::dispatch::UiDragDropInputEvent {
                metadata: UiInputEventMetadata::default(),
                kind: zircon_runtime_interface::ui::dispatch::UiDragDropInputEventKind::Begin,
                session_id: None,
                point,
                payload: None,
            },
        ))
        .expect("the actual canvas admits the normalized drag input");
    let invalid_drag = pane(&h, &id).preview_input_result.unwrap();
    assert!(
        !invalid_drag.rejected_effects.is_empty(),
        "authored drag labels do not manufacture an operation payload"
    );
    assert!(!pane(&h, &id).source_dirty);
    select(&h, &id, "WorkbenchSlider");
    let range = published(&h, &id);
    let point = h
        .root_ui
        .ui_asset_preview_point_for_test(&id.0, root_point(&range, "WorkbenchSlider"));
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Down, point));
    assert!(published(&h, &id).focus_state.captured.is_some());
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(
            UiPointerEventKind::Move,
            UiPoint::new(-20.0, point.y),
        ));
    let moved = pane(&h, &id).preview_input_result.unwrap();
    assert!(moved
        .widget_events
        .iter()
        .any(|event| matches!(event, UiWidgetEvent::ValueChange { .. })));
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(
            UiPointerEventKind::Up,
            UiPoint::new(-20.0, point.y),
        ));
    assert!(published(&h, &id).focus_state.captured.is_none());
    let current = published(&h, &id);
    let point = h
        .root_ui
        .ui_asset_preview_point_for_test(&id.0, root_point(&current, "WorkbenchSlider"));
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Down, point));
    assert!(published(&h, &id).focus_state.captured.is_some());
    h.root_ui.dispatch_ui_asset_native_pointer_left_for_test();
    let left = published(&h, &id);
    assert!(left.focus_state.captured.is_none() && left.focus_state.pressed.is_none());
    let point = h
        .root_ui
        .ui_asset_preview_point_for_test(&id.0, root_point(&left, "WorkbenchSlider"));
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Down, point));
    assert!(published(&h, &id).focus_state.captured.is_some());
    assert!(
        h.root_ui.dispatch_ui_asset_native_close_for_test(),
        "actual clean native close admission"
    );
    let closed = published(&h, &id);
    assert!(closed.window_state.closed);
    assert!(closed.focus_state.captured.is_none() && closed.focus_state.pressed.is_none());
    assert!(closed.focus_state.focused.is_none());
    let result = pane(&h, &id).preview_input_result.unwrap();
    assert!(h
        .root_ui
        .dispatch_retained_pane_platform_input(UiInputEvent::Pointer(pointer(
            UiPointerEventKind::Down,
            point
        )))
        .is_none());
    callback(
        &h,
        &id,
        Arc::clone(&closed),
        UiInputEvent::Pointer(pointer(UiPointerEventKind::Down, UiPoint::new(1.0, 1.0))),
    );
    assert!(Arc::ptr_eq(&published(&h, &id), &closed));
    assert!(Arc::ptr_eq(
        &pane(&h, &id).preview_input_result.unwrap(),
        &result
    ));
    assert!(!pane(&h, &id).source_dirty);
}

fn palette_search_native_key(
    h: &ChildWindowHostHarness,
    key: Key,
    physical: KeyCode,
    modifiers: ModifiersState,
) {
    h.root_ui.dispatch_native_key_for_test(
        key_event(
            key,
            PhysicalKey::Code(physical),
            None,
            ElementState::Pressed,
        ),
        modifiers,
    );
    h.host.borrow_mut().recompute_if_dirty();
}

fn assert_palette_search_navigation_only(
    before: &crate::ui::asset_editor::UiAssetEditorPanePresentation,
    after: &crate::ui::asset_editor::UiAssetEditorPanePresentation,
) {
    assert_eq!(after.source_text, before.source_text);
    assert_eq!(after.source_dirty, before.source_dirty);
    assert_eq!(after.can_undo, before.can_undo);
    assert_eq!(after.can_redo, before.can_redo);
    assert_eq!(
        after.palette_search.as_ref().unwrap().source_revision(),
        before.palette_search.as_ref().unwrap().source_revision()
    );
}

#[test]
fn palette_search_native_keyboard_keeps_raw_whitespace_until_clear_without_document_history() {
    let _guard = lock_env();
    let (h, id) = open_ui_asset();
    let before = pane(&h, &id);
    let initial_count = before.palette_search.as_ref().unwrap().result_count();
    click_control(&h, &id, "PaletteSearchInput");
    let input = host_context(&h.root_ui)
        .get_text_input_focus()
        .palette_search_target
        .expect("the actual native Palette hit must capture its source target");
    assert_eq!(input.instance_id(), id.0.as_str());
    assert!(h.root_ui.palette_search_input_is_current(&input, "query"));

    // These are committed native text edits, not an IME preedit or a clear-X click.
    // Control characters (for example Tab) have a separate routing contract.
    for query in ["   ", "\u{00a0}\u{2003}\u{3000}"] {
        h.root_ui.dispatch_native_text_for_test(query);
        h.host.borrow_mut().recompute_if_dirty();
        let typed = pane(&h, &id);
        let snapshot = typed.palette_search.as_ref().unwrap();
        assert_eq!(
            snapshot.query(),
            query,
            "the Session must retain the raw query"
        );
        assert_eq!(snapshot.result_count(), initial_count);
        assert_palette_search_navigation_only(&before, &typed);
        let focus = host_context(&h.root_ui).get_text_input_focus();
        assert_eq!(focus.value_text.as_str(), query);
        assert_eq!(focus.caret_scalar_offset, Some(query.chars().count()));
        assert!(focus.palette_search_target.as_ref() == Some(&input));

        palette_search_native_key(
            &h,
            Key::Character("a".into()),
            KeyCode::KeyA,
            ModifiersState::CONTROL,
        );
        let selected = host_context(&h.root_ui).get_text_input_focus();
        assert_eq!(selected.selection_anchor_scalar_offset, Some(0));
        assert_eq!(selected.caret_scalar_offset, Some(query.chars().count()));
        palette_search_native_key(
            &h,
            Key::Named(NamedKey::Delete),
            KeyCode::Delete,
            ModifiersState::empty(),
        );
        let cleared = pane(&h, &id);
        assert_eq!(cleared.palette_search.as_ref().unwrap().query(), "");
        assert_eq!(
            cleared.palette_search.as_ref().unwrap().result_count(),
            initial_count
        );
        assert_palette_search_navigation_only(&before, &cleared);
        let focus = host_context(&h.root_ui).get_text_input_focus();
        assert_eq!(focus.control_id.as_str(), "PaletteSearchInput");
        assert_eq!(focus.value_text.as_str(), "");
        assert_eq!(focus.caret_scalar_offset, Some(0));
        assert_eq!(focus.selection_anchor_scalar_offset, None);
        assert!(focus.palette_search_target.as_ref() == Some(&input));
        assert!(h.root_ui.palette_search_input_is_current(&input, "query"));
    }
}

#[test]
fn palette_search_native_unicode_caret_clear_and_reopen_reject_old_query_callback() {
    let _guard = lock_env();
    let (h, id) = open_ui_asset();
    let before = pane(&h, &id);
    click_control(&h, &id, "PaletteSearchInput");
    let original_target = host_context(&h.root_ui)
        .get_text_input_focus()
        .palette_search_target
        .expect("actual Palette field press establishes the original input allocation");
    h.root_ui.dispatch_native_text_for_test("中🙂\u{2003}");
    h.host.borrow_mut().recompute_if_dirty();
    assert_eq!(
        pane(&h, &id).palette_search.as_ref().unwrap().query(),
        "中🙂\u{2003}"
    );
    assert_eq!(
        host_context(&h.root_ui)
            .get_text_input_focus()
            .caret_scalar_offset,
        Some(3)
    );
    palette_search_native_key(
        &h,
        Key::Named(NamedKey::ArrowLeft),
        KeyCode::ArrowLeft,
        ModifiersState::empty(),
    );
    assert_eq!(
        host_context(&h.root_ui)
            .get_text_input_focus()
            .caret_scalar_offset,
        Some(2)
    );
    palette_search_native_key(
        &h,
        Key::Named(NamedKey::Backspace),
        KeyCode::Backspace,
        ModifiersState::empty(),
    );
    assert_eq!(
        pane(&h, &id).palette_search.as_ref().unwrap().query(),
        "中\u{2003}"
    );
    assert_eq!(
        host_context(&h.root_ui)
            .get_text_input_focus()
            .caret_scalar_offset,
        Some(1)
    );
    palette_search_native_key(
        &h,
        Key::Character("a".into()),
        KeyCode::KeyA,
        ModifiersState::CONTROL,
    );
    palette_search_native_key(
        &h,
        Key::Named(NamedKey::Delete),
        KeyCode::Delete,
        ModifiersState::empty(),
    );
    assert_eq!(pane(&h, &id).palette_search.as_ref().unwrap().query(), "");
    assert_palette_search_navigation_only(&before, &pane(&h, &id));

    h.root_ui.dispatch_native_focus_lost_for_test();
    assert!(!host_context(&h.root_ui).get_text_input_focus().is_active());
    h.root_ui.dispatch_native_text_for_test("retired 中文");
    h.host.borrow_mut().recompute_if_dirty();
    assert_eq!(pane(&h, &id).palette_search.as_ref().unwrap().query(), "");
    click_control(&h, &id, "PaletteSearchInput");
    let fresh_focus = host_context(&h.root_ui).get_text_input_focus();
    let fresh_target = fresh_focus.palette_search_target.as_ref().unwrap();
    assert!(
        fresh_target != &original_target,
        "a new hit must not revive the retired field"
    );
    assert!(h
        .root_ui
        .palette_search_input_is_current(fresh_target, "query"));
    assert!(!h
        .root_ui
        .palette_search_input_is_current(&original_target, "query"));
    pane_surface_host(&h.root_ui).invoke_ui_asset_collection_event(
        id.0.as_str().into(),
        "palette".into(),
        "query".into(),
        -1,
        Some(original_target),
        "late callback".into(),
    );
    h.host.borrow_mut().recompute_if_dirty();
    assert_eq!(pane(&h, &id).palette_search.as_ref().unwrap().query(), "");
    assert!(host_context(&h.root_ui).get_text_input_focus() == fresh_focus);
    h.root_ui.dispatch_native_text_for_test("Button");
    h.host.borrow_mut().recompute_if_dirty();
    assert_eq!(
        pane(&h, &id).palette_search.as_ref().unwrap().query(),
        "Button"
    );
    assert_palette_search_navigation_only(&before, &pane(&h, &id));
}

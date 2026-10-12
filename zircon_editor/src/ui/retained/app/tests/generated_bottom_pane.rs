use super::*;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
use std::collections::BTreeSet;
use std::sync::Arc;
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiImeInputEvent, UiImeInputEventKind, UiInputEvent, UiInputEventMetadata,
        UiKeyboardInputEvent, UiKeyboardInputState, UiPointerEvent, UiPointerInputEvent,
        UiTextInputEvent,
    },
    layout::UiPoint,
    surface::{UiPointerButton, UiPointerEventKind, UiSurfaceFrame},
    window::UiWindowInputPumpEvent,
};

fn pointer(kind: UiPointerEventKind, point: UiPoint) -> UiPointerInputEvent {
    UiPointerInputEvent {
        metadata: UiInputEventMetadata::default(),
        event: UiPointerEvent::new(kind, point).with_button(UiPointerButton::Primary),
        precise_scroll: None,
    }
}

fn apply_layout(h: &ChildWindowHostHarness, command: LayoutCommand) {
    let mut host = h.host.borrow_mut();
    let effects = callback_dispatch::dispatch_layout_command(&host.runtime, command)
        .expect("actual Generated layout command admission");
    host.apply_dispatch_effects(effects);
    host.refresh_ui();
    host.recompute_if_dirty();
}

fn published(h: &ChildWindowHostHarness, id: &ViewInstanceId) -> Arc<UiSurfaceFrame> {
    let data = h.root_ui.generated_bottom_pane_for_test(&id.0);
    let source = data
        .source_surface_frame
        .expect("published Generated native body source");
    let state = h
        .host
        .borrow()
        .builtin_template_runtime
        .generated_bottom_owner_state_for_test(&id.0)
        .expect("the existing builtin runtime owns the actual pane Surface");
    assert!(state.0);
    assert!(
        Arc::ptr_eq(&source, &state.2),
        "paint and input publish the same holder frame"
    );
    source
}

fn sequence(h: &ChildWindowHostHarness) -> u64 {
    let journal = h.host.borrow().runtime.journal();
    journal
        .records()
        .iter()
        .map(|record| record.sequence.0)
        .max()
        .unwrap_or(0)
}

fn callback(
    h: &ChildWindowHostHarness,
    id: &ViewInstanceId,
    source: Arc<UiSurfaceFrame>,
    input: UiInputEvent,
) {
    pane_surface_host(&h.root_ui).invoke_retained_pane_input(
        id.0.as_str().into(),
        source,
        UiWindowInputPumpEvent::Input(input),
    );
}

fn props(
    h: &ChildWindowHostHarness,
    id: &ViewInstanceId,
    control: &str,
) -> std::collections::BTreeMap<String, UiValue> {
    h.host
        .borrow()
        .builtin_template_runtime
        .generated_bottom_control_properties_for_test(&id.0, control)
        .expect("actual same-Surface control properties")
}

fn assert_painted(h: &ChildWindowHostHarness, source: &Arc<UiSurfaceFrame>) {
    assert!(
        h.root_ui
            .ui_asset_preview_paint_sources_for_test()
            .iter()
            .any(|(frame, _, rect)| Arc::ptr_eq(frame, source)
                && rect.width > 0.0
                && rect.height > 0.0),
        "the actual pane painter must retain the current positive, clipped source command"
    );
}

// Reveal off-screen authored rows through the existing normalized scroll owner.
// The probe uses actual arranged frames and clip frames; it never edits a Surface layout.
fn reveal(h: &ChildWindowHostHarness, id: &ViewInstanceId, control: &str) -> UiPoint {
    for _ in 0..96 {
        if let Some(point) = h
            .root_ui
            .generated_bottom_control_point_for_test(&id.0, control)
        {
            return point;
        }
        let source = published(h, id);
        let node = source
            .arranged_tree
            .nodes
            .iter()
            .find(|node| node.control_id.as_deref() == Some(control))
            .expect("authored control remains in the current arranged source");
        let cx = node.frame.x + node.frame.width * 0.5;
        let cy = node.frame.y + node.frame.height * 0.5;
        let horizontal = cx < node.clip_frame.x || cx >= node.clip_frame.x + node.clip_frame.width;
        let scroller = if horizontal {
            if control.starts_with("WorkbenchGeneratedBottomMode") {
                "WorkbenchGeneratedBottomModes"
            } else {
                "WorkbenchGeneratedBottomRouteTableHorizontalViewport"
            }
        } else {
            "WorkbenchGeneratedBottomRouteTable"
        };
        let center = if horizontal { cx } else { cy };
        let clip_start = if horizontal {
            node.clip_frame.x
        } else {
            node.clip_frame.y
        };
        let delta = if center < clip_start { -96.0 } else { 96.0 };
        let point = h
            .root_ui
            .generated_bottom_control_point_for_test(&id.0, scroller)
            .expect("actual visible native scroll owner");
        let mut input = pointer(UiPointerEventKind::Scroll, point);
        input.event = input.event.with_scroll_delta(delta);
        h.root_ui
            .dispatch_retained_pane_platform_input(UiInputEvent::Pointer(input))
            .expect("actual normalized Generated scroll admission");
        h.host.borrow_mut().recompute_if_dirty();
        assert!(
            !Arc::ptr_eq(&source, &published(h, id)),
            "a real scroll must publish changed geometry"
        );
    }
    panic!("actual normalized scrolling did not expose {control}");
}

fn assert_record(h: &ChildWindowHostHarness, binding: &EditorUiBinding, baseline: u64) {
    let journal = h.host.borrow().runtime.journal();
    let record = journal
        .records()
        .iter()
        .max_by_key(|record| record.sequence.0)
        .expect("actual Controller observation");
    assert_eq!(
        record.sequence.0,
        baseline + 1,
        "one authored event makes one observation even when PressNode records coalesce"
    );
    assert_eq!(
        record.source,
        crate::core::editor_event::EditorEventSource::RetainedHost
    );
    assert_eq!(
        record.binding_path.as_deref(),
        Some(binding.path().native_prefix().as_str())
    );
    let EditorUiBindingPayload::MenuAction { action_id } = binding.payload() else {
        panic!("canonical authored MenuAction");
    };
    let node_path = format!("{}/{}", binding.path().view_id, binding.path().control_id);
    assert_eq!(
        record.operation_arguments,
        Some(serde_json::json!({
            "control_id": binding.path().control_id, "node_path": node_path, "action_id": action_id,
        }))
    );
    assert!(
        matches!(&record.event, EditorEvent::Transient(EditorEventTransient::PressNode { node_path: path, pressed: false })
        if path == &node_path)
    );
    assert_eq!(
        record.operation_display_name.as_deref(),
        Some("Component Lab Preview Action")
    );
    assert_eq!(
        record.operation_group.as_deref(),
        Some("ComponentLabPreview")
    );
    assert_eq!(
        record.undo_policy,
        crate::core::editor_event::EditorEventUndoPolicy::NonUndoable
    );
    assert!(
        record.effects.is_empty()
            && record.transaction_id.is_none()
            && record.save_generation.is_none()
    );
    assert_eq!(record.before_revision, record.after_revision);
    assert!(record.result.error.is_none());
}

fn assert_association(
    h: &ChildWindowHostHarness,
    id: &ViewInstanceId,
    binding_id: &str,
    control: &str,
    kind: &UiEventKind,
) {
    let source = published(h, id);
    let node = source
        .arranged_tree
        .nodes
        .iter()
        .find(|node| node.control_id.as_deref() == Some(control))
        .expect("the original control is in the actual Surface");
    let result = h
        .host
        .borrow()
        .builtin_template_runtime
        .generated_bottom_pane_input_result(&id.0)
        .expect("actual normalized input result");
    let reports: Vec<_> = result
        .component_events
        .iter()
        .filter(|report| {
            report.delivered
                && report.template_action.as_ref().is_some_and(|action| {
                    action.is_action()
                        && action.target_id() == format!("generated-bottom-surface/{binding_id}")
                })
        })
        .collect();
    assert_eq!(
        reports.len(),
        1,
        "exactly the authored producer association is dispatched"
    );
    assert_eq!(reports[0].target, node.node_id);
    match kind {
        UiEventKind::Change => assert!(
            matches!(&reports[0].event, UiComponentEvent::ValueChanged { value: UiValue::String(value), .. } if value == "生成 Ω")
        ),
        UiEventKind::Submit => assert!(
            matches!(&reports[0].event, UiComponentEvent::Commit { value: UiValue::String(value), .. } if value == "生成 Ω")
        ),
        UiEventKind::Click => assert!(matches!(&reports[0].event, UiComponentEvent::Commit { .. })),
        _ => panic!("unexpected authored producer"),
    }
}

fn assert_feedback(h: &ChildWindowHostHarness, id: &ViewInstanceId, binding: &EditorUiBinding) {
    let EditorUiBindingPayload::MenuAction { action_id } = binding.payload() else {
        panic!("canonical MenuAction");
    };
    let (updates, _) = callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge::generated_bottom_pane_feedback_attributes(action_id);
    for (control, attributes) in updates {
        let actual = props(h, id, &control);
        for (name, value) in &attributes {
            let expected = match value {
                toml::Value::Boolean(value) => UiValue::Bool(*value),
                toml::Value::String(value) => UiValue::String(value.clone()),
                _ => panic!("existing Generated feedback type"),
            };
            assert_eq!(
                actual.get(name),
                Some(&expected),
                "{control}.{name} must be written to the same actual Surface"
            );
        }
        if let Some(toml::Value::Boolean(selected)) = attributes.get("selected") {
            if attributes.get("checked") == Some(&toml::Value::Boolean(*selected)) {
                let subtree = h
                    .host
                    .borrow()
                    .builtin_template_runtime
                    .generated_bottom_subtree_selection_for_test(&id.0, &control)
                    .expect("actual authored mode subtree");
                assert!(!subtree.is_empty());
                for (node, actual_selected, actual_checked) in subtree {
                    assert_eq!(
                        actual_selected,
                        Some(*selected),
                        "{control} descendant {node:?} selected state"
                    );
                    assert_eq!(
                        actual_checked, *selected,
                        "{control} descendant {node:?} checked state"
                    );
                }
            }
        }
    }
}

fn click(h: &ChildWindowHostHarness, id: &ViewInstanceId, control: &str) -> u64 {
    let point = reveal(h, id, control);
    let source = published(h, id);
    assert_painted(h, &source);
    let node = source
        .arranged_tree
        .nodes
        .iter()
        .find(|node| node.control_id.as_deref() == Some(control))
        .unwrap();
    let local = UiPoint::new(
        (node.frame.x.max(node.clip_frame.x)
            + (node.frame.x + node.frame.width).min(node.clip_frame.x + node.clip_frame.width))
            * 0.5,
        (node.frame.y.max(node.clip_frame.y)
            + (node.frame.y + node.frame.height).min(node.clip_frame.y + node.clip_frame.height))
            * 0.5,
    );
    assert_eq!(
        zircon_runtime::ui::surface::hit_test_surface_frame(&source, local).top_hit,
        Some(node.node_id),
        "actual authored control hit"
    );
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Down, point));
    h.host.borrow_mut().recompute_if_dirty();
    let baseline = sequence(h);
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Up, point));
    h.host.borrow_mut().recompute_if_dirty();
    baseline
}

fn has_composition(source: &UiSurfaceFrame) -> bool {
    source.render_extract.list.commands.iter().any(|command| {
        command
            .text_layout
            .as_ref()
            .and_then(|layout| layout.editable.as_ref())
            .is_some_and(|editable| editable.composition.is_some())
    })
}

impl UiHostWindow {
    /// Caller holds the existing environment lock; no nested lock is taken here.
    pub(crate) fn exercise_generated_bottom_sources_for_test(
    ) -> Vec<zircon_runtime::ui::v2::UiV2SourceFileReceipt> {
        let h = ChildWindowHostHarness::new("generated_bottom_single_surface");
        h.activate_workbench_page();
        assert!(
            h.host
                .borrow()
                .builtin_template_runtime
                .generated_bottom_pane_ids()
                .is_empty(),
            "startup does not eagerly construct the Generated owner"
        );
        let id = h.open_view("editor.generated_bottom");
        h.activate_drawer_tab(ActivityDrawerSlot::Bottom, &id.0);
        apply_layout(
            &h,
            LayoutCommand::SetDrawerExtent {
                slot: ActivityDrawerSlot::Bottom,
                extent: 600.0,
            },
        );
        let initial = published(&h, &id);
        assert_painted(&h, &initial);
        for control in [
            "WorkbenchGeneratedBottomDrawer",
            "WorkbenchGeneratedBottomPanelHost",
        ] {
            let node = initial
                .arranged_tree
                .nodes
                .iter()
                .find(|node| node.control_id.as_deref() == Some(control))
                .expect("actual wrapper and overridden Panel root");
            assert!(
                node.frame.width > 0.0
                    && node.frame.height > 0.0
                    && node.clip_frame.width > 0.0
                    && node.clip_frame.height > 0.0
            );
        }
        let bindings = h
            .host
            .borrow()
            .builtin_template_runtime
            .generated_bottom_bindings_for_test(&id.0);
        assert_eq!(bindings.len(), 46);
        assert_eq!(
            bindings
                .iter()
                .map(|(id, _, _, _)| id)
                .collect::<BTreeSet<_>>()
                .len(),
            46
        );
        assert_eq!(
            bindings
                .iter()
                .map(|(_, control, _, _)| control)
                .collect::<BTreeSet<_>>()
                .len(),
            45
        );
        assert_eq!(
            bindings
                .iter()
                .filter(|(_, _, kind, _)| *kind == UiEventKind::Click)
                .count(),
            44
        );
        assert_eq!(
            bindings
                .iter()
                .filter(|(_, _, kind, _)| *kind == UiEventKind::Change)
                .count(),
            1
        );
        assert_eq!(
            bindings
                .iter()
                .filter(|(_, _, kind, _)| *kind == UiEventKind::Submit)
                .count(),
            1
        );
        let mut exercised = BTreeSet::new();
        for (binding_id, control, kind, binding) in &bindings {
            assert_eq!(
                format!("{}/{}", binding.path().view_id, binding.path().control_id),
                *binding_id
            );
            assert_eq!(binding.path().event_kind, *kind);
            if *kind != UiEventKind::Click {
                continue;
            }
            let baseline = click(&h, &id, control);
            assert_association(&h, &id, binding_id, control, kind);
            assert_record(&h, binding, baseline);
            assert_feedback(&h, &id, binding);
            exercised.insert(binding_id.clone());
        }
        let filter = "WorkbenchGeneratedBottomFilterField";
        let point = reveal(&h, &id, filter);
        let baseline = sequence(&h);
        h.root_ui
            .dispatch_ui_asset_normalized_pointer_for_test(pointer(
                UiPointerEventKind::Move,
                point,
            ));
        assert_eq!(
            sequence(&h),
            baseline,
            "ordinary None association cannot manufacture a MenuAction"
        );
        let ordinary = h
            .host
            .borrow()
            .builtin_template_runtime
            .generated_bottom_pane_input_result(&id.0)
            .unwrap();
        assert!(ordinary
            .component_events
            .iter()
            .any(|report| report.delivered && report.template_action.is_none()));
        h.root_ui
            .dispatch_ui_asset_normalized_pointer_for_test(pointer(
                UiPointerEventKind::Down,
                point,
            ));
        h.root_ui
            .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Up, point));
        assert!(h.root_ui.retained_pane_accepts_text_input());
        let change = bindings
            .iter()
            .find(|(_, _, kind, _)| *kind == UiEventKind::Change)
            .unwrap();
        let baseline = sequence(&h);
        h.root_ui
            .dispatch_retained_pane_platform_input(UiInputEvent::Text(UiTextInputEvent {
                metadata: UiInputEventMetadata::default(),
                text: "生成 Ω".into(),
            }))
            .expect("actual Generated text input");
        assert_association(&h, &id, &change.0, &change.1, &change.2);
        assert_record(&h, &change.3, baseline);
        assert_feedback(&h, &id, &change.3);
        exercised.insert(change.0.clone());
        let submit = bindings
            .iter()
            .find(|(_, _, kind, _)| *kind == UiEventKind::Submit)
            .unwrap();
        let baseline = sequence(&h);
        h.root_ui
            .dispatch_retained_pane_platform_input(UiInputEvent::Keyboard(UiKeyboardInputEvent {
                metadata: UiInputEventMetadata::default(),
                state: UiKeyboardInputState::Pressed,
                key_code: 0,
                scan_code: None,
                physical_key: "Enter".into(),
                logical_key: "Enter".into(),
                text: None,
            }))
            .expect("actual Generated submit input");
        assert_association(&h, &id, &submit.0, &submit.1, &submit.2);
        assert_record(&h, &submit.3, baseline);
        assert_feedback(&h, &id, &submit.3);
        exercised.insert(submit.0.clone());
        assert_eq!(
            exercised,
            bindings
                .iter()
                .map(|(id, _, _, _)| id.clone())
                .collect::<BTreeSet<_>>()
        );
        assert_eq!(
            props(&h, &id, filter).get("value"),
            Some(&UiValue::String("生成 Ω".into()))
        );

        // Native field callbacks must resolve the same current projection's separate producer kinds.
        let direct = h.root_ui.generated_bottom_pane_for_test(&id.0);
        let direct_filter = direct
            .nodes
            .iter()
            .find(|node| node.control_id.as_str() == filter)
            .unwrap();
        assert_ne!(direct_filter.edit_action_id, direct_filter.commit_action_id);
        let before_edit = published(&h, &id);
        let baseline = sequence(&h);
        pane_surface_host(&h.root_ui).invoke_surface_control_edited(
            filter.into(),
            direct_filter.edit_action_id.clone(),
            "direct Ω".into(),
        );
        assert_record(&h, &change.3, baseline);
        assert_eq!(
            props(&h, &id, filter).get("value"),
            Some(&UiValue::String("direct Ω".into()))
        );
        let after_edit = published(&h, &id);
        assert!(!Arc::ptr_eq(&before_edit, &after_edit));
        assert_painted(&h, &after_edit);
        let direct = h.root_ui.generated_bottom_pane_for_test(&id.0);
        let direct_filter = direct
            .nodes
            .iter()
            .find(|node| node.control_id.as_str() == filter)
            .unwrap();
        let baseline = sequence(&h);
        pane_surface_host(&h.root_ui).invoke_surface_control_edited(
            filter.into(),
            direct_filter.commit_action_id.clone(),
            "direct Ω".into(),
        );
        assert_record(&h, &submit.3, baseline);
        assert_eq!(
            props(&h, &id, filter).get("value"),
            Some(&UiValue::String("direct Ω".into()))
        );

        let current = published(&h, &id);
        assert_painted(&h, &current);
        let receipts = h
            .host
            .borrow()
            .builtin_template_runtime
            .generated_bottom_pane_source_receipts_for_test(&id.0)
            .expect("receipts only follow actual visible/current source input and paint");
        let original = zircon_runtime::ui::v2::UiV2PrototypeStoreFileCache::new()
            .load_store([
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("assets/ui/editor/host/generated_bottom_body.zui"),
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("assets/ui/theme/editor_workbench_strict.zui"),
            ])
            .expect("same current physical Generated source and strict theme closure");
        assert_eq!(receipts, original.source_receipts);
        for suffix in [
            "generated_bottom_body.zui",
            "workbench_generated_bottom_drawer.zui",
            "workbench_generated_bottom_panel.zui",
        ] {
            assert!(receipts.iter().any(
                |receipt| receipt.physical_path.ends_with(suffix) && receipt.sha256.len() == 64
            ));
        }

        // A published token and frame cannot survive another real Surface publication.
        let old = h.root_ui.generated_bottom_pane_for_test(&id.0);
        let old_node = old
            .nodes
            .iter()
            .find(|node| node.control_id.as_str() == "WorkbenchGeneratedBottomModeOutput")
            .unwrap();
        let old_token = old_node.action_id.clone();
        assert!(old_token.as_str().contains("/generated-bottom/e"));
        let baseline = click(&h, &id, "WorkbenchGeneratedBottomModeRuntime");
        let runtime_binding = bindings
            .iter()
            .find(|(_, control, _, _)| control == "WorkbenchGeneratedBottomModeRuntime")
            .unwrap();
        assert_record(&h, &runtime_binding.3, baseline);
        let fresh = published(&h, &id);
        assert!(!Arc::ptr_eq(&current, &fresh));
        let baseline = sequence(&h);
        pane_surface_host(&h.root_ui).invoke_surface_control_clicked(
            "WorkbenchGeneratedBottomModeOutput".into(),
            old_token.clone(),
        );
        callback(
            &h,
            &id,
            current.clone(),
            UiInputEvent::Pointer(pointer(UiPointerEventKind::Down, UiPoint::new(1.0, 1.0))),
        );
        assert_eq!(sequence(&h), baseline);
        assert!(Arc::ptr_eq(&published(&h, &id), &fresh));

        // Hide keeps Surface values while retiring actual focus, IME, capture, frame and action epoch.
        let point = reveal(&h, &id, filter);
        h.root_ui
            .dispatch_ui_asset_normalized_pointer_for_test(pointer(
                UiPointerEventKind::Down,
                point,
            ));
        let held_result = h
            .host
            .borrow()
            .builtin_template_runtime
            .generated_bottom_pane_input_result(&id.0)
            .unwrap();
        let held_source = published(&h, &id);
        assert!(held_source.focus_state.focused.is_some());
        assert!(
            held_source.focus_state.captured.is_some() || held_source.focus_state.pressed.is_some()
        );
        let UiInputEvent::Pointer(held_pointer) = &held_result.event else {
            panic!("actual held press");
        };
        let mut old_up = held_pointer.clone();
        old_up.event = UiPointerEvent::new(UiPointerEventKind::Up, held_pointer.event.point)
            .with_button(UiPointerButton::Primary);
        let preserved = props(&h, &id, filter).get("value").unwrap().clone();
        h.root_ui
            .dispatch_retained_pane_platform_input(UiInputEvent::Ime(UiImeInputEvent {
                metadata: UiInputEventMetadata::default(),
                kind: UiImeInputEventKind::Preedit,
                text: "临时".into(),
                cursor_range: None,
                preedit_clauses: Vec::new(),
                delete_surrounding: None,
            }))
            .expect("actual Generated composition before hiding");
        assert!(has_composition(&published(&h, &id)));
        let active_epoch = h
            .host
            .borrow()
            .builtin_template_runtime
            .generated_bottom_owner_state_for_test(&id.0)
            .unwrap()
            .1;
        let held_tokens = h.root_ui.generated_bottom_pane_for_test(&id.0);
        let held_filter = held_tokens
            .nodes
            .iter()
            .find(|node| node.control_id.as_str() == filter)
            .unwrap();
        let held_edit = held_filter.edit_action_id.clone();
        let held_commit = held_filter.commit_action_id.clone();
        assert!(
            held_edit.as_str().contains("/generated-bottom/e")
                && held_commit.as_str().contains("/generated-bottom/e")
        );
        apply_layout(
            &h,
            LayoutCommand::SetDrawerMode {
                slot: ActivityDrawerSlot::Bottom,
                mode: ActivityDrawerMode::Collapsed,
            },
        );
        let hidden = h
            .host
            .borrow()
            .builtin_template_runtime
            .generated_bottom_owner_state_for_test(&id.0)
            .unwrap();
        assert!(!hidden.0 && hidden.1 != active_epoch);
        assert_eq!(hidden.2.window_state.focused, Some(false));
        assert!(
            hidden.2.focus_state.focused.is_none()
                && hidden.2.focus_state.pressed.is_none()
                && hidden.2.focus_state.captured.is_none()
        );
        assert!(!has_composition(&hidden.2));
        assert_eq!(props(&h, &id, filter).get("value"), Some(&preserved));
        assert!(
            !h.root_ui.retained_pane_input_source_visible(&id.0)
                && !h.root_ui.has_retained_pane_input_address()
        );
        assert!(!h.root_ui.retained_pane_accepts_text_input());
        assert!(h
            .host
            .borrow()
            .builtin_template_runtime
            .generated_bottom_pane_source_receipts_for_test(&id.0)
            .is_none());
        assert!(!h
            .root_ui
            .ui_asset_preview_paint_sources_for_test()
            .iter()
            .any(|(source, _, _)| source.tree_id == hidden.2.tree_id));
        let baseline = sequence(&h);
        assert!(h
            .root_ui
            .dispatch_retained_pane_platform_input(UiInputEvent::Pointer(old_up.clone()))
            .is_none());
        callback(
            &h,
            &id,
            held_source.clone(),
            UiInputEvent::Pointer(old_up.clone()),
        );
        for token in [held_edit.clone(), held_commit.clone()] {
            pane_surface_host(&h.root_ui).invoke_surface_control_edited(
                filter.into(),
                token,
                "rejected".into(),
            );
        }
        assert_eq!(sequence(&h), baseline);
        assert!(Arc::ptr_eq(
            &h.host
                .borrow()
                .builtin_template_runtime
                .generated_bottom_owner_state_for_test(&id.0)
                .unwrap()
                .2,
            &hidden.2
        ));
        assert_eq!(props(&h, &id, filter).get("value"), Some(&preserved));
        h.activate_drawer_tab(ActivityDrawerSlot::Bottom, &id.0);
        let reopened = published(&h, &id);
        assert!(!Arc::ptr_eq(&held_source, &reopened));
        let reopened_epoch = h
            .host
            .borrow()
            .builtin_template_runtime
            .generated_bottom_owner_state_for_test(&id.0)
            .unwrap()
            .1;
        assert_ne!(reopened_epoch, hidden.1);
        assert_eq!(props(&h, &id, filter).get("value"), Some(&preserved));
        callback(&h, &id, held_source, UiInputEvent::Pointer(old_up));
        pane_surface_host(&h.root_ui).invoke_surface_control_edited(
            filter.into(),
            held_commit,
            "rejected".into(),
        );
        assert_eq!(sequence(&h), baseline);
        assert!(Arc::ptr_eq(&published(&h, &id), &reopened));
        let point = reveal(&h, &id, filter);
        h.root_ui
            .dispatch_ui_asset_normalized_pointer_for_test(pointer(
                UiPointerEventKind::Down,
                point,
            ));
        h.root_ui
            .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Up, point));
        assert!(h.root_ui.retained_pane_accepts_text_input());
        assert_eq!(published(&h, &id).window_state.focused, Some(true));
        let final_source = published(&h, &id);
        let final_tokens = h.root_ui.generated_bottom_pane_for_test(&id.0);
        let final_edit = final_tokens
            .nodes
            .iter()
            .find(|node| node.control_id.as_str() == filter)
            .unwrap()
            .edit_action_id
            .clone();
        apply_layout(
            &h,
            LayoutCommand::CloseView {
                instance_id: id.clone(),
            },
        );
        assert!(!h
            .host
            .borrow()
            .builtin_template_runtime
            .owns_generated_bottom_pane(&id.0));
        assert!(h
            .host
            .borrow()
            .builtin_template_runtime
            .generated_bottom_owner_state_for_test(&id.0)
            .is_none());
        assert!(!h.root_ui.has_retained_pane_input_address());
        let baseline = sequence(&h);
        callback(
            &h,
            &id,
            final_source,
            UiInputEvent::Pointer(pointer(UiPointerEventKind::Down, UiPoint::new(1.0, 1.0))),
        );
        pane_surface_host(&h.root_ui).invoke_surface_control_edited(
            filter.into(),
            final_edit,
            "rejected".into(),
        );
        assert_eq!(sequence(&h), baseline);
        assert!(h
            .host
            .borrow()
            .builtin_template_runtime
            .generated_bottom_pane_source_receipts_for_test(&id.0)
            .is_none());
        receipts
    }
}

#[test]
fn generated_bottom_authored_bindings_share_the_native_source_surface_and_retire() {
    let _guard = lock_env();
    UiHostWindow::exercise_generated_bottom_sources_for_test();
}

#[test]
fn generated_bottom_public_floating_attach_retires_old_input_and_republishes_current_source() {
    let _guard = lock_env();
    let h = ChildWindowHostHarness::new("generated_bottom_public_floating_attach");
    h.activate_workbench_page();
    let id = h.open_view("editor.generated_bottom");
    h.activate_drawer_tab(ActivityDrawerSlot::Bottom, &id.0);
    apply_layout(
        &h,
        LayoutCommand::SetDrawerExtent {
            slot: ActivityDrawerSlot::Bottom,
            extent: 600.0,
        },
    );
    let child_id = MainPageId::new("window:generated-input-owner");
    let child =
        h.detach_views_to_child_window(&["editor.assets#1", id.0.as_str()], child_id.0.as_str());
    let filter = "WorkbenchGeneratedBottomFilterField";
    let child_source = child
        .generated_bottom_pane_for_test(&id.0)
        .source_surface_frame
        .expect("actual active Generated source in child A");
    let child_owner = h
        .host
        .borrow()
        .builtin_template_runtime
        .generated_bottom_owner_state_for_test(&id.0)
        .unwrap();
    assert!(child_owner.0 && Arc::ptr_eq(&child_source, &child_owner.2));
    assert!(child
        .ui_asset_preview_paint_sources_for_test()
        .iter()
        .any(|(source, _, rect)| Arc::ptr_eq(source, &child_source)
            && rect.width > 0.0
            && rect.height > 0.0));
    let point = child
        .generated_bottom_control_point_for_test(&id.0, filter)
        .expect("actual visible authored field in child A");
    let node = child_source
        .arranged_tree
        .nodes
        .iter()
        .find(|node| node.control_id.as_deref() == Some(filter))
        .unwrap();
    let local = UiPoint::new(
        (node.frame.x.max(node.clip_frame.x)
            + (node.frame.x + node.frame.width).min(node.clip_frame.x + node.clip_frame.width))
            * 0.5,
        (node.frame.y.max(node.clip_frame.y)
            + (node.frame.y + node.frame.height).min(node.clip_frame.y + node.clip_frame.height))
            * 0.5,
    );
    assert_eq!(
        zircon_runtime::ui::surface::hit_test_surface_frame(&child_source, local).top_hit,
        Some(node.node_id),
        "the old window routes a real hit on the authored field"
    );
    child.dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Down, point));
    assert!(child.has_retained_pane_input_address() && child.retained_pane_accepts_text_input());
    let held_result = h
        .host
        .borrow()
        .builtin_template_runtime
        .generated_bottom_pane_input_result(&id.0)
        .expect("child A owns the actual normalized held press");
    let UiInputEvent::Pointer(held_pointer) = &held_result.event else {
        panic!("actual held child pointer");
    };
    let mut old_up = held_pointer.clone();
    old_up.event = UiPointerEvent::new(UiPointerEventKind::Up, held_pointer.event.point)
        .with_button(UiPointerButton::Primary);
    let preserved = props(&h, &id, filter).get("value").unwrap().clone();
    child
        .dispatch_retained_pane_platform_input(UiInputEvent::Ime(UiImeInputEvent {
            metadata: UiInputEventMetadata::default(),
            kind: UiImeInputEventKind::Preedit,
            text: "交接临时".into(),
            cursor_range: None,
            preedit_clauses: Vec::new(),
            delete_surrounding: None,
        }))
        .expect("actual child IME before the public Attach");
    let held_source = child
        .generated_bottom_pane_for_test(&id.0)
        .source_surface_frame
        .unwrap();
    assert!(held_source.focus_state.focused.is_some());
    assert!(
        held_source.focus_state.captured.is_some() || held_source.focus_state.pressed.is_some()
    );
    assert!(has_composition(&held_source));
    let old_epoch = h
        .host
        .borrow()
        .builtin_template_runtime
        .generated_bottom_owner_state_for_test(&id.0)
        .unwrap()
        .1;
    let held_tokens = child.generated_bottom_pane_for_test(&id.0);
    let held_filter = held_tokens
        .nodes
        .iter()
        .find(|node| node.control_id.as_str() == filter)
        .unwrap();
    let held_edit = held_filter.edit_action_id.clone();
    let held_commit = held_filter.commit_action_id.clone();
    let held_click = held_tokens
        .nodes
        .iter()
        .find(|node| node.control_id.as_str() == "WorkbenchGeneratedBottomModeOutput")
        .unwrap()
        .action_id
        .clone();

    // Keep A alive with Assets while moving its addressed, active pane through the public path.
    let layout_baseline = h.journal_len();
    apply_layout(
        &h,
        LayoutCommand::AttachView {
            instance_id: id.clone(),
            target: crate::ui::workbench::view::ViewHost::Drawer(ActivityDrawerSlot::Bottom),
            anchor: None,
        },
    );
    assert!(h.delta_events_since(layout_baseline).iter().any(|event| {
        matches!(
            event,
            EditorEvent::Layout(EventLayoutCommand::AttachView {
                instance_id: EventViewInstanceId(moved),
                target: crate::core::editor_event::ViewHost::Drawer(EventActivityDrawerSlot::Bottom),
                ..
            }) if moved == &id.0
        )
    }));
    assert!(child.window().is_visible());
    assert!(h
        .host
        .borrow()
        .native_window_presenters
        .window(&child_id)
        .is_some());
    let child_generation = child.get_host_presentation_generation();
    assert!(child_generation
        .structure()
        .native_floating_surface_data
        .floating_windows
        .iter()
        .any(|window| window.window_id.as_str() == child_id.0.as_str()
            && window.active_pane.id.as_str() == "editor.assets#1"));
    assert!(!child.retained_pane_input_source_visible(&id.0));
    assert!(!child.has_retained_pane_input_address());
    assert!(!child.retained_pane_accepts_text_input());

    // This must already be a current holder Source, without a reopen or unrelated refresh.
    let current = published(&h, &id);
    assert_painted(&h, &current);
    let current_owner = h
        .host
        .borrow()
        .builtin_template_runtime
        .generated_bottom_owner_state_for_test(&id.0)
        .unwrap();
    assert_ne!(current_owner.1, old_epoch);
    assert!(!Arc::ptr_eq(&held_source, &current));
    assert_eq!(current.window_state.focused, Some(false));
    assert!(
        current.focus_state.focused.is_none()
            && current.focus_state.pressed.is_none()
            && current.focus_state.captured.is_none()
    );
    assert!(!has_composition(&current));
    assert_eq!(props(&h, &id, filter).get("value"), Some(&preserved));
    assert!(!h.root_ui.has_retained_pane_input_address());
    assert!(h
        .host
        .borrow()
        .builtin_template_runtime
        .generated_bottom_pane_input_result(&id.0)
        .is_none());

    let baseline = sequence(&h);
    assert!(child
        .dispatch_retained_pane_platform_input(UiInputEvent::Pointer(old_up.clone()))
        .is_none());
    // Use the visible destination callback so rejection depends on Source/epoch authority.
    callback(&h, &id, held_source, UiInputEvent::Pointer(old_up));
    for token in [held_edit, held_commit] {
        pane_surface_host(&h.root_ui).invoke_surface_control_edited(
            filter.into(),
            token,
            "rejected".into(),
        );
    }
    pane_surface_host(&h.root_ui)
        .invoke_surface_control_clicked("WorkbenchGeneratedBottomModeOutput".into(), held_click);
    assert_eq!(sequence(&h), baseline);
    assert_eq!(props(&h, &id, filter).get("value"), Some(&preserved));
    assert!(Arc::ptr_eq(&published(&h, &id), &current));
    assert!(h
        .host
        .borrow()
        .builtin_template_runtime
        .generated_bottom_pane_input_result(&id.0)
        .is_none());

    let bindings = h
        .host
        .borrow()
        .builtin_template_runtime
        .generated_bottom_bindings_for_test(&id.0);
    let change = bindings
        .iter()
        .find(|(_, _, kind, _)| *kind == UiEventKind::Change)
        .unwrap();
    let point = reveal(&h, &id, filter);
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Down, point));
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Move, point));
    h.root_ui
        .dispatch_ui_asset_normalized_pointer_for_test(pointer(UiPointerEventKind::Up, point));
    assert!(h.root_ui.retained_pane_accepts_text_input());
    let baseline = sequence(&h);
    h.root_ui
        .dispatch_retained_pane_platform_input(UiInputEvent::Text(UiTextInputEvent {
            metadata: UiInputEventMetadata::default(),
            text: "生成 Ω".into(),
        }))
        .expect("actual current destination text input after the handoff");
    assert_association(&h, &id, &change.0, &change.1, &change.2);
    assert_record(&h, &change.3, baseline);
    assert_feedback(&h, &id, &change.3);
    assert_eq!(
        props(&h, &id, filter).get("value"),
        Some(&UiValue::String("生成 Ω".into()))
    );
    assert_painted(&h, &published(&h, &id));

    let runtime_binding = bindings
        .iter()
        .find(|(_, control, _, _)| control == "WorkbenchGeneratedBottomModeRuntime")
        .unwrap();
    let baseline = click(&h, &id, &runtime_binding.1);
    assert_association(
        &h,
        &id,
        &runtime_binding.0,
        &runtime_binding.1,
        &runtime_binding.2,
    );
    assert_record(&h, &runtime_binding.3, baseline);
    assert_feedback(&h, &id, &runtime_binding.3);
}

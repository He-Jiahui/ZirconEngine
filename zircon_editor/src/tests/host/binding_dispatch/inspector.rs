use serde_json::json;
use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;
use zircon_runtime_interface::ui::binding::UiBindingValue;

use super::support;
use crate::core::editing::engine::HistoryContextId;
use crate::core::editing::intent::EditorIntent;
use crate::core::editor_event::InspectorFieldChange;
use crate::core::editor_message::{EditorMessagePayload, EditorTopic, TransactionMessage};
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use crate::ui::binding_dispatch::apply_inspector_binding;

#[test]
fn inspector_binding_applies_batch_changes_to_editor_state() {
    let mut state = support::test_state();
    let cube = support::cube_id(&state);
    state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();

    let binding = EditorUiBinding::new(
        "InspectorView",
        "ApplyBatchButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::inspector_field_batch(
            "entity://selected",
            vec![
                InspectorFieldChange::new("name", UiBindingValue::string("Bound Cube")),
                InspectorFieldChange::new("parent", UiBindingValue::Null),
                InspectorFieldChange::new("transform.translation.x", UiBindingValue::Float(4.0)),
                InspectorFieldChange::new("transform.translation.y", UiBindingValue::Float(5.0)),
                InspectorFieldChange::new("transform.translation.z", UiBindingValue::Float(6.0)),
            ],
        ),
    );

    assert!(apply_inspector_binding(&mut state, &binding).unwrap());
    state
        .world
        .with_world(|scene: &zircon_runtime::scene::Scene| {
            let node = scene.find_node(cube).unwrap();
            assert_eq!(node.name, "Bound Cube");
            assert_eq!(
                node.transform.translation,
                zircon_runtime_interface::math::Vec3::new(4.0, 5.0, 6.0)
            );
        });
}

#[test]
fn inspector_binding_applies_dynamic_plugin_component_fields_with_undo_history() {
    let mut state = support::test_state();
    let cube = support::cube_id(&state);
    state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();
    state.world.with_world_mut(|scene| {
        scene
            .register_component_type(
                ComponentTypeDescriptor::new(
                    "weather.Component.CloudLayer",
                    "weather",
                    "Cloud Layer",
                )
                .with_property("coverage", "scalar", true),
            )
            .unwrap();
        scene
            .set_dynamic_component(
                cube,
                "weather.Component.CloudLayer",
                json!({ "coverage": 0.25 }),
            )
            .unwrap();
    });

    let binding = EditorUiBinding::new(
        "InspectorView",
        "ApplyBatchButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::inspector_field_batch(
            "entity://selected",
            vec![InspectorFieldChange::new(
                "weather.Component.CloudLayer.coverage",
                UiBindingValue::string("0.90"),
            )],
        ),
    );

    assert!(apply_inspector_binding(&mut state, &binding).unwrap());
    state.world.with_world(|scene| {
        assert_eq!(
            scene.dynamic_component(cube, "weather.Component.CloudLayer"),
            Some(&json!({ "coverage": 0.9 }))
        );
    });

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.with_world(|scene| {
        assert_eq!(
            scene.dynamic_component(cube, "weather.Component.CloudLayer"),
            Some(&json!({ "coverage": 0.25 }))
        );
    });
}

#[test]
fn inspector_binding_rejects_dynamic_plugin_component_field_when_schema_is_unloaded() {
    let mut state = support::test_state();
    let cube = support::cube_id(&state);
    state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();
    state.world.with_world_mut(|scene| {
        scene
            .set_dynamic_component(
                cube,
                "weather.Component.CloudLayer",
                json!({ "coverage": 0.25 }),
            )
            .unwrap();
    });

    let binding = EditorUiBinding::new(
        "InspectorView",
        "ApplyBatchButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::inspector_field_batch(
            "entity://selected",
            vec![InspectorFieldChange::new(
                "weather.Component.CloudLayer.coverage",
                UiBindingValue::string("0.90"),
            )],
        ),
    );

    let error = apply_inspector_binding(&mut state, &binding).unwrap_err();
    assert!(error
        .to_string()
        .contains("unsupported inspector field weather.Component.CloudLayer.coverage"));
}

#[test]
fn inspector_binding_restores_selection_and_draft_after_late_unsupported_field() {
    let mut state = support::test_state();
    let cube = support::cube_id(&state);
    let camera = support::camera_id(&state);
    state
        .apply_intent(EditorIntent::SelectNode(camera))
        .unwrap();
    state
        .viewport_controller
        .selection_mut()
        .replace_active([camera, cube], Some(camera));
    state.bind_transaction_context().unwrap();
    state.update_name_field("Saved Camera Draft".to_string());
    state.update_parent_field("42".to_string());
    state.update_translation_field(0, "1.25".to_string());
    state.update_translation_field(1, "2.50".to_string());
    state.update_translation_field(2, "3.75".to_string());
    state.update_scale_field(0, "4.00".to_string());
    state.update_scale_field(1, "5.00".to_string());
    state.update_scale_field(2, "6.00".to_string());
    state.update_dynamic_component_field("saved.Component.value", "preserved".to_string());
    let selection_before = state.viewport_controller.selection().clone();
    let name_before = state.name_field.to_string();
    let parent_before = state.parent_field.clone();
    let translation_before = state.transform_fields.clone();
    let scale_before = state.scale_fields.clone();
    let dynamic_fields_before = state.inspector_dynamic_fields.clone();
    let orbit_before = state.viewport_controller.orbit_target();
    let status_before = state.status_line.clone();
    let console_before = state.console_output();
    let transaction_selection_before = support::transaction_selection(&state);
    let transaction_selection_snapshot_before = support::transaction_selection_snapshot(&state);

    let binding = EditorUiBinding::new(
        "InspectorView",
        "ApplyBatchButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::inspector_field_batch(
            format!("node://{cube}"),
            vec![
                InspectorFieldChange::new("name", UiBindingValue::string("Changed Cube")),
                InspectorFieldChange::new(
                    "unsupported.Component.field",
                    UiBindingValue::string("invalid"),
                ),
            ],
        ),
    );

    let error = apply_inspector_binding(&mut state, &binding).unwrap_err();
    assert!(error
        .to_string()
        .contains("unsupported inspector field unsupported.Component.field"));
    assert_eq!(state.viewport_controller.selection(), &selection_before);
    assert_eq!(state.name_field, name_before);
    assert_eq!(state.parent_field, parent_before);
    assert_eq!(state.transform_fields, translation_before);
    assert_eq!(state.scale_fields, scale_before);
    assert_eq!(state.inspector_dynamic_fields, dynamic_fields_before);
    assert_eq!(state.viewport_controller.orbit_target(), orbit_before);
    assert_eq!(state.status_line, status_before);
    assert_eq!(state.console_output(), console_before);
    assert_eq!(
        support::transaction_selection(&state),
        transaction_selection_before
    );
    assert_eq!(
        support::transaction_selection_snapshot(&state),
        transaction_selection_snapshot_before
    );
}

#[test]
fn inspector_binding_restores_stale_core_selection_after_failure() {
    let mut state = support::test_state();
    let cube = support::cube_id(&state);
    let camera = support::camera_id(&state);
    state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();
    state.bind_transaction_context().unwrap();
    let transaction_selection_snapshot_before = support::transaction_selection_snapshot(&state);
    assert_eq!(support::transaction_selection(&state).primary(), Some(cube));

    state
        .apply_intent(EditorIntent::SelectNode(camera))
        .unwrap();
    assert_eq!(
        state.viewport_controller.selection().active_primary(),
        Some(camera)
    );
    assert_eq!(
        support::transaction_selection_snapshot(&state),
        transaction_selection_snapshot_before
    );
    state.update_name_field("Saved Camera Draft".to_string());
    state.update_parent_field("42".to_string());
    state.update_translation_field(0, "1.25".to_string());
    state.update_translation_field(1, "2.50".to_string());
    state.update_translation_field(2, "3.75".to_string());
    state.update_scale_field(0, "4.00".to_string());
    state.update_scale_field(1, "5.00".to_string());
    state.update_scale_field(2, "6.00".to_string());
    state.update_dynamic_component_field("saved.Component.value", "preserved".to_string());
    let selection_before = state.viewport_controller.selection().clone();
    let name_before = state.name_field.to_string();
    let parent_before = state.parent_field.clone();
    let translation_before = state.transform_fields.clone();
    let scale_before = state.scale_fields.clone();
    let dynamic_fields_before = state.inspector_dynamic_fields.clone();
    let orbit_before = state.viewport_controller.orbit_target();
    let status_before = state.status_line.clone();
    let console_before = state.console_output();

    let binding = EditorUiBinding::new(
        "InspectorView",
        "ApplyBatchButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::inspector_field_batch(
            format!("node://{cube}"),
            vec![
                InspectorFieldChange::new("name", UiBindingValue::string("Changed Cube")),
                InspectorFieldChange::new(
                    "unsupported.Component.field",
                    UiBindingValue::string("invalid"),
                ),
            ],
        ),
    );

    let error = apply_inspector_binding(&mut state, &binding).unwrap_err();
    assert!(error
        .to_string()
        .contains("unsupported inspector field unsupported.Component.field"));
    assert_eq!(state.viewport_controller.selection(), &selection_before);
    assert_eq!(state.name_field, name_before);
    assert_eq!(state.parent_field, parent_before);
    assert_eq!(state.transform_fields, translation_before);
    assert_eq!(state.scale_fields, scale_before);
    assert_eq!(state.inspector_dynamic_fields, dynamic_fields_before);
    assert_eq!(state.viewport_controller.orbit_target(), orbit_before);
    assert_eq!(state.status_line, status_before);
    assert_eq!(state.console_output(), console_before);
    assert_eq!(
        support::transaction_selection_snapshot(&state),
        transaction_selection_snapshot_before
    );
}

#[test]
fn inspector_binding_restores_every_boundary_after_late_transaction_application_failure() {
    let mut state = support::test_state();
    let cube = support::cube_id(&state);
    let camera = support::camera_id(&state);
    state
        .apply_intent(EditorIntent::SelectNode(camera))
        .unwrap();
    state
        .viewport_controller
        .selection_mut()
        .replace_active([camera, cube], Some(camera));
    state.bind_transaction_context().unwrap();
    state.update_name_field("Saved Camera Draft".to_string());
    state.update_parent_field("42".to_string());
    state.update_translation_field(0, "1.25".to_string());
    state.update_translation_field(1, "2.50".to_string());
    state.update_translation_field(2, "3.75".to_string());
    state.update_scale_field(0, "4.00".to_string());
    state.update_scale_field(1, "5.00".to_string());
    state.update_scale_field(2, "6.00".to_string());
    let selection_before = state.viewport_controller.selection().clone();
    let name_before = state.name_field.to_string();
    let parent_before = state.parent_field.clone();
    let translation_before = state.transform_fields.clone();
    let scale_before = state.scale_fields.clone();
    let dynamic_fields_before = state.inspector_dynamic_fields.clone();
    let orbit_before = state.viewport_controller.orbit_target();
    let status_before = state.status_line.clone();
    let console_before = state.console_output();
    let transaction_selection_before = support::transaction_selection(&state);
    let transaction_selection_snapshot_before = support::transaction_selection_snapshot(&state);
    let history_before = state
        .transactions()
        .history_generation_snapshot(HistoryContextId::Global)
        .unwrap();
    let history_status_before = state
        .transactions()
        .history_status(HistoryContextId::Global)
        .unwrap();
    let world_before = state.world.snapshot();
    let transaction_subscriber = state
        .context
        .bus()
        .register_subscriber([EditorTopic::transaction()])
        .unwrap();

    let binding = EditorUiBinding::new(
        "InspectorView",
        "ApplyBatchButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::inspector_field_batch(
            format!("node://{cube}"),
            vec![
                InspectorFieldChange::new("name", UiBindingValue::string("Changed Cube")),
                InspectorFieldChange::new("parent", UiBindingValue::string("999999")),
            ],
        ),
    );

    let error = apply_inspector_binding(&mut state, &binding).unwrap_err();
    assert!(error.to_string().contains("missing parent 999999"));
    let lifecycle = state
        .context
        .bus()
        .drain_deliveries(transaction_subscriber)
        .into_iter()
        .map(|delivery| match delivery.message().payload() {
            EditorMessagePayload::Transaction(message) => message.clone(),
            payload => panic!("expected transaction event, received {payload:?}"),
        })
        .collect::<Vec<_>>();
    assert!(matches!(
        lifecycle.as_slice(),
        [
            TransactionMessage::Started {
                transaction: started_transaction,
                label: started_label,
                ..
            },
            TransactionMessage::Canceled {
                transaction: canceled_transaction,
                label: canceled_label,
                ..
            },
        ] if started_transaction == canceled_transaction
            && started_label == "Apply inspector changes"
            && canceled_label == "Apply inspector changes"
    ));
    assert_eq!(state.viewport_controller.selection(), &selection_before);
    assert_eq!(state.name_field, name_before);
    assert_eq!(state.parent_field, parent_before);
    assert_eq!(state.transform_fields, translation_before);
    assert_eq!(state.scale_fields, scale_before);
    assert_eq!(state.inspector_dynamic_fields, dynamic_fields_before);
    assert_eq!(state.viewport_controller.orbit_target(), orbit_before);
    assert_eq!(state.status_line, status_before);
    assert_eq!(state.console_output(), console_before);
    assert_eq!(
        support::transaction_selection(&state),
        transaction_selection_before
    );
    assert_eq!(
        support::transaction_selection_snapshot(&state),
        transaction_selection_snapshot_before
    );
    assert_eq!(
        state
            .transactions()
            .history_generation_snapshot(HistoryContextId::Global)
            .unwrap(),
        history_before
    );
    assert_eq!(
        state
            .transactions()
            .history_status(HistoryContextId::Global)
            .unwrap(),
        history_status_before
    );
    assert_eq!(state.world.snapshot(), world_before);
}

#[test]
fn inspector_binding_cancels_post_apply_selection_sync_failure_before_history_commit() {
    let mut state = support::test_state();
    let cube = support::cube_id(&state);
    let camera = support::camera_id(&state);
    state
        .apply_intent(EditorIntent::SelectNode(camera))
        .unwrap();
    state.bind_transaction_context().unwrap();
    state.update_name_field("Saved Camera Draft".to_string());
    state.update_parent_field("42".to_string());
    state.update_translation_field(0, "1.25".to_string());
    state.update_translation_field(1, "2.50".to_string());
    state.update_translation_field(2, "3.75".to_string());
    state.update_scale_field(0, "4.00".to_string());
    state.update_scale_field(1, "5.00".to_string());
    state.update_scale_field(2, "6.00".to_string());
    state.update_dynamic_component_field("saved.Component.value", "preserved".to_string());
    let selection_before = state.viewport_controller.selection().clone();
    let name_before = state.name_field.to_string();
    let parent_before = state.parent_field.clone();
    let translation_before = state.transform_fields.clone();
    let scale_before = state.scale_fields.clone();
    let dynamic_fields_before = state.inspector_dynamic_fields.clone();
    let orbit_before = state.viewport_controller.orbit_target();
    let status_before = state.status_line.clone();
    let console_before = state.console_output();
    let transaction_selection_before = support::transaction_selection(&state);
    let transaction_selection_snapshot_before = support::transaction_selection_snapshot(&state);
    let history_before = state
        .transactions()
        .history_generation_snapshot(HistoryContextId::Global)
        .unwrap();
    let history_status_before = state
        .transactions()
        .history_status(HistoryContextId::Global)
        .unwrap();
    let world_before = state.world.snapshot();
    let transaction_subscriber = state
        .context
        .bus()
        .register_subscriber([EditorTopic::transaction()])
        .unwrap();
    state.fail_next_transaction_selection_sync_for_test();

    let binding = EditorUiBinding::new(
        "InspectorView",
        "ApplyBatchButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::inspector_field_batch(
            format!("node://{cube}"),
            [InspectorFieldChange::new(
                "name",
                UiBindingValue::string("Changed Cube"),
            )],
        ),
    );

    let error = apply_inspector_binding(&mut state, &binding).unwrap_err();
    assert!(error
        .to_string()
        .contains("forced transaction selection synchronization failure"));
    let lifecycle = state
        .context
        .bus()
        .drain_deliveries(transaction_subscriber)
        .into_iter()
        .map(|delivery| match delivery.message().payload() {
            EditorMessagePayload::Transaction(message) => message.clone(),
            payload => panic!("expected transaction event, received {payload:?}"),
        })
        .collect::<Vec<_>>();
    assert!(matches!(
        lifecycle.as_slice(),
        [
            TransactionMessage::Started {
                transaction: started_transaction,
                label: started_label,
                ..
            },
            TransactionMessage::Canceled {
                transaction: canceled_transaction,
                label: canceled_label,
                ..
            },
        ] if started_transaction == canceled_transaction
            && started_label == "Apply inspector changes"
            && canceled_label == "Apply inspector changes"
    ));
    assert_eq!(state.viewport_controller.selection(), &selection_before);
    assert_eq!(state.name_field, name_before);
    assert_eq!(state.parent_field, parent_before);
    assert_eq!(state.transform_fields, translation_before);
    assert_eq!(state.scale_fields, scale_before);
    assert_eq!(state.inspector_dynamic_fields, dynamic_fields_before);
    assert_eq!(state.viewport_controller.orbit_target(), orbit_before);
    assert_eq!(state.status_line, status_before);
    assert_eq!(state.console_output(), console_before);
    assert_eq!(
        support::transaction_selection(&state),
        transaction_selection_before
    );
    assert_eq!(
        support::transaction_selection_snapshot(&state),
        transaction_selection_snapshot_before
    );
    assert_eq!(
        state
            .transactions()
            .history_generation_snapshot(HistoryContextId::Global)
            .unwrap(),
        history_before
    );
    assert_eq!(
        state
            .transactions()
            .history_status(HistoryContextId::Global)
            .unwrap(),
        history_status_before
    );
    assert_eq!(state.world.snapshot(), world_before);
}

#[test]
fn inspector_binding_rejects_an_active_gizmo_without_cancelling_its_preview() {
    let mut state = support::test_state();
    let cube = support::cube_id(&state);
    let camera = support::camera_id(&state);
    state
        .apply_intent(EditorIntent::SelectNode(camera))
        .unwrap();
    state.bind_transaction_context().unwrap();
    assert!(state.begin_gizmo_transaction().unwrap());
    assert!(state.has_active_gizmo_interaction());
    let selection_before = state.viewport_controller.selection().clone();
    let world_before = state.world.snapshot();
    let orbit_before = state.viewport_controller.orbit_target();
    let status_before = state.status_line.clone();
    let console_before = state.console_output();
    let transaction_selection_snapshot_before = support::transaction_selection_snapshot(&state);

    let binding = EditorUiBinding::new(
        "InspectorView",
        "ApplyBatchButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::inspector_field_batch(
            format!("node://{cube}"),
            [InspectorFieldChange::new(
                "transform.translation.x",
                UiBindingValue::Float(42.0),
            )],
        ),
    );

    let error = apply_inspector_binding(&mut state, &binding).unwrap_err();
    assert!(error
        .to_string()
        .contains("cannot apply inspector changes while a gizmo interaction is active"));
    assert!(state.has_active_gizmo_interaction());
    assert_eq!(state.viewport_controller.selection(), &selection_before);
    assert_eq!(state.world.snapshot(), world_before);
    assert_eq!(state.viewport_controller.orbit_target(), orbit_before);
    assert_eq!(state.status_line, status_before);
    assert_eq!(state.console_output(), console_before);
    assert_eq!(
        support::transaction_selection_snapshot(&state),
        transaction_selection_snapshot_before
    );
}

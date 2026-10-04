use std::sync::{Arc, Mutex};

use zircon_editor::core::editor_event::{EditorEvent, EditorViewportEvent};
use zircon_editor::core::editor_extension::EditorExtensionRegistry;
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_editor::EditorPlugin;
use zircon_plugin_physics_runtime::PhysicsOverlayFrame;
use zircon_runtime::core::framework::physics::{PhysicsColliderShape, PhysicsColliderSyncState};
use zircon_runtime::core::framework::render::SceneGizmoKind;
use zircon_runtime::core::math::{Transform, Vec3};

use crate::runtime_mirror::{PhysicsPieMirror, PhysicsPieMirrorApply};
use crate::viewport_overlay_provider::{
    physics_viewport_overlay_provider_registration, PhysicsViewportOverlayProvider,
};
use crate::{
    editor_plugin, PHYSICS_AUTHORING_CAPABILITY, PHYSICS_OVERLAY_CONSUMER_ID,
    PHYSICS_OVERLAY_FRAME_EVENT_ID, PHYSICS_OVERLAY_FRAME_PAYLOAD_SCHEMA,
    PHYSICS_OVERLAY_PROVIDER_ID,
};

#[test]
fn physics_provider_extracts_canonical_pie_colliders_and_clears_on_session_end() {
    let mirror = Arc::new(Mutex::new(PhysicsPieMirror::default()));
    {
        let mut state = mirror.lock().unwrap();
        state.begin_session(17);
        assert_eq!(
            state.apply_overlay_frame(17, 1, overlay_frame()),
            PhysicsPieMirrorApply::Applied
        );
    }

    let registration = physics_viewport_overlay_provider_registration(mirror.clone());
    assert_eq!(registration.provider_id(), PHYSICS_OVERLAY_PROVIDER_ID);
    assert_eq!(
        registration.required_capabilities(),
        &[PHYSICS_AUTHORING_CAPABILITY.to_string()]
    );

    let provider = PhysicsViewportOverlayProvider::new(mirror.clone());
    let extracts = provider.extract_current(Some(41));
    assert_eq!(extracts.len(), 2);
    assert_eq!(extracts[0].owner, 41);
    assert_eq!(extracts[0].kind, SceneGizmoKind::Physics);
    assert!(extracts[0].selected);
    assert_eq!(extracts[0].lines.len(), 12);
    assert_eq!(extracts[0].pick_shapes.len(), extracts[0].lines.len());
    assert_eq!(extracts[1].lines.len(), 48);
    assert_eq!(extracts[1].pick_shapes.len(), extracts[1].lines.len());
    assert_ne!(extracts[0].lines[0].color, extracts[1].lines[0].color);

    assert!(mirror.lock().unwrap().end_session(17));
    assert!(provider.extract_current(Some(41)).is_empty());
}

#[test]
fn physics_pie_mirror_rejects_cross_session_stale_sequence_and_stale_generation() {
    let mut mirror = PhysicsPieMirror::default();
    mirror.begin_session(12);
    assert_eq!(
        mirror.apply_overlay_frame(12, 2, overlay_frame()),
        PhysicsPieMirrorApply::Applied
    );
    assert_eq!(
        mirror.apply_overlay_frame(12, 1, PhysicsOverlayFrame::default()),
        PhysicsPieMirrorApply::Stale
    );
    assert_eq!(
        mirror.apply_overlay_frame(99, 3, PhysicsOverlayFrame::default()),
        PhysicsPieMirrorApply::WrongSession
    );
    assert_eq!(
        mirror.apply_overlay_frame(
            12,
            3,
            PhysicsOverlayFrame {
                owner_generation: 8,
                ..PhysicsOverlayFrame::default()
            },
        ),
        PhysicsPieMirrorApply::StaleOwnerGeneration
    );
    assert_eq!(mirror.sequence(), Some(2));
    assert_eq!(mirror.owner_generation(), Some(9));
}

#[test]
fn physics_editor_declaration_projects_the_runtime_overlay_consumer() {
    let plugin = editor_plugin();
    let report = plugin.registration_report();
    assert!(report.is_success(), "{:?}", report.diagnostics);
    let manifest = report
        .runtime_event_consumers
        .registration(PHYSICS_OVERLAY_CONSUMER_ID)
        .expect("physics overlay consumer registration")
        .manifest();
    assert_eq!(manifest.event_id, PHYSICS_OVERLAY_FRAME_EVENT_ID);
    assert_eq!(
        manifest.payload_schema,
        PHYSICS_OVERLAY_FRAME_PAYLOAD_SCHEMA
    );
    assert_eq!(manifest.required_capability, PHYSICS_AUTHORING_CAPABILITY);
}

#[test]
fn physics_overlay_operation_routes_to_the_shared_provider_toggle() {
    let plugin = editor_plugin();
    let mut registry = EditorExtensionRegistry::default();
    plugin
        .register_editor_extensions(&mut registry)
        .expect("physics plugin registers its provider and command");
    let operation = EditorOperationPath::parse(crate::PHYSICS_TOGGLE_OVERLAY_OPERATION).unwrap();
    let event = registry
        .pending_command(&operation)
        .and_then(|command| command.event());
    assert_eq!(
        event,
        Some(&EditorEvent::Viewport(
            EditorViewportEvent::ToggleOverlayProvider {
                provider_id: PHYSICS_OVERLAY_PROVIDER_ID.to_owned(),
            },
        ))
    );
}

fn overlay_frame() -> PhysicsOverlayFrame {
    PhysicsOverlayFrame {
        owner_generation: 9,
        colliders: vec![
            collider(
                41,
                false,
                PhysicsColliderShape::Box {
                    half_extents: [0.5, 1.0, 0.25],
                },
            ),
            collider(73, true, PhysicsColliderShape::Sphere { radius: 0.75 }),
        ],
    }
}

fn collider(entity: u64, sensor: bool, shape: PhysicsColliderShape) -> PhysicsColliderSyncState {
    PhysicsColliderSyncState {
        entity,
        shape,
        sensor,
        layer: 0,
        collision_group: u32::MAX,
        collision_mask: u32::MAX,
        material: None,
        material_override: None,
        transform: Transform::from_translation(Vec3::new(entity as f32, 0.0, 0.0)),
    }
}

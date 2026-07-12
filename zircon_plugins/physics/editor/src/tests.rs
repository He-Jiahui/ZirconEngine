use super::*;

#[test]
fn physics_editor_plugin_contributes_authoring_extensions() {
    let registration = plugin_registration();

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration
        .capabilities
        .contains(&PHYSICS_AUTHORING_CAPABILITY.to_string()));
    assert_eq!(
        editor_plugin().declaration().mirrored_runtime_package_id(),
        Some(PLUGIN_ID)
    );
    assert!(registration
        .package_manifest
        .capabilities
        .contains(&zircon_plugin_physics_runtime::PHYSICS_RUNTIME_CAPABILITY.to_string()));
    assert!(registration
        .package_manifest
        .capabilities
        .contains(&PHYSICS_AUTHORING_CAPABILITY.to_string()));
    assert!(registration
        .extensions
        .views()
        .iter()
        .any(|view| view.id() == PHYSICS_AUTHORING_VIEW_ID));
    assert!(registration
        .extensions
        .drawers()
        .iter()
        .any(|drawer| drawer.id() == PHYSICS_DRAWER_ID));
    assert!(registration
        .extensions
        .ui_templates()
        .iter()
        .any(|template| template.id() == PHYSICS_TEMPLATE_ID));
    assert!(registration
        .extensions
        .menu_items()
        .iter()
        .any(|menu| menu.operation().as_str() == "view.physics.authoring.open"));
    assert!(registration
        .extensions
        .command_ids()
        .any(|operation| operation.as_str() == "view.physics.authoring.open"));
}

#[test]
fn overlay_registration_snapshot_matches_physics_debug_contract() {
    let registration = plugin_registration();

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration
        .extensions
        .viewport_tool_modes()
        .iter()
        .any(|mode| {
            mode.id() == PHYSICS_DEBUG_OVERLAY_MODE_ID
                && mode.view_id() == PHYSICS_DEBUG_VIEW_ID
                && mode.activate_operation().as_str() == PHYSICS_TOGGLE_OVERLAY_OPERATION
        }));
    assert!(registration.extensions.menu_items().iter().any(|menu| {
        menu.path() == "View/Debug Overlays/Physics"
            && menu.operation().as_str() == PHYSICS_TOGGLE_OVERLAY_OPERATION
    }));
    assert!(registration
        .extensions
        .views()
        .iter()
        .any(|view| view.id() == PHYSICS_DIAGNOSTICS_VIEW_ID));
}

#[test]
fn generated_profile_covers_all_mapped_bones() {
    use zircon_runtime::core::math::{Transform, Vec3};

    let profile = generate_initial_ragdoll_profile(
        "hero",
        &[
            RagdollSkeletonBone::new("Armature/Hips", None, Transform::default()),
            RagdollSkeletonBone::new(
                "Armature/Hips/Spine",
                Some("Armature/Hips"),
                Transform::from_translation(Vec3::new(0.0, 0.6, 0.0)),
            ),
            RagdollSkeletonBone::new(
                "Armature/Hips/Spine/Head",
                Some("Armature/Hips/Spine"),
                Transform::from_translation(Vec3::new(0.0, 0.35, 0.0)),
            ),
        ],
    )
    .expect("valid skeleton should generate a profile");

    assert_eq!(profile.id, "hero");
    assert_eq!(profile.bones.len(), 3);
    assert_eq!(
        profile
            .bones
            .iter()
            .map(|bone| bone.bone_path.as_str())
            .collect::<Vec<_>>(),
        [
            "Armature/Hips",
            "Armature/Hips/Spine",
            "Armature/Hips/Spine/Head"
        ]
    );
    assert!(profile.validate().is_ok());

    let registration = plugin_registration();
    assert!(registration
        .extensions
        .asset_editors()
        .iter()
        .any(|editor| editor.asset_kind() == RAGDOLL_PROFILE_ASSET_KIND));
}

#[test]
fn physics_overlay_colors_triggers_separately_from_solid_colliders() {
    use zircon_runtime::core::framework::physics::{
        PhysicsColliderShape, PhysicsColliderSyncState, PhysicsWorldSyncState,
    };
    use zircon_runtime::core::math::Transform;

    let mut sync = PhysicsWorldSyncState::default();
    for (entity, sensor) in [(7, false), (8, true)] {
        sync.colliders.push(PhysicsColliderSyncState {
            entity,
            shape: PhysicsColliderShape::Sphere { radius: 0.5 },
            sensor,
            layer: 0,
            collision_group: u32::MAX,
            collision_mask: u32::MAX,
            material: None,
            material_override: None,
            transform: Transform::default(),
        });
    }

    let overlay = build_physics_overlay(&sync);
    assert_eq!(overlay.len(), 2);
    assert_eq!(overlay[0].color, PhysicsOverlayColor::Collider);
    assert_eq!(overlay[1].color, PhysicsOverlayColor::Trigger);
}

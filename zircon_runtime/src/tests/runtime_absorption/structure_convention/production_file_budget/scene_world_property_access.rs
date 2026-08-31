use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_scene_world_property_access_physics_writes_are_child_owner() {
    let parent = read_runtime_src("scene/world/property_access/write.rs");
    let animation = read_runtime_src("scene/world/property_access/write/animation.rs");
    let camera = read_runtime_src("scene/world/property_access/write/camera.rs");
    let lighting = read_runtime_src("scene/world/property_access/write/lighting.rs");
    let mesh = read_runtime_src("scene/world/property_access/write/mesh.rs");
    let physics = read_runtime_src("scene/world/property_access/write/physics.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ecs_doc = read_repo("docs/zircon_runtime/scene/ecs.md");

    assert_contains_all(
        "property-access write parent keeps dispatch and non-physics writers",
        &parent,
        &[
            "mod animation;",
            "mod camera;",
            "mod lighting;",
            "mod mesh;",
            "mod physics;",
            "pub fn set_property",
            "fn set_transform_property",
            "\"camera\" => self.set_camera_property",
            "self.set_mesh_renderer_property(entity, &segments, value, property_path)",
            "self.set_ambient_light_property(entity, &segments, value, property_path)",
            "self.set_animation_player_property(entity, &segments, value, property_path)",
            "\"rigidbody\" => self.set_rigid_body_property(entity, &segments, value, property_path),",
            "\"collider\" => self.set_collider_property(entity, &segments, value, property_path),",
            "\"joint\" => self.set_joint_property(entity, &segments, value, property_path),",
            ".set_dynamic_component_property(entity, property_path, value)",
        ],
    );
    for moved_owner in [
        "fn set_rigid_body_property",
        "fn set_collider_property",
        "fn set_joint_property",
        "ColliderShape",
        "parse_rigid_body_type",
        "parse_combine_rule",
        "parse_joint_kind",
        "normalized_identifier_matches",
        "Vec3::splat(0.5)",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "scene/world/property_access/write.rs should delegate {moved_owner} to write/physics.rs"
        );
    }
    for moved_owner in [
        "CameraComponent",
        "MeshRenderer",
        "AmbientLight",
        "DirectionalLight",
        "PointLight",
        "RectLight",
        "SpotLight",
        "AnimationPlayerComponent",
        "AnimationSequencePlayerComponent",
        "AnimationGraphPlayerComponent",
        "AnimationStateMachinePlayerComponent",
        "SceneError::ReadOnlyProperty",
        "SceneError::InvalidPropertyIndex",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "scene/world/property_access/write.rs should delegate {moved_owner} to a component-domain child owner"
        );
    }
    assert_contains_all(
        "animation child owns animation component write branches",
        &animation,
        &[
            "pub(super) fn set_animation_skeleton_property",
            "pub(super) fn set_animation_player_property",
            "pub(super) fn set_animation_sequence_player_property",
            "pub(super) fn set_animation_graph_player_property",
            "pub(super) fn set_animation_state_machine_player_property",
            "set_animation_player_like_property",
            "let next = if next.is_empty() { None } else { Some(next) };",
        ],
    );
    assert_contains_all(
        "camera child owns camera write branches",
        &camera,
        &["pub(super) fn set_camera_property", "CameraComponent"],
    );
    assert_contains_all(
        "lighting child owns concrete light write branches",
        &lighting,
        &[
            "pub(super) fn set_ambient_light_property",
            "pub(super) fn set_directional_light_property",
            "pub(super) fn set_point_light_property",
            "pub(super) fn set_rect_light_property",
            "pub(super) fn set_spot_light_property",
        ],
    );
    assert_contains_all(
        "mesh child owns mesh renderer write branches and typed read-only errors",
        &mesh,
        &[
            "pub(super) fn set_mesh_renderer_property",
            "MeshRenderer",
            "SceneError::ReadOnlyProperty",
            "SceneError::InvalidPropertyIndex",
        ],
    );
    assert_contains_all(
        "physics child owns rigid body, collider, and joint write branches",
        &physics,
        &[
            "pub(super) fn set_rigid_body_property",
            "pub(super) fn set_collider_property",
            "pub(super) fn set_joint_property",
            "parse_rigid_body_type",
            "parse_combine_rule",
            "parse_joint_kind",
            "normalized_identifier_matches",
            "ColliderShape::Box",
            "ColliderShape::Sphere",
            "ColliderShape::Capsule",
            ".set_dynamic_component_property(entity, property_path, value)",
            "self.mark_node_cache_dirty()",
        ],
    );

    for (path, source) in [
        ("scene/world/property_access/write.rs", parent.as_str()),
        (
            "scene/world/property_access/write/animation.rs",
            animation.as_str(),
        ),
        (
            "scene/world/property_access/write/camera.rs",
            camera.as_str(),
        ),
        (
            "scene/world/property_access/write/lighting.rs",
            lighting.as_str(),
        ),
        ("scene/world/property_access/write/mesh.rs", mesh.as_str()),
        (
            "scene/world/property_access/write/physics.rs",
            physics.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("scene ECS doc", ecs_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 scene world property-access physics write owner split",
                "runtime_15_scene_world_property_access_physics_owner_split_static_passed_cargo_timeout_no_result",
                "scene/world/property_access/write.rs",
                "scene/world/property_access/write/physics.rs",
                "runtime_15_scene_world_property_access_physics_writes_are_child_owner",
            ],
        );
    }
}

#[test]
fn runtime_15_scene_world_property_access_physics_entries_are_child_owner() {
    let parent = read_runtime_src("scene/world/property_access/entries.rs");
    let physics = read_runtime_src("scene/world/property_access/entries/physics.rs");
    let collider_shape = read_runtime_src("scene/world/property_access/entries/collider_shape.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ecs_doc = read_repo("docs/zircon_runtime/scene/ecs.md");

    assert_contains_all(
        "property-access entry parent keeps traversal, non-physics projection, and capacity routing",
        &parent,
        &[
            "mod physics;",
            "pub(super) fn property_entries",
            "visit_physics_property_entries(entity, &mut visitor)",
            "fn property_entry_capacity_hint",
            "physics_property_entry_capacity_hint(entity)",
            "fn dynamic_scene_value_from_json",
        ],
    );
    for moved_owner in [
        "RigidBody.kind",
        "Collider.sensor",
        "Collider.shape.kind",
        "Joint.kind",
        "ColliderShape",
        "RigidBodyType",
        "JointKind",
        "combine_rule_label",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "scene/world/property_access/entries.rs should delegate {moved_owner} to entries/physics.rs"
        );
    }
    assert_contains_all(
        "physics child owns rigid body, collider dispatch, and joint property-entry projection",
        &physics,
        &[
            "pub(super) fn visit_physics_property_entries",
            "pub(super) fn physics_property_entry_capacity_hint",
            "RigidBody.kind",
            "Collider.sensor",
            "Joint.kind",
            "visit_collider_shape_property_entries",
            "combine_rule_label",
            "RigidBodyType::Kinematic",
            "JointKind::Generic6Dof",
        ],
    );
    assert_contains_all(
        "collider-shape child owns shape-specific property-entry projection",
        &collider_shape,
        &[
            "ColliderShape::Box",
            "ColliderShape::Sphere",
            "ColliderShape::Capsule",
            r#"let path = format!("{prefix}.{}", $suffix);"#,
            r#"push_shape_entry!("kind""#,
        ],
    );

    for (path, source) in [
        ("scene/world/property_access/entries.rs", parent.as_str()),
        (
            "scene/world/property_access/entries/physics.rs",
            physics.as_str(),
        ),
        (
            "scene/world/property_access/entries/collider_shape.rs",
            collider_shape.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("scene ECS doc", ecs_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 scene world property-access physics entry owner split",
                "runtime_15_scene_world_property_access_physics_entries_owner_split_static_passed_cargo_lock_blocked",
                "scene/world/property_access/entries.rs",
                "scene/world/property_access/entries/physics.rs",
                "runtime_15_scene_world_property_access_physics_entries_are_child_owner",
            ],
        );
    }
}

use std::fs;

use crate::asset::{
    AssetReference, AssetUri, SceneAsset, SceneCameraTargetAsset, SceneEntityAsset,
    SceneMeshInstanceAsset, SceneMeshPrimitiveBindingAsset, SceneMobilityAsset, TransformAsset,
};
use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::physics::{PhysicsCombineRule, PhysicsMaterialMetadata};
use crate::core::framework::render::{
    ProjectionMode, RenderCameraClearColor, RenderCameraTarget, RenderViewportRect,
};
use crate::core::math::{UVec2, Vec2, Vec3};
use crate::scene::components::{
    AmbientLight, CameraComponent, ColliderShape, JointKind, RectLight, RigidBodyType,
};

use crate::scene::components::NodeKind;
use crate::scene::world::World;

use super::support::{
    create_test_project, project_animation_clip_handle, project_animation_graph_handle,
    project_animation_sequence_handle, project_animation_skeleton_handle,
    project_animation_state_machine_handle, project_material_handle, project_mesh_handle,
    project_model_handle, project_physics_material_handle, unique_temp_project_root,
};

#[test]
fn scene_assets_instantiate_world_with_asset_bound_meshes() {
    let root = unique_temp_project_root("scene_asset");
    let project = create_test_project(&root);
    let world = World::load_scene_from_uri(
        &project,
        &AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
    )
    .unwrap();

    let mesh_node = world
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, NodeKind::Mesh))
        .unwrap();
    let mesh = mesh_node.mesh.as_ref().unwrap();
    assert_eq!(
        mesh.model,
        project_model_handle(&project, "res://models/triangle.obj")
    );
    assert_eq!(
        mesh.mesh,
        Some(project_mesh_handle(&project, "res://meshes/triangle.zmesh"))
    );
    assert_eq!(
        mesh.material,
        project_material_handle(&project, "res://materials/grid.zmaterial")
    );
    assert!(mesh.primitives.is_empty());

    let saved = world.to_scene_asset(&project).unwrap();
    let saved_mesh = saved
        .entities
        .iter()
        .find_map(|entity| entity.mesh.as_ref())
        .unwrap();
    assert_eq!(saved_mesh.model.to_string(), "res://models/triangle.obj");
    assert_eq!(
        saved_mesh.mesh.as_ref().map(ToString::to_string),
        Some("res://meshes/triangle.zmesh".to_string())
    );
    assert_eq!(
        saved_mesh.material.to_string(),
        "res://materials/grid.zmaterial"
    );
    assert!(saved_mesh.primitives.is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn render_extract_keeps_asset_bound_meshes_without_editor_selection_overlay() {
    let root = unique_temp_project_root("scene_gizmo");
    let project = create_test_project(&root);
    let world = World::load_scene_from_uri(
        &project,
        &AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
    )
    .unwrap();
    let mesh_node = world
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, NodeKind::Mesh))
        .unwrap()
        .id;

    let extract = world.to_render_extract();
    let mesh = extract
        .scene
        .meshes
        .iter()
        .find(|mesh| mesh.node_id == mesh_node)
        .unwrap();
    assert_eq!(
        mesh.model,
        project_model_handle(&project, "res://models/triangle.obj")
    );
    assert_eq!(
        mesh.mesh,
        Some(project_mesh_handle(&project, "res://meshes/triangle.zmesh"))
    );
    assert_eq!(
        mesh.material,
        project_material_handle(&project, "res://materials/grid.zmaterial")
    );
    assert!(extract.overlays.selection.is_empty());
    assert!(extract
        .scene
        .meshes
        .iter()
        .any(|mesh| mesh.node_id == mesh_node));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn scene_assets_roundtrip_primitive_mesh_material_bindings() {
    let root = unique_temp_project_root("scene_primitive_bindings");
    let project = create_test_project(&root);
    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 77,
            name: "PrimitiveBindings".to_string(),
            parent: None,
            transform: TransformAsset::default(),
            active: true,
            render_layer_mask: 0x0000_0001,
            mobility: SceneMobilityAsset::Dynamic,
            camera: None,
            mesh: Some(SceneMeshInstanceAsset {
                model: asset_reference("res://models/triangle.obj"),
                mesh: None,
                material: asset_reference("res://materials/grid.zmaterial"),
                primitives: vec![SceneMeshPrimitiveBindingAsset {
                    mesh: asset_reference("res://meshes/triangle.zmesh"),
                    material: asset_reference("res://materials/grid.zmaterial"),
                }],
            }),
            ambient_light: None,
            directional_light: None,
            point_light: None,
            rect_light: None,
            spot_light: None,
            rigid_body: None,
            collider: None,
            joint: None,
            animation_skeleton: None,
            animation_player: None,
            animation_sequence_player: None,
            animation_graph_player: None,
            animation_state_machine_player: None,
            terrain: None,
            tilemap: None,
            prefab_instance: None,
        }],
    };

    let world = World::from_scene_asset(&project, &scene).unwrap();
    let mesh_node = world
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, NodeKind::Mesh))
        .unwrap();
    let mesh = mesh_node.mesh.as_ref().unwrap();
    assert_eq!(mesh.primitives.len(), 1);
    assert_eq!(
        mesh.primitives[0].mesh,
        project_mesh_handle(&project, "res://meshes/triangle.zmesh")
    );
    assert_eq!(
        mesh.primitives[0].material,
        project_material_handle(&project, "res://materials/grid.zmaterial")
    );

    let extract = world.to_render_extract();
    let render_mesh = extract
        .scene
        .meshes
        .iter()
        .find(|mesh| mesh.node_id == mesh_node.id)
        .unwrap();
    assert_eq!(render_mesh.mesh, Some(mesh.primitives[0].mesh));
    assert_eq!(render_mesh.material, mesh.primitives[0].material);

    let saved = world.to_scene_asset(&project).unwrap();
    let saved_binding = &saved.entities[0].mesh.as_ref().unwrap().primitives[0];
    assert_eq!(
        saved_binding.mesh.to_string(),
        "res://meshes/triangle.zmesh"
    );
    assert_eq!(
        saved_binding.material.to_string(),
        "res://materials/grid.zmaterial"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn scene_assets_roundtrip_asset_bound_physics_and_animation_components() {
    let root = unique_temp_project_root("scene_physics_animation");
    let project = create_test_project(&root);
    let world = World::load_scene_from_uri(
        &project,
        &AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
    )
    .unwrap();

    let mesh_node = world
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, NodeKind::Mesh))
        .unwrap()
        .id;

    let rigid_body = world.rigid_body(mesh_node).unwrap();
    assert_eq!(rigid_body.body_type, RigidBodyType::Dynamic);
    assert_eq!(rigid_body.mass, 2.5);

    let collider = world.collider(mesh_node).unwrap();
    assert_eq!(
        collider.shape,
        ColliderShape::Box {
            half_extents: Vec3::new(0.5, 0.5, 0.5),
        }
    );
    assert_eq!(
        collider.material,
        Some(project_physics_material_handle(
            &project,
            "res://physics/default.physics_material.toml"
        ))
    );
    assert_eq!(
        collider.material_override,
        Some(PhysicsMaterialMetadata {
            static_friction: 0.7,
            dynamic_friction: 0.4,
            restitution: 0.2,
            friction_combine: PhysicsCombineRule::Maximum,
            restitution_combine: PhysicsCombineRule::Average,
        })
    );

    let joint = world.joint(mesh_node).unwrap();
    assert_eq!(joint.joint_type, JointKind::Fixed);
    assert_eq!(joint.connected_entity, Some(world.active_camera()));

    let skeleton = world.animation_skeleton(mesh_node).unwrap();
    assert_eq!(
        skeleton.skeleton,
        project_animation_skeleton_handle(&project, "res://animation/hero.skeleton.zranim")
    );

    let animation_player = world.animation_player(mesh_node).unwrap();
    assert_eq!(
        animation_player.clip,
        project_animation_clip_handle(&project, "res://animation/hero.clip.zranim")
    );
    assert_eq!(animation_player.weight, 0.8);

    let sequence_player = world.animation_sequence_player(mesh_node).unwrap();
    assert_eq!(
        sequence_player.sequence,
        project_animation_sequence_handle(&project, "res://animation/hero.sequence.zranim")
    );
    assert!(!sequence_player.looping);

    let graph_player = world.animation_graph_player(mesh_node).unwrap();
    assert_eq!(
        graph_player.graph,
        project_animation_graph_handle(&project, "res://animation/hero.graph.zranim")
    );
    assert_eq!(
        graph_player.parameters.get("speed"),
        Some(&AnimationParameterValue::Scalar(1.5))
    );

    let state_machine_player = world.animation_state_machine_player(mesh_node).unwrap();
    assert_eq!(
        state_machine_player.state_machine,
        project_animation_state_machine_handle(
            &project,
            "res://animation/hero.state_machine.zranim"
        )
    );
    assert_eq!(
        state_machine_player.active_state.as_deref(),
        Some("Locomotion")
    );

    let saved = world.to_scene_asset(&project).unwrap();
    let saved_mesh = saved
        .entities
        .iter()
        .find(|entity| entity.entity == mesh_node)
        .unwrap();
    assert_eq!(
        saved_mesh
            .collider
            .as_ref()
            .and_then(|collider| collider.material.as_ref())
            .unwrap()
            .to_string(),
        "res://physics/default.physics_material.toml"
    );
    assert_eq!(
        saved_mesh
            .animation_player
            .as_ref()
            .unwrap()
            .clip
            .to_string(),
        "res://animation/hero.clip.zranim"
    );
    assert_eq!(
        saved_mesh
            .animation_graph_player
            .as_ref()
            .unwrap()
            .parameters
            .get("grounded"),
        Some(&AnimationParameterValue::Bool(true))
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn scene_assets_roundtrip_camera_product_fields() {
    let root = unique_temp_project_root("scene_camera_products");
    let project = create_test_project(&root);
    let mut world = World::load_scene_from_uri(
        &project,
        &AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
    )
    .unwrap();
    let camera = world.active_camera();
    *world.get_mut::<CameraComponent>(camera).unwrap() = CameraComponent {
        projection_mode: ProjectionMode::Orthographic,
        fov_y_radians: 0.7,
        ortho_size: 18.0,
        z_near: 0.02,
        z_far: 900.0,
        target: RenderCameraTarget::Headless {
            size: UVec2::new(512, 256),
        },
        viewport: Some(RenderViewportRect::new(
            UVec2::new(12, 24),
            UVec2::new(320, 160),
        )),
        order: 7,
        is_active: false,
        hdr: true,
        exposure_ev100: 10.5,
        clear_color: RenderCameraClearColor::None,
        msaa_samples: 8,
    };

    let saved = world.to_scene_asset(&project).unwrap();
    let saved_camera = saved
        .entities
        .iter()
        .find(|entity| entity.entity == camera)
        .and_then(|entity| entity.camera.as_ref())
        .unwrap();
    assert_eq!(saved_camera.projection_mode, ProjectionMode::Orthographic);
    assert!(matches!(
        &saved_camera.target,
        SceneCameraTargetAsset::Headless { size: [512, 256] }
    ));
    assert_eq!(saved_camera.order, 7);
    assert!(!saved_camera.active);
    assert!(saved_camera.hdr);
    assert_eq!(saved_camera.msaa_samples, 8);

    let loaded = World::from_scene_asset(&project, &saved).unwrap();
    let loaded_camera = loaded.find_node(camera).unwrap().camera.unwrap();
    assert_eq!(loaded_camera.projection_mode, ProjectionMode::Orthographic);
    assert_eq!(loaded_camera.ortho_size, 18.0);
    assert!(matches!(
        loaded_camera.target,
        RenderCameraTarget::Headless { size } if size == UVec2::new(512, 256)
    ));
    assert_eq!(loaded_camera.order, 7);
    assert!(!loaded_camera.is_active);
    assert!(loaded_camera.hdr);
    assert_eq!(loaded_camera.clear_color, RenderCameraClearColor::None);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn scene_assets_roundtrip_ambient_and_rect_light_product_fields() {
    let root = unique_temp_project_root("scene_light_products");
    let project = create_test_project(&root);
    let mut world = World::load_scene_from_uri(
        &project,
        &AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
    )
    .unwrap();

    let ambient = world.spawn_node(NodeKind::AmbientLight);
    world
        .set_ambient_light(
            ambient,
            Some(AmbientLight {
                color: Vec3::new(0.2, 0.25, 0.3),
                intensity: 96.0,
                affects_lightmapped_meshes: false,
            }),
        )
        .unwrap();

    let rect = world.spawn_node(NodeKind::RectLight);
    world
        .set_rect_light(
            rect,
            Some(RectLight {
                color: Vec3::new(1.0, 0.72, 0.35),
                intensity: 72_000.0,
                range: 18.0,
                size: Vec2::new(3.5, 1.5),
            }),
        )
        .unwrap();

    let saved = world.to_scene_asset(&project).unwrap();
    let saved_ambient = saved
        .entities
        .iter()
        .find(|entity| entity.entity == ambient)
        .and_then(|entity| entity.ambient_light.as_ref())
        .unwrap();
    assert_eq!(saved_ambient.color, [0.2, 0.25, 0.3]);
    assert_eq!(saved_ambient.intensity, 96.0);
    assert!(!saved_ambient.affects_lightmapped_meshes);

    let saved_rect = saved
        .entities
        .iter()
        .find(|entity| entity.entity == rect)
        .and_then(|entity| entity.rect_light.as_ref())
        .unwrap();
    assert_eq!(saved_rect.color, [1.0, 0.72, 0.35]);
    assert_eq!(saved_rect.intensity, 72_000.0);
    assert_eq!(saved_rect.range, 18.0);
    assert_eq!(saved_rect.size, [3.5, 1.5]);

    let loaded = World::from_scene_asset(&project, &saved).unwrap();
    assert!(matches!(
        loaded.find_node(ambient).unwrap().kind,
        NodeKind::AmbientLight
    ));
    assert_eq!(
        loaded.ambient_light(ambient).unwrap().color,
        Vec3::new(0.2, 0.25, 0.3)
    );
    assert!(
        !loaded
            .ambient_light(ambient)
            .unwrap()
            .affects_lightmapped_meshes
    );
    assert!(matches!(
        loaded.find_node(rect).unwrap().kind,
        NodeKind::RectLight
    ));
    assert_eq!(loaded.rect_light(rect).unwrap().size, Vec2::new(3.5, 1.5));

    let _ = fs::remove_dir_all(root);
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

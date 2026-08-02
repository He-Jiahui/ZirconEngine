use super::vampire_root;
use crate::asset::AssetUri;
use crate::asset::project::ProjectManager;
use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::render::FallbackSkyboxKind;
use crate::scene::world::World;

#[test]
fn vampire_example_scene_extracts_playable_third_person_meshes() {
    let root = vampire_root();
    let mut project = ProjectManager::open(&root).unwrap();
    project
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    project.scan_and_import().unwrap();

    let world = World::load_scene_from_uri(
        &project,
        &AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
    )
    .unwrap();
    assert!(
        world.animation_skeleton(2).is_some(),
        "loaded player should carry an animation skeleton component"
    );
    let player_state_machine = world
        .animation_state_machine_player(2)
        .expect("loaded player should carry a state machine player component");
    assert_eq!(
        player_state_machine.parameters.get("moving"),
        Some(&AnimationParameterValue::Bool(false))
    );
    assert_eq!(
        player_state_machine.parameters.get("attacking"),
        Some(&AnimationParameterValue::Bool(false))
    );
    let extract = world.to_render_frame_extract();
    let mesh_entities = extract
        .geometry
        .meshes
        .iter()
        .map(|mesh| mesh.node_id)
        .collect::<Vec<_>>();

    for animated_mesh_entity in [202, 203, 204, 205, 206, 207] {
        assert!(
            mesh_entities.contains(&animated_mesh_entity),
            "render extract should contain animated vampire body mesh node {animated_mesh_entity}"
        );
    }
    assert!(mesh_entities.contains(&3));
    assert!(mesh_entities.contains(&20));
    assert!(mesh_entities.contains(&24));
    assert!(mesh_entities.contains(&119));
    assert!(mesh_entities.contains(&123));
    assert!(mesh_entities.contains(&133));
    assert!(mesh_entities.contains(&135));
    for grass_batch_entity in [151, 152, 153, 154, 155, 156] {
        assert!(
            mesh_entities.contains(&grass_batch_entity),
            "render extract should include static grass batch entity {grass_batch_entity}"
        );
    }
    let grass_static_batch = extract
        .geometry
        .static_batches
        .iter()
        .find(|batch| batch.entities == vec![151, 152, 153, 154, 155, 156])
        .expect("six grass billboard entities should collapse into one runtime static batch");
    assert_eq!(grass_static_batch.instance_count(), 6);
    assert!(mesh_entities.len() >= 51);
    assert!(extract.view.camera.is_active);
    assert_eq!(extract.view.scene_camera_entity, Some(1));
    assert!(extract.view.camera.transform.forward().z > 0.7);
    assert!(extract.view.camera.transform.forward().y < -0.4);
    assert!(extract.post_process.preview.skybox_enabled);
    assert_eq!(
        extract.post_process.preview.fallback_skybox,
        FallbackSkyboxKind::ProceduralGradient
    );
}

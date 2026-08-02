use std::path::Path;

use super::vampire_root;
use crate::asset::project::{ProjectManager, ProjectManifest};
use crate::asset::{
    AlphaMode, AssetKind, AssetUri, MaterialAsset, ModelAsset, SceneAsset, SceneMobilityAsset,
    TerrainAsset, ZShaderDocumentV2,
};
use crate::builtin::RuntimePluginId;
use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::animation::{AnimationGraphAsset, AnimationStateMachineAsset};
use crate::core::framework::navigation::NavMeshAsset;
use crate::core::resource::ResourceState;
use crate::script::discover_vm_plugin_packages;

#[test]
fn vampire_example_manifest_scene_and_scripts_are_importable() {
    let root = vampire_root();
    let manifest = ProjectManifest::load(root.join("zircon-project.toml")).unwrap();

    assert_eq!(manifest.name, "Zircon Vampire Roguelite");
    assert_eq!(
        manifest.default_scene,
        AssetUri::parse("res://scenes/main.scene.toml").unwrap()
    );
    assert_eq!(manifest.scripts.package_roots, ["scripts"]);
    assert_eq!(manifest.scripts.startup_packages, ["vampire_game"]);
    assert!(manifest.plugins.selections.iter().any(|selection| {
        selection.enabled
            && selection.required
            && RuntimePluginId::parse_key(&selection.id) == Some(RuntimePluginId::GltfImporter)
    }));

    let scene = SceneAsset::from_toml_str(
        &std::fs::read_to_string(root.join("assets/scenes/main.scene.toml")).unwrap(),
    )
    .unwrap();
    let overview = scene.overview();
    assert!(overview.entity_count >= 66);
    assert!(overview.mesh_instance_count >= 51);
    assert!(
        overview.terrain_count >= 1,
        "vampire scene should include a real terrain asset reference, not only terrain-like props"
    );
    assert!(overview.light_count >= 11);
    assert!(
        overview
            .entities
            .iter()
            .any(|entity| entity.has_post_process_settings)
    );
    assert!(
        overview
            .entities
            .iter()
            .any(|entity| entity.has_post_process_volume)
    );
    assert!(scene.entities.iter().any(|entity| {
        entity
            .script_bindings
            .iter()
            .any(|binding| binding.package == "vampire_game" && binding.module == "main")
    }));
    assert!(
        scene.entities.iter().all(|entity| {
            !entity.name.contains("Health Bar")
                && entity.script_bindings.iter().all(|binding| {
                    !matches!(
                        binding
                            .properties
                            .get("role")
                            .and_then(|value| value.as_str()),
                        Some("health_bar_fill" | "health_bar_back")
                    )
                })
        }),
        "health bars must be authored through dynamic scene HUD data instead of scene mesh entities"
    );
    let player = scene
        .entities
        .iter()
        .find(|entity| entity.entity == 2)
        .expect("player entity");
    assert!(
        player.animation_skeleton.is_some(),
        "player should bind the imported vampire animation skeleton"
    );
    let player_state_machine = player
        .animation_state_machine_player
        .as_ref()
        .expect("player should bind a locomotion state machine");
    assert_eq!(
        player_state_machine.state_machine.locator,
        AssetUri::parse("res://animation/vampire_locomotion.state_machine.zranim").unwrap()
    );
    assert_eq!(
        player_state_machine.parameters.get("moving"),
        Some(&AnimationParameterValue::Bool(false))
    );
    assert_eq!(
        player_state_machine.parameters.get("attacking"),
        Some(&AnimationParameterValue::Bool(false))
    );
    for (entity, name, parent, has_mesh) in [
        (201, "Node1:root", Some(2), false),
        (202, "Node2:torso", Some(201), true),
        (203, "Node3:arm-right", Some(202), true),
        (204, "Node4:arm-left", Some(202), true),
        (205, "Node5:head", Some(202), true),
        (206, "Node6:leg-right", Some(201), true),
        (207, "Node7:leg-left", Some(201), true),
    ] {
        let actor_node = scene
            .entities
            .iter()
            .find(|candidate| candidate.entity == entity)
            .unwrap_or_else(|| panic!("missing vampire actor node {entity}"));
        assert_eq!(actor_node.name, name);
        assert_eq!(actor_node.parent, parent);
        assert_eq!(actor_node.mesh.is_some(), has_mesh);
    }
    for scenic_anchor in [
        "Baked Jungle Terrain",
        "Near West Glow Orchid",
        "Near East Glow Orchid",
        "Midline Broadleaf Left",
        "Midline Broadleaf Right",
        "Left Jungle Relic Flame",
        "Right Jungle Relic Flame",
        "Left Near Root Wall",
        "Right Near Root Wall",
        "Canopy West Ridge",
        "Canopy East Ridge",
        "Northwest Fern Bank",
        "Northeast Fern Bank",
        "Static Grass Batch Foreground West",
        "Static Grass Batch Foreground East",
        "Static Grass Batch West Path Edge",
        "Static Grass Batch East Path Edge",
        "Static Grass Batch North Fern Merge",
        "Static Grass Batch South Fern Merge",
    ] {
        assert!(
            overview
                .entities
                .iter()
                .any(|entity| entity.name == scenic_anchor && entity.has_mesh),
            "missing camera-visible jungle dressing entity {scenic_anchor}"
        );
    }
    let grass_batch_entities = scene
        .entities
        .iter()
        .filter(|entity| entity.name.starts_with("Static Grass Batch "))
        .collect::<Vec<_>>();
    assert_eq!(
        grass_batch_entities.len(),
        6,
        "grass should be represented by a small number of authored static batch entities"
    );
    for grass_batch in grass_batch_entities {
        assert_eq!(
            grass_batch.mobility,
            SceneMobilityAsset::Static,
            "{} should be static so the renderer can treat it as a static-batch candidate",
            grass_batch.name
        );
        let mesh = grass_batch
            .mesh
            .as_ref()
            .unwrap_or_else(|| panic!("{} should carry a grass mesh", grass_batch.name));
        assert_eq!(
            mesh.model.locator,
            AssetUri::parse("res://models/grass_billboard_static_batch.model.toml").unwrap()
        );
        assert_eq!(
            mesh.material.locator,
            AssetUri::parse("res://materials/forest_grass_billboard.zmaterial").unwrap()
        );
    }
    let terrain_entity = scene
        .entities
        .iter()
        .find(|entity| entity.name == "Baked Jungle Terrain")
        .expect("baked jungle terrain entity");
    assert!(
        terrain_entity.mesh.is_some() && terrain_entity.terrain.is_some(),
        "Baked Jungle Terrain should keep a visible mesh and reference terrain heightfield data"
    );
    assert!(
        terrain_entity.transform.scale[1] >= 1.8,
        "terrain mesh should exaggerate height enough to read as rugged forest ground"
    );
    assert_eq!(
        terrain_entity
            .terrain
            .as_ref()
            .map(|terrain| &terrain.terrain.locator),
        Some(&AssetUri::parse("res://terrain/jungle_clearing.terrain.toml").unwrap())
    );
    for lighting_anchor in [
        "Near West Orchid Light",
        "Near East Orchid Light",
        "Left Jungle Relic Light",
        "Right Jungle Relic Light",
        "West Firefly Light",
        "East Firefly Light",
    ] {
        assert!(
            overview
                .entities
                .iter()
                .any(|entity| entity.name == lighting_anchor && entity.has_point_light),
            "missing camera-visible local light {lighting_anchor}"
        );
    }
    for model in [
        "jungle_terrain.model.toml",
        "jungle_broadleaf.model.toml",
        "jungle_fern_cluster.model.toml",
        "grass_billboard_static_batch.model.toml",
    ] {
        assert!(root.join("assets/models").join(model).exists());
        ModelAsset::from_toml_str(
            &std::fs::read_to_string(root.join("assets/models").join(model)).unwrap(),
        )
        .unwrap();
    }
    for glb in [
        "character-vampire.glb",
        "character-skeleton.glb",
        "character-zombie.glb",
        "character-ghost.glb",
        "crypt-large.glb",
        "fire-basket.glb",
    ] {
        assert!(
            root.join("assets/models/kenney_graveyard")
                .join(glb)
                .exists(),
            "missing Kenney GLB {glb}"
        );
    }
    for asset in [
        "assets/materials/player_blood.zmaterial",
        "assets/materials/ghoul_shadow.zmaterial",
        "assets/materials/arena_stone.zmaterial",
        "assets/materials/jungle_ground.zmaterial",
        "assets/materials/jungle_path_mud.zmaterial",
        "assets/materials/jungle_leaf.zmaterial",
        "assets/materials/jungle_trunk.zmaterial",
        "assets/materials/jungle_mist.zmaterial",
        "assets/materials/forest_grass_billboard.zmaterial",
        "assets/materials/pale_bone.zmaterial",
        "assets/materials/ghost_mist.zmaterial",
        "assets/materials/ember_flame.zmaterial",
    ] {
        MaterialAsset::from_toml_str(&std::fs::read_to_string(root.join(asset)).unwrap()).unwrap();
    }
    assert!(
        root.join("assets/textures/jungle_ground_albedo.png")
            .exists(),
        "missing jungle ground albedo texture"
    );
    let jungle_terrain = TerrainAsset::from_toml_str(
        &std::fs::read_to_string(root.join("assets/terrain/jungle_clearing.terrain.toml")).unwrap(),
    )
    .unwrap();
    jungle_terrain.validate_dimensions().unwrap();
    assert_eq!(jungle_terrain.width * jungle_terrain.height, 81);
    let (terrain_min, terrain_max) = jungle_terrain
        .height_samples
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), value| {
            (min.min(*value), max.max(*value))
        });
    assert!(
        terrain_max - terrain_min >= 2.0,
        "terrain heightfield should be visibly rugged, range={}",
        terrain_max - terrain_min
    );
    assert!(
        jungle_terrain.layers.iter().any(|layer| {
            layer.material.as_ref().is_some_and(|material| {
                material.locator
                    == AssetUri::parse("res://materials/jungle_ground.zmaterial").unwrap()
            })
        }),
        "terrain layer stack should bind the jungle ground material"
    );
    let jungle_ground = MaterialAsset::from_toml_str(
        &std::fs::read_to_string(root.join("assets/materials/jungle_ground.zmaterial")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        jungle_ground
            .base_color_texture
            .as_ref()
            .map(|texture| &texture.locator),
        Some(&AssetUri::parse("res://textures/jungle_ground_albedo.png").unwrap()),
        "jungle ground should bind the authored terrain albedo texture"
    );
    assert!(
        jungle_ground.base_color[3] > 0.955 && jungle_ground.base_color[3] < 0.96,
        "jungle ground alpha should mark the forest ground shader path"
    );
    assert_eq!(
        jungle_ground.shader.locator,
        AssetUri::parse("res://shaders/vampire_forest").unwrap()
    );
    let grass_material = MaterialAsset::from_toml_str(
        &std::fs::read_to_string(root.join("assets/materials/forest_grass_billboard.zmaterial"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        grass_material.shader.locator,
        AssetUri::parse("res://shaders/vampire_forest").unwrap()
    );
    assert!(grass_material.double_sided);
    assert_eq!(grass_material.alpha_mode, AlphaMode::Opaque);
    assert!(
        grass_material.base_color[3] > 0.92 && grass_material.base_color[3] < 0.955,
        "grass material alpha should mark the forest grass shader path"
    );
    let grass_model = ModelAsset::from_toml_str(
        &std::fs::read_to_string(
            root.join("assets/models/grass_billboard_static_batch.model.toml"),
        )
        .unwrap(),
    )
    .unwrap();
    let grass_overview = grass_model.overview();
    assert_eq!(grass_overview.primitive_count, 1);
    assert!(
        grass_overview.vertex_count >= 48 && grass_overview.render_primitive_count >= 24,
        "grass batch model should merge many billboard cards into one static primitive: vertices={} triangles={}",
        grass_overview.vertex_count,
        grass_overview.render_primitive_count
    );
    let nav_mesh = toml::from_str::<NavMeshAsset>(
        &std::fs::read_to_string(root.join("assets/navigation/main.navmesh.toml")).unwrap(),
    )
    .unwrap();
    assert!(
        nav_mesh.polygons.len() >= 8,
        "jungle navigation should be baked as multiple corridor/clearing polygons"
    );
    assert!(
        nav_mesh.vertices.iter().any(|vertex| vertex[1].abs() > 0.5),
        "jungle navmesh should follow the authored uneven terrain height"
    );
    ZShaderDocumentV2::from_toml_str(
        &std::fs::read_to_string(root.join("assets/shaders/default_pbr/default_pbr.zshader"))
            .unwrap(),
    )
    .unwrap();
    for graph in [
        "assets/animation/vampire_idle.graph.zranim",
        "assets/animation/vampire_move.graph.zranim",
        "assets/animation/vampire_attack.graph.zranim",
    ] {
        let graph_asset =
            AnimationGraphAsset::from_bytes(&std::fs::read(root.join(graph)).unwrap())
                .unwrap_or_else(|error| {
                    panic!("{graph} should decode as an animation graph: {error}")
                });
        assert!(
            !graph_asset.nodes.is_empty(),
            "{graph} should contain clip/output graph nodes"
        );
    }
    let locomotion_state_machine = AnimationStateMachineAsset::from_bytes(
        &std::fs::read(root.join("assets/animation/vampire_locomotion.state_machine.zranim"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(locomotion_state_machine.entry_state, "Idle");
    assert!(
        locomotion_state_machine
            .states
            .iter()
            .any(|state| state.name == "Move")
    );
    assert!(
        locomotion_state_machine
            .states
            .iter()
            .any(|state| state.name == "Attack")
    );
    let shader_source =
        std::fs::read_to_string(root.join("assets/shaders/default_pbr/default_pbr.wgsl")).unwrap();
    assert!(
        shader_source.contains("fn vampire_actor_detail_color"),
        "vampire shader should visibly enhance color-map actor materials instead of leaving them as flat palette blocks"
    );
    assert!(
        shader_source.contains("material_properties.data0.a > 0.96"),
        "vampire actor detail path should be gated by the project actor-material alpha marker"
    );
    for marker in [
        "fn forest_ground_detail_color",
        "fn forest_ground_detail_mask",
        "fn forest_foliage_detail_color",
        "fn forest_grass_detail_color",
        "fn forest_grass_detail_mask",
        "fn vampire_detail_normal",
        "fn vampire_shadowed_direct_visibility",
        "fn vampire_micro_occlusion",
        "fn vampire_material_specular",
        "fn vampire_wet_surface_reflection",
    ] {
        assert!(
            shader_source.contains(marker),
            "vampire shader should include forest rendering marker {marker}"
        );
    }
    for marker in [
        "textureSample(metallic_roughness_tex",
        "textureSample(occlusion_tex",
        "zr_gpu_scene_shadow_params",
        "shadow_visibility",
        "wet_reflection",
    ] {
        assert!(
            shader_source.contains(marker),
            "vampire shader should include realistic material/light marker {marker}"
        );
    }
    for (material, shader_uri) in [
        (
            "assets/materials/player_blood.zmaterial",
            "res://shaders/vampire_actor",
        ),
        (
            "assets/materials/pale_bone.zmaterial",
            "res://shaders/vampire_actor",
        ),
        (
            "assets/materials/ghoul_shadow.zmaterial",
            "res://shaders/vampire_actor",
        ),
        (
            "assets/materials/ghost_mist.zmaterial",
            "res://shaders/vampire_effect",
        ),
    ] {
        let loaded =
            MaterialAsset::from_toml_str(&std::fs::read_to_string(root.join(material)).unwrap())
                .unwrap();
        assert!(
            loaded.base_color_texture.is_some(),
            "{material} should bind the imported GLB color-map texture"
        );
        assert!(
            loaded.base_color[3] > 0.96,
            "{material} should opt into the actor detail shader path via the alpha marker"
        );
        assert_eq!(
            loaded.shader.locator,
            AssetUri::parse(shader_uri).unwrap(),
            "{material} should route through its dedicated vampire shader"
        );
    }
    for (shader, marker) in [
        (
            "assets/shaders/vampire_actor/vampire_actor.zshader",
            "vampire_actor_variant.wgsl",
        ),
        (
            "assets/shaders/vampire_forest/vampire_forest.zshader",
            "vampire_forest_variant.wgsl",
        ),
        (
            "assets/shaders/vampire_effect/vampire_effect.zshader",
            "vampire_effect_variant.wgsl",
        ),
    ] {
        let document =
            ZShaderDocumentV2::from_toml_str(&std::fs::read_to_string(root.join(shader)).unwrap())
                .unwrap_or_else(|error| panic!("{shader} should parse as zshader: {error}"));
        assert!(
            document.wgsl_files().iter().any(|file| file == marker),
            "{shader} should include its own variant source marker"
        );
    }
    for material_uri in [
        "res://materials/player_blood.zmaterial",
        "res://materials/pale_bone.zmaterial",
        "res://materials/ghoul_shadow.zmaterial",
        "res://materials/ghost_mist.zmaterial",
    ] {
        let material_locator = AssetUri::parse(material_uri).unwrap();
        assert!(
            scene.entities.iter().any(|entity| {
                entity.mesh.as_ref().is_some_and(|mesh| {
                    &mesh.material.locator == &material_locator
                        || mesh
                            .primitives
                            .iter()
                            .any(|primitive| &primitive.material.locator == &material_locator)
                })
            }),
            "{material_uri} should be assigned to scene actor mesh bindings"
        );
    }
    for enemy in scene.entities.iter().filter(|entity| {
        entity.script_bindings.iter().any(|binding| {
            binding
                .properties
                .get("role")
                .and_then(|value| value.as_str())
                == Some("enemy")
        })
    }) {
        assert!(
            enemy.script_bindings.iter().any(|binding| {
                binding
                    .properties
                    .get("behavior_tree")
                    .and_then(|value| value.as_str())
                    == Some("graveyard_enemy_bt")
            }),
            "enemy {} should explicitly bind the authored behavior-tree contract",
            enemy.entity
        );
        let mesh = enemy
            .mesh
            .as_ref()
            .unwrap_or_else(|| panic!("enemy {} should keep a visible GLB mesh", enemy.entity));
        let model_uri = mesh.model.locator.to_string();
        assert!(
            model_uri.starts_with("res://models/kenney_graveyard/character-")
                && !model_uri.contains("ghoul_capsule"),
            "enemy {} should use a Kenney GLB actor model, got {}",
            enemy.entity,
            model_uri
        );
        let archetype = enemy
            .script_bindings
            .iter()
            .find_map(|binding| {
                binding
                    .properties
                    .get("archetype")
                    .and_then(|value| value.as_str())
            })
            .unwrap_or("unknown");
        let minimum_primitives = if archetype == "ghost" { 3 } else { 6 };
        assert!(
            mesh.primitives.len() >= minimum_primitives,
            "enemy {} ({archetype}) should use a multi-primitive GLB actor, not a simple capsule fallback",
            enemy.entity,
        );
    }
    let script_source = std::fs::read_to_string(root.join("scripts/vampire_game/main.zr")).unwrap();
    let runtime_source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/script/vm");
    assert!(
        !runtime_source_root.join("vampire_gameplay.rs").exists()
            && !runtime_source_root.join("vampire_gameplay").exists()
            && !runtime_source_root
                .join("gameplay_host/vampire.rs")
                .exists(),
        "vampire gameplay must live in ZR script plus generic host APIs, not Rust vampire delegate modules"
    );
    assert!(
        script_source.contains("pub onStart(entity: int, dt: float): int")
            && script_source.contains("pub onUpdate(entity: int, dt: float): int"),
        "vampire script should export real ZR module functions for runtime callbacks"
    );
    assert!(
        !script_source.contains("vampire_start") && !script_source.contains("vampire_tick"),
        "vampire script must not trampoline into Rust vampire_start/vampire_tick delegates"
    );
    for marker in [
        "gameplay.key_pressed",
        "gameplay.translate",
        "gameplay.face_direction",
        "gameplay.camera_follow",
        "gameplay.follow_position",
        "gameplay.nearest_by_script_property",
        "gameplay.nav_move_towards_entity",
        "gameplay.damage_entity",
        "gameplay.set_world_hud_bar",
        "gameplay.set_animation_bool",
        "gameplay.set_particle_sprites",
        "gameplay.menu_state",
        "gameplay.control_state",
        "vampire.run_state",
        "Start Game",
        "Retry",
    ] {
        assert!(
            script_source.contains(marker),
            "vampire ZR script should drive gameplay through generic host API marker {marker}"
        );
    }
    let packages = discover_vm_plugin_packages(root.join("scripts")).unwrap();
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].package.manifest.name, "vampire_game");
    assert_eq!(packages[0].backend_name, "zr_vm:project");
    let plugin_manifest = std::fs::read_to_string(root.join("scripts/vampire_game/plugin.toml"))
        .expect("vampire plugin manifest should be readable");
    assert!(
        !plugin_manifest.contains("zr_vm_fallback:project"),
        "vampire should not route gameplay through the project fallback backend"
    );

    let mut project = ProjectManager::open(&root).unwrap();
    project
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    let records = project.scan_and_import().unwrap();
    assert!(
        records
            .iter()
            .any(|record| record.primary_locator() == &manifest.default_scene)
    );
    assert!(records.iter().any(|record| record.kind == AssetKind::Shader
        && record.primary_locator() == &AssetUri::parse("res://shaders/default_pbr").unwrap()));
    for shader_uri in [
        "res://shaders/vampire_actor",
        "res://shaders/vampire_forest",
        "res://shaders/vampire_effect",
    ] {
        let shader_uri = AssetUri::parse(shader_uri).unwrap();
        let record = records
            .iter()
            .find(|record| {
                record.kind == AssetKind::Shader && record.primary_locator() == &shader_uri
            })
            .unwrap_or_else(|| panic!("missing imported shader asset {shader_uri}"));
        assert_eq!(
            record.state,
            ResourceState::Ready,
            "shader asset {shader_uri} should import cleanly: {:?}",
            record.diagnostics
        );
    }
    let default_pbr =
        std::fs::read_to_string(root.join("assets/shaders/default_pbr/default_pbr.wgsl")).unwrap();
    assert!(
        default_pbr.contains("vampire_ground_light_floor")
            && default_pbr.contains("vec3<f32>(0.18, 0.30, 0.12)")
            && default_pbr.contains("base_color.a > 0.99"),
        "vampire ground shader should keep a readable terrain light floor and avoid classifying jungle ground as black arena stone"
    );
    assert!(
        records
            .iter()
            .any(|record| record.kind == AssetKind::Texture
                && record.primary_locator()
                    == &AssetUri::parse("res://textures/jungle_ground_albedo.png").unwrap())
    );
    for (uri, kind) in [
        (
            "res://animation/vampire_idle.graph.zranim",
            AssetKind::AnimationGraph,
        ),
        (
            "res://animation/vampire_move.graph.zranim",
            AssetKind::AnimationGraph,
        ),
        (
            "res://animation/vampire_attack.graph.zranim",
            AssetKind::AnimationGraph,
        ),
        (
            "res://animation/vampire_locomotion.state_machine.zranim",
            AssetKind::AnimationStateMachine,
        ),
    ] {
        let uri = AssetUri::parse(uri).unwrap();
        let record = records
            .iter()
            .find(|record| record.kind == kind && record.primary_locator() == &uri)
            .unwrap_or_else(|| panic!("missing imported animation asset {uri}"));
        assert_eq!(
            record.state,
            ResourceState::Ready,
            "animation asset {uri} should import cleanly: {:?}",
            record.diagnostics
        );
    }
    let vampire_model_uri =
        AssetUri::parse("res://models/kenney_graveyard/character-vampire.glb").unwrap();
    let vampire_model = records
        .iter()
        .find(|record| {
            record.kind == AssetKind::Model && record.primary_locator() == &vampire_model_uri
        })
        .expect("vampire GLB record");
    assert_eq!(
        vampire_model.state,
        ResourceState::Ready,
        "vampire GLB import diagnostics: {:?}",
        vampire_model.diagnostics
    );
    assert!(
        vampire_model.artifact_locator().is_some(),
        "vampire GLB should produce a model artifact"
    );
    for uri in [
        "res://models/kenney_graveyard/character-vampire.glb#Animation1",
        "res://models/kenney_graveyard/character-vampire.glb#Animation1/Skeleton",
        "res://models/kenney_graveyard/character-vampire.glb#Animation2",
        "res://models/kenney_graveyard/character-vampire.glb#Animation19",
    ] {
        let uri = AssetUri::parse(uri).unwrap();
        assert!(
            records
                .iter()
                .any(|record| record.primary_locator() == &uri
                    && record.state == ResourceState::Ready),
            "vampire GLB should emit ready animation subasset {uri}"
        );
    }
}

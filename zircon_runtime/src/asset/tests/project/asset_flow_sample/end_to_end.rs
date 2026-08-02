use super::assertions::{
    assert_dependencies, assert_loaded_with_dependencies, assert_ready_record, load_material,
    load_mesh, load_model, load_scene, load_shader, load_texture, resource_record,
    texture_bind_group_layout, uri,
};
use super::fixtures::{
    write_bc1_texture, write_default_pbr_shader_package, write_minimal_textured_gltf,
    write_sample_material, write_sample_shader_package,
};
use super::importers::{
    project_asset_manager_with_sample_importers, project_manager_with_sample_importers,
};
use super::*;

#[test]
fn project_manager_imports_minimal_gltf_material_shader_mesh_sample() {
    let root = unique_temp_project_root("project_manager_minimal_asset_flow");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new("MinimalAssetFlow", uri("res://models/hero.gltf#Scene0"), 1)
        .save(paths.manifest_path())
        .unwrap();

    write_minimal_textured_gltf(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("models")
            .join("hero.gltf"),
    );
    write_sample_shader_package(&paths);
    write_default_pbr_shader_package(&paths);
    write_sample_material(&paths);
    write_bc1_texture(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("textures")
            .join("hero_albedo_bc1.dds"),
    );

    let mut manager = project_manager_with_sample_importers(&root);
    let imported = manager.scan_and_import().unwrap();

    assert!(imported.len() >= 12);
    assert_ready_record(&manager, "res://models/hero.gltf", AssetKind::Model);
    assert_ready_record(
        &manager,
        "res://models/hero.gltf#Texture0",
        AssetKind::Texture,
    );
    assert_ready_record(
        &manager,
        "res://models/hero.gltf#Material0",
        AssetKind::Material,
    );
    assert_ready_record(&manager, "res://models/hero.gltf#Mesh0", AssetKind::Model);
    assert_ready_record(
        &manager,
        "res://models/hero.gltf#Mesh0/Primitive0",
        AssetKind::Mesh,
    );
    assert_ready_record(&manager, "res://models/hero.gltf#Node0", AssetKind::Scene);
    assert_ready_record(&manager, "res://models/hero.gltf#Scene0", AssetKind::Scene);
    assert_ready_record(&manager, "res://shaders/lit_sample", AssetKind::Shader);
    assert_ready_record(&manager, "res://shaders/default_pbr", AssetKind::Shader);
    assert_ready_record(
        &manager,
        "res://materials/hero_surface.zmaterial",
        AssetKind::Material,
    );
    assert_ready_record(
        &manager,
        "res://textures/hero_albedo_bc1.dds",
        AssetKind::Texture,
    );

    assert_dependencies(
        &manager,
        "res://models/hero.gltf#Scene0",
        &[
            "res://models/hero.gltf#Node0",
            "res://models/hero.gltf#Mesh0",
            "res://models/hero.gltf#Mesh0/Primitive0",
            "res://models/hero.gltf#Material0",
        ],
    );
    assert_dependencies(
        &manager,
        "res://models/hero.gltf#Material0",
        &[
            "res://models/hero.gltf#Texture0",
            "res://shaders/default_pbr",
        ],
    );
    assert_dependencies(
        &manager,
        "res://materials/hero_surface.zmaterial",
        &[
            "res://shaders/lit_sample",
            "res://textures/hero_albedo_bc1.dds",
        ],
    );

    let asset_manager = Arc::new(project_asset_manager_with_sample_importers());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    assert_loaded_with_dependencies::<crate::asset::SceneAsset>(
        &asset_manager,
        "res://models/hero.gltf#Scene0",
    );
    assert_loaded_with_dependencies::<ModelAsset>(&asset_manager, "res://models/hero.gltf");
    assert_loaded_with_dependencies::<MeshAsset>(
        &asset_manager,
        "res://models/hero.gltf#Mesh0/Primitive0",
    );
    assert_loaded_with_dependencies::<MaterialAsset>(
        &asset_manager,
        "res://materials/hero_surface.zmaterial",
    );
    assert_loaded_with_dependencies::<ShaderAsset>(&asset_manager, "res://shaders/lit_sample");
    assert_loaded_with_dependencies::<TextureAsset>(
        &asset_manager,
        "res://textures/hero_albedo_bc1.dds",
    );

    let root_model = load_model(&manager, "res://models/hero.gltf");
    assert_eq!(root_model.primitives.len(), 1);
    assert_eq!(root_model.primitives[0].vertices.len(), 3);
    assert_eq!(root_model.primitives[0].indices, vec![0, 1, 2]);
    assert_eq!(
        root_model.primitives[0]
            .mesh
            .as_ref()
            .map(|reference| reference.locator.clone()),
        Some(uri("res://models/hero.gltf#Mesh0/Primitive0"))
    );

    let scene = load_scene(&manager, "res://models/hero.gltf#Scene0");
    let scene_record = resource_record(&manager, "res://models/hero.gltf#Scene0");
    assert_eq!(scene.entities.len(), 1);
    let entity_mesh = scene.entities[0].mesh.as_ref().expect("scene mesh entity");
    assert_eq!(
        entity_mesh.model.locator,
        uri("res://models/hero.gltf#Mesh0")
    );
    assert_eq!(
        entity_mesh.material.locator,
        uri("res://models/hero.gltf#Material0")
    );
    assert!(entity_mesh.mesh.is_none());
    assert_eq!(entity_mesh.morph_weights, vec![0.5]);
    assert_eq!(entity_mesh.primitives.len(), 1);
    assert_eq!(
        entity_mesh.primitives[0].mesh.locator,
        uri("res://models/hero.gltf#Mesh0/Primitive0")
    );
    assert_eq!(
        entity_mesh.primitives[0].material.locator,
        uri("res://models/hero.gltf#Material0")
    );
    let scene_overview = scene.overview();
    assert_eq!(scene_overview.direct_reference_count, 4);
    assert_eq!(scene_overview.direct_mesh_reference_count, 1);
    assert_eq!(scene_overview.mesh_primitive_binding_count, 1);
    assert_eq!(scene_overview.morph_weight_count, 1);
    let scene_management = SceneAssetManagementRecordSet::from_records(vec![
        scene.management_record(scene_record.id()),
    ]);
    assert_eq!(scene_management.summary.scene_count, 1);
    assert_eq!(scene_management.summary.entity_count, 1);
    assert_eq!(scene_management.summary.mesh_instance_count, 1);
    assert_eq!(scene_management.summary.direct_reference_count, 4);
    assert_eq!(scene_management.summary.direct_mesh_reference_count, 1);
    assert_eq!(scene_management.summary.mesh_primitive_binding_count, 1);
    assert_eq!(scene_management.summary.morph_weight_count, 1);
    assert_eq!(scene_management.summary.mesh_material_binding_count, 1);
    let scene_entity_management = SceneEntityManagementRecordSet::from_records(
        scene_management.records[0].entity_management_records(),
    );
    assert_eq!(
        scene_entity_management.records[0]
            .entity
            .direct_mesh_reference_count,
        1
    );
    assert_eq!(
        scene_entity_management.records[0]
            .entity
            .mesh_primitive_binding_count,
        1
    );
    assert_eq!(
        scene_entity_management.records[0].entity.morph_weight_count,
        1
    );
    assert_eq!(
        scene_entity_management.summary.direct_mesh_reference_count,
        1
    );
    assert_eq!(
        scene_entity_management.summary.mesh_primitive_binding_count,
        1
    );
    assert_eq!(scene_entity_management.summary.morph_weight_count, 1);

    let mesh_model = load_model(&manager, "res://models/hero.gltf#Mesh0");
    let mesh_model_record = resource_record(&manager, "res://models/hero.gltf#Mesh0");
    let model_management = ModelAssetManagementRecordSet::from_records(vec![
        mesh_model.management_record(mesh_model_record.id()),
    ]);
    assert_eq!(model_management.summary.model_count, 1);
    assert_eq!(model_management.summary.mesh_reference_count, 1);
    assert_eq!(model_management.summary.vertex_count, 3);
    assert_eq!(model_management.summary.index_count, 3);

    let mesh = load_mesh(&manager, "res://models/hero.gltf#Mesh0/Primitive0");
    let mesh_record = resource_record(&manager, "res://models/hero.gltf#Mesh0/Primitive0");
    let mesh_overview = mesh.overview().unwrap();
    assert_eq!(mesh_overview.vertex_count, 3);
    assert_eq!(mesh_overview.index_count, 3);
    assert_eq!(mesh.morph_targets.len(), 1);
    assert_eq!(mesh_overview.morph_target_count, 1);
    assert_eq!(mesh_overview.morph_target_attribute_count, 1);
    assert_eq!(
        mesh.morph_targets[0]
            .attributes
            .get(MESH_ATTRIBUTE_POSITION),
        Some(&MeshAttributeValues::Float32x3(vec![
            [0.0, 0.0, 0.2],
            [0.0, 0.0, 0.2],
            [0.0, 0.0, 0.2],
        ]))
    );
    let mesh_management = MeshAssetManagementRecordSet::from_results(vec![(
        mesh_record.id(),
        mesh.management_record(mesh_record.id()),
    )]);
    assert_eq!(mesh_management.summary.valid_mesh_count, 1);
    assert_eq!(mesh_management.summary.vertex_count, 3);
    assert_eq!(mesh_management.summary.index_count, 3);
    assert_eq!(mesh_management.summary.morph_target_count, 1);
    assert_eq!(mesh_management.summary.morph_target_attribute_count, 1);

    let gltf_material = load_material(&manager, "res://models/hero.gltf#Material0");
    assert_eq!(
        gltf_material
            .base_color_texture
            .as_ref()
            .map(|reference| reference.locator.clone()),
        Some(uri("res://models/hero.gltf#Texture0"))
    );

    let sample_material = load_material(&manager, "res://materials/hero_surface.zmaterial");
    let sample_shader = load_shader(&manager, "res://shaders/lit_sample");
    assert_eq!(
        sample_material.shader.locator,
        uri("res://shaders/lit_sample")
    );
    let base_color_slot = sample_material
        .texture_slots
        .get("base_color")
        .expect("base_color slot");
    assert_eq!(
        base_color_slot
            .reference
            .as_ref()
            .map(|reference| reference.locator.clone()),
        Some(uri("res://textures/hero_albedo_bc1.dds"))
    );
    assert_eq!(base_color_slot.fallback.as_deref(), Some("white"));
    let sample_material_readiness = sample_material.readiness_report_with_shader_contract(
        &sample_shader,
        |reference| {
            manager
                .registry()
                .get_by_locator(&reference.locator)
                .is_some()
        },
        |reference| {
            manager
                .registry()
                .get_by_locator(&reference.locator)
                .is_some()
        },
    );
    assert!(
        sample_material_readiness.is_ready(),
        "{sample_material_readiness:#?}"
    );
    let material_management =
        MaterialAssetManagementRecordSet::from_records(vec![sample_material.management_record(
            resource_record(&manager, "res://materials/hero_surface.zmaterial").id(),
        )]);
    assert_eq!(material_management.summary.material_count, 1);
    assert_eq!(material_management.summary.texture_reference_count, 2);
    assert_eq!(material_management.summary.fallback_texture_slot_count, 1);

    let shader_management = ShaderAssetManagementRecordSet::from_records(vec![
        sample_shader.management_record(resource_record(&manager, "res://shaders/lit_sample").id()),
    ]);
    let aggregate_management = AssetManagementRecordSets::from_record_sets(
        model_management,
        mesh_management,
        scene_management,
        scene_entity_management,
        material_management,
        RenderMaterialManagementRecordSet::default(),
        shader_management,
    );
    assert_eq!(aggregate_management.summary.managed_record_count, 6);
    assert_eq!(aggregate_management.summary.degraded_record_count, 0);
    assert_eq!(
        aggregate_management
            .summary
            .entity_direct_mesh_reference_count,
        1
    );
    assert_eq!(
        aggregate_management
            .summary
            .entity_mesh_primitive_binding_count,
        1
    );
    assert_eq!(aggregate_management.summary.mesh_morph_target_count, 1);
    assert_eq!(
        aggregate_management
            .summary
            .mesh_morph_target_attribute_count,
        1
    );
    assert_eq!(aggregate_management.summary.entity_morph_weight_count, 1);

    let project_management = asset_manager.asset_management_record_sets();
    assert_eq!(project_management.summary.managed_record_count, 17);
    assert_eq!(project_management.summary.degraded_record_count, 2);
    assert_eq!(project_management.summary.model_count, 4);
    assert_eq!(project_management.summary.mesh_count, 1);
    assert_eq!(project_management.summary.valid_mesh_count, 1);
    assert_eq!(project_management.summary.scene_count, 2);
    assert_eq!(project_management.summary.entity_count, 2);
    assert_eq!(project_management.summary.material_count, 5);
    assert_eq!(project_management.summary.material_ready_count, 3);
    assert_eq!(project_management.summary.material_degraded_count, 2);
    assert_eq!(project_management.summary.material_issue_row_count, 2);
    assert_eq!(project_management.summary.prepared_material_count, 0);
    assert_eq!(project_management.summary.shader_count, 3);
    assert_eq!(project_management.summary.shader_issue_row_count, 0);
    assert_eq!(
        project_management
            .summary
            .entity_direct_mesh_reference_count,
        2
    );
    assert_eq!(
        project_management
            .summary
            .entity_mesh_primitive_binding_count,
        2
    );
    assert_eq!(project_management.summary.mesh_morph_target_count, 1);
    assert_eq!(
        project_management.summary.mesh_morph_target_attribute_count,
        1
    );
    assert_eq!(project_management.summary.entity_morph_weight_count, 2);
    assert_eq!(
        asset_manager.asset_management_overview().summary,
        project_management.summary
    );
    assert_eq!(
        asset_manager.asset_management_family_summaries(),
        project_management.families
    );
    assert_eq!(
        asset_manager.asset_management_family_status_index(),
        project_management.family_status_index
    );
    let project_degraded_status_view =
        project_management.family_status_view(AssetManagementFamilyStatus::Degraded);
    assert_eq!(
        project_degraded_status_view.families,
        vec![AssetManagementFamilyKind::Material]
    );
    assert_eq!(project_degraded_status_view.total_record_count, 5);
    assert_eq!(project_degraded_status_view.ready_record_count, 3);
    assert_eq!(project_degraded_status_view.degraded_record_count, 2);
    assert_eq!(project_degraded_status_view.issue_row_count, 2);
    assert_eq!(
        asset_manager.asset_management_family_status_view(AssetManagementFamilyStatus::Degraded),
        project_degraded_status_view
    );
    assert_eq!(
        project_management.family_issue_index.with_issues,
        vec![AssetManagementFamilyKind::Material]
    );
    assert_eq!(
        asset_manager.asset_management_family_issue_index(),
        project_management.family_issue_index
    );
    let project_issue_view =
        project_management.family_issue_view(AssetManagementFamilyIssueBucket::WithIssues);
    assert_eq!(
        project_issue_view.families,
        vec![AssetManagementFamilyKind::Material]
    );
    assert_eq!(project_issue_view.issue_row_count, 2);
    assert_eq!(
        asset_manager
            .asset_management_family_issue_view(AssetManagementFamilyIssueBucket::WithIssues),
        project_issue_view
    );

    let RenderBackend { device, queue, .. } = RenderBackend::new_offscreen().unwrap();
    let texture_layout = texture_bind_group_layout(&device);
    let streamer =
        ResourceStreamer::new_for_test(asset_manager.clone(), &device, &queue, &texture_layout);
    let streamer_management = streamer.asset_management_record_sets();
    assert_eq!(streamer_management, project_management);
    assert_eq!(
        streamer.asset_management_overview().summary,
        streamer_management.summary
    );
    assert_eq!(
        streamer.asset_management_family_summaries(),
        streamer_management.families
    );
    assert_eq!(
        streamer.asset_management_family_status_index(),
        streamer_management.family_status_index
    );
    assert_eq!(
        streamer.asset_management_family_status_view(AssetManagementFamilyStatus::Degraded),
        streamer_management.family_status_view(AssetManagementFamilyStatus::Degraded)
    );
    assert_eq!(
        streamer.asset_management_family_issue_index(),
        streamer_management.family_issue_index
    );
    assert_eq!(
        streamer.asset_management_family_issue_view(AssetManagementFamilyIssueBucket::WithIssues),
        streamer_management.family_issue_view(AssetManagementFamilyIssueBucket::WithIssues)
    );

    let compressed_texture = load_texture(&manager, "res://textures/hero_albedo_bc1.dds");
    assert_eq!(
        compressed_texture
            .upload_readiness(TextureUploadSupport::uncompressed_only())
            .unsupported_reason(),
        Some("gpu device does not support BC compressed textures")
    );

    let _ = fs::remove_dir_all(root);
}

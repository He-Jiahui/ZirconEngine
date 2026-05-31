use crate::asset::{
    AssetManagementFamilyKind, AssetManagementFamilyStatus, AssetManagementRecordSets,
    MaterialAssetManagementRecordSet, MaterialAssetManagementRecordSetSummary,
    MeshAssetManagementRecordFailure, MeshAssetManagementRecordSet,
    MeshAssetManagementRecordSetSummary, ModelAssetManagementRecordSet,
    ModelAssetManagementRecordSetSummary, SceneAssetManagementRecordSet,
    SceneAssetManagementRecordSetSummary, SceneEntityManagementRecordSet,
    SceneEntityManagementRecordSetSummary, ShaderAssetManagementRecordSet,
    ShaderAssetManagementRecordSetSummary,
};
use crate::core::framework::render::{
    RenderMaterialManagementRecordSet, RenderMaterialManagementRecordSummary,
};
use crate::core::resource::ResourceId;

#[test]
fn asset_management_record_sets_summarize_asset_family_lists() {
    let invalid_mesh_id = ResourceId::from_stable_label("mesh:aggregate-invalid");
    let models = ModelAssetManagementRecordSet {
        records: Vec::new(),
        summary: ModelAssetManagementRecordSetSummary {
            model_count: 2,
            primitive_count: 4,
            mesh_referenced_model_count: 2,
            mesh_reference_count: 4,
            ..Default::default()
        },
    };
    let meshes = MeshAssetManagementRecordSet {
        records: Vec::new(),
        failures: vec![MeshAssetManagementRecordFailure {
            mesh_id: invalid_mesh_id,
            diagnostic: "missing position attribute".to_string(),
        }],
        summary: MeshAssetManagementRecordSetSummary {
            mesh_count: 3,
            valid_mesh_count: 2,
            invalid_mesh_count: 1,
            vertex_count: 36,
            ..Default::default()
        },
    };
    let scenes = SceneAssetManagementRecordSet {
        records: Vec::new(),
        summary: SceneAssetManagementRecordSetSummary {
            scene_count: 1,
            entity_count: 5,
            mesh_instance_count: 3,
            ..Default::default()
        },
    };
    let scene_entities = SceneEntityManagementRecordSet {
        records: Vec::new(),
        summary: SceneEntityManagementRecordSetSummary {
            scene_count: 1,
            entity_count: 5,
            active_entity_count: 4,
            root_entity_count: 2,
            direct_reference_count: 8,
            camera_count: 1,
            mesh_instance_count: 3,
            direct_mesh_reference_count: 0,
            mesh_primitive_binding_count: 0,
            mesh_material_binding_count: 3,
            collider_material_binding_count: 1,
            light_count: 2,
            physics_component_count: 2,
            animation_binding_count: 1,
            terrain_count: 1,
            tilemap_count: 1,
            prefab_instance_count: 1,
        },
    };
    let material_assets = MaterialAssetManagementRecordSet {
        records: Vec::new(),
        summary: MaterialAssetManagementRecordSetSummary {
            material_count: 4,
            ready_count: 2,
            issue_material_count: 2,
            property_override_count: 6,
            texture_slot_count: 7,
            texture_reference_count: 5,
            fallback_texture_slot_count: 2,
            validation_error_count: 3,
            validation_diagnostic_count: 2,
            direct_reference_count: 9,
        },
    };
    let materials = RenderMaterialManagementRecordSet {
        summary: RenderMaterialManagementRecordSummary {
            total_count: 6,
            ready_count: 3,
            diagnostic_count: 1,
            fallback_count: 1,
            invalid_count: 1,
            validation_error_count: 7,
            fallback_usage_count: 3,
            diagnostic_row_count: 2,
            ..Default::default()
        },
        ..Default::default()
    };
    let shaders = ShaderAssetManagementRecordSet {
        records: Vec::new(),
        summary: ShaderAssetManagementRecordSetSummary {
            shader_count: 2,
            ready_count: 1,
            not_ready_count: 1,
            validation_diagnostic_count: 5,
            ..Default::default()
        },
    };

    let aggregate = AssetManagementRecordSets::from_record_sets(
        models,
        meshes,
        scenes,
        scene_entities,
        material_assets,
        materials,
        shaders,
    );
    let overview = aggregate.overview();

    assert_eq!(aggregate.summary.managed_record_count, 17);
    assert_eq!(aggregate.summary.degraded_record_count, 4);
    assert_eq!(aggregate.summary.model_count, 2);
    assert_eq!(aggregate.summary.model_mesh_referenced_model_count, 2);
    assert_eq!(aggregate.summary.model_mesh_reference_count, 4);
    assert_eq!(aggregate.summary.mesh_count, 3);
    assert_eq!(aggregate.summary.valid_mesh_count, 2);
    assert_eq!(aggregate.summary.invalid_mesh_count, 1);
    assert_eq!(aggregate.summary.scene_count, 1);
    assert_eq!(aggregate.summary.scene_entity_count, 5);
    assert_eq!(aggregate.summary.entity_count, 5);
    assert_eq!(aggregate.summary.active_entity_count, 4);
    assert_eq!(aggregate.summary.root_entity_count, 2);
    assert_eq!(aggregate.summary.entity_direct_reference_count, 8);
    assert_eq!(aggregate.summary.entity_camera_count, 1);
    assert_eq!(aggregate.summary.entity_mesh_instance_count, 3);
    assert_eq!(aggregate.summary.entity_direct_mesh_reference_count, 0);
    assert_eq!(aggregate.summary.entity_mesh_primitive_binding_count, 0);
    assert_eq!(aggregate.summary.entity_mesh_material_binding_count, 3);
    assert_eq!(aggregate.summary.entity_collider_material_binding_count, 1);
    assert_eq!(aggregate.summary.entity_light_count, 2);
    assert_eq!(aggregate.summary.entity_physics_component_count, 2);
    assert_eq!(aggregate.summary.entity_animation_binding_count, 1);
    assert_eq!(aggregate.summary.entity_terrain_count, 1);
    assert_eq!(aggregate.summary.entity_tilemap_count, 1);
    assert_eq!(aggregate.summary.entity_prefab_instance_count, 1);
    assert_eq!(aggregate.summary.material_count, 4);
    assert_eq!(aggregate.summary.material_ready_count, 2);
    assert_eq!(aggregate.summary.material_degraded_count, 2);
    assert_eq!(aggregate.summary.material_issue_row_count, 5);
    assert_eq!(aggregate.summary.material_property_override_count, 6);
    assert_eq!(aggregate.summary.material_texture_slot_count, 7);
    assert_eq!(aggregate.summary.material_texture_reference_count, 5);
    assert_eq!(aggregate.summary.material_fallback_texture_slot_count, 2);
    assert_eq!(aggregate.summary.material_validation_error_count, 3);
    assert_eq!(aggregate.summary.material_validation_diagnostic_count, 2);
    assert_eq!(aggregate.summary.material_direct_reference_count, 9);
    assert_eq!(aggregate.summary.prepared_material_count, 6);
    assert_eq!(aggregate.summary.prepared_material_ready_count, 3);
    assert_eq!(aggregate.summary.prepared_material_degraded_count, 3);
    assert_eq!(aggregate.summary.prepared_material_issue_row_count, 12);
    assert_eq!(aggregate.summary.shader_count, 2);
    assert_eq!(aggregate.summary.shader_ready_count, 1);
    assert_eq!(aggregate.summary.shader_not_ready_count, 1);
    assert_eq!(aggregate.summary.shader_validation_diagnostic_count, 5);
    assert_eq!(aggregate.meshes.failures[0].mesh_id, invalid_mesh_id);
    assert_eq!(
        aggregate
            .family_summaries()
            .iter()
            .map(|family| (
                family.kind,
                family.status,
                family.total_record_count,
                family.ready_record_count,
                family.degraded_record_count,
                family.issue_row_count,
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                AssetManagementFamilyKind::Model,
                AssetManagementFamilyStatus::Ready,
                2,
                2,
                0,
                0,
            ),
            (
                AssetManagementFamilyKind::Mesh,
                AssetManagementFamilyStatus::Degraded,
                3,
                2,
                1,
                1,
            ),
            (
                AssetManagementFamilyKind::Scene,
                AssetManagementFamilyStatus::Ready,
                1,
                1,
                0,
                0,
            ),
            (
                AssetManagementFamilyKind::Entity,
                AssetManagementFamilyStatus::Ready,
                5,
                5,
                0,
                0,
            ),
            (
                AssetManagementFamilyKind::Material,
                AssetManagementFamilyStatus::Degraded,
                4,
                2,
                2,
                5,
            ),
            (
                AssetManagementFamilyKind::Shader,
                AssetManagementFamilyStatus::Degraded,
                2,
                1,
                1,
                5,
            ),
        ]
    );
    assert!(aggregate.family_status_index.empty.is_empty());
    assert_eq!(
        aggregate
            .family_status_index()
            .families_for_status(AssetManagementFamilyStatus::Ready),
        &[
            AssetManagementFamilyKind::Model,
            AssetManagementFamilyKind::Scene,
            AssetManagementFamilyKind::Entity
        ]
    );
    assert_eq!(
        aggregate.family_status_index.degraded,
        vec![
            AssetManagementFamilyKind::Mesh,
            AssetManagementFamilyKind::Material,
            AssetManagementFamilyKind::Shader,
        ]
    );
    assert_eq!(aggregate.family_status_index.total_family_count(), 6);
    assert_eq!(aggregate.family_status_index.degraded_family_count(), 3);
    assert!(aggregate.family_status_index.has_degraded_families());
    assert_eq!(overview.summary, aggregate.summary);
    assert_eq!(overview.family_summaries(), aggregate.family_summaries());
    assert_eq!(
        overview.family_status_index(),
        aggregate.family_status_index()
    );
}

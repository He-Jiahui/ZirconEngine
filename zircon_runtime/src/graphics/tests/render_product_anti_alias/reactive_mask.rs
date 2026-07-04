use std::{collections::BTreeMap, sync::Arc};

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{AlphaMode, AssetReference, AssetUri, MaterialAsset};
use crate::core::framework::render::{
    AntiAliasSettings, FallbackSkyboxKind, GeometryExtract, GeometryPhaseInput,
    PreviewEnvironmentExtract, RenderFrameExtract, RenderFramework, RenderLayerSet,
    RenderMaterialAlphaMode, RenderMeshSnapshot, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderStats, RenderViewportDescriptor, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::graphics::WgpuRenderFramework;

use super::{
    anti_alias_product_profile, assert_taa_resolve_product_stats,
    submit_and_capture_anti_alias_product, TAA_REACTIVE_MASK_CLEAR_EXECUTOR_ID,
    TAA_REACTIVE_MASK_MESH_EXECUTOR_ID, TAA_RESOLVE_EXECUTOR_ID,
};

#[test]
fn render_product_taa_authored_reactive_mask_records_material_writer_path() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let inert_material = register_taa_reactive_product_material(
        &asset_manager,
        "res://materials/taa-reactive-none.zmaterial",
        0.0,
    );
    let reactive_material = register_taa_reactive_product_material(
        &asset_manager,
        "res://materials/taa-reactive-authored.zmaterial",
        1.0,
    );
    let framework = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();

    framework
        .set_quality_profile(
            viewport,
            anti_alias_product_profile("runtime-taa-authored-reactive-mask", true)
                .with_temporal_history(true),
        )
        .unwrap();

    let (_, inert_stats) = submit_and_capture_anti_alias_product(
        &framework,
        viewport,
        authored_reactive_mask_taa_product_extract(viewport_size, inert_material),
    );
    let (_, reactive_stats) = submit_and_capture_anti_alias_product(
        &framework,
        viewport,
        authored_reactive_mask_taa_product_extract(viewport_size, reactive_material),
    );

    assert_taa_resolve_product_stats(&inert_stats);
    assert_taa_reactive_mask_graph_executed(&inert_stats);
    assert_eq!(inert_stats.last_material_count, 1);
    assert_eq!(inert_stats.last_material_ready_count, 1);
    assert_eq!(inert_stats.last_material_fallback_count, 0);
    assert_eq!(inert_stats.last_mesh_opaque_draw_count, 1);
    assert_eq!(inert_stats.last_mesh_taa_reactive_mask_command_count, 0);

    assert_taa_resolve_product_stats(&reactive_stats);
    assert_taa_reactive_mask_graph_executed(&reactive_stats);
    assert_eq!(reactive_stats.last_material_count, 1);
    assert_eq!(reactive_stats.last_material_ready_count, 1);
    assert_eq!(reactive_stats.last_material_fallback_count, 0);
    assert_eq!(reactive_stats.last_mesh_opaque_draw_count, 1);
    assert_eq!(reactive_stats.last_mesh_taa_reactive_mask_command_count, 1);
}

#[test]
fn render_product_taa_transparent_reactive_mask_records_alpha_writer_path() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let transparent_material = register_taa_reactive_product_material_with_alpha(
        &asset_manager,
        "res://materials/taa-reactive-transparent-alpha.zmaterial",
        0.0,
        AlphaMode::Blend,
        0.45,
    );
    let framework = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();

    framework
        .set_quality_profile(
            viewport,
            anti_alias_product_profile("runtime-taa-transparent-reactive-mask", true)
                .with_temporal_history(true),
        )
        .unwrap();

    let (_, transparent_stats) = submit_and_capture_anti_alias_product(
        &framework,
        viewport,
        authored_reactive_mask_taa_product_extract_with_alpha_mode(
            viewport_size,
            transparent_material,
            RenderMaterialAlphaMode::Blend,
        ),
    );

    assert_taa_resolve_product_stats(&transparent_stats);
    assert_taa_reactive_mask_graph_executed(&transparent_stats);
    assert_eq!(transparent_stats.last_material_count, 1);
    assert_eq!(transparent_stats.last_material_ready_count, 1);
    assert_eq!(transparent_stats.last_material_fallback_count, 0);
    assert_eq!(transparent_stats.last_mesh_opaque_draw_count, 0);
    assert_eq!(transparent_stats.last_mesh_alpha_mask_draw_count, 0);
    assert_eq!(transparent_stats.last_mesh_transparent_draw_count, 1);
    assert_eq!(
        transparent_stats.last_mesh_taa_reactive_mask_command_count,
        1
    );
}

fn register_taa_reactive_product_material(
    asset_manager: &ProjectAssetManager,
    locator: &str,
    strength: f32,
) -> ResourceId {
    register_taa_reactive_product_material_asset(
        asset_manager,
        locator,
        taa_reactive_product_material(strength),
    )
}

fn register_taa_reactive_product_material_with_alpha(
    asset_manager: &ProjectAssetManager,
    locator: &str,
    strength: f32,
    alpha_mode: AlphaMode,
    alpha: f32,
) -> ResourceId {
    register_taa_reactive_product_material_asset(
        asset_manager,
        locator,
        taa_reactive_product_material_with_alpha(strength, alpha_mode, alpha),
    )
}

fn register_taa_reactive_product_material_asset(
    asset_manager: &ProjectAssetManager,
    locator: &str,
    material: MaterialAsset,
) -> ResourceId {
    let material_uri = AssetUri::parse(locator).unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("material insert");
    material_id
}

fn taa_reactive_product_material(strength: f32) -> MaterialAsset {
    taa_reactive_product_material_with_alpha(strength, AlphaMode::Opaque, 1.0)
}

fn taa_reactive_product_material_with_alpha(
    strength: f32,
    alpha_mode: AlphaMode,
    alpha: f32,
) -> MaterialAsset {
    let mut property_values = BTreeMap::new();
    property_values.insert(
        "taa_reactive_mask_strength".to_string(),
        toml::Value::Float(strength as f64),
    );
    MaterialAsset {
        name: Some(format!("TaaReactiveMask{strength:.2}")),
        shader: AssetReference::from_locator(AssetUri::parse("builtin://shader/pbr.wgsl").unwrap()),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: [0.1, 0.85, 0.65, alpha],
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode,
        double_sided: false,
        property_values,
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn authored_reactive_mask_taa_product_extract(
    viewport_size: UVec2,
    material: ResourceId,
) -> RenderFrameExtract {
    authored_reactive_mask_taa_product_extract_with_alpha_mode(
        viewport_size,
        material,
        RenderMaterialAlphaMode::Opaque,
    )
}

fn authored_reactive_mask_taa_product_extract_with_alpha_mode(
    viewport_size: UVec2,
    material: ResourceId,
    alpha_mode: RenderMaterialAlphaMode,
) -> RenderFrameExtract {
    let node_id = 821;
    let mesh = authored_reactive_mask_mesh(node_id, material);
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(803),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)),
                    ..ViewportCameraSnapshot::default()
                },
                meshes: vec![mesh.clone()],
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            environment: crate::core::framework::render::EnvironmentExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    );
    extract.geometry = GeometryExtract::from_meshes_and_phase_inputs(
        extract.view.core_pipeline,
        vec![mesh.clone()],
        vec![GeometryPhaseInput::new(
            node_id,
            0,
            alpha_mode,
            mesh.transform.translation.z,
        )],
    );
    extract.apply_viewport_size(viewport_size);
    extract.view.anti_alias = AntiAliasSettings::taa();
    extract
}

fn authored_reactive_mask_mesh(node_id: u64, material: ResourceId) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: node_id << 16,
        transform_revision: 0,
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, -3.0),
            scale: Vec3::splat(0.8),
            ..Transform::default()
        },
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(material),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        static_state: Default::default(),
        render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
    }
}

fn assert_taa_reactive_mask_graph_executed(stats: &RenderStats) {
    for executor_id in [
        TAA_REACTIVE_MASK_CLEAR_EXECUTOR_ID,
        TAA_REACTIVE_MASK_MESH_EXECUTOR_ID,
        TAA_RESOLVE_EXECUTOR_ID,
    ] {
        assert!(
            stats
                .last_graph_executed_executor_ids
                .contains(&executor_id.to_string()),
            "expected executor `{}` in executed executor ids: {:?}",
            executor_id,
            stats.last_graph_executed_executor_ids
        );
    }
}

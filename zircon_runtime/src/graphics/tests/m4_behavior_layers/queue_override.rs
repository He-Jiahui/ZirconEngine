use super::{RenderFixture, average_channel_in_region, centered_quad_transform};
use crate::asset::AssetUri;
use crate::asset::assets::{AlphaMode, MaterialAsset};
use crate::asset::pipeline::manager::AssetManager;
use crate::core::framework::render::{
    CorePipelineKind, GeometryExtract, GeometryPhaseInput, RenderFramework, RenderLayerSet,
    RenderMaterialAlphaMode, RenderMeshSnapshot, RenderPhase, RenderPhaseMeshSource,
    RenderQualityProfile,
};
use crate::core::math::{UVec2, Vec4};
use crate::core::resource::{
    MaterialMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::scene::components::{Mobility, default_render_layer_mask};

#[test]
fn render_product_queue_override_reorders_draws() {
    let fixture = RenderFixture::new("graphics_m4_queue_override_product", [1.0, 1.0, 1.0, 1.0]);
    let green_queue_override_material = insert_queue_product_material(
        &fixture,
        "res://materials/queue-override-opaque-green.zmaterial",
        AlphaMode::Opaque,
    );
    let red_geometry_material = insert_queue_product_material(
        &fixture,
        "res://materials/queue-geometry-red.zmaterial",
        AlphaMode::Opaque,
    );
    let blue_transparent_material = insert_queue_product_material(
        &fixture,
        "res://materials/queue-transparent-blue.zmaterial",
        AlphaMode::Blend,
    );

    let mut extract = fixture.frame_extract(Vec::new(), Vec::new(), |_| {});
    extract.geometry = GeometryExtract::from_meshes_and_phase_inputs(
        CorePipelineKind::Core3d,
        vec![
            queue_product_mesh(
                &fixture,
                10,
                green_queue_override_material,
                Vec4::new(0.0, 1.0, 0.0, 1.0),
            ),
            queue_product_mesh(
                &fixture,
                20,
                red_geometry_material,
                Vec4::new(1.0, 0.0, 0.0, 1.0),
            ),
            queue_product_mesh(
                &fixture,
                30,
                blue_transparent_material,
                Vec4::new(0.0, 0.0, 1.0, 0.45),
            ),
        ],
        vec![
            GeometryPhaseInput::new(10, 0, RenderMaterialAlphaMode::Opaque, 0.0)
                .with_render_queue(2_900),
            GeometryPhaseInput::new(20, 1, RenderMaterialAlphaMode::Opaque, 0.0),
            GeometryPhaseInput::new(30, 2, RenderMaterialAlphaMode::Blend, 0.0)
                .with_render_queue(3_000),
        ],
    );
    assert_eq!(
        extract
            .geometry
            .phase_queue
            .items
            .iter()
            .map(|item| (item.phase, item.mesh_source))
            .collect::<Vec<_>>(),
        vec![
            (RenderPhase::Opaque3d, RenderPhaseMeshSource::MeshIndex(1)),
            (
                RenderPhase::Transparent3d,
                RenderPhaseMeshSource::MeshIndex(0)
            ),
            (
                RenderPhase::Transparent3d,
                RenderPhaseMeshSource::MeshIndex(2)
            ),
        ]
    );

    let server = fixture.builtin_server();
    let frame = fixture.render_extract(
        &server,
        extract,
        RenderQualityProfile::new("queue-override-product")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false)
            .with_bloom(false)
            .with_color_grading(false)
            .with_anti_alias(false),
    );
    let stats = server.query_stats().unwrap();
    assert!(
        stats
            .last_graph_executed_executor_ids
            .contains(&"mesh.transparent".to_string())
    );

    let sample_origin = UVec2::new(
        fixture.viewport_size.x / 2 - 12,
        fixture.viewport_size.y / 2 - 12,
    );
    let sample_size = UVec2::new(24, 24);
    let red = average_channel_in_region(&frame, sample_origin, sample_size, 0);
    let green = average_channel_in_region(&frame, sample_origin, sample_size, 1);
    let blue = average_channel_in_region(&frame, sample_origin, sample_size, 2);

    assert!(
        green > red + 32.0 && blue > 48.0,
        "queue=2900 should draw after Geometry red and before Transparent blue; red={red:.2}, green={green:.2}, blue={blue:.2}"
    );
}

fn insert_queue_product_material(
    fixture: &RenderFixture,
    material_uri: &str,
    alpha_mode: AlphaMode,
) -> ResourceHandle<MaterialMarker> {
    let material_uri = AssetUri::parse(material_uri).unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    fixture
        .asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            MaterialAsset {
                name: Some("QueueOverrideProduct".to_string()),
                shader: super::asset_reference("res://shaders/flat_color.wgsl"),
                parent: None,
                options: Default::default(),
                queue: None,
                base_color: [1.0, 1.0, 1.0, 1.0],
                base_color_texture: Some(super::asset_reference("res://textures/white.png")),
                normal_texture: None,
                metallic: 0.0,
                roughness: 1.0,
                metallic_roughness_texture: None,
                occlusion_texture: None,
                emissive: [0.0, 0.0, 0.0],
                emissive_texture: None,
                alpha_mode,
                double_sided: false,
                property_values: Default::default(),
                texture_slots: Default::default(),
                validation_diagnostics: Vec::new(),
            },
        )
        .expect("queue product material insert");
    ResourceHandle::new(material_id)
}

fn queue_product_mesh(
    fixture: &RenderFixture,
    node_id: u64,
    material: ResourceHandle<MaterialMarker>,
    tint: Vec4,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: node_id << 16,
        transform_revision: 0,
        transform: centered_quad_transform(0.96),
        model: fixture.model,
        mesh: None,
        material,
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint,
        mobility: Mobility::Dynamic,
        static_state: Default::default(),
        common: crate::core::framework::render::RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(default_render_layer_mask()),
            ..Default::default()
        },
    }
}

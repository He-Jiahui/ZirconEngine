use crate::core::framework::render::render_mesh_stable_instance_key;
use crate::core::framework::render::{
    CorePipelineKind, FallbackSkyboxKind, PreviewEnvironmentExtract, PrimitiveRelevance,
    RenderFrameExtract, RenderLayerSet, RenderMaterialAlphaMode, RenderOverlayExtract,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{UVec2, Vec4};
use crate::graphics::scene::resources::default_pipeline_key;
use crate::graphics::visibility::{
    FrameVisibility, ViewCullingStats, ViewVisibilityContext, VisibilityBounds, VisibilityViewKey,
};
use crate::graphics::ViewportRenderFrame;

fn production_source() -> &'static str {
    include_str!("../build.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("mesh draw builder should retain a test-module boundary")
}

#[test]
fn omitted_direct_light_preparation_skips_packing_and_gpu_light_buffer_writes() {
    let source = production_source();
    let build_function = source
        .split("pub(crate) fn build_mesh_draws(")
        .nth(1)
        .expect("mesh draw builder function");
    let omitted_branch = build_function
        .split("if let Some(direct_lighting_enabled) = direct_lighting_preparation {")
        .nth(1)
        .and_then(|source| source.split("let (gpu_scene_upload_report").next())
        .expect("direct-light preparation branch");

    assert!(omitted_branch.contains("pack_lighting_extract_with_cookies("));
    assert!(omitted_branch.contains("gpu_scene.write_lights("));
    assert!(
        !build_function[..build_function
            .find("if let Some(direct_lighting_enabled) = direct_lighting_preparation {")
            .expect("direct-light preparation gate")]
            .contains("pack_lighting_extract_with_cookies("),
        "light packing must stay behind the profile-controlled preparation gate"
    );
    assert!(
        !build_function[..build_function
            .find("if let Some(direct_lighting_enabled) = direct_lighting_preparation {")
            .expect("direct-light preparation gate")]
            .contains("gpu_scene.write_lights("),
        "GPU light-buffer writes must stay behind the profile-controlled preparation gate"
    );
}

#[test]
fn material_submission_revision_tracks_final_pipeline_and_binding_identities() {
    let pipeline = default_pipeline_key();
    let textures = [11, 13, 17, 19, 23, 29];
    let revision = super::material_submission_revision(
        7,
        &pipeline,
        textures,
        31,
        37,
        crate::core::framework::render::CastShadowsMode::On,
    );

    let mut changed_pipeline = pipeline.clone();
    let resources = crate::core::resource::ResourceManager::new();
    let locator =
        crate::core::resource::ResourceLocator::parse("res://shaders/submission.zshader").unwrap();
    resources
        .register_record(crate::core::resource::ResourceRecord::new(
            pipeline.shader_id,
            crate::core::resource::ResourceKind::Shader,
            locator,
        ))
        .unwrap();
    changed_pipeline.shader_dependency_identity = resources
        .readiness_generation()
        .row_identity(pipeline.shader_id);
    assert_ne!(
        revision,
        super::material_submission_revision(
            7,
            &changed_pipeline,
            textures,
            31,
            37,
            crate::core::framework::render::CastShadowsMode::On,
        ),
        "transitive shader generations must invalidate cached submission payloads"
    );

    let mut changed_textures = textures;
    changed_textures[0] = 43;
    assert_ne!(
        revision,
        super::material_submission_revision(
            7,
            &pipeline,
            changed_textures,
            31,
            37,
            crate::core::framework::render::CastShadowsMode::On,
        ),
        "mip residency resource replacement must invalidate cached material bind groups"
    );
    assert_ne!(
        revision,
        super::material_submission_revision(
            7,
            &pipeline,
            textures,
            31,
            37,
            crate::core::framework::render::CastShadowsMode::TwoSided,
        ),
        "effective renderer shadow raster mode must invalidate cached commands"
    );
    assert_eq!(
        super::material_submission_revision(
            0,
            &pipeline,
            textures,
            31,
            37,
            crate::core::framework::render::CastShadowsMode::On,
        ),
        0,
        "missing source authority must not become cacheable through process-local identities"
    );
}

#[test]
fn mesh_visibility_states_keep_sibling_primitives_independent() {
    let main_visible_stable_instance_key = render_mesh_stable_instance_key(1, 0);
    let shadow_visible_stable_instance_key = render_mesh_stable_instance_key(1, 1);
    let frame = ViewportRenderFrame::from_extract(
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(11),
            RenderSceneSnapshot {
                scene: RenderSceneGeometryExtract {
                    camera: ViewportCameraSnapshot::default(),
                    meshes: Vec::new(),
                    directional_lights: Vec::new(),
                    point_lights: Vec::new(),
                    spot_lights: Vec::new(),
                    ambient_lights: Vec::new(),
                    rect_lights: Vec::new(),
                },
                overlays: RenderOverlayExtract::default(),
                environment: crate::core::framework::render::EnvironmentExtract::default(),
                preview: PreviewEnvironmentExtract {
                    lighting_enabled: true,
                    skybox_enabled: false,
                    fallback_skybox: FallbackSkyboxKind::None,
                    clear_color: Vec4::ZERO,
                },
                virtual_geometry_debug: None,
            },
        ),
        UVec2::new(320, 240),
    )
    .with_frame_visibility(FrameVisibility {
        entities: vec![1, 1],
        stable_instance_keys: vec![
            main_visible_stable_instance_key,
            shadow_visible_stable_instance_key,
        ],
        bounds: vec![
            VisibilityBounds {
                center: crate::core::math::Vec3::new(0.0, 0.0, -5.0),
                radius: 1.0,
            },
            VisibilityBounds {
                center: crate::core::math::Vec3::new(0.0, 8.0, -5.0),
                radius: 1.0,
            },
        ],
        render_layer_masks: vec![
            RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
        ],
        relevance: vec![opaque_shadow_relevance(), opaque_shadow_relevance()],
        relevance_generation: 0,
        views: vec![
            ViewVisibilityContext {
                view: VisibilityViewKey::MainCamera,
                camera: ViewportCameraSnapshot::default(),
                visible: vec![0],
                stats: ViewCullingStats::default(),
            },
            ViewVisibilityContext {
                view: VisibilityViewKey::ShadowCascade {
                    light: 99,
                    cascade: 0,
                },
                camera: ViewportCameraSnapshot::default(),
                visible: vec![1],
                stats: ViewCullingStats::default(),
            },
        ],
    });

    let states = super::mesh_visibility_states(&frame);

    assert_eq!(states.len(), 2);
    let main_receiver = states
        .get(&main_visible_stable_instance_key)
        .expect("main-view receiver state");
    assert!(main_receiver.main_view_visible);
    assert!(!main_receiver.shadow_view_visible);

    let shadow_only_caster = states
        .get(&shadow_visible_stable_instance_key)
        .expect("shadow-only caster state");
    assert!(!shadow_only_caster.main_view_visible);
    assert!(shadow_only_caster.shadow_view_visible);
    assert!(shadow_only_caster.relevance.shadow_caster());
}

fn opaque_shadow_relevance() -> PrimitiveRelevance {
    PrimitiveRelevance::for_mesh_view(
        &RenderLayerSet::layer(0),
        CorePipelineKind::Core3d,
        &RenderLayerSet::layer(0),
        Mobility::Static,
        RenderMaterialAlphaMode::Opaque,
    )
}

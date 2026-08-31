use crate::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::Vec4;
use crate::graphics::{CompiledRenderPipeline, RenderPassStage, RenderPipelineAsset};

#[test]
fn default_forward_plus_keeps_screen_space_ui_as_graph_tail() {
    assert_screen_space_ui_is_terminal(RenderPipelineAsset::default_forward_plus());
}

#[test]
fn default_deferred_keeps_screen_space_ui_as_graph_tail() {
    assert_screen_space_ui_is_terminal(RenderPipelineAsset::default_deferred());
}

fn assert_screen_space_ui_is_terminal(pipeline: RenderPipelineAsset) {
    let compiled = pipeline.compile(&test_extract()).unwrap();

    assert_stage_before(&compiled, RenderPassStage::Overlay, RenderPassStage::Ui);
    assert_stage_before(&compiled, RenderPassStage::Debug, RenderPassStage::Ui);
    assert_pass_before(&compiled, "overlay-gizmo", "runtime-ui");
    assert_eq!(
        compiled
            .graph()
            .passes()
            .last()
            .map(|pass| pass.name.as_str()),
        Some("runtime-ui"),
        "screen-space UI must remain the terminal graph pass for default 3D pipelines"
    );
}

fn assert_stage_before(
    compiled: &CompiledRenderPipeline,
    earlier: RenderPassStage,
    later: RenderPassStage,
) {
    let earlier_index = stage_index(compiled, earlier);
    let later_index = stage_index(compiled, later);
    assert!(
        earlier_index < later_index,
        "{earlier:?} should execute before {later:?}"
    );
}

fn stage_index(compiled: &CompiledRenderPipeline, stage: RenderPassStage) -> usize {
    compiled
        .execution_passes_in_graph_order()
        .position(|execution_pass| execution_pass.stage == stage)
        .unwrap_or_else(|| panic!("compiled pipeline should include {stage:?}"))
}

fn assert_pass_before(compiled: &CompiledRenderPipeline, earlier: &str, later: &str) {
    let earlier_index = pass_index(compiled, earlier);
    let later_index = pass_index(compiled, later);
    assert!(
        earlier_index < later_index,
        "{earlier} should execute before {later}"
    );
}

fn pass_index(compiled: &CompiledRenderPipeline, name: &str) -> usize {
    compiled
        .graph()
        .passes()
        .iter()
        .position(|pass| pass.name == name)
        .unwrap_or_else(|| panic!("compiled pipeline should include {name}"))
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
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
    )
}

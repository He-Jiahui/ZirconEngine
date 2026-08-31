use super::execute_compiled_scene_graph_stages::{
    EARLY_GRAPH_STAGES, FORWARD_PRE_SCENE_GRAPH_STAGES, LATE_GRAPH_STAGES, active_late_graph_stages,
};
use super::execute_graph_stage::RenderGraphStageExecution;
use crate::core::framework::render::RenderPipelineHandle;
use crate::core::framework::render::RenderPluginRendererOutputs;
use crate::graphics::CompiledRenderPipeline;
use crate::graphics::pipeline::{RenderGraphExecutionPassMetadata, RenderPassStage};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionRecord, RenderGraphExecutionResources,
};
use crate::render_graph::{PassFlags, QueueLane, RenderGraphBuilder};

#[test]
fn compiled_scene_graph_stage_lists_keep_early_and_late_boundaries() {
    assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::Opaque2d));
    assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::AlphaMask2d));
    assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::Transparent2d));
    assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::Deferred));
    assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::AmbientOcclusion));
    assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::Lighting));
    assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::AlphaMask3d));
    assert!(LATE_GRAPH_STAGES.contains(&RenderPassStage::Ui));
    assert!(LATE_GRAPH_STAGES.contains(&RenderPassStage::Overlay));
    assert!(LATE_GRAPH_STAGES.contains(&RenderPassStage::Debug));
    assert!(!LATE_GRAPH_STAGES.contains(&RenderPassStage::Present));
    assert_eq!(
        FORWARD_PRE_SCENE_GRAPH_STAGES,
        &[RenderPassStage::AmbientOcclusion, RenderPassStage::Lighting]
    );
    let source = include_str!("execute_compiled_scene_graph_stages.rs");
    assert!(source.contains(
        "if surface_frame.is_some() || frame.output_target().texture_handle().is_some()"
    ));
}

#[test]
fn compiled_scene_stage_execution_keeps_history_reseed_passes_on_invalid_frames() {
    let source = include_str!("execute_compiled_scene_graph_stages.rs");
    let iterator_return = ["-> impl ", "Iterator<Item = RenderPassStage>"].concat();

    assert!(!source.contains("without_history_resources()"));
    assert!(!source.contains("frame.clone()"));
    assert!(source.contains(&iterator_return));
}

#[test]
fn late_stage_selection_consumes_packet_stage_order() {
    let source = include_str!("execute_compiled_scene_graph_stages.rs");

    assert!(source.contains("pipeline\n        .execution_stages_in_graph_order()"));
    assert!(!source.contains("pipeline\n        .execution_batches()"));
    assert!(!source.contains("pipeline\n        .execution_passes_in_graph_order()"));
}

#[test]
fn active_late_graph_stages_follow_compiled_pipeline_order() {
    let default_3d = compiled_pipeline_with_stages(vec![
        RenderPassStage::DepthPrepass,
        RenderPassStage::PostProcess,
        RenderPassStage::Overlay,
        RenderPassStage::Debug,
        RenderPassStage::Ui,
    ]);
    assert_eq!(
        active_late_graph_stages(&default_3d).collect::<Vec<_>>(),
        vec![
            RenderPassStage::Overlay,
            RenderPassStage::Debug,
            RenderPassStage::Ui
        ]
    );

    let core2d = compiled_pipeline_with_stages(vec![
        RenderPassStage::Opaque2d,
        RenderPassStage::PostProcess,
        RenderPassStage::Ui,
        RenderPassStage::Overlay,
        RenderPassStage::Debug,
    ]);
    assert_eq!(
        active_late_graph_stages(&core2d).collect::<Vec<_>>(),
        vec![
            RenderPassStage::Ui,
            RenderPassStage::Overlay,
            RenderPassStage::Debug
        ]
    );
}

#[test]
fn active_late_graph_stages_ignore_culled_passes() {
    let mut builder = RenderGraphBuilder::new("late-stage-culling-test");
    let culled = builder.add_pass("culled-overlay", QueueLane::Graphics);
    let live = builder.add_pass("live-post-process", QueueLane::Graphics);
    builder
        .set_pass_flags(
            live,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .expect("late-stage culling test root");
    let pipeline = CompiledRenderPipeline::from_parts(
        crate::graphics::pipeline::CompiledRenderPipelineParts {
            handle: RenderPipelineHandle::new(101),
            name: "late-stage-culling-test".to_string(),
            renderer_name: "late-stage-culling-test".to_string(),
            execution_pass_metadata: vec![
                RenderGraphExecutionPassMetadata::new(culled, RenderPassStage::Overlay),
                RenderGraphExecutionPassMetadata::new(live, RenderPassStage::PostProcess),
            ],
            enabled_features: Vec::new(),
            required_extract_sections: Vec::new(),
            capability_requirements: Vec::new(),
            history_bindings: Vec::new(),
            environment_ibl_bake_request: None,
            ambient_occlusion_profile: None,
            half_resolution_transparency_depth_sigma:
                crate::core::framework::render::DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA,
            graph: builder.compile().expect("late-stage culling test graph"),
        },
    )
    .expect("late-stage culling test packet");

    assert_eq!(
        active_late_graph_stages(&pipeline).collect::<Vec<_>>(),
        Vec::<RenderPassStage>::new()
    );
}

#[test]
fn compiled_scene_execution_coverage_rejects_duplicate_and_missing_live_passes() {
    let pipeline = compiled_pipeline_with_stages(vec![
        RenderPassStage::Opaque3d,
        RenderPassStage::Transparent3d,
    ]);
    let resources = RenderGraphExecutionResources::new();
    let mut record = RenderGraphExecutionRecord::default();
    let mut plugin_outputs = RenderPluginRendererOutputs::default();
    let mut execution =
        RenderGraphStageExecution::new(&resources, &mut record, &mut plugin_outputs, None, None);

    execution
        .admit_graph_pass(&pipeline, 0)
        .expect("first live pass should be admitted");
    let duplicate = execution
        .admit_graph_pass(&pipeline, 0)
        .expect_err("duplicate live pass must fail closed");
    assert!(duplicate.to_string().contains("more than once"));
    let out_of_range = execution
        .admit_graph_pass(&pipeline, 99)
        .expect_err("out-of-range pass must fail closed");
    assert!(
        out_of_range
            .to_string()
            .contains("references missing graph pass index")
    );
    let missing = execution
        .validate_graph_execution(&pipeline)
        .expect_err("unadmitted live pass must fail closed");
    assert!(
        missing
            .to_string()
            .contains("did not execute live graph pass")
    );
}

fn compiled_pipeline_with_stages(stages: Vec<RenderPassStage>) -> CompiledRenderPipeline {
    let mut builder = RenderGraphBuilder::new("stage-order-test");
    let mut execution_pass_metadata = Vec::with_capacity(stages.len());
    for (index, stage) in stages.into_iter().enumerate() {
        let pass = builder.add_pass(format!("stage-order-test-{index}"), QueueLane::Graphics);
        builder
            .set_pass_flags(
                pass,
                PassFlags {
                    has_side_effects: true,
                    ..PassFlags::default()
                },
            )
            .expect("stage order test root");
        execution_pass_metadata.push(RenderGraphExecutionPassMetadata::new(pass, stage));
    }
    CompiledRenderPipeline::from_parts(crate::graphics::pipeline::CompiledRenderPipelineParts {
        handle: RenderPipelineHandle::new(100),
        name: "stage-order-test".to_string(),
        renderer_name: "stage-order-test".to_string(),
        execution_pass_metadata,
        enabled_features: Vec::new(),
        required_extract_sections: Vec::new(),
        capability_requirements: Vec::new(),
        history_bindings: Vec::new(),
        environment_ibl_bake_request: None,
        ambient_occlusion_profile: None,
        half_resolution_transparency_depth_sigma:
            crate::core::framework::render::DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA,
        graph: builder.compile().expect("stage order test graph"),
    })
    .expect("stage-order test execution packet")
}

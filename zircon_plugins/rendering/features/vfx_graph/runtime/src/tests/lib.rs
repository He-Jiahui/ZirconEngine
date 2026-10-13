use super::*;
use zircon_runtime::render_graph::RenderGraphComputeDispatchExtent;

#[path = "../../../../../test_support/frame_smoke.rs"]
mod frame_smoke;

#[test]
fn vfx_graph_compile_report_requires_spawn_and_material() {
    let report = compile_vfx_graph(&VfxGraphAsset {
        name: "sparks".to_string(),
        max_particles: 1024,
        nodes: vec![VfxGraphNode::SpawnRate {
            particles_per_second: 64.0,
        }],
    });

    assert_eq!(report.simulation_pass, "vfx-graph-simulate");
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("shader graph material")));
}

#[test]
fn vfx_feature_registers_two_runtime_passes_and_dependencies() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(!report.manifest.enabled_by_default);
    assert!(report
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.plugin_id == "particles"));
    assert_eq!(report.extensions.render_features()[0].stage_passes.len(), 2);
    let pass = &report.extensions.render_features()[0].stage_passes[0];
    let workload = pass
        .compute_workload
        .as_ref()
        .expect("vfx graph simulation pass should declare workload");
    assert_eq!(pass.queue, QueueLane::AsyncCompute);
    assert_eq!(workload.pipeline_label, VFX_GRAPH_SIMULATION_PIPELINE_LABEL);
    assert_eq!(workload.workgroup_size, VFX_GRAPH_SIMULATION_WORKGROUP_SIZE);
    assert_eq!(
        workload.dispatch_extent,
        RenderGraphComputeDispatchExtent::Fixed(VFX_GRAPH_SIMULATION_DISPATCH_GROUPS)
    );
}

#[test]
fn vfx_graph_feature_executes_a_real_product_frame() {
    let compile = compile_vfx_graph(&VfxGraphAsset {
        name: "product-frame-smoke".to_string(),
        max_particles: 64,
        nodes: vec![
            VfxGraphNode::SpawnRate {
                particles_per_second: 8.0,
            },
            VfxGraphNode::ShaderGraphMaterial {
                shader_graph: "smoke-material".to_string(),
            },
        ],
    });
    assert!(compile.diagnostics.is_empty(), "{:?}", compile.diagnostics);

    let (frame, stats) = frame_smoke::render_nonempty_product_frame(
        [render_feature_descriptor()],
        render_pass_executor_registrations(),
    );
    assert_eq!((frame.width, frame.height), (96, 72));
    for executor_id in [SIMULATION_EXECUTOR_ID, TRANSPARENT_EXECUTOR_ID] {
        assert!(
            stats
                .last_graph_executed_executor_ids
                .iter()
                .any(|executor| executor == executor_id),
            "VFX feature executor `{executor_id}` should run; actual={:?}",
            stats.last_graph_executed_executor_ids
        );
    }
}

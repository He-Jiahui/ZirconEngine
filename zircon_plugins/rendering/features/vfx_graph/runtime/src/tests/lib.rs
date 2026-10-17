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
    assert!(report.extensions.render_features()[0]
        .stage_passes
        .iter()
        .all(|pass| pass.stage == RenderPassStage::Transparent3d));
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
    let asset = VfxGraphAsset {
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
    };
    let compile = compile_vfx_graph(&asset);
    assert!(compile.diagnostics.is_empty(), "{:?}", compile.diagnostics);

    use zircon_plugin_rendering_shader_graph_runtime::{
        ShaderGraphAsset, ShaderGraphNode, ShaderGraphTarget,
    };
    let resolver = |reference: &str| {
        (reference == "smoke-material").then(|| ShaderGraphAsset {
            name: reference.into(),
            target: ShaderGraphTarget::Material,
            nodes: vec![
                ShaderGraphNode::ConstantColor {
                    id: "color".into(),
                    value: [0.8, 0.2, 0.1, 1.0],
                },
                ShaderGraphNode::ColorOutput {
                    input: "color".into(),
                },
            ],
        })
    };
    let graph = compile_vfx_graph_program(&asset, &resolver)
        .expect("supported VFX graph should lower to a real particle program");
    let layout = graph.particle_program().layout.clone();
    let runtime = VfxRuntime::default();
    runtime
        .configure(
            graph,
            82_003,
            zircon_runtime::core::math::Transform::default(),
        )
        .unwrap();
    runtime.tick(0.5).unwrap();
    runtime.request_gpu_probe().unwrap();
    let (frame, stats) = frame_smoke::render_nonempty_product_frame_with_prepare(
        [runtime.feature_descriptor()],
        runtime.executor_registrations(),
        [runtime.prepare_registration()],
        |extract| {
            // Entity visibility is input only. No CPU counters or neutral GPU-frame payload
            // are injected; simulation and indirect instance count are produced by the GPU.
            extract.particles.emitters.push(82_003);
        },
    );
    let [counters, indirect, particles, alive_indices] = runtime
        .take_gpu_probe()
        .expect("submitted VFX GPU buffers should map");
    assert_eq!(
        counters[0], 4,
        "8 particles/sec over 0.5 sec must spawn four real GPU particles"
    );
    assert_eq!(counters[1], 4);
    assert_eq!(indirect, vec![6, 4, 0, 0]);
    let alive = layout.attribute("alive").unwrap();
    let color = layout.attribute("color").unwrap();
    let position = layout.attribute("position").unwrap();
    for slot in &alive_indices[..counters[0] as usize] {
        assert!(*slot < layout.capacity);
        assert_eq!(
            particles[(alive.word_offset + slot * alive.element_words) as usize],
            1
        );
        assert_eq!(
            f32::from_bits(particles[(color.word_offset + slot * color.element_words) as usize]),
            1.0
        );
        for axis in 0..3 {
            assert!(f32::from_bits(
                particles[(position.word_offset + slot * position.element_words + axis) as usize]
            )
            .is_finite());
        }
    }
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

#[test]
fn vfx_graph_lowering_rejects_unresolved_and_texture_materials() {
    use zircon_plugin_rendering_shader_graph_runtime::{
        ShaderGraphAsset, ShaderGraphNode, ShaderGraphTarget,
    };
    let asset = VfxGraphAsset {
        name: "unsupported".into(),
        max_particles: 64,
        nodes: vec![
            VfxGraphNode::SpawnRate {
                particles_per_second: 8.0,
            },
            VfxGraphNode::ShaderGraphMaterial {
                shader_graph: "graph".into(),
            },
        ],
    };
    assert!(matches!(
        compile_vfx_graph_program(&asset, &|_: &str| None),
        Err(VfxCompileError::UnresolvedMaterial { node_index: 1, .. })
    ));
    let resolver = |_: &str| {
        Some(ShaderGraphAsset {
            name: "texture".into(),
            target: ShaderGraphTarget::Material,
            nodes: vec![
                ShaderGraphNode::TextureSample {
                    id: "sample".into(),
                    binding: 0,
                },
                ShaderGraphNode::ColorOutput {
                    input: "sample".into(),
                },
            ],
        })
    };
    assert!(matches!(
        compile_vfx_graph_program(&asset, &resolver),
        Err(VfxCompileError::Unsupported { node_index: 1, .. })
    ));
}

#[test]
fn vfx_graph_lowering_uses_only_the_requested_emitter() {
    use zircon_plugin_rendering_shader_graph_runtime::{
        ShaderGraphAsset, ShaderGraphNode, ShaderGraphTarget,
    };
    let asset = VfxGraphAsset {
        name: "single-emitter-lowering".into(),
        max_particles: 64,
        nodes: vec![
            VfxGraphNode::SpawnRate {
                particles_per_second: 8.0,
            },
            VfxGraphNode::ShaderGraphMaterial {
                shader_graph: "material".into(),
            },
        ],
    };
    let resolver = |reference: &str| {
        (reference == "material").then(|| ShaderGraphAsset {
            name: reference.into(),
            target: ShaderGraphTarget::Material,
            nodes: vec![
                ShaderGraphNode::ConstantColor {
                    id: "color".into(),
                    value: [1.0; 4],
                },
                ShaderGraphNode::ColorOutput {
                    input: "color".into(),
                },
            ],
        })
    };
    let graph = compile_vfx_graph_program(&asset, &resolver)
        .expect("supported graph should lower exactly its requested emitter");
    let emitters = &graph.particle_asset().emitters;
    assert_eq!(emitters.len(), 1);
    assert_eq!(emitters[0].spawn_rate_per_second, 8.0);
    assert_eq!(emitters[0].max_particles, 64);
    assert_eq!(graph.particle_program().layout.emitter_count, 1);
    assert_eq!(graph.particle_program().layout.capacity, 64);
}

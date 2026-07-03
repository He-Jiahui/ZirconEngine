use zircon_runtime::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassExecutionContext,
    RenderPassExecutorRegistration, RenderPassStage,
};
use zircon_runtime::render_graph::{QueueLane, RenderGraphComputeWorkload};

mod capability;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    RenderingVfxGraphRuntimeFeature,
};

pub const FEATURE_ID: &str = "rendering.vfx_graph";
pub const FEATURE_NAME: &str = "vfx_graph";
pub const SIMULATION_EXECUTOR_ID: &str = "vfx-graph.simulate";
pub const TRANSPARENT_EXECUTOR_ID: &str = "vfx-graph.transparent";
pub const VFX_EMITTER_COMPONENT_TYPE: &str = "rendering.Component.VfxEmitter";
const VFX_GRAPH_SIMULATION_PIPELINE_LABEL: &str = "zircon-vfx-graph-simulate";
const VFX_GRAPH_SIMULATION_WORKGROUP_SIZE: [u32; 3] = [64, 1, 1];
const VFX_GRAPH_SIMULATION_DISPATCH_GROUPS: [u32; 3] = [1, 1, 1];

#[derive(Clone, Debug, PartialEq)]
pub struct VfxGraphAsset {
    pub name: String,
    pub max_particles: u32,
    pub nodes: Vec<VfxGraphNode>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VfxGraphNode {
    SpawnRate { particles_per_second: f32 },
    Lifetime { seconds: f32 },
    Velocity { value: [f32; 3] },
    ColorOverLife { start: [f32; 4], end: [f32; 4] },
    ShaderGraphMaterial { shader_graph: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VfxGraphCompileReport {
    pub simulation_pass: String,
    pub render_pass: String,
    pub diagnostics: Vec<String>,
}

pub fn compile_vfx_graph(asset: &VfxGraphAsset) -> VfxGraphCompileReport {
    let mut diagnostics = Vec::new();
    if asset.max_particles == 0 {
        diagnostics.push(format!("vfx graph `{}` has zero max particles", asset.name));
    }
    if !asset
        .nodes
        .iter()
        .any(|node| matches!(node, VfxGraphNode::SpawnRate { .. }))
    {
        diagnostics.push(format!("vfx graph `{}` has no spawn node", asset.name));
    }
    if !asset
        .nodes
        .iter()
        .any(|node| matches!(node, VfxGraphNode::ShaderGraphMaterial { .. }))
    {
        diagnostics.push(format!(
            "vfx graph `{}` has no shader graph material",
            asset.name
        ));
    }
    VfxGraphCompileReport {
        simulation_pass: "vfx-graph-simulate".to_string(),
        render_pass: "vfx-graph-transparent".to_string(),
        diagnostics,
    }
}

pub fn vfx_emitter_component_descriptor() -> zircon_runtime::plugin::ComponentTypeDescriptor {
    zircon_runtime::plugin::ComponentTypeDescriptor::new(
        VFX_EMITTER_COMPONENT_TYPE,
        zircon_plugin_rendering_runtime::PLUGIN_ID,
        "VFX Emitter",
    )
    .with_property("graph", "asset:vfx_graph", true)
    .with_property("rate_multiplier", "float", true)
}

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        FEATURE_NAME,
        vec![
            "view".to_string(),
            "vfx".to_string(),
            "particles".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent,
                "vfx-graph-simulate",
                QueueLane::AsyncCompute,
            )
            .with_executor_id(SIMULATION_EXECUTOR_ID)
            .with_compute_workload(RenderGraphComputeWorkload::fixed(
                VFX_GRAPH_SIMULATION_PIPELINE_LABEL,
                VFX_GRAPH_SIMULATION_WORKGROUP_SIZE,
                VFX_GRAPH_SIMULATION_DISPATCH_GROUPS,
            ))
            .write_buffer("vfx-particle-state"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent,
                "vfx-graph-transparent",
                QueueLane::Graphics,
            )
            .with_executor_id(TRANSPARENT_EXECUTOR_ID)
            .read_buffer("vfx-particle-state")
            .read_texture("scene-depth")
            .read_texture("scene-color")
            .write_texture("scene-color"),
        ],
    )
}

pub fn render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    vec![
        RenderPassExecutorRegistration::new(SIMULATION_EXECUTOR_ID, noop_render_executor),
        RenderPassExecutorRegistration::new(TRANSPARENT_EXECUTOR_ID, noop_render_executor),
    ]
}

fn noop_render_executor(_context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::render_graph::RenderGraphComputeDispatchExtent;

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
}

//! VFX graph lowering and feature-local execution using the shared particle backend.
use std::sync::{Arc, Mutex};

use super::{
    render_feature_descriptor, vfx_emitter_component_descriptor, VfxGraphAsset, VfxGraphNode,
    SIMULATION_EXECUTOR_ID, TRANSPARENT_EXECUTOR_ID,
};
use zircon_plugin_particles_runtime::{
    compile_particle_gpu_program, particle_graph_runtime_prepare_collector_registration,
    ParticleColorKey, ParticleEmitterAsset, ParticleGpuFragmentMaterial, ParticleGpuProgram,
    ParticleGpuRuntimeOwnerHandle, ParticleGpuTransparentRenderConfig,
    ParticleGpuTransparentRenderParams, ParticleGraphResourceNames, ParticleScalarRange,
    ParticleSimulationBackend, ParticleSystemAsset, ParticleSystemComponent, ParticleVec3Range,
    ParticlesManager,
};
use zircon_plugin_rendering_shader_graph_runtime::{
    compile_shader_graph_to_wgsl, ShaderGraphAsset, ShaderGraphNode, ShaderGraphTarget,
};
use zircon_runtime::core::framework::render::RenderPluginRendererOutputs;
use zircon_runtime::core::math::{Transform, Vec3, Vec4};
use zircon_runtime::graphics::{
    GraphicsError, RenderPassExecutionContext, RenderPassExecutor, RenderPassExecutorRegistration,
    RenderPassGpuResourceFactory, RuntimePrepareCollector, RuntimePrepareCollectorContext,
    RuntimePrepareCollectorRegistration, RuntimePrepareFrameTransaction,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VfxCompileError {
    InvalidParameter {
        node_index: usize,
        reason: String,
    },
    Unsupported {
        node_index: usize,
        reason: String,
    },
    UnresolvedMaterial {
        node_index: usize,
        reference: String,
    },
}

pub trait VfxShaderGraphResolver {
    fn resolve(&self, reference: &str) -> Option<ShaderGraphAsset>;
}
impl<F: Fn(&str) -> Option<ShaderGraphAsset>> VfxShaderGraphResolver for F {
    fn resolve(&self, reference: &str) -> Option<ShaderGraphAsset> {
        self(reference)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VfxCompiledGraph {
    particle_asset: ParticleSystemAsset,
    particle_program: ParticleGpuProgram,
    material: ParticleGpuFragmentMaterial,
}

impl VfxCompiledGraph {
    pub fn particle_asset(&self) -> &ParticleSystemAsset {
        &self.particle_asset
    }
    pub fn particle_program(&self) -> &ParticleGpuProgram {
        &self.particle_program
    }
}

pub fn compile_vfx_graph_program(
    asset: &VfxGraphAsset,
    resolver: &dyn VfxShaderGraphResolver,
) -> Result<VfxCompiledGraph, VfxCompileError> {
    let invalid = |node_index, reason: &str| VfxCompileError::InvalidParameter {
        node_index,
        reason: reason.into(),
    };
    if asset.max_particles == 0
        || asset.max_particles > zircon_plugin_particles_runtime::PARTICLE_GPU_MAX_PARTICLES
    {
        return Err(invalid(0, "particle capacity is outside the backend range"));
    }
    let mut emitter =
        ParticleEmitterAsset::sprite(&asset.name).with_max_particles(asset.max_particles);
    let mut seen = [false; 5];
    let mut material = None;
    for (index, node) in asset.nodes.iter().enumerate() {
        let kind = match node {
            VfxGraphNode::SpawnRate { .. } => 0,
            VfxGraphNode::Lifetime { .. } => 1,
            VfxGraphNode::Velocity { .. } => 2,
            VfxGraphNode::ColorOverLife { .. } => 3,
            VfxGraphNode::ShaderGraphMaterial { .. } => 4,
        };
        if std::mem::replace(&mut seen[kind], true) {
            return Err(invalid(index, "duplicate node has ambiguous semantics"));
        }
        match node {
            VfxGraphNode::SpawnRate {
                particles_per_second,
            } => {
                if !particles_per_second.is_finite() || *particles_per_second < 0.0 {
                    return Err(invalid(index, "spawn rate must be finite and nonnegative"));
                }
                emitter.spawn_rate_per_second = *particles_per_second;
            }
            VfxGraphNode::Lifetime { seconds } => {
                if !seconds.is_finite() || *seconds <= 0.0 {
                    return Err(invalid(index, "lifetime must be finite and positive"));
                }
                emitter.lifetime = ParticleScalarRange::constant(*seconds);
            }
            VfxGraphNode::Velocity { value } => {
                if !value.iter().all(|v| v.is_finite()) {
                    return Err(invalid(index, "velocity must be finite"));
                }
                emitter.initial_velocity = ParticleVec3Range::constant(Vec3::from_array(*value));
            }
            VfxGraphNode::ColorOverLife { start, end } => {
                if !start.iter().chain(end).all(|v| v.is_finite()) {
                    return Err(invalid(index, "color must be finite"));
                }
                emitter.start_color = Vec4::ONE;
                emitter.color_over_lifetime = vec![
                    ParticleColorKey::new(0.0, Vec4::from_array(*start)),
                    ParticleColorKey::new(1.0, Vec4::from_array(*end)),
                ];
            }
            VfxGraphNode::ShaderGraphMaterial { shader_graph } => {
                let graph = resolver.resolve(shader_graph).ok_or_else(|| {
                    VfxCompileError::UnresolvedMaterial {
                        node_index: index,
                        reference: shader_graph.clone(),
                    }
                })?;
                if graph.target != ShaderGraphTarget::Material
                    || graph.nodes.iter().any(|node| {
                        matches!(
                            node,
                            ShaderGraphNode::TextureSample { .. }
                                | ShaderGraphNode::MaterialOutput { .. }
                        )
                    })
                {
                    return Err(VfxCompileError::Unsupported { node_index: index, reason: "particle materials require a binding-free color Material shader graph; texture bindings and roughness have no particle consumer".into() });
                }
                let compiled = compile_shader_graph_to_wgsl(&graph);
                if !compiled.diagnostics.is_empty() {
                    return Err(invalid(index, &compiled.diagnostics.join("; ")));
                }
                material = Some(ParticleGpuFragmentMaterial {
                    wgsl: compiled.wgsl,
                });
            }
        }
    }
    if !seen[0] {
        return Err(invalid(0, "graph has no spawn node"));
    }
    let material = material.ok_or_else(|| invalid(0, "graph has no shader graph material"))?;
    let particle_asset = ParticleSystemAsset::new(&asset.name)
        .with_backend(ParticleSimulationBackend::Gpu)
        .with_emitters(vec![emitter]);
    let particle_program = compile_particle_gpu_program(&particle_asset);
    if !particle_program.diagnostics.is_empty() {
        return Err(invalid(
            0,
            "particle backend cannot lower this graph without diagnostics",
        ));
    }
    Ok(VfxCompiledGraph {
        particle_asset,
        particle_program,
        material,
    })
}

const RESOURCES: ParticleGraphResourceNames = ParticleGraphResourceNames {
    particles_a: "vfx.gpu.particles-a",
    particles_b: "vfx.gpu.particles-b",
    emitter_params: "vfx.gpu.emitter-params",
    counters: "vfx.gpu.counters",
    alive_indices: "vfx.gpu.alive-indices",
    indirect_draw_args: "vfx.gpu.indirect-draw-args",
};

/// Each configured feature owns one particle population and transaction stream. Independent
/// features never share aliases or mutexes; preparation and both executors share this instance.
#[derive(Clone, Default)]
pub struct VfxRuntime {
    manager: ParticlesManager,
    owner: ParticleGpuRuntimeOwnerHandle,
    graph: Arc<Mutex<Option<VfxCompiledGraph>>>,
    probe: Arc<Mutex<ProbeState>>,
}

#[derive(Default)]
struct ProbeState {
    requested: bool,
    submitted: bool,
    pending: Option<ProbePending>,
}
struct ProbePending {
    buffer: wgpu::Buffer,
    lengths: [u64; 4],
    wait: Box<dyn FnOnce() -> Result<(), String> + Send>,
}

impl VfxRuntime {
    pub fn configure(
        &self,
        graph: VfxCompiledGraph,
        entity: u64,
        transform: Transform,
    ) -> Result<(), String> {
        let mut configured = self.graph.lock().map_err(|_| "VFX graph lock poisoned")?;
        if configured.is_some() {
            return Err("VFX runtime is already configured".into());
        }
        self.owner
            .lock()
            .map_err(|e| e.to_string())?
            .set_fragment_material(Some(graph.material.clone()))
            .map_err(|e| e.to_string())?;
        self.manager
            .instantiate(
                ParticleSystemComponent::new(entity, graph.particle_asset.clone())
                    .with_transform(transform),
            )
            .map_err(|e| e.to_string())?;
        *configured = Some(graph);
        Ok(())
    }

    pub fn tick(&self, dt: f32) -> Result<(), String> {
        self.manager.tick(dt).map_err(|e| e.to_string())
    }

    pub fn feature_descriptor(&self) -> zircon_runtime::graphics::RenderFeatureDescriptor {
        let mut descriptor = render_feature_descriptor();
        if let Ok(graph) = self.graph.lock() {
            if let Some(graph) = graph.as_ref() {
                descriptor.stage_passes[0].compute_workload = Some(
                    zircon_runtime::render_graph::RenderGraphComputeWorkload::fixed(
                        super::VFX_GRAPH_SIMULATION_PIPELINE_LABEL,
                        super::VFX_GRAPH_SIMULATION_WORKGROUP_SIZE,
                        [graph.particle_program.layout.dispatch_workgroups(64), 1, 1],
                    ),
                );
            }
        }
        descriptor
    }

    pub fn prepare_registration(&self) -> RuntimePrepareCollectorRegistration {
        let delegate = particle_graph_runtime_prepare_collector_registration(
            "vfx-graph.particle-prepare",
            self.manager.clone(),
            self.owner.clone(),
            RESOURCES,
        );
        RuntimePrepareCollectorRegistration::new_collector(
            "vfx-graph.runtime-prepare",
            Arc::new(VfxPrepare {
                delegate,
                probe: Arc::clone(&self.probe),
            }),
        )
    }

    pub fn executor_registrations(&self) -> Vec<RenderPassExecutorRegistration> {
        vec![
            RenderPassExecutorRegistration::new_executor(
                SIMULATION_EXECUTOR_ID,
                Arc::new(VfxExecutor {
                    runtime: self.clone(),
                    simulation: true,
                }),
            ),
            RenderPassExecutorRegistration::new_executor(
                TRANSPARENT_EXECUTOR_ID,
                Arc::new(VfxExecutor {
                    runtime: self.clone(),
                    simulation: false,
                }),
            ),
        ]
    }

    /// Requests one bounded diagnostic copy. Bytes are copied by the real simulate executor,
    /// and may only be consumed after successful product submission.
    pub fn request_gpu_probe(&self) -> Result<(), String> {
        let mut probe = self.probe.lock().map_err(|_| "VFX probe lock poisoned")?;
        if probe.requested || probe.pending.is_some() {
            return Err("VFX GPU probe already pending".into());
        }
        probe.requested = true;
        probe.submitted = false;
        Ok(())
    }

    pub fn take_gpu_probe(&self) -> Result<[Vec<u32>; 4], String> {
        let pending = {
            let mut probe = self.probe.lock().map_err(|_| "VFX probe lock poisoned")?;
            if !probe.submitted {
                return Err("VFX GPU probe has no successful submission".into());
            }
            probe.submitted = false;
            probe
                .pending
                .take()
                .ok_or("VFX GPU probe was not encoded")?
        };
        let slice = pending.buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result.map_err(|e| e.to_string()));
        });
        (pending.wait)()?;
        receiver
            .recv_timeout(std::time::Duration::from_secs(30))
            .map_err(|e| e.to_string())??;
        let mapped = slice.get_mapped_range();
        let mut streams: [Vec<u32>; 4] = Default::default();
        let mut offset = 0;
        for (stream, bytes) in streams.iter_mut().zip(pending.lengths) {
            let end = offset + usize::try_from(bytes).map_err(|_| "VFX probe length overflow")?;
            *stream = mapped
                .get(offset..end)
                .ok_or("VFX probe range mismatch")?
                .chunks_exact(4)
                .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect();
            offset = end;
        }
        drop(mapped);
        pending.buffer.unmap();
        Ok(streams)
    }
}

struct VfxPrepare {
    delegate: RuntimePrepareCollectorRegistration,
    probe: Arc<Mutex<ProbeState>>,
}
impl RuntimePrepareCollector for VfxPrepare {
    fn requests_gpu_readback(&self) -> bool {
        true
    }
    fn collect(
        &self,
        context: &mut RuntimePrepareCollectorContext<'_>,
    ) -> Result<RenderPluginRendererOutputs, GraphicsError> {
        let requested = self
            .probe
            .lock()
            .map_err(|_| GraphicsError::Asset("VFX probe lock poisoned".into()))?
            .requested;
        if requested {
            let commit_probe = Arc::clone(&self.probe);
            let rollback_probe = Arc::clone(&self.probe);
            context.register_frame_transaction(RuntimePrepareFrameTransaction::new(
                "vfx-graph.diagnostic-probe",
                move || {
                    if let Ok(mut probe) = commit_probe.lock() {
                        probe.submitted = probe.pending.is_some();
                    }
                },
                move || {
                    if let Ok(mut probe) = rollback_probe.lock() {
                        probe.pending = None;
                        probe.requested = false;
                        probe.submitted = false;
                    }
                },
            ));
        }
        self.delegate.collect(context)
    }
}

struct VfxExecutor {
    runtime: VfxRuntime,
    simulation: bool,
}
impl RenderPassExecutor for VfxExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        let expected = if self.simulation {
            SIMULATION_EXECUTOR_ID
        } else {
            TRANSPARENT_EXECUTOR_ID
        };
        if context.executor_id.as_str() != expected {
            return Err("VFX executor identity mismatch".into());
        }
        let gpu = context
            .gpu_mut()
            .ok_or("VFX requires a real GPU execution context")?;
        let mut owner = self.runtime.owner.lock().map_err(|e| e.to_string())?;
        if self.simulation {
            {
                let native = gpu.native_context();
                owner
                    .record_prepared_simulation(native.encoder)
                    .map_err(|e| e.to_string())?;
            }
            let mut probe = self
                .runtime
                .probe
                .lock()
                .map_err(|_| "VFX probe lock poisoned")?;
            if probe.requested {
                let bindings = owner.active_bindings().map_err(|e| e.to_string())?;
                let buffers = [
                    bindings.counters,
                    bindings.indirect_draw_args,
                    bindings.particles_b,
                    bindings.alive_indices,
                ];
                let lengths = buffers.map(|buffer| buffer.size());
                let total = lengths
                    .iter()
                    .try_fold(0_u64, |sum, length| sum.checked_add(*length))
                    .ok_or("VFX probe size overflow")?;
                if total > 64 * 1024 * 1024 {
                    return Err("VFX probe exceeds the 64 MiB diagnostic bound".into());
                }
                let native = gpu.native_context();
                let staging = native.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("vfx-real-particle-probe"),
                    contents: &vec![
                        0;
                        usize::try_from(total).map_err(|_| "VFX probe size overflow")?
                    ],
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                });
                let mut offset = 0;
                for (buffer, length) in buffers.into_iter().zip(lengths) {
                    native
                        .encoder
                        .copy_buffer_to_buffer(buffer, 0, &staging, offset, length);
                    offset += length;
                }
                let wait = Box::new(native.readback_completion_waiter());
                probe.pending = Some(ProbePending {
                    buffer: staging,
                    lengths,
                    wait,
                });
                probe.requested = false;
            }
        } else {
            let rendered = gpu.record_particle_gpu_transparent_to_resources(
                "scene-color",
                "scene-depth",
                |draw| {
                    owner
                        .record_transparent_render(
                            draw.device,
                            draw.scene_bind_group_layout,
                            ParticleGpuTransparentRenderConfig::new(
                                draw.target_format,
                                draw.depth_format,
                            ),
                            draw.buffer_uploads,
                            draw.encoder,
                            draw.color_view,
                            draw.depth_view,
                            draw.scene_bind_group,
                            ParticleGpuTransparentRenderParams::new(
                                draw.camera_right,
                                draw.camera_up,
                                1.0,
                            ),
                            draw.render_region,
                        )
                        .map_err(|e| e.to_string())
                },
            )?;
            if !rendered {
                return Err("VFX transparent pass has no encoded particle population".into());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct RenderingVfxGraphRuntimeFeature {
    runtime: VfxRuntime,
}
impl RenderingVfxGraphRuntimeFeature {
    pub fn with_runtime(runtime: VfxRuntime) -> Self {
        Self { runtime }
    }
    pub fn runtime(&self) -> &VfxRuntime {
        &self.runtime
    }
}
impl zircon_runtime::plugin::RuntimePluginFeature for RenderingVfxGraphRuntimeFeature {
    fn manifest(&self) -> zircon_runtime::plugin::PluginFeatureBundleManifest {
        feature_manifest()
    }
    fn register(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_component(vfx_emitter_component_descriptor())?;
        registry.register_render_feature(self.runtime.feature_descriptor())?;
        registry.register_runtime_prepare_collector(self.runtime.prepare_registration())?;
        for registration in self.runtime.executor_registrations() {
            registry.register_render_pass_executor(registration)?;
        }
        Ok(())
    }
}
pub fn runtime_plugin_feature() -> RenderingVfxGraphRuntimeFeature {
    RenderingVfxGraphRuntimeFeature::default()
}
pub fn plugin_feature_registration(
) -> zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport {
    zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport::from_feature(
        &runtime_plugin_feature(),
    )
}
pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_runtime::feature_manifest(
        zircon_plugin_rendering_runtime::RenderingFeatureKind::VfxGraph,
    )
}

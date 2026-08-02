use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use zircon_runtime::core::framework::render::{
    RenderParticleGpuFrameExtract, RenderParticleGpuReadbackOutputs,
};
use zircon_runtime::core::math::Real;

use crate::{ParticleEmitterHandle, ParticleSimulationBackend, ParticleSystemAsset};

use super::backend::{ParticleGpuBackend, ParticleGpuBackendError, ParticleGpuBuffers};
use super::planner::{ParticleGpuFrameParams, ParticleGpuFramePlanner};
use super::ParticleGpuReadbackRequest;
use crate::service::ParticleGpuRuntimeInstance;

pub struct ParticleGpuRuntimeFrame {
    pub outputs: RenderParticleGpuReadbackOutputs,
}

pub struct ParticleGpuRuntimeBufferBindings<'a> {
    pub particles_a: &'a wgpu::Buffer,
    pub particles_b: &'a wgpu::Buffer,
    pub emitter_params: &'a wgpu::Buffer,
    pub alive_indices: &'a wgpu::Buffer,
    pub indirect_draw_args: &'a wgpu::Buffer,
    pub counters: &'a wgpu::Buffer,
    pub debug_readback: &'a wgpu::Buffer,
}

#[derive(Clone, Default)]
pub struct ParticleGpuRuntimeOwnerHandle {
    inner: Arc<Mutex<ParticleGpuRuntimeOwner>>,
}

impl ParticleGpuRuntimeOwnerHandle {
    pub fn new(owner: ParticleGpuRuntimeOwner) -> Self {
        Self {
            inner: Arc::new(Mutex::new(owner)),
        }
    }

    pub fn lock(
        &self,
    ) -> Result<MutexGuard<'_, ParticleGpuRuntimeOwner>, ParticleGpuRuntimeOwnerError> {
        self.inner
            .lock()
            .map_err(|_| ParticleGpuRuntimeOwnerError::Poisoned)
    }
}

#[derive(Debug)]
pub enum ParticleGpuRuntimeOwnerError {
    Backend(ParticleGpuBackendError),
    Simulation(crate::ParticleSimulationError),
    MissingExecutedBackend,
    Poisoned,
}

impl fmt::Display for ParticleGpuRuntimeOwnerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backend(error) => write!(f, "{error}"),
            Self::Simulation(error) => write!(f, "{error}"),
            Self::MissingExecutedBackend => {
                write!(f, "particle GPU runtime owner has no executed backend")
            }
            Self::Poisoned => write!(f, "particle GPU runtime owner mutex poisoned"),
        }
    }
}

impl std::error::Error for ParticleGpuRuntimeOwnerError {}

impl From<ParticleGpuBackendError> for ParticleGpuRuntimeOwnerError {
    fn from(value: ParticleGpuBackendError) -> Self {
        Self::Backend(value)
    }
}

impl From<crate::ParticleSimulationError> for ParticleGpuRuntimeOwnerError {
    fn from(value: crate::ParticleSimulationError) -> Self {
        Self::Simulation(value)
    }
}

#[derive(Default)]
pub struct ParticleGpuRuntimeOwner {
    states: BTreeMap<ParticleEmitterHandle, ParticleGpuRuntimeState>,
    aggregate_asset: Option<ParticleSystemAsset>,
    aggregate_backend: Option<ParticleGpuBackend>,
    aggregate_executed: bool,
}

impl ParticleGpuRuntimeOwner {
    pub fn execute_instances(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        instances: &[ParticleGpuRuntimeInstance],
    ) -> Result<Option<ParticleGpuRuntimeFrame>, ParticleGpuRuntimeOwnerError> {
        self.retain_instances(instances);
        let playing_instances = instances
            .iter()
            .filter(|instance| instance.playing)
            .collect::<Vec<_>>();
        if playing_instances.is_empty() {
            self.aggregate_executed = false;
            return Ok(None);
        }

        for instance in &playing_instances {
            if !self.states.contains_key(&instance.handle) {
                self.states
                    .insert(instance.handle, ParticleGpuRuntimeState::new(instance));
            }
            self.states
                .get_mut(&instance.handle)
                .expect("particle GPU runtime state was inserted")
                .sync_asset(instance);
        }

        let aggregate_asset = aggregate_asset_for(&playing_instances);
        let aggregate_changed = self
            .aggregate_asset
            .as_ref()
            .map_or(true, |existing| existing != &aggregate_asset);
        if aggregate_changed {
            self.aggregate_backend = Some(ParticleGpuBackend::new(device, &aggregate_asset)?);
            self.aggregate_asset = Some(aggregate_asset);
            self.aggregate_executed = false;
        }

        let aggregate_frame = self.aggregate_frame_for(&playing_instances)?;
        self.aggregate_backend
            .as_mut()
            .ok_or(ParticleGpuRuntimeOwnerError::MissingExecutedBackend)?
            .execute_frame(
                queue,
                encoder,
                &aggregate_frame,
                ParticleGpuReadbackRequest::None,
            )?;
        self.aggregate_executed = true;

        Ok(Some(ParticleGpuRuntimeFrame {
            outputs: readback_outputs_from_frame(aggregate_frame.expected_frame_extract()),
        }))
    }

    pub fn active_bindings(
        &self,
    ) -> Result<ParticleGpuRuntimeBufferBindings<'_>, ParticleGpuRuntimeOwnerError> {
        let backend = self
            .aggregate_backend
            .as_ref()
            .filter(|_| self.aggregate_executed)
            .ok_or(ParticleGpuRuntimeOwnerError::MissingExecutedBackend)?;
        Ok(bindings_from_buffers(backend.active_buffers()))
    }

    pub fn record_transparent_render(
        &mut self,
        device: &wgpu::Device,
        scene_layout: &wgpu::BindGroupLayout,
        config: super::ParticleGpuTransparentRenderConfig,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        params: super::ParticleGpuTransparentRenderParams,
        render_region: zircon_runtime::graphics::ViewportRenderRegion,
    ) -> Result<bool, ParticleGpuRuntimeOwnerError> {
        let Some(backend) = self
            .aggregate_backend
            .as_mut()
            .filter(|_| self.aggregate_executed)
        else {
            return Ok(false);
        };
        if !backend.transparent_render_enabled() {
            backend.enable_transparent_rendering(device, scene_layout, config);
        }
        backend.record_transparent_render(
            queue,
            encoder,
            color_view,
            depth_view,
            scene_bind_group,
            params,
            render_region,
        )?;
        Ok(true)
    }

    fn retain_instances(&mut self, instances: &[ParticleGpuRuntimeInstance]) {
        self.states
            .retain(|handle, _| instances.iter().any(|instance| instance.handle == *handle));
        if self.states.is_empty() {
            self.aggregate_asset = None;
            self.aggregate_backend = None;
            self.aggregate_executed = false;
        }
    }

    fn aggregate_frame_for(
        &mut self,
        instances: &[&ParticleGpuRuntimeInstance],
    ) -> Result<ParticleGpuFrameParams, ParticleGpuRuntimeOwnerError> {
        let layout = &self
            .aggregate_backend
            .as_ref()
            .ok_or(ParticleGpuRuntimeOwnerError::MissingExecutedBackend)?
            .program()
            .layout;
        let mut emitters = Vec::with_capacity(layout.emitter_count as usize);
        let mut aggregate_emitter_index = 0usize;
        let mut max_dt: Real = 0.0;
        let mut max_age_seconds: Real = 0.0;

        for instance in instances {
            let state = self
                .states
                .get_mut(&instance.handle)
                .ok_or(ParticleGpuRuntimeOwnerError::MissingExecutedBackend)?;
            let dt = state.frame_delta(instance.age_seconds);
            let frame = state
                .planner
                .build_frame(dt, instance.component.transform)?;
            max_dt = max_dt.max(frame.dt);
            max_age_seconds = max_age_seconds.max(frame.age_seconds);

            for emitter in frame.emitters {
                let Some(layout_emitter) = layout.emitters.get(aggregate_emitter_index) else {
                    break;
                };
                let mut emitter = emitter;
                emitter.emitter_index = layout_emitter.emitter_index;
                emitter.base_slot = layout_emitter.base_slot;
                emitter.capacity = layout_emitter.capacity;
                emitter.spawn_count = emitter.spawn_count.min(layout_emitter.capacity);
                emitters.push(emitter);
                aggregate_emitter_index += 1;
            }
        }

        Ok(ParticleGpuFrameParams {
            dt: max_dt,
            age_seconds: max_age_seconds,
            emitters,
        })
    }
}

struct ParticleGpuRuntimeState {
    asset: crate::ParticleSystemAsset,
    planner: ParticleGpuFramePlanner,
    last_age_seconds: Real,
}

impl ParticleGpuRuntimeState {
    fn new(instance: &ParticleGpuRuntimeInstance) -> Self {
        let asset = instance.component.asset.clone();
        let planner = ParticleGpuFramePlanner::new(asset.clone());
        Self {
            asset,
            planner,
            last_age_seconds: 0.0,
        }
    }

    fn sync_asset(&mut self, instance: &ParticleGpuRuntimeInstance) {
        if self.asset == instance.component.asset {
            return;
        }
        self.asset = instance.component.asset.clone();
        self.planner = ParticleGpuFramePlanner::new(self.asset.clone());
        self.last_age_seconds = 0.0;
    }

    fn frame_delta(&mut self, age_seconds: Real) -> Real {
        let age_seconds = age_seconds.max(0.0);
        if age_seconds < self.last_age_seconds {
            self.planner.reset();
            self.last_age_seconds = 0.0;
        }
        let dt = age_seconds - self.last_age_seconds;
        self.last_age_seconds = age_seconds;
        dt
    }
}

fn aggregate_asset_for(instances: &[&ParticleGpuRuntimeInstance]) -> ParticleSystemAsset {
    let mut emitters = Vec::new();
    for instance in instances {
        emitters.extend(
            instance
                .component
                .asset
                .emitters
                .iter()
                .cloned()
                .map(|mut emitter| {
                    emitter.id = format!(
                        "{}:{}:{}",
                        instance.handle.raw(),
                        instance.component.asset.id,
                        emitter.id
                    );
                    emitter
                }),
        );
    }

    ParticleSystemAsset::new("runtime-prepare-aggregate")
        .with_backend(ParticleSimulationBackend::Gpu)
        .with_emitters(emitters)
}

fn readback_outputs_from_frame(
    frame: RenderParticleGpuFrameExtract,
) -> RenderParticleGpuReadbackOutputs {
    RenderParticleGpuReadbackOutputs {
        alive_count: frame.alive_count,
        spawned_total: frame.spawned_total,
        debug_flags: 0,
        per_emitter_spawned: frame.per_emitter_spawned,
        indirect_draw_args: frame.indirect_draw_args,
    }
}

fn bindings_from_buffers(buffers: ParticleGpuBuffers<'_>) -> ParticleGpuRuntimeBufferBindings<'_> {
    ParticleGpuRuntimeBufferBindings {
        particles_a: buffers.particles_a,
        particles_b: buffers.particles_b,
        emitter_params: buffers.emitter_params,
        alive_indices: buffers.alive_indices,
        indirect_draw_args: buffers.indirect_draw_args,
        counters: buffers.counters,
        debug_readback: buffers.debug_readback,
    }
}

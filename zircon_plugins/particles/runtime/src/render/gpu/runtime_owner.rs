use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use zircon_runtime::core::framework::render::{
    RenderParticleGpuFrameExtract, RenderParticleGpuReadbackOutputs,
};
use zircon_runtime::core::math::Real;
use zircon_runtime::graphics::{RenderPassBufferUploadSink, RuntimePrepareDeviceEpoch};

use crate::{ParticleEmitterHandle, ParticleSimulationBackend, ParticleSystemAsset};

use super::ParticleGpuReadbackRequest;
use super::backend::{
    ParticleGpuBackend, ParticleGpuBackendError, ParticleGpuBackendFrameCommit, ParticleGpuBuffers,
};
use super::neutral_buffers::ParticleGpuNeutralBuffers;
use super::planner::{ParticleGpuFrameParams, ParticleGpuFramePlanner, ParticleGpuPlannerCommit};
use crate::service::ParticleGpuRuntimeInstance;

pub struct ParticleGpuRuntimeFrame {
    pub outputs: RenderParticleGpuReadbackOutputs,
    transaction_id: u64,
}

impl ParticleGpuRuntimeFrame {
    pub(crate) fn transaction_id(&self) -> u64 {
        self.transaction_id
    }
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

    pub(crate) fn commit_frame_transaction(&self, transaction_id: u64) {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .commit_frame_transaction(transaction_id);
    }

    pub(crate) fn rollback_frame_transaction(&self, transaction_id: u64) {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .rollback_frame_transaction(transaction_id);
    }
}

#[derive(Debug)]
pub enum ParticleGpuRuntimeOwnerError {
    Backend(ParticleGpuBackendError),
    Simulation(crate::ParticleSimulationError),
    MissingExecutedBackend,
    PendingFrameTransaction,
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
            Self::PendingFrameTransaction => {
                write!(
                    f,
                    "particle GPU runtime owner still has a pending frame transaction"
                )
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
    active_device_epoch: Option<RuntimePrepareDeviceEpoch>,
    states: BTreeMap<ParticleEmitterHandle, ParticleGpuRuntimeState>,
    aggregate_asset: Option<ParticleSystemAsset>,
    aggregate_backend: Option<ParticleGpuBackend>,
    aggregate_executed: bool,
    pending_frame: Option<ParticleGpuRuntimePendingFrame>,
    next_frame_transaction_id: u64,
    neutral_buffers: ParticleGpuNeutralBuffers,
}

struct ParticleGpuRuntimePendingFrame {
    transaction_id: u64,
    backend_commit: ParticleGpuBackendFrameCommit,
    state_commits: Vec<ParticleGpuRuntimeStateCommit>,
}

struct ParticleGpuRuntimeStateCommit {
    handle: ParticleEmitterHandle,
    planner: ParticleGpuPlannerCommit,
    last_age_seconds: Real,
}

impl ParticleGpuRuntimeOwner {
    pub fn activate_device_epoch(&mut self, device_epoch: RuntimePrepareDeviceEpoch) -> bool {
        let changed = self.active_device_epoch != Some(device_epoch);
        if changed {
            self.release_device_epoch_resources();
        }
        self.active_device_epoch = Some(device_epoch);
        changed
    }

    pub fn prepare_neutral_frame(
        &mut self,
        device: &wgpu::Device,
        frame: &RenderParticleGpuFrameExtract,
    ) -> Option<ParticleGpuRuntimeBufferBindings<'_>> {
        debug_assert!(self.pending_frame.is_none());
        self.aggregate_executed = false;
        self.neutral_buffers.prepare(device, frame)
    }

    pub fn deactivate(&mut self) {
        self.pending_frame = None;
        self.retain_instances(&[]);
    }

    pub fn execute_instances(
        &mut self,
        device: &wgpu::Device,
        buffer_uploads: &mut dyn RenderPassBufferUploadSink,
        encoder: &mut wgpu::CommandEncoder,
        instances: &[ParticleGpuRuntimeInstance],
    ) -> Result<Option<ParticleGpuRuntimeFrame>, ParticleGpuRuntimeOwnerError> {
        if self.pending_frame.is_some() {
            return Err(ParticleGpuRuntimeOwnerError::PendingFrameTransaction);
        }
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

        let (aggregate_frame, state_commits) =
            self.prepare_aggregate_frame_for(&playing_instances)?;
        let backend_commit = self
            .aggregate_backend
            .as_mut()
            .ok_or(ParticleGpuRuntimeOwnerError::MissingExecutedBackend)?
            .execute_frame(
                buffer_uploads,
                encoder,
                &aggregate_frame,
                ParticleGpuReadbackRequest::None,
            )?;
        self.next_frame_transaction_id = self.next_frame_transaction_id.wrapping_add(1).max(1);
        let transaction_id = self.next_frame_transaction_id;
        self.pending_frame = Some(ParticleGpuRuntimePendingFrame {
            transaction_id,
            backend_commit,
            state_commits,
        });

        Ok(Some(ParticleGpuRuntimeFrame {
            outputs: readback_outputs_from_frame(aggregate_frame.expected_frame_extract()),
            transaction_id,
        }))
    }

    pub fn active_bindings(
        &self,
    ) -> Result<ParticleGpuRuntimeBufferBindings<'_>, ParticleGpuRuntimeOwnerError> {
        let backend = self
            .aggregate_backend
            .as_ref()
            .filter(|_| self.pending_frame.is_some() || self.aggregate_executed)
            .ok_or(ParticleGpuRuntimeOwnerError::MissingExecutedBackend)?;
        let buffers = self.pending_frame.as_ref().map_or_else(
            || backend.active_buffers(),
            |pending| backend.prepared_buffers(pending.backend_commit),
        );
        Ok(bindings_from_buffers(buffers))
    }

    pub fn record_transparent_render(
        &mut self,
        device: &wgpu::Device,
        scene_layout: &wgpu::BindGroupLayout,
        config: super::ParticleGpuTransparentRenderConfig,
        buffer_uploads: &mut dyn RenderPassBufferUploadSink,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        params: super::ParticleGpuTransparentRenderParams,
        render_region: zircon_runtime::graphics::ViewportRenderRegion,
    ) -> Result<bool, ParticleGpuRuntimeOwnerError> {
        let prepared_output_buffer_index = self
            .pending_frame
            .as_ref()
            .map(|pending| pending.backend_commit.output_buffer_index());
        if prepared_output_buffer_index.is_none() && !self.aggregate_executed {
            return Ok(false);
        }
        let Some(backend) = self.aggregate_backend.as_mut() else {
            return Ok(false);
        };
        if !backend.transparent_render_enabled() {
            backend.enable_transparent_rendering(device, scene_layout, config);
        }
        let output_buffer_index =
            prepared_output_buffer_index.unwrap_or_else(|| backend.active_buffer_index());
        backend.record_transparent_render(
            buffer_uploads,
            encoder,
            color_view,
            depth_view,
            scene_bind_group,
            params,
            render_region,
            output_buffer_index,
        )?;
        Ok(true)
    }

    pub(crate) fn commit_frame_transaction(&mut self, transaction_id: u64) {
        let Some(pending) = self.pending_frame.take() else {
            debug_assert!(false, "particle GPU frame transaction is missing at commit");
            return;
        };
        if pending.transaction_id != transaction_id {
            debug_assert_eq!(pending.transaction_id, transaction_id);
            self.pending_frame = Some(pending);
            return;
        }

        for state_commit in pending.state_commits {
            let Some(state) = self.states.get_mut(&state_commit.handle) else {
                debug_assert!(
                    false,
                    "particle GPU runtime state disappeared before commit"
                );
                continue;
            };
            state.planner.commit_prepared_state(state_commit.planner);
            state.last_age_seconds = state_commit.last_age_seconds;
        }
        let backend_committed = self
            .aggregate_backend
            .as_mut()
            .is_some_and(|backend| backend.commit_prepared_frame(pending.backend_commit));
        debug_assert!(backend_committed);
        self.aggregate_executed = backend_committed;
    }

    pub(crate) fn rollback_frame_transaction(&mut self, transaction_id: u64) {
        let should_rollback = self
            .pending_frame
            .as_ref()
            .is_some_and(|pending| pending.transaction_id == transaction_id);
        debug_assert!(should_rollback || self.pending_frame.is_none());
        if should_rollback {
            self.pending_frame = None;
        }
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

    fn release_device_epoch_resources(&mut self) {
        self.pending_frame = None;
        self.states.clear();
        self.aggregate_asset = None;
        self.aggregate_backend = None;
        self.aggregate_executed = false;
        self.neutral_buffers = ParticleGpuNeutralBuffers::default();
    }

    fn prepare_aggregate_frame_for(
        &self,
        instances: &[&ParticleGpuRuntimeInstance],
    ) -> Result<
        (ParticleGpuFrameParams, Vec<ParticleGpuRuntimeStateCommit>),
        ParticleGpuRuntimeOwnerError,
    > {
        let layout = &self
            .aggregate_backend
            .as_ref()
            .ok_or(ParticleGpuRuntimeOwnerError::MissingExecutedBackend)?
            .program()
            .layout;
        let mut emitters = Vec::with_capacity(layout.emitter_count as usize);
        let mut state_commits = Vec::with_capacity(instances.len());
        let mut aggregate_emitter_index = 0usize;
        let mut max_dt: Real = 0.0;
        let mut max_age_seconds: Real = 0.0;

        for instance in instances {
            let state = self
                .states
                .get(&instance.handle)
                .ok_or(ParticleGpuRuntimeOwnerError::MissingExecutedBackend)?;
            let (frame, planner_commit, last_age_seconds) = state.prepare_frame(instance)?;
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
            state_commits.push(ParticleGpuRuntimeStateCommit {
                handle: instance.handle,
                planner: planner_commit,
                last_age_seconds,
            });
        }

        Ok((
            ParticleGpuFrameParams {
                dt: max_dt,
                age_seconds: max_age_seconds,
                emitters,
            },
            state_commits,
        ))
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

    fn prepare_frame(
        &self,
        instance: &ParticleGpuRuntimeInstance,
    ) -> Result<
        (ParticleGpuFrameParams, ParticleGpuPlannerCommit, Real),
        crate::ParticleSimulationError,
    > {
        let age_seconds = instance.age_seconds.max(0.0);
        let prepared = if age_seconds < self.last_age_seconds {
            ParticleGpuFramePlanner::new(self.asset.clone())
                .prepare_frame(age_seconds, instance.component.transform)?
        } else {
            self.planner.prepare_frame(
                age_seconds - self.last_age_seconds,
                instance.component.transform,
            )?
        };
        let (frame, planner_commit) = prepared.into_parts();
        Ok((frame, planner_commit, age_seconds))
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

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::rhi::{DeviceGeneration, DeviceId};

    fn epoch(generation: u64) -> RuntimePrepareDeviceEpoch {
        RuntimePrepareDeviceEpoch::new(DeviceId::new(17), DeviceGeneration::new(generation))
    }

    #[test]
    fn device_epoch_change_releases_persistent_particle_gpu_state() {
        let mut owner = ParticleGpuRuntimeOwner::default();
        assert!(owner.activate_device_epoch(epoch(3)));
        owner.aggregate_asset = Some(ParticleSystemAsset::new("device-epoch-test"));
        owner.aggregate_executed = true;
        owner.next_frame_transaction_id = 41;

        assert!(!owner.activate_device_epoch(epoch(3)));
        assert!(owner.aggregate_asset.is_some());
        assert!(owner.aggregate_executed);

        assert!(owner.activate_device_epoch(epoch(4)));
        assert_eq!(owner.active_device_epoch, Some(epoch(4)));
        assert!(owner.states.is_empty());
        assert!(owner.aggregate_asset.is_none());
        assert!(owner.aggregate_backend.is_none());
        assert!(!owner.aggregate_executed);
        assert!(owner.pending_frame.is_none());
        assert_eq!(owner.next_frame_transaction_id, 41);
    }
}

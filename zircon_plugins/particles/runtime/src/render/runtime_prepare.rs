use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::render::{
    RenderParticleGpuFrameExtract, RenderParticleGpuReadbackOutputs, RenderPluginRendererOutputs,
};
use zircon_runtime::graphics::{
    GraphicsError, RuntimeGpuReadback, RuntimePrepareCollector, RuntimePrepareCollectorContext,
    RuntimePrepareCollectorRegistration, RuntimePrepareFrameTransaction,
};

use crate::ParticlesManager;

use super::gpu::{
    ParticleGpuCounterReadback, ParticleGpuRuntimeBufferBindings, ParticleGpuRuntimeOwnerHandle,
    PARTICLE_GPU_COUNTER_WORDS_BASE, PARTICLE_GPU_INDIRECT_DRAW_WORDS, PARTICLE_GPU_MAX_PARTICLES,
    PARTICLE_GPU_NEUTRAL_MAX_EMITTERS,
};

const COLLECTOR_ID: &str = "particles.runtime-prepare";
const PARTICLES_A_RESOURCE: &str = "particles.gpu.particles-a";
const PARTICLES_B_RESOURCE: &str = "particles.gpu.particles-b";
const EMITTER_PARAMS_RESOURCE: &str = "particles.gpu.emitter-params";
const COUNTERS_RESOURCE: &str = "particles.gpu.counters";
const ALIVE_INDICES_RESOURCE: &str = "particles.gpu.alive-indices";
const INDIRECT_DRAW_ARGS_RESOURCE: &str = "particles.gpu.indirect-draw-args";
const DEBUG_READBACK_RESOURCE: &str = "particles.gpu.debug-readback";

pub fn particle_runtime_prepare_collector_registration() -> RuntimePrepareCollectorRegistration {
    RuntimePrepareCollectorRegistration::new_collector(
        COLLECTOR_ID,
        Arc::new(NeutralParticleRuntimePrepareCollector {
            runtime_owner: ParticleGpuRuntimeOwnerHandle::default(),
        }),
    )
}

pub fn particle_runtime_prepare_collector_registration_with_manager(
    manager: ParticlesManager,
) -> RuntimePrepareCollectorRegistration {
    particle_runtime_prepare_collector_registration_with_manager_and_owner(
        manager,
        ParticleGpuRuntimeOwnerHandle::default(),
    )
}

pub fn particle_runtime_prepare_collector_registration_with_manager_and_owner(
    manager: ParticlesManager,
    runtime_owner: ParticleGpuRuntimeOwnerHandle,
) -> RuntimePrepareCollectorRegistration {
    RuntimePrepareCollectorRegistration::new_collector(
        COLLECTOR_ID,
        Arc::new(ParticleRuntimePrepareCollector::new(manager, runtime_owner)),
    )
}

struct ParticleRuntimePrepareCollector {
    manager: ParticlesManager,
    runtime_owner: ParticleGpuRuntimeOwnerHandle,
    pending_readbacks: Arc<Mutex<VecDeque<ParticleGpuSharedReadback>>>,
}

struct NeutralParticleRuntimePrepareCollector {
    runtime_owner: ParticleGpuRuntimeOwnerHandle,
}

impl RuntimePrepareCollector for NeutralParticleRuntimePrepareCollector {
    fn collect(
        &self,
        context: &mut RuntimePrepareCollectorContext<'_>,
    ) -> Result<RenderPluginRendererOutputs, GraphicsError> {
        collect_neutral_particle_gpu_runtime_prepare(context, &self.runtime_owner)
    }
}

impl ParticleRuntimePrepareCollector {
    fn new(manager: ParticlesManager, runtime_owner: ParticleGpuRuntimeOwnerHandle) -> Self {
        Self {
            manager,
            runtime_owner,
            pending_readbacks: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

impl RuntimePrepareCollector for ParticleRuntimePrepareCollector {
    fn requests_gpu_readback(&self) -> bool {
        true
    }

    fn collect(
        &self,
        context: &mut RuntimePrepareCollectorContext<'_>,
    ) -> Result<RenderPluginRendererOutputs, GraphicsError> {
        collect_real_particle_gpu_runtime_prepare(
            context,
            &self.manager,
            &self.runtime_owner,
            &self.pending_readbacks,
        )
    }
}

fn collect_neutral_particle_gpu_runtime_prepare(
    context: &mut RuntimePrepareCollectorContext<'_>,
    runtime_owner: &ParticleGpuRuntimeOwnerHandle,
) -> Result<RenderPluginRendererOutputs, GraphicsError> {
    let mut owner = runtime_owner
        .lock()
        .map_err(|error| GraphicsError::Asset(error.to_string()))?;
    owner.activate_device_epoch(context.device_epoch());
    owner.deactivate();
    let Some(frame) = context.frame_extract().particles.gpu_frame.as_ref() else {
        return Ok(RenderPluginRendererOutputs::default());
    };

    let bindings = {
        let gpu = context.gpu_recording_context();
        owner.prepare_neutral_frame(gpu.device, frame)
    };
    let particles = neutral_readback_outputs_from_frame(frame);
    if let Some(bindings) = bindings {
        register_neutral_particle_external_buffers(context, bindings);
    }

    Ok(RenderPluginRendererOutputs {
        particles,
        ..RenderPluginRendererOutputs::default()
    })
}

fn collect_real_particle_gpu_runtime_prepare(
    context: &mut RuntimePrepareCollectorContext<'_>,
    manager: &ParticlesManager,
    runtime_owner: &ParticleGpuRuntimeOwnerHandle,
    pending_readbacks: &Arc<Mutex<VecDeque<ParticleGpuSharedReadback>>>,
) -> Result<RenderPluginRendererOutputs, GraphicsError> {
    let device_epoch_changed = runtime_owner
        .lock()
        .map_err(|error| GraphicsError::Asset(error.to_string()))?
        .activate_device_epoch(context.device_epoch());
    if device_epoch_changed {
        pending_readbacks
            .lock()
            .map_err(|_| GraphicsError::BufferMap("particle readback queue lock poisoned".into()))?
            .clear();
    }
    let instances = manager.gpu_runtime_instances();
    if instances.is_empty() {
        pending_readbacks
            .lock()
            .map_err(|_| GraphicsError::BufferMap("particle readback queue lock poisoned".into()))?
            .clear();
        return collect_neutral_particle_gpu_runtime_prepare(context, runtime_owner);
    }

    let completed_outputs = take_completed_particle_readback(pending_readbacks)?;
    let readback_capacity_available = pending_readbacks
        .lock()
        .map_err(|_| GraphicsError::BufferMap("particle readback queue lock poisoned".into()))?
        .len()
        < RuntimePrepareCollectorContext::MAX_IN_FLIGHT_GPU_READBACK_FRAMES;
    if !context.gpu_work_admitted() || !readback_capacity_available {
        let owner = runtime_owner
            .lock()
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        if let Ok(bindings) = owner.active_bindings() {
            register_real_particle_external_buffers(context, bindings);
        }
        return Ok(RenderPluginRendererOutputs {
            particles: completed_outputs.unwrap_or_default(),
            ..RenderPluginRendererOutputs::default()
        });
    }
    let mut owner = runtime_owner
        .lock()
        .map_err(|error| GraphicsError::Asset(error.to_string()))?;
    let frame = {
        let gpu = context.gpu_recording_context();
        let zircon_runtime::graphics::RuntimePrepareGpuRecordingContext {
            device,
            device_epoch: _,
            encoder,
            mut buffer_uploads,
            frame_transactions: _,
        } = gpu;
        owner
            .execute_instances(device, &mut buffer_uploads, encoder, &instances)
            .map_err(|error| GraphicsError::Asset(error.to_string()))?
    };
    let Some(frame) = frame else {
        return Ok(RenderPluginRendererOutputs::default());
    };
    let transaction_id = frame.transaction_id();
    context.register_frame_transaction(RuntimePrepareFrameTransaction::new(
        "particles.gpu.runtime-frame",
        {
            let runtime_owner = runtime_owner.clone();
            move || runtime_owner.commit_frame_transaction(transaction_id)
        },
        {
            let runtime_owner = runtime_owner.clone();
            move || runtime_owner.rollback_frame_transaction(transaction_id)
        },
    ));

    let fallback_outputs = frame.outputs;
    let bindings = owner
        .active_bindings()
        .map_err(|error| GraphicsError::Asset(error.to_string()))?;
    let pending_readback = enqueue_particle_readback(
        context,
        &bindings,
        fallback_outputs.per_emitter_spawned.len() as u32,
    )?;
    context.register_frame_transaction(RuntimePrepareFrameTransaction::new(
        "particles.gpu.readback",
        {
            let pending_readbacks = Arc::clone(pending_readbacks);
            move || {
                pending_readbacks
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .push_back(pending_readback);
            }
        },
        || {},
    ));
    register_real_particle_external_buffers(context, bindings);

    Ok(RenderPluginRendererOutputs {
        particles: completed_outputs.unwrap_or(fallback_outputs),
        ..RenderPluginRendererOutputs::default()
    })
}

struct ParticleGpuSharedReadback {
    emitter_count: u32,
    counters: RuntimeGpuReadback,
    indirect_draw_args: RuntimeGpuReadback,
}

impl ParticleGpuSharedReadback {
    fn is_ready(&self) -> bool {
        self.counters.is_ready() && self.indirect_draw_args.is_ready()
    }

    fn collect_ready(self) -> Result<RenderParticleGpuReadbackOutputs, GraphicsError> {
        let counter_bytes = self
            .counters
            .try_take()
            .expect("ready particle counter readback remains available")?;
        let indirect_bytes = self
            .indirect_draw_args
            .try_take()
            .expect("ready particle indirect readback remains available")?;
        let counters = bytes_as_u32s(&counter_bytes)?;
        let indirect = bytes_as_u32s(&indirect_bytes)?;
        let indirect_draw_args: [u32; 4] = indirect.try_into().map_err(|words: Vec<u32>| {
            GraphicsError::BufferMap(format!(
                "particle indirect readback returned {} words instead of four",
                words.len()
            ))
        })?;
        let counters = ParticleGpuCounterReadback::from_words(&counters, self.emitter_count)
            .map_err(|error| GraphicsError::BufferMap(error.to_string()))?;
        Ok(counters.to_render_outputs(indirect_draw_args))
    }
}

fn enqueue_particle_readback(
    context: &mut RuntimePrepareCollectorContext<'_>,
    bindings: &ParticleGpuRuntimeBufferBindings<'_>,
    emitter_count: u32,
) -> Result<ParticleGpuSharedReadback, GraphicsError> {
    let counter_words = PARTICLE_GPU_COUNTER_WORDS_BASE + emitter_count;
    let counters = context.request_gpu_readback(
        "particles.counters",
        bindings.counters,
        0..u64::from(counter_words) * std::mem::size_of::<u32>() as u64,
    )?;
    let indirect_draw_args = context.request_gpu_readback(
        "particles.indirect-draw-args",
        bindings.indirect_draw_args,
        0..u64::from(PARTICLE_GPU_INDIRECT_DRAW_WORDS) * std::mem::size_of::<u32>() as u64,
    )?;
    Ok(ParticleGpuSharedReadback {
        emitter_count,
        counters,
        indirect_draw_args,
    })
}

fn take_completed_particle_readback(
    pending_readbacks: &Mutex<VecDeque<ParticleGpuSharedReadback>>,
) -> Result<Option<RenderParticleGpuReadbackOutputs>, GraphicsError> {
    let mut pending_readbacks = pending_readbacks
        .lock()
        .map_err(|_| GraphicsError::BufferMap("particle readback queue lock poisoned".into()))?;
    if !pending_readbacks
        .front()
        .is_some_and(ParticleGpuSharedReadback::is_ready)
    {
        return Ok(None);
    }
    pending_readbacks
        .pop_front()
        .map(ParticleGpuSharedReadback::collect_ready)
        .transpose()
}

fn bytes_as_u32s(bytes: &[u8]) -> Result<Vec<u32>, GraphicsError> {
    if bytes.len() % std::mem::size_of::<u32>() != 0 {
        return Err(GraphicsError::BufferMap(format!(
            "particle readback returned an unaligned {} byte payload",
            bytes.len()
        )));
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
        .collect())
}

fn register_real_particle_external_buffers(
    context: &mut RuntimePrepareCollectorContext<'_>,
    bindings: ParticleGpuRuntimeBufferBindings<'_>,
) {
    register_real_external_buffer(context, PARTICLES_A_RESOURCE, bindings.particles_a);
    register_real_external_buffer(context, PARTICLES_B_RESOURCE, bindings.particles_b);
    register_real_external_buffer(context, EMITTER_PARAMS_RESOURCE, bindings.emitter_params);
    register_real_external_buffer(context, COUNTERS_RESOURCE, bindings.counters);
    register_real_external_buffer(context, ALIVE_INDICES_RESOURCE, bindings.alive_indices);
    register_real_external_buffer(
        context,
        INDIRECT_DRAW_ARGS_RESOURCE,
        bindings.indirect_draw_args,
    );
    register_real_external_buffer(context, DEBUG_READBACK_RESOURCE, bindings.debug_readback);
}

fn register_neutral_particle_external_buffers(
    context: &mut RuntimePrepareCollectorContext<'_>,
    bindings: ParticleGpuRuntimeBufferBindings<'_>,
) {
    register_external_buffer(context, PARTICLES_A_RESOURCE, bindings.particles_a);
    register_external_buffer(context, PARTICLES_B_RESOURCE, bindings.particles_b);
    register_external_buffer(context, EMITTER_PARAMS_RESOURCE, bindings.emitter_params);
    register_external_buffer(context, COUNTERS_RESOURCE, bindings.counters);
    register_external_buffer(context, ALIVE_INDICES_RESOURCE, bindings.alive_indices);
    register_external_buffer(
        context,
        INDIRECT_DRAW_ARGS_RESOURCE,
        bindings.indirect_draw_args,
    );
    register_external_buffer(context, DEBUG_READBACK_RESOURCE, bindings.debug_readback);
}

fn register_external_buffer(
    context: &mut RuntimePrepareCollectorContext<'_>,
    logical_name: &'static str,
    buffer: &wgpu::Buffer,
) {
    register_external_buffer_with_label(context, logical_name, "neutral-frame", buffer);
}

fn register_real_external_buffer(
    context: &mut RuntimePrepareCollectorContext<'_>,
    logical_name: &'static str,
    buffer: &wgpu::Buffer,
) {
    register_external_buffer_with_label(context, logical_name, "backend", buffer);
}

fn register_external_buffer_with_label(
    context: &mut RuntimePrepareCollectorContext<'_>,
    logical_name: &'static str,
    source_label: &'static str,
    buffer: &wgpu::Buffer,
) {
    let backing_name = particle_external_buffer_backing_name(logical_name, source_label);
    context.register_static_external_buffer_binding_with_backing(
        logical_name,
        backing_name,
        buffer,
    );
}

fn particle_external_buffer_backing_name(
    logical_name: &'static str,
    source_label: &'static str,
) -> &'static str {
    match (logical_name, source_label) {
        (PARTICLES_A_RESOURCE, "neutral-frame") => "particles.gpu.particles-a:neutral-frame",
        (PARTICLES_B_RESOURCE, "neutral-frame") => "particles.gpu.particles-b:neutral-frame",
        (EMITTER_PARAMS_RESOURCE, "neutral-frame") => "particles.gpu.emitter-params:neutral-frame",
        (COUNTERS_RESOURCE, "neutral-frame") => "particles.gpu.counters:neutral-frame",
        (ALIVE_INDICES_RESOURCE, "neutral-frame") => "particles.gpu.alive-indices:neutral-frame",
        (INDIRECT_DRAW_ARGS_RESOURCE, "neutral-frame") => {
            "particles.gpu.indirect-draw-args:neutral-frame"
        }
        (DEBUG_READBACK_RESOURCE, "neutral-frame") => "particles.gpu.debug-readback:neutral-frame",
        (PARTICLES_A_RESOURCE, "backend") => "particles.gpu.particles-a:backend",
        (PARTICLES_B_RESOURCE, "backend") => "particles.gpu.particles-b:backend",
        (EMITTER_PARAMS_RESOURCE, "backend") => "particles.gpu.emitter-params:backend",
        (COUNTERS_RESOURCE, "backend") => "particles.gpu.counters:backend",
        (ALIVE_INDICES_RESOURCE, "backend") => "particles.gpu.alive-indices:backend",
        (INDIRECT_DRAW_ARGS_RESOURCE, "backend") => "particles.gpu.indirect-draw-args:backend",
        (DEBUG_READBACK_RESOURCE, "backend") => "particles.gpu.debug-readback:backend",
        _ => "particles.gpu.unknown:runtime-prepare",
    }
}

fn readback_outputs_from_frame(
    frame: &RenderParticleGpuFrameExtract,
) -> RenderParticleGpuReadbackOutputs {
    RenderParticleGpuReadbackOutputs {
        alive_count: frame.alive_count,
        spawned_total: frame.spawned_total,
        debug_flags: 0,
        per_emitter_spawned: frame.per_emitter_spawned.clone(),
        indirect_draw_args: frame.indirect_draw_args,
    }
}

fn neutral_readback_outputs_from_frame(
    frame: &RenderParticleGpuFrameExtract,
) -> RenderParticleGpuReadbackOutputs {
    let mut indirect_draw_args = frame.indirect_draw_args;
    indirect_draw_args[1] = indirect_draw_args[1].min(PARTICLE_GPU_MAX_PARTICLES);
    RenderParticleGpuReadbackOutputs {
        alive_count: frame.alive_count.min(PARTICLE_GPU_MAX_PARTICLES),
        spawned_total: frame.spawned_total.min(PARTICLE_GPU_MAX_PARTICLES),
        debug_flags: 0,
        per_emitter_spawned: frame
            .per_emitter_spawned
            .iter()
            .take(PARTICLE_GPU_NEUTRAL_MAX_EMITTERS as usize)
            .copied()
            .collect(),
        indirect_draw_args,
    }
}

#[cfg(test)]
#[path = "tests/runtime_prepare.rs"]
mod tests;

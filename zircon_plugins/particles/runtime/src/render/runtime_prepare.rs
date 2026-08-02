use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::render::{
    RenderParticleGpuFrameExtract, RenderParticleGpuReadbackOutputs, RenderPluginRendererOutputs,
};
use zircon_runtime::graphics::{
    GraphicsError, RuntimeGpuReadback, RuntimePrepareCollector, RuntimePrepareCollectorContext,
    RuntimePrepareCollectorRegistration,
};

use crate::ParticlesManager;

use super::gpu::{
    ParticleGpuCounterReadback, ParticleGpuRuntimeBufferBindings, ParticleGpuRuntimeOwnerHandle,
    PARTICLE_GPU_COUNTER_WORDS_BASE, PARTICLE_GPU_INDIRECT_DRAW_WORDS,
};

const COLLECTOR_ID: &str = "particles.runtime-prepare";
const PARTICLES_A_RESOURCE: &str = "particles.gpu.particles-a";
const PARTICLES_B_RESOURCE: &str = "particles.gpu.particles-b";
const EMITTER_PARAMS_RESOURCE: &str = "particles.gpu.emitter-params";
const COUNTERS_RESOURCE: &str = "particles.gpu.counters";
const ALIVE_INDICES_RESOURCE: &str = "particles.gpu.alive-indices";
const INDIRECT_DRAW_ARGS_RESOURCE: &str = "particles.gpu.indirect-draw-args";
const DEBUG_READBACK_RESOURCE: &str = "particles.gpu.debug-readback";
const PARTICLE_WORDS_PER_NEUTRAL_SLOT: u32 = 16;
const EMITTER_PARAMS_BYTES_PER_EMITTER: u64 = 256;

pub fn particle_runtime_prepare_collector_registration() -> RuntimePrepareCollectorRegistration {
    RuntimePrepareCollectorRegistration::new(COLLECTOR_ID, particle_runtime_prepare_collector)
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
    pending_readbacks: Mutex<VecDeque<ParticleGpuSharedReadback>>,
}

impl ParticleRuntimePrepareCollector {
    fn new(manager: ParticlesManager, runtime_owner: ParticleGpuRuntimeOwnerHandle) -> Self {
        Self {
            manager,
            runtime_owner,
            pending_readbacks: Mutex::new(VecDeque::new()),
        }
    }
}

impl RuntimePrepareCollector for ParticleRuntimePrepareCollector {
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

fn particle_runtime_prepare_collector(
    context: &mut RuntimePrepareCollectorContext<'_>,
) -> Result<RenderPluginRendererOutputs, GraphicsError> {
    let Some(frame) = context
        .frame_extract()
        .particles
        .gpu_frame
        .as_ref()
        .cloned()
    else {
        return Ok(RenderPluginRendererOutputs::default());
    };

    register_neutral_particle_external_buffers(context, &frame);

    Ok(RenderPluginRendererOutputs {
        particles: readback_outputs_from_frame(&frame),
        ..RenderPluginRendererOutputs::default()
    })
}

fn collect_real_particle_gpu_runtime_prepare(
    context: &mut RuntimePrepareCollectorContext<'_>,
    manager: &ParticlesManager,
    runtime_owner: &ParticleGpuRuntimeOwnerHandle,
    pending_readbacks: &Mutex<VecDeque<ParticleGpuSharedReadback>>,
) -> Result<RenderPluginRendererOutputs, GraphicsError> {
    let instances = manager.gpu_runtime_instances();
    if instances.is_empty() {
        pending_readbacks
            .lock()
            .map_err(|_| GraphicsError::BufferMap("particle readback queue lock poisoned".into()))?
            .clear();
        return particle_runtime_prepare_collector(context);
    }

    let completed_outputs = take_completed_particle_readback(pending_readbacks)?;
    let mut owner = runtime_owner
        .lock()
        .map_err(|error| GraphicsError::Asset(error.to_string()))?;
    let Some(frame) = owner
        .execute_instances(context.device, context.queue, context.encoder, &instances)
        .map_err(|error| GraphicsError::Asset(error.to_string()))?
    else {
        return Ok(RenderPluginRendererOutputs::default());
    };

    let fallback_outputs = frame.outputs;
    let bindings = owner
        .active_bindings()
        .map_err(|error| GraphicsError::Asset(error.to_string()))?;
    enqueue_particle_readback(
        context,
        &bindings,
        fallback_outputs.per_emitter_spawned.len() as u32,
        pending_readbacks,
    )?;
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
    pending_readbacks: &Mutex<VecDeque<ParticleGpuSharedReadback>>,
) -> Result<(), GraphicsError> {
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
    pending_readbacks
        .lock()
        .map_err(|_| GraphicsError::BufferMap("particle readback queue lock poisoned".into()))?
        .push_back(ParticleGpuSharedReadback {
            emitter_count,
            counters,
            indirect_draw_args,
        });
    Ok(())
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
    frame: &RenderParticleGpuFrameExtract,
) {
    let particle_slot_count = neutral_particle_slot_count(frame);
    let emitter_count = neutral_emitter_count(frame);
    let particle_buffer_bytes = neutral_particle_buffer_bytes(particle_slot_count);
    let counter_words = counter_word_count(emitter_count);
    let counter_bytes = word_bytes(counter_words);
    let alive_indices_bytes = word_bytes(particle_slot_count.max(1) as u64);
    let indirect_draw_bytes = word_bytes(PARTICLE_GPU_INDIRECT_DRAW_WORDS);
    let debug_readback_bytes = counter_bytes + indirect_draw_bytes;

    let particles_a = create_external_buffer(
        context.device,
        "zircon-particle-runtime-prepare-particles-a",
        particle_buffer_bytes,
        particle_storage_usage(),
    );
    let particles_b = create_external_buffer(
        context.device,
        "zircon-particle-runtime-prepare-particles-b",
        particle_buffer_bytes,
        particle_storage_usage(),
    );
    let emitter_params = create_external_buffer(
        context.device,
        "zircon-particle-runtime-prepare-emitter-params",
        emitter_count.max(1) as u64 * EMITTER_PARAMS_BYTES_PER_EMITTER,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    );
    let counters = create_external_buffer(
        context.device,
        "zircon-particle-runtime-prepare-counters",
        counter_bytes,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
    );
    let alive_indices = create_external_buffer(
        context.device,
        "zircon-particle-runtime-prepare-alive-indices",
        alive_indices_bytes,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
    );
    let indirect_draw_args = create_external_buffer(
        context.device,
        "zircon-particle-runtime-prepare-indirect-draw-args",
        indirect_draw_bytes,
        wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::INDIRECT,
    );
    let debug_readback = create_external_buffer(
        context.device,
        "zircon-particle-runtime-prepare-debug-readback",
        debug_readback_bytes,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
    );

    context
        .queue
        .write_buffer(&counters, 0, &counter_words_for_frame(frame));
    context.queue.write_buffer(
        &alive_indices,
        0,
        &alive_index_words(frame.alive_count.min(particle_slot_count)),
    );
    context.queue.write_buffer(
        &indirect_draw_args,
        0,
        &u32_words_to_bytes(&frame.indirect_draw_args),
    );
    context
        .encoder
        .copy_buffer_to_buffer(&counters, 0, &debug_readback, 0, counter_bytes);
    context.encoder.copy_buffer_to_buffer(
        &indirect_draw_args,
        0,
        &debug_readback,
        counter_bytes,
        indirect_draw_bytes,
    );

    register_external_buffer(context, PARTICLES_A_RESOURCE, &particles_a);
    register_external_buffer(context, PARTICLES_B_RESOURCE, &particles_b);
    register_external_buffer(context, EMITTER_PARAMS_RESOURCE, &emitter_params);
    register_external_buffer(context, COUNTERS_RESOURCE, &counters);
    register_external_buffer(context, ALIVE_INDICES_RESOURCE, &alive_indices);
    register_external_buffer(context, INDIRECT_DRAW_ARGS_RESOURCE, &indirect_draw_args);
    register_external_buffer(context, DEBUG_READBACK_RESOURCE, &debug_readback);
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
    context.register_external_buffer_binding_with_backing(
        logical_name,
        format!("{logical_name}:runtime-prepare-{source_label}"),
        buffer,
    );
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

fn create_external_buffer(
    device: &wgpu::Device,
    label: &'static str,
    size: u64,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size.max(std::mem::size_of::<u32>() as u64),
        usage,
        mapped_at_creation: false,
    })
}

fn particle_storage_usage() -> wgpu::BufferUsages {
    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC
}

fn neutral_particle_slot_count(frame: &RenderParticleGpuFrameExtract) -> u32 {
    frame
        .alive_count
        .max(frame.spawned_total)
        .max(frame.indirect_draw_args[1])
        .max(1)
}

fn neutral_emitter_count(frame: &RenderParticleGpuFrameExtract) -> u32 {
    frame.per_emitter_spawned.len().max(1) as u32
}

fn neutral_particle_buffer_bytes(slot_count: u32) -> u64 {
    word_bytes(slot_count.max(1) as u64 * PARTICLE_WORDS_PER_NEUTRAL_SLOT as u64)
}

fn counter_word_count(emitter_count: u32) -> u64 {
    PARTICLE_GPU_COUNTER_WORDS_BASE as u64 + emitter_count.max(1) as u64
}

fn counter_words_for_frame(frame: &RenderParticleGpuFrameExtract) -> Vec<u8> {
    let mut words = vec![frame.alive_count, frame.spawned_total, 0, 0];
    words.extend(frame.per_emitter_spawned.iter().copied());
    let expected_words = counter_word_count(neutral_emitter_count(frame)) as usize;
    if words.len() < expected_words {
        words.resize(expected_words, 0);
    }
    u32_words_to_bytes(&words)
}

fn alive_index_words(alive_count: u32) -> Vec<u8> {
    let words = (0..alive_count.max(1)).collect::<Vec<_>>();
    u32_words_to_bytes(&words)
}

fn word_bytes(word_count: u64) -> u64 {
    word_count.max(1) * std::mem::size_of::<u32>() as u64
}

fn u32_words_to_bytes(words: &[u32]) -> Vec<u8> {
    words
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_runtime_prepare_neutral_frame_sizes_cover_readback_payload() {
        let frame = RenderParticleGpuFrameExtract {
            alive_count: 3,
            spawned_total: 5,
            per_emitter_spawned: vec![2, 3],
            indirect_draw_args: [6, 3, 0, 0],
        };

        assert_eq!(neutral_particle_slot_count(&frame), 5);
        assert_eq!(neutral_emitter_count(&frame), 2);
        assert_eq!(
            counter_word_count(neutral_emitter_count(&frame)),
            PARTICLE_GPU_COUNTER_WORDS_BASE as u64 + 2
        );
        assert_eq!(
            counter_words_for_frame(&frame),
            u32_words_to_bytes(&[3, 5, 0, 0, 2, 3])
        );
        assert_eq!(alive_index_words(3), u32_words_to_bytes(&[0, 1, 2]));
        assert_eq!(
            readback_outputs_from_frame(&frame),
            RenderParticleGpuReadbackOutputs {
                alive_count: 3,
                spawned_total: 5,
                debug_flags: 0,
                per_emitter_spawned: vec![2, 3],
                indirect_draw_args: [6, 3, 0, 0],
            }
        );
    }

    #[test]
    fn particle_runtime_prepare_neutral_frame_uses_minimum_nonzero_buffers() {
        let frame = RenderParticleGpuFrameExtract::default();

        assert_eq!(neutral_particle_slot_count(&frame), 1);
        assert_eq!(neutral_emitter_count(&frame), 1);
        assert_eq!(
            counter_words_for_frame(&frame),
            u32_words_to_bytes(&[0, 0, 0, 0, 0])
        );
        assert_eq!(alive_index_words(0), u32_words_to_bytes(&[0]));
    }

    #[test]
    fn particle_runtime_prepare_registration_id_is_stable() {
        let registration = particle_runtime_prepare_collector_registration();

        assert_eq!(registration.collector_id(), COLLECTOR_ID);
    }
}

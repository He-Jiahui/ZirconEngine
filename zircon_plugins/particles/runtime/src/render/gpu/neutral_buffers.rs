use zircon_runtime::core::framework::render::RenderParticleGpuFrameExtract;

use super::program::{PARTICLE_GPU_COUNTER_WORDS_BASE, PARTICLE_GPU_INDIRECT_DRAW_WORDS};
use super::runtime_owner::ParticleGpuRuntimeBufferBindings;
use super::PARTICLE_GPU_MAX_PARTICLES;

const PARTICLE_WORDS_PER_NEUTRAL_SLOT: u64 = 16;
const EMITTER_PARAMS_BYTES_PER_EMITTER: u64 = 256;
const WORD_BYTES: u64 = std::mem::size_of::<u32>() as u64;
// Neutral fallback only needs bounded typed resource identity; it never simulates emitters.
pub(crate) const PARTICLE_GPU_NEUTRAL_MAX_EMITTERS: u32 = 4_096;

const STORAGE_USAGE: wgpu::BufferUsages = wgpu::BufferUsages::STORAGE
    .union(wgpu::BufferUsages::COPY_DST)
    .union(wgpu::BufferUsages::COPY_SRC);
const INDIRECT_USAGE: wgpu::BufferUsages = STORAGE_USAGE.union(wgpu::BufferUsages::INDIRECT);

#[derive(Default)]
pub(super) struct ParticleGpuNeutralBuffers {
    device: Option<wgpu::Device>,
    buffers: Option<NeutralBufferBundle>,
    particle_capacity: u32,
    emitter_capacity: u32,
    last_frame: Option<NeutralFrameShadow>,
    counters_scratch: Vec<u8>,
    alive_indices_scratch: Vec<u8>,
}

impl ParticleGpuNeutralBuffers {
    pub(super) fn prepare<'a>(
        &'a mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        frame: &RenderParticleGpuFrameExtract,
    ) -> Option<ParticleGpuRuntimeBufferBindings<'a>> {
        if neutral_frame_is_empty(frame) {
            self.last_frame = None;
            return None;
        }

        let required_particle_capacity = neutral_particle_slot_count(frame).next_power_of_two();
        let required_emitter_capacity = neutral_emitter_count(frame).next_power_of_two();
        let same_device = self
            .device
            .as_ref()
            .is_some_and(|current| current == device);
        let must_rebuild = !same_device
            || self.buffers.is_none()
            || required_particle_capacity > self.particle_capacity
            || required_emitter_capacity > self.emitter_capacity;
        if must_rebuild {
            let particle_capacity = if same_device {
                self.particle_capacity.max(required_particle_capacity)
            } else {
                required_particle_capacity
            };
            let emitter_capacity = if same_device {
                self.emitter_capacity.max(required_emitter_capacity)
            } else {
                required_emitter_capacity
            };
            self.particle_capacity = particle_capacity;
            self.emitter_capacity = emitter_capacity;
            self.buffers = Some(NeutralBufferBundle::new(
                device,
                particle_capacity,
                emitter_capacity,
            ));
            self.device = Some(device.clone());
            self.last_frame = None;
        }

        let counters_changed = self
            .last_frame
            .as_ref()
            .is_none_or(|last| !last.counters_match(frame));
        let alive_indices_changed = self
            .last_frame
            .as_ref()
            .is_none_or(|last| last.alive_count != bounded_particle_count(frame.alive_count));
        let indirect_args_changed = self.last_frame.as_ref().is_none_or(|last| {
            last.indirect_draw_args != bounded_indirect_draw_args(frame)
                || last.emitter_count != neutral_emitter_count(frame)
        });
        let buffers = self
            .buffers
            .as_ref()
            .expect("neutral buffers were prepared");

        if counters_changed {
            write_counter_bytes(&mut self.counters_scratch, frame, self.emitter_capacity);
            queue.write_buffer(&buffers.counters, 0, &self.counters_scratch);
            encoder.copy_buffer_to_buffer(
                &buffers.counters,
                0,
                &buffers.debug_readback,
                0,
                self.counters_scratch.len() as u64,
            );
        }
        if alive_indices_changed {
            write_alive_index_bytes(
                &mut self.alive_indices_scratch,
                frame.alive_count.min(self.particle_capacity),
            );
            queue.write_buffer(&buffers.alive_indices, 0, &self.alive_indices_scratch);
        }
        if indirect_args_changed {
            let indirect_draw_args = words4_to_bytes(bounded_indirect_draw_args(frame));
            queue.write_buffer(&buffers.indirect_draw_args, 0, &indirect_draw_args);
            encoder.copy_buffer_to_buffer(
                &buffers.indirect_draw_args,
                0,
                &buffers.debug_readback,
                current_counter_byte_count(frame),
                indirect_draw_args.len() as u64,
            );
        }

        let shadow = self
            .last_frame
            .get_or_insert_with(NeutralFrameShadow::default);
        if counters_changed {
            shadow.capture_counters(frame);
        }
        if indirect_args_changed {
            shadow.indirect_draw_args = bounded_indirect_draw_args(frame);
        }
        Some(buffers.bindings())
    }
}

struct NeutralBufferBundle {
    particles_a: wgpu::Buffer,
    particles_b: wgpu::Buffer,
    emitter_params: wgpu::Buffer,
    counters: wgpu::Buffer,
    alive_indices: wgpu::Buffer,
    indirect_draw_args: wgpu::Buffer,
    debug_readback: wgpu::Buffer,
}

impl NeutralBufferBundle {
    fn new(device: &wgpu::Device, particle_capacity: u32, emitter_capacity: u32) -> Self {
        let particle_bytes =
            u64::from(particle_capacity) * PARTICLE_WORDS_PER_NEUTRAL_SLOT * WORD_BYTES;
        let emitter_bytes = u64::from(emitter_capacity) * EMITTER_PARAMS_BYTES_PER_EMITTER;
        let counter_byte_capacity =
            u64::from(PARTICLE_GPU_COUNTER_WORDS_BASE + emitter_capacity) * WORD_BYTES;
        let alive_index_bytes = u64::from(particle_capacity) * WORD_BYTES;
        let indirect_bytes = u64::from(PARTICLE_GPU_INDIRECT_DRAW_WORDS) * WORD_BYTES;

        Self {
            particles_a: create_buffer(
                device,
                "zircon-particle-neutral-particles-a",
                particle_bytes,
                STORAGE_USAGE,
            ),
            particles_b: create_buffer(
                device,
                "zircon-particle-neutral-particles-b",
                particle_bytes,
                STORAGE_USAGE,
            ),
            emitter_params: create_buffer(
                device,
                "zircon-particle-neutral-emitter-params",
                emitter_bytes,
                STORAGE_USAGE,
            ),
            counters: create_buffer(
                device,
                "zircon-particle-neutral-counters",
                counter_byte_capacity,
                STORAGE_USAGE,
            ),
            alive_indices: create_buffer(
                device,
                "zircon-particle-neutral-alive-indices",
                alive_index_bytes,
                STORAGE_USAGE,
            ),
            indirect_draw_args: create_buffer(
                device,
                "zircon-particle-neutral-indirect-draw-args",
                indirect_bytes,
                INDIRECT_USAGE,
            ),
            debug_readback: create_buffer(
                device,
                "zircon-particle-neutral-debug-readback",
                counter_byte_capacity + indirect_bytes,
                STORAGE_USAGE,
            ),
        }
    }

    fn bindings(&self) -> ParticleGpuRuntimeBufferBindings<'_> {
        ParticleGpuRuntimeBufferBindings {
            particles_a: &self.particles_a,
            particles_b: &self.particles_b,
            emitter_params: &self.emitter_params,
            alive_indices: &self.alive_indices,
            indirect_draw_args: &self.indirect_draw_args,
            counters: &self.counters,
            debug_readback: &self.debug_readback,
        }
    }
}

#[derive(Default)]
struct NeutralFrameShadow {
    alive_count: u32,
    spawned_total: u32,
    emitter_count: u32,
    per_emitter_spawned: Vec<u32>,
    indirect_draw_args: [u32; 4],
}

impl NeutralFrameShadow {
    fn counters_match(&self, frame: &RenderParticleGpuFrameExtract) -> bool {
        self.alive_count == bounded_particle_count(frame.alive_count)
            && self.spawned_total == bounded_particle_count(frame.spawned_total)
            && self.emitter_count == neutral_emitter_count(frame)
            && self.per_emitter_spawned.iter().copied().eq(frame
                .per_emitter_spawned
                .iter()
                .copied()
                .take(self.emitter_count as usize))
    }

    fn capture_counters(&mut self, frame: &RenderParticleGpuFrameExtract) {
        self.alive_count = bounded_particle_count(frame.alive_count);
        self.spawned_total = bounded_particle_count(frame.spawned_total);
        self.emitter_count = neutral_emitter_count(frame);
        self.per_emitter_spawned.clear();
        self.per_emitter_spawned.extend(
            frame
                .per_emitter_spawned
                .iter()
                .copied()
                .take(self.emitter_count as usize),
        );
        self.indirect_draw_args = bounded_indirect_draw_args(frame);
    }
}

fn create_buffer(
    device: &wgpu::Device,
    label: &'static str,
    size: u64,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size.max(WORD_BYTES),
        usage,
        mapped_at_creation: true,
    });
    buffer.slice(..).get_mapped_range_mut().fill(0);
    buffer.unmap();
    buffer
}

fn neutral_frame_is_empty(frame: &RenderParticleGpuFrameExtract) -> bool {
    frame.alive_count == 0
        && frame.spawned_total == 0
        && frame.per_emitter_spawned.iter().all(|count| *count == 0)
        && frame.indirect_draw_args[1] == 0
}

fn neutral_particle_slot_count(frame: &RenderParticleGpuFrameExtract) -> u32 {
    frame
        .alive_count
        .max(frame.spawned_total)
        .max(frame.indirect_draw_args[1])
        .max(1)
        .min(PARTICLE_GPU_MAX_PARTICLES)
}

fn neutral_emitter_count(frame: &RenderParticleGpuFrameExtract) -> u32 {
    frame
        .per_emitter_spawned
        .len()
        .max(1)
        .min(PARTICLE_GPU_NEUTRAL_MAX_EMITTERS as usize) as u32
}

fn write_counter_bytes(
    bytes: &mut Vec<u8>,
    frame: &RenderParticleGpuFrameExtract,
    emitter_capacity: u32,
) {
    bytes.clear();
    for word in [
        bounded_particle_count(frame.alive_count),
        bounded_particle_count(frame.spawned_total),
        0,
        0,
    ]
    .into_iter()
    .chain(
        frame
            .per_emitter_spawned
            .iter()
            .copied()
            .take(emitter_capacity as usize),
    ) {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
}

fn current_counter_byte_count(frame: &RenderParticleGpuFrameExtract) -> u64 {
    u64::from(PARTICLE_GPU_COUNTER_WORDS_BASE + neutral_emitter_count(frame)) * WORD_BYTES
}

fn bounded_particle_count(count: u32) -> u32 {
    count.min(PARTICLE_GPU_MAX_PARTICLES)
}

fn bounded_indirect_draw_args(frame: &RenderParticleGpuFrameExtract) -> [u32; 4] {
    let mut args = frame.indirect_draw_args;
    args[1] = bounded_particle_count(args[1]);
    args
}

fn write_alive_index_bytes(bytes: &mut Vec<u8>, alive_count: u32) {
    bytes.clear();
    for index in 0..alive_count.max(1) {
        bytes.extend_from_slice(&index.to_le_bytes());
    }
}

fn words4_to_bytes(words: [u32; 4]) -> [u8; 16] {
    let mut bytes = [0; 16];
    for (chunk, word) in bytes.chunks_exact_mut(4).zip(words) {
        chunk.copy_from_slice(&word.to_le_bytes());
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_capacity_uses_bounded_power_of_two_growth() {
        let frame = RenderParticleGpuFrameExtract {
            alive_count: 3,
            spawned_total: 5,
            per_emitter_spawned: vec![2, 3, 0],
            indirect_draw_args: [6, 3, 0, 0],
        };

        assert_eq!(neutral_particle_slot_count(&frame).next_power_of_two(), 8);
        assert_eq!(neutral_emitter_count(&frame).next_power_of_two(), 4);
        assert!(!neutral_frame_is_empty(&frame));
        assert!(neutral_frame_is_empty(
            &RenderParticleGpuFrameExtract::default()
        ));
    }

    #[test]
    fn neutral_payload_encoding_reuses_callers_scratch() {
        let frame = RenderParticleGpuFrameExtract {
            alive_count: 2,
            spawned_total: 3,
            per_emitter_spawned: vec![1, 2],
            indirect_draw_args: [6, 2, 0, 0],
        };
        let mut counters = Vec::with_capacity(32);
        let mut alive = Vec::with_capacity(16);

        write_counter_bytes(&mut counters, &frame, 2);
        write_alive_index_bytes(&mut alive, frame.alive_count);

        assert_eq!(counters.len(), 6 * std::mem::size_of::<u32>());
        assert_eq!(
            alive,
            [0_u32, 1]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect::<Vec<_>>()
        );
        assert_eq!(words4_to_bytes(frame.indirect_draw_args).len(), 16);
    }

    #[test]
    fn neutral_payload_caps_counts_to_allocated_particle_capacity() {
        let frame = RenderParticleGpuFrameExtract {
            alive_count: PARTICLE_GPU_MAX_PARTICLES + 1,
            spawned_total: PARTICLE_GPU_MAX_PARTICLES + 2,
            indirect_draw_args: [6, PARTICLE_GPU_MAX_PARTICLES + 3, 0, 0],
            ..RenderParticleGpuFrameExtract::default()
        };
        let mut counters = Vec::new();

        write_counter_bytes(&mut counters, &frame, 1);

        let words = counters
            .chunks_exact(std::mem::size_of::<u32>())
            .map(|bytes| u32::from_le_bytes(bytes.try_into().expect("word-sized chunk")))
            .collect::<Vec<_>>();
        assert_eq!(words[0], PARTICLE_GPU_MAX_PARTICLES);
        assert_eq!(words[1], PARTICLE_GPU_MAX_PARTICLES);
        assert_eq!(
            bounded_indirect_draw_args(&frame)[1],
            PARTICLE_GPU_MAX_PARTICLES
        );
    }

    #[test]
    fn neutral_emitter_metadata_stays_within_its_local_fallback_budget() {
        let frame = RenderParticleGpuFrameExtract {
            per_emitter_spawned: vec![1; PARTICLE_GPU_NEUTRAL_MAX_EMITTERS as usize + 1],
            ..RenderParticleGpuFrameExtract::default()
        };

        assert_eq!(
            neutral_emitter_count(&frame),
            PARTICLE_GPU_NEUTRAL_MAX_EMITTERS
        );
    }

    #[test]
    fn neutral_source_keeps_creation_and_upload_behind_change_gates() {
        let source = include_str!("neutral_buffers.rs")
            .split("\n#[cfg(test)]")
            .next()
            .unwrap_or_default();

        assert_eq!(source.matches("device.create_buffer(").count(), 1);
        assert_eq!(source.matches(": create_buffer(").count(), 7);
        assert_eq!(source.matches("if counters_changed").count(), 2);
        assert_eq!(source.matches("if alive_indices_changed").count(), 1);
        assert_eq!(source.matches("if indirect_args_changed").count(), 2);
        assert!(source.contains("if neutral_frame_is_empty(frame)"));
        assert!(source.contains("current == device"));
    }
}

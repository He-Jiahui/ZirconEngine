use zircon_runtime::core::framework::render::RenderParticleGpuFrameExtract;

use super::program::PARTICLE_GPU_INDIRECT_DRAW_WORDS;
use super::runtime_owner::ParticleGpuRuntimeBufferBindings;

const WORD_BYTES: u64 = std::mem::size_of::<u32>() as u64;
const NEUTRAL_IDENTITY_BYTES: u64 = WORD_BYTES;
const NEUTRAL_INDIRECT_BYTES: u64 = PARTICLE_GPU_INDIRECT_DRAW_WORDS as u64 * WORD_BYTES;
// Neutral fallback only needs bounded typed resource identity; it never simulates emitters.
pub(crate) const PARTICLE_GPU_NEUTRAL_MAX_EMITTERS: u32 = 4_096;

const STORAGE_USAGE: wgpu::BufferUsages = wgpu::BufferUsages::STORAGE
    .union(wgpu::BufferUsages::COPY_DST)
    .union(wgpu::BufferUsages::COPY_SRC);
const INDIRECT_USAGE: wgpu::BufferUsages = STORAGE_USAGE.union(wgpu::BufferUsages::INDIRECT);

#[derive(Default)]
pub(super) struct ParticleGpuNeutralBuffers {
    buffers: Option<NeutralBufferBundle>,
}

impl ParticleGpuNeutralBuffers {
    pub(super) fn prepare<'a>(
        &'a mut self,
        device: &wgpu::Device,
        frame: &RenderParticleGpuFrameExtract,
    ) -> Option<ParticleGpuRuntimeBufferBindings<'a>> {
        if neutral_frame_is_empty(frame) {
            return None;
        }

        if self.buffers.is_none() {
            self.buffers = Some(NeutralBufferBundle::new(device));
        }
        Some(
            self.buffers
                .as_ref()
                .expect("neutral identity buffers were prepared")
                .bindings(),
        )
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
    fn new(device: &wgpu::Device) -> Self {
        Self {
            particles_a: create_buffer(
                device,
                "zircon-particle-neutral-particles-a",
                NEUTRAL_IDENTITY_BYTES,
                STORAGE_USAGE,
            ),
            particles_b: create_buffer(
                device,
                "zircon-particle-neutral-particles-b",
                NEUTRAL_IDENTITY_BYTES,
                STORAGE_USAGE,
            ),
            emitter_params: create_buffer(
                device,
                "zircon-particle-neutral-emitter-params",
                NEUTRAL_IDENTITY_BYTES,
                STORAGE_USAGE,
            ),
            counters: create_buffer(
                device,
                "zircon-particle-neutral-counters",
                NEUTRAL_IDENTITY_BYTES,
                STORAGE_USAGE,
            ),
            alive_indices: create_buffer(
                device,
                "zircon-particle-neutral-alive-indices",
                NEUTRAL_IDENTITY_BYTES,
                STORAGE_USAGE,
            ),
            indirect_draw_args: create_buffer(
                device,
                "zircon-particle-neutral-indirect-draw-args",
                NEUTRAL_INDIRECT_BYTES,
                INDIRECT_USAGE,
            ),
            debug_readback: create_buffer(
                device,
                "zircon-particle-neutral-debug-readback",
                NEUTRAL_IDENTITY_BYTES,
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

fn create_buffer(
    device: &wgpu::Device,
    label: &'static str,
    size: u64,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    // Wgpu initializes unmapped buffers lazily before first read; no host staging is required.
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size.max(WORD_BYTES),
        usage,
        mapped_at_creation: false,
    })
}

fn neutral_frame_is_empty(frame: &RenderParticleGpuFrameExtract) -> bool {
    frame.alive_count == 0
        && frame.spawned_total == 0
        && frame.per_emitter_spawned.iter().all(|count| *count == 0)
        && frame.indirect_draw_args[1] == 0
}

#[cfg(test)]
#[path = "tests/neutral_buffers.rs"]
mod tests;

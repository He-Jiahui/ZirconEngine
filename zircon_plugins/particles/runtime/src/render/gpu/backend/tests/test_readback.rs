use std::sync::mpsc;

use super::super::readback::ParticleGpuCounterReadback;
use super::{ParticleGpuBackend, ParticleGpuBackendError};

impl ParticleGpuBackend {
    pub fn read_counter_readback(
        &self,
        device: &wgpu::Device,
    ) -> Result<ParticleGpuCounterReadback, ParticleGpuBackendError> {
        let word_count =
            (self.program.resources.counter_bytes / std::mem::size_of::<u32>() as u64) as usize;
        let words = read_buffer_u32s_at(device, &self.debug_readback_buffer, 0, word_count)?;
        ParticleGpuCounterReadback::from_words(&words, self.program.layout.emitter_count)
            .map_err(ParticleGpuBackendError::from)
    }

    pub fn read_indirect_draw_args_readback(
        &self,
        device: &wgpu::Device,
    ) -> Result<[u32; 4], ParticleGpuBackendError> {
        let words = read_buffer_u32s_at(
            device,
            &self.debug_readback_buffer,
            self.program.resources.counter_bytes,
            4,
        )?;
        Ok([words[0], words[1], words[2], words[3]])
    }

    pub fn read_render_outputs_readback(
        &self,
        device: &wgpu::Device,
    ) -> Result<
        zircon_runtime::core::framework::render::RenderParticleGpuReadbackOutputs,
        ParticleGpuBackendError,
    > {
        let counters = self.read_counter_readback(device)?;
        let indirect_draw_args = self.read_indirect_draw_args_readback(device)?;
        Ok(counters.to_render_outputs(indirect_draw_args))
    }
}

fn read_buffer_u32s_at(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    byte_offset: u64,
    word_count: usize,
) -> Result<Vec<u32>, ParticleGpuBackendError> {
    if word_count == 0 {
        return Ok(Vec::new());
    }

    let byte_count = word_count * std::mem::size_of::<u32>();
    let map_offset = byte_offset - (byte_offset % wgpu::MAP_ALIGNMENT);
    let mapped_prefix_bytes = (byte_offset - map_offset) as usize;
    let mapped_byte_count = mapped_prefix_bytes + byte_count;
    let slice = buffer.slice(map_offset..byte_offset + byte_count as u64);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|error| ParticleGpuBackendError::ReadbackMap(error.to_string()))?;
    receiver
        .recv()
        .map_err(|error| ParticleGpuBackendError::ReadbackMap(error.to_string()))?
        .map_err(|error| ParticleGpuBackendError::ReadbackMap(error.to_string()))?;

    let mapped = slice.get_mapped_range();
    let words = mapped[mapped_prefix_bytes..mapped_byte_count]
        .chunks_exact(std::mem::size_of::<u32>())
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect::<Vec<_>>();
    drop(mapped);
    buffer.unmap();

    Ok(words)
}

use std::sync::mpsc;

use zircon_runtime::graphics::GraphicsError;

pub(super) fn read_buffer_u32s(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    word_count: usize,
) -> Result<Vec<u32>, GraphicsError> {
    if word_count == 0 {
        return Ok(Vec::new());
    }

    let slice = buffer.slice(..(word_count * std::mem::size_of::<u32>()) as u64);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|error| GraphicsError::BufferMap(error.to_string()))?;
    receiver
        .recv()
        .map_err(|error| GraphicsError::BufferMap(error.to_string()))?
        .map_err(|error| GraphicsError::BufferMap(error.to_string()))?;

    let mapped = slice.get_mapped_range();
    let data = bytemuck::cast_slice(&mapped[..]).to_vec();
    drop(mapped);
    buffer.unmap();

    Ok(data)
}

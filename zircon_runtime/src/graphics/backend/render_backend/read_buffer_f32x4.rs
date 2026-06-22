use std::sync::mpsc;

use crate::graphics::debug_markers::{insert_marker, RENDERDOC_MARKER_READBACK};
use crate::graphics::types::GraphicsError;

const F32X4_BYTE_LEN: u64 = 16;

pub(crate) fn read_buffer_f32x4(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    source: &wgpu::Buffer,
) -> Result<[f32; 4], GraphicsError> {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-readback-f32x4"),
        size: F32X4_BYTE_LEN,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-readback-f32x4-encoder"),
    });
    insert_marker(&mut encoder, RENDERDOC_MARKER_READBACK);
    encoder.copy_buffer_to_buffer(source, 0, &buffer, 0, F32X4_BYTE_LEN);
    queue.submit([encoder.finish()]);

    let slice = buffer.slice(..);
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
    let words = f32x4_from_le_bytes(&mapped);
    drop(mapped);
    buffer.unmap();

    Ok(words)
}

fn f32x4_from_le_bytes(bytes: &[u8]) -> [f32; 4] {
    let mut words = [0.0_f32; 4];
    for (word, chunk) in words.iter_mut().zip(bytes.chunks_exact(4)) {
        *word = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }
    words
}

#[cfg(test)]
mod tests {
    use super::f32x4_from_le_bytes;

    #[test]
    fn f32x4_readback_decodes_little_endian_words() {
        let mut bytes = Vec::new();
        for word in [1.0_f32, 2.5, -4.0, 0.25] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }

        assert_eq!(f32x4_from_le_bytes(&bytes), [1.0, 2.5, -4.0, 0.25]);
    }
}

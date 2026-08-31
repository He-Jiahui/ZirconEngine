// Staged Plan 11 acquisition helpers land before the runtime bake scheduler consumes them.

use std::sync::mpsc;

use crate::core::framework::render::SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT;
use crate::graphics::debug_markers::{RENDERDOC_MARKER_READBACK, insert_marker};
use crate::graphics::types::GraphicsError;

const F32X4_BYTE_LEN: u64 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BufferByteReadback {
    pub source_offset: u64,
    pub byte_len: u64,
    pub label: &'static str,
}

pub(crate) fn read_buffer_bytes(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    source: &wgpu::Buffer,
    readback: BufferByteReadback,
) -> Result<Vec<u8>, GraphicsError> {
    validate_buffer_copy_size(readback.byte_len)?;
    let buffer_label = format!("{}-buffer", readback.label);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&buffer_label),
        size: readback.byte_len,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let encoder_label = format!("{}-encoder", readback.label);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some(&encoder_label),
    });
    insert_marker(&mut encoder, RENDERDOC_MARKER_READBACK);
    encoder.copy_buffer_to_buffer(
        source,
        readback.source_offset,
        &buffer,
        0,
        readback.byte_len,
    );
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
    let bytes = mapped.to_vec();
    drop(mapped);
    buffer.unmap();

    Ok(bytes)
}

pub(crate) fn read_buffer_f32x4_array_bytes(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    source: &wgpu::Buffer,
    vec4_count: usize,
    label: &'static str,
) -> Result<Vec<u8>, GraphicsError> {
    let byte_len = f32x4_array_readback_size_bytes(vec4_count)
        .ok_or_else(|| GraphicsError::BufferMap("invalid f32x4 array readback size".into()))?;
    read_buffer_bytes(
        device,
        queue,
        source,
        BufferByteReadback {
            source_offset: 0,
            byte_len,
            label,
        },
    )
}

pub(crate) fn read_buffer_sh9_f32x4_bytes(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    source: &wgpu::Buffer,
) -> Result<Vec<u8>, GraphicsError> {
    read_buffer_f32x4_array_bytes(
        device,
        queue,
        source,
        SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT,
        "zircon-readback-sh9-f32x4",
    )
}

fn f32x4_array_readback_size_bytes(vec4_count: usize) -> Option<u64> {
    u64::try_from(vec4_count)
        .ok()
        .and_then(|count| count.checked_mul(F32X4_BYTE_LEN))
        .filter(|size| *size > 0)
}

fn validate_buffer_copy_size(byte_len: u64) -> Result<(), GraphicsError> {
    if byte_len == 0 {
        return Err(GraphicsError::BufferMap(
            "buffer readback byte length must be non-zero".into(),
        ));
    }
    if byte_len % wgpu::COPY_BUFFER_ALIGNMENT as u64 != 0 {
        return Err(GraphicsError::BufferMap(format!(
            "buffer readback byte length {byte_len} is not aligned to {}",
            wgpu::COPY_BUFFER_ALIGNMENT
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{F32X4_BYTE_LEN, f32x4_array_readback_size_bytes, validate_buffer_copy_size};
    use crate::core::framework::render::{
        IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES, SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT,
    };

    #[test]
    fn sh9_f32x4_readback_size_matches_artifact_layout() {
        assert_eq!(
            f32x4_array_readback_size_bytes(SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT),
            Some(IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64)
        );
        assert_eq!(IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64, 9 * F32X4_BYTE_LEN);
    }

    #[test]
    fn buffer_readback_size_rejects_zero_and_unaligned_copies() {
        assert!(validate_buffer_copy_size(0).is_err());
        assert!(validate_buffer_copy_size(3).is_err());
        assert!(validate_buffer_copy_size(wgpu::COPY_BUFFER_ALIGNMENT as u64).is_ok());
    }

    #[test]
    fn synchronous_buffer_readback_owner_is_test_only() {
        let backend_root = include_str!("mod.rs");
        let graphics_backend_root = include_str!("../mod.rs");

        assert!(backend_root.contains("#[cfg(test)]\nmod read_buffer_bytes;"));
        assert!(backend_root.contains("#[cfg(test)]\npub(crate) use read_buffer_bytes::{"));
        assert!(
            graphics_backend_root
                .contains("#[cfg(test)]\npub(crate) use render_backend::{\n    read_buffer_bytes")
        );
    }
}

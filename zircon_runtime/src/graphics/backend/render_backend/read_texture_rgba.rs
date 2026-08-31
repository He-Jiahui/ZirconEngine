use std::sync::mpsc;

use crate::core::math::UVec2;

use crate::graphics::debug_markers::{RENDERDOC_MARKER_READBACK, insert_marker};
use crate::graphics::types::GraphicsError;

pub(crate) fn read_texture_rgba(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: UVec2,
) -> Result<Vec<u8>, GraphicsError> {
    let bytes_per_pixel = 4_u32;
    let unpadded_bytes_per_row = size.x * bytes_per_pixel;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buffer_size = padded_bytes_per_row as u64 * size.y as u64;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-readback"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-readback-encoder"),
    });
    insert_marker(&mut encoder, RENDERDOC_MARKER_READBACK);
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(size.y),
            },
        },
        wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
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
    let rgba = strip_padded_rgba_rows(
        &mapped,
        size.y as usize,
        unpadded_bytes_per_row as usize,
        padded_bytes_per_row as usize,
    );
    drop(mapped);
    buffer.unmap();

    Ok(rgba)
}

fn strip_padded_rgba_rows(
    mapped: &[u8],
    row_count: usize,
    unpadded_bytes_per_row: usize,
    padded_bytes_per_row: usize,
) -> Vec<u8> {
    let output_len = row_count * unpadded_bytes_per_row;
    if unpadded_bytes_per_row == padded_bytes_per_row {
        return mapped[..output_len].to_vec();
    }

    let mut rgba = vec![0_u8; output_len];
    for row in 0..row_count {
        let source_offset = row * padded_bytes_per_row;
        let target_offset = row * unpadded_bytes_per_row;
        rgba[target_offset..target_offset + unpadded_bytes_per_row]
            .copy_from_slice(&mapped[source_offset..source_offset + unpadded_bytes_per_row]);
    }
    rgba
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::strip_padded_rgba_rows;

    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn optimization_batch_dc_rgba8_readback_preserves_padded_rows() {
        let mapped = [
            1, 2, 3, 4, 5, 6, 7, 8, 90, 91, 92, 93, 94, 95, 96, 97, 9, 10, 11, 12, 13, 14, 15, 16,
            80, 81, 82, 83, 84, 85, 86, 87,
        ];

        assert_eq!(
            strip_padded_rgba_rows(&mapped, 2, 8, 16),
            (1_u8..=16).collect::<Vec<_>>()
        );
    }

    #[test]
    fn optimization_batch_dc_rgba8_readback_preserves_aligned_rows() {
        let mapped = (0..=255_u8).cycle().take(1_024).collect::<Vec<_>>();

        assert_eq!(
            strip_padded_rgba_rows(&mapped, 4, 256, 256),
            legacy_strip_padded_rgba_rows(&mapped, 4, 256, 256)
        );
    }

    #[test]
    fn optimization_batch_dc_rgba8_readback_uses_contiguous_copy_for_aligned_rows() {
        let production = include_str!("read_texture_rgba.rs")
            .split_once("#[cfg(test)]")
            .expect("production source and tests must remain separated")
            .0;

        assert!(production.contains("if unpadded_bytes_per_row == padded_bytes_per_row"));
        assert!(production.contains("return mapped[..output_len].to_vec();"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_dc_runtime408_rgba8_aligned_readback_copy_p95() {
        const ROW_BYTES: usize = 256;
        const ROW_COUNT: usize = 32_768;
        let mapped = (0..=255_u8)
            .cycle()
            .take(ROW_BYTES * ROW_COUNT)
            .collect::<Vec<_>>();

        for _ in 0..3 {
            black_box(legacy_strip_padded_rgba_rows(
                black_box(&mapped),
                ROW_COUNT,
                ROW_BYTES,
                ROW_BYTES,
            ));
            black_box(strip_padded_rgba_rows(
                black_box(&mapped),
                ROW_COUNT,
                ROW_BYTES,
                ROW_BYTES,
            ));
        }

        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(|| {
                    legacy_strip_padded_rgba_rows(&mapped, ROW_COUNT, ROW_BYTES, ROW_BYTES)
                }));
                optimized.push(measure(|| {
                    strip_padded_rgba_rows(&mapped, ROW_COUNT, ROW_BYTES, ROW_BYTES)
                }));
            } else {
                optimized.push(measure(|| {
                    strip_padded_rgba_rows(&mapped, ROW_COUNT, ROW_BYTES, ROW_BYTES)
                }));
                legacy.push(measure(|| {
                    legacy_strip_padded_rgba_rows(&mapped, ROW_COUNT, ROW_BYTES, ROW_BYTES)
                }));
            }
        }

        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME408_RGBA8_ALIGNED_READBACK_COPY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} row_count={ROW_COUNT} row_bytes={ROW_BYTES} payload_bytes={} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            mapped.len(),
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn legacy_strip_padded_rgba_rows(
        mapped: &[u8],
        row_count: usize,
        unpadded_bytes_per_row: usize,
        padded_bytes_per_row: usize,
    ) -> Vec<u8> {
        let mut rgba = vec![0_u8; row_count * unpadded_bytes_per_row];
        for row in 0..row_count {
            let source_offset = row * padded_bytes_per_row;
            let target_offset = row * unpadded_bytes_per_row;
            rgba[target_offset..target_offset + unpadded_bytes_per_row]
                .copy_from_slice(&mapped[source_offset..source_offset + unpadded_bytes_per_row]);
        }
        rgba
    }

    fn measure(run: impl FnOnce() -> Vec<u8>) -> u128 {
        let started = Instant::now();
        black_box(run());
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}

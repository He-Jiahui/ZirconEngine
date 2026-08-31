use std::sync::mpsc;

use crate::graphics::debug_markers::{RENDERDOC_MARKER_READBACK, insert_marker};
use crate::graphics::types::GraphicsError;

pub(crate) fn read_texture_rgba16float_3d(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: [u32; 3],
) -> Result<Vec<u8>, GraphicsError> {
    let bytes_per_pixel = 8_u32;
    let unpadded_bytes_per_row = size[0] * bytes_per_pixel;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let rows_per_image = size[1].max(1);
    let buffer_size = padded_bytes_per_row as u64 * rows_per_image as u64 * size[2].max(1) as u64;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-readback-rgba16float-3d"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-readback-rgba16float-3d-encoder"),
    });
    insert_marker(&mut encoder, RENDERDOC_MARKER_READBACK);
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(rows_per_image),
            },
        },
        wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: size[2],
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
    let rgba = strip_padded_rgba16float_3d_rows(
        &mapped,
        size,
        unpadded_bytes_per_row as usize,
        padded_bytes_per_row as usize,
    );
    drop(mapped);
    buffer.unmap();

    Ok(rgba)
}

fn strip_padded_rgba16float_3d_rows(
    mapped: &[u8],
    size: [u32; 3],
    unpadded_bytes_per_row: usize,
    padded_bytes_per_row: usize,
) -> Vec<u8> {
    let row_count = size[1] as usize;
    let slice_count = size[2] as usize;
    let output_len = size[0] as usize * row_count * slice_count * 8;
    if unpadded_bytes_per_row == padded_bytes_per_row {
        return mapped[..output_len].to_vec();
    }

    let mut rgba = vec![0_u8; output_len];
    for slice in 0..slice_count {
        for row in 0..row_count {
            let source_offset =
                slice * row_count * padded_bytes_per_row + row * padded_bytes_per_row;
            let target_offset =
                slice * row_count * unpadded_bytes_per_row + row * unpadded_bytes_per_row;
            rgba[target_offset..target_offset + unpadded_bytes_per_row]
                .copy_from_slice(&mapped[source_offset..source_offset + unpadded_bytes_per_row]);
        }
    }
    rgba
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::strip_padded_rgba16float_3d_rows;

    #[test]
    fn rgba16float_3d_readback_strips_row_padding_per_slice() {
        let size = [2, 2, 2];
        let unpadded = 16;
        let padded = 32;
        let mut mapped = vec![0_u8; padded * size[1] as usize * size[2] as usize];
        for (index, row) in mapped.chunks_exact_mut(padded).enumerate() {
            row[..unpadded].fill(index as u8 + 1);
            row[unpadded..].fill(255);
        }

        let stripped = strip_padded_rgba16float_3d_rows(&mapped, size, unpadded, padded);

        assert_eq!(stripped.len(), 64);
        assert_eq!(&stripped[0..16], &[1_u8; 16]);
        assert_eq!(&stripped[16..32], &[2_u8; 16]);
        assert_eq!(&stripped[32..48], &[3_u8; 16]);
        assert_eq!(&stripped[48..64], &[4_u8; 16]);
        assert!(!stripped.contains(&255));
    }

    #[test]
    fn optimization_batch_db_aligned_readback_matches_row_copy() {
        let size = [4, 3, 2];
        let bytes_per_row = size[0] as usize * 8;
        let mapped = (0..bytes_per_row * size[1] as usize * size[2] as usize)
            .map(|value| value as u8)
            .collect::<Vec<_>>();

        assert_eq!(
            strip_padded_rgba16float_3d_rows(&mapped, size, bytes_per_row, bytes_per_row),
            legacy_strip_rows(&mapped, size, bytes_per_row, bytes_per_row)
        );
    }

    #[test]
    fn optimization_batch_db_aligned_readback_uses_contiguous_copy() {
        let source = include_str!("read_texture_rgba16float_3d.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("if unpadded_bytes_per_row == padded_bytes_per_row"));
        assert!(production.contains("return mapped[..output_len].to_vec();"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_db_aligned_readback_copy_p95() {
        const SIZE: [u32; 3] = [128, 128, 64];
        const SAMPLE_COUNT: usize = 17;
        const BYTES_PER_ROW: usize = SIZE[0] as usize * 8;
        let mapped = (0..BYTES_PER_ROW * SIZE[1] as usize * SIZE[2] as usize)
            .map(|index| index.wrapping_mul(131) as u8)
            .collect::<Vec<_>>();

        let (legacy_samples, optimized_samples) = paired_samples::<SAMPLE_COUNT>(
            || legacy_strip_rows(&mapped, SIZE, BYTES_PER_ROW, BYTES_PER_ROW),
            || strip_padded_rgba16float_3d_rows(&mapped, SIZE, BYTES_PER_ROW, BYTES_PER_ROW),
        );
        assert_eq!(
            legacy_strip_rows(&mapped, SIZE, BYTES_PER_ROW, BYTES_PER_ROW),
            strip_padded_rgba16float_3d_rows(&mapped, SIZE, BYTES_PER_ROW, BYTES_PER_ROW)
        );

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        println!(
            "PERF_RESULT RUNTIME407_RGBA16FLOAT_ALIGNED_READBACK_COPY_BENCH_V1 bytes={} rows={} samples={SAMPLE_COUNT} sample_order=alternating legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}",
            mapped.len(),
            SIZE[1] as usize * SIZE[2] as usize,
        );
        assert!(
            optimized_p95 * 10 <= legacy_p95 * 7,
            "optimized P95 {optimized_p95}ns must be no more than 70% of legacy P95 {legacy_p95}ns"
        );
    }

    fn legacy_strip_rows(
        mapped: &[u8],
        size: [u32; 3],
        unpadded_bytes_per_row: usize,
        padded_bytes_per_row: usize,
    ) -> Vec<u8> {
        let row_count = size[1] as usize;
        let slice_count = size[2] as usize;
        let mut rgba = vec![0_u8; size[0] as usize * row_count * slice_count * 8];
        for slice in 0..slice_count {
            for row in 0..row_count {
                let source_offset =
                    slice * row_count * padded_bytes_per_row + row * padded_bytes_per_row;
                let target_offset =
                    slice * row_count * unpadded_bytes_per_row + row * unpadded_bytes_per_row;
                rgba[target_offset..target_offset + unpadded_bytes_per_row].copy_from_slice(
                    &mapped[source_offset..source_offset + unpadded_bytes_per_row],
                );
            }
        }
        rgba
    }

    fn paired_samples<const SAMPLE_COUNT: usize, T>(
        mut legacy: impl FnMut() -> T,
        mut optimized: impl FnMut() -> T,
    ) -> (Vec<u128>, Vec<u128>) {
        black_box(legacy());
        black_box(optimized());
        let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample_index in 0..SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(sample(&mut legacy));
                optimized_samples.push(sample(&mut optimized));
            } else {
                optimized_samples.push(sample(&mut optimized));
                legacy_samples.push(sample(&mut legacy));
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn sample<T>(operation: &mut impl FnMut() -> T) -> u128 {
        let started = Instant::now();
        let result = black_box(operation());
        let elapsed = started.elapsed().as_nanos();
        black_box(result);
        elapsed
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        assert!(!sorted.is_empty());
        assert!((1..=100).contains(&percentile));
        sorted[(sorted.len() * percentile).div_ceil(100) - 1]
    }
}

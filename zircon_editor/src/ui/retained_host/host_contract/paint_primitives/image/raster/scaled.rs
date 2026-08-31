use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::PixelRect;
use super::pixel::write_bilinear_rgba_pixel;

pub(in crate::ui::retained_host::host_contract) fn draw_scaled_rgba_image_pixels(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    target: &PixelRect,
    image_width: u32,
    image_height: u32,
    rgba: &[u8],
) {
    let rect_width = rect.width.max(1.0);
    let rect_height = rect.height.max(1.0);
    let frame_width = frame.width() as usize;
    let bytes = frame.as_bytes_mut();
    let source_x_samples = (target.x0..target.x1)
        .map(|x| source_axis_sample(x, rect.x, rect_width, image_width))
        .collect::<Vec<_>>();

    for y in target.y0..target.y1 {
        let source_y = source_axis_sample(y, rect.y, rect_height, image_height);
        let mut destination_offset = (y as usize * frame_width + target.x0 as usize) * 4;
        for sample in &source_x_samples {
            write_bilinear_rgba_pixel(
                bytes,
                destination_offset,
                rgba,
                image_width,
                [sample.lower, sample.upper],
                [source_y.lower, source_y.upper],
                [sample.mix, source_y.mix],
            );
            destination_offset += 4;
        }
    }
}

#[derive(Clone, Copy)]
struct SourceAxisSample {
    lower: u32,
    upper: u32,
    mix: f32,
}

fn source_axis_sample(
    destination: u32,
    destination_origin: f32,
    destination_extent: f32,
    source_extent: u32,
) -> SourceAxisSample {
    let coordinate = source_sample_coordinate(
        destination,
        destination_origin,
        destination_extent,
        source_extent,
    );
    let lower = coordinate.floor() as u32;
    SourceAxisSample {
        lower,
        upper: lower.saturating_add(1).min(source_extent - 1),
        mix: coordinate - lower as f32,
    }
}

fn source_sample_coordinate(
    destination: u32,
    destination_origin: f32,
    destination_extent: f32,
    source_extent: u32,
) -> f32 {
    ((((destination as f32 + 0.5 - destination_origin) / destination_extent)
        * source_extent as f32)
        - 0.5)
        .clamp(0.0, source_extent.saturating_sub(1) as f32)
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::{source_axis_sample, source_sample_coordinate};

    #[test]
    fn optimization_batch_20260830et_editor554_reuses_x_axis_samples_across_rows() {
        let production = include_str!("scaled.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("scaled image production source");

        assert!(production.contains("let source_x_samples"));
        assert!(production.contains("for sample in &source_x_samples"));
        assert!(!production.contains("source_sample_coordinate(x,"));
    }

    #[test]
    fn optimization_batch_20260830et_editor554_cached_samples_match_direct_coordinates() {
        for destination in 3..197 {
            let coordinate = source_sample_coordinate(destination, 2.25, 211.5, 127);
            let sample = source_axis_sample(destination, 2.25, 211.5, 127);
            let lower = coordinate.floor() as u32;

            assert_eq!(sample.lower, lower);
            assert_eq!(sample.upper, lower.saturating_add(1).min(126));
            assert_eq!(sample.mix, coordinate - lower as f32);
        }
    }

    #[test]
    #[ignore = "deterministic performance marker"]
    fn optimization_batch_20260830et_editor554_x_axis_sample_cache_benchmark() {
        const WIDTH: u32 = 512;
        const HEIGHT: u32 = 512;
        const SAMPLES: usize = 9;
        let mut legacy_samples = Vec::with_capacity(SAMPLES);
        let mut optimized_samples = Vec::with_capacity(SAMPLES);

        for _ in 0..SAMPLES {
            let started = Instant::now();
            let mut checksum = 0_u32;
            for _ in 0..HEIGHT {
                for x in 0..WIDTH {
                    checksum ^= source_axis_sample(x, 0.25, 511.5, 384).lower;
                }
            }
            black_box(checksum);
            legacy_samples.push(started.elapsed());

            let started = Instant::now();
            let cached = (0..WIDTH)
                .map(|x| source_axis_sample(x, 0.25, 511.5, 384))
                .collect::<Vec<_>>();
            let mut checksum = 0_u32;
            for _ in 0..HEIGHT {
                for sample in &cached {
                    checksum ^= sample.lower;
                }
            }
            black_box(checksum);
            optimized_samples.push(started.elapsed());
        }

        legacy_samples.sort_unstable();
        optimized_samples.sort_unstable();
        println!(
            "EDITOR554_X_AXIS_SAMPLE_CACHE_BENCH_V1 legacy={:?} optimized={:?}",
            legacy_samples[SAMPLES / 2],
            optimized_samples[SAMPLES / 2]
        );
    }
}

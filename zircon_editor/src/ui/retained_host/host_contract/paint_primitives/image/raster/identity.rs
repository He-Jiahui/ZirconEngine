use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::PixelRect;

pub(in crate::ui::retained_host::host_contract) fn try_copy_opaque_identity_image_rows(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    target: &PixelRect,
    image_width: u32,
    image_height: u32,
    rgba: &[u8],
) -> bool {
    if !is_identity_image_mapping(rect, image_width, image_height) {
        return false;
    }

    let source_x0 = (target.x0 as i64 - rect.x as i64).max(0) as usize;
    let source_y0 = (target.y0 as i64 - rect.y as i64).max(0) as usize;
    let width = (target.x1 - target.x0) as usize;
    let height = (target.y1 - target.y0) as usize;
    let image_width = image_width as usize;
    let image_height = image_height as usize;
    if width == 0
        || height == 0
        || source_x0 + width > image_width
        || source_y0 + height > image_height
    {
        return false;
    }

    let row_byte_len = width * 4;
    let source_stride = image_width * 4;
    let mut source_start = (source_y0 * image_width + source_x0) * 4;
    for _ in 0..height {
        let source_end = source_start + row_byte_len;
        if !rgba[source_start..source_end]
            .chunks_exact(4)
            .all(|pixel| pixel[3] == 255)
        {
            return false;
        }
        source_start += source_stride;
    }

    let frame_width = frame.width() as usize;
    let destination_stride = frame_width * 4;
    let mut source_start = (source_y0 * image_width + source_x0) * 4;
    let mut destination_start = (target.y0 as usize * frame_width + target.x0 as usize) * 4;
    let bytes = frame.as_bytes_mut();
    for _ in 0..height {
        let source_end = source_start + row_byte_len;
        let destination_end = destination_start + row_byte_len;
        bytes[destination_start..destination_end].copy_from_slice(&rgba[source_start..source_end]);
        source_start += source_stride;
        destination_start += destination_stride;
    }
    true
}

fn is_identity_image_mapping(rect: &FrameRect, image_width: u32, image_height: u32) -> bool {
    rect.x.fract().abs() <= f32::EPSILON
        && rect.y.fract().abs() <= f32::EPSILON
        && (rect.width - image_width as f32).abs() <= f32::EPSILON
        && (rect.height - image_height as f32).abs() <= f32::EPSILON
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    #[test]
    fn optimization_batch_20260830et_editor555_advances_identity_rows_by_stride() {
        let production = include_str!("identity.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("identity image production source");

        assert!(production.contains("source_start += source_stride"));
        assert!(production.contains("destination_start += destination_stride"));
        assert!(!production.contains("source_y0 + row"));
        assert!(!production.contains("target.y0 as usize + row"));
    }

    #[test]
    #[ignore = "deterministic performance marker"]
    fn optimization_batch_20260830et_editor555_identity_row_stride_benchmark() {
        const HEIGHT: usize = 65_536;
        const SAMPLES: usize = 9;
        let image_width = black_box(257_usize);
        let frame_width = black_box(521_usize);
        let mut legacy_samples = Vec::with_capacity(SAMPLES);
        let mut optimized_samples = Vec::with_capacity(SAMPLES);

        for _ in 0..SAMPLES {
            let started = Instant::now();
            let mut checksum = 0_usize;
            for row in 0..HEIGHT {
                let source = ((17 + row) * image_width + 3) * 4;
                let destination = ((29 + row) * frame_width + 7) * 4;
                checksum ^= source ^ destination;
            }
            black_box(checksum);
            legacy_samples.push(started.elapsed());

            let started = Instant::now();
            let source_stride = image_width * 4;
            let destination_stride = frame_width * 4;
            let mut source = (17 * image_width + 3) * 4;
            let mut destination = (29 * frame_width + 7) * 4;
            let mut checksum = 0_usize;
            for _ in 0..HEIGHT {
                checksum ^= source ^ destination;
                source += source_stride;
                destination += destination_stride;
            }
            black_box(checksum);
            optimized_samples.push(started.elapsed());
        }

        legacy_samples.sort_unstable();
        optimized_samples.sort_unstable();
        println!(
            "EDITOR555_IDENTITY_ROW_STRIDE_BENCH_V1 legacy={:?} optimized={:?}",
            legacy_samples[SAMPLES / 2],
            optimized_samples[SAMPLES / 2]
        );
    }
}

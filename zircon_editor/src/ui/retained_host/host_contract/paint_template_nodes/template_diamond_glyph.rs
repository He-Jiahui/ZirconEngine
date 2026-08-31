use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::Arc;

use super::super::data::FrameRect;
use super::render_commands::HostPaintCommand;

const DIAMOND_SAMPLES_PER_AXIS: u32 = 4;
const MAX_CACHED_DIAMOND_RASTERS: usize = 32;
const MAX_DIAMOND_RASTER_EDGE: u32 = 257;

thread_local! {
    static CACHED_DIAMOND_RASTERS: RefCell<VecDeque<CachedDiamondRaster>> =
        const { RefCell::new(VecDeque::new()) };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DiamondRasterKey {
    edge: u32,
    color: [u8; 4],
}

struct CachedDiamondRaster {
    key: DiamondRasterKey,
    resource_key: String,
    rgba: Arc<[u8]>,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_aa_diamond(
    commands: &mut Vec<HostPaintCommand>,
    x: f32,
    y: f32,
    radius: i32,
    color: [u8; 4],
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let Ok(radius) = u32::try_from(radius) else {
        return;
    };
    let Some(edge) = radius
        .checked_mul(2)
        .and_then(|diameter| diameter.checked_add(1))
    else {
        return;
    };
    if edge > MAX_DIAMOND_RASTER_EDGE {
        return;
    }

    let raster = cached_diamond_raster(DiamondRasterKey { edge, color });
    let half_edge = edge as f32 * 0.5;
    commands.push(HostPaintCommand::image_pixels(
        FrameRect {
            x: x - half_edge,
            y: y - half_edge,
            width: edge as f32,
            height: edge as f32,
        },
        Some(clip.clone()),
        order,
        raster.resource_key,
        edge,
        edge,
        raster.rgba,
        None,
        opacity,
    ));
}

fn cached_diamond_raster(key: DiamondRasterKey) -> CachedDiamondRaster {
    CACHED_DIAMOND_RASTERS.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(index) = cache.iter().position(|entry| entry.key == key) {
            let entry = cache.remove(index).expect("cached index must remain valid");
            let result = CachedDiamondRaster {
                key: entry.key,
                resource_key: entry.resource_key.clone(),
                rgba: Arc::clone(&entry.rgba),
            };
            cache.push_front(entry);
            return result;
        }

        let entry = CachedDiamondRaster {
            key,
            resource_key: diamond_resource_key(key),
            rgba: diamond_pixels(key.edge, key.color).into(),
        };
        let result = CachedDiamondRaster {
            key: entry.key,
            resource_key: entry.resource_key.clone(),
            rgba: Arc::clone(&entry.rgba),
        };
        cache.push_front(entry);
        cache.truncate(MAX_CACHED_DIAMOND_RASTERS);
        result
    })
}

fn diamond_pixels(edge: u32, color: [u8; 4]) -> Vec<u8> {
    let mut rgba = vec![0; edge as usize * edge as usize * 4];
    for y in 0..edge {
        for x in 0..edge {
            let coverage = diamond_sample_coverage(x, y, edge);
            if coverage == 0 {
                continue;
            }
            let offset = ((y as usize * edge as usize) + x as usize) * 4;
            rgba[offset..offset + 3].copy_from_slice(&color[..3]);
            rgba[offset + 3] = scale_alpha_by_coverage(color[3], coverage);
        }
    }
    rgba
}

fn diamond_sample_coverage(x: u32, y: u32, edge: u32) -> u8 {
    let center = edge as f32 * 0.5;
    let radius = center;
    let mut covered_samples = 0;
    for sample_y in 0..DIAMOND_SAMPLES_PER_AXIS {
        for sample_x in 0..DIAMOND_SAMPLES_PER_AXIS {
            let px = x as f32 + (sample_x as f32 + 0.5) / DIAMOND_SAMPLES_PER_AXIS as f32;
            let py = y as f32 + (sample_y as f32 + 0.5) / DIAMOND_SAMPLES_PER_AXIS as f32;
            if (px - center).abs() + (py - center).abs() <= radius {
                covered_samples += 1;
            }
        }
    }
    let sample_count = DIAMOND_SAMPLES_PER_AXIS * DIAMOND_SAMPLES_PER_AXIS;
    ((covered_samples * 255 + sample_count / 2) / sample_count) as u8
}

fn scale_alpha_by_coverage(alpha: u8, coverage: u8) -> u8 {
    ((u16::from(alpha) * u16::from(coverage) + 127) / 255) as u8
}

fn diamond_resource_key(key: DiamondRasterKey) -> String {
    format!(
        "icon-raster:analytic-diamond:{}:{:02x}{:02x}{:02x}{:02x}",
        key.edge, key.color[0], key.color[1], key.color[2], key.color[3]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diamond_raster_contains_transparent_opaque_and_fractional_pixels() {
        let rgba = diamond_pixels(7, [20, 30, 40, 255]);
        let alphas = rgba.chunks_exact(4).map(|pixel| pixel[3]);

        assert!(alphas.clone().any(|alpha| alpha == 0));
        assert!(alphas.clone().any(|alpha| alpha == 255));
        assert!(alphas.any(|alpha| (1..=254).contains(&alpha)));
    }

    #[test]
    fn repeated_diamond_rasters_share_pixel_storage() {
        let key = DiamondRasterKey {
            edge: 7,
            color: [50, 60, 70, 255],
        };
        let first = cached_diamond_raster(key);
        let second = cached_diamond_raster(key);

        assert_eq!(first.resource_key, second.resource_key);
        assert!(Arc::ptr_eq(&first.rgba, &second.rgba));
    }

    #[test]
    fn one_diamond_emits_one_image_command() {
        let mut commands = Vec::new();
        push_aa_diamond(
            &mut commands,
            20.0,
            30.0,
            3,
            [80, 90, 100, 255],
            &FrameRect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            7,
            1.0,
        );

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].frame.x, 16.5);
        assert_eq!(commands[0].frame.y, 26.5);
        assert_eq!(commands[0].frame.width, 7.0);
        assert_eq!(commands[0].frame.height, 7.0);
        assert!(commands[0].image_pixels.is_some());
    }
}

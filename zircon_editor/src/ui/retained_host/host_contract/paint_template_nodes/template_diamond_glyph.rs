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
    source_edge: u32,
    target_edge_bits: u32,
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
    radius: f32,
    color: [u8; 4],
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !radius.is_finite() || radius < 0.0 {
        return;
    }
    let target_edge = radius.mul_add(2.0, 1.0);
    if !target_edge.is_finite() || target_edge <= 0.0 {
        return;
    }
    let source_edge = target_edge.ceil() as u32;
    if source_edge == 0 || source_edge > MAX_DIAMOND_RASTER_EDGE {
        return;
    }

    let raster = cached_diamond_raster(DiamondRasterKey {
        source_edge,
        target_edge_bits: target_edge.to_bits(),
        color,
    });
    let half_edge = target_edge * 0.5;
    commands.push(HostPaintCommand::image_pixels(
        FrameRect {
            x: x - half_edge,
            y: y - half_edge,
            width: target_edge,
            height: target_edge,
        },
        Some(clip.clone()),
        order,
        raster.resource_key,
        source_edge,
        source_edge,
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
            rgba: diamond_pixels_for_target(
                key.source_edge,
                f32::from_bits(key.target_edge_bits),
                key.color,
            )
            .into(),
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
    diamond_pixels_for_target(edge, edge as f32, color)
}

fn diamond_pixels_for_target(source_edge: u32, target_edge: f32, color: [u8; 4]) -> Vec<u8> {
    let edge = source_edge;
    let mut rgba = vec![0; edge as usize * edge as usize * 4];
    for y in 0..edge {
        for x in 0..edge {
            let coverage = diamond_sample_coverage(x, y, source_edge, target_edge);
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

fn diamond_sample_coverage(x: u32, y: u32, source_edge: u32, target_edge: f32) -> u8 {
    let source_to_target = target_edge / source_edge as f32;
    let center = target_edge * 0.5;
    let radius = center;
    let mut covered_samples = 0;
    for sample_y in 0..DIAMOND_SAMPLES_PER_AXIS {
        for sample_x in 0..DIAMOND_SAMPLES_PER_AXIS {
            let px = (x as f32 + (sample_x as f32 + 0.5) / DIAMOND_SAMPLES_PER_AXIS as f32)
                * source_to_target;
            let py = (y as f32 + (sample_y as f32 + 0.5) / DIAMOND_SAMPLES_PER_AXIS as f32)
                * source_to_target;
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
        "icon-raster:analytic-diamond:{}:{:08x}:{:02x}{:02x}{:02x}{:02x}",
        key.source_edge,
        key.target_edge_bits,
        key.color[0],
        key.color[1],
        key.color[2],
        key.color[3]
    )
}

#[cfg(test)]
#[path = "tests/template_diamond_glyph.rs"]
mod tests;

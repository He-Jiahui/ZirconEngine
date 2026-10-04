use std::cell::RefCell;
use std::collections::VecDeque;
use std::f32::consts::PI;
use std::rc::Rc;

use crate::ui::retained_host::host_contract::paint_color::{
    linear_to_srgb_byte, srgb_byte_to_linear,
};

const CIRCULAR_THICKNESS_FACTOR: f32 = 0.16;
const CIRCULAR_THICKNESS_MIN: f32 = 3.0;
const CIRCULAR_THICKNESS_MAX: f32 = 6.0;
const CIRCULAR_PROGRESS_SAMPLES_PER_AXIS: u32 = 4;
const MAX_CACHED_CIRCULAR_TOPOLOGIES: usize = 4;

#[cfg(test)]
#[path = "pixels/tests/topology_front_hit_tests.rs"]
mod topology_front_hit_tests;

thread_local! {
    static CIRCULAR_PROGRESS_TOPOLOGIES: RefCell<VecDeque<Rc<CircularProgressTopology>>> =
        const { RefCell::new(VecDeque::new()) };
}

struct CircularProgressTopology {
    size: u32,
    target_size_bits: u32,
    ring_pixels: Vec<CircularProgressRingPixel>,
}

#[derive(Clone, Copy)]
struct CircularProgressRingPixel {
    offset: usize,
    turn: f32,
    angular_distance_per_turn: f32,
    coverage: u8,
}

pub(super) fn circular_progress_pixels(
    size: u32,
    percent: f32,
    track: [u8; 4],
    fill: [u8; 4],
) -> Vec<u8> {
    circular_progress_pixels_for_target(size, size as f32, percent, track, fill)
}

pub(super) fn circular_progress_pixels_for_target(
    source_size: u32,
    target_size: f32,
    percent: f32,
    track: [u8; 4],
    fill: [u8; 4],
) -> Vec<u8> {
    let mut rgba = vec![0; source_size as usize * source_size as usize * 4];
    let topology = circular_progress_topology_for_target(source_size, target_size);
    let percent = normalized_circular_progress_percent(percent);
    for pixel in &topology.ring_pixels {
        let fill_coverage =
            circular_progress_fill_coverage(percent, pixel.turn, pixel.angular_distance_per_turn);
        let color = mix_srgba_linear_by_coverage(track, fill, fill_coverage);
        rgba[pixel.offset..pixel.offset + 4]
            .copy_from_slice(&scale_alpha_by_coverage(color, pixel.coverage));
    }
    rgba
}

fn circular_progress_fill_coverage(percent: f32, turn: f32, angular_distance_per_turn: f32) -> f32 {
    if percent <= 0.0 {
        return 0.0;
    }
    if percent >= 1.0 {
        return 1.0;
    }

    let signed_start_turn = if turn <= 0.5 { turn } else { turn - 1.0 };
    let start_coverage = (signed_start_turn * angular_distance_per_turn + 0.5).clamp(0.0, 1.0);
    let end_coverage = ((percent - turn) * angular_distance_per_turn + 0.5).clamp(0.0, 1.0);
    start_coverage.min(end_coverage)
}

fn mix_srgba_linear_by_coverage(track: [u8; 4], fill: [u8; 4], coverage: f32) -> [u8; 4] {
    let coverage = coverage.clamp(0.0, 1.0);
    if coverage <= 0.0 {
        return track;
    }
    if coverage >= 1.0 {
        return fill;
    }

    let track_alpha = f32::from(track[3]) / 255.0 * (1.0 - coverage);
    let fill_alpha = f32::from(fill[3]) / 255.0 * coverage;
    let output_alpha = track_alpha + fill_alpha;
    if output_alpha <= f32::EPSILON {
        return [0, 0, 0, 0];
    }

    let mut color = [0, 0, 0, 0];
    for channel in 0..3 {
        let premultiplied_linear = srgb_byte_to_linear(track[channel]) * track_alpha
            + srgb_byte_to_linear(fill[channel]) * fill_alpha;
        color[channel] = linear_to_srgb_byte(premultiplied_linear / output_alpha);
    }
    color[3] = (output_alpha * 255.0).round().clamp(0.0, 255.0) as u8;
    color
}

fn scale_alpha_by_coverage(mut color: [u8; 4], coverage: u8) -> [u8; 4] {
    color[3] = ((u16::from(color[3]) * u16::from(coverage) + 127) / 255) as u8;
    color
}

pub(super) fn normalized_circular_progress_percent(percent: f32) -> f32 {
    if !percent.is_finite() || percent <= 0.0 {
        0.0
    } else if percent >= 1.0 {
        1.0
    } else {
        percent
    }
}

fn circular_progress_topology(size: u32) -> Rc<CircularProgressTopology> {
    circular_progress_topology_for_target(size, size as f32)
}

fn circular_progress_topology_for_target(
    source_size: u32,
    target_size: f32,
) -> Rc<CircularProgressTopology> {
    if let Some(topology) = CIRCULAR_PROGRESS_TOPOLOGIES.with(|cache| {
        let mut cache = cache.borrow_mut();
        cached_circular_progress_topology_for_target(&mut cache, source_size, target_size)
    }) {
        return topology;
    }

    let topology = Rc::new(build_circular_progress_topology(source_size, target_size));
    CIRCULAR_PROGRESS_TOPOLOGIES.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.push_front(Rc::clone(&topology));
        cache.truncate(MAX_CACHED_CIRCULAR_TOPOLOGIES);
    });
    topology
}

fn cached_circular_progress_topology(
    cache: &mut VecDeque<Rc<CircularProgressTopology>>,
    size: u32,
) -> Option<Rc<CircularProgressTopology>> {
    cached_circular_progress_topology_for_target(cache, size, size as f32)
}

fn cached_circular_progress_topology_for_target(
    cache: &mut VecDeque<Rc<CircularProgressTopology>>,
    source_size: u32,
    target_size: f32,
) -> Option<Rc<CircularProgressTopology>> {
    if let Some(topology) = cache.front().filter(|topology| {
        topology.size == source_size && topology.target_size_bits == target_size.to_bits()
    }) {
        return Some(Rc::clone(topology));
    }
    let index = cache.iter().skip(1).position(|topology| {
        topology.size == source_size && topology.target_size_bits == target_size.to_bits()
    })? + 1;
    let topology = cache.remove(index)?;
    cache.push_front(Rc::clone(&topology));
    Some(topology)
}

fn build_circular_progress_topology(
    source_size: u32,
    target_size: f32,
) -> CircularProgressTopology {
    let mut ring_pixels = Vec::new();
    let source_to_target = target_size / source_size as f32;
    let center = target_size * 0.5;
    let radius = (target_size * 0.5 - 0.5).max(1.0);
    let thickness = (target_size * CIRCULAR_THICKNESS_FACTOR)
        .clamp(CIRCULAR_THICKNESS_MIN, CIRCULAR_THICKNESS_MAX);
    let inner = (radius - thickness).max(0.0);
    for y in 0..source_size {
        for x in 0..source_size {
            let coverage = annulus_pixel_coverage(x, y, source_to_target, center, inner, radius);
            if coverage == 0 {
                continue;
            }
            let dx = (x as f32 + 0.5) * source_to_target - center;
            let dy = (y as f32 + 0.5) * source_to_target - center;
            let distance = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx);
            let turn = ((angle + PI * 0.5).rem_euclid(PI * 2.0)) / (PI * 2.0);
            let offset = ((y as usize * source_size as usize) + x as usize) * 4;
            ring_pixels.push(CircularProgressRingPixel {
                offset,
                turn,
                angular_distance_per_turn: PI * 2.0 * distance.max(0.5),
                coverage,
            });
        }
    }
    CircularProgressTopology {
        size: source_size,
        target_size_bits: target_size.to_bits(),
        ring_pixels,
    }
}

fn annulus_pixel_coverage(
    x: u32,
    y: u32,
    source_to_target: f32,
    center: f32,
    inner_radius: f32,
    outer_radius: f32,
) -> u8 {
    let mut covered_samples = 0;
    for sample_y in 0..CIRCULAR_PROGRESS_SAMPLES_PER_AXIS {
        for sample_x in 0..CIRCULAR_PROGRESS_SAMPLES_PER_AXIS {
            let px = (x as f32
                + (sample_x as f32 + 0.5) / CIRCULAR_PROGRESS_SAMPLES_PER_AXIS as f32)
                * source_to_target;
            let py = (y as f32
                + (sample_y as f32 + 0.5) / CIRCULAR_PROGRESS_SAMPLES_PER_AXIS as f32)
                * source_to_target;
            let dx = px - center;
            let dy = py - center;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance >= inner_radius && distance <= outer_radius {
                covered_samples += 1;
            }
        }
    }
    let sample_count = CIRCULAR_PROGRESS_SAMPLES_PER_AXIS * CIRCULAR_PROGRESS_SAMPLES_PER_AXIS;
    ((covered_samples * 255 + sample_count / 2) / sample_count) as u8
}

#[cfg(test)]
#[path = "tests/pixels.rs"]
mod tests;

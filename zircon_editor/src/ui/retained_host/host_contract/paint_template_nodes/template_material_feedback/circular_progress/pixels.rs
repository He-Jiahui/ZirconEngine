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
const MAX_CACHED_CIRCULAR_TOPOLOGIES: usize = 4;

#[cfg(test)]
mod topology_front_hit_tests;

thread_local! {
    static CIRCULAR_PROGRESS_TOPOLOGIES: RefCell<VecDeque<Rc<CircularProgressTopology>>> =
        const { RefCell::new(VecDeque::new()) };
}

struct CircularProgressTopology {
    size: u32,
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
    let mut rgba = vec![0; size as usize * size as usize * 4];
    let topology = circular_progress_topology(size);
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
    if let Some(topology) = CIRCULAR_PROGRESS_TOPOLOGIES.with(|cache| {
        let mut cache = cache.borrow_mut();
        cached_circular_progress_topology(&mut cache, size)
    }) {
        return topology;
    }

    let topology = Rc::new(build_circular_progress_topology(size));
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
    if let Some(topology) = cache.front().filter(|topology| topology.size == size) {
        return Some(Rc::clone(topology));
    }
    let index = cache
        .iter()
        .skip(1)
        .position(|topology| topology.size == size)?
        + 1;
    let topology = cache.remove(index)?;
    cache.push_front(Rc::clone(&topology));
    Some(topology)
}

fn build_circular_progress_topology(size: u32) -> CircularProgressTopology {
    let mut ring_pixels = Vec::new();
    let center = size as f32 * 0.5;
    let radius = (size as f32 * 0.5 - 0.5).max(1.0);
    let thickness = (size as f32 * CIRCULAR_THICKNESS_FACTOR)
        .clamp(CIRCULAR_THICKNESS_MIN, CIRCULAR_THICKNESS_MAX);
    let inner = (radius - thickness).max(0.0);
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let distance = (dx * dx + dy * dy).sqrt();
            let coverage = (annulus_pixel_coverage(distance, inner, radius) * 255.0).round() as u8;
            if coverage == 0 {
                continue;
            }
            let angle = dy.atan2(dx);
            let turn = ((angle + PI * 0.5).rem_euclid(PI * 2.0)) / (PI * 2.0);
            let offset = ((y as usize * size as usize) + x as usize) * 4;
            ring_pixels.push(CircularProgressRingPixel {
                offset,
                turn,
                angular_distance_per_turn: PI * 2.0 * distance.max(0.5),
                coverage,
            });
        }
    }
    CircularProgressTopology { size, ring_pixels }
}

fn annulus_pixel_coverage(distance: f32, inner_radius: f32, outer_radius: f32) -> f32 {
    let outer_coverage = (outer_radius + 0.5 - distance).clamp(0.0, 1.0);
    let inner_coverage = (distance - inner_radius + 0.5).clamp(0.0, 1.0);
    outer_coverage.min(inner_coverage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circular_progress_topology_is_reused_for_a_stable_size() {
        let first = circular_progress_topology(31);
        let second = circular_progress_topology(31);

        assert!(std::rc::Rc::ptr_eq(&first, &second));
    }

    #[test]
    fn invalid_progress_matches_empty_determinate_progress() {
        let track = [10, 20, 30, 255];
        let fill = [200, 100, 50, 255];

        assert_eq!(
            circular_progress_pixels(24, f32::NAN, track, fill),
            circular_progress_pixels(24, 0.0, track, fill)
        );
    }

    #[test]
    fn circular_progress_silhouette_contains_fractional_edge_coverage() {
        let pixels = circular_progress_pixels(24, 0.5, [20, 30, 40, 255], [80, 90, 100, 255]);
        let alphas = pixels.chunks_exact(4).map(|pixel| pixel[3]);

        assert!(alphas.clone().any(|alpha| alpha == 0));
        assert!(alphas.clone().any(|alpha| alpha == 255));
        assert!(
            alphas.into_iter().any(|alpha| (1..=254).contains(&alpha)),
            "the final-size circular progress raster must keep analytic edge coverage"
        );
    }

    #[test]
    fn circular_progress_endpoint_contains_linear_color_coverage() {
        let pixels = circular_progress_pixels(24, 0.375, [0, 0, 0, 255], [255, 255, 255, 255]);

        assert!(
            pixels.chunks_exact(4).any(|pixel| {
                pixel[3] == 255 && pixel[0] > 0 && pixel[0] < 255 && pixel[0] == pixel[1]
            }),
            "a non-axis-aligned progress endpoint must not remain a binary color staircase"
        );
    }

    #[test]
    fn circular_progress_endpoint_mix_resolves_in_linear_light() {
        let mixed = mix_srgba_linear_by_coverage([0, 0, 0, 255], [255, 255, 255, 255], 0.5);

        assert!((187..=189).contains(&mixed[0]));
        assert_eq!(mixed[0], mixed[1]);
        assert_eq!(mixed[1], mixed[2]);
        assert_eq!(mixed[3], 255);
    }
}

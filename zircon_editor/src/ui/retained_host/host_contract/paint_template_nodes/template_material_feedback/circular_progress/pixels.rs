use std::cell::RefCell;
use std::collections::VecDeque;
use std::f32::consts::PI;
use std::rc::Rc;

const CIRCULAR_THICKNESS_FACTOR: f32 = 0.16;
const CIRCULAR_THICKNESS_MIN: f32 = 3.0;
const CIRCULAR_THICKNESS_MAX: f32 = 6.0;
const MAX_CACHED_CIRCULAR_TOPOLOGIES: usize = 4;

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
        let color = if pixel.turn <= percent { fill } else { track };
        rgba[pixel.offset..pixel.offset + 4].copy_from_slice(&color);
    }
    rgba
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
        let index = cache.iter().position(|topology| topology.size == size)?;
        let topology = cache.remove(index)?;
        cache.push_front(Rc::clone(&topology));
        Some(topology)
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

fn build_circular_progress_topology(size: u32) -> CircularProgressTopology {
    let mut ring_pixels = Vec::new();
    let center = size as f32 * 0.5;
    let radius = (size as f32 * 0.5 - 0.5).max(1.0);
    let thickness = (size as f32 * CIRCULAR_THICKNESS_FACTOR)
        .clamp(CIRCULAR_THICKNESS_MIN, CIRCULAR_THICKNESS_MAX);
    let inner = (radius - thickness).max(0.0);
    let inner_squared = inner * inner;
    let radius_squared = radius * radius;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let distance_squared = dx * dx + dy * dy;
            if distance_squared < inner_squared || distance_squared > radius_squared {
                continue;
            }
            let angle = dy.atan2(dx);
            let turn = ((angle + PI * 0.5).rem_euclid(PI * 2.0)) / (PI * 2.0);
            let offset = ((y as usize * size as usize) + x as usize) * 4;
            ring_pixels.push(CircularProgressRingPixel { offset, turn });
        }
    }
    CircularProgressTopology { size, ring_pixels }
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
}

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
fn fractional_target_uses_ceil_source_without_changing_ring_geometry() {
    let pixels =
        circular_progress_pixels_for_target(32, 31.25, 0.5, [20, 30, 40, 255], [80, 90, 100, 255]);
    assert_eq!(pixels.len(), 32 * 32 * 4);
    let topology = circular_progress_topology_for_target(32, 31.25);
    assert_eq!(topology.size, 32);
    assert_eq!(topology.target_size_bits, 31.25_f32.to_bits());
    assert!(pixels
        .chunks_exact(4)
        .any(|pixel| (1..=254).contains(&pixel[3])));
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

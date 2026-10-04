use std::f32::consts::TAU;

use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::compiler::state_machine::AnimationCompiledBlendSpace2DSample;

use super::*;

#[test]
fn stable_delaunay_prepares_complete_cocircular_hull() {
    const SAMPLE_COUNT: usize = 96;
    let samples = (0..SAMPLE_COUNT)
        .map(|index| {
            let angle = TAU * index as f32 / SAMPLE_COUNT as f32;
            sample([angle.cos(), angle.sin()], index)
        })
        .collect::<Vec<_>>();

    let blend = BlendSpace2D::from_compiled(&samples).unwrap();

    assert_eq!(blend.triangle_count(), SAMPLE_COUNT - 2);
    assert_eq!(blend.hull_edges.len(), SAMPLE_COUNT);
    assert_eq!(
        blend
            .neighbors
            .iter()
            .flat_map(|neighbors| neighbors.iter())
            .filter(|neighbor| neighbor.is_some())
            .count(),
        2 * (SAMPLE_COUNT - 3)
    );
    let weights = blend.sample(Vec2::ZERO).unwrap();
    assert!((weights.weight_sum() - 1.0).abs() <= 1.0e-5);
}

#[test]
fn prepared_hull_sampling_does_not_rebuild_an_edge_map() {
    let source = include_str!("../blend_space_2d.rs");

    assert!(source.contains("hull_edges: Box<[[usize; 2]]>"));
    assert!(!source.contains(
        "fn sample_hull(&self, point: Vec2) -> Option<BlendSpaceWeights3> {\n        let mut edges"
    ));
}

#[test]
fn outside_hull_sampling_retains_boundary_triangle_hint() {
    let samples = [
        sample([-1.0, -1.0], 0),
        sample([1.0, -1.0], 1),
        sample([1.0, 1.0], 2),
        sample([-1.0, 1.0], 3),
    ];
    let blend = BlendSpace2D::from_compiled(&samples).unwrap();

    let (weights, hint) = blend.sample_with_hint(Vec2::new(2.0, 0.25), None).unwrap();
    let hint = hint.expect("outside-hull sampling retains the boundary triangle");
    let (_, repeated_hint) = blend
        .sample_with_hint(Vec2::new(2.0, 0.25), Some(hint))
        .unwrap();

    assert!((weights.weight_sum() - 1.0).abs() <= 1.0e-5);
    assert_eq!(repeated_hint, Some(hint));
}

#[test]
fn retained_walk_matches_exhaustive_sampling_inside_and_outside_the_hull() {
    let samples = [
        sample([-1.0, -1.0], 0),
        sample([0.0, -0.8], 1),
        sample([1.0, -1.0], 2),
        sample([-0.9, 0.1], 3),
        sample([0.1, 0.0], 4),
        sample([0.8, 0.3], 5),
        sample([-0.7, 1.0], 6),
        sample([0.2, 0.9], 7),
        sample([1.0, 1.0], 8),
    ];
    let blend = BlendSpace2D::from_compiled(&samples).unwrap();
    let mut hint = None;
    for y in -16..=16 {
        for x in -16..=16 {
            let point = Vec2::new(x as Real * 0.1, y as Real * 0.1);
            let (walked, next_hint) = blend.sample_with_hint(point, hint).unwrap();
            let (exhaustive, _) = blend.sample_after_failed_walk(point).unwrap();

            assert_weight_maps_close(walked, exhaustive, samples.len(), point);
            hint = next_hint;
        }
    }
}

fn assert_weight_maps_close(
    actual: BlendSpaceWeights3,
    expected: BlendSpaceWeights3,
    sample_count: usize,
    point: Vec2,
) {
    let mut actual_weights = vec![0.0; sample_count];
    let mut expected_weights = vec![0.0; sample_count];
    for (sample, weight) in actual.as_pairs() {
        actual_weights[sample as usize] += weight;
    }
    for (sample, weight) in expected.as_pairs() {
        expected_weights[sample as usize] += weight;
    }
    for (actual, expected) in actual_weights.into_iter().zip(expected_weights) {
        assert!(
            (actual - expected).abs() <= 1.0e-5,
            "walked and exhaustive weights differ at {point:?}: {actual} != {expected}"
        );
    }
}

fn sample(position: [f32; 2], index: usize) -> AnimationCompiledBlendSpace2DSample {
    AnimationCompiledBlendSpace2DSample {
        position: Vec2::from_array(position),
        graph: AssetReference::from_locator(
            AssetUri::parse(&format!("res://animation/direction-{index}.zranim")).unwrap(),
        ),
    }
}

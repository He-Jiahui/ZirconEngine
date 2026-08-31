use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::ai::{
    AiAgentTickReport, AiBehaviorDebugFrame, AiBehaviorDebugSnapshot, AiDecisionStatus,
    AiPerceptionDebugSnapshot, AiPerceptionSense, AiPerceptionSnapshot, AiPerceptionStimulus,
};
use zircon_runtime::core::framework::render::{SceneGizmoKind, SceneGizmoOverlayExtract};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};
use zircon_runtime::core::math::Vec3;

use super::{
    append_circle, append_sight_cone, append_stimulus, build_ai_perception_overlay_with_options,
    overlay_capacity, valid_radius, AiPerceptionOverlayOptions, HEARING_COLOR,
};
use crate::runtime_mirror::{AiPieMirror, AiPieMirrorApply};

const LINE_AGENT_COUNT: usize = 1_024;
const STIMULUS_AGENT_COUNT: usize = 256;
const STIMULI_PER_AGENT: usize = 32;
const BENCHMARK_ITERATIONS: usize = 8;
const BENCHMARK_SAMPLES: usize = 21;

#[test]
fn overlay_capacity_matches_default_overlay_output_lengths() {
    let world = WorldHandle::new(3);
    let mirror = mirror_fixture(4, 3);
    let options = AiPerceptionOverlayOptions::default();

    let capacity = overlay_capacity(&world, &mirror, options);
    let overlay = build_ai_perception_overlay_with_options(101, &world, &mirror, options);

    assert_eq!(overlay.lines.len(), capacity.lines);
    assert_eq!(overlay.pick_shapes.len(), capacity.pick_shapes);
    assert!(overlay.lines.capacity() >= capacity.lines);
    assert!(overlay.pick_shapes.capacity() >= capacity.pick_shapes);
}

#[test]
fn overlay_build_preallocates_line_and_pick_shape_buffers() {
    let source = include_str!("../overlay.rs");
    let build = source
        .split("fn build_ai_perception_overlay_with_options(")
        .nth(1)
        .and_then(|body| body.split("\n}").next())
        .expect("overlay build body");

    assert!(build.contains("overlay_capacity(world, mirror, options)"));
    assert!(build.contains("Vec::with_capacity(capacity.lines)"));
    assert!(build.contains("Vec::with_capacity(capacity.pick_shapes)"));
    assert!(build.contains(
        "false,\n        Vec::with_capacity(capacity.lines),\n        Vec::new(),\n        Vec::new(),\n        Vec::with_capacity(capacity.pick_shapes),"
    ));
}

#[test]
#[ignore = "release-only performance evidence"]
fn preallocated_overlay_lines_release_benchmark_evidence() {
    let world = WorldHandle::new(3);
    let mirror = mirror_fixture(LINE_AGENT_COUNT, 0);
    let options = AiPerceptionOverlayOptions {
        sight_cone: true,
        hearing_radius: true,
        stimuli: false,
    };
    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || benchmark_overlay_builds(&world, &mirror, options, true),
        || benchmark_overlay_builds(&world, &mirror, options, false),
    );
    print_performance_result(
        "plugins15_preallocated_editor_overlay_lines",
        &legacy_samples,
        &optimized_samples,
        format!(
            "agents={LINE_AGENT_COUNT} stimuli_per_agent=0 iterations_per_sample={BENCHMARK_ITERATIONS} legacy_initial_line_capacity=0 optimized_exact_line_capacity=1"
        ),
        19,
        20,
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn preallocated_overlay_pick_shapes_release_benchmark_evidence() {
    let world = WorldHandle::new(3);
    let mirror = mirror_fixture(STIMULUS_AGENT_COUNT, STIMULI_PER_AGENT);
    let options = AiPerceptionOverlayOptions {
        sight_cone: false,
        hearing_radius: false,
        stimuli: true,
    };
    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || benchmark_overlay_builds(&world, &mirror, options, true),
        || benchmark_overlay_builds(&world, &mirror, options, false),
    );
    print_performance_result(
        "plugins15_preallocated_editor_overlay_pick_shapes",
        &legacy_samples,
        &optimized_samples,
        format!(
            "agents={STIMULUS_AGENT_COUNT} stimuli_per_agent={STIMULI_PER_AGENT} iterations_per_sample={BENCHMARK_ITERATIONS} legacy_initial_pick_shape_capacity=0 optimized_exact_pick_shape_capacity=1"
        ),
        19,
        20,
    );
}

fn benchmark_overlay_builds(
    world: &WorldHandle,
    mirror: &AiPieMirror,
    options: AiPerceptionOverlayOptions,
    legacy: bool,
) -> u64 {
    let mut checksum = 0_u64;
    for _ in 0..BENCHMARK_ITERATIONS {
        let overlay = if legacy {
            legacy_build_overlay(101, world, mirror, options)
        } else {
            build_ai_perception_overlay_with_options(101, world, mirror, options)
        };
        checksum += black_box(overlay.lines.len() + overlay.pick_shapes.len()) as u64;
    }
    checksum
}

fn legacy_build_overlay(
    owner: EntityId,
    world: &WorldHandle,
    mirror: &AiPieMirror,
    options: AiPerceptionOverlayOptions,
) -> SceneGizmoOverlayExtract {
    let mut overlay = SceneGizmoOverlayExtract::new(
        owner,
        SceneGizmoKind::AiPerception,
        false,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    for frame in mirror.agents_in_world(world) {
        let Some(debug) = frame.perception_debug.as_ref() else {
            continue;
        };
        if !debug.position.is_finite() {
            continue;
        }
        overlay.pick_shapes.push(
            zircon_runtime::core::framework::render::OverlayPickShape::Sphere {
                center: debug.position,
                radius: 0.18,
            },
        );
        if options.sight_cone {
            append_sight_cone(
                &mut overlay.lines,
                debug.position,
                debug.forward,
                debug.sight_fov_degrees,
                debug.sight_range,
            );
        }
        if options.hearing_radius {
            append_circle(
                &mut overlay.lines,
                debug.position,
                debug.hearing_radius,
                HEARING_COLOR,
            );
            if valid_radius(debug.hearing_radius) {
                overlay.pick_shapes.push(
                    zircon_runtime::core::framework::render::OverlayPickShape::Circle {
                        center: debug.position,
                        normal: Vec3::Y,
                        radius: debug.hearing_radius,
                        thickness: 0.08,
                    },
                );
            }
        }
        if options.stimuli {
            for stimulus in frame
                .perception
                .as_ref()
                .into_iter()
                .flat_map(|perception| perception.stimuli.iter())
            {
                append_stimulus(&mut overlay, debug.position, stimulus);
            }
        }
    }
    overlay
}

fn mirror_fixture(agent_count: usize, stimuli_per_agent: usize) -> AiPieMirror {
    let mut mirror = AiPieMirror::default();
    mirror.begin_session(9);
    let frames = (0..agent_count)
        .map(|agent| {
            let entity = agent as u64 + 1;
            AiBehaviorDebugFrame {
                report: AiAgentTickReport {
                    world: WorldHandle::new(3),
                    entity,
                    status: AiDecisionStatus::Running,
                    active_node: Some("scan".to_owned()),
                    diagnostic: None,
                },
                behavior_tree: Some("guard".to_owned()),
                blackboard: Vec::new(),
                perception: Some(AiPerceptionSnapshot {
                    agent: entity,
                    stimuli: (0..stimuli_per_agent)
                        .map(|stimulus| AiPerceptionStimulus {
                            source: stimulus as u64 + 10_000,
                            sense: AiPerceptionSense::Hearing,
                            position: Vec3::new(stimulus as f32 + 1.0, 0.0, agent as f32 + 1.0),
                            strength: 1.0,
                            age_seconds: 0.0,
                        })
                        .collect(),
                }),
                perception_debug: Some(AiPerceptionDebugSnapshot {
                    position: Vec3::new(agent as f32 + 1.0, 0.0, 2.0),
                    forward: Vec3::Z,
                    sight_fov_degrees: 90.0,
                    sight_range: 6.0,
                    hearing_radius: 4.0,
                }),
            }
        })
        .collect();
    assert_eq!(
        mirror.apply_debug_snapshot(
            9,
            1,
            AiBehaviorDebugSnapshot {
                world: WorldHandle::new(3),
                frames,
            },
        ),
        AiPieMirrorApply::Applied
    );
    mirror
}

fn benchmark_paired_samples(
    mut legacy: impl FnMut() -> u64,
    mut optimized: impl FnMut() -> u64,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    for sample_index in 0..BENCHMARK_SAMPLES {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample(operation: &mut impl FnMut() -> u64) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn print_performance_result(
    name: &str,
    legacy_samples: &[u128],
    optimized_samples: &[u128],
    dimensions: String,
    maximum_numerator: u128,
    maximum_denominator: u128,
) {
    let legacy_p50 = percentile(legacy_samples, 50);
    let legacy_p95 = percentile(legacy_samples, 95);
    let optimized_p50 = percentile(optimized_samples, 50);
    let optimized_p95 = percentile(optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(legacy_samples);
    let optimized_ns = benchmark_samples_csv(optimized_samples);
    println!(
        "PERF_RESULT {name} {dimensions} samples={BENCHMARK_SAMPLES} sample_pairs={BENCHMARK_SAMPLES} sample_order=alternating percentile_method=nearest_rank legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * maximum_denominator <= legacy_p95 * maximum_numerator,
        "optimized P95 {optimized_p95}ns must be no more than {maximum_numerator}/{maximum_denominator} of legacy P95 {legacy_p95}ns"
    );
}

fn benchmark_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile));
    let index = (sorted.len() * percentile).div_ceil(100) - 1;
    sorted[index]
}

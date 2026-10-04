use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::core::framework::animation::avatar_mask::animation_target_id_matches;

const SAMPLE_COUNT: usize = 17;
const CHECKS_PER_SAMPLE: usize = 1_200;

#[test]
fn optimization_batch_20260830cc_timeline_rejects_direct_target_before_scanning_avatar_mask() {
    let track = AnimationTimelineTrackDescriptor::bone_transform("Rig/Spine/Chest", 1)
        .with_avatar_mask(large_mask(32));

    assert!(!track.allows_target("Rig/Hands/Left"));

    let source = include_str!("../timeline.rs");
    let allows_target = source
        .split("pub fn allows_target")
        .nth(1)
        .and_then(|source| source.split("pub fn muted").next())
        .expect("read timeline target filtering");
    let direct_rejection = allows_target
        .find("if !target.matches")
        .expect("a direct target mismatch must return immediately");
    let mask_scan = allows_target
        .find("allows_prepared_target")
        .expect("matching tracks must retain avatar-mask filtering");

    assert!(
        direct_rejection < mask_scan,
        "direct target rejection must happen before avatar-mask evaluation"
    );
}

#[test]
#[ignore = "Windows Release performance qualification"]
fn optimization_batch_20260830cc_animation_target_rejection_p95() {
    let mask = large_mask(256);
    let track = AnimationTimelineTrackDescriptor::bone_transform(
        "Character/Hero/Rig/Spine/Direct_Other",
        1,
    )
    .with_avatar_mask(mask.clone());
    let target = "Character/Hero/Rig/Hands/Missing_Target";
    let mut avatar_baseline = Vec::with_capacity(SAMPLE_COUNT);
    let mut avatar_optimized = Vec::with_capacity(SAMPLE_COUNT);
    let mut timeline_baseline = Vec::with_capacity(SAMPLE_COUNT);
    let mut timeline_optimized = Vec::with_capacity(SAMPLE_COUNT);
    let mut sink = 0usize;

    for sample_index in 0..SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            sink += sample(&mut avatar_baseline, || legacy_mask_allows(&mask, target));
            sink += sample(&mut avatar_optimized, || mask.allows_target(target));
            sink += sample(&mut timeline_baseline, || {
                legacy_timeline_allows(&track, target)
            });
            sink += sample(&mut timeline_optimized, || track.allows_target(target));
        } else {
            sink += sample(&mut avatar_optimized, || mask.allows_target(target));
            sink += sample(&mut avatar_baseline, || legacy_mask_allows(&mask, target));
            sink += sample(&mut timeline_optimized, || track.allows_target(target));
            sink += sample(&mut timeline_baseline, || {
                legacy_timeline_allows(&track, target)
            });
        }
    }

    let avatar_baseline_p50 = percentile(&avatar_baseline, 50);
    let avatar_baseline_p95 = percentile(&avatar_baseline, 95);
    let avatar_optimized_p50 = percentile(&avatar_optimized, 50);
    let avatar_optimized_p95 = percentile(&avatar_optimized, 95);
    let timeline_baseline_p50 = percentile(&timeline_baseline, 50);
    let timeline_baseline_p95 = percentile(&timeline_baseline, 95);
    let timeline_optimized_p50 = percentile(&timeline_optimized, 50);
    let timeline_optimized_p95 = percentile(&timeline_optimized, 95);

    println!(
        "RUNTIME170_AVATAR_MASK_REJECTION_BENCH_V1 baseline_p50_ns={avatar_baseline_p50} optimized_p50_ns={avatar_optimized_p50} baseline_p95_ns={avatar_baseline_p95} optimized_p95_ns={avatar_optimized_p95}"
    );
    println!(
        "RUNTIME170_TIMELINE_DIRECT_REJECTION_BENCH_V1 baseline_p50_ns={timeline_baseline_p50} optimized_p50_ns={timeline_optimized_p50} baseline_p95_ns={timeline_baseline_p95} optimized_p95_ns={timeline_optimized_p95}"
    );

    assert!(
        avatar_optimized_p95 * 100 <= avatar_baseline_p95 * 70,
        "avatar-mask optimized P95 must be at most 70% of baseline"
    );
    assert!(
        timeline_optimized_p95 * 100 <= timeline_baseline_p95 * 10,
        "timeline optimized P95 must be at most 10% of baseline"
    );
    black_box(sink);
}

fn large_mask(entry_count: usize) -> AnimationAvatarMask {
    AnimationAvatarMask {
        id: "benchmark".to_string(),
        included_target_ids: (0..entry_count)
            .map(|index| format!("Rig/Spine/Bone_{index:03}"))
            .collect(),
        excluded_target_ids: (0..entry_count)
            .map(|index| format!("Rig/Face/Bone_{index:03}"))
            .collect(),
        weight: 1.0,
    }
}

fn legacy_mask_allows(mask: &AnimationAvatarMask, target_id: &str) -> bool {
    let target_id = target_id.trim();
    if target_id.is_empty() {
        return false;
    }
    let included = mask.included_target_ids.is_empty()
        || mask
            .included_target_ids
            .iter()
            .any(|candidate| animation_target_id_matches(candidate, target_id));
    let excluded = mask
        .excluded_target_ids
        .iter()
        .any(|candidate| animation_target_id_matches(candidate, target_id));
    included && !excluded
}

fn legacy_timeline_allows(track: &AnimationTimelineTrackDescriptor, target_id: &str) -> bool {
    let target_id = target_id.trim();
    if target_id.is_empty() || track.muted {
        return false;
    }
    let direct_match = track
        .target_id
        .as_deref()
        .map(|candidate| animation_target_id_matches(candidate, target_id))
        .unwrap_or(true);
    let mask_match = track
        .avatar_mask
        .as_ref()
        .map(|mask| legacy_mask_allows(mask, target_id))
        .unwrap_or(true);
    direct_match && mask_match
}

fn sample(samples: &mut Vec<u128>, mut operation: impl FnMut() -> bool) -> usize {
    let started = Instant::now();
    let mut hits = 0usize;
    for _ in 0..CHECKS_PER_SAMPLE {
        hits += black_box(operation()) as usize;
    }
    samples.push(started.elapsed().as_nanos());
    hits
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() - 1) * percentile / 100]
}

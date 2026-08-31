use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::animation::{
    AnimationChannelAsset, AnimationClipBoneTrackAsset, AnimationInterpolationAsset,
    AnimationSkeletonAsset, AnimationSkeletonBoneAsset,
};

use super::{skeleton_bone_path, ClipTrackBoneIndex};

const BENCH_BONE_COUNT: usize = 128;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn clip_track_bone_index_preserves_target_and_first_duplicate_resolution() {
    let skeleton = AnimationSkeletonAsset {
        name: Some("duplicate-leaves".to_string()),
        bones: vec![
            bone("Root", None),
            bone("Hand", Some(0)),
            bone("Branch", Some(0)),
            bone("Hand", Some(2)),
        ],
    };
    let tracks = vec![
        track("Hand", None),
        track("ignored", Some("Root/Branch/Hand")),
        track("Hand", Some("  ")),
    ];

    let index = ClipTrackBoneIndex::new(&skeleton, &tracks);

    assert_eq!(index.resolve(&tracks[0]), Some(1));
    assert_eq!(index.resolve(&tracks[1]), Some(3));
    assert_eq!(index.resolve(&tracks[2]), Some(1));
}

#[test]
#[ignore = "release-only clip track bone index benchmark"]
fn clip_track_bone_index_release_benchmark_evidence() {
    let (skeleton, tracks) = benchmark_fixture();
    assert_eq!(
        legacy_checksum(&skeleton, &tracks),
        indexed_checksum(&skeleton, &tracks)
    );

    let (legacy_samples, indexed_samples) = paired_samples(
        || measure_legacy(&skeleton, &tracks),
        || measure_indexed(&skeleton, &tracks),
    );
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let indexed_p50_ns = percentile(&indexed_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let indexed_p95_ns = percentile(&indexed_samples, 95);

    println!(
        "PERF_RESULT plan=Runtime08C task=clip_track_bone_index \
sample_pairs={SAMPLE_PAIRS} bone_count={BENCH_BONE_COUNT} track_count={BENCH_BONE_COUNT} \
legacy_lookup=track_times_bone_path_scan optimized_lookup=single_borrowed_name_and_path_index \
pair_order=alternating_legacy_even legacy_first_pairs=11 indexed_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} indexed_p50_ns={indexed_p50_ns} \
legacy_p95_ns={legacy_p95_ns} indexed_p95_ns={indexed_p95_ns} \
legacy_raw_ns={} indexed_raw_ns={}",
        raw(&legacy_samples),
        raw(&indexed_samples),
    );

    assert!(
        indexed_p95_ns.saturating_mul(5) <= legacy_p95_ns,
        "clip track index must reduce P95 by at least 80%: \
legacy={legacy_p95_ns}ns indexed={indexed_p95_ns}ns"
    );
}

fn benchmark_fixture() -> (AnimationSkeletonAsset, Vec<AnimationClipBoneTrackAsset>) {
    let mut bones = Vec::with_capacity(BENCH_BONE_COUNT);
    let mut paths = Vec::with_capacity(BENCH_BONE_COUNT);
    let mut path = String::new();
    for index in 0..BENCH_BONE_COUNT {
        let name = format!("Bone{index:03}");
        if !path.is_empty() {
            path.push('/');
        }
        path.push_str(&name);
        paths.push(path.clone());
        bones.push(bone(
            &name,
            index.checked_sub(1).map(|parent| parent as u32),
        ));
    }
    let tracks = paths
        .into_iter()
        .enumerate()
        .rev()
        .map(|(index, path)| track(&format!("fallback-{index}"), Some(&path)))
        .collect();
    (
        AnimationSkeletonAsset {
            name: Some("benchmark-chain".to_string()),
            bones,
        },
        tracks,
    )
}

fn bone(name: &str, parent_index: Option<u32>) -> AnimationSkeletonBoneAsset {
    AnimationSkeletonBoneAsset {
        name: name.to_string(),
        parent_index,
        local_translation: [0.0; 3],
        local_rotation: [0.0, 0.0, 0.0, 1.0],
        local_scale: [1.0; 3],
    }
}

fn track(bone_name: &str, target_id: Option<&str>) -> AnimationClipBoneTrackAsset {
    let empty_channel = || AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Linear,
        keys: Vec::new(),
    };
    AnimationClipBoneTrackAsset {
        bone_name: bone_name.to_string(),
        target_id: target_id.map(str::to_string),
        translation: empty_channel(),
        rotation: empty_channel(),
        scale: empty_channel(),
    }
}

fn legacy_checksum(
    skeleton: &AnimationSkeletonAsset,
    tracks: &[AnimationClipBoneTrackAsset],
) -> usize {
    tracks
        .iter()
        .filter_map(|track| legacy_resolve(skeleton, track))
        .sum()
}

fn legacy_resolve(
    skeleton: &AnimationSkeletonAsset,
    track: &AnimationClipBoneTrackAsset,
) -> Option<usize> {
    if let Some(target_id) = track
        .target_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    {
        if let Some(index) = skeleton
            .bones
            .iter()
            .position(|bone| bone.name == target_id)
        {
            return Some(index);
        }
        if let Some(index) = skeleton.bones.iter().enumerate().find_map(|(index, _)| {
            (skeleton_bone_path(skeleton, index)? == target_id).then_some(index)
        }) {
            return Some(index);
        }
    }
    skeleton
        .bones
        .iter()
        .position(|bone| bone.name == track.bone_name)
}

fn indexed_checksum(
    skeleton: &AnimationSkeletonAsset,
    tracks: &[AnimationClipBoneTrackAsset],
) -> usize {
    let index = ClipTrackBoneIndex::new(skeleton, tracks);
    tracks.iter().filter_map(|track| index.resolve(track)).sum()
}

fn paired_samples(
    mut measure_legacy: impl FnMut() -> u128,
    mut measure_indexed: impl FnMut() -> u128,
) -> (Vec<u128>, Vec<u128>) {
    for _ in 0..4 {
        black_box(measure_legacy());
        black_box(measure_indexed());
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut indexed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            indexed_samples.push(measure_indexed());
        } else {
            indexed_samples.push(measure_indexed());
            legacy_samples.push(measure_legacy());
        }
    }
    (legacy_samples, indexed_samples)
}

fn measure_legacy(
    skeleton: &AnimationSkeletonAsset,
    tracks: &[AnimationClipBoneTrackAsset],
) -> u128 {
    let started = Instant::now();
    black_box(legacy_checksum(black_box(skeleton), black_box(tracks)));
    started.elapsed().as_nanos().max(1)
}

fn measure_indexed(
    skeleton: &AnimationSkeletonAsset,
    tracks: &[AnimationClipBoneTrackAsset],
) -> u128 {
    let started = Instant::now();
    black_box(indexed_checksum(black_box(skeleton), black_box(tracks)));
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

use std::collections::HashMap;

use zircon_runtime::core::framework::animation::{
    AnimationClipAsset, AnimationClipBoneTrackAsset, AnimationSkeletonAsset,
    AnimationSkeletonBoneAsset,
};
use zircon_runtime::core::framework::animation::{
    AnimationError, AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource, AnimationResult,
};
use zircon_runtime::core::math::{Quat, Real, Transform, Vec3};

use super::sampling::{
    quaternion_array_is_normalizable, real_array_is_finite, resolve_sample_time, sample_quaternion,
    sample_vec3,
};
use crate::channel_sampling::AnimationChannelSampleExt;

const BONE_INDEX_MIN_PROJECTED_COMPARISONS: usize = 2_048;

pub(super) fn sample_clip_pose(
    skeleton: &AnimationSkeletonAsset,
    clip: &AnimationClipAsset,
    time_seconds: Real,
    looping: bool,
) -> AnimationResult<AnimationPoseOutput> {
    let sample_time = resolve_sample_time(clip.duration_seconds, time_seconds, looping);
    let mut bones = skeleton
        .bones
        .iter()
        .map(animation_pose_bone_from_skeleton)
        .collect::<AnimationResult<Vec<_>>>()?;
    let track_bone_lookup = ClipTrackBoneLookup::new(skeleton, &clip.tracks);

    for track in &clip.tracks {
        let Some(bone_index) = track_bone_lookup.resolve(track) else {
            continue;
        };
        let Some(bone) = bones.get_mut(bone_index) else {
            continue;
        };
        if let Some(sample) = track.translation.sample(sample_time) {
            bone.local_transform.translation = sample_vec3(&sample)?;
        }
        if let Some(sample) = track.rotation.sample(sample_time) {
            bone.local_transform.rotation = sample_quaternion(&sample)?;
        }
        if let Some(sample) = track.scale.sample(sample_time) {
            bone.local_transform.scale = sample_vec3(&sample)?;
        }
    }

    Ok(AnimationPoseOutput {
        source: AnimationPoseSource::Clip,
        active_state: None,
        bones,
    })
}

fn should_index_bone_lookup(bone_count: usize, track_count: usize) -> bool {
    bone_count.saturating_mul(track_count) >= BONE_INDEX_MIN_PROJECTED_COMPARISONS
}

struct ClipTrackBoneLookup<'skeleton> {
    skeleton: &'skeleton AnimationSkeletonAsset,
    bone_names: Option<HashMap<&'skeleton str, usize>>,
    bone_paths: HashMap<String, usize>,
}

impl<'skeleton> ClipTrackBoneLookup<'skeleton> {
    fn new(
        skeleton: &'skeleton AnimationSkeletonAsset,
        tracks: &[AnimationClipBoneTrackAsset],
    ) -> Self {
        if !should_index_bone_lookup(skeleton.bones.len(), tracks.len()) {
            return Self {
                skeleton,
                bone_names: None,
                bone_paths: HashMap::new(),
            };
        }

        let mut bone_names = HashMap::with_capacity(skeleton.bones.len());
        for (index, bone) in skeleton.bones.iter().enumerate() {
            bone_names.entry(bone.name.as_str()).or_insert(index);
        }
        let needs_path_index = tracks.iter().any(|track| {
            track
                .target_id
                .as_deref()
                .map(str::trim)
                .filter(|target_id| !target_id.is_empty())
                .is_some_and(|target_id| !bone_names.contains_key(target_id))
        });
        let mut bone_paths = HashMap::with_capacity(if needs_path_index {
            skeleton.bones.len()
        } else {
            0
        });
        if needs_path_index {
            for index in 0..skeleton.bones.len() {
                if let Some(path) = skeleton_bone_path(skeleton, index) {
                    bone_paths.entry(path).or_insert(index);
                }
            }
        }
        Self {
            skeleton,
            bone_names: Some(bone_names),
            bone_paths,
        }
    }

    fn resolve(&self, track: &AnimationClipBoneTrackAsset) -> Option<usize> {
        let Some(bone_names) = self.bone_names.as_ref() else {
            return resolve_clip_track_bone_index(self.skeleton, track);
        };
        if let Some(target_id) = track
            .target_id
            .as_deref()
            .map(str::trim)
            .filter(|target_id| !target_id.is_empty())
        {
            if let Some(index) = bone_names.get(target_id).copied() {
                return Some(index);
            }
            if let Some(index) = self.bone_paths.get(target_id).copied() {
                return Some(index);
            }
        }
        bone_names.get(track.bone_name.as_str()).copied()
    }
}

fn animation_pose_bone_from_skeleton(
    bone: &AnimationSkeletonBoneAsset,
) -> AnimationResult<AnimationPoseBone> {
    if !real_array_is_finite(&bone.local_translation) {
        return Err(AnimationError::NonFiniteSkeletonBind {
            bone: bone.name.clone(),
            field: "translation",
        });
    }
    if !real_array_is_finite(&bone.local_rotation) {
        return Err(AnimationError::NonFiniteSkeletonBind {
            bone: bone.name.clone(),
            field: "rotation",
        });
    }
    if !quaternion_array_is_normalizable(&bone.local_rotation) {
        return Err(AnimationError::ZeroLengthSkeletonBindRotation {
            bone: bone.name.clone(),
        });
    }
    if !real_array_is_finite(&bone.local_scale) {
        return Err(AnimationError::NonFiniteSkeletonBind {
            bone: bone.name.clone(),
            field: "scale",
        });
    }

    Ok(AnimationPoseBone {
        name: bone.name.clone(),
        local_transform: Transform {
            translation: Vec3::from_array(bone.local_translation),
            rotation: Quat::from_array(bone.local_rotation).normalize(),
            scale: Vec3::from_array(bone.local_scale),
        },
    })
}

fn resolve_clip_track_bone_index(
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

fn skeleton_bone_path(skeleton: &AnimationSkeletonAsset, index: usize) -> Option<String> {
    let bone = skeleton.bones.get(index)?;
    let mut segments = vec![bone.name.clone()];
    let mut parent = bone.parent_index;
    while let Some(parent_index) = parent {
        let parent_bone = skeleton.bones.get(parent_index as usize)?;
        segments.push(parent_bone.name.clone());
        parent = parent_bone.parent_index;
    }
    segments.reverse();
    Some(segments.join("/"))
}

#[cfg(test)]
mod optimization_batch_20260830cl_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime::core::framework::animation::{
        AnimationChannelAsset, AnimationClipBoneTrackAsset, AnimationInterpolationAsset,
        AnimationSkeletonAsset, AnimationSkeletonBoneAsset,
    };

    use super::{should_index_bone_lookup, skeleton_bone_path, ClipTrackBoneLookup};

    const SAMPLE_PAIRS: usize = 21;
    const BENCH_BONE_COUNT: usize = 128;

    #[test]
    fn optimization_batch_20260830cl_pose_lookup_preserves_resolution_order() {
        let skeleton = AnimationSkeletonAsset {
            name: Some("duplicate-leaves".to_string()),
            bones: vec![
                bone("Root", None),
                bone("Hand", Some(0)),
                bone("Branch", Some(0)),
                bone("Hand", Some(2)),
            ],
        };
        let mut tracks = vec![
            track("Hand", None),
            track("ignored", Some("Root/Branch/Hand")),
            track("Hand", Some("missing/path")),
        ];
        tracks.extend((0..509).map(|_| track("Hand", None)));
        let lookup = ClipTrackBoneLookup::new(&skeleton, &tracks);

        assert_eq!(lookup.resolve(&tracks[0]), Some(1));
        assert_eq!(lookup.resolve(&tracks[1]), Some(3));
        assert_eq!(lookup.resolve(&tracks[2]), Some(1));
    }

    #[test]
    fn optimization_batch_20260830cl_pose_lookup_is_adaptive() {
        assert!(!should_index_bone_lookup(16, 16));
        assert!(!should_index_bone_lookup(32, 32));
        assert!(should_index_bone_lookup(64, 64));
        assert!(should_index_bone_lookup(128, 256));
    }

    #[test]
    #[ignore = "release-only adaptive clip track bone lookup benchmark"]
    fn optimization_batch_20260830cl_pose_lookup_release_benchmark() {
        let (skeleton, tracks) = benchmark_fixture();
        assert_eq!(
            legacy_checksum(&skeleton, &tracks),
            indexed_checksum(&skeleton, &tracks)
        );

        let (legacy_samples, optimized_samples) = paired_samples(
            || measure_legacy(&skeleton, &tracks),
            || measure_indexed(&skeleton, &tracks),
        );
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "RUNTIME170_PLUGIN_POSE_ADAPTIVE_INDEX_BENCH_V1 sample_pairs={SAMPLE_PAIRS} bone_count={BENCH_BONE_COUNT} track_count={BENCH_BONE_COUNT} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );
        assert!(
            optimized_p95_ns.saturating_mul(5) <= legacy_p95_ns,
            "adaptive pose lookup must reduce large path-target P95 by at least 80%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
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
        let lookup = ClipTrackBoneLookup::new(skeleton, tracks);
        tracks
            .iter()
            .filter_map(|track| lookup.resolve(track))
            .sum()
    }

    fn paired_samples(
        mut legacy: impl FnMut() -> u128,
        mut optimized: impl FnMut() -> u128,
    ) -> (Vec<u128>, Vec<u128>) {
        for _ in 0..4 {
            black_box(legacy());
            black_box(optimized());
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(legacy());
                optimized_samples.push(optimized());
            } else {
                optimized_samples.push(optimized());
                legacy_samples.push(legacy());
            }
        }
        (legacy_samples, optimized_samples)
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
}

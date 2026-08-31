use crate::asset::assets::ImportedAsset;
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};
use crate::core::framework::animation::{
    AnimationClipAsset, AnimationGraphAsset, AnimationSequenceAsset, AnimationSkeletonAsset,
    AnimationStateMachineAsset,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AnimationAssetKind {
    Skeleton,
    Clip,
    Sequence,
    Graph,
    StateMachine,
}

impl AnimationAssetKind {
    fn from_file_name(file_name: &str) -> Option<Self> {
        const SUFFIXES: [(&str, AnimationAssetKind); 5] = [
            (".skeleton.zranim", AnimationAssetKind::Skeleton),
            (".clip.zranim", AnimationAssetKind::Clip),
            (".sequence.zranim", AnimationAssetKind::Sequence),
            (".graph.zranim", AnimationAssetKind::Graph),
            (".state_machine.zranim", AnimationAssetKind::StateMachine),
        ];
        SUFFIXES.iter().find_map(|(suffix, kind)| {
            file_name
                .get(file_name.len().checked_sub(suffix.len())?..)
                .is_some_and(|tail| tail.eq_ignore_ascii_case(suffix))
                .then_some(*kind)
        })
    }
}

pub(crate) fn import_animation_asset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let file_name = context
        .source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();

    match AnimationAssetKind::from_file_name(file_name) {
        Some(AnimationAssetKind::Skeleton) => {
            AnimationSkeletonAsset::from_bytes(&context.source_bytes)
                .map(ImportedAsset::AnimationSkeleton)
                .map(|asset| AssetImportOutcome::new(context.uri.clone(), asset))
                .map_err(AssetImportError::AnimationAsset)
        }
        Some(AnimationAssetKind::Clip) => AnimationClipAsset::from_bytes(&context.source_bytes)
            .map(ImportedAsset::AnimationClip)
            .map(|asset| AssetImportOutcome::new(context.uri.clone(), asset))
            .map_err(AssetImportError::AnimationAsset),
        Some(AnimationAssetKind::Sequence) => {
            AnimationSequenceAsset::from_bytes(&context.source_bytes)
                .map(ImportedAsset::AnimationSequence)
                .map(|asset| AssetImportOutcome::new(context.uri.clone(), asset))
                .map_err(AssetImportError::AnimationAsset)
        }
        Some(AnimationAssetKind::Graph) => AnimationGraphAsset::from_bytes(&context.source_bytes)
            .map(ImportedAsset::AnimationGraph)
            .map(|asset| AssetImportOutcome::new(context.uri.clone(), asset))
            .map_err(AssetImportError::AnimationAsset),
        Some(AnimationAssetKind::StateMachine) => {
            AnimationStateMachineAsset::from_bytes(&context.source_bytes)
                .map(ImportedAsset::AnimationStateMachine)
                .map(|asset| AssetImportOutcome::new(context.uri.clone(), asset))
                .map_err(AssetImportError::AnimationAsset)
        }
        None => Err(AssetImportError::UnsupportedFormat(format!(
            "unknown animation asset suffix for {}",
            context.source_path.display()
        ))),
    }
}

#[cfg(test)]
mod plugins07_animation_hotpath_tests {
    use std::{hint::black_box, time::Instant};

    use super::*;

    const FILE_NAMES: [&str; 10] = [
        "Hero.SKELETON.ZRANIM",
        "hero.skeleton.zranim",
        "Walk.CLIP.ZRANIM",
        "walk.clip.zranim",
        "Combo.SEQUENCE.ZRANIM",
        "combo.sequence.zranim",
        "Locomotion.GRAPH.ZRANIM",
        "locomotion.graph.zranim",
        "Combat.STATE_MACHINE.ZRANIM",
        "combat.state_machine.zranim",
    ];

    fn legacy_animation_asset_kind(file_name: &str) -> Option<AnimationAssetKind> {
        let lower_name = file_name.to_ascii_lowercase();
        if lower_name.ends_with(".skeleton.zranim") {
            Some(AnimationAssetKind::Skeleton)
        } else if lower_name.ends_with(".clip.zranim") {
            Some(AnimationAssetKind::Clip)
        } else if lower_name.ends_with(".sequence.zranim") {
            Some(AnimationAssetKind::Sequence)
        } else if lower_name.ends_with(".graph.zranim") {
            Some(AnimationAssetKind::Graph)
        } else if lower_name.ends_with(".state_machine.zranim") {
            Some(AnimationAssetKind::StateMachine)
        } else {
            None
        }
    }

    #[test]
    fn plugins07_builtin_import_hotpath_animation_suffix_preserves_case_insensitive_kinds() {
        for file_name in FILE_NAMES {
            assert_eq!(
                AnimationAssetKind::from_file_name(file_name),
                legacy_animation_asset_kind(file_name),
            );
        }
        assert_eq!(AnimationAssetKind::from_file_name("readme.txt"), None);
    }

    #[test]
    #[ignore = "release-only borrowed animation suffix benchmark"]
    fn plugins07_builtin_import_hotpath_release_borrowed_animation_suffix_p95_gate() {
        const SAMPLE_PAIRS: usize = 21;
        const CHECKS_PER_SAMPLE: usize = 100_000;

        fn measure(classify: impl Fn(&str) -> Option<AnimationAssetKind>) -> u128 {
            let started = Instant::now();
            for check in 0..CHECKS_PER_SAMPLE {
                let file_name = black_box(FILE_NAMES[check % FILE_NAMES.len()]);
                black_box(classify(file_name));
            }
            started.elapsed().as_nanos().max(1)
        }

        let (legacy_samples, optimized_samples) = alternating_samples(
            SAMPLE_PAIRS,
            || measure(legacy_animation_asset_kind),
            || measure(AnimationAssetKind::from_file_name),
        );
        report_and_assert(
            "plugins07_builtin_animation_suffix_dispatch",
            SAMPLE_PAIRS,
            CHECKS_PER_SAMPLE,
            FILE_NAMES.len(),
            CHECKS_PER_SAMPLE,
            &legacy_samples,
            &optimized_samples,
        );
    }

    fn alternating_samples(
        sample_pairs: usize,
        mut legacy: impl FnMut() -> u128,
        mut optimized: impl FnMut() -> u128,
    ) -> (Vec<u128>, Vec<u128>) {
        for _ in 0..4 {
            black_box(legacy());
            black_box(optimized());
        }
        let mut legacy_samples = Vec::with_capacity(sample_pairs);
        let mut optimized_samples = Vec::with_capacity(sample_pairs);
        for pair in 0..sample_pairs {
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

    fn report_and_assert(
        name: &str,
        sample_pairs: usize,
        checks_per_sample: usize,
        variants: usize,
        legacy_owned_strings_per_sample: usize,
        legacy_samples: &[u128],
        optimized_samples: &[u128],
    ) {
        let legacy_p95_ns = percentile(legacy_samples, 95);
        let optimized_p95_ns = percentile(optimized_samples, 95);
        let improvement_percent = improvement_percent(legacy_p95_ns, optimized_p95_ns);
        println!(
            "PERF_RESULT {name} sample_pairs={sample_pairs} \
checks_per_sample={checks_per_sample} variants={variants} \
order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_owned_strings_per_sample={legacy_owned_strings_per_sample} \
optimized_owned_strings_per_sample=0 legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} \
threshold_percent=50 legacy_ns={} optimized_ns={}",
            raw(legacy_samples),
            raw(optimized_samples),
        );
        assert!(
            optimized_p95_ns.saturating_mul(2) <= legacy_p95_ns,
            "borrowed builtin import classification must reduce P95 by at least 50%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn improvement_percent(legacy: u128, optimized: u128) -> u128 {
        if optimized >= legacy {
            0
        } else {
            legacy.saturating_sub(optimized).saturating_mul(100) / legacy.max(1)
        }
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}

use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use super::{FontFamilyCandidateScope, dedupe_families, dedupe_scoped_families};
use crate::text::FontFamilyName;

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const FAMILIES_PER_BUILD: usize = 256;

#[test]
fn optimization_batch_20260826fd_runtime199_capacity_preserves_font_family_deduplication() {
    let families = vec![
        FontFamilyName::from(" Inter "),
        FontFamilyName::from("inter"),
        FontFamilyName::from(""),
        FontFamilyName::from("Noto Sans"),
        FontFamilyName::from("Fira Mono"),
        FontFamilyName::from("NOTO SANS"),
    ];

    let deduped = dedupe_families(families);

    assert_eq!(deduped.len(), 3);
    assert_eq!(deduped[0].as_str(), "Inter");
    assert_eq!(deduped[1].as_str(), "Noto Sans");
    assert_eq!(deduped[2].as_str(), "Fira Mono");
    assert!(deduped.capacity() >= 6);
}

#[test]
fn optimization_batch_20260826fd_runtime199_font_family_dedupe_reserves_iterator_lower_bound() {
    let source = include_str!("../matching.rs");
    assert!(source.contains("let families = families.into_iter();"));
    assert!(source.contains("let (capacity, _) = families.size_hint();"));
    assert!(source.contains("HashSet::with_capacity(capacity)"));
    assert!(source.contains("Vec::with_capacity(capacity)"));
}

#[test]
fn scoped_font_family_dedupe_preserves_or_upgrades_external_fallback_authority() {
    let local_only = dedupe_scoped_families([(
        FontFamilyName::from("Owner Typeface"),
        FontFamilyCandidateScope::OwnerLocalOnly,
    )]);
    let explicitly_external = dedupe_scoped_families([
        (
            FontFamilyName::from("Owner Typeface"),
            FontFamilyCandidateScope::OwnerLocalOnly,
        ),
        (
            FontFamilyName::from("owner typeface"),
            FontFamilyCandidateScope::OwnerThenGlobal,
        ),
    ]);

    assert_eq!(local_only.len(), 1);
    assert_eq!(
        local_only[0].scope,
        FontFamilyCandidateScope::OwnerLocalOnly
    );
    assert_eq!(explicitly_external.len(), 1);
    assert_eq!(
        explicitly_external[0].scope,
        FontFamilyCandidateScope::OwnerThenGlobal
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fd_runtime199_font_family_dedupe_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME199_FONT_FAMILY_DEDUPE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} families_per_build={FAMILIES_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=2 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut identities = if reserve {
            HashSet::with_capacity(FAMILIES_PER_BUILD)
        } else {
            HashSet::new()
        };
        let mut families = if reserve {
            Vec::with_capacity(FAMILIES_PER_BUILD)
        } else {
            Vec::new()
        };
        for family in 0..FAMILIES_PER_BUILD {
            if identities.insert(black_box(family)) {
                families.push(family);
            }
        }
        checksum ^= black_box(identities.len() ^ families.len() ^ families.capacity());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

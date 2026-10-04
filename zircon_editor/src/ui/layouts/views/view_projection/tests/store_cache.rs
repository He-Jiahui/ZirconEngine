use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const ROOT_COUNT: usize = 8_192;
const ROOT_LOOKUP_COUNT: usize = 65_536;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn roots() -> Vec<PathBuf> {
    (0..ROOT_COUNT)
        .map(|index| {
            PathBuf::from(format!(
                "generated/projects/with/a/long/shared/prefix/project_{index:05}/assets"
            ))
        })
        .collect()
}

fn lookups(roots: &[PathBuf]) -> Vec<PathBuf> {
    (0..ROOT_LOOKUP_COUNT)
        .map(|index| roots[(index * 4_099) % roots.len()].clone())
        .collect()
}

fn ordered_match_count(
    source_roots: &BTreeMap<PathBuf, Option<PathBuf>>,
    unavailable_roots: &BTreeSet<PathBuf>,
    lookups: &[PathBuf],
) -> usize {
    lookups
        .iter()
        .filter(|root| {
            source_roots.contains_key(root.as_path()) && unavailable_roots.contains(root.as_path())
        })
        .count()
}

fn hash_match_count(
    source_roots: &HashMap<PathBuf, Option<PathBuf>>,
    unavailable_roots: &HashSet<PathBuf>,
    lookups: &[PathBuf],
) -> usize {
    lookups
        .iter()
        .filter(|root| {
            source_roots.contains_key(root.as_path()) && unavailable_roots.contains(root.as_path())
        })
        .count()
}

#[test]
fn v2_store_invalidation_ignores_non_zui_file_events() {
    assert!(!affects_v2_source(
        &Event::new(EventKind::Modify(notify::event::ModifyKind::Any))
            .add_path(PathBuf::from("icon.svg"))
    ));
    assert!(affects_v2_source(
        &Event::new(EventKind::Modify(notify::event::ModifyKind::Any))
            .add_path(PathBuf::from("layout.zui"))
    ));
}

#[test]
fn optimization_batch_20260826ad_editor01_hash_root_caches_preserve_unavailable_membership() {
    let source = PathBuf::from("project/assets/ui/views/main.zui");
    let root = PathBuf::from("project/assets");
    let mut invalidation = ViewTemplateStoreInvalidation::new();
    invalidation
        .source_roots
        .insert(source.clone(), Some(root.clone()));
    invalidation.unavailable_roots.insert(root.clone());

    assert_eq!(
        invalidation.source_roots.get(&source),
        Some(&Some(root.clone()))
    );
    assert!(invalidation.unavailable_roots.contains(&root));
    assert!(invalidation.watchers.is_empty());
}

#[test]
fn optimization_batch_20260826ad_editor01_view_store_uses_hash_caches_and_ordered_watch_roots() {
    let source = include_str!("../store_cache.rs");

    assert!(source.contains("use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};"));
    assert!(source.contains("unavailable_roots: HashSet<PathBuf>"));
    assert!(source.contains("source_roots: HashMap<PathBuf, Option<PathBuf>>"));
    assert!(source.contains("let mut roots = BTreeSet::new();"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ad_editor01_view_store_root_hash_caches_performance_evidence() {
    let roots = roots();
    let lookups = lookups(&roots);
    let ordered_source_roots = roots
        .iter()
        .cloned()
        .map(|root| (root.clone(), Some(root)))
        .collect::<BTreeMap<_, _>>();
    let ordered_unavailable_roots = roots.iter().cloned().collect::<BTreeSet<_>>();
    let hash_source_roots = roots
        .iter()
        .cloned()
        .map(|root| (root.clone(), Some(root)))
        .collect::<HashMap<_, _>>();
    let hash_unavailable_roots = roots.iter().cloned().collect::<HashSet<_>>();
    assert_eq!(
        ordered_match_count(&ordered_source_roots, &ordered_unavailable_roots, &lookups,),
        hash_match_count(&hash_source_roots, &hash_unavailable_roots, &lookups)
    );

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_match_count(
                black_box(&ordered_source_roots),
                black_box(&ordered_unavailable_roots),
                black_box(&lookups),
            ));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_match_count(
                black_box(&hash_source_roots),
                black_box(&hash_unavailable_roots),
                black_box(&lookups),
            ));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_match_count(
                black_box(&hash_source_roots),
                black_box(&hash_unavailable_roots),
                black_box(&lookups),
            ));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_match_count(
                black_box(&ordered_source_roots),
                black_box(&ordered_unavailable_roots),
                black_box(&lookups),
            ));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "EDITOR01_VIEW_STORE_ROOT_HASH_CACHES_BENCH_V1 \
             cached_roots={ROOT_COUNT} lookups={ROOT_LOOKUP_COUNT} \
             ordered_watch_registration=true ordered_p95_ns={} hash_p95_ns={}",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
        "hash-cache P95 {:?} exceeded 60% of ordered-cache P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}

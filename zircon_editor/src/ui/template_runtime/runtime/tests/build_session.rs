use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::super::runtime_host::{
    clear_v2_template_file_cache_for_tests, v2_template_file_cache_len_for_tests,
};
use super::*;

const IMPORT_ADMISSION_COUNT: usize = 65_536;
const UNIQUE_IMPORT_COUNT: usize = 8_192;
const PREALLOCATED_IMPORT_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn import_references() -> Vec<String> {
    (0..IMPORT_ADMISSION_COUNT)
        .map(|index| {
            format!(
                "res://ui/imports/{:04}.zui",
                (index * 4_099) % UNIQUE_IMPORT_COUNT
            )
        })
        .collect()
}

fn legacy_import_admission_count(references: &[String]) -> usize {
    let mut seen = BTreeSet::new();
    references
        .iter()
        .filter(|reference| seen.insert((*reference).clone()))
        .count()
}

fn optimized_import_admission_count(references: &[String]) -> usize {
    let mut seen = HashSet::new();
    references
        .iter()
        .filter(|reference| admit_import_reference(&mut seen, reference))
        .count()
}

fn preallocation_import_references() -> Vec<String> {
    (0..PREALLOCATED_IMPORT_COUNT)
        .map(|index| format!("res://ui/imports/preallocated_{index:05}.zui"))
        .collect()
}

fn unreserved_import_count(references: &[String]) -> usize {
    let mut seen = HashSet::new();
    references
        .iter()
        .filter(|reference| admit_import_reference(&mut seen, reference))
        .count()
}

fn reserved_import_count(references: &[String]) -> usize {
    let mut seen = HashSet::with_capacity(references.len());
    references
        .iter()
        .filter(|reference| admit_import_reference(&mut seen, reference))
        .count()
}

#[test]
fn builtin_v2_template_file_cache_is_reused_across_runtime_instances() {
    clear_v2_template_file_cache_for_tests();
    builtin_template_compile_cache()
        .lock()
        .expect("tree-template compile cache mutex should not be poisoned")
        .clear();
    builtin_template_document_cache()
        .lock()
        .expect("tree-template document cache mutex should not be poisoned")
        .clear();

    let mut first = EditorUiHostRuntime::default();
    first
        .load_builtin_host_templates()
        .expect("first runtime should load builtin templates");
    let v2_entries_after_first = v2_template_file_cache_len_for_tests();

    let mut second = EditorUiHostRuntime::default();
    second
        .load_builtin_host_templates()
        .expect("second runtime should reuse builtin template cache");

    assert!(v2_entries_after_first > 0);
    assert_eq!(
        v2_template_file_cache_len_for_tests(),
        v2_entries_after_first,
        "second runtime should not reload or recompile additional v2 builtin documents"
    );
    assert_eq!(
        builtin_template_compile_cache()
            .lock()
            .expect("tree-template compile cache mutex should not be poisoned")
            .len(),
        0,
        "v2 builtin host templates should bypass the tree-template compiler cache"
    );
    assert_eq!(
        builtin_template_document_cache()
            .lock()
            .expect("tree-template document cache mutex should not be poisoned")
            .len(),
        0,
        "v2 builtin host templates should bypass the tree-template document cache"
    );
}

#[test]
fn optimization_batch_20260826r_editor01_hash_membership_preserves_first_import_admission() {
    let mut seen = HashSet::new();

    assert!(admit_import_reference(&mut seen, "res://ui/shared.zui"));
    assert!(!admit_import_reference(&mut seen, "res://ui/shared.zui"));
    assert!(admit_import_reference(&mut seen, "res://ui/other.zui"));
    assert_eq!(seen.len(), 2);
}

#[test]
fn optimization_batch_20260826r_editor01_builtin_templates_use_hash_membership() {
    let source = include_str!("../build_session.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::{BTreeMap, HashSet};"));
    assert!(production.contains("collect::<HashSet<_>>()"));
    assert_eq!(production.matches("Option<&HashSet<&str>>").count(), 2);
    assert_eq!(production.matches("&mut HashSet<String>").count(), 2);
    assert!(!production.contains("BTreeSet"));
    assert!(
        production.find("seen_imports.contains(reference)").unwrap()
            < production
                .find("seen_imports.insert(reference.to_owned())")
                .unwrap()
    );
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826r_editor01_builtin_template_hash_membership_performance_evidence() {
    let references = import_references();
    assert_eq!(
        legacy_import_admission_count(&references),
        UNIQUE_IMPORT_COUNT
    );
    assert_eq!(
        optimized_import_admission_count(&references),
        UNIQUE_IMPORT_COUNT
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_import_admission_count(black_box(&references)));
            legacy_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(optimized_import_admission_count(black_box(&references)));
            optimized_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(optimized_import_admission_count(black_box(&references)));
            optimized_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(legacy_import_admission_count(black_box(&references)));
            legacy_samples.push(started.elapsed());
        }
    }

    let legacy_p95 = percentile_95(&mut legacy_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "EDITOR01_BUILTIN_TEMPLATE_HASH_MEMBERSHIP_BENCH_V1 admissions={IMPORT_ADMISSION_COUNT} \
             unique_imports={UNIQUE_IMPORT_COUNT} legacy_string_allocations={IMPORT_ADMISSION_COUNT} \
             optimized_string_allocations={UNIQUE_IMPORT_COUNT} legacy_p95_ns={} optimized_p95_ns={}",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 60,
        "hash-membership P95 {:?} exceeded 60% of tree-membership P95 {:?}",
        optimized_p95,
        legacy_p95,
    );
}

#[test]
fn optimization_batch_im_editor623_template_imports_preallocate_root_membership() {
    let source = include_str!("../build_session.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("let initial_import_count = document"));
    assert!(production.contains(".widgets"));
    assert!(production.contains(".saturating_add(document.imports.styles.len());"));
    assert!(
        production.contains("let mut seen_imports = HashSet::with_capacity(initial_import_count);")
    );
    assert!(production.contains("seen_imports.insert(reference.to_owned())"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_im_editor623_preallocated_template_import_performance_evidence() {
    let references = preallocation_import_references();
    assert_eq!(
        unreserved_import_count(&references),
        reserved_import_count(&references)
    );

    black_box(unreserved_import_count(black_box(&references)));
    black_box(reserved_import_count(black_box(&references)));

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(unreserved_import_count(black_box(&references)));
            unreserved_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(reserved_import_count(black_box(&references)));
            reserved_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(reserved_import_count(black_box(&references)));
            reserved_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(unreserved_import_count(black_box(&references)));
            unreserved_samples.push(started.elapsed());
        }
    }

    let unreserved_p95 = percentile_95(&mut unreserved_samples);
    let reserved_p95 = percentile_95(&mut reserved_samples);
    println!(
        "EDITOR623_PREALLOCATED_TEMPLATE_IMPORT_BENCH_V1 \
             imports={PREALLOCATED_IMPORT_COUNT} owned_identity=true \
             unreserved_p95_ns={} reserved_p95_ns={}",
        unreserved_p95.as_nanos(),
        reserved_p95.as_nanos(),
    );
    assert!(
        reserved_p95.as_nanos() * 100 <= unreserved_p95.as_nanos() * 85,
        "preallocated import membership P95 {:?} exceeded 85% of unreserved P95 {:?}",
        reserved_p95,
        unreserved_p95,
    );
}

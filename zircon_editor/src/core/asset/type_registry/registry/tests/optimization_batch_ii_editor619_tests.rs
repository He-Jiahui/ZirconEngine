use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::core::asset::{AssetContextCommandDescriptor, AssetCreationTemplateDescriptor};
use crate::core::editor_operation::EditorOperationPath;

use super::*;

const ENTRY_ID_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn entry_ids() -> Vec<String> {
    (0..ENTRY_ID_COUNT)
        .map(|index| format!("asset.generated.owner.validation.entry.{index:05}"))
        .collect()
}

fn ordered_unique_count(entry_ids: &[String]) -> usize {
    let mut unique = BTreeSet::new();
    entry_ids
        .iter()
        .filter(|entry_id| unique.insert(entry_id.as_str()))
        .count()
}

fn hash_unique_count(entry_ids: &[String]) -> usize {
    let mut unique = HashSet::with_capacity(entry_ids.len());
    entry_ids
        .iter()
        .filter(|entry_id| unique.insert(entry_id.as_str()))
        .count()
}

#[test]
fn optimization_batch_ii_editor619_entry_owner_validation_preserves_first_duplicate_errors() {
    let asset_type = AssetTypeId::parse("sample.asset").unwrap();
    let operation = EditorOperationPath::parse("sample.asset.create").unwrap();
    let templates = vec![
        AssetCreationTemplateDescriptor::new("sample.template", "First", operation.clone()),
        AssetCreationTemplateDescriptor::new("sample.template", "Second", operation.clone()),
    ];
    let template_error =
        validate_new_creation_template_owners(&asset_type, "plugin.sample", &templates)
            .unwrap_err();
    assert!(matches!(
        template_error,
        AssetTypeRegistryError::DuplicateEntryOwner {
            collection: "creation_templates",
            entry_id,
            first_owner,
            second_owner,
            ..
        } if entry_id == "sample.template"
            && first_owner == "plugin.sample"
            && second_owner == "plugin.sample"
    ));

    let commands = vec![
        AssetContextCommandDescriptor::new("sample.command", "First", operation.clone()),
        AssetContextCommandDescriptor::new("sample.command", "Second", operation),
    ];
    let command_error =
        validate_new_context_command_owners(&asset_type, "plugin.sample", &commands).unwrap_err();
    assert!(matches!(
        command_error,
        AssetTypeRegistryError::DuplicateEntryOwner {
            collection: "context_commands",
            entry_id,
            first_owner,
            second_owner,
            ..
        } if entry_id == "sample.command"
            && first_owner == "plugin.sample"
            && second_owner == "plugin.sample"
    ));
}

#[test]
fn optimization_batch_ii_editor619_entry_owner_validation_uses_preallocated_hash_membership() {
    let source = include_str!("../../registry.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::{BTreeMap, HashMap, HashSet};"));
    assert!(production.contains("let mut entry_ids = HashSet::with_capacity(templates.len());"));
    assert!(production.contains("let mut entry_ids = HashSet::with_capacity(commands.len());"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_ii_editor619_hash_entry_owner_validation_performance_evidence() {
    let entry_ids = entry_ids();
    assert_eq!(
        ordered_unique_count(&entry_ids),
        hash_unique_count(&entry_ids)
    );

    black_box(ordered_unique_count(black_box(&entry_ids)));
    black_box(hash_unique_count(black_box(&entry_ids)));

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&entry_ids)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_unique_count(black_box(&entry_ids)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_unique_count(black_box(&entry_ids)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&entry_ids)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "EDITOR619_HASH_ENTRY_OWNER_VALIDATION_BENCH_V1 \
         entries={ENTRY_ID_COUNT} borrowed_identity=true \
         ordered_p95_ns={} hash_p95_ns={}",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 40,
        "hash entry-owner validation P95 {:?} exceeded 40% of ordered P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}

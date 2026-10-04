use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::render::RenderLayerSet;
use crate::core::framework::scene::Mobility;
use crate::core::resource::ResourceId;

use super::super::visibility_batch_key::VisibilityBatchKey;
use super::*;

const SAMPLE_PAIRS: usize = 17;
const BATCHES_PER_SAMPLE: usize = 64;
const MEMBERS_PER_BATCH: usize = 128;

#[test]
fn visible_batch_projection_preserves_order_and_zip_truncation() {
    let batches = vec![
        batch("first", vec![1, 2, 3], vec![10, 20]),
        batch("empty", vec![4], vec![40]),
        batch("second", vec![2, 5], vec![22, 50]),
    ];
    let visible = BTreeSet::from([2, 3]);

    let projected = VisibilityContext::visible_batches_for_stable_instance_keys(&batches, &visible);

    assert_eq!(projected.len(), 2);
    assert_eq!(projected[0].key.material_id, batches[0].key.material_id);
    assert_eq!(projected[0].stable_instance_keys, vec![2]);
    assert_eq!(projected[0].entities, vec![20]);
    assert_eq!(projected[1].key.material_id, batches[2].key.material_id);
    assert_eq!(projected[1].stable_instance_keys, vec![2]);
    assert_eq!(projected[1].entities, vec![22]);
}

#[test]
fn visibility_projection_reserves_known_input_bounds() {
    let source = include_str!("../visibility_context.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("visibility context implementation");

    assert!(implementation.contains("Vec::with_capacity(self.renderable_entities.len())"));
    assert!(implementation.contains("Vec::with_capacity(self.bvh_instances.len())"));
    assert!(implementation.contains("Vec::with_capacity(batches.len())"));
    assert!(implementation.contains("batch.stable_instance_keys.len().min(batch.entities.len())"));
    assert!(!implementation.contains("members.into_iter().unzip()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cg_runtime_visibility_projection_capacity_p95() {
    let batches = benchmark_batches();
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&batches, false));
            optimized.push(measure(&batches, true));
        } else {
            optimized.push(measure(&batches, true));
            legacy.push(measure(&batches, false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME385_VISIBILITY_PROJECTION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} batches_per_sample={BATCHES_PER_SAMPLE} members_per_batch={MEMBERS_PER_BATCH} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn batch(label: &str, stable_instance_keys: Vec<u64>, entities: Vec<u64>) -> VisibilityBatch {
    VisibilityBatch {
        key: VisibilityBatchKey {
            render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            material_id: ResourceId::from_stable_label(&format!("tests/{label}/material")),
            model_id: ResourceId::from_stable_label(&format!("tests/{label}/model")),
            mobility: Mobility::Dynamic,
        },
        stable_instance_keys,
        entities,
    }
}

fn benchmark_batches() -> Vec<Vec<u64>> {
    (0..BATCHES_PER_SAMPLE)
        .map(|batch| {
            (0..MEMBERS_PER_BATCH)
                .map(|member| (batch * MEMBERS_PER_BATCH + member) as u64)
                .collect()
        })
        .collect()
}

fn measure(batches: &[Vec<u64>], use_capacity: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..32 {
        let mut projected = if use_capacity {
            Vec::with_capacity(batches.len())
        } else {
            Vec::new()
        };
        for batch in black_box(batches) {
            if use_capacity {
                let mut keys = Vec::with_capacity(batch.len());
                let mut entities = Vec::with_capacity(batch.len());
                for (entity, key) in batch.iter().enumerate() {
                    if key % 3 != 0 {
                        keys.push(*key);
                        entities.push(entity as u64);
                    }
                }
                projected.push((keys, entities));
            } else {
                let members = batch
                    .iter()
                    .enumerate()
                    .filter(|(_, key)| *key % 3 != 0)
                    .map(|(entity, key)| (*key, entity as u64))
                    .collect::<Vec<_>>();
                projected.push(members.into_iter().unzip());
            }
        }
        checksum ^= projected.iter().map(|(keys, _)| keys.len()).sum::<usize>();
        black_box(projected);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], p: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

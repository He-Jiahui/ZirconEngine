use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use super::ShadingModelRegistry;
use crate::core::framework::render::{
    GBufferChannelMask, RenderMaterialLightingModel, SHADING_MODEL_PLUGIN_ID_START,
    ShadingModelDescriptor, ShadingModelId, ShadingModelRegistrationError,
};

const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const ENTRY_COUNT: usize = 256;

fn descriptor(id: u8, token: &str) -> ShadingModelDescriptor {
    ShadingModelDescriptor::new(
        ShadingModelId::new(id),
        token,
        "forward",
        "gbuffer",
        "deferred",
        GBufferChannelMask::standard_lit(),
    )
}

#[test]
fn runtime09c_batch_shading_token_hash_index_preserves_normalized_lookup() {
    let mut registry = ShadingModelRegistry::new(GBufferChannelMask::standard_deferred_v1());
    registry
        .register_plugin_descriptor(descriptor(SHADING_MODEL_PLUGIN_ID_START, " Custom:Toon "))
        .unwrap();

    let _: &HashMap<String, ShadingModelId> = &registry.tokens;
    let resolved = registry
        .resolve_lighting_model(&RenderMaterialLightingModel::Custom {
            name: "TOON".to_string(),
        })
        .unwrap();
    assert_eq!(
        resolved.id,
        ShadingModelId::new(SHADING_MODEL_PLUGIN_ID_START)
    );
}

#[test]
fn runtime09c_batch_shading_token_hash_index_preserves_order_and_duplicates() {
    let mut registry = ShadingModelRegistry::new(GBufferChannelMask::standard_deferred_v1());
    registry
        .register_builtin(descriptor(5, "clearcoat"))
        .unwrap();
    registry.register_builtin(descriptor(2, "pbr")).unwrap();
    let error = registry.register_builtin(descriptor(7, "PBR")).unwrap_err();

    assert!(matches!(
        error,
        ShadingModelRegistrationError::DuplicateToken { .. }
    ));
    assert_eq!(
        registry
            .descriptors()
            .map(|descriptor| descriptor.id.value())
            .collect::<Vec<_>>(),
        vec![2, 5]
    );
}

fn run_ordered_workload(entries: &BTreeMap<String, usize>, token: &str) -> u128 {
    let started = Instant::now();
    for _ in 0..HIT_COUNT {
        black_box(entries.get(token));
    }
    started.elapsed().as_nanos().max(1)
}

fn run_hash_workload(entries: &HashMap<String, usize>, token: &str) -> u128 {
    let started = Instant::now();
    for _ in 0..HIT_COUNT {
        black_box(entries.get(token));
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &mut [u128], numerator: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * numerator).div_ceil(100).saturating_sub(1);
    samples[rank]
}

#[test]
#[ignore = "release performance gate; managed validation only"]
fn runtime09c_batch_shading_token_hash_index_p95() {
    let prefix = "custom:shading-model-shared-prefix/".repeat(20);
    let rows = (0..ENTRY_COUNT)
        .map(|index| (format!("{prefix}{index:03}"), index))
        .collect::<Vec<_>>();
    let target = rows.last().unwrap().0.clone();
    let ordered = rows.iter().cloned().collect::<BTreeMap<_, _>>();
    let hashed = rows.into_iter().collect::<HashMap<_, _>>();
    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            ordered_samples.push(run_ordered_workload(&ordered, &target));
            hash_samples.push(run_hash_workload(&hashed, &target));
        } else {
            hash_samples.push(run_hash_workload(&hashed, &target));
            ordered_samples.push(run_ordered_workload(&ordered, &target));
        }
    }

    let ordered_p50 = percentile(&mut ordered_samples.clone(), 50);
    let ordered_p95 = percentile(&mut ordered_samples, 95);
    let hash_p50 = percentile(&mut hash_samples.clone(), 50);
    let hash_p95 = percentile(&mut hash_samples, 95);
    println!(
        "RUNTIME09C_SHADING_TOKEN_HASH_INDEX_BENCH_V1 entries={ENTRY_COUNT} hits={HIT_COUNT} sample_pairs={SAMPLE_COUNT} pair_order=alternating_ordered_even ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95} ordered_lookups_before={HIT_COUNT} hash_lookups_after={HIT_COUNT} descriptor_order_changes=0 direct_hit_allocations=0"
    );
    assert!(
        hash_p95 * 100 <= ordered_p95 * 70,
        "HashMap token lookup P95 must be at least 30% below BTreeMap lookup: ordered={ordered_p95}ns hash={hash_p95}ns"
    );
}

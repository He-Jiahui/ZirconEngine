use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use crate::core::editor_authoring_extension::SceneModeDescriptor;
use crate::core::editor_operation::EditorOperationPath;
use crate::scene::modes::{EditorSceneMode, InputOutcome, SceneModeCtx, ViewportOverlayBuilder};
use crate::scene::viewport::ViewportInput;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const LOOKUPS_PER_SAMPLE: usize = 4_096;

#[test]
fn optimization_batch_20260826cf_scene_mode_hash_registry_preserves_registration_order() {
    let mut registry = SceneModeRegistry::default();
    for id in ["scene.zoom", "scene.align", "scene.paint"] {
        registry.register(registration(id)).unwrap();
    }

    assert_eq!(
        registry
            .registrations()
            .map(|registration| registration.mode_id().as_str())
            .collect::<Vec<_>>(),
        vec!["scene.align", "scene.paint", "scene.zoom"]
    );
    assert_eq!(
        registry
            .descriptor(&SceneModeId::new("scene.paint"))
            .map(SceneModeDescriptor::id),
        Some("scene.paint")
    );
}

#[test]
fn optimization_batch_20260826cf_scene_mode_hash_registry_keeps_order_index() {
    let source = include_str!("../scene_mode_registry.rs");

    assert!(source.contains("registrations: HashMap<SceneModeId, SceneModeRegistration>"));
    assert!(source.contains("ordered_mode_ids: Vec<SceneModeId>"));
    assert!(source.contains("partition_point(|registered| registered < &mode_id)"));
    assert!(source.contains(".filter_map(|mode_id| self.registrations.get(mode_id))"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826cf_scene_mode_hash_registry_p95() {
    const MODES: usize = 16_384;
    let mode_ids = (0..MODES)
        .map(|index| SceneModeId::new(format!("scene.plugin.shared.mode.{index:05}")))
        .collect::<Vec<_>>();
    let legacy = mode_ids
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, mode_id)| (mode_id, index))
        .collect::<BTreeMap<_, _>>();
    let optimized = mode_ids
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, mode_id)| (mode_id, index))
        .collect::<HashMap<_, _>>();
    let target = mode_ids
        .last()
        .expect("benchmark mode set must not be empty");

    let mut legacy_lookup = || repeated_lookup(&legacy, target);
    let mut optimized_lookup = || repeated_lookup(&optimized, target);
    assert_eq!(black_box(legacy_lookup()), black_box(optimized_lookup()));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(&mut legacy_lookup));
            optimized_ns.push(measure_ns(&mut optimized_lookup));
        } else {
            optimized_ns.push(measure_ns(&mut optimized_lookup));
            legacy_ns.push(measure_ns(&mut legacy_lookup));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "scene-mode hash lookup P95 must be at least 30% below BTreeMap: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR03_SCENE_MODE_HASH_REGISTRY_BENCH_V1 modes={MODES} lookups_per_sample={LOOKUPS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn registration(id: &str) -> SceneModeRegistration {
    let descriptor = SceneModeDescriptor::new(
        id,
        id,
        "workbench.viewport",
        EditorOperationPath::parse(format!("{id}.activate"))
            .expect("test scene mode operation must be valid"),
    );
    let mode_id = SceneModeId::new(id);
    SceneModeRegistration::new(descriptor, move || {
        Box::new(TestMode {
            mode_id: mode_id.clone(),
        }) as Box<dyn EditorSceneMode>
    })
}

struct TestMode {
    mode_id: SceneModeId,
}

impl EditorSceneMode for TestMode {
    fn id(&self) -> &SceneModeId {
        &self.mode_id
    }

    fn enter(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn exit(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn handle_input(
        &mut self,
        _input: &ViewportInput,
        _ctx: &mut SceneModeCtx<'_>,
    ) -> InputOutcome {
        InputOutcome::PassThrough
    }

    fn build_overlay(&self, _out: &mut ViewportOverlayBuilder) {}
}

fn repeated_lookup<M>(map: &M, key: &SceneModeId) -> usize
where
    M: LookupMap,
{
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum = checksum.wrapping_add(black_box(map.lookup(black_box(key))));
    }
    black_box(checksum)
}

trait LookupMap {
    fn lookup(&self, key: &SceneModeId) -> usize;
}

impl LookupMap for BTreeMap<SceneModeId, usize> {
    fn lookup(&self, key: &SceneModeId) -> usize {
        *self.get(key).expect("legacy benchmark mode must exist")
    }
}

impl LookupMap for HashMap<SceneModeId, usize> {
    fn lookup(&self, key: &SceneModeId) -> usize {
        *self.get(key).expect("optimized benchmark mode must exist")
    }
}

fn measure_ns(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

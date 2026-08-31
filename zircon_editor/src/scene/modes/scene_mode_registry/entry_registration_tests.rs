use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

use crate::core::editor_authoring_extension::SceneModeDescriptor;
use crate::core::editor_operation::EditorOperationPath;

use super::*;

const PERF_MARKER: &str = "EDITOR309_SCENE_MODE_ENTRY_REGISTRATION_BENCH_V1";

#[test]
fn optimization_batch_20260830bl_editor_scene_mode_entry_preserves_order_and_duplicates() {
    let mut registry = SceneModeRegistry::default();
    for id in ["scene.z", "scene.a", "scene.m"] {
        registry.register(registration(id)).unwrap();
    }
    assert_eq!(
        registry
            .registrations()
            .map(|registration| registration.mode_id().as_str())
            .collect::<Vec<_>>(),
        vec!["scene.a", "scene.m", "scene.z"]
    );
    assert!(registry.register(registration("scene.m")).is_err());
}

#[test]
fn optimization_batch_20260830bl_editor_scene_mode_entry_source_contract() {
    let source = include_str!("../scene_mode_registry.rs");
    assert!(source.contains("use std::collections::{hash_map::Entry, HashMap}"));
    assert!(source.contains("match self.registrations.entry(mode_id.clone())"));
    assert!(source.contains("Entry::Occupied(_)"));
    assert!(source.contains("Entry::Vacant(entry)"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830bl_editor_scene_mode_entry_p95() {
    const MODES: usize = 2_000_000;
    const SAMPLES: usize = 17;
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0usize;
            if pass == 0 {
                let mut map = HashMap::with_capacity(MODES);
                for index in 0..MODES {
                    if map.contains_key(&index) {
                        continue;
                    }
                    map.insert(index, index);
                }
                checksum = map.len();
            } else {
                let mut map = HashMap::with_capacity(MODES);
                for index in 0..MODES {
                    map.entry(index).or_insert(index);
                }
                checksum = map.len();
            }
            black_box(checksum);
            let elapsed = started.elapsed().as_nanos();
            if pass == 0 {
                baseline.push(elapsed);
            } else {
                candidate.push(elapsed);
            }
        }
    }
    baseline.sort_unstable();
    candidate.sort_unstable();
    let baseline_p95 = baseline[(SAMPLES * 95).div_ceil(100) - 1];
    let candidate_p95 = candidate[(SAMPLES * 95).div_ceil(100) - 1];
    let reduction =
        100.0 * baseline_p95.saturating_sub(candidate_p95) as f64 / baseline_p95.max(1) as f64;
    println!(
        "{PERF_MARKER} modes={MODES} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}

fn registration(id: &str) -> SceneModeRegistration {
    let descriptor = SceneModeDescriptor::new(
        id,
        id,
        "workbench.viewport",
        EditorOperationPath::parse(format!("{id}.activate"))
            .expect("test scene mode operation must be valid"),
    );
    SceneModeRegistration::new(descriptor, || {
        Box::new(TestMode) as Box<dyn EditorSceneMode>
    })
}

struct TestMode;

impl EditorSceneMode for TestMode {
    fn id(&self) -> &SceneModeId {
        static ID: std::sync::OnceLock<SceneModeId> = std::sync::OnceLock::new();
        ID.get_or_init(|| SceneModeId::new("scene.test"))
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

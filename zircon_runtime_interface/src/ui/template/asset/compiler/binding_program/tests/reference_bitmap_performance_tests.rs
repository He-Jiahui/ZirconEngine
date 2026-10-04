//! 用引用位图旧校验器对照稠密顺序校验，覆盖缺项、重复、跳号、代际错配和源序号错配。
use super::*;

fn compiled_binding(binding_id: u32, generation: u64) -> UiCompiledBinding {
    UiCompiledBinding {
        handle: UiCompiledBindingHandle {
            generation: UiCompiledBindingGeneration::new(generation),
            binding_id: UiBindingId::new(binding_id),
        },
        owner_asset_id: UiCompiledAssetId::default(),
        node_id: UiCompiledNodeId::default(),
        source_binding_index: binding_id,
        event: UiEventKind::Click,
        mode: UiBindingMode::Event,
        component_event: None,
        route_id: None,
        action_id: None,
        payload_missing_policy: UiBindingMissingValuePolicy::default(),
        payload_fields: Vec::new(),
        targets: Vec::new(),
    }
}

fn reference_program(binding_ids: Vec<u32>, binding_count: usize) -> UiCompiledBindingProgram {
    UiCompiledBindingProgram::new(
        UiCompiledBindingGeneration::new(1),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![UiCompiledNodeBindings {
            owner_asset_id: UiCompiledAssetId::default(),
            binding_ids: binding_ids.into_iter().map(UiBindingId::new).collect(),
        }],
        (0..binding_count)
            .map(|index| compiled_binding(index as u32, 1))
            .collect(),
    )
}

#[test]
fn dense_binding_sequence_preserves_reference_bitmap_results() {
    let mut generation_mismatch = reference_program(vec![0], 1);
    generation_mismatch.bindings[0].handle.generation = UiCompiledBindingGeneration::new(2);
    let mut source_index_mismatch = reference_program(vec![0], 1);
    source_index_mismatch.bindings[0].source_binding_index = 1;
    let cases = [
        reference_program(vec![0, 1, 2, 3], 4),
        reference_program(vec![0, 1, 2], 4),
        reference_program(vec![0, 0], 2),
        reference_program(vec![0, 2], 3),
        generation_mismatch,
        source_index_mismatch,
    ];

    for program in cases {
        assert_eq!(
            program.node_bindings_are_well_formed(),
            program.node_bindings_are_well_formed_bitmap(),
        );
    }
}

#[test]
#[ignore = "release-only dense binding sequence benchmark"]
fn runtime_interface03_batch32_dense_binding_sequence_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const BINDING_COUNT: usize = 128;
    const LOOKUP_COUNT: usize = 10_000;
    const SAMPLE_COUNT: usize = 11;
    let program = reference_program((0..BINDING_COUNT as u32).collect(), BINDING_COUNT);
    let mut bitmap_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut dense_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_bitmap = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(program.node_bindings_are_well_formed_bitmap());
            }
            started.elapsed().as_nanos()
        };
        let measure_dense = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(program.node_bindings_are_well_formed());
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            bitmap_samples.push(measure_bitmap());
            dense_samples.push(measure_dense());
        } else {
            dense_samples.push(measure_dense());
            bitmap_samples.push(measure_bitmap());
        }
    }

    bitmap_samples.sort_unstable();
    dense_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_DENSE_BINDING_SEQUENCE_BENCH_V1 bindings={BINDING_COUNT} lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} bitmap_p95_ns={} dense_p95_ns={}",
        bitmap_samples[p95], dense_samples[p95],
    );
    assert!(
        dense_samples[p95].saturating_mul(5) <= bitmap_samples[p95].saturating_mul(4),
        "dense binding sequence must improve P95 by at least 20%: bitmap={}ns dense={}ns",
        bitmap_samples[p95],
        dense_samples[p95],
    );
}

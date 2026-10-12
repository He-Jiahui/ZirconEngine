//! 比较复用与逐绑定新建属性位图的判定结果；样例覆盖跨绑定复用合法、行内重复、越界及乱序字段。
use super::*;

fn compiled_binding(binding_id: u32, property_ids: &[u32]) -> UiCompiledBinding {
    UiCompiledBinding {
        handle: UiCompiledBindingHandle {
            generation: UiCompiledBindingGeneration::new(1),
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
        payload_fields: property_ids
            .iter()
            .map(|property_id| UiCompiledActionPayloadField {
                property: UiPropertyId::new(*property_id),
                value: UiCompiledActionPayloadValue::Unavailable,
            })
            .collect(),
        targets: Vec::new(),
    }
}

fn payload_program(property_count: usize, binding_fields: &[Vec<u32>]) -> UiCompiledBindingProgram {
    UiCompiledBindingProgram::new(
        UiCompiledBindingGeneration::new(1),
        Vec::new(),
        (0..property_count)
            .map(|index| format!("property_{index:04}"))
            .collect(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        binding_fields
            .iter()
            .enumerate()
            .map(|(index, fields)| compiled_binding(index as u32, fields))
            .collect(),
    )
}

#[test]
fn reused_payload_field_bitmap_preserves_allocating_results() {
    let cases = [
        payload_program(4, &[vec![0, 1, 2], vec![0, 1, 2]]),
        payload_program(4, &[vec![0, 0]]),
        payload_program(4, &[vec![4]]),
        payload_program(4, &[vec![2, 1]]),
        payload_program(4, &[Vec::new(), Vec::new()]),
    ];

    for program in cases {
        assert_eq!(
            program.payload_fields_are_well_formed_reused(),
            program.payload_fields_are_well_formed_allocating(),
        );
    }
}

#[test]
#[ignore = "release-only reused payload-field bitmap benchmark"]
fn runtime_interface03_batch31_reused_payload_field_bitmap_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const PROPERTY_COUNT: usize = 4_096;
    const BINDING_COUNT: usize = 512;
    const LOOKUP_COUNT: usize = 64;
    const SAMPLE_COUNT: usize = 11;
    let binding_fields = (0..BINDING_COUNT)
        .map(|_| vec![0, 1, 2, 3])
        .collect::<Vec<_>>();
    let program = payload_program(PROPERTY_COUNT, &binding_fields);
    let mut allocating_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut reused_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_allocating = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(program.payload_fields_are_well_formed_allocating());
            }
            started.elapsed().as_nanos()
        };
        let measure_reused = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(program.payload_fields_are_well_formed_reused());
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            allocating_samples.push(measure_allocating());
            reused_samples.push(measure_reused());
        } else {
            reused_samples.push(measure_reused());
            allocating_samples.push(measure_allocating());
        }
    }

    allocating_samples.sort_unstable();
    reused_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_REUSED_PAYLOAD_FIELD_BITMAP_BENCH_V1 properties={PROPERTY_COUNT} bindings={BINDING_COUNT} fields_per_binding=4 lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} allocating_p95_ns={} reused_p95_ns={}",
        allocating_samples[p95], reused_samples[p95],
    );
    assert!(
        reused_samples[p95].saturating_mul(2) <= allocating_samples[p95],
        "reused payload-field bitmap must improve P95 by at least 50%: allocating={}ns reused={}ns",
        allocating_samples[p95],
        reused_samples[p95],
    );
}

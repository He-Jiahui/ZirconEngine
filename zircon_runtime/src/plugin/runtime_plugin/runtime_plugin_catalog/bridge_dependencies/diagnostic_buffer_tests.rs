use std::hint::black_box;
use std::time::Instant;

use super::RuntimePluginBridgeDisableBlocker;

const SAMPLE_PAIRS: usize = 21;
const DIAGNOSTICS_PER_SAMPLE: usize = 8_192;
const INTERFACES_PER_DIAGNOSTIC: usize = 32;

#[test]
fn optimization_batch_20260826dj_runtime153_bridge_blocker_preserves_diagnostic_contract() {
    let blocker = RuntimePluginBridgeDisableBlocker {
        provider_package_id: "render.provider".to_string(),
        dependent_package_id: "editor.consumer".to_string(),
        interface_ids: vec!["render.mesh.v1".to_string(), "render.scene.v2".to_string()],
    };
    assert_eq!(
        blocker.diagnostic(),
        "bridge.strong_target_disable_blocked: provider plugin `render.provider` cannot be disabled while dependent plugin `editor.consumer` requires interfaces [`render.mesh.v1`, `render.scene.v2`]"
    );

    let empty = RuntimePluginBridgeDisableBlocker {
        interface_ids: Vec::new(),
        ..blocker
    };
    assert!(empty.diagnostic().ends_with("requires interfaces []"));
}

#[test]
fn optimization_batch_20260826dj_runtime153_bridge_blocker_uses_exact_output_buffer() {
    let blocker = fixture_blocker();
    let diagnostic = blocker.diagnostic();
    assert_eq!(diagnostic.len(), diagnostic.capacity());

    let source = include_str!("../bridge_dependencies.rs");
    assert!(source.contains("String::with_capacity(self.diagnostic_len())"));
    assert!(source.contains("self.write_diagnostic(&mut diagnostic);"));
    assert!(source.contains("pub(super) fn write_diagnostic("));
    assert!(source.contains("diagnostic.push_str(interface_id);"));
    assert!(!source.contains(".map(|interface_id| format!(\"`{interface_id}`\"))"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dj_runtime153_bridge_blocker_single_buffer_diagnostic_bench() {
    let blocker = fixture_blocker();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&blocker, legacy_diagnostic));
            optimized_samples.push(measure(
                &blocker,
                RuntimePluginBridgeDisableBlocker::diagnostic,
            ));
        } else {
            optimized_samples.push(measure(
                &blocker,
                RuntimePluginBridgeDisableBlocker::diagnostic,
            ));
            legacy_samples.push(measure(&blocker, legacy_diagnostic));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME153_BRIDGE_BLOCKER_SINGLE_BUFFER_DIAGNOSTIC_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
diagnostics_per_sample={DIAGNOSTICS_PER_SAMPLE} interfaces_per_diagnostic={INTERFACES_PER_DIAGNOSTIC} \
legacy_allocations_per_diagnostic=35 optimized_allocations_per_diagnostic=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single-buffer bridge diagnostic P95 {optimized_p95_ns}ns must be at most 70% of nested formatting P95 {legacy_p95_ns}ns"
    );
}

fn fixture_blocker() -> RuntimePluginBridgeDisableBlocker {
    RuntimePluginBridgeDisableBlocker {
        provider_package_id: "render.provider.package".to_string(),
        dependent_package_id: "editor.consumer.package".to_string(),
        interface_ids: (0..INTERFACES_PER_DIAGNOSTIC)
            .map(|index| format!("render.interface.{index:02}.v1"))
            .collect(),
    }
}

fn legacy_diagnostic(blocker: &RuntimePluginBridgeDisableBlocker) -> String {
    format!(
        "bridge.strong_target_disable_blocked: provider plugin `{}` cannot be disabled while dependent plugin `{}` requires interfaces [{}]",
        blocker.provider_package_id,
        blocker.dependent_package_id,
        blocker
            .interface_ids
            .iter()
            .map(|interface_id| format!("`{interface_id}`"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn measure(
    blocker: &RuntimePluginBridgeDisableBlocker,
    render: fn(&RuntimePluginBridgeDisableBlocker) -> String,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..DIAGNOSTICS_PER_SAMPLE {
        checksum ^= black_box(render(black_box(blocker))).len();
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

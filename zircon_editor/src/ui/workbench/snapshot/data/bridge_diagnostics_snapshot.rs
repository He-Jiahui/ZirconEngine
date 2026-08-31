use zircon_runtime::core::framework::bridge::BridgeInterfaceStatus;
use zircon_runtime::plugin::{BridgeDiagnosticsMatrix, BridgeInterfaceSnapshot};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditorBridgeDiagnosticsSnapshot {
    pub summary: EditorBridgeDiagnosticsSummarySnapshot,
    pub rows: Vec<EditorBridgeInterfaceRowSnapshot>,
    pub diagnostic_lines: Vec<String>,
}

impl EditorBridgeDiagnosticsSnapshot {
    pub fn from_runtime_matrix(matrix: &BridgeDiagnosticsMatrix) -> Self {
        Self {
            summary: EditorBridgeDiagnosticsSummarySnapshot {
                total_interfaces: matrix.summary.total_interfaces,
                enabled_interfaces: matrix.summary.enabled_interfaces,
                disabled_interfaces: matrix.summary.disabled_interfaces,
                installed_providers: matrix.summary.installed_providers,
                missing_providers: matrix.summary.missing_providers,
                enabled_calls: matrix.summary.enabled_calls,
                not_enabled_calls: matrix.summary.not_enabled_calls,
            },
            rows: matrix
                .rows
                .iter()
                .map(EditorBridgeInterfaceRowSnapshot::from_runtime_row)
                .collect(),
            diagnostic_lines: matrix.diagnostic_lines(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditorBridgeDiagnosticsSummarySnapshot {
    pub total_interfaces: usize,
    pub enabled_interfaces: usize,
    pub disabled_interfaces: usize,
    pub installed_providers: usize,
    pub missing_providers: usize,
    pub enabled_calls: u64,
    pub not_enabled_calls: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorBridgeInterfaceRowSnapshot {
    pub slot: u32,
    pub interface_id: String,
    pub owner_module_slot: u32,
    pub generation: u32,
    pub provider_installed: bool,
    pub status: String,
    pub enabled_calls: u64,
    pub not_enabled_calls: u64,
}

impl EditorBridgeInterfaceRowSnapshot {
    fn from_runtime_row(row: &BridgeInterfaceSnapshot) -> Self {
        Self {
            slot: row.slot.raw(),
            interface_id: row.interface_id.clone(),
            owner_module_slot: row.owner.raw(),
            generation: row.generation,
            provider_installed: row.provider_installed,
            status: bridge_status_label(row.status),
            enabled_calls: row.diagnostics.enabled_calls,
            not_enabled_calls: row.diagnostics.not_enabled_calls,
        }
    }
}

fn bridge_status_label(status: BridgeInterfaceStatus) -> String {
    String::from(match status {
        BridgeInterfaceStatus::Absent => "Absent",
        BridgeInterfaceStatus::Enabled => "Enabled",
        BridgeInterfaceStatus::Disabled => "Disabled",
    })
}

#[cfg(test)]
mod optimization_batch_fa_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const LABELS_PER_SAMPLE: usize = 262_144;

    #[test]
    fn optimization_batch_fa_editor389_preserves_bridge_status_labels() {
        for status in [
            BridgeInterfaceStatus::Absent,
            BridgeInterfaceStatus::Enabled,
            BridgeInterfaceStatus::Disabled,
        ] {
            assert_eq!(bridge_status_label(status), format!("{status:?}"));
        }
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fa_editor389_direct_bridge_status_label_benchmark() {
        for _ in 0..4 {
            black_box(measure(|status| format!("{status:?}")));
            black_box(measure(bridge_status_label));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure(|status| format!("{status:?}")));
                optimized_samples.push(measure(bridge_status_label));
            } else {
                optimized_samples.push(measure(bridge_status_label));
                legacy_samples.push(measure(|status| format!("{status:?}")));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn measure(mut label: impl FnMut(BridgeInterfaceStatus) -> String) -> u128 {
        let statuses = [
            BridgeInterfaceStatus::Absent,
            BridgeInterfaceStatus::Enabled,
            BridgeInterfaceStatus::Disabled,
        ];
        let started = Instant::now();
        let mut checksum = 0_usize;
        for index in 0..LABELS_PER_SAMPLE {
            let value = label(black_box(statuses[index % statuses.len()]));
            checksum = checksum.wrapping_add(black_box(value.len()));
            black_box(value);
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "EDITOR389_DIRECT_BRIDGE_STATUS_LABEL_BENCH_V1 sample_pairs={SAMPLE_PAIRS} labels_per_sample={LABELS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=35",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(65) / 100,
            "direct bridge status labels must reduce P95 by at least 35%"
        );
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}

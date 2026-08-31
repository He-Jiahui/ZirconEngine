use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowExitCondition {
    OnPrimaryClosed,
    #[default]
    OnAllClosed,
    DontExit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowLifecyclePolicy {
    pub exit_condition: WindowExitCondition,
    pub close_when_requested: bool,
}

impl WindowLifecyclePolicy {
    pub fn with_exit_condition(mut self, exit_condition: WindowExitCondition) -> Self {
        self.exit_condition = exit_condition;
        self
    }

    pub fn with_close_when_requested(mut self, close_when_requested: bool) -> Self {
        self.close_when_requested = close_when_requested;
        self
    }

    pub fn should_close_on_request(self) -> bool {
        self.close_when_requested
    }

    pub fn should_exit_after_primary_close(self) -> bool {
        self.close_when_requested
            && matches!(
                self.exit_condition,
                WindowExitCondition::OnPrimaryClosed | WindowExitCondition::OnAllClosed
            )
    }

    pub fn diagnostic_lines(self) -> [String; 2] {
        [
            exit_condition_diagnostic_line(self.exit_condition),
            close_when_requested_diagnostic_line(self.close_when_requested),
        ]
    }
}

fn exit_condition_diagnostic_line(exit_condition: WindowExitCondition) -> String {
    const PREFIX: &str = "window.exit_condition=";
    let value = match exit_condition {
        WindowExitCondition::OnPrimaryClosed => "OnPrimaryClosed",
        WindowExitCondition::OnAllClosed => "OnAllClosed",
        WindowExitCondition::DontExit => "DontExit",
    };
    let mut line = String::with_capacity(PREFIX.len() + value.len());
    line.push_str(PREFIX);
    line.push_str(value);
    line
}

fn close_when_requested_diagnostic_line(close_when_requested: bool) -> String {
    const PREFIX: &str = "window.close_when_requested=";
    let value = if close_when_requested {
        "true"
    } else {
        "false"
    };
    let mut line = String::with_capacity(PREFIX.len() + value.len());
    line.push_str(PREFIX);
    line.push_str(value);
    line
}

impl Default for WindowLifecyclePolicy {
    fn default() -> Self {
        Self {
            exit_condition: WindowExitCondition::OnAllClosed,
            close_when_requested: true,
        }
    }
}

#[cfg(test)]
mod optimization_batch_fh_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const SNAPSHOTS_PER_SAMPLE: usize = 262_144;

    #[test]
    fn optimization_batch_fh_runtime464_lifecycle_diagnostics_preserve_bytes() {
        for policy in [
            WindowLifecyclePolicy::default(),
            WindowLifecyclePolicy::default()
                .with_exit_condition(WindowExitCondition::OnPrimaryClosed)
                .with_close_when_requested(false),
            WindowLifecyclePolicy::default()
                .with_exit_condition(WindowExitCondition::DontExit)
                .with_close_when_requested(true),
        ] {
            assert_eq!(policy.diagnostic_lines(), legacy_diagnostic_lines(policy));
        }
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fh_runtime464_direct_lifecycle_diagnostics_benchmark() {
        let policy = WindowLifecyclePolicy::default()
            .with_exit_condition(WindowExitCondition::OnPrimaryClosed)
            .with_close_when_requested(false);
        for _ in 0..4 {
            black_box(measure(legacy_diagnostic_lines, policy));
            black_box(measure(WindowLifecyclePolicy::diagnostic_lines, policy));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure(legacy_diagnostic_lines, policy));
                optimized_samples.push(measure(WindowLifecyclePolicy::diagnostic_lines, policy));
            } else {
                optimized_samples.push(measure(WindowLifecyclePolicy::diagnostic_lines, policy));
                legacy_samples.push(measure(legacy_diagnostic_lines, policy));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn legacy_diagnostic_lines(policy: WindowLifecyclePolicy) -> [String; 2] {
        [
            format!("window.exit_condition={:?}", policy.exit_condition),
            format!(
                "window.close_when_requested={}",
                policy.close_when_requested
            ),
        ]
    }

    fn measure(
        mut build: impl FnMut(WindowLifecyclePolicy) -> [String; 2],
        policy: WindowLifecyclePolicy,
    ) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..SNAPSHOTS_PER_SAMPLE {
            let lines = black_box(build(black_box(policy)));
            checksum = checksum.wrapping_add(lines[0].len() + lines[1].len());
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
            "RUNTIME464_DIRECT_LIFECYCLE_DIAGNOSTICS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} snapshots_per_sample={SNAPSHOTS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=20",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(80),
            "optimized p95 {optimized_p95}ns must be at most 80% of legacy p95 {legacy_p95}ns"
        );
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}

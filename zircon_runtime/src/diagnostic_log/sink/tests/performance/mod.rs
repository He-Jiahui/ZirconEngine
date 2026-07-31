mod case;
mod configuration;
mod critical;
mod output;
mod pacing;
mod report;
mod resources;
mod rss;
mod validation;

use std::time::Duration;

use case::run_case;

const LOG_RATES_PER_SECOND: [usize; 3] = [1, 1_000, 100_000];
const CALLER_COUNTS: [usize; 2] = [1, 64];
const SCOPED_RULE_COUNTS: [usize; 3] = [0, 10, 1_000];
const SINK_DELAYS: [Duration; 3] = [
    Duration::ZERO,
    Duration::from_millis(10),
    Duration::from_millis(100),
];

#[test]
fn perf_mvp_434_matrix_shape_stays_complete() {
    assert_eq!(
        LOG_RATES_PER_SECOND.len()
            * CALLER_COUNTS.len()
            * SCOPED_RULE_COUNTS.len()
            * SINK_DELAYS.len(),
        54
    );
}

#[test]
#[ignore = "PERF-MVP-434 evidence gate; run in a managed Windows test lane"]
fn perf_mvp_434_bounded_log_storm_matrix() {
    critical::run_critical_backpressure_companion();
    for logs_per_second in LOG_RATES_PER_SECOND {
        for caller_count in CALLER_COUNTS {
            for scoped_rule_count in SCOPED_RULE_COUNTS {
                for sink_delay in SINK_DELAYS {
                    run_case(logs_per_second, caller_count, scoped_rule_count, sink_delay).print();
                }
            }
        }
    }
}

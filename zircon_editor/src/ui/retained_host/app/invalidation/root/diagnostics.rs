use crate::ui::retained_host::HostInvalidationDiagnostics;

use super::HostInvalidationRoot;

impl HostInvalidationRoot {
    // 同一组失效计数分别供宿主性能日志和 UI 诊断快照消费，帮助辨别局部刷新与完整重建。
    pub(in crate::ui::retained_host::app) fn stats_summary(&self) -> String {
        invalidation_stats_summary([
            self.total_requests,
            self.layout_requests,
            self.presentation_requests,
            self.render_requests,
            self.paint_only_requests,
            self.hit_test_requests,
            self.window_metrics_requests,
            self.slow_path_rebuilds,
            self.render_rebuilds,
        ])
    }

    pub(in crate::ui::retained_host::app) fn diagnostics_snapshot(
        &self,
    ) -> HostInvalidationDiagnostics {
        HostInvalidationDiagnostics {
            slow_path_rebuild_count: self.slow_path_rebuilds,
            render_rebuild_count: self.render_rebuilds,
            paint_only_request_count: self.paint_only_requests,
        }
    }
}

fn invalidation_stats_summary(counts: [u64; 9]) -> String {
    const FIELD_PREFIXES: [&str; 9] = [
        "requests=",
        " layout=",
        " presentation=",
        " render=",
        " paint_only=",
        " hit_test=",
        " window_metrics=",
        " slow_path=",
        " render_path=",
    ];

    let mut summary = String::with_capacity(256);
    for (prefix, count) in FIELD_PREFIXES.into_iter().zip(counts) {
        summary.push_str(prefix);
        push_u64_decimal(&mut summary, count);
    }
    summary
}

fn push_u64_decimal(output: &mut String, mut value: u64) {
    let mut digits = [0_u8; 20];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    for digit in &digits[start..] {
        output.push(char::from(*digit));
    }
}

#[cfg(test)]
#[path = "tests/diagnostics_optimization_batch_fb_tests.rs"]
mod optimization_batch_fb_tests;

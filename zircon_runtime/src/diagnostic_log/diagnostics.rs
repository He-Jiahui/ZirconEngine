use std::time::Duration;

use crate::core::diagnostics::{
    DiagnosticSeriesCurrentSnapshot, DiagnosticSeriesSnapshot, DiagnosticStoreCurrentSnapshot,
    DiagnosticStoreSnapshot,
};

use super::sink::write_log_lazy;

pub const DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT: Duration = Duration::from_secs(1);
const NANOS_PER_SECOND: u128 = 1_000_000_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticStoreLogSchedule {
    wait_duration: Duration,
    elapsed: Duration,
    last_periods_due: u64,
    coalesced_periods: u64,
    enabled: bool,
}

pub fn format_diagnostic_store_snapshot(snapshot: &DiagnosticStoreSnapshot) -> Vec<String> {
    snapshot
        .series
        .iter()
        .filter_map(format_diagnostic_series)
        .collect()
}

pub fn format_diagnostic_store_current_snapshot(
    snapshot: &DiagnosticStoreCurrentSnapshot,
) -> Vec<String> {
    snapshot
        .series
        .iter()
        .map(format_diagnostic_current_series)
        .collect()
}

pub fn write_diagnostic_store_snapshot(scope: &str, snapshot: &DiagnosticStoreSnapshot) {
    for series in snapshot
        .series
        .iter()
        .filter(|series| series.current.is_some())
    {
        write_log_lazy(scope, || {
            format_diagnostic_series(series)
                .expect("diagnostic series with a current value must format")
        });
    }
}

pub fn write_diagnostic_store_current_snapshot(
    scope: &str,
    snapshot: &DiagnosticStoreCurrentSnapshot,
) {
    for series in &snapshot.series {
        write_log_lazy(scope, || format_diagnostic_current_series(series));
    }
}

impl DiagnosticStoreLogSchedule {
    pub const fn disabled() -> Self {
        Self {
            wait_duration: Duration::ZERO,
            elapsed: Duration::ZERO,
            last_periods_due: 0,
            coalesced_periods: 0,
            enabled: false,
        }
    }

    pub const fn repeating(wait_duration: Duration) -> Self {
        Self {
            wait_duration,
            elapsed: Duration::ZERO,
            last_periods_due: 0,
            coalesced_periods: 0,
            enabled: true,
        }
    }

    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub const fn wait_duration(&self) -> Duration {
        self.wait_duration
    }

    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub const fn last_periods_due(&self) -> u64 {
        self.last_periods_due
    }

    pub const fn coalesced_periods(&self) -> u64 {
        self.coalesced_periods
    }

    pub fn tick(&mut self, delta: Duration) -> bool {
        self.last_periods_due = 0;
        if !self.enabled {
            return false;
        }
        if self.wait_duration.is_zero() {
            self.elapsed = Duration::ZERO;
            self.last_periods_due = 1;
            return true;
        }

        self.elapsed = self.elapsed.saturating_add(delta);
        if self.elapsed < self.wait_duration {
            return false;
        }

        let elapsed_nanos = self.elapsed.as_nanos();
        let wait_nanos = self.wait_duration.as_nanos();
        let periods_due = elapsed_nanos / wait_nanos;
        self.elapsed = duration_from_nanos(elapsed_nanos % wait_nanos);
        self.last_periods_due = saturating_u128_to_u64(periods_due);
        self.coalesced_periods = self
            .coalesced_periods
            .saturating_add(saturating_u128_to_u64(periods_due.saturating_sub(1)));
        true
    }
}

fn duration_from_nanos(nanos: u128) -> Duration {
    Duration::new(
        (nanos / NANOS_PER_SECOND) as u64,
        (nanos % NANOS_PER_SECOND) as u32,
    )
}

fn saturating_u128_to_u64(value: u128) -> u64 {
    value.min(u64::MAX as u128) as u64
}

fn format_diagnostic_series(series: &DiagnosticSeriesSnapshot) -> Option<String> {
    Some(format_diagnostic_values(
        series.path.as_str(),
        series.current?,
        series.unit.as_deref(),
        series.smoothed,
        series.min,
        series.max,
    ))
}

fn format_diagnostic_current_series(series: &DiagnosticSeriesCurrentSnapshot) -> String {
    format_diagnostic_values(
        series.path.as_str(),
        series.current,
        series.unit.as_deref(),
        series.smoothed,
        series.min,
        series.max,
    )
}

fn format_diagnostic_values(
    path: &str,
    current: f64,
    unit: Option<&str>,
    smoothed: Option<f64>,
    min: Option<f64>,
    max: Option<f64>,
) -> String {
    let unit = unit.unwrap_or("");
    let mut line = format!("{path}: {current:.6}{unit}");
    if let Some(smoothed) = smoothed {
        line.push_str(&format!(" (smoothed {:.6}{}", smoothed, unit));
        if let Some(min) = min {
            line.push_str(&format!(", min {:.6}{}", min, unit));
        }
        if let Some(max) = max {
            line.push_str(&format!(", max {:.6}{}", max, unit));
        }
        line.push(')');
    }
    line
}

#[cfg(test)]
mod tests;

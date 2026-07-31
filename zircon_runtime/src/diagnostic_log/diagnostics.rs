use std::time::Duration;

use crate::core::diagnostics::{DiagnosticSeriesSnapshot, DiagnosticStoreSnapshot};

use super::sink::write_log_lazy;

pub const DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT: Duration = Duration::from_secs(1);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticStoreLogSchedule {
    wait_duration: Duration,
    elapsed: Duration,
    enabled: bool,
}

pub fn format_diagnostic_store_snapshot(snapshot: &DiagnosticStoreSnapshot) -> Vec<String> {
    snapshot
        .series
        .iter()
        .filter_map(format_diagnostic_series)
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

impl DiagnosticStoreLogSchedule {
    pub const fn disabled() -> Self {
        Self {
            wait_duration: Duration::ZERO,
            elapsed: Duration::ZERO,
            enabled: false,
        }
    }

    pub const fn repeating(wait_duration: Duration) -> Self {
        Self {
            wait_duration,
            elapsed: Duration::ZERO,
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

    pub fn tick(&mut self, delta: Duration) -> bool {
        if !self.enabled {
            return false;
        }
        if self.wait_duration.is_zero() {
            self.elapsed = Duration::ZERO;
            return true;
        }

        self.elapsed = self.elapsed.saturating_add(delta);
        if self.elapsed < self.wait_duration {
            return false;
        }
        while self.elapsed >= self.wait_duration {
            self.elapsed -= self.wait_duration;
        }
        true
    }
}

fn format_diagnostic_series(series: &DiagnosticSeriesSnapshot) -> Option<String> {
    let current = series.current?;
    let unit = series.unit.as_deref().unwrap_or("");
    let mut line = format!("{}: {:.6}{}", series.path.as_str(), current, unit);
    if let Some(smoothed) = series.smoothed {
        line.push_str(&format!(" (smoothed {:.6}{}", smoothed, unit));
        if let Some(min) = series.min {
            line.push_str(&format!(", min {:.6}{}", min, unit));
        }
        if let Some(max) = series.max {
            line.push_str(&format!(", max {:.6}{}", max, unit));
        }
        line.push(')');
    }
    Some(line)
}

#[cfg(test)]
mod tests;

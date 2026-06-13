use std::time::Duration;

use crate::core::diagnostics::{DiagnosticSeriesSnapshot, DiagnosticStoreSnapshot};

use super::sink::write_log;

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
    for line in format_diagnostic_store_snapshot(snapshot) {
        write_log(scope, line);
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
mod tests {
    use crate::core::diagnostics::DiagnosticStore;

    use super::{
        format_diagnostic_store_snapshot, DiagnosticStoreLogSchedule,
        DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT,
    };
    use std::time::Duration;

    #[test]
    fn diagnostic_store_snapshot_formats_current_smoothed_min_and_max() {
        let mut store = DiagnosticStore::new(4);
        store.record("time.frame_time", 1, 20.0, Some("ms"), ["time", "frame"]);
        store.record("time.frame_time", 2, 30.0, Some("ms"), ["time", "frame"]);

        let lines = format_diagnostic_store_snapshot(&store.snapshot());

        assert_eq!(
            lines,
            vec!["time.frame_time: 30.000000ms (smoothed 21.000000ms, min 20.000000ms, max 30.000000ms)"]
        );
    }

    #[test]
    fn diagnostic_store_log_schedule_repeats_after_wait_duration() {
        let mut schedule = DiagnosticStoreLogSchedule::repeating(DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT);

        assert!(schedule.is_enabled());
        assert_eq!(schedule.wait_duration(), Duration::from_secs(1));
        assert!(!schedule.tick(Duration::from_millis(400)));
        assert_eq!(schedule.elapsed(), Duration::from_millis(400));
        assert!(!schedule.tick(Duration::from_millis(500)));
        assert_eq!(schedule.elapsed(), Duration::from_millis(900));
        assert!(schedule.tick(Duration::from_millis(150)));
        assert_eq!(schedule.elapsed(), Duration::from_millis(50));
    }

    #[test]
    fn diagnostic_store_log_schedule_can_be_disabled_or_every_tick() {
        let mut disabled = DiagnosticStoreLogSchedule::disabled();
        let mut every_tick = DiagnosticStoreLogSchedule::repeating(Duration::ZERO);

        assert!(!disabled.is_enabled());
        assert!(!disabled.tick(Duration::from_secs(10)));
        assert!(every_tick.tick(Duration::ZERO));
        assert!(every_tick.tick(Duration::from_millis(16)));
        assert_eq!(every_tick.elapsed(), Duration::ZERO);
    }

    #[test]
    fn diagnostic_log_snapshot_bridge_stays_single_owner() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let diagnostic_log_dir = manifest_dir.join("src").join("diagnostic_log");
        let core_diagnostics_dir = manifest_dir
            .join("src")
            .join("core")
            .join("runtime")
            .join("diagnostics");

        for entry in std::fs::read_dir(&diagnostic_log_dir).unwrap_or_else(|error| {
            panic!("failed to read {}: {error}", diagnostic_log_dir.display())
        }) {
            let entry = entry
                .unwrap_or_else(|error| panic!("failed to read diagnostic_log entry: {error}"));
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("rs")
                || path.file_name().and_then(|value| value.to_str()) == Some("diagnostics.rs")
            {
                continue;
            }
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            for forbidden_bridge_token in [
                "core::diagnostics",
                "DiagnosticStoreSnapshot",
                "DiagnosticSeriesSnapshot",
            ] {
                assert!(
                    !source.contains(forbidden_bridge_token),
                    "diagnostic_log file {} should not bypass diagnostics.rs with `{forbidden_bridge_token}`",
                    path.display()
                );
            }
        }

        let mut core_diagnostic_files = Vec::new();
        collect_rust_files(&core_diagnostics_dir, &mut core_diagnostic_files);
        for path in core_diagnostic_files {
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            for forbidden_log_token in [
                "diagnostic_log",
                "write_log",
                "write_diagnostic",
                "initialize_process_log",
                "DiagnosticLog",
            ] {
                assert!(
                    !source.contains(forbidden_log_token),
                    "core diagnostics file {} should not depend on process-log token `{forbidden_log_token}`",
                    path.display()
                );
            }
        }
    }

    fn collect_rust_files(root: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(root)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        {
            let entry =
                entry.unwrap_or_else(|error| panic!("failed to read diagnostics entry: {error}"));
            let path = entry.path();
            if path.is_dir() {
                collect_rust_files(&path, files);
                continue;
            }
            if path.extension().and_then(|value| value.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
}

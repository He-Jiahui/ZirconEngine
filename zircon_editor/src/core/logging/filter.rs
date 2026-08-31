use std::collections::BTreeSet;

use super::{LogChannel, LogEntry, LogSeverity};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogFilter {
    channel_mask: u8,
    minimum_severity: LogSeverity,
}

impl Default for LogFilter {
    fn default() -> Self {
        Self {
            channel_mask: 0,
            minimum_severity: LogSeverity::Info,
        }
    }
}

impl LogFilter {
    pub fn new(channels: BTreeSet<LogChannel>, minimum_severity: LogSeverity) -> Self {
        Self {
            channel_mask: log_channel_mask(&channels),
            minimum_severity,
        }
    }

    pub fn matches(&self, entry: &LogEntry) -> bool {
        entry.severity() >= self.minimum_severity
            && log_channel_allowed(self.channel_mask, entry.source().channel())
    }
}

const fn log_channel_bit(channel: LogChannel) -> u8 {
    match channel {
        LogChannel::Editor => 1 << 0,
        LogChannel::Runtime => 1 << 1,
        LogChannel::Play => 1 << 2,
        LogChannel::Plugin => 1 << 3,
        LogChannel::Import => 1 << 4,
        LogChannel::ScriptBuild => 1 << 5,
    }
}

fn log_channel_mask(channels: &BTreeSet<LogChannel>) -> u8 {
    channels
        .iter()
        .fold(0, |mask, channel| mask | log_channel_bit(*channel))
}

const fn log_channel_allowed(channel_mask: u8, channel: LogChannel) -> bool {
    channel_mask == 0 || channel_mask & log_channel_bit(channel) != 0
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::super::LogSource;
    use super::*;

    const CHANNEL_LOOKUP_COUNT: usize = 1_048_576;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn channel_lookups() -> Vec<LogChannel> {
        let channels = [
            LogChannel::Editor,
            LogChannel::Runtime,
            LogChannel::Play,
            LogChannel::Plugin,
            LogChannel::Import,
            LogChannel::ScriptBuild,
        ];
        (0..CHANNEL_LOOKUP_COUNT)
            .map(|index| channels[(index * 5) % channels.len()])
            .collect()
    }

    fn ordered_channel_match_count(
        channels: &BTreeSet<LogChannel>,
        lookups: &[LogChannel],
    ) -> usize {
        lookups
            .iter()
            .filter(|channel| channels.contains(*channel))
            .count()
    }

    fn bitmask_channel_match_count(channel_mask: u8, lookups: &[LogChannel]) -> usize {
        lookups
            .iter()
            .filter(|channel| log_channel_allowed(channel_mask, **channel))
            .count()
    }

    #[test]
    fn optimization_batch_20260826v_editor11_log_filter_preserves_channel_and_severity_rules() {
        let filter = LogFilter::new(BTreeSet::from([LogChannel::Runtime]), LogSeverity::Warning);
        let runtime_warning = LogEntry::new(
            LogSource::runtime(),
            LogSeverity::Warning,
            "runtime warning",
            1,
            None,
        )
        .unwrap();
        let runtime_info = LogEntry::new(
            LogSource::runtime(),
            LogSeverity::Info,
            "runtime info",
            2,
            None,
        )
        .unwrap();
        let editor_warning = LogEntry::new(
            LogSource::editor(),
            LogSeverity::Warning,
            "editor warning",
            3,
            None,
        )
        .unwrap();

        assert!(filter.matches(&runtime_warning));
        assert!(!filter.matches(&runtime_info));
        assert!(!filter.matches(&editor_warning));
        assert!(LogFilter::default().matches(&editor_warning));
    }

    #[test]
    fn optimization_batch_20260826v_editor11_log_filter_uses_channel_bitmask() {
        let source = include_str!("filter.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("channel_mask: u8"));
        assert!(production.contains("fn log_channel_bit("));
        assert!(production.contains("fn log_channel_allowed("));
        assert!(production.contains("channel_mask & log_channel_bit(channel) != 0"));
        assert!(!production.contains("channels.contains"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260826v_editor11_log_channel_bitmask_performance_evidence() {
        let channels = BTreeSet::from([
            LogChannel::Runtime,
            LogChannel::Plugin,
            LogChannel::ScriptBuild,
        ]);
        let channel_mask = log_channel_mask(&channels);
        let lookups = channel_lookups();
        assert_eq!(
            ordered_channel_match_count(&channels, &lookups),
            bitmask_channel_match_count(channel_mask, &lookups)
        );

        let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut bitmask_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(ordered_channel_match_count(
                    black_box(&channels),
                    black_box(&lookups),
                ));
                ordered_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(bitmask_channel_match_count(
                    black_box(channel_mask),
                    black_box(&lookups),
                ));
                bitmask_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(bitmask_channel_match_count(
                    black_box(channel_mask),
                    black_box(&lookups),
                ));
                bitmask_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(ordered_channel_match_count(
                    black_box(&channels),
                    black_box(&lookups),
                ));
                ordered_samples.push(started.elapsed());
            }
        }

        let ordered_p95 = percentile_95(&mut ordered_samples);
        let bitmask_p95 = percentile_95(&mut bitmask_samples);
        println!(
            "EDITOR11_LOG_CHANNEL_BITMASK_BENCH_V1 channel_count=6 lookups={CHANNEL_LOOKUP_COUNT} \
             ordered_lookup_class=log_n bitmask_operations_per_lookup=2 \
             ordered_p95_ns={} bitmask_p95_ns={}",
            ordered_p95.as_nanos(),
            bitmask_p95.as_nanos(),
        );
        assert!(
            bitmask_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 35,
            "bitmask P95 {:?} exceeded 35% of ordered-set P95 {:?}",
            bitmask_p95,
            ordered_p95,
        );
    }
}

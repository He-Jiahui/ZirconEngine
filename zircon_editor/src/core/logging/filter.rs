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

    pub(crate) const fn from_channel(
        channel: Option<LogChannel>,
        minimum_severity: LogSeverity,
    ) -> Self {
        Self {
            channel_mask: match channel {
                Some(channel) => log_channel_bit(channel),
                None => 0,
            },
            minimum_severity,
        }
    }

    pub(crate) const fn is_unfiltered(&self) -> bool {
        self.channel_mask == 0 && matches!(self.minimum_severity, LogSeverity::Info)
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
#[path = "tests/filter_optimization_tests.rs"]
mod optimization_tests;

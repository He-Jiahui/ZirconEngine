use std::collections::BTreeSet;

use super::{LogChannel, LogEntry, LogSeverity};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogFilter {
    channels: BTreeSet<LogChannel>,
    minimum_severity: LogSeverity,
}

impl Default for LogFilter {
    fn default() -> Self {
        Self {
            channels: BTreeSet::new(),
            minimum_severity: LogSeverity::Info,
        }
    }
}

impl LogFilter {
    pub fn new(channels: BTreeSet<LogChannel>, minimum_severity: LogSeverity) -> Self {
        Self {
            channels,
            minimum_severity,
        }
    }

    pub fn matches(&self, entry: &LogEntry) -> bool {
        entry.severity() >= self.minimum_severity
            && (self.channels.is_empty() || self.channels.contains(&entry.source().channel()))
    }
}

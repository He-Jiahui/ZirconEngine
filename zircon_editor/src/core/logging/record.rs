use super::LogEntry;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogRecord {
    sequence: u64,
    entry: LogEntry,
}

impl LogRecord {
    pub(super) fn new(sequence: u64, entry: LogEntry) -> Self {
        Self { sequence, entry }
    }

    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn entry(&self) -> &LogEntry {
        &self.entry
    }

    pub(super) fn format_line(&self) -> String {
        let source = escape_line(&self.entry.source().to_string());
        let message = escape_line(self.entry.message());
        let jump = self
            .entry
            .jump()
            .map(|jump| escape_line(&jump.to_string()))
            .unwrap_or_else(|| "none".to_owned());
        format!(
            "sequence={} frame={} severity={:?} source={} jump={} message={}\n",
            self.sequence,
            self.entry.timestamp_frame(),
            self.entry.severity(),
            source,
            jump,
            message,
        )
    }
}

fn escape_line(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

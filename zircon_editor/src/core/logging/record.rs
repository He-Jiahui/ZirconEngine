use super::LogEntry;

#[derive(Clone, Debug, PartialEq, Eq)]
/// 权威存储分配序号后的日志记录；UI 跳转和磁盘文件都以此序号定位。
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

    /// 滚动文件每条记录只占一行，正文中的换行须转义以保留记录边界。
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
    let escaped_capacity = value.len().saturating_add(
        value
            .bytes()
            .filter(|byte| matches!(byte, b'\\' | b'\r' | b'\n'))
            .count(),
    );
    let mut escaped = String::with_capacity(escaped_capacity);
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '\r' => escaped.push_str("\\r"),
            '\n' => escaped.push_str("\\n"),
            character => escaped.push(character),
        }
    }
    escaped
}

#[cfg(test)]
#[path = "record/tests/single_pass_escape_tests.rs"]
mod single_pass_escape_tests;

use std::collections::VecDeque;

const MAX_HOST_WINDOW_DIAGNOSTICS: usize = 64;
const MAX_HOST_WINDOW_DIAGNOSTIC_BYTES: usize = 8 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HostWindowDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HostWindowDiagnostic {
    severity: HostWindowDiagnosticSeverity,
    message: String,
}

impl HostWindowDiagnostic {
    pub(crate) fn new(severity: HostWindowDiagnosticSeverity, message: impl Into<String>) -> Self {
        Self {
            severity,
            message: message.into(),
        }
    }

    pub(crate) const fn severity(&self) -> HostWindowDiagnosticSeverity {
        self.severity
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }

    fn retained_bytes(&self) -> usize {
        self.message.len()
    }
}

#[derive(Default)]
pub(crate) struct HostWindowDiagnosticQueue {
    entries: VecDeque<HostWindowDiagnostic>,
    retained_bytes: usize,
    dropped_entries: u64,
}

impl HostWindowDiagnosticQueue {
    pub(crate) fn push(&mut self, diagnostic: HostWindowDiagnostic) {
        let diagnostic = self.bounded_diagnostic(diagnostic);
        let retained_bytes = diagnostic.retained_bytes();
        while self.entries.len() >= MAX_HOST_WINDOW_DIAGNOSTICS
            || self.retained_bytes.saturating_add(retained_bytes) > MAX_HOST_WINDOW_DIAGNOSTIC_BYTES
        {
            let Some(dropped) = self.entries.pop_front() else {
                break;
            };
            self.retained_bytes = self.retained_bytes.saturating_sub(dropped.retained_bytes());
            self.dropped_entries = self.dropped_entries.saturating_add(1);
        }
        self.retained_bytes = self.retained_bytes.saturating_add(retained_bytes);
        self.entries.push_back(diagnostic);
    }

    pub(crate) fn drain(&mut self) -> Vec<HostWindowDiagnostic> {
        let mut diagnostics = self.entries.drain(..).collect::<Vec<_>>();
        self.retained_bytes = 0;
        if self.dropped_entries != 0 {
            diagnostics.push(HostWindowDiagnostic::new(
                HostWindowDiagnosticSeverity::Warning,
                format!(
                    "editor_host_window diagnostics_dropped={}",
                    self.dropped_entries
                ),
            ));
            self.dropped_entries = 0;
        }
        diagnostics
    }

    fn bounded_diagnostic(&self, diagnostic: HostWindowDiagnostic) -> HostWindowDiagnostic {
        if diagnostic.retained_bytes() <= MAX_HOST_WINDOW_DIAGNOSTIC_BYTES {
            diagnostic
        } else {
            HostWindowDiagnostic::new(
                diagnostic.severity(),
                "editor_host_window diagnostic exceeds the bounded queue limit.",
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HostWindowDiagnostic, HostWindowDiagnosticQueue, HostWindowDiagnosticSeverity};

    #[test]
    fn oversized_window_diagnostic_uses_a_bounded_fallback_without_losing_severity() {
        let mut queue = HostWindowDiagnosticQueue::default();

        queue.push(HostWindowDiagnostic::new(
            HostWindowDiagnosticSeverity::Error,
            "x".repeat(9 * 1024),
        ));

        let diagnostics = queue.drain();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].severity(),
            HostWindowDiagnosticSeverity::Error
        );
        assert_eq!(
            diagnostics[0].message(),
            "editor_host_window diagnostic exceeds the bounded queue limit."
        );
    }

    #[test]
    fn queue_evicts_oldest_entry_when_the_byte_budget_would_be_exceeded() {
        let mut queue = HostWindowDiagnosticQueue::default();
        let oldest = "a".repeat(6 * 1024);
        let latest = "b".repeat(6 * 1024);

        queue.push(HostWindowDiagnostic::new(
            HostWindowDiagnosticSeverity::Info,
            oldest,
        ));
        queue.push(HostWindowDiagnostic::new(
            HostWindowDiagnosticSeverity::Warning,
            latest.clone(),
        ));

        let diagnostics = queue.drain();
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].message(), latest.as_str());
        assert_eq!(
            diagnostics[1].message(),
            "editor_host_window diagnostics_dropped=1"
        );
    }

    #[test]
    fn queue_reports_eviction_without_retaining_unbounded_window_diagnostics() {
        let mut queue = HostWindowDiagnosticQueue::default();
        for index in 0..65 {
            queue.push(HostWindowDiagnostic::new(
                HostWindowDiagnosticSeverity::Info,
                format!("native window diagnostic {index}"),
            ));
        }

        let diagnostics = queue.drain();
        assert_eq!(diagnostics.len(), 65);
        assert_eq!(diagnostics[0].message(), "native window diagnostic 1");
        assert_eq!(
            diagnostics
                .last()
                .expect("drop report should be present")
                .message(),
            "editor_host_window diagnostics_dropped=1"
        );
        assert_eq!(
            diagnostics
                .last()
                .expect("drop report should be present")
                .severity(),
            HostWindowDiagnosticSeverity::Warning
        );
    }
}

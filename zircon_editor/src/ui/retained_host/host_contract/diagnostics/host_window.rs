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
        let dropped_entries = std::mem::take(&mut self.dropped_entries);
        let mut diagnostics = Vec::with_capacity(
            self.entries
                .len()
                .saturating_add(usize::from(dropped_entries != 0)),
        );
        diagnostics.extend(self.entries.drain(..));
        self.retained_bytes = 0;
        if dropped_entries != 0 {
            diagnostics.push(HostWindowDiagnostic::new(
                HostWindowDiagnosticSeverity::Warning,
                format!("editor_host_window diagnostics_dropped={dropped_entries}"),
            ));
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
#[path = "tests/host_window.rs"]
mod tests;

#[cfg(test)]
#[path = "host_window/tests/drain_capacity_tests.rs"]
mod drain_capacity_tests;

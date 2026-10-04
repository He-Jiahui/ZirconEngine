use std::process::Child;

use crate::core::process::terminate_process_tree;

/// Ensures an unwinding export job cannot detach a still-running child process.
pub(in crate::ui::host) struct ExportProcessChildGuard {
    child: Child,
    label: String,
    armed: bool,
}

impl ExportProcessChildGuard {
    pub(in crate::ui::host) fn new(child: Child, label: impl Into<String>) -> Self {
        Self {
            child,
            label: label.into(),
            armed: true,
        }
    }

    pub(in crate::ui::host) fn child_mut(&mut self) -> &mut Child {
        &mut self.child
    }

    pub(in crate::ui::host) fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for ExportProcessChildGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        if self.child.try_wait().is_ok_and(|status| status.is_some()) {
            return;
        }
        let termination = terminate_process_tree(&mut self.child, &self.label);
        if termination.succeeded {
            let _ = self.child.wait();
        }
    }
}

#[cfg(all(test, any(windows, unix)))]
#[path = "tests/child_guard.rs"]
mod tests;

use serde::Serialize;

use super::{ProductExitClass, ProductProcessExitCode};

pub const PRODUCT_TERMINAL_RECEIPT_SCHEMA_VERSION: u8 = 1;

/// A host code is interpreted through the registry. A commandlet owns its raw u8 code,
/// even when that number is also assigned to a host class.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum ProductTerminalPrimary {
    Host {
        class: ProductExitClass,
        reason: &'static str,
    },
    Commandlet {
        code: u8,
    },
}

impl ProductTerminalPrimary {
    pub const fn exit_code(self) -> ProductProcessExitCode {
        match self {
            Self::Host { class, .. } => ProductProcessExitCode::from_class(class),
            Self::Commandlet { code } => ProductProcessExitCode::from_code(code),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductTerminalSecondary {
    ShutdownIncomplete,
    ReportWriteFailed,
    IpcWriteFailed,
    ProfilingFlushFailed,
    DiagnosticLogShutdownFailed,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductTerminalStatus {
    #[default]
    Unknown,
    Completed,
    Failed,
}

/// One process terminal projection. A later teardown failure never changes the runner's
/// primary result; it only changes a successful process projection to shutdown failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductTerminalOutcome {
    primary: ProductTerminalPrimary,
    secondary: Vec<ProductTerminalSecondary>,
    attempt: Option<u64>,
    generation: Option<u64>,
    shutdown_durability: ProductTerminalStatus,
    report_write: ProductTerminalStatus,
    ipc_write: ProductTerminalStatus,
    profiling_flush: ProductTerminalStatus,
    diagnostic_log_shutdown: ProductTerminalStatus,
}

impl ProductTerminalOutcome {
    pub fn host(class: ProductExitClass, reason: &'static str) -> Self {
        Self::new(ProductTerminalPrimary::Host { class, reason })
    }

    pub fn commandlet(code: u8) -> Self {
        Self::new(ProductTerminalPrimary::Commandlet { code })
    }

    fn new(primary: ProductTerminalPrimary) -> Self {
        Self {
            primary,
            secondary: Vec::with_capacity(5),
            attempt: None,
            generation: None,
            shutdown_durability: ProductTerminalStatus::Unknown,
            report_write: ProductTerminalStatus::Unknown,
            ipc_write: ProductTerminalStatus::Unknown,
            profiling_flush: ProductTerminalStatus::Unknown,
            diagnostic_log_shutdown: ProductTerminalStatus::Unknown,
        }
    }

    pub fn with_identity(mut self, attempt: u64, generation: Option<u64>) -> Self {
        self.attempt = Some(attempt);
        self.generation = generation;
        self
    }

    pub const fn primary(&self) -> ProductTerminalPrimary {
        self.primary
    }

    pub fn secondary(&self) -> &[ProductTerminalSecondary] {
        &self.secondary
    }

    pub fn exit_code(&self) -> ProductProcessExitCode {
        let primary = self.primary.exit_code();
        if primary.is_failure() {
            primary
        } else if self.secondary.is_empty() {
            ProductProcessExitCode::Success
        } else {
            ProductProcessExitCode::from_class(ProductExitClass::ShutdownFailure)
        }
    }

    pub fn observe_shutdown_durability(&mut self, completed: bool) {
        self.shutdown_durability = observed_status(self.shutdown_durability, completed);
        if !completed {
            self.record_secondary(ProductTerminalSecondary::ShutdownIncomplete);
        }
    }

    pub fn observe_report_write(&mut self, completed: bool) {
        self.report_write = observed_status(self.report_write, completed);
        if !completed {
            self.record_secondary(ProductTerminalSecondary::ReportWriteFailed);
        }
    }

    pub fn observe_ipc_write(&mut self, completed: bool) {
        self.ipc_write = observed_status(self.ipc_write, completed);
        if !completed {
            self.record_secondary(ProductTerminalSecondary::IpcWriteFailed);
        }
    }

    pub fn observe_profiling_flush(&mut self, completed: bool) {
        self.profiling_flush = observed_status(self.profiling_flush, completed);
        if !completed {
            self.record_secondary(ProductTerminalSecondary::ProfilingFlushFailed);
        }
    }

    pub fn observe_diagnostic_log_shutdown(&mut self, completed: bool) {
        self.diagnostic_log_shutdown = observed_status(self.diagnostic_log_shutdown, completed);
        if !completed {
            self.record_secondary(ProductTerminalSecondary::DiagnosticLogShutdownFailed);
        }
    }

    fn record_secondary(&mut self, failure: ProductTerminalSecondary) {
        if !self.secondary.contains(&failure) {
            self.secondary.push(failure);
        }
    }

    pub fn receipt(&self) -> ProductTerminalReceipt<'_> {
        ProductTerminalReceipt {
            schema_version: PRODUCT_TERMINAL_RECEIPT_SCHEMA_VERSION,
            attempt: self.attempt,
            generation: self.generation,
            primary: self.primary,
            primary_exit_code: self.primary.exit_code().code(),
            process_exit_code: self.exit_code().code(),
            secondary: &self.secondary,
            shutdown_durability: self.shutdown_durability,
            report_write: self.report_write,
            ipc_write: self.ipc_write,
            profiling_flush: self.profiling_flush,
            diagnostic_log_shutdown: self.diagnostic_log_shutdown,
        }
    }
}

const fn observed_status(
    previous: ProductTerminalStatus,
    completed: bool,
) -> ProductTerminalStatus {
    match previous {
        ProductTerminalStatus::Failed => ProductTerminalStatus::Failed,
        _ if completed => ProductTerminalStatus::Completed,
        _ => ProductTerminalStatus::Failed,
    }
}

/// Versioned machine-readable projection. Missing identity or status means the producer did not
/// observe it; a serialized receipt alone does not prove durable delivery.
#[derive(Debug, Serialize)]
pub struct ProductTerminalReceipt<'a> {
    schema_version: u8,
    attempt: Option<u64>,
    generation: Option<u64>,
    primary: ProductTerminalPrimary,
    primary_exit_code: u8,
    process_exit_code: u8,
    secondary: &'a [ProductTerminalSecondary],
    shutdown_durability: ProductTerminalStatus,
    report_write: ProductTerminalStatus,
    ipc_write: ProductTerminalStatus,
    profiling_flush: ProductTerminalStatus,
    diagnostic_log_shutdown: ProductTerminalStatus,
}

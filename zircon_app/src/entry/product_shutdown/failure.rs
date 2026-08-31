use std::fmt::{self, Display, Formatter};

use super::ProductHostPhase;

/// Operational severity retained by the product failure ledger.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProductFailureSeverity {
    Recoverable,
    Terminal,
    Emergency,
}

impl ProductFailureSeverity {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Recoverable => "recoverable",
            Self::Terminal => "terminal",
            Self::Emergency => "emergency",
        }
    }
}

/// One ordered, bounded product failure observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProductFailureRecord {
    sequence: u64,
    phase: ProductHostPhase,
    severity: ProductFailureSeverity,
    owner: &'static str,
    message: String,
}

impl ProductFailureRecord {
    pub(super) fn new(
        sequence: u64,
        phase: ProductHostPhase,
        severity: ProductFailureSeverity,
        owner: &'static str,
        message: String,
    ) -> Self {
        Self {
            sequence,
            phase,
            severity,
            owner,
            message,
        }
    }

    pub(crate) const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub(crate) const fn phase(&self) -> ProductHostPhase {
        self.phase
    }

    pub(crate) const fn severity(&self) -> ProductFailureSeverity {
        self.severity
    }

    pub(crate) const fn owner(&self) -> &'static str {
        self.owner
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

impl Display for ProductFailureRecord {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "sequence={} phase={} severity={} owner={} message={}",
            self.sequence,
            self.phase.as_str(),
            self.severity.as_str(),
            self.owner,
            self.message
        )
    }
}

/// Immutable terminal snapshot of the bounded failure ledger.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ProductFailureReport {
    records: Vec<ProductFailureRecord>,
    suppressed_count: u64,
}

impl ProductFailureReport {
    pub(super) fn new(records: Vec<ProductFailureRecord>, suppressed_count: u64) -> Self {
        Self {
            records,
            suppressed_count,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub(crate) fn records(&self) -> &[ProductFailureRecord] {
        &self.records
    }

    pub(crate) fn primary(&self) -> Option<&ProductFailureRecord> {
        self.records.first()
    }

    pub(crate) fn secondary(&self) -> &[ProductFailureRecord] {
        self.records.get(1..).unwrap_or_default()
    }

    pub(crate) const fn suppressed_count(&self) -> u64 {
        self.suppressed_count
    }
}

impl Display for ProductFailureReport {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "recorded={} suppressed={}",
            self.records.len(),
            self.suppressed_count
        )?;
        for record in &self.records {
            write!(formatter, " | {record}")?;
        }
        Ok(())
    }
}

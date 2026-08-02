use crate::core::framework::render::{
    ShaderVariantPrewarmExecutionBudget, ShaderVariantPrewarmExecutionBudgetError,
    ShaderVariantPrewarmManifest, ShaderVariantPrewarmReport,
};
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(super) enum ShaderPrewarmExecutionPreflightError {
    #[error(transparent)]
    InvalidBudget(#[from] ShaderVariantPrewarmExecutionBudgetError),
    #[error("shader prewarm source table resident byte count overflowed")]
    SourceTableResidentBytesOverflow,
    #[error(
        "shader prewarm source table requires {requested_bytes} resident bytes; budget is {max_bytes}"
    )]
    SourceTableBudgetExceeded {
        requested_bytes: usize,
        max_bytes: usize,
    },
}

pub(super) fn preflight_execution_budget(
    manifest: &ShaderVariantPrewarmManifest,
    budget: ShaderVariantPrewarmExecutionBudget,
) -> Result<(), ShaderPrewarmExecutionPreflightError> {
    budget.validate()?;
    let resident_source_bytes = manifest
        .source_table_resident_bytes()
        .ok_or(ShaderPrewarmExecutionPreflightError::SourceTableResidentBytesOverflow)?;
    if resident_source_bytes > budget.max_resident_source_bytes {
        return Err(
            ShaderPrewarmExecutionPreflightError::SourceTableBudgetExceeded {
                requested_bytes: resident_source_bytes,
                max_bytes: budget.max_resident_source_bytes,
            },
        );
    }
    Ok(())
}

pub(super) fn execution_budget_preflight_failure_report(
    manifest: &ShaderVariantPrewarmManifest,
    budget: ShaderVariantPrewarmExecutionBudget,
    error: impl std::fmt::Display,
) -> ShaderVariantPrewarmReport {
    let mut report = ShaderVariantPrewarmReport::default();
    report.execution_budget.configure(budget);
    if let Some(resident_source_bytes) = manifest.source_table_resident_bytes() {
        report
            .execution_budget
            .record_source_residency(resident_source_bytes);
    }
    report.execution_budget.record_rejected();
    report.record_preflight_error(error.to_string());
    report
}

pub(super) fn with_execution_budget(
    mut report: ShaderVariantPrewarmReport,
    manifest: &ShaderVariantPrewarmManifest,
    budget: ShaderVariantPrewarmExecutionBudget,
) -> ShaderVariantPrewarmReport {
    report.execution_budget.configure(budget);
    if let Some(resident_source_bytes) = manifest.source_table_resident_bytes() {
        report
            .execution_budget
            .record_source_residency(resident_source_bytes);
    }
    report
}

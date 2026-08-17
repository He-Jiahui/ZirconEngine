use zircon_runtime_host::foreign_output::RuntimeOwnedOutputReleaser;
use zircon_runtime_interface::{ZrOwnedResultV2, ZrStatus};

use super::{ensure_status, RuntimeLibraryError};

pub(super) fn release_owned_result(
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
    operation: &'static str,
) -> Result<(), RuntimeLibraryError> {
    zircon_runtime_host::foreign_output::release_owned_result(output, releaser, operation)
        .map_err(Into::into)
}

pub(super) fn release_owned_result_after_error<T>(
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
    error: RuntimeLibraryError,
    release_operation: &'static str,
) -> Result<T, RuntimeLibraryError> {
    match release_owned_result(output, releaser, release_operation) {
        Ok(()) => Err(error),
        Err(release_error) => Err(error.with_cleanup_failure(&release_error)),
    }
}

#[cfg(test)]
pub(super) fn release_owned_result_after_result<T>(
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
    result: Result<T, RuntimeLibraryError>,
    release_operation: &'static str,
) -> Result<T, RuntimeLibraryError> {
    match (
        result,
        release_owned_result(output, releaser, release_operation),
    ) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(release_error)) => Err(release_error),
        (Err(error), Err(release_error)) => Err(error.with_cleanup_failure(&release_error)),
    }
}

pub(super) fn validate_owned_result_releasing_on_error(
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
    operation: &'static str,
    release_operation: &'static str,
) -> Result<ZrOwnedResultV2, RuntimeLibraryError> {
    zircon_runtime_host::foreign_output::validate_owned_result_releasing_on_error(
        output,
        releaser,
        operation,
        release_operation,
    )
    .map_err(Into::into)
}

pub(super) fn ensure_status_releasing_output_on_error(
    status: ZrStatus,
    operation: &'static str,
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
    release_operation: &'static str,
) -> Result<(), RuntimeLibraryError> {
    let Err(error) = ensure_status(status, operation) else {
        return Ok(());
    };
    release_owned_result_after_error(output, releaser, error, release_operation)
}

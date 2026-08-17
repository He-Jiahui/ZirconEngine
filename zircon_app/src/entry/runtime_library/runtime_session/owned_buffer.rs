use zircon_runtime_interface::{ZrOwnedByteBuffer, ZrStatus};

use super::{ensure_status, RuntimeLibraryError};

pub(super) fn release_owned_buffer(
    output: ZrOwnedByteBuffer,
    operation: &'static str,
) -> Result<(), RuntimeLibraryError> {
    zircon_runtime_host::foreign_output::release_owned_buffer(output, operation).map_err(Into::into)
}

pub(super) fn release_owned_buffer_after_error<T>(
    output: ZrOwnedByteBuffer,
    error: RuntimeLibraryError,
    release_operation: &'static str,
) -> Result<T, RuntimeLibraryError> {
    match release_owned_buffer(output, release_operation) {
        Ok(()) => Err(error),
        Err(release_error) => Err(error.with_cleanup_failure(&release_error)),
    }
}

#[cfg(test)]
pub(super) fn release_owned_buffer_after_result<T>(
    output: ZrOwnedByteBuffer,
    result: Result<T, RuntimeLibraryError>,
    release_operation: &'static str,
) -> Result<T, RuntimeLibraryError> {
    match (result, release_owned_buffer(output, release_operation)) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(release_error)) => Err(release_error),
        (Err(error), Err(release_error)) => Err(error.with_cleanup_failure(&release_error)),
    }
}

pub(super) fn validate_owned_buffer(
    output: &ZrOwnedByteBuffer,
    operation: &'static str,
) -> Result<(), RuntimeLibraryError> {
    zircon_runtime_host::foreign_output::validate_owned_buffer(output, operation)
        .map_err(Into::into)
}

pub(super) fn validate_owned_buffer_releasing_on_error(
    output: ZrOwnedByteBuffer,
    operation: &'static str,
    release_operation: &'static str,
) -> Result<(), RuntimeLibraryError> {
    match validate_owned_buffer(&output, operation) {
        Ok(()) => Ok(()),
        Err(error) => release_owned_buffer_after_error(output, error, release_operation),
    }
}

pub(super) fn ensure_status_releasing_output_on_error(
    status: ZrStatus,
    operation: &'static str,
    output: ZrOwnedByteBuffer,
    release_operation: &'static str,
) -> Result<(), RuntimeLibraryError> {
    let Err(error) = ensure_status(status, operation) else {
        return Ok(());
    };
    release_owned_buffer_after_error(output, error, release_operation)
}

use zircon_runtime_host::foreign_output::RuntimeOwnedOutputReleaser;
use zircon_runtime_interface::{ZrOwnedResultV2, ZrStatus};

use super::{ensure_status, RuntimeLibraryError};

/// # Safety
///
/// `output` must originate from the live runtime session bound to `releaser`, and this call must
/// hold its unique release authority.
pub(super) unsafe fn release_owned_result(
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
    operation: &'static str,
) -> Result<(), RuntimeLibraryError> {
    unsafe {
        zircon_runtime_host::foreign_output::release_owned_result(output, releaser, operation)
    }
    .map_err(Into::into)
}

/// # Safety
///
/// The safety requirements of [`release_owned_result`] apply.
pub(super) unsafe fn release_owned_result_after_error<T>(
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
    error: RuntimeLibraryError,
    release_operation: &'static str,
) -> Result<T, RuntimeLibraryError> {
    match unsafe { release_owned_result(output, releaser, release_operation) } {
        Ok(()) => Err(error),
        Err(release_error) => Err(error.with_cleanup_failure(&release_error)),
    }
}

#[cfg(test)]
/// # Safety
///
/// The safety requirements of [`release_owned_result`] apply.
pub(super) unsafe fn release_owned_result_after_result<T>(
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
    result: Result<T, RuntimeLibraryError>,
    release_operation: &'static str,
) -> Result<T, RuntimeLibraryError> {
    match (result, unsafe {
        release_owned_result(output, releaser, release_operation)
    }) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(release_error)) => Err(release_error),
        (Err(error), Err(release_error)) => Err(error.with_cleanup_failure(&release_error)),
    }
}

/// # Safety
///
/// If validation fails, the safety requirements of [`release_owned_result`] apply.
pub(super) unsafe fn validate_owned_result_releasing_on_error(
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
    operation: &'static str,
    release_operation: &'static str,
) -> Result<ZrOwnedResultV2, RuntimeLibraryError> {
    unsafe {
        zircon_runtime_host::foreign_output::validate_owned_result_releasing_on_error(
            output,
            releaser,
            operation,
            release_operation,
        )
    }
    .map_err(Into::into)
}

/// # Safety
///
/// `status` and `output` must originate from the live runtime session bound to `releaser`.
pub(super) unsafe fn ensure_status_releasing_output_on_error(
    status: ZrStatus,
    operation: &'static str,
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
    release_operation: &'static str,
) -> Result<(), RuntimeLibraryError> {
    let Err(error) = ensure_status(status, operation) else {
        return Ok(());
    };
    unsafe { release_owned_result_after_error(output, releaser, error, release_operation) }
}

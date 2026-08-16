use zircon_runtime_interface::{ZrOwnedByteBuffer, ZrStatus};

use super::{ensure_status, RuntimeLibraryError};

pub(super) fn release_owned_buffer(
    output: ZrOwnedByteBuffer,
    operation: &'static str,
) -> Result<(), RuntimeLibraryError> {
    let Some(free) = output.free else {
        return Ok(());
    };
    ensure_status(unsafe { free(output) }, operation)
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
    if output.len > output.capacity {
        return Err(RuntimeLibraryError::new(format!(
            "{operation} returned malformed storage: len {} exceeds capacity {}",
            output.len, output.capacity
        )));
    }
    if output.len > isize::MAX as usize || output.capacity > isize::MAX as usize {
        return Err(RuntimeLibraryError::new(format!(
            "{operation} returned malformed storage: len {} and capacity {} exceed the maximum Rust slice allocation",
            output.len, output.capacity
        )));
    }
    if output.data.is_null() {
        return if output.len == 0 && output.capacity == 0 {
            Ok(())
        } else {
            Err(RuntimeLibraryError::new(format!(
                "{operation} returned malformed storage: null data with len {} and capacity {}",
                output.len, output.capacity
            )))
        };
    }
    if output.free.is_none() {
        return Err(RuntimeLibraryError::new(format!(
            "{operation} returned owned storage without a free callback"
        )));
    }
    Ok(())
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

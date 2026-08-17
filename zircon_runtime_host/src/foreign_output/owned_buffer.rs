//! Validation and release helpers for runtime-owned ABI buffers.

use zircon_runtime_interface::ZrOwnedByteBuffer;

use super::RuntimeForeignOutputError;

pub fn release_owned_buffer(
    output: ZrOwnedByteBuffer,
    operation: &'static str,
) -> Result<(), RuntimeForeignOutputError> {
    let Some(free) = output.free else {
        return Ok(());
    };
    match RuntimeForeignOutputError::from_status(unsafe { free(output) }, operation) {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

pub fn release_owned_buffer_after_error<T>(
    output: ZrOwnedByteBuffer,
    error: RuntimeForeignOutputError,
    release_operation: &'static str,
) -> Result<T, RuntimeForeignOutputError> {
    match release_owned_buffer(output, release_operation) {
        Ok(()) => Err(error),
        Err(release_error) => Err(error.with_cleanup_failure(&release_error)),
    }
}

pub fn release_owned_buffer_after_result<T>(
    output: ZrOwnedByteBuffer,
    result: Result<T, RuntimeForeignOutputError>,
    release_operation: &'static str,
) -> Result<T, RuntimeForeignOutputError> {
    match (result, release_owned_buffer(output, release_operation)) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(release_error)) => Err(release_error),
        (Err(error), Err(release_error)) => Err(error.with_cleanup_failure(&release_error)),
    }
}

pub fn validate_owned_buffer(
    output: &ZrOwnedByteBuffer,
    operation: &'static str,
) -> Result<(), RuntimeForeignOutputError> {
    if output.len > output.capacity {
        return Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned malformed storage: len {} exceeds capacity {}",
            output.len, output.capacity
        )));
    }
    if output.len > isize::MAX as usize || output.capacity > isize::MAX as usize {
        return Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned malformed storage: len {} and capacity {} exceed the maximum Rust slice allocation",
            output.len, output.capacity
        )));
    }
    if output.data.is_null() {
        return if output.len == 0 && output.capacity == 0 {
            Ok(())
        } else {
            Err(RuntimeForeignOutputError::protocol_violation(format!(
                "{operation} returned malformed storage: null data with len {} and capacity {}",
                output.len, output.capacity
            )))
        };
    }
    if output.free.is_none() {
        return Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned owned storage without a free callback"
        )));
    }
    Ok(())
}

pub fn validate_owned_buffer_releasing_on_error(
    output: ZrOwnedByteBuffer,
    operation: &'static str,
    release_operation: &'static str,
) -> Result<(), RuntimeForeignOutputError> {
    match validate_owned_buffer(&output, operation) {
        Ok(()) => Ok(()),
        Err(error) => release_owned_buffer_after_error(output, error, release_operation),
    }
}

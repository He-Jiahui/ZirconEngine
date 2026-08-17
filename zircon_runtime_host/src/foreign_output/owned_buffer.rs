//! Validation and release helpers for runtime-owned ABI results.

use zircon_runtime_interface::{
    ZrOwnedResultV2, ZrRuntimeAllocationId, ZrRuntimeReleaseAllocationFnV2, ZrRuntimeSessionHandle,
    ZrStatus,
};

use super::RuntimeForeignOutputError;

#[derive(Clone, Copy)]
pub struct RuntimeOwnedOutputReleaser {
    session: ZrRuntimeSessionHandle,
    release: ZrRuntimeReleaseAllocationFnV2,
}

impl RuntimeOwnedOutputReleaser {
    pub const fn new(
        session: ZrRuntimeSessionHandle,
        release: ZrRuntimeReleaseAllocationFnV2,
    ) -> Self {
        Self { session, release }
    }

    pub const fn session(self) -> ZrRuntimeSessionHandle {
        self.session
    }

    fn release(self, allocation: ZrRuntimeAllocationId) -> ZrStatus {
        unsafe { (self.release)(self.session, allocation) }
    }
}

pub fn release_owned_result(
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
    operation: &'static str,
) -> Result<(), RuntimeForeignOutputError> {
    if output.is_empty() {
        return Ok(());
    }
    if !output.allocation.is_valid() {
        return Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned runtime-owned storage without an allocation ID"
        )));
    }
    match RuntimeForeignOutputError::from_status(releaser.release(output.allocation), operation) {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

pub fn release_owned_result_after_error<T>(
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
    error: RuntimeForeignOutputError,
    release_operation: &'static str,
) -> Result<T, RuntimeForeignOutputError> {
    match release_owned_result(output, releaser, release_operation) {
        Ok(()) => Err(error),
        Err(release_error) => Err(error.with_cleanup_failure(&release_error)),
    }
}

pub fn release_owned_result_after_result<T>(
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
    result: Result<T, RuntimeForeignOutputError>,
    release_operation: &'static str,
) -> Result<T, RuntimeForeignOutputError> {
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

pub fn validate_owned_result(
    output: &ZrOwnedResultV2,
    operation: &'static str,
) -> Result<usize, RuntimeForeignOutputError> {
    let len = usize::try_from(output.len).map_err(|_| {
        RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned a length that exceeds the host address space"
        ))
    })?;
    if len > isize::MAX as usize {
        return Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned length {len} above the maximum Rust slice length"
        )));
    }
    if output.data.is_null() {
        return if len == 0 && !output.allocation.is_valid() {
            Ok(0)
        } else {
            Err(RuntimeForeignOutputError::protocol_violation(format!(
                "{operation} returned null data with len {} and allocation {}",
                output.len,
                output.allocation.raw()
            )))
        };
    }
    if len == 0 {
        return Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned non-null data for an empty payload"
        )));
    }
    if !output.allocation.is_valid() {
        return Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned owned storage without an allocation ID"
        )));
    }
    Ok(len)
}

pub fn validate_owned_result_releasing_on_error(
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
    operation: &'static str,
    release_operation: &'static str,
) -> Result<ZrOwnedResultV2, RuntimeForeignOutputError> {
    match validate_owned_result(&output, operation) {
        Ok(_) => Ok(output),
        Err(error) => release_owned_result_after_error(output, releaser, error, release_operation),
    }
}

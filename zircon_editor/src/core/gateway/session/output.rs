use std::slice;
use std::sync::Arc;

use zircon_runtime_host::foreign_output::{
    release_owned_result, validate_owned_result, validate_owned_result_releasing_on_error,
    RuntimeForeignOutputError, RuntimeForeignOutputKind, RuntimeForeignOutputState,
    RuntimeOwnedOutputReleaser,
};
use zircon_runtime_interface::{ZrOwnedResultV2, ZrStatus};

use super::super::GatewayError;

pub(super) struct GatewayOwnedOutput {
    raw: Option<ZrOwnedResultV2>,
    releaser: RuntimeOwnedOutputReleaser,
    foreign_output: Arc<RuntimeForeignOutputState>,
    kind: RuntimeForeignOutputKind,
}

impl GatewayOwnedOutput {
    fn new(
        raw: ZrOwnedResultV2,
        releaser: RuntimeOwnedOutputReleaser,
        foreign_output: Arc<RuntimeForeignOutputState>,
        kind: RuntimeForeignOutputKind,
    ) -> Self {
        Self {
            raw: Some(raw),
            releaser,
            foreign_output,
            kind,
        }
    }

    pub(super) fn bytes(&self, operation: &'static str) -> Result<&[u8], GatewayError> {
        let raw = self.raw.as_ref().ok_or_else(|| GatewayError::Protocol {
            message: format!("{operation} attempted to use released storage"),
        })?;
        let len = validate_owned_result(raw, operation)?;
        if len == 0 {
            return Ok(&[]);
        }
        Ok(unsafe { slice::from_raw_parts(raw.data, len) })
    }

    pub(super) fn release(mut self) -> Result<(), GatewayError> {
        let Some(raw) = self.raw.take() else {
            return Ok(());
        };
        match release_owned_result(raw, self.releaser, "release runtime gateway output") {
            Ok(()) => Ok(()),
            Err(error) => self
                .foreign_output
                .reject_protocol(self.kind, error)
                .map_err(Into::into),
        }
    }

    pub(super) fn release_after_protocol_error<T>(
        mut self,
        error: GatewayError,
    ) -> Result<T, GatewayError> {
        let error = RuntimeForeignOutputError::protocol_violation(error.to_string());
        let error = match self.raw.take() {
            Some(raw) => {
                match release_owned_result(raw, self.releaser, "release runtime gateway output") {
                    Ok(()) => error,
                    Err(release_error) => error.with_cleanup_failure(&release_error),
                }
            }
            None => error,
        };
        self.foreign_output
            .reject_protocol(self.kind, error)
            .map_err(Into::into)
    }
}

impl Drop for GatewayOwnedOutput {
    fn drop(&mut self) {
        let Some(raw) = self.raw.take() else {
            return;
        };
        if let Err(error) =
            release_owned_result(raw, self.releaser, "release runtime gateway output")
        {
            let _ = self.foreign_output.reject_protocol::<()>(self.kind, error);
        }
    }
}

pub(super) fn capture_owned_output(
    foreign_output: Arc<RuntimeForeignOutputState>,
    status: ZrStatus,
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
    operation: &'static str,
) -> Result<GatewayOwnedOutput, GatewayError> {
    let kind = RuntimeForeignOutputKind::SessionProtocol;
    let output = foreign_output.ensure_call_succeeded(
        status,
        output,
        releaser,
        kind,
        operation,
        "release runtime frame output",
    )?;
    let output = match validate_owned_result_releasing_on_error(
        output,
        releaser,
        operation,
        "release runtime frame output after invalid capture",
    ) {
        Ok(output) => output,
        Err(error) => {
            return foreign_output
                .reject_protocol(kind, error)
                .map_err(Into::into)
        }
    };
    Ok(GatewayOwnedOutput::new(
        output,
        releaser,
        foreign_output,
        kind,
    ))
}

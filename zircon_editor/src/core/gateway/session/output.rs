use std::slice;
use std::sync::Arc;

use zircon_runtime_host::foreign_output::{
    release_owned_buffer, validate_owned_buffer, validate_owned_buffer_releasing_on_error,
    RuntimeForeignOutputError, RuntimeForeignOutputKind, RuntimeForeignOutputState,
};
use zircon_runtime_interface::{ZrOwnedByteBuffer, ZrStatus};

use super::super::GatewayError;

pub(super) struct GatewayOwnedOutput {
    raw: Option<ZrOwnedByteBuffer>,
    foreign_output: Arc<RuntimeForeignOutputState>,
    kind: RuntimeForeignOutputKind,
}

impl GatewayOwnedOutput {
    fn new(
        raw: ZrOwnedByteBuffer,
        foreign_output: Arc<RuntimeForeignOutputState>,
        kind: RuntimeForeignOutputKind,
    ) -> Self {
        Self {
            raw: Some(raw),
            foreign_output,
            kind,
        }
    }

    pub(super) fn bytes(&self, operation: &'static str) -> Result<&[u8], GatewayError> {
        let raw = self.raw.as_ref().ok_or_else(|| GatewayError::Protocol {
            message: format!("{operation} attempted to use released storage"),
        })?;
        validate_owned_buffer(raw, operation)?;
        if raw.len == 0 {
            return Ok(&[]);
        }
        Ok(unsafe { slice::from_raw_parts(raw.data.cast_const(), raw.len) })
    }

    pub(super) fn release(mut self) -> Result<(), GatewayError> {
        let Some(raw) = self.raw.take() else {
            return Ok(());
        };
        match release_owned_buffer(raw, "free runtime gateway output") {
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
            Some(raw) => match release_owned_buffer(raw, "free runtime gateway output") {
                Ok(()) => error,
                Err(release_error) => error.with_cleanup_failure(&release_error),
            },
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
        if let Err(error) = release_owned_buffer(raw, "free runtime gateway output") {
            let _ = self.foreign_output.reject_protocol::<()>(self.kind, error);
        }
    }
}

pub(super) fn capture_owned_output(
    foreign_output: Arc<RuntimeForeignOutputState>,
    status: ZrStatus,
    output: ZrOwnedByteBuffer,
    operation: &'static str,
) -> Result<GatewayOwnedOutput, GatewayError> {
    let kind = RuntimeForeignOutputKind::SessionProtocol;
    foreign_output.ensure_call_succeeded(
        status,
        output,
        kind,
        operation,
        "free runtime frame output",
    )?;
    if let Err(error) = validate_owned_buffer_releasing_on_error(
        output,
        operation,
        "free runtime frame output after invalid capture",
    ) {
        return foreign_output
            .reject_protocol(kind, error)
            .map_err(Into::into);
    }
    Ok(GatewayOwnedOutput::new(output, foreign_output, kind))
}

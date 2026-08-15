use std::slice;

use serde::de::DeserializeOwned;
use zircon_runtime_interface::{ZrOwnedByteBuffer, ZrStatus};

use super::super::GatewayError;
use super::protocol::ensure_status;

pub(super) struct GatewayOwnedOutput {
    raw: Option<ZrOwnedByteBuffer>,
}

impl GatewayOwnedOutput {
    pub(super) fn new(raw: ZrOwnedByteBuffer) -> Self {
        Self { raw: Some(raw) }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.raw.as_ref().is_none_or(|raw| raw.len == 0)
    }

    pub(super) fn len(&self) -> usize {
        self.raw.as_ref().map_or(0, |raw| raw.len)
    }

    pub(super) fn validate(&self, operation: &'static str) -> Result<(), GatewayError> {
        let Some(raw) = self.raw.as_ref() else {
            return Err(GatewayError::Protocol {
                message: format!("{operation} attempted to use released storage"),
            });
        };
        if raw.len > raw.capacity {
            return Err(GatewayError::Protocol {
                message: format!(
                    "{operation} returned malformed storage: len {} exceeds capacity {}",
                    raw.len, raw.capacity
                ),
            });
        }
        if raw.len > isize::MAX as usize || raw.capacity > isize::MAX as usize {
            return Err(GatewayError::Protocol {
                message: format!(
                    "{operation} returned malformed storage: len {} and capacity {} exceed the maximum Rust slice allocation",
                    raw.len, raw.capacity
                ),
            });
        }
        if raw.data.is_null() {
            return if raw.len == 0 && raw.capacity == 0 {
                Ok(())
            } else {
                Err(GatewayError::Protocol {
                    message: format!(
                        "{operation} returned malformed storage: null data with len {} and capacity {}",
                        raw.len, raw.capacity
                    ),
                })
            };
        }
        if !raw.data.is_null() && raw.free.is_none() {
            return Err(GatewayError::Protocol {
                message: format!("{operation} returned owned storage without a free callback"),
            });
        }
        Ok(())
    }

    pub(super) fn bytes(&self, operation: &'static str) -> Result<&[u8], GatewayError> {
        self.validate(operation)?;
        let raw = self.raw.as_ref().ok_or_else(|| GatewayError::Protocol {
            message: format!("{operation} attempted to decode released storage"),
        })?;
        if raw.len == 0 {
            return Ok(&[]);
        }
        Ok(unsafe { slice::from_raw_parts(raw.data.cast_const(), raw.len) })
    }

    pub(super) fn release(mut self) -> Result<(), GatewayError> {
        let Some(raw) = self.raw.take() else {
            return Ok(());
        };
        if let Some(free) = raw.free {
            ensure_status(unsafe { free(raw) }, "free runtime gateway output")?;
        }
        Ok(())
    }
}

impl Drop for GatewayOwnedOutput {
    fn drop(&mut self) {
        let Some(raw) = self.raw.take() else {
            return;
        };
        if let Some(free) = raw.free {
            let _ = unsafe { free(raw) };
        }
    }
}

pub(super) fn decode_owned_output<T: DeserializeOwned>(
    output: GatewayOwnedOutput,
    operation: &'static str,
) -> Result<T, GatewayError> {
    let decoded = match output.bytes(operation) {
        Ok([]) => Err(GatewayError::Protocol {
            message: format!("{operation} returned an empty payload"),
        }),
        Ok(bytes) => serde_json::from_slice(bytes).map_err(|error| GatewayError::Protocol {
            message: format!("{operation} returned invalid JSON: {error}"),
        }),
        Err(error) => Err(error),
    };
    let released = output.release();
    match (decoded, released) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(release_error)) => Err(GatewayError::Protocol {
            message: format!("{error}; cleanup also failed: {release_error}"),
        }),
    }
}

pub(super) fn validate_output_status(
    status: ZrStatus,
    output: ZrOwnedByteBuffer,
    operation: &'static str,
) -> Result<GatewayOwnedOutput, GatewayError> {
    let output = GatewayOwnedOutput::new(output);
    let validation_error = output.validate(operation).err();
    let status_error = ensure_status(status, operation).err();
    if status_error.is_none() && validation_error.is_none() {
        return Ok(output);
    }

    let released = output.release();
    match (status_error, validation_error, released) {
        (Some(status_error), None, Ok(())) => Err(status_error),
        (status_error, Some(validation_error), Ok(())) => Err(GatewayError::Protocol {
            message: match status_error {
                Some(status_error) => format!("{status_error}; {validation_error}"),
                None => validation_error.to_string(),
            },
        }),
        (status_error, validation_error, Err(release_error)) => Err(GatewayError::Protocol {
            message: format!(
                "{}; cleanup also failed: {release_error}",
                status_error
                    .map(|error| error.to_string())
                    .or_else(|| validation_error.map(|error| error.to_string()))
                    .unwrap_or_else(|| operation.to_string())
            ),
        }),
        (None, None, Ok(())) => Err(GatewayError::Protocol {
            message: format!("{operation} reached an inconsistent validated-output cleanup state"),
        }),
    }
}

pub(super) fn release_output_after_error<T>(
    output: GatewayOwnedOutput,
    error: GatewayError,
) -> Result<T, GatewayError> {
    match output.release() {
        Ok(()) => Err(error),
        Err(release_error) => Err(GatewayError::Protocol {
            message: format!("{error}; cleanup also failed: {release_error}"),
        }),
    }
}

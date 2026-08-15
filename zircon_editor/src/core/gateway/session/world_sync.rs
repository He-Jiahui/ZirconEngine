use zircon_runtime_interface::world_sync::{
    InvalidationBatch, WatchRegistration, WatchToken, WorldQuery, WorldQueryResult,
};
use zircon_runtime_interface::{ZrByteSlice, ZrOwnedByteBuffer};

use super::super::GatewayError;
use super::gateway::SessionGateway;
use super::output::{decode_owned_output, release_output_after_error, validate_output_status};
use super::protocol::ensure_status;

const MAX_WORLD_SYNC_RESPONSE_BYTES: usize = 1024 * 1024;

impl SessionGateway {
    pub(super) fn query_world(&self, query: WorldQuery) -> Result<WorldQueryResult, GatewayError> {
        let query_world = Self::required(self.api.query_world, "runtime.world_sync.query")?;
        let request = serde_json::to_vec(&query).map_err(|error| GatewayError::Protocol {
            message: format!("runtime world query request cannot be encoded as JSON: {error}"),
        })?;
        let mut output = ZrOwnedByteBuffer::empty();
        let status = unsafe {
            query_world(
                self.session,
                ZrByteSlice {
                    data: request.as_ptr(),
                    len: request.len(),
                },
                &mut output,
            )
        };
        let output = validate_output_status(status, output, "query runtime world")?;
        let output_len = output.len();
        if output_len > MAX_WORLD_SYNC_RESPONSE_BYTES {
            return release_output_after_error(
                output,
                GatewayError::Protocol {
                    message: format!(
                        "runtime world query returned {} encoded bytes; maximum is {MAX_WORLD_SYNC_RESPONSE_BYTES}",
                        output_len
                    ),
                },
            );
        }
        decode_owned_output(output, "query runtime world")
    }

    pub(super) fn watch_world(
        &self,
        registration: WatchRegistration,
    ) -> Result<WatchToken, GatewayError> {
        let watch_world = Self::required(self.api.watch_world, "runtime.world_sync.watch")?;
        let request =
            serde_json::to_vec(&registration).map_err(|error| GatewayError::Protocol {
                message: format!("runtime world watch request cannot be encoded as JSON: {error}"),
            })?;
        let mut token = WatchToken::new(0);
        ensure_status(
            unsafe {
                watch_world(
                    self.session,
                    ZrByteSlice {
                        data: request.as_ptr(),
                        len: request.len(),
                    },
                    &mut token,
                )
            },
            "register runtime world watch",
        )?;
        if !token.is_valid() {
            return Err(GatewayError::Protocol {
                message: "runtime returned an invalid world watch token".to_owned(),
            });
        }
        Ok(token)
    }

    pub(super) fn unwatch_world(&self, token: WatchToken) -> Result<bool, GatewayError> {
        if !token.is_valid() {
            return Err(GatewayError::Protocol {
                message: "cannot revoke an invalid runtime world watch token".to_owned(),
            });
        }
        let unwatch_world = Self::required(self.api.unwatch_world, "runtime.world_sync.unwatch")?;
        let mut removed = u8::MAX;
        ensure_status(
            unsafe { unwatch_world(self.session, token, &mut removed) },
            "revoke runtime world watch",
        )?;
        match removed {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(GatewayError::Protocol {
                message: format!(
                    "runtime world watch revocation returned invalid boolean {removed}"
                ),
            }),
        }
    }

    pub(super) fn drain_world_invalidations(&self) -> Result<Vec<InvalidationBatch>, GatewayError> {
        let drain = Self::required(
            self.api.drain_world_invalidations,
            "runtime.world_sync.drain",
        )?;
        let mut output = ZrOwnedByteBuffer::empty();
        let status = unsafe { drain(self.session, &mut output) };
        let output = validate_output_status(status, output, "drain runtime world invalidations")?;
        if output.is_empty() {
            output.release()?;
            return Ok(Vec::new());
        }
        let output_len = output.len();
        if output_len > MAX_WORLD_SYNC_RESPONSE_BYTES {
            return release_output_after_error(
                output,
                GatewayError::Protocol {
                    message: format!(
                        "runtime world invalidation drain returned {} encoded bytes; maximum is {MAX_WORLD_SYNC_RESPONSE_BYTES}",
                        output_len
                    ),
                },
            );
        }
        decode_owned_output(output, "drain runtime world invalidations")
    }
}

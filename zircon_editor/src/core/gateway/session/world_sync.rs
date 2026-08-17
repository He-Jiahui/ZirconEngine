use zircon_runtime_host::foreign_output::{
    world_invalidation_item_count, world_query_item_count, RuntimeForeignOutputKind,
    WORLD_INVALIDATION_OUTPUT_BUDGET, WORLD_QUERY_OUTPUT_BUDGET,
};
use zircon_runtime_interface::world_sync::{
    InvalidationBatch, WatchRegistration, WatchToken, WorldQuery, WorldQueryResult,
};
use zircon_runtime_interface::{ZrByteSlice, ZrOwnedByteBuffer};

use super::super::GatewayError;
use super::gateway::SessionGateway;
use super::protocol::ensure_status;

impl SessionGateway {
    pub(super) fn query_world(&self, query: WorldQuery) -> Result<WorldQueryResult, GatewayError> {
        self.ensure_output_available(RuntimeForeignOutputKind::WorldQuery)?;
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
        self.decode_output(
            status,
            output,
            RuntimeForeignOutputKind::WorldQuery,
            WORLD_QUERY_OUTPUT_BUDGET,
            "query runtime world",
            "free runtime world query output",
            |result: &WorldQueryResult| Ok::<usize, GatewayError>(world_query_item_count(result)),
        )?
        .ok_or_else(|| GatewayError::Protocol {
            message: "query runtime world returned an empty payload".to_owned(),
        })
    }

    pub(super) fn watch_world(
        &self,
        registration: WatchRegistration,
    ) -> Result<WatchToken, GatewayError> {
        self.ensure_session_available("register runtime world watch")?;
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
            return self.reject_protocol(
                RuntimeForeignOutputKind::WorldInvalidations,
                GatewayError::Protocol {
                    message: "runtime returned an invalid world watch token".to_owned(),
                },
            );
        }
        Ok(token)
    }

    pub(super) fn unwatch_world(&self, token: WatchToken) -> Result<bool, GatewayError> {
        self.ensure_session_available("revoke runtime world watch")?;
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
            _ => self.reject_protocol(
                RuntimeForeignOutputKind::WorldInvalidations,
                GatewayError::Protocol {
                    message: format!(
                        "runtime world watch revocation returned invalid boolean {removed}"
                    ),
                },
            ),
        }
    }

    pub(super) fn drain_world_invalidations(&self) -> Result<Vec<InvalidationBatch>, GatewayError> {
        self.ensure_output_available(RuntimeForeignOutputKind::WorldInvalidations)?;
        let drain = Self::required(
            self.api.drain_world_invalidations,
            "runtime.world_sync.drain",
        )?;
        let mut output = ZrOwnedByteBuffer::empty();
        let status = unsafe { drain(self.session, &mut output) };
        self.decode_output(
            status,
            output,
            RuntimeForeignOutputKind::WorldInvalidations,
            WORLD_INVALIDATION_OUTPUT_BUDGET,
            "drain runtime world invalidations",
            "free runtime world invalidation output",
            |batches: &Vec<InvalidationBatch>| {
                Ok::<usize, GatewayError>(batches.iter().fold(0_usize, |count, batch| {
                    count.saturating_add(world_invalidation_item_count(batch))
                }))
            },
        )
        .map(|batches| batches.unwrap_or_default())
    }
}

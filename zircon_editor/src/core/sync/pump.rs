use thiserror::Error;
use zircon_runtime_interface::world_sync::{WatchRegistration, WatchToken};

use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::{
    EditorMessage, EditorTopic, EditorViewInvalidationMask, SharedEditorMessageBus,
};
use crate::core::gateway::{EditorRuntimeGateway, EditorRuntimeGatewayHandle, GatewayError};

use super::{WorldWatchMap, WorldWatchMapError};

#[cfg(test)]
mod tests;

/// Bus topic for transport-neutral runtime world facts.
pub const TOPIC_WORLD_FACT: &str = "editor.world_fact";

const WORLD_FACT_SCHEMA_ID: &str = "zircon.editor.world_fact.v1";

/// Outcome of one editor-frame world synchronization drain.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorldSyncPumpReport {
    transport_available: bool,
    batches: usize,
    published_facts: usize,
    matched_tokens: usize,
    dirty_views: usize,
    duplicate_tokens: usize,
    unknown_tokens: usize,
    last_generation: Option<u64>,
}

impl WorldSyncPumpReport {
    /// Returns false only when the current gateway has no world-sync drain capability yet.
    pub fn transport_available(&self) -> bool {
        self.transport_available
    }

    /// Returns the number of runtime batches consumed by this frame.
    pub fn batches(&self) -> usize {
        self.batches
    }

    /// Returns the number of immutable runtime facts published to the editor bus.
    pub fn published_facts(&self) -> usize {
        self.published_facts
    }

    /// Returns the number of runtime tokens that matched a live editor view binding.
    pub fn matched_tokens(&self) -> usize {
        self.matched_tokens
    }

    /// Returns the number of views marked dirty after per-batch coalescing.
    pub fn dirty_views(&self) -> usize {
        self.dirty_views
    }

    /// Returns duplicate runtime tokens reported by malformed batches.
    pub fn duplicate_tokens(&self) -> usize {
        self.duplicate_tokens
    }

    /// Returns runtime tokens that no longer have an editor view binding.
    pub fn unknown_tokens(&self) -> usize {
        self.unknown_tokens
    }

    /// Returns the most recent runtime generation consumed in this frame.
    pub fn last_generation(&self) -> Option<u64> {
        self.last_generation
    }
}

/// Errors at the explicit editor/runtime world-sync boundary.
#[derive(Debug, Error)]
pub enum WorldSyncPumpError {
    #[error(transparent)]
    Gateway(#[from] GatewayError),
    #[error(transparent)]
    WatchMap(#[from] WorldWatchMapError),
    #[error("world fact cannot be encoded for the editor message bus: {0}")]
    FactEncoding(#[from] serde_json::Error),
    #[error(
        "runtime world generations regressed from {previous} to {observed} without a gateway replacement"
    )]
    GenerationRegression { previous: u64, observed: u64 },
    #[error("runtime reused the live world watch token {token:?}")]
    TokenCollision { token: WatchToken },
    #[error(
        "editor rejected runtime world watch token {token:?}, and compensating runtime unwatch failed: {cleanup}"
    )]
    WatchRegistrationCleanup {
        token: WatchToken,
        cleanup: GatewayError,
    },
}

/// Session-scoped owner for runtime watch tokens and one fixed editor-frame drain.
///
/// This is the only runtime-to-editor projection point: each call to [`Self::pump`] performs one
/// gateway drain, publishes immutable facts, and writes coalesced view dirtiness into the shared
/// editor bus. Runtime gateway replacement clears old token bindings because tokens belong to the
/// retired session, not the stable gateway handle.
#[derive(Clone, Debug, Default)]
pub struct WorldSyncPump {
    watches: WorldWatchMap,
    gateway_generation: Option<u64>,
    last_generation: Option<u64>,
}

impl WorldSyncPump {
    /// Returns the editor-owned token-to-view bindings for diagnostics and lifecycle ownership.
    pub fn watches(&self) -> &WorldWatchMap {
        &self.watches
    }

    /// Registers a runtime watch and binds its opaque token to one editor view.
    ///
    /// Invalid editor bindings are rejected before runtime allocation. If a malformed runtime
    /// returns a colliding token, the existing editor binding is retained: unwatching an opaque
    /// colliding token could revoke the valid subscription already owned by this session. A
    /// binding failure after a newly allocated token still compensates with `unwatch_world`.
    pub fn watch_view(
        &mut self,
        gateway: &EditorRuntimeGatewayHandle,
        registration: WatchRegistration,
        view: ViewInstanceId,
        mask: EditorViewInvalidationMask,
    ) -> Result<WatchToken, WorldSyncPumpError> {
        self.watch_view_with_gateway_generation(gateway, registration, view, mask)
            .map(|(token, _)| token)
    }

    /// Registers a view watch and returns the runtime generation that issued its token.
    ///
    /// The token and generation are captured under the gateway replacement lock. Owners that
    /// retain tokens across frames must store them together, because a replacement runtime may
    /// reuse an opaque token value.
    pub(crate) fn watch_view_with_gateway_generation(
        &mut self,
        gateway: &EditorRuntimeGatewayHandle,
        registration: WatchRegistration,
        view: ViewInstanceId,
        mask: EditorViewInvalidationMask,
    ) -> Result<(WatchToken, u64), WorldSyncPumpError> {
        if mask.is_empty() {
            return Err(WorldWatchMapError::EmptyInvalidationMask.into());
        }
        gateway.with_current_gateway_generation(|generation, runtime| {
            self.synchronize_gateway_generation_value(generation);
            if let Some(token) = self.watches.token_for(&view, &registration, mask) {
                return Ok((token, generation));
            }

            let token = runtime.watch_world(registration.clone())?;
            self.reject_live_watch_token(token)?;
            if let Err(error) = self.watches.bind(token, registration, view, mask) {
                return match runtime.unwatch_world(token) {
                    Ok(_) => Err(error.into()),
                    Err(cleanup) => {
                        Err(WorldSyncPumpError::WatchRegistrationCleanup { token, cleanup })
                    }
                };
            }
            Ok((token, generation))
        })
    }

    /// Revokes the runtime token and clears its editor projection.
    ///
    /// A token no longer bound after a gateway-generation change is already stale and must not be
    /// submitted to the current runtime, where an opaque value could name a new-session watch.
    pub fn unwatch_view(
        &mut self,
        gateway: &EditorRuntimeGatewayHandle,
        token: WatchToken,
    ) -> Result<bool, WorldSyncPumpError> {
        gateway.with_current_gateway_generation(|generation, runtime| {
            self.synchronize_gateway_generation_value(generation);
            if self.watches.binding(token).is_none() {
                return Ok(false);
            }
            let runtime_removed = runtime.unwatch_world(token)?;
            let editor_removed = self.watches.unbind_token(token).is_some();
            Ok(runtime_removed || editor_removed)
        })
    }

    /// Consumes exactly one runtime invalidation drain for an editor frame.
    ///
    /// A detached or not-yet-upgraded transport returns an explicit unavailable report. It does
    /// not emulate world sync or silently substitute a snapshot path.
    pub fn pump(
        &mut self,
        gateway: &EditorRuntimeGatewayHandle,
        bus: &SharedEditorMessageBus,
    ) -> Result<WorldSyncPumpReport, WorldSyncPumpError> {
        self.synchronize_gateway_generation(gateway);
        let batches = match gateway.drain_world_invalidations() {
            Ok(batches) => batches,
            Err(GatewayError::CapabilityMissing {
                capability: "runtime.world_sync.drain",
            }) => return Ok(WorldSyncPumpReport::default()),
            Err(error) => return Err(error.into()),
        };

        let mut previous_generation = self.last_generation;
        for batch in &batches {
            if let Some(previous) = previous_generation {
                if batch.generation < previous {
                    return Err(WorldSyncPumpError::GenerationRegression {
                        previous,
                        observed: batch.generation,
                    });
                }
            }
            previous_generation = Some(batch.generation);
        }

        let topic = world_fact_topic();
        let mut report = WorldSyncPumpReport {
            transport_available: true,
            batches: batches.len(),
            ..Default::default()
        };
        for batch in batches {
            for fact in &batch.facts {
                let payload = serde_json::to_value(fact)?;
                bus.publish(
                    topic.clone(),
                    EditorMessage::custom(WORLD_FACT_SCHEMA_ID, payload),
                );
                report.published_facts += 1;
            }

            let projection = self.watches.project(&batch);
            report.matched_tokens += projection.matched_tokens();
            report.duplicate_tokens += projection.duplicate_tokens().len();
            report.unknown_tokens += projection.unknown_tokens().len();
            bus.mark_view_dirty_set(projection.dirty());
            report.dirty_views += projection.dirty().len();
            report.last_generation = Some(batch.generation);
        }
        self.last_generation = previous_generation;
        Ok(report)
    }

    fn synchronize_gateway_generation(&mut self, gateway: &EditorRuntimeGatewayHandle) {
        self.synchronize_gateway_generation_value(gateway.generation());
    }

    fn synchronize_gateway_generation_value(&mut self, generation: u64) {
        if self.gateway_generation == Some(generation) {
            return;
        }
        self.gateway_generation = Some(generation);
        self.last_generation = None;
        self.watches.drain_tokens();
    }

    fn reject_live_watch_token(&self, token: WatchToken) -> Result<(), WorldSyncPumpError> {
        self.watches
            .binding(token)
            .is_none()
            .then_some(())
            .ok_or(WorldSyncPumpError::TokenCollision { token })
    }
}

fn world_fact_topic() -> EditorTopic {
    EditorTopic::parse(TOPIC_WORLD_FACT).expect("world fact topic is a static valid editor topic")
}

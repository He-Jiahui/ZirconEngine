use std::sync::OnceLock;

use thiserror::Error;
use zircon_runtime_interface::world_sync::{WatchRegistration, WatchToken, WorldFact};

use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::{
    EditorMessage, EditorMessageSchemaId, EditorTopic, EditorViewInvalidationMask,
    SharedEditorMessageBus,
};
use crate::core::gateway::{
    EditorRuntimeGateway, EditorRuntimeGatewayHandle, GatewayError, GatewaySessionIdentity,
};

use super::{WorldWatchMap, WorldWatchMapError};

#[cfg(test)]
mod tests;

/// Bus topic for transport-neutral runtime world facts.
pub const TOPIC_WORLD_FACT: &str = "editor.world_fact";

const WORLD_FACT_LOCAL_SCHEMA: &str = "world_fact.v1";

fn world_fact_schema_id() -> &'static EditorMessageSchemaId {
    static SCHEMA_ID: OnceLock<EditorMessageSchemaId> = OnceLock::new();
    SCHEMA_ID.get_or_init(|| {
        EditorMessageSchemaId::editor(WORLD_FACT_LOCAL_SCHEMA)
            .expect("the built-in world-fact schema id is valid")
    })
}

/// Opaque runtime watch token qualified by the transport that issued it.
///
/// This is the only watch receipt that may outlive a gateway call. The raw token is unwrapped
/// solely after the current lease has been proven to have the same complete identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QualifiedWatchToken {
    token: WatchToken,
    identity: GatewaySessionIdentity,
}

impl QualifiedWatchToken {
    pub(crate) fn new(token: WatchToken, identity: GatewaySessionIdentity) -> Self {
        Self { token, identity }
    }

    pub(crate) const fn token(&self) -> WatchToken {
        self.token
    }

    pub(crate) fn identity(&self) -> &GatewaySessionIdentity {
        &self.identity
    }
}

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
    stale_gateway_drains: usize,
    drain_identity: Option<GatewaySessionIdentity>,
    drain_gateway_generation: Option<u64>,
    last_generation: Option<u64>,
    advanced_world_replacement_epoch: Option<u64>,
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

    /// Returns the number of origin drains discarded because replacement won before projection.
    pub fn stale_gateway_drains(&self) -> usize {
        self.stale_gateway_drains
    }

    /// Returns the complete identity of the transport that performed the drain.
    pub fn drain_identity(&self) -> Option<&GatewaySessionIdentity> {
        self.drain_identity.as_ref()
    }

    /// Returns the gateway generation that performed the drain, including a discarded stale drain.
    pub fn drain_gateway_generation(&self) -> Option<u64> {
        self.drain_gateway_generation
    }

    /// Returns the most recent runtime generation consumed in this frame.
    pub fn last_generation(&self) -> Option<u64> {
        self.last_generation
    }

    /// Returns the newest world replacement awaiting editor retirement acknowledgement.
    pub fn advanced_world_replacement_epoch(&self) -> Option<u64> {
        self.advanced_world_replacement_epoch
    }
}

/// Per-watch outcome recorded by an explicit world-sync shutdown.
///
/// The editor always retires its local binding. A remote unwatch is attempted only when the
/// current gateway still has the identity that issued the token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldSyncShutdownWatchReceipt {
    token: WatchToken,
    disposition: WorldSyncShutdownWatchDisposition,
}

impl WorldSyncShutdownWatchReceipt {
    /// Returns the opaque runtime token that was retired locally.
    pub fn token(&self) -> WatchToken {
        self.token
    }

    /// Returns the remote cleanup result for this token.
    pub fn disposition(&self) -> &WorldSyncShutdownWatchDisposition {
        &self.disposition
    }
}

/// Remote cleanup result for one locally retired world watch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorldSyncShutdownWatchDisposition {
    /// The origin runtime confirmed it removed the token.
    Unwatched,
    /// The origin runtime had already retired the token.
    AlreadyAbsent,
    /// The current gateway identity differs from the identity that issued the token.
    StaleIdentity,
    /// The origin runtime could not complete the best-effort unwatch.
    Failed(GatewayError),
}

/// Complete local-retirement and best-effort remote-cleanup record for world watches.
///
/// This is deliberately a receipt rather than an all-or-nothing result: shutdown must retire
/// editor state even after the remote transport has been replaced or lost.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldSyncShutdownReceipt {
    origin_identity: Option<GatewaySessionIdentity>,
    current_identity: GatewaySessionIdentity,
    watches: Vec<WorldSyncShutdownWatchReceipt>,
}

impl WorldSyncShutdownReceipt {
    /// Returns the identity that issued the retired watch set, when one was still tracked.
    pub fn origin_identity(&self) -> Option<&GatewaySessionIdentity> {
        self.origin_identity.as_ref()
    }

    /// Returns the identity published by the gateway when shutdown began.
    pub fn current_identity(&self) -> &GatewaySessionIdentity {
        &self.current_identity
    }

    /// Returns all locally retired watches in deterministic token order.
    pub fn watches(&self) -> &[WorldSyncShutdownWatchReceipt] {
        &self.watches
    }

    /// Returns the number of remote unwatch calls confirmed by the origin runtime.
    pub fn unwatched_count(&self) -> usize {
        self.watches
            .iter()
            .filter(|watch| {
                matches!(
                    watch.disposition(),
                    WorldSyncShutdownWatchDisposition::Unwatched
                )
            })
            .count()
    }

    /// Returns the number of watches skipped because their origin transport was no longer live.
    pub fn stale_identity_count(&self) -> usize {
        self.watches
            .iter()
            .filter(|watch| {
                matches!(
                    watch.disposition(),
                    WorldSyncShutdownWatchDisposition::StaleIdentity
                )
            })
            .count()
    }

    /// Returns the number of remote cleanup failures retained for the session coordinator.
    pub fn failed_count(&self) -> usize {
        self.watches
            .iter()
            .filter(|watch| {
                matches!(
                    watch.disposition(),
                    WorldSyncShutdownWatchDisposition::Failed(_)
                )
            })
            .count()
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
    #[error("runtime published the reserved zero world replacement epoch")]
    ZeroWorldReplacementEpoch,
    #[error(
        "runtime world replacement epoch regressed from {previous} to {observed} without a gateway replacement"
    )]
    WorldReplacementEpochRegression { previous: u64, observed: u64 },
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
    gateway_identity: Option<GatewaySessionIdentity>,
    last_generation: Option<u64>,
    last_world_replacement_epoch: Option<u64>,
    pending_world_replacement_epoch: Option<u64>,
}

impl WorldSyncPump {
    /// Returns the editor-owned token-to-view bindings for diagnostics and lifecycle ownership.
    pub fn watches(&self) -> &WorldWatchMap {
        &self.watches
    }

    /// Confirms that the editor retired every state derived from this replacement identity.
    pub fn acknowledge_world_replacement(&mut self, replacement_epoch: u64) -> bool {
        if self.pending_world_replacement_epoch != Some(replacement_epoch) {
            return false;
        }
        self.pending_world_replacement_epoch = None;
        true
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
    ) -> Result<QualifiedWatchToken, WorldSyncPumpError> {
        self.watch_view_with_identity(gateway, registration, view, mask)
    }

    /// Registers a view watch and returns the complete identity that issued its token.
    ///
    /// The token and identity are captured from one immutable gateway lease. Owners that retain
    /// tokens across frames must store them together, because a replacement runtime may reuse an
    /// opaque token value.
    pub(crate) fn watch_view_with_identity(
        &mut self,
        gateway: &EditorRuntimeGatewayHandle,
        registration: WatchRegistration,
        view: ViewInstanceId,
        mask: EditorViewInvalidationMask,
    ) -> Result<QualifiedWatchToken, WorldSyncPumpError> {
        if mask.is_empty() {
            return Err(WorldWatchMapError::EmptyInvalidationMask.into());
        }
        let lease = gateway.current_lease();
        let generation = lease.generation();
        let identity = lease.identity().clone();
        self.synchronize_gateway_identity(identity.clone());
        if let Some(token) = self.watches.token_for(&view, &registration, mask) {
            return Ok(QualifiedWatchToken::new(token, identity));
        }

        let token = lease.gateway().watch_world(registration.clone())?;
        let current = gateway.current_lease();
        if current.identity() != &identity {
            let _ = lease.gateway().unwatch_world(token);
            return Err(GatewayError::StaleGeneration {
                expected_generation: generation,
                current_generation: current.generation(),
            }
            .into());
        }
        self.reject_live_watch_token(token)?;
        if let Err(error) = self.watches.bind(token, registration, view, mask) {
            return match lease.gateway().unwatch_world(token) {
                Ok(_) => Err(error.into()),
                Err(cleanup) => {
                    Err(WorldSyncPumpError::WatchRegistrationCleanup { token, cleanup })
                }
            };
        }
        Ok(QualifiedWatchToken::new(token, identity))
    }

    /// Revokes the runtime token and clears its editor projection.
    ///
    /// A token no longer bound after a gateway-generation change is already stale and must not be
    /// submitted to the current runtime, where an opaque value could name a new-session watch.
    pub fn unwatch_view(
        &mut self,
        gateway: &EditorRuntimeGatewayHandle,
        token: &QualifiedWatchToken,
    ) -> Result<bool, WorldSyncPumpError> {
        let lease = gateway.current_lease();
        if token.identity() != lease.identity() {
            self.synchronize_gateway_identity(lease.identity().clone());
            return Ok(false);
        }
        self.synchronize_gateway_identity(lease.identity().clone());
        if self.watches.binding(token.token()).is_none() {
            return Ok(false);
        }
        let runtime_removed = lease.gateway().unwatch_world(token.token())?;
        let editor_removed = self.watches.unbind_token(token.token()).is_some();
        Ok(runtime_removed || editor_removed)
    }

    /// Retires every local watch and attempts origin-only remote cleanup.
    ///
    /// This method is the explicit lifecycle boundary consumed by the future session coordinator.
    /// It does not let a replacement gateway receive stale opaque tokens, and it returns cleanup
    /// failures instead of restoring local watches or keeping the editor in an active state.
    pub fn shutdown(&mut self, gateway: &EditorRuntimeGatewayHandle) -> WorldSyncShutdownReceipt {
        let origin_identity = self.gateway_identity.take();
        let tokens = self.watches.drain_tokens();
        self.last_generation = None;
        self.last_world_replacement_epoch = None;
        self.pending_world_replacement_epoch = None;

        let lease = gateway.current_lease();
        let current_identity = lease.identity().clone();
        let can_unwatch_origin = origin_identity.as_ref() == Some(&current_identity);
        let watches = tokens
            .into_iter()
            .map(|token| {
                let disposition = if !can_unwatch_origin {
                    WorldSyncShutdownWatchDisposition::StaleIdentity
                } else {
                    match lease.gateway().unwatch_world(token) {
                        Ok(true) => WorldSyncShutdownWatchDisposition::Unwatched,
                        Ok(false) => WorldSyncShutdownWatchDisposition::AlreadyAbsent,
                        Err(error) => WorldSyncShutdownWatchDisposition::Failed(error),
                    }
                };
                WorldSyncShutdownWatchReceipt { token, disposition }
            })
            .collect();

        WorldSyncShutdownReceipt {
            origin_identity,
            current_identity,
            watches,
        }
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
        let lease = gateway.current_lease();
        let identity = lease.identity().clone();
        self.synchronize_gateway_identity(identity.clone());
        let batches = match lease.gateway().drain_world_invalidations() {
            Ok(batches) => batches,
            Err(GatewayError::CapabilityMissing {
                capability: "runtime.world_sync.drain",
            }) => return Ok(WorldSyncPumpReport::default()),
            Err(error) => return Err(error.into()),
        };
        let current = gateway.current_lease();
        if current.identity() != &identity {
            self.synchronize_gateway_identity(current.identity().clone());
            return Ok(WorldSyncPumpReport {
                transport_available: true,
                stale_gateway_drains: 1,
                drain_identity: Some(identity),
                drain_gateway_generation: Some(lease.generation()),
                ..Default::default()
            });
        }

        let mut previous_generation = self.last_generation;
        let mut previous_world_replacement_epoch = self.last_world_replacement_epoch;
        let mut pending_world_replacement_epoch = self.pending_world_replacement_epoch;
        let mut advanced_world_replacement_epoch = pending_world_replacement_epoch;
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
            let replacement_reached_live_watch = batch
                .dirty
                .iter()
                .any(|token| self.watches.binding(*token).is_some());
            for fact in &batch.facts {
                let WorldFact::WorldReplaced { replacement_epoch } = fact else {
                    continue;
                };
                if world_replacement_epoch_advanced(
                    previous_world_replacement_epoch,
                    *replacement_epoch,
                )? {
                    previous_world_replacement_epoch = Some(*replacement_epoch);
                    if replacement_reached_live_watch {
                        pending_world_replacement_epoch = Some(*replacement_epoch);
                        advanced_world_replacement_epoch = Some(*replacement_epoch);
                    }
                }
            }
        }

        let topic = world_fact_topic();
        let mut report = WorldSyncPumpReport {
            transport_available: true,
            batches: batches.len(),
            drain_identity: Some(identity),
            drain_gateway_generation: Some(lease.generation()),
            advanced_world_replacement_epoch,
            ..Default::default()
        };
        for batch in batches {
            for fact in &batch.facts {
                let payload = serde_json::to_value(fact)?;
                bus.publish(
                    topic.clone(),
                    EditorMessage::custom(world_fact_schema_id().clone(), payload),
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
        self.last_world_replacement_epoch = previous_world_replacement_epoch;
        self.pending_world_replacement_epoch = pending_world_replacement_epoch;
        Ok(report)
    }

    fn synchronize_gateway_identity(&mut self, identity: GatewaySessionIdentity) {
        if self.gateway_identity.as_ref() == Some(&identity) {
            return;
        }
        self.gateway_identity = Some(identity);
        self.last_generation = None;
        self.last_world_replacement_epoch = None;
        self.pending_world_replacement_epoch = None;
        self.watches.clear();
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

fn world_replacement_epoch_advanced(
    previous: Option<u64>,
    observed: u64,
) -> Result<bool, WorldSyncPumpError> {
    if observed == 0 {
        return Err(WorldSyncPumpError::ZeroWorldReplacementEpoch);
    }
    if let Some(previous) = previous {
        if observed < previous {
            return Err(WorldSyncPumpError::WorldReplacementEpochRegression { previous, observed });
        }
    }
    Ok(previous != Some(observed))
}

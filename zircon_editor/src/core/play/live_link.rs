use std::error::Error as StdError;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use crate::core::gateway::{
    EditorRuntimeGatewayHandle, GatewayError, GatewaySessionIdentity, SharedEditorRuntimeGateway,
};

/// Identifies one runtime world attached to the editor play domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlayInstanceId(u64);

impl PlayInstanceId {
    pub const fn raw(self) -> u64 {
        self.0
    }

    #[cfg(test)]
    pub(crate) const fn for_test(raw: u64) -> Self {
        assert!(raw != 0, "test play instance identity must be non-zero");
        Self(raw)
    }
}

impl TryFrom<u64> for PlayInstanceId {
    type Error = &'static str;

    fn try_from(raw: u64) -> Result<Self, Self::Error> {
        (raw != 0)
            .then_some(Self(raw))
            .ok_or("play instance identity must be non-zero")
    }
}

impl Serialize for PlayInstanceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

impl<'de> Deserialize<'de> for PlayInstanceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_from(u64::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

/// Selects the authoritative world facade for an editor operation.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum WorldDomain {
    #[default]
    Edit,
    Play(PlayInstanceId),
}

#[derive(Debug, Error)]
pub enum PlayDomainLinkError {
    #[error("play gateway cannot be detached while runtime mode is {mode:?}")]
    RuntimeStillActive { mode: super::PlayModeKind },
    #[error("play gateway terminal detachment is already in progress")]
    TerminalDetachInProgress,
    #[error("play domain {attached:?} is already attached")]
    AlreadyAttached { attached: PlayInstanceId },
    #[error("play domain {requested:?} is not the attached domain {attached:?}")]
    AttachmentMismatch {
        requested: PlayInstanceId,
        attached: Option<PlayInstanceId>,
    },
    #[error(
        "play domain {instance:?} no longer has the expected gateway identity {expected:?}; current identity is {current:?}"
    )]
    GatewayIdentityMismatch {
        instance: PlayInstanceId,
        expected: GatewaySessionIdentity,
        current: GatewaySessionIdentity,
    },
    #[error("play instance identity space is exhausted")]
    InstanceIdExhausted,
    #[error(transparent)]
    Gateway(#[from] GatewayError),
}

#[derive(Debug)]
pub enum PlayTerminalGatewayDetachError<E> {
    Preparation(E),
    Domain(PlayDomainLinkError),
}

impl<E: fmt::Display> fmt::Display for PlayTerminalGatewayDetachError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Preparation(error) => {
                write!(
                    formatter,
                    "play-domain retirement preparation failed: {error}"
                )
            }
            Self::Domain(error) => fmt::Display::fmt(error, formatter),
        }
    }
}

impl<E: StdError + 'static> StdError for PlayTerminalGatewayDetachError<E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Preparation(error) => Some(error),
            Self::Domain(error) => Some(error),
        }
    }
}

/// Owns the replaceable runtime transport for one attached play world.
///
/// The link never owns the authoring world. Its stable handle lets runtime consumers keep a
/// transport reference while attach and detach only change the play-domain route.
pub struct PlayDomainLink {
    gateway: EditorRuntimeGatewayHandle,
    attached: RwLock<Option<PlayInstanceId>>,
    next_instance_id: AtomicU64,
}

impl Default for PlayDomainLink {
    fn default() -> Self {
        Self::with_gateway_handle(EditorRuntimeGatewayHandle::detached())
    }
}

impl PlayDomainLink {
    pub(crate) fn with_gateway_handle(gateway: EditorRuntimeGatewayHandle) -> Self {
        Self {
            gateway,
            attached: RwLock::new(None),
            next_instance_id: AtomicU64::new(0),
        }
    }

    pub(crate) fn attach(
        &self,
        gateway: SharedEditorRuntimeGateway,
    ) -> Result<PlayInstanceId, PlayDomainLinkError> {
        let mut attached = self
            .attached
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(attached) = *attached {
            return Err(PlayDomainLinkError::AlreadyAttached { attached });
        }

        let instance = self.next_instance_id()?;
        self.gateway
            .replace_for_play(gateway, Some(instance.raw()))?;
        *attached = Some(instance);
        Ok(instance)
    }

    /// Detaches only when the attached play instance still names the session selected for
    /// shutdown. A stable handle may publish a replacement runtime without changing the play
    /// instance, so the session-qualified check belongs inside the attachment gate.
    pub(crate) fn detach_matching_identity(
        &self,
        instance: PlayInstanceId,
        expected_identity: &GatewaySessionIdentity,
    ) -> Result<(), PlayDomainLinkError> {
        let mut attached = self
            .attached
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *attached != Some(instance) {
            return Err(PlayDomainLinkError::AttachmentMismatch {
                requested: instance,
                attached: *attached,
            });
        }
        self.gateway
            .detach_at_identity(expected_identity)
            .map_err(|error| match error {
                GatewayError::StaleGeneration {
                    expected_generation: _,
                    current_generation: _,
                } => PlayDomainLinkError::GatewayIdentityMismatch {
                    instance,
                    expected: expected_identity.clone(),
                    current: self.gateway.identity(),
                },
                error => PlayDomainLinkError::Gateway(error),
            })?;
        *attached = None;
        Ok(())
    }

    pub fn attached_domain(&self) -> Option<WorldDomain> {
        let attached = *self
            .attached
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        attached.map(WorldDomain::Play)
    }

    pub fn gateway(&self, instance: PlayInstanceId) -> Option<EditorRuntimeGatewayHandle> {
        (*self
            .attached
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            == Some(instance))
        .then(|| self.gateway.clone())
    }

    pub(crate) fn gateway_handle(&self) -> EditorRuntimeGatewayHandle {
        self.gateway.clone()
    }

    fn next_instance_id(&self) -> Result<PlayInstanceId, PlayDomainLinkError> {
        let previous = self
            .next_instance_id
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map_err(|_| PlayDomainLinkError::InstanceIdExhausted)?;
        Ok(PlayInstanceId(previous + 1))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{DetachedEditorRuntimeGateway, PlayDomainLink, PlayDomainLinkError, WorldDomain};

    #[test]
    fn play_world_domain_roundtrip_rejects_the_reserved_zero_identity() {
        let domain = WorldDomain::Play(super::PlayInstanceId::for_test(7));
        let encoded = serde_json::to_string(&domain).expect("play domain should serialize");
        assert_eq!(
            serde_json::from_str::<WorldDomain>(&encoded).expect("play domain should roundtrip"),
            domain
        );
        assert!(serde_json::from_str::<WorldDomain>(r#"{"kind":"play","data":0}"#).is_err());
    }

    #[test]
    fn identity_guard_refuses_to_detach_a_replaced_play_gateway() {
        let link = PlayDomainLink::default();
        let instance = link
            .attach(Arc::new(DetachedEditorRuntimeGateway))
            .expect("the test play link should attach");
        let gateway = link
            .gateway(instance)
            .expect("the attached play gateway should remain reachable");
        let captured_identity = gateway.identity();
        gateway
            .replace_for_play(Arc::new(DetachedEditorRuntimeGateway), Some(instance.raw()))
            .expect("the test should replace the stable play gateway");

        let error = link
            .detach_matching_identity(instance, &captured_identity)
            .expect_err("a shutdown capture must not detach the replacement gateway");

        assert!(matches!(
            error,
            PlayDomainLinkError::GatewayIdentityMismatch { .. }
        ));
        assert_eq!(link.attached_domain(), Some(WorldDomain::Play(instance)));
    }
}

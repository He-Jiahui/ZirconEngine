use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use thiserror::Error;

use crate::core::gateway::{
    DetachedEditorRuntimeGateway, EditorRuntimeGatewayHandle, GatewayError,
    SharedEditorRuntimeGateway,
};

/// Identifies one runtime world attached to the editor play domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlayInstanceId(u64);

impl PlayInstanceId {
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Selects the authoritative world facade for an editor operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WorldDomain {
    Edit,
    Play(PlayInstanceId),
}

#[derive(Debug, Error)]
pub enum PlayDomainLinkError {
    #[error("play domain {attached:?} is already attached")]
    AlreadyAttached { attached: PlayInstanceId },
    #[error("play domain {requested:?} is not the attached domain {attached:?}")]
    AttachmentMismatch {
        requested: PlayInstanceId,
        attached: Option<PlayInstanceId>,
    },
    #[error("play instance identity space is exhausted")]
    InstanceIdExhausted,
    #[error(transparent)]
    Gateway(#[from] GatewayError),
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
        Self {
            gateway: EditorRuntimeGatewayHandle::detached(),
            attached: RwLock::new(None),
            next_instance_id: AtomicU64::new(0),
        }
    }
}

impl PlayDomainLink {
    pub fn attach(
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
        self.gateway.replace(gateway)?;
        *attached = Some(instance);
        Ok(instance)
    }

    pub fn detach(&self, instance: PlayInstanceId) -> Result<(), PlayDomainLinkError> {
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
            .replace(Arc::new(DetachedEditorRuntimeGateway))?;
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

use std::{marker::PhantomData, time::Duration, time::Instant};

use super::super::Asset;
use super::AssetEvent;
use crate::core::resource::{
    approximate_event_bytes, ResourceEventReceiver, ResourceEventRecvError,
    ResourceEventRecvTimeoutError, ResourceEventTryRecvError,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AssetEventPoll<TAsset: Asset> {
    Relevant {
        event: AssetEvent<TAsset>,
        approximate_bytes: usize,
    },
    Filtered {
        approximate_bytes: usize,
    },
}

impl<TAsset: Asset> AssetEventPoll<TAsset> {
    pub(crate) fn approximate_bytes(&self) -> usize {
        match self {
            Self::Relevant {
                approximate_bytes, ..
            }
            | Self::Filtered { approximate_bytes } => *approximate_bytes,
        }
    }
}

pub struct AssetEventReceiver<TAsset: Asset> {
    receiver: ResourceEventReceiver,
    _asset: PhantomData<fn() -> TAsset>,
}

impl<TAsset: Asset> AssetEventReceiver<TAsset> {
    fn new(receiver: ResourceEventReceiver) -> Self {
        Self {
            receiver,
            _asset: PhantomData,
        }
    }

    pub fn recv(&self) -> Result<AssetEvent<TAsset>, ResourceEventRecvError> {
        loop {
            let event = self.receiver.recv()?;
            if let Some(event) = AssetEvent::from_resource_event(event) {
                return Ok(event);
            }
        }
    }

    pub fn recv_timeout(
        &self,
        timeout: Duration,
    ) -> Result<AssetEvent<TAsset>, ResourceEventRecvTimeoutError> {
        let started = Instant::now();
        loop {
            let remaining = timeout.saturating_sub(started.elapsed());
            let event = self.receiver.recv_timeout(remaining)?;
            if let Some(event) = AssetEvent::from_resource_event(event) {
                return Ok(event);
            }
            if started.elapsed() >= timeout {
                return Err(ResourceEventRecvTimeoutError::Timeout);
            }
        }
    }

    pub fn try_recv(&self) -> Result<AssetEvent<TAsset>, ResourceEventTryRecvError> {
        loop {
            let event = self.receiver.try_recv()?;
            if let Some(event) = AssetEvent::from_resource_event(event) {
                return Ok(event);
            }
        }
    }

    pub(crate) fn try_recv_one(&self) -> Result<AssetEventPoll<TAsset>, ResourceEventTryRecvError> {
        let resource_event = self.receiver.try_recv()?;
        let approximate_bytes = approximate_event_bytes(&resource_event);
        Ok(match AssetEvent::from_resource_event(resource_event) {
            Some(event) => AssetEventPoll::Relevant {
                event,
                approximate_bytes,
            },
            None => AssetEventPoll::Filtered { approximate_bytes },
        })
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.receiver.is_empty()
    }
}

pub(crate) fn typed_event_receiver<TAsset: Asset>(
    resource_events: ResourceEventReceiver,
) -> AssetEventReceiver<TAsset> {
    AssetEventReceiver::new(resource_events)
}

use std::time::Duration;

use crossbeam_channel::{
    bounded, select, unbounded, Receiver, RecvError, RecvTimeoutError, Sender, TryRecvError,
};

use super::{Asset, Handle};
use crate::core::resource::{
    ResourceEvent, ResourceEventKind, ResourceKind, ResourceLocator, ResourceMarker,
};
use crate::core::{spawn_named_thread, ChannelReceiver};

pub struct AssetEventReceiver<TAsset: Asset> {
    receiver: Receiver<AssetEvent<TAsset>>,
    _shutdown: Sender<()>,
}

impl<TAsset: Asset> AssetEventReceiver<TAsset> {
    fn new(receiver: Receiver<AssetEvent<TAsset>>, shutdown: Sender<()>) -> Self {
        Self {
            receiver,
            _shutdown: shutdown,
        }
    }

    pub fn recv(&self) -> Result<AssetEvent<TAsset>, RecvError> {
        self.receiver.recv()
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Result<AssetEvent<TAsset>, RecvTimeoutError> {
        self.receiver.recv_timeout(timeout)
    }

    pub fn try_recv(&self) -> Result<AssetEvent<TAsset>, TryRecvError> {
        self.receiver.try_recv()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetEvent<TAsset: Asset> {
    Added {
        handle: Handle<TAsset>,
        locator: Option<ResourceLocator>,
        revision: u64,
    },
    Modified {
        handle: Handle<TAsset>,
        locator: Option<ResourceLocator>,
        revision: u64,
    },
    Removed {
        handle: Handle<TAsset>,
        locator: Option<ResourceLocator>,
        revision: u64,
    },
    Renamed {
        handle: Handle<TAsset>,
        locator: Option<ResourceLocator>,
        previous_locator: Option<ResourceLocator>,
        revision: u64,
    },
    ReloadFailed {
        handle: Handle<TAsset>,
        locator: Option<ResourceLocator>,
        revision: u64,
    },
}

impl<TAsset: Asset> AssetEvent<TAsset> {
    pub fn from_resource_event(event: ResourceEvent) -> Option<Self> {
        (event.resource_kind == TAsset::Marker::KIND).then(|| {
            let handle = Handle::new(event.id);
            match event.kind {
                ResourceEventKind::Added => Self::Added {
                    handle,
                    locator: event.locator,
                    revision: event.revision,
                },
                ResourceEventKind::Updated => Self::Modified {
                    handle,
                    locator: event.locator,
                    revision: event.revision,
                },
                ResourceEventKind::Removed => Self::Removed {
                    handle,
                    locator: event.locator,
                    revision: event.revision,
                },
                ResourceEventKind::Renamed => Self::Renamed {
                    handle,
                    locator: event.locator,
                    previous_locator: event.previous_locator,
                    revision: event.revision,
                },
                ResourceEventKind::ReloadFailed => Self::ReloadFailed {
                    handle,
                    locator: event.locator,
                    revision: event.revision,
                },
            }
        })
    }

    pub fn handle(&self) -> Handle<TAsset> {
        match self {
            Self::Added { handle, .. }
            | Self::Modified { handle, .. }
            | Self::Removed { handle, .. }
            | Self::Renamed { handle, .. }
            | Self::ReloadFailed { handle, .. } => *handle,
        }
    }

    pub fn kind(&self) -> ResourceKind {
        TAsset::Marker::KIND
    }
}

pub(crate) fn typed_event_receiver<TAsset: Asset>(
    resource_events: ChannelReceiver<ResourceEvent>,
) -> AssetEventReceiver<TAsset> {
    let (sender, receiver) = unbounded();
    let (shutdown_sender, shutdown_receiver) = bounded::<()>(0);
    let _ = spawn_named_thread(format!("asset-event-filter-{}", TAsset::LABEL), move || {
        loop {
            select! {
                recv(resource_events) -> event => match event {
                    Ok(event) => {
                        if let Some(event) = AssetEvent::<TAsset>::from_resource_event(event) {
                            if sender.send(event).is_err() {
                                break;
                            }
                        }
                    }
                    Err(_) => break,
                },
                recv(shutdown_receiver) -> _ => break,
            }
        }
    });
    AssetEventReceiver::new(receiver, shutdown_sender)
}

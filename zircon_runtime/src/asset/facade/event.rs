use std::time::Duration;

use crossbeam_channel::{
    bounded, select, unbounded, Receiver, RecvError, RecvTimeoutError, Sender, TryRecvError,
};
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetEventKind {
    Added,
    Modified,
    Removed,
    Renamed,
    ReloadFailed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::TextureAsset;
    use crate::core::resource::ResourceId;

    fn locator(value: &str) -> ResourceLocator {
        ResourceLocator::parse(value).expect("valid resource locator")
    }

    #[test]
    fn typed_asset_events_roundtrip_for_tooling_snapshots() {
        let handle = Handle::<TextureAsset>::new(ResourceId::from_stable_label(
            "typed asset event snapshot",
        ));
        let added = AssetEvent::Added {
            handle,
            locator: Some(locator("res://textures/event-snapshot.png")),
            revision: 1,
        };

        let added_json = serde_json::to_string(&added).expect("serialize added event");
        let decoded_added: AssetEvent<TextureAsset> =
            serde_json::from_str(&added_json).expect("deserialize added event");
        assert_eq!(decoded_added, added);
        assert!(added_json.contains("\"added\""));
        assert!(added_json.contains("\"revision\":1"));
        assert_eq!(added.event_kind(), AssetEventKind::Added);
        assert_eq!(
            added.locator(),
            Some(&locator("res://textures/event-snapshot.png"))
        );
        assert_eq!(added.previous_locator(), None);
        assert_eq!(added.revision(), 1);

        let renamed = AssetEvent::Renamed {
            handle,
            locator: Some(locator("res://textures/event-snapshot-renamed.png")),
            previous_locator: Some(locator("res://textures/event-snapshot.png")),
            revision: 2,
        };
        let renamed_json = serde_json::to_string(&renamed).expect("serialize renamed event");
        let decoded_renamed: AssetEvent<TextureAsset> =
            serde_json::from_str(&renamed_json).expect("deserialize renamed event");
        assert_eq!(decoded_renamed, renamed);
        assert!(renamed_json.contains("\"renamed\""));
        assert!(renamed_json.contains("event-snapshot-renamed.png"));
        assert!(renamed_json.contains("event-snapshot.png"));
        assert_eq!(decoded_renamed.handle().id(), handle.id());
        assert_eq!(decoded_renamed.event_kind(), AssetEventKind::Renamed);
        assert_eq!(
            decoded_renamed.locator(),
            Some(&locator("res://textures/event-snapshot-renamed.png"))
        );
        assert_eq!(
            decoded_renamed.previous_locator(),
            Some(&locator("res://textures/event-snapshot.png"))
        );
        assert_eq!(decoded_renamed.revision(), 2);
        assert_eq!(
            serde_json::to_string(&AssetEventKind::ReloadFailed).expect("serialize event kind"),
            "\"reload_failed\""
        );
    }
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

    pub fn event_kind(&self) -> AssetEventKind {
        match self {
            Self::Added { .. } => AssetEventKind::Added,
            Self::Modified { .. } => AssetEventKind::Modified,
            Self::Removed { .. } => AssetEventKind::Removed,
            Self::Renamed { .. } => AssetEventKind::Renamed,
            Self::ReloadFailed { .. } => AssetEventKind::ReloadFailed,
        }
    }

    pub fn locator(&self) -> Option<&ResourceLocator> {
        match self {
            Self::Added { locator, .. }
            | Self::Modified { locator, .. }
            | Self::Removed { locator, .. }
            | Self::Renamed { locator, .. }
            | Self::ReloadFailed { locator, .. } => locator.as_ref(),
        }
    }

    pub fn previous_locator(&self) -> Option<&ResourceLocator> {
        match self {
            Self::Renamed {
                previous_locator, ..
            } => previous_locator.as_ref(),
            Self::Added { .. }
            | Self::Modified { .. }
            | Self::Removed { .. }
            | Self::ReloadFailed { .. } => None,
        }
    }

    pub fn revision(&self) -> u64 {
        match self {
            Self::Added { revision, .. }
            | Self::Modified { revision, .. }
            | Self::Removed { revision, .. }
            | Self::Renamed { revision, .. }
            | Self::ReloadFailed { revision, .. } => *revision,
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

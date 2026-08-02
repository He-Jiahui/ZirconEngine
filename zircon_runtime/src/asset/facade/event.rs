use std::{marker::PhantomData, time::Duration, time::Instant};

use serde::{Deserialize, Serialize};

use super::{Asset, Handle};
use crate::core::resource::{
    ResourceEvent, ResourceEventKind, ResourceEventReceiver, ResourceEventRecvError,
    ResourceEventRecvTimeoutError, ResourceEventTryRecvError, ResourceKind, ResourceLocator,
    ResourceMarker, approximate_event_bytes,
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

    #[test]
    fn typed_asset_receiver_skips_other_resource_kinds_without_a_filter_thread() {
        let resources = crate::core::resource::ResourceManager::new();
        let typed = typed_event_receiver::<TextureAsset>(resources.subscribe());
        let shader_id = ResourceId::from_stable_label("typed event unrelated shader");
        let texture_id = ResourceId::from_stable_label("typed event target texture");
        resources.register_record(crate::core::resource::ResourceRecord::new(
            shader_id,
            ResourceKind::Shader,
            locator("res://shaders/unrelated.wgsl"),
        ));
        resources.register_record(crate::core::resource::ResourceRecord::new(
            texture_id,
            ResourceKind::Texture,
            locator("res://textures/target.png"),
        ));

        let event = typed.try_recv().expect("typed texture event");

        assert_eq!(event.handle().id(), texture_id);
        assert_eq!(event.revision(), 2);
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
    resource_events: ResourceEventReceiver,
) -> AssetEventReceiver<TAsset> {
    AssetEventReceiver::new(resource_events)
}

use super::super::{Asset, Handle};
use super::{AssetEvent, AssetEventKind};
use crate::core::resource::{
    ResourceEvent, ResourceEventKind, ResourceKind, ResourceLocator, ResourceMarker,
};

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

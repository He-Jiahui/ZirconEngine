use crate::core::resource::{ResourceRecord, ResourceState, RuntimeResourceState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetLoadState {
    NotLoaded,
    Loading,
    Loaded,
    Failed,
    Reloading,
}

impl AssetLoadState {
    pub fn is_loading_class(&self) -> bool {
        matches!(self, Self::Loading | Self::Reloading)
    }

    pub(crate) fn from_resource(
        record: Option<&ResourceRecord>,
        runtime_state: Option<RuntimeResourceState>,
        has_payload: bool,
    ) -> Self {
        let Some(record) = record else {
            return Self::NotLoaded;
        };

        if record.state == ResourceState::Error
            || matches!(runtime_state, Some(RuntimeResourceState::Error))
        {
            return Self::Failed;
        }

        if record.state == ResourceState::Reloading
            || matches!(runtime_state, Some(RuntimeResourceState::Reloading))
        {
            return Self::Reloading;
        }

        if record.state == ResourceState::Pending
            || matches!(runtime_state, Some(RuntimeResourceState::Loading))
        {
            return Self::Loading;
        }

        if record.state == ResourceState::Ready && has_payload {
            return Self::Loaded;
        }

        Self::NotLoaded
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecursiveDependencyLoadState {
    NotLoaded,
    Loading,
    Loaded,
    Failed,
    Reloading,
}

impl From<AssetLoadState> for RecursiveDependencyLoadState {
    fn from(value: AssetLoadState) -> Self {
        match value {
            AssetLoadState::NotLoaded => Self::NotLoaded,
            AssetLoadState::Loading => Self::Loading,
            AssetLoadState::Loaded => Self::Loaded,
            AssetLoadState::Failed => Self::Failed,
            AssetLoadState::Reloading => Self::Reloading,
        }
    }
}

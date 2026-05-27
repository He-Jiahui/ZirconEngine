use serde::{Deserialize, Serialize};

use crate::core::resource::{ResourceRecord, ResourceState, RuntimeResourceState};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetLoadState {
    NotLoaded,
    Loading,
    Loaded,
    Failed,
    Reloading,
}

impl AssetLoadState {
    pub fn is_not_loaded(&self) -> bool {
        matches!(self, Self::NotLoaded)
    }

    pub fn is_loading_class(&self) -> bool {
        matches!(self, Self::Loading | Self::Reloading)
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self, Self::Loaded)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed)
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyLoadState {
    NotLoaded,
    Loading,
    Loaded,
    Failed,
    Reloading,
}

impl DependencyLoadState {
    pub fn is_not_loaded(&self) -> bool {
        matches!(self, Self::NotLoaded)
    }

    pub fn is_loading_class(&self) -> bool {
        matches!(self, Self::Loading | Self::Reloading)
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self, Self::Loaded)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

impl RecursiveDependencyLoadState {
    pub fn is_not_loaded(&self) -> bool {
        matches!(self, Self::NotLoaded)
    }

    pub fn is_loading_class(&self) -> bool {
        matches!(self, Self::Loading | Self::Reloading)
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self, Self::Loaded)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetLoadStates {
    pub load_state: AssetLoadState,
    pub dependency_load_state: DependencyLoadState,
    pub recursive_dependency_load_state: RecursiveDependencyLoadState,
}

impl AssetLoadStates {
    pub fn is_loaded(&self) -> bool {
        self.load_state.is_loaded()
    }

    pub fn has_not_loaded_state(&self) -> bool {
        self.load_state.is_not_loaded()
            || self.dependency_load_state.is_not_loaded()
            || self.recursive_dependency_load_state.is_not_loaded()
    }

    pub fn is_loading_class(&self) -> bool {
        self.load_state.is_loading_class()
            || self.dependency_load_state.is_loading_class()
            || self.recursive_dependency_load_state.is_loading_class()
    }

    pub fn is_failed(&self) -> bool {
        self.load_state.is_failed()
            || self.dependency_load_state.is_failed()
            || self.recursive_dependency_load_state.is_failed()
    }

    pub fn is_loaded_with_direct_dependencies(&self) -> bool {
        self.is_loaded() && self.dependency_load_state.is_loaded()
    }

    pub fn is_loaded_with_dependencies(&self) -> bool {
        self.is_loaded_with_direct_dependencies()
            && matches!(
                self.recursive_dependency_load_state,
                RecursiveDependencyLoadState::Loaded
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_load_states_classification_helpers_cover_tooling_status_rows() {
        let loaded = AssetLoadStates {
            load_state: AssetLoadState::Loaded,
            dependency_load_state: DependencyLoadState::Loaded,
            recursive_dependency_load_state: RecursiveDependencyLoadState::Loaded,
        };
        assert!(loaded.is_loaded_with_dependencies());
        assert!(!loaded.has_not_loaded_state());
        assert!(!loaded.is_loading_class());
        assert!(!loaded.is_failed());

        let reloading_dependency = AssetLoadStates {
            load_state: AssetLoadState::Loaded,
            dependency_load_state: DependencyLoadState::Loaded,
            recursive_dependency_load_state: RecursiveDependencyLoadState::Reloading,
        };
        assert!(reloading_dependency.is_loaded_with_direct_dependencies());
        assert!(!reloading_dependency.is_loaded_with_dependencies());
        assert!(!reloading_dependency.has_not_loaded_state());
        assert!(reloading_dependency.is_loading_class());
        assert!(!reloading_dependency.is_failed());

        let failed_direct_dependency = AssetLoadStates {
            load_state: AssetLoadState::Loaded,
            dependency_load_state: DependencyLoadState::Failed,
            recursive_dependency_load_state: RecursiveDependencyLoadState::Failed,
        };
        assert!(failed_direct_dependency.is_loaded());
        assert!(!failed_direct_dependency.is_loaded_with_direct_dependencies());
        assert!(!failed_direct_dependency.has_not_loaded_state());
        assert!(!failed_direct_dependency.is_loading_class());
        assert!(failed_direct_dependency.is_failed());

        let missing_root = AssetLoadStates {
            load_state: AssetLoadState::NotLoaded,
            dependency_load_state: DependencyLoadState::NotLoaded,
            recursive_dependency_load_state: RecursiveDependencyLoadState::NotLoaded,
        };
        assert!(!missing_root.is_loaded());
        assert!(missing_root.has_not_loaded_state());
        assert!(!missing_root.is_loading_class());
        assert!(!missing_root.is_failed());
    }
}

impl From<AssetLoadState> for DependencyLoadState {
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

use thiserror::Error;

use zircon_runtime::asset::AssetImportError;
use zircon_runtime::core::resource::ResourceLocatorError;
use zircon_runtime::core::CoreError;
use zircon_runtime::scene::world::SceneProjectError;

use crate::core::asset::DirtyRegistryError;
use crate::core::extension::{
    SaveError, ToolkitInstanceIdError, ToolkitLayoutError, ToolkitRegistryError,
};
use crate::core::project::ProjectAuthorityError;

#[derive(Debug, Error)]
pub enum EditorError {
    #[error("{0}")]
    Layout(String),
    #[error("{0}")]
    Registry(String),
    #[error("document toolkit instance {instance:?} is not registered")]
    DocumentToolkitNotRegistered { instance: String },
    #[error("document toolkit registry failed: {source}")]
    DocumentToolkitRegistry {
        #[from]
        #[source]
        source: ToolkitRegistryError,
    },
    #[error("document toolkit save failed: {source}")]
    DocumentToolkitSave {
        #[from]
        #[source]
        source: SaveError,
    },
    #[error("document toolkit instance id failed: {source}")]
    DocumentToolkitInstanceId {
        #[from]
        #[source]
        source: ToolkitInstanceIdError,
    },
    #[error("document toolkit layout failed: {source}")]
    DocumentToolkitLayout {
        #[from]
        #[source]
        source: ToolkitLayoutError,
    },
    #[error("document dirty registry failed: {source}")]
    DirtyRegistry {
        #[from]
        #[source]
        source: DirtyRegistryError,
    },
    #[error("{0}")]
    Project(String),
    #[error("Hub focus was forwarded to active editor process {process_id}")]
    HubFocusForwarded { process_id: u32 },
    #[error("project authority failed: {source}")]
    ProjectAuthority {
        #[from]
        #[source]
        source: ProjectAuthorityError,
    },
    #[error("{0}")]
    UiAsset(String),
    #[error("asset import failed: {source}")]
    AssetImport {
        #[from]
        #[source]
        source: AssetImportError,
    },
    #[error("resource locator failed: {source}")]
    ResourceLocator {
        #[from]
        #[source]
        source: ResourceLocatorError,
    },
    #[error("runtime service failed: {source}")]
    Core {
        #[from]
        #[source]
        source: CoreError,
    },
    #[error("project document failed: {source}")]
    SceneProject {
        #[from]
        #[source]
        source: SceneProjectError,
    },
    #[error("asset watcher failed: {source}")]
    AssetWatcher {
        #[from]
        #[source]
        source: notify::Error,
    },
}

impl EditorError {
    pub(crate) const fn hub_focus_forwarded_process_id(&self) -> Option<u32> {
        match self {
            Self::HubFocusForwarded { process_id } => Some(*process_id),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::EditorError;
    use zircon_runtime::asset::AssetImportError;
    use zircon_runtime::core::resource::{ResourceLocator, ResourceLocatorError};

    #[test]
    fn typed_asset_and_uri_errors_remain_in_the_editor_error_source_chain() {
        let asset_error: EditorError = AssetImportError::MissingProjectAssetRoot.into();
        assert!(asset_error
            .source()
            .is_some_and(|source| source.downcast_ref::<AssetImportError>().is_some()));

        let uri_source = ResourceLocator::parse("not-a-resource-uri").unwrap_err();
        let uri_error: EditorError = uri_source.into();
        assert!(uri_error
            .source()
            .is_some_and(|source| source.downcast_ref::<ResourceLocatorError>().is_some()));
    }

    #[test]
    fn hub_focus_forwarding_exposes_the_existing_editor_process_id() {
        let error = EditorError::HubFocusForwarded { process_id: 913 };

        assert_eq!(error.hub_focus_forwarded_process_id(), Some(913));
        assert_eq!(
            EditorError::Project("other".to_string()).hub_focus_forwarded_process_id(),
            None
        );
    }
}

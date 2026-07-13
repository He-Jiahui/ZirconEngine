use thiserror::Error;

use zircon_runtime::asset::AssetImportError;
use zircon_runtime::core::resource::ResourceLocatorError;
use zircon_runtime::core::CoreError;
use zircon_runtime::scene::world::SceneProjectError;

use crate::core::project::ProjectAuthorityError;

#[derive(Debug, Error)]
pub enum EditorError {
    #[error("{0}")]
    Layout(String),
    #[error("{0}")]
    Registry(String),
    #[error("{0}")]
    Project(String),
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
}

//! Asset data and resource handles available without desktop authoring services.

#[path = "assets/mod.rs"]
pub mod assets;
#[path = "exports.rs"]
mod exports;
#[path = "facade/mod.rs"]
pub mod facade;
#[path = "reference_resolution_error.rs"]
mod reference_resolution_error;
#[path = "pipeline/types.rs"]
pub(crate) mod types;

pub use exports::*;
pub use facade::{
    Asset, AssetDependencyReadiness, AssetEvent, AssetEventKind, AssetEventReceiver,
    AssetLoadState, AssetLoadStates, AssetReadinessNode, AssetReadinessReport, Assets,
    DependencyLoadState, Handle, RecursiveDependencyLoadState,
};
pub use reference_resolution_error::ReferenceResolutionError;

pub mod prelude {
    pub use super::facade::{Asset, Assets, Handle};
    pub use super::{
        AssetId, AssetKind, AssetReference, AssetUri, AssetUuid, FontAsset, ImportedAsset,
        MaterialAsset, MeshAsset, MeshVertex, ModelAsset, TextureAsset,
    };
}

#[cfg(feature = "cooked-assets")]
#[path = "runtime/mod.rs"]
pub mod runtime;
#[cfg(feature = "cooked-assets")]
pub use runtime::{runtime_asset_manager_handle, RuntimeAssetManager, RuntimeAssetManagerAccess};
// Compatibility for existing renderer callers; this implementation has no authoring services.
#[cfg(feature = "cooked-assets")]
pub use runtime::{
    runtime_asset_manager_handle as project_asset_manager_handle,
    RuntimeAssetManager as ProjectAssetManager,
    RuntimeAssetManagerAccess as ProjectAssetManagerAccess,
};
#[cfg(feature = "cooked-assets")]
pub const PROJECT_ASSET_MANAGER_NAME: &str = "AssetModule.Manager.ProjectAssetManager";
#[cfg(feature = "cooked-assets")]
#[path = "portable_artifact.rs"]
pub mod artifact;
#[cfg(feature = "cooked-assets")]
#[path = "codec_error.rs"]
mod codec_error;
#[cfg(feature = "cooked-assets")]
pub(crate) use codec_error::AssetCodecError;

#[path = "runtime/change_types.rs"]
pub mod watch;
pub const ASSET_MODULE_NAME: &str = "AssetModule";

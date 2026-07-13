//! Canonical editor-side asset type contracts.

mod source_authority;
mod toolkit_route;
mod type_registry;

pub use source_authority::{
    AssetSourceAuthority, AssetSourceKind, AssetSourceWritePolicy, AssetWriteAccess,
};
pub use toolkit_route::AssetToolkitOpenRoute;
pub use type_registry::{
    builtin_asset_type_definition, AssetContextCommandAccess, AssetContextCommandDescriptor,
    AssetCreationTemplateDescriptor, AssetToolkitDescriptor, AssetTypeContribution,
    AssetTypeDefinition, AssetTypeId, AssetTypeIdError, AssetTypePresentation, AssetTypeRegistry,
    AssetTypeRegistryError, ThumbnailPlaceholderPalette, ThumbnailProviderDescriptor,
};

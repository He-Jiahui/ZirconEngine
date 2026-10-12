#[path = "asset_type_id.rs"]
mod asset_type_id;
#[path = "builtin.rs"]
mod builtin;
#[path = "context_command.rs"]
mod context_command;
#[path = "contribution.rs"]
mod contribution;
#[path = "creation_template.rs"]
mod creation_template;
#[path = "definition.rs"]
mod definition;
#[path = "error.rs"]
mod error;
#[path = "presentation.rs"]
mod presentation;
#[path = "registry.rs"]
mod registry;
#[path = "thumbnail_provider.rs"]
mod thumbnail_provider;
#[path = "toolkit.rs"]
mod toolkit;

pub use asset_type_id::{AssetTypeId, AssetTypeIdError};
pub use builtin::builtin_asset_type_definition;
pub use context_command::{AssetContextCommandAccess, AssetContextCommandDescriptor};
pub use contribution::AssetTypeContribution;
pub use creation_template::AssetCreationTemplateDescriptor;
pub use definition::AssetTypeDefinition;
pub use error::AssetTypeRegistryError;
pub use presentation::AssetTypePresentation;
pub use registry::{AssetCreationMenuEntry, AssetCreationMenuGeneration, AssetTypeRegistry};
pub use thumbnail_provider::{ThumbnailPlaceholderPalette, ThumbnailProviderDescriptor};
pub use toolkit::AssetToolkitDescriptor;

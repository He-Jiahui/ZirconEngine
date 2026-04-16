mod asset_reference;
mod asset_uuid;
mod stable_uuid;

pub use asset_reference::AssetReference;
pub use asset_uuid::AssetUuid;
pub(crate) use stable_uuid::stable_uuid_from_components;

#[path = "layout.rs"]
mod layout;
#[path = "validation.rs"]
mod validation;

pub use layout::{
    SpriteAtlasAsset, SpriteAtlasEntry, SpriteAtlasPadding, SpriteAtlasRect, SpriteAtlasUvRect,
};
pub use validation::{validate_sprite_atlas_asset, SpriteAtlasValidationError};

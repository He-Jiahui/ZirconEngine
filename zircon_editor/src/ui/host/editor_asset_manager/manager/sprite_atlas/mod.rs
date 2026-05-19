mod artifact;
mod config;
mod diagnostics;
mod packer;

pub use artifact::{write_sprite_atlas_artifacts, SpriteAtlasArtifactReport};
pub use config::SpriteAtlasBuildConfig;
pub use diagnostics::SpriteAtlasBuildDiagnostics;
pub use packer::{
    decode_sprite_atlas_source_image, pack_sprite_atlas_sources, PackedSpriteAtlas,
    SpriteAtlasBuildError, SpriteAtlasSourceImage,
};

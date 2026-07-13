//! Feature-gated text API used by the offline `.zsdf` command-line tool.

mod bake;
mod error;
mod inspect;
mod pack;
mod request;

pub use bake::{bake_font_sdf_artifact, FontSdfBakeArtifact, FontSdfBakeReport};
pub use error::FontSdfBakeError;
pub use inspect::{inspect_font_sdf_artifact, FontSdfArtifactInspection};
pub use request::{FontSdfBakeMode, FontSdfBakeRequest, FontSdfGlyphSelection};

//! Feature-gated text API used by the offline `.zsdf` command-line tool.

mod bake;
mod error;
mod inspect;
mod pack;
mod request;

pub use bake::{FontSdfBakeArtifact, FontSdfBakeReport, bake_font_sdf_artifact};
pub use error::FontSdfBakeError;
pub use inspect::{FontSdfArtifactInspection, inspect_font_sdf_artifact};
pub use request::{FontSdfBakeMode, FontSdfBakeRequest, FontSdfGlyphSelection};

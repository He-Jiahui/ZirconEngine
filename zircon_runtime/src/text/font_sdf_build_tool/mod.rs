//! Feature-gated text API used by the offline `.zsdf` command-line tool.

#[path = "bake.rs"]
mod bake;
#[path = "error.rs"]
mod error;
#[path = "inspect.rs"]
mod inspect;
#[path = "pack.rs"]
mod pack;
#[path = "request.rs"]
mod request;

pub use bake::{bake_font_sdf_artifact, FontSdfBakeArtifact, FontSdfBakeReport};
pub use error::FontSdfBakeError;
pub use inspect::{inspect_font_sdf_artifact, FontSdfArtifactInspection};
pub use request::{FontSdfBakeMode, FontSdfBakeRequest, FontSdfGlyphSelection};

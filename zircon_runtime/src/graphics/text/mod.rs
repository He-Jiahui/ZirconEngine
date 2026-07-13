//! Shared runtime text services used by UI and render-side text consumers.

pub(crate) mod atlas;
pub(crate) mod cache;
pub(crate) mod font;
#[cfg(feature = "font-sdf-build-tool")]
pub mod font_sdf_build_tool;
pub(crate) mod layout;
pub(crate) mod parallel;
pub(crate) mod raster;
pub(crate) mod rich;
pub(crate) mod sdf;
pub(crate) mod shaping;

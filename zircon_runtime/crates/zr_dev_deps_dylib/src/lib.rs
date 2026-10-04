//! Development-only linkage anchor for heavy Runtime dependencies.

#[cfg(feature = "runtime-assets")]
#[allow(unused_imports, clippy::single_component_path_imports)]
use gltf as _;
#[cfg(feature = "runtime-assets")]
#[allow(unused_imports, clippy::single_component_path_imports)]
use image as _;
#[cfg(feature = "runtime-assets")]
#[allow(unused_imports, clippy::single_component_path_imports)]
use meshopt as _;
#[cfg(feature = "naga")]
#[allow(unused_imports, clippy::single_component_path_imports)]
use naga as _;
#[cfg(feature = "wgpu")]
#[allow(unused_imports, clippy::single_component_path_imports)]
use wgpu as _;

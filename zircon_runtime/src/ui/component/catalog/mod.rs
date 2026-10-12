#[path = "editor_showcase.rs"]
mod editor_showcase;
#[path = "material_foundation/mod.rs"]
mod material_foundation;
#[path = "palette_view.rs"]
mod palette_view;
#[path = "registry.rs"]
mod registry;

pub use palette_view::UiComponentPaletteEntry;
pub use registry::UiComponentDescriptorRegistry;

#[path = "catalog.rs"]
mod catalog;
#[path = "entry.rs"]
mod entry;
#[path = "instantiate.rs"]
mod instantiate;
#[path = "native_slots.rs"]
mod native_slots;
#[path = "placement.rs"]
mod placement;

pub(crate) use catalog::UiAssetPaletteCatalog;
pub(crate) use entry::{UiAssetPaletteEntry, UiAssetPaletteEntryKind};
pub(crate) use instantiate::{
    can_convert_selected_node_to_reference, convert_selected_node_to_reference,
    insert_palette_item_with_placement, node_accepts_palette_children,
};
pub(crate) use placement::{PaletteInsertMode, UiAssetPaletteInsertionPlacement};

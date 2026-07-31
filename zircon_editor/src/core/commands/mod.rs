//! Editor command registry, keymap, and command-palette projection.

mod asset_write_target;
mod contribution;
mod defaults;
mod descriptor;
mod document_kind;
mod eval_snapshot_handle;
mod key_chord;
mod keymap;
mod menu;
mod menu_model;
mod palette;
mod play_mode_predicate;
mod registry;
mod registry_handle;
mod when;

pub use asset_write_target::AssetWriteTargetDescriptor;
pub use contribution::EditorCommandContributionSet;
pub use descriptor::{
    EditorCommandAction, EditorCommandCategory, EditorCommandDescriptor,
    EditorCommandMenuProjection,
};
pub use document_kind::{DocumentKind, DocumentKindError};
pub use eval_snapshot_handle::CommandEvalSnapshotHandle;
pub use key_chord::{EditorKeyChord, EditorKeyChordParseError};
pub use keymap::{EditorKeyBinding, EditorKeymap, EditorKeymapConflict, EditorKeymapError};
pub use menu_model::{MenuBarModel, MenuItemModel, MenuModel};
pub use palette::{
    EditorCommandPaletteCatalog, EditorCommandPaletteEntry, EditorCommandPaletteMru,
    EditorCommandPaletteQueryMetrics, EditorCommandPaletteQueryWindow,
};
pub use play_mode_predicate::PlayModePredicate;
pub use registry::{EditorCommandDispatchError, EditorCommandRegistry, EditorCommandRegistryError};
pub use registry_handle::EditorCommandRegistryHandle;
pub use when::{CommandEvalCtx, WhenClause};

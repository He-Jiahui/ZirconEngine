//! Editor command registry, keymap, and command-palette projection.

mod asset_write_target;
mod contribution;
mod defaults;
mod descriptor;
mod document_kind;
mod eval_snapshot_handle;
mod execution;
mod key_chord;
mod keymap;
mod menu;
mod menu_model;
mod palette;
mod play_mode_predicate;
mod presentation;
mod registry;
mod registry_handle;
mod when;

pub use asset_write_target::AssetWriteTargetDescriptor;
pub use contribution::EditorCommandContributionSet;
pub(crate) use contribution::project_command_registry_from_contributions;
pub use descriptor::{
    EditorCommandAction, EditorCommandCategory, EditorCommandDescriptor,
    EditorCommandMenuProjection,
};
pub use document_kind::{DocumentKind, DocumentKindError};
pub use eval_snapshot_handle::CommandEvalSnapshotHandle;
pub use execution::{
    EditorCommandExecutionContract, EditorCommandExecutionReceipt, EditorCommandExecutorRegistry,
    EditorCommandExecutorRegistryError, EditorCommandResourceBudget,
    EditorCommandResourceBudgetError, EditorCommandResultCodecId, EditorCommandResultCodecIdError,
    MAX_EDITOR_COMMAND_EXECUTION_TIME_MS, MAX_EDITOR_COMMAND_INPUT_BYTES,
    MAX_EDITOR_COMMAND_OUTPUT_BYTES, NativeCommandExecutorRegistration,
    NativePluginEditorCommandBinding,
};
pub use key_chord::{EditorKeyChord, EditorKeyChordParseError};
pub(crate) use key_chord::{EditorKeyChordSignature, EditorKeyboardChordInput};
pub use keymap::{EditorKeyBinding, EditorKeymap, EditorKeymapConflict, EditorKeymapError};
pub use menu_model::{MenuBarModel, MenuItemModel, MenuModel};
pub use palette::{
    EditorCommandPaletteCatalog, EditorCommandPaletteEntry, EditorCommandPaletteMru,
    EditorCommandPaletteQueryMetrics, EditorCommandPaletteQueryWindow,
};
pub use play_mode_predicate::PlayModePredicate;
pub use presentation::{
    EditorCommandLocalizationSource, EditorCommandMenuPath, EditorCommandMenuSegment,
    EditorCommandMenuSegmentId, EditorCommandPresentation,
};
pub use registry::{EditorCommandDispatchError, EditorCommandRegistry, EditorCommandRegistryError};
pub use registry_handle::EditorCommandRegistryHandle;
pub use when::{CommandEvalCtx, WhenClause};

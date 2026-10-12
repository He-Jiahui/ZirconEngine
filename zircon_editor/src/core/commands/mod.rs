//! Editor command registry, keymap, and command-palette projection.

#[path = "asset_write_target.rs"]
mod asset_write_target;
#[path = "contribution.rs"]
mod contribution;
#[path = "defaults.rs"]
mod defaults;
#[path = "descriptor.rs"]
mod descriptor;
#[path = "document_kind.rs"]
mod document_kind;
#[path = "eval_snapshot_handle.rs"]
mod eval_snapshot_handle;
#[path = "execution.rs"]
mod execution;
#[path = "key_chord.rs"]
mod key_chord;
#[path = "keymap.rs"]
mod keymap;
#[path = "menu.rs"]
mod menu;
#[path = "menu_model.rs"]
mod menu_model;
#[path = "palette.rs"]
mod palette;
#[path = "play_mode_predicate.rs"]
mod play_mode_predicate;
#[path = "presentation/mod.rs"]
mod presentation;
#[path = "registry.rs"]
mod registry;
#[path = "registry_handle.rs"]
mod registry_handle;
#[path = "when.rs"]
mod when;

pub use asset_write_target::AssetWriteTargetDescriptor;
pub(crate) use contribution::project_command_registry_from_contributions;
pub use contribution::EditorCommandContributionSet;
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
    NativeCommandExecutorRegistration, NativePluginEditorCommandBinding,
    MAX_EDITOR_COMMAND_EXECUTION_TIME_MS, MAX_EDITOR_COMMAND_INPUT_BYTES,
    MAX_EDITOR_COMMAND_OUTPUT_BYTES,
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

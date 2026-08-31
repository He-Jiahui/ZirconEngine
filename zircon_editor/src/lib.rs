//! Editor host UI built on Retained, with viewport frames coming from core graphics.

pub mod core;
pub mod scene;
pub mod ui;

pub use core::commands::{
    CommandEvalCtx, CommandEvalSnapshotHandle, DocumentKind, DocumentKindError,
    EditorCommandAction, EditorCommandCategory, EditorCommandDescriptor,
    EditorCommandDispatchError, EditorCommandExecutionContract, EditorCommandExecutionReceipt,
    EditorCommandExecutorRegistry, EditorCommandExecutorRegistryError, EditorCommandPaletteEntry,
    EditorCommandRegistry, EditorCommandRegistryError, EditorCommandResourceBudget,
    EditorCommandResourceBudgetError, EditorCommandResultCodecId, EditorCommandResultCodecIdError,
    EditorKeyBinding, EditorKeyChord, EditorKeyChordParseError, EditorKeymap, EditorKeymapConflict,
    EditorKeymapError, NativeCommandExecutorRegistration, NativePluginEditorCommandBinding,
    PlayModePredicate, WhenClause, MAX_EDITOR_COMMAND_EXECUTION_TIME_MS,
    MAX_EDITOR_COMMAND_INPUT_BYTES, MAX_EDITOR_COMMAND_OUTPUT_BYTES,
};
pub use core::editing::intent::EditorIntent;
pub use core::gateway::{
    DetachedEditorRuntimeGateway, EditorRuntimeFrame, EditorRuntimeGateway,
    EditorRuntimeGatewayHandle, GatewayError, InProcessGateway, PluginActivationState,
    PluginSummaryEntry, RuntimeCapabilities, SessionGateway, SessionProfileKind,
    SharedEditorRuntimeGateway,
};
pub use core::gui_startup_request::EditorGuiStartupRequest;
pub use core::plugin::{
    EditorExtensionCatalogReport, EditorPlugin, EditorPluginDescriptor,
    EditorPluginRegistrationReport,
};
pub use ui::host::module::{
    module_descriptor, EditorHostDriver, EditorModule, EDITOR_ASSET_MANAGER_NAME,
    EDITOR_COMMAND_REGISTRY_NAME, EDITOR_HOST_DRIVER_NAME, EDITOR_KEYMAP_NAME, EDITOR_MANAGER_NAME,
    EDITOR_MODULE_NAME,
};
pub use ui::retained_host::{
    run_editor, run_editor_with_config, run_editor_with_startup_request,
    run_retained_host_automation, EditorHostRunConfig, RetainedHostAutomationResult,
};

#[cfg(test)]
mod tests;

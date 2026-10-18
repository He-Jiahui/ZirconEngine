//! Editor extension contracts shared by headless authoring and UI hosts.

#[path = "inspector.rs"]
mod inspector;
#[path = "settings_page_projection.rs"]
mod settings_page_projection;
#[path = "slots.rs"]
mod slots;
#[path = "store.rs"]
mod store;
#[path = "toolkit/mod.rs"]
mod toolkit;

pub use inspector::{
    FieldEditorContainer, FieldEditorDefinition, FieldEditorFactory, FieldEditorInit,
    FieldEditorInstance, FieldEditorKind, InspectTarget, InspectTargetType, InspectorCustomization,
    InspectorCustomizationChain, InspectorCustomizationDescriptor, InspectorCustomizationSurface,
    InspectorField, InspectorLayout, InspectorLayoutBuilder, InspectorLayoutRow,
    InspectorRegistrationError,
};
pub use settings_page_projection::{
    LocalizedSettingsCategory, LocalizedSettingsPage, SettingsPageProjection,
};
pub use slots::{DefaultWorkbenchPreset, WorkbenchSlot};
pub use store::{
    CapabilitySet, ContributionBatch, ContributionChange, ContributionChangeKind,
    ContributionCounts, ContributionDelta, ContributionError, ContributionSnapshot,
    ContributionSource, ContributionStore, ContributionTicket, PluginContributionId, RevokeReport,
};
pub use toolkit::{
    DocumentAutosavePayload, DocumentCloseLease, DocumentEditLease, DocumentSaveReport,
    DocumentToolkit, DocumentToolkitDescriptor, DocumentToolkitRegistry, DocumentToolkitSnapshot,
    SaveContextError, SaveCtx, SaveError, SaveHookPanic, SaveReason, ToolkitArea, ToolkitAreaSlot,
    ToolkitInstanceId, ToolkitInstanceIdError, ToolkitLayout, ToolkitLayoutError,
    ToolkitRegistryError, ToolkitSaveFailure,
};
pub(crate) use toolkit::{
    DocumentCloseAdmission, DocumentSourceWritePublication, DocumentSourceWriteReceipt,
};

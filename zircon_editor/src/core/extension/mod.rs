//! Editor extension contracts shared by headless authoring and UI hosts.

mod inspector;
mod settings_page_projection;
mod slots;
mod store;
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
    DocumentAutosavePayload, DocumentCloseLease, DocumentSaveReport, DocumentToolkit,
    DocumentToolkitDescriptor, DocumentToolkitRegistry, DocumentToolkitSnapshot, SaveContextError,
    SaveCtx, SaveError, SaveReason, ToolkitArea, ToolkitAreaSlot, ToolkitInstanceId,
    ToolkitInstanceIdError, ToolkitLayout, ToolkitLayoutError, ToolkitRegistryError,
    ToolkitSaveFailure,
};
pub(crate) use toolkit::{DocumentSourceWritePublication, DocumentSourceWriteReceipt};

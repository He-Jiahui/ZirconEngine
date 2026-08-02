//! Editor extension contracts shared by headless authoring and UI hosts.

mod inspector;
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
pub use slots::{DefaultWorkbenchPreset, WorkbenchSlot};
pub use store::{
    CapabilitySet, ContributionBatch, ContributionChange, ContributionChangeKind,
    ContributionCounts, ContributionDelta, ContributionError, ContributionSnapshot,
    ContributionSource, ContributionStore, ContributionTicket, PluginContributionId, RevokeReport,
};
pub use toolkit::{
    DocumentCloseLease, DocumentSaveReport, DocumentToolkit, DocumentToolkitDescriptor,
    DocumentToolkitRegistry, DocumentToolkitSnapshot, SaveContextError, SaveCtx, SaveError,
    SaveReason, ToolkitArea, ToolkitAreaSlot, ToolkitInstanceId, ToolkitInstanceIdError,
    ToolkitLayout, ToolkitLayoutError, ToolkitRegistryError, ToolkitSaveFailure,
};

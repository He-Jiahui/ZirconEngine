//! Editor-plugin descriptor, lifecycle, and catalog boundaries.

mod admission;
mod capability_report;
mod catalog;
mod catalog_gen;
mod catalog_snapshot;
mod catalog_store;
mod descriptor;
mod extension_catalog_report;
mod extension_materialization;
mod isolation;
mod lifecycle_message_bridge;
mod manager;
mod materializer;
mod panel_source;
mod phases;
mod projection;
mod registration;
pub mod sdk;

// Catalog read and validation surface.
pub use admission::EditorPluginCatalogAdmissionError;
pub use capability_report::EditorCapabilityReport;
pub(crate) use catalog::EditorPluginCatalog;
pub use catalog::EditorPluginHandle;
pub use catalog_snapshot::EditorPluginCatalogSnapshot;
pub use extension_catalog_report::EditorExtensionCatalogReport;
pub use isolation::{EditorPluginBoundaryFailure, run_editor_plugin_boundary};
pub use lifecycle_message_bridge::{
    EditorPluginLifecycleMessageBridge, EditorPluginLifecycleMessagePumpReport,
};
pub use manager::EditorPluginManager;
pub use manager::{
    EditorPluginDiscovery, EditorPluginDiscoveryError, EditorPluginSource, EditorPluginState,
    EditorPluginTransitionError,
};
pub use materializer::{
    SerializedContributionMaterializationError, materialize_serialized_contribution_batch,
};
pub use panel_source::{EditorPluginPanelRow, EditorPluginPanelSource};
pub use phases::EditorPluginLoadingPhase;
pub use projection::{EditorPluginCatalogEntry, EditorPluginCatalogProjection};

// Plugin declaration and lifecycle surface.
pub use descriptor::{EditorPlugin, EditorPluginDescriptor};
pub use registration::EditorPluginRegistrationReport;

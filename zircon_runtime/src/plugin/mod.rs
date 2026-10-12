#[path = "bridge.rs"]
mod bridge;
#[path = "capability_status.rs"]
mod capability_status;
#[path = "core_profiles.rs"]
mod core_profiles;
#[path = "export_build_plan/mod.rs"]
mod export_build_plan;
#[path = "extension_registry/mod.rs"]
mod extension_registry;
#[path = "extension_registry_error.rs"]
mod extension_registry_error;
#[path = "native.rs"]
pub mod native;
#[path = "loader/mod.rs"]
mod native_plugin_loader;
#[path = "package_manifest/mod.rs"]
mod package_manifest;
#[path = "package_service/mod.rs"]
pub mod package_service;
#[path = "plugin_maturity.rs"]
mod plugin_maturity;
#[path = "runtime/mod.rs"]
mod runtime_plugin;
#[path = "runtime_profile.rs"]
mod runtime_profile;
#[path = "ui_component_descriptor.rs"]
mod ui_component_descriptor;

pub use bridge::{
    BridgeDiagnosticsMatrix, BridgeEntry, BridgeImport, BridgeInterfaceSnapshot,
    BridgeOwnerTransitionReport, BridgeTableDiagnosticsSummary, FrozenBridgeTable, WeakBridge,
};
pub use capability_status::{CapabilityStatus, CapabilityStatusManifest};
pub use core_profiles::{EditorCoreProfile, RuntimeCoreProfile};
pub use export_build_plan::{
    ExportBuildPlan, ExportBuildPlanError, ExportGeneratedFile, ExportLinkedFeatureSourceReceipt,
    ExportMaterializeReport, ExportPlanAdmissionProof, ExportValidateGeneratedFileSummary,
    ExportValidatePlanSummary, ExportValidateProfileSummary, ExportValidateReport,
    LibraryEmbedCompileHostPlan, LibraryEmbedCompileHostTarget, LibraryEmbedLinkedRuntimeCrate,
    NativeDynamicPackageAbiV3Contract, NativeDynamicPackageExportPlan,
};
pub use extension_registry::{
    ExtensionKey, ExtensionOwnership, ExtensionSlot, FrozenExtensionTable, PluginModuleId,
    RuntimeExtensionRegistry, TypedExtensionPoint,
};
pub use extension_registry_error::RuntimeExtensionRegistryError;
pub use package_manifest::{
    PluginDependencyManifest, PluginDistributionManifest, PluginEventCatalogManifest,
    PluginEventConsumerManifest, PluginEventManifest, PluginFeatureBundleManifest,
    PluginFeatureDependency, PluginInterfaceManifest, PluginInterfaceMethodManifest,
    PluginModuleKind, PluginModuleManifest, PluginOptionManifest, PluginPackageKind,
    PluginPackageManifest, PluginPackageRole, PluginShaderModuleManifest, PluginShaderModuleSource,
    PluginShaderPermutationIdManifest, PluginShaderPermutationManifest, ShaderModuleSourceBinding,
};
pub use plugin_maturity::PluginMaturity;
pub use runtime_plugin::{
    CapabilityView, CompiledProjectPluginPlan, PluginCatalogGeneration,
    RuntimeExtensionCatalogReport, RuntimePlugin, RuntimePluginBridgeDependent,
    RuntimePluginBridgeDisableBlocker, RuntimePluginBridgeLifecycleBlock,
    RuntimePluginBridgeLifecycleError, RuntimePluginBridgeLifecycleEvent,
    RuntimePluginBridgeLifecycleOutcome, RuntimePluginBridgeLifecycleReport,
    RuntimePluginBridgeLifecycleState, RuntimePluginCatalog, RuntimePluginCatalogAuthority,
    RuntimePluginCatalogCandidate, RuntimePluginCatalogPreparationError,
    RuntimePluginCatalogPreparedGeneration, RuntimePluginCatalogProjectPlanCacheMetrics,
    RuntimePluginCatalogProjectPlanMetrics, RuntimePluginCatalogProjectionMetrics,
    RuntimePluginCatalogPublicationError, RuntimePluginCatalogSnapshot, RuntimePluginCatalogUpdate,
    RuntimePluginCatalogUpdateMetrics, RuntimePluginCatalogUpdateOutcome, RuntimePluginDescriptor,
    RuntimePluginDescriptorBuilder, RuntimePluginFeature, RuntimePluginFeatureBlock,
    RuntimePluginFeatureDependencyReport, RuntimePluginFeatureRegistrationReport,
    RuntimePluginModuleProposal, RuntimePluginRegistrationReport,
};
pub use runtime_profile::{
    RuntimePluginAvailabilityCategory, RuntimePluginAvailabilityEntry,
    RuntimePluginAvailabilityGeneration, RuntimePluginAvailabilityReport,
    RuntimePluginAvailabilityRow, RuntimePluginAvailabilitySummary, RuntimeProfileDescriptor,
    RuntimeProfileFeaturePreset, RuntimeProfilePluginSelection, RUNTIME_PROFILE_FEATURE_PRESETS,
};
pub use ui_component_descriptor::UiComponentDescriptor;

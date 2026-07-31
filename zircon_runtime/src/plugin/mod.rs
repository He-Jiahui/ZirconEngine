mod bridge;
mod capability_status;
mod core_profiles;
mod export_build_plan;
mod extension_registry;
mod extension_registry_error;
pub mod native;
mod native_plugin_loader;
mod package_manifest;
mod plugin_maturity;
mod runtime_plugin;
mod runtime_profile;
mod ui_component_descriptor;

pub use bridge::{
    BridgeDiagnosticsMatrix, BridgeEntry, BridgeImport, BridgeInterfaceSnapshot,
    BridgeOwnerTransitionReport, BridgeTableDiagnosticsSummary, FrozenBridgeTable, WeakBridge,
};
pub use capability_status::{CapabilityStatus, CapabilityStatusManifest};
pub use core_profiles::{EditorCoreProfile, RuntimeCoreProfile};
pub use export_build_plan::{
    ExportBuildPlan, ExportBuildPlanError, ExportGeneratedFile, ExportMaterializeReport,
    ExportValidateGeneratedFileSummary, ExportValidatePlanSummary, ExportValidateProfileSummary,
    ExportValidateReport, LibraryEmbedCompileHostPlan, LibraryEmbedCompileHostTarget,
    LibraryEmbedLinkedRuntimeCrate, NativeDynamicPackageAbiV3Contract,
    NativeDynamicPackageExportPlan,
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
    PluginPackageManifest, PluginShaderModuleManifest, PluginShaderPermutationIdManifest,
    PluginShaderPermutationManifest,
};
pub use plugin_maturity::PluginMaturity;
pub use runtime_plugin::{
    CapabilityView, RuntimeExtensionCatalogReport, RuntimePlugin, RuntimePluginBridgeDependent,
    RuntimePluginBridgeDisableBlocker, RuntimePluginBridgeLifecycleBlock,
    RuntimePluginBridgeLifecycleError, RuntimePluginBridgeLifecycleEvent,
    RuntimePluginBridgeLifecycleOutcome, RuntimePluginBridgeLifecycleReport,
    RuntimePluginBridgeLifecycleState, RuntimePluginCatalog,
    RuntimePluginCatalogProjectPlanMetrics, RuntimePluginCatalogProjectionMetrics,
    RuntimePluginCatalogUpdate, RuntimePluginCatalogUpdateMetrics,
    RuntimePluginCatalogUpdateOutcome, RuntimePluginDescriptor, RuntimePluginDescriptorBuilder,
    RuntimePluginFeature, RuntimePluginFeatureBlock, RuntimePluginFeatureDependencyReport,
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};
pub use runtime_profile::{
    RuntimePluginAvailabilityCategory, RuntimePluginAvailabilityEntry,
    RuntimePluginAvailabilityGeneration, RuntimePluginAvailabilityReport,
    RuntimePluginAvailabilityRow, RuntimePluginAvailabilitySummary, RuntimeProfileDescriptor,
    RuntimeProfileFeaturePreset, RuntimeProfilePluginSelection, RUNTIME_PROFILE_FEATURE_PRESETS,
};
pub use ui_component_descriptor::UiComponentDescriptor;

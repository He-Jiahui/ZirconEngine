mod bridge;
mod capability_status;
mod component_type_descriptor;
mod core_profiles;
mod export_build_plan;
mod export_profile;
mod extension_registry;
mod extension_registry_error;
pub mod native;
mod native_plugin_loader;
mod package_manifest;
mod plugin_maturity;
mod project_plugin_manifest;
mod runtime_plugin;
mod runtime_profile;
mod scene_hook;
mod ui_component_descriptor;

pub use bridge::{
    BridgeDiagnosticsMatrix, BridgeDiagnosticsSnapshot, BridgeEntry, BridgeGuard,
    BridgeInterfaceSnapshot, BridgeInterfaceStatus, BridgeOwnerTransitionMode,
    BridgeOwnerTransitionReport, BridgeTableDiagnosticsSummary, FrozenBridgeTable, InterfaceSlot,
    StrongBridge, WeakBridge,
};
pub use capability_status::{CapabilityStatus, CapabilityStatusManifest};
pub use component_type_descriptor::{ComponentPropertyDescriptor, ComponentTypeDescriptor};
pub use core_profiles::{EditorCoreProfile, RuntimeCoreProfile};
pub use export_build_plan::{
    ExportBuildPlan, ExportGeneratedFile, ExportMaterializeReport, ExportPipelineStage,
    ExportValidateGeneratedFileSummary, ExportValidatePlanSummary, ExportValidateProfileSummary,
    ExportValidateReport, LibraryEmbedCompileHostPlan, LibraryEmbedCompileHostTarget,
    LibraryEmbedLinkedRuntimeCrate, NativeDynamicPackageAbiV3Contract,
    NativeDynamicPackageExportPlan,
};
pub use export_profile::{
    ExportBuildMode, ExportPackagingStrategy, ExportPlatformHostKind, ExportPlatformPluginStrategy,
    ExportPlatformPolicy, ExportPlatformResourceStrategy, ExportProfile, ExportTargetPlatform,
};
pub use extension_registry::{
    ExtensionKey, ExtensionOwnership, ExtensionSlot, FrozenExtensionTable, PluginModuleId,
    RuntimeExtensionRegistry,
};
pub use extension_registry_error::RuntimeExtensionRegistryError;
pub use package_manifest::{
    PluginDependencyManifest, PluginEventCatalogManifest, PluginEventManifest,
    PluginFeatureBundleManifest, PluginFeatureDependency, PluginInterfaceManifest,
    PluginInterfaceMethodManifest, PluginModuleKind, PluginModuleManifest, PluginOptionManifest,
    PluginPackageKind, PluginPackageManifest,
};
pub use plugin_maturity::PluginMaturity;
pub use project_plugin_manifest::{
    ProjectPluginFeatureSelection, ProjectPluginManifest, ProjectPluginSelection,
};
pub use runtime_plugin::{
    CapabilityView, PluginFinishContext, PluginRuntimeContext, RuntimeExtensionCatalogReport,
    RuntimePlugin, RuntimePluginBridgeDependent, RuntimePluginBridgeDisableBlocker,
    RuntimePluginBridgeLifecycleBlock, RuntimePluginBridgeLifecycleError,
    RuntimePluginBridgeLifecycleEvent, RuntimePluginBridgeLifecycleOutcome,
    RuntimePluginBridgeLifecycleReport, RuntimePluginBridgeLifecycleState, RuntimePluginCatalog,
    RuntimePluginDescriptor, RuntimePluginFeature, RuntimePluginFeatureBlock,
    RuntimePluginFeatureDependencyReport, RuntimePluginFeatureRegistrationReport,
    RuntimePluginRegistrationReport,
};
pub use runtime_profile::{
    RuntimePluginAvailabilityCategory, RuntimePluginAvailabilityEntry,
    RuntimePluginAvailabilityReport, RuntimeProfileDescriptor, RuntimeProfileId,
    RuntimeProfilePluginSelection,
};
pub use scene_hook::{
    SceneRuntimeHook, SceneRuntimeHookContext, SceneRuntimeHookDescriptor,
    SceneRuntimeHookRegistration,
};
pub use ui_component_descriptor::UiComponentDescriptor;

//! Common imports for plugin package declarations.

#[cfg(feature = "editor")]
pub use crate::editor::EditorPluginDeclaration;
pub use crate::{
    default_export_packaging, default_supported_platforms, importer_runtime_supported_platforms,
    importer_runtime_supported_targets, BridgeError, ImporterRuntimeManifestBuilder,
    PluginFeatureBundleBuilder, PluginInterface, PluginManifestBuilder, PluginModuleBuilder,
    RuntimePluginDeclaration, RuntimePluginModuleRegistration, RuntimePluginRegistrationBuilder,
    RuntimePluginRuntimeSceneSystemBuilder, TestRuntime, TestRuntimeBaseModule, TestRuntimeBuilder,
    TestRuntimeError, WeakBridge, NATIVE_ABI_VERSION_V3, NATIVE_DESCRIPTOR_SYMBOL_V3,
    SDK_API_VERSION,
};
pub use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};
pub use zircon_runtime::plugin::{
    ExportPackagingStrategy, ExportTargetPlatform, PluginMaturity, PluginModuleKind,
    PluginPackageManifest, RuntimePluginDescriptor,
};

//! Common imports for plugin package declarations.

#[cfg(feature = "editor")]
pub use crate::editor::EditorPluginDeclaration;
pub use crate::{
    default_export_packaging, default_supported_platforms, importer_runtime_supported_platforms,
    importer_runtime_supported_targets, BridgeError, BridgeImport, ImporterRuntimeManifestBuilder,
    PluginFeatureBundleBuilder, PluginInterface, PluginManifestBuilder, PluginModuleBuilder,
    RuntimePluginDeclaration, RuntimePluginModuleRegistration, RuntimePluginRegistrationBuilder,
    RuntimePluginRuntimeSceneSystemBuilder, TestRuntime, TestRuntimeBaseModule, TestRuntimeBuilder,
    TestRuntimeError, WeakBridge, NATIVE_ABI_VERSION_V3, NATIVE_DESCRIPTOR_SYMBOL_V3,
    SDK_API_VERSION,
};
pub use zircon_runtime::builtin::RuntimePluginId;
pub use zircon_runtime::core::framework::project::{ExportPackagingStrategy, ExportTargetPlatform};
pub use zircon_runtime::core::{InitLevel, ModuleDependencySpec};
pub use zircon_runtime::plugin::{
    PluginMaturity, PluginModuleKind, PluginPackageManifest, RuntimePluginDescriptor,
};

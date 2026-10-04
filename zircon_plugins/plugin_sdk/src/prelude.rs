//! Common imports for plugin package declarations.
//! 本模块仅在 runtime feature 下可用；Native-only 消费者直接导入 declaration 与 native 接口。

#[cfg(feature = "editor")]
pub use crate::editor::EditorPluginDeclaration;
pub use crate::{
    default_export_packaging, default_supported_platforms, importer_runtime_supported_platforms,
    importer_runtime_supported_targets, BridgeError, BridgeImport, ImporterRuntimeManifestBuilder,
    PluginFeatureBundleBuilder, PluginInterface, PluginManifestBuilder, PluginModuleBuilder,
    PluginPackageRole, RuntimePluginDeclaration, RuntimePluginModuleRegistration,
    RuntimePluginRegistrationBuilder, RuntimePluginRuntimeSceneSystemBuilder, TestRuntime,
    TestRuntimeBaseModule, TestRuntimeBuilder, TestRuntimeError, WeakBridge, NATIVE_ABI_VERSION_V3,
    NATIVE_DESCRIPTOR_SYMBOL_V3, SDK_API_VERSION,
};
pub use zircon_runtime::builtin::RuntimePluginId;
pub use zircon_runtime::core::framework::project::{ExportPackagingStrategy, ExportTargetPlatform};
pub use zircon_runtime::core::{InitLevel, ModuleDependencySpec};
pub use zircon_runtime::plugin::{
    PluginMaturity, PluginModuleKind, PluginPackageManifest, RuntimePluginDescriptor,
};

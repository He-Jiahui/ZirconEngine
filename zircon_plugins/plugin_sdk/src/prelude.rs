//! Common imports for plugin package declarations.

#[cfg(feature = "editor")]
pub use crate::editor::EditorPluginDeclaration;
pub use crate::{
    default_export_packaging, default_supported_platforms, PluginFeatureBundleBuilder,
    PluginManifestBuilder, PluginModuleBuilder, RuntimePluginDeclaration,
    RuntimePluginModuleRegistration, RuntimePluginRegistrationBuilder,
    RuntimePluginRuntimeSceneSystemBuilder, TestRuntime, TestRuntimeBaseModule, TestRuntimeBuilder,
    TestRuntimeError, SDK_API_VERSION,
};
pub use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};
pub use zircon_runtime::plugin::{
    ExportPackagingStrategy, ExportTargetPlatform, PluginMaturity, PluginModuleKind,
    PluginPackageManifest, RuntimePluginDescriptor,
};

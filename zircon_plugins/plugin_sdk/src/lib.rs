//! Shared builder APIs for Zircon plugin packages.
//!
//! Package declarations are runtime-independent SDK data. Runtime and native
//! features project that declaration into their respective descriptor and ABI
//! contracts without introducing another metadata authority.
//! feature gate 让声明与 Native 消费者和运行时、编辑器辅助 API 保持边界清晰。

#[cfg(feature = "declaration")]
#[path = "declaration.rs"]
mod declaration;
#[cfg(feature = "native")]
#[path = "dist.rs"]
pub mod dist;
#[cfg(feature = "editor")]
#[path = "editor.rs"]
pub mod editor;
#[cfg(feature = "editor_contribution")]
#[path = "editor_contribution.rs"]
pub mod editor_contribution;
#[cfg(feature = "runtime")]
#[path = "manifest/mod.rs"]
pub mod manifest;
#[cfg(feature = "native")]
#[path = "native.rs"]
pub mod native;
#[cfg(feature = "runtime")]
#[path = "prelude.rs"]
pub mod prelude;
#[cfg(feature = "runtime")]
#[path = "registration.rs"]
pub mod registration;
#[cfg(feature = "runtime")]
#[path = "runtime.rs"]
pub mod runtime;
#[cfg(feature = "runtime")]
#[path = "runtime_exports.rs"]
mod runtime_exports;
#[cfg(feature = "runtime")]
#[path = "test.rs"]
pub mod test;

#[cfg(feature = "declaration")]
pub use declaration::{
    NativePluginEntryDeclaration, PluginCapabilityRole, PluginDeclaration, PluginMaturityLevel,
    PluginPackageRole, PluginPackaging, PluginPlatform, PluginTarget,
};
#[cfg(feature = "editor")]
pub use editor::EditorPluginDeclaration;
#[cfg(feature = "editor_contribution")]
pub use editor_contribution::EditorContributionBuilder;
#[cfg(feature = "runtime")]
pub use manifest::{
    default_export_packaging, default_supported_platforms, importer_runtime_supported_platforms,
    importer_runtime_supported_targets, ImporterRuntimeManifestBuilder, PluginFeatureBundleBuilder,
    PluginManifestBuilder, PluginModuleBuilder, NATIVE_ABI_VERSION_V3, NATIVE_DESCRIPTOR_SYMBOL_V3,
    SDK_API_VERSION,
};
#[cfg(feature = "runtime")]
pub use registration::{
    RuntimePluginModuleRegistration, RuntimePluginRegistrationBuilder,
    RuntimePluginRuntimeSceneSystemBuilder,
};
#[cfg(feature = "runtime")]
pub use runtime::RuntimePluginDeclaration;
#[cfg(feature = "runtime")]
pub use test::{TestRuntime, TestRuntimeBaseModule, TestRuntimeBuilder, TestRuntimeError};
#[cfg(feature = "runtime")]
pub use zircon_runtime::core::framework::bridge::{BridgeError, PluginInterface};
#[cfg(feature = "runtime")]
pub use zircon_runtime::plugin::{BridgeImport, WeakBridge};

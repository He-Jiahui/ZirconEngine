//! Shared builder APIs for Zircon plugin packages.
//!
//! Package declarations are runtime-independent SDK data. Runtime and native
//! features project that declaration into their respective descriptor and ABI
//! contracts without introducing another metadata authority.

#[cfg(feature = "declaration")]
mod declaration;
#[cfg(feature = "native")]
pub mod dist;
#[cfg(feature = "editor")]
pub mod editor;
#[cfg(feature = "editor_contribution")]
pub mod editor_contribution;
#[cfg(feature = "runtime")]
pub mod manifest;
#[cfg(feature = "native")]
pub mod native;
#[cfg(feature = "runtime")]
pub mod prelude;
#[cfg(feature = "runtime")]
pub mod registration;
#[cfg(feature = "runtime")]
pub mod runtime;
#[cfg(feature = "runtime")]
mod runtime_exports;
#[cfg(feature = "runtime")]
pub mod test;

#[cfg(feature = "declaration")]
pub use declaration::{
    NativePluginEntryDeclaration, PluginCapabilityRole, PluginDeclaration, PluginMaturityLevel,
    PluginPackaging, PluginPlatform, PluginTarget,
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

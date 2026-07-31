use crate::runtime::RuntimePluginDeclaration;
use zircon_runtime::builtin::RuntimePluginId;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{ExportPackagingStrategy, ExportTargetPlatform};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::plugin::{PluginMaturity, RuntimePluginDescriptor};

/// Metadata that belongs to a plugin package rather than its runtime behavior.
///
/// The declaration intentionally contains no registration callbacks or importer
/// logic. Those remain in the plugin crate, while this value supplies the
/// descriptor fields that must agree with generated package metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginDeclaration {
    id: &'static str,
    display_name: &'static str,
    category: &'static str,
    module_name: &'static str,
    module_description: &'static str,
    target_modes: &'static [RuntimeTargetMode],
    supported_platforms: &'static [ExportTargetPlatform],
    capabilities: &'static [&'static str],
    maturity: PluginMaturity,
    default_packaging: &'static [ExportPackagingStrategy],
}

impl PluginDeclaration {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        id: &'static str,
        display_name: &'static str,
        category: &'static str,
        module_name: &'static str,
        module_description: &'static str,
        target_modes: &'static [RuntimeTargetMode],
        supported_platforms: &'static [ExportTargetPlatform],
        capabilities: &'static [&'static str],
        maturity: PluginMaturity,
        default_packaging: &'static [ExportPackagingStrategy],
    ) -> Self {
        Self {
            id,
            display_name,
            category,
            module_name,
            module_description,
            target_modes,
            supported_platforms,
            capabilities,
            maturity,
            default_packaging,
        }
    }

    pub const fn id(self) -> &'static str {
        self.id
    }

    pub const fn display_name(self) -> &'static str {
        self.display_name
    }

    pub const fn category(self) -> &'static str {
        self.category
    }

    pub const fn module_name(self) -> &'static str {
        self.module_name
    }

    pub const fn target_modes(self) -> &'static [RuntimeTargetMode] {
        self.target_modes
    }

    pub const fn supported_platforms(self) -> &'static [ExportTargetPlatform] {
        self.supported_platforms
    }

    pub const fn capabilities(self) -> &'static [&'static str] {
        self.capabilities
    }

    pub const fn maturity(self) -> PluginMaturity {
        self.maturity
    }

    pub const fn default_packaging(self) -> &'static [ExportPackagingStrategy] {
        self.default_packaging
    }

    pub fn module_descriptor(self) -> ModuleDescriptor {
        ModuleDescriptor::new(self.module_name, self.module_description)
    }

    pub fn runtime_declaration(
        self,
        runtime_id: RuntimePluginId,
        crate_name: impl Into<String>,
    ) -> RuntimePluginDeclaration {
        let declaration =
            RuntimePluginDeclaration::new(self.id, self.display_name, runtime_id, crate_name)
                .with_category(self.category)
                .with_target_modes(self.target_modes.iter().copied())
                .with_maturity(self.maturity)
                .with_default_packaging(self.default_packaging.iter().copied());

        self.capabilities
            .iter()
            .fold(declaration, |declaration, capability| {
                declaration.with_capability(*capability)
            })
    }

    pub fn runtime_descriptor(
        self,
        runtime_id: RuntimePluginId,
        crate_name: impl Into<String>,
    ) -> RuntimePluginDescriptor {
        self.runtime_declaration(runtime_id, crate_name)
            .with_module_descriptor(self.module_descriptor())
            .into_descriptor()
    }
}

/// Declare package metadata once and project it into the runtime descriptor.
///
/// The macro is deliberately data-only: registration and callback behavior stay
/// in the plugin crate, where their ownership remains visible to maintainers.
#[macro_export]
macro_rules! declare_plugin {
    (
        $(#[$metadata:meta])*
        $visibility:vis $declaration:ident {
            id: $id_constant:ident = $id:literal,
            display_name: $display_name:literal,
            category: $category:ident,
            module: $module_constant:ident = $module_name:literal,
            runtime_crate: $runtime_crate_constant:ident = $runtime_crate_name:literal,
            module_description: $module_description:literal,
            targets: [$($target:ident),+ $(,)?],
            platforms: [$($platform:ident),+ $(,)?],
            capabilities: [$($capability_constant:ident = $capability:literal),+ $(,)?],
            maturity: $maturity:ident,
            packaging: [$($packaging:ident),+ $(,)?] $(,)?
        }
    ) => {
        $(#[$metadata])*
        $visibility const $id_constant: &str = $id;
        $visibility const $module_constant: &str = $module_name;
        $visibility const $runtime_crate_constant: &str = $runtime_crate_name;
        $(
            $visibility const $capability_constant: &str = $capability;
        )+
        $visibility const $declaration: $crate::PluginDeclaration =
            $crate::PluginDeclaration::new(
                $id_constant,
                $display_name,
                stringify!($category),
                $module_constant,
                $module_description,
                &[$($crate::declare_plugin!(@target $target)),+],
                &[$($crate::declare_plugin!(@platform $platform)),+],
                &[$($capability_constant),+],
                $crate::declare_plugin!(@maturity $maturity),
                &[$($crate::declare_plugin!(@packaging $packaging)),+],
            );
    };
    (@target client_runtime) => {
        zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime
    };
    (@target server_runtime) => {
        zircon_runtime::core::framework::platform::RuntimeTargetMode::ServerRuntime
    };
    (@target editor_host) => {
        zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost
    };
    (@platform windows) => {
        zircon_runtime::core::framework::project::ExportTargetPlatform::Windows
    };
    (@platform linux) => {
        zircon_runtime::core::framework::project::ExportTargetPlatform::Linux
    };
    (@platform macos) => {
        zircon_runtime::core::framework::project::ExportTargetPlatform::Macos
    };
    (@platform android) => {
        zircon_runtime::core::framework::project::ExportTargetPlatform::Android
    };
    (@platform ios) => {
        zircon_runtime::core::framework::project::ExportTargetPlatform::Ios
    };
    (@platform web_gpu) => {
        zircon_runtime::core::framework::project::ExportTargetPlatform::WebGpu
    };
    (@platform wasm) => {
        zircon_runtime::core::framework::project::ExportTargetPlatform::Wasm
    };
    (@platform headless) => {
        zircon_runtime::core::framework::project::ExportTargetPlatform::Headless
    };
    (@maturity core) => {
        zircon_runtime::plugin::PluginMaturity::Core
    };
    (@maturity stable) => {
        zircon_runtime::plugin::PluginMaturity::Stable
    };
    (@maturity beta) => {
        zircon_runtime::plugin::PluginMaturity::Beta
    };
    (@maturity experimental) => {
        zircon_runtime::plugin::PluginMaturity::Experimental
    };
    (@maturity externalized) => {
        zircon_runtime::plugin::PluginMaturity::Externalized
    };
    (@maturity stub) => {
        zircon_runtime::plugin::PluginMaturity::Stub
    };
    (@maturity deprecated) => {
        zircon_runtime::plugin::PluginMaturity::Deprecated
    };
    (@packaging source_template) => {
        zircon_runtime::core::framework::project::ExportPackagingStrategy::SourceTemplate
    };
    (@packaging library_embed) => {
        zircon_runtime::core::framework::project::ExportPackagingStrategy::LibraryEmbed
    };
    (@packaging native_dynamic) => {
        zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::declare_plugin! {
        TEST_PLUGIN_DECLARATION {
            id: TEST_PLUGIN_ID = "sdk_test_plugin",
            display_name: "SDK Test Plugin",
            category: runtime,
            module: TEST_MODULE_NAME = "sdk_test_plugin.runtime",
            runtime_crate: TEST_RUNTIME_CRATE_NAME = "zircon_plugin_sdk_test",
            module_description: "Runtime metadata declaration fixture",
            targets: [client_runtime, editor_host],
            platforms: [windows, linux, macos],
            capabilities: [TEST_CAPABILITY = "runtime.plugin.sdk_test_plugin"],
            maturity: beta,
            packaging: [source_template, native_dynamic],
        }
    }

    #[test]
    fn declaration_projects_all_standard_descriptor_metadata() {
        let descriptor = TEST_PLUGIN_DECLARATION
            .runtime_descriptor(RuntimePluginId::GltfImporter, "zircon_plugin_sdk_test");

        assert_eq!(descriptor.package_id(), TEST_PLUGIN_ID);
        assert_eq!(TEST_RUNTIME_CRATE_NAME, "zircon_plugin_sdk_test");
        assert_eq!(descriptor.category(), "runtime");
        assert_eq!(descriptor.maturity(), PluginMaturity::Beta);
        assert_eq!(
            descriptor.target_modes(),
            TEST_PLUGIN_DECLARATION.target_modes()
        );
        assert_eq!(descriptor.capabilities(), [TEST_CAPABILITY.to_string()]);
        assert_eq!(descriptor.module_descriptor().name, TEST_MODULE_NAME);
        assert_eq!(
            descriptor.package_manifest().default_packaging.as_slice(),
            TEST_PLUGIN_DECLARATION.default_packaging()
        );
        assert_eq!(
            TEST_PLUGIN_DECLARATION.supported_platforms(),
            &[
                ExportTargetPlatform::Windows,
                ExportTargetPlatform::Linux,
                ExportTargetPlatform::Macos,
            ]
        );
    }
}

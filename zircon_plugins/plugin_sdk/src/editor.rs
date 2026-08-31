//! Editor-side plugin authoring helpers.

pub use zircon_editor;
pub use zircon_runtime;

use zircon_editor::{
    core::runtime_event_consumer::{
        EditorRuntimeEventConsumerRegistration, EditorRuntimeEventConsumerRegistry,
    },
    EditorPlugin, EditorPluginDescriptor, EditorPluginRegistrationReport,
};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::plugin::{PluginMaturity, PluginPackageManifest};

use crate::{PluginManifestBuilder, RuntimePluginDeclaration};

#[derive(Clone, Debug)]
pub struct EditorPluginDeclaration {
    descriptor: EditorPluginDescriptor,
    base_manifest: PluginPackageManifest,
    mirrored_runtime_package_id: Option<String>,
    runtime_event_consumers: EditorRuntimeEventConsumerRegistry,
    diagnostics: Vec<String>,
}

impl EditorPluginDeclaration {
    pub fn new(
        package_id: impl Into<String>,
        display_name: impl Into<String>,
        crate_name: impl Into<String>,
    ) -> Self {
        let package_id = package_id.into();
        let display_name = display_name.into();
        let crate_name = crate_name.into();
        Self {
            descriptor: EditorPluginDescriptor::new(
                package_id.clone(),
                display_name.clone(),
                crate_name,
            ),
            base_manifest: PluginManifestBuilder::new(package_id, display_name)
                .with_supported_targets([RuntimeTargetMode::EditorHost])
                .build(),
            mirrored_runtime_package_id: None,
            runtime_event_consumers: EditorRuntimeEventConsumerRegistry::default(),
            diagnostics: Vec::new(),
        }
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        let category = category.into();
        self.descriptor = self.descriptor.with_category(category.clone());
        self.base_manifest = self.base_manifest.with_category(category);
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.base_manifest.description = description.into();
        self
    }

    pub fn with_maturity(mut self, maturity: PluginMaturity) -> Self {
        self.base_manifest = self.base_manifest.with_maturity(maturity);
        self
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        let capability = capability.into();
        self.descriptor = self.descriptor.with_capability(capability.clone());
        self.base_manifest = self.base_manifest.with_capability(capability);
        self
    }

    pub fn with_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for capability in capabilities {
            self = self.with_capability(capability);
        }
        self
    }

    pub fn with_runtime_event_consumer_registration(
        mut self,
        registration: EditorRuntimeEventConsumerRegistration,
    ) -> Self {
        let manifest = registration.manifest().clone();
        match self.runtime_event_consumers.register(registration) {
            Ok(()) => {
                self.descriptor = self.descriptor.with_event_consumer(manifest);
            }
            Err(error) => self.diagnostics.push(error.to_string()),
        }
        self
    }

    pub fn mirrors_runtime(mut self, runtime: &RuntimePluginDeclaration) -> Self {
        self = self.mirrors_runtime_manifest(runtime.package_manifest());
        self
    }

    pub fn mirrors_runtime_manifest(mut self, runtime_manifest: PluginPackageManifest) -> Self {
        let editor_capabilities = self.descriptor.capabilities.clone();
        let asset_roots = std::mem::take(&mut self.base_manifest.asset_roots);
        let content_roots = std::mem::take(&mut self.base_manifest.content_roots);
        let mut base_manifest = runtime_manifest;
        self.descriptor.package_id = base_manifest.id.clone();
        self.mirrored_runtime_package_id = Some(base_manifest.id.clone());
        for capability in editor_capabilities {
            push_unique(&mut base_manifest.capabilities, capability);
        }
        merge_unique(&mut base_manifest.asset_roots, asset_roots);
        merge_unique(&mut base_manifest.content_roots, content_roots);
        self.base_manifest = base_manifest;
        self
    }

    pub fn with_asset_root(mut self, asset_root: impl Into<String>) -> Self {
        self.base_manifest = self.base_manifest.with_asset_root(asset_root);
        self
    }

    pub fn with_content_root(mut self, content_root: impl Into<String>) -> Self {
        self.base_manifest = self.base_manifest.with_content_root(content_root);
        self
    }

    pub fn descriptor(&self) -> &EditorPluginDescriptor {
        &self.descriptor
    }

    pub fn base_manifest(&self) -> PluginPackageManifest {
        self.base_manifest.clone()
    }

    pub fn package_manifest(&self) -> PluginPackageManifest {
        self.descriptor.attach_to_package(self.base_manifest())
    }

    pub fn capabilities(&self) -> &[String] {
        &self.descriptor.capabilities
    }

    pub fn mirrored_runtime_package_id(&self) -> Option<&str> {
        self.mirrored_runtime_package_id.as_deref()
    }

    pub fn runtime_event_consumers(&self) -> EditorRuntimeEventConsumerRegistry {
        self.runtime_event_consumers.clone()
    }

    pub fn registration_report(&self, plugin: &dyn EditorPlugin) -> EditorPluginRegistrationReport {
        let mut report = EditorPluginRegistrationReport::from_plugin(plugin, self.base_manifest());
        report.diagnostics.extend(self.diagnostics.iter().cloned());
        report
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn merge_unique(values: &mut Vec<String>, incoming: Vec<String>) {
    if values.is_empty() {
        *values = incoming;
        return;
    }
    for value in incoming {
        push_unique(values, value);
    }
}

#[macro_export]
macro_rules! authoring_plugin {
    (
        $(#[$meta:meta])*
        $vis:vis struct $plugin_ty:ident {
            package_id: $package_id:expr,
            display_name: $display_name:expr,
            crate_name: $crate_name:expr,
            category: $category:expr,
            description: $description:expr,
            maturity: $maturity:expr,
            $(mirrors_runtime: $runtime_declaration:expr,)?
            $(mirrors_runtime_manifest: $runtime_manifest:expr,)?
            capabilities: $capabilities:expr,
            $(runtime_event_consumers: $runtime_event_consumers:expr,)?
            $(asset_root: $asset_root:expr,)?
            $(content_root: $content_root:expr,)?
            register_extensions: $register_extensions:path $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug)]
        $vis struct $plugin_ty {
            declaration: $crate::editor::EditorPluginDeclaration,
        }

        impl $plugin_ty {
            pub fn new() -> Self {
                Self::default()
            }

            pub fn declaration(&self) -> &$crate::editor::EditorPluginDeclaration {
                &self.declaration
            }

            pub fn package_manifest(
                &self,
            ) -> $crate::editor::zircon_runtime::plugin::PluginPackageManifest {
                self.declaration.package_manifest()
            }

            pub fn editor_capabilities(&self) -> ::std::vec::Vec<::std::string::String> {
                self.declaration.capabilities().to_vec()
            }

            pub fn registration_report(
                &self,
            ) -> $crate::editor::zircon_editor::EditorPluginRegistrationReport {
                self.declaration.registration_report(self)
            }
        }

        impl ::std::default::Default for $plugin_ty {
            fn default() -> Self {
                let declaration = $crate::editor::EditorPluginDeclaration::new(
                    $package_id,
                    $display_name,
                    $crate_name,
                )
                .with_category($category)
                .with_description($description)
                .with_maturity($maturity);
                $(
                    let declaration = declaration.mirrors_runtime(&$runtime_declaration);
                )?
                $(
                    let declaration = declaration.mirrors_runtime_manifest($runtime_manifest);
                )?
                let declaration = declaration.with_capabilities(($capabilities).iter().copied());
                $(
                    let declaration = ($runtime_event_consumers).into_iter().fold(
                        declaration,
                        |declaration, registration| {
                            declaration.with_runtime_event_consumer_registration(registration)
                        },
                    );
                )?
                $(
                    let declaration = declaration.with_asset_root($asset_root);
                )?
                $(
                    let declaration = declaration.with_content_root($content_root);
                )?
                Self { declaration }
            }
        }

        impl $crate::editor::zircon_editor::EditorPlugin for $plugin_ty {
            fn descriptor(&self) -> &$crate::editor::zircon_editor::EditorPluginDescriptor {
                self.declaration.descriptor()
            }

            fn register_editor_extensions(
                &self,
                registry: &mut $crate::editor::zircon_editor::core::editor_extension::EditorExtensionRegistry,
            ) -> ::std::result::Result<
                (),
                $crate::editor::zircon_editor::core::editor_extension::EditorExtensionRegistryError,
            > {
                $register_extensions(registry)
            }

            fn runtime_event_consumers(
                &self,
            ) -> $crate::editor::zircon_editor::core::runtime_event_consumer::EditorRuntimeEventConsumerRegistry {
                self.declaration.runtime_event_consumers()
            }
        }
    };
}

pub use crate::authoring_plugin;

#[cfg(test)]
mod tests {
    use zircon_editor::core::editor_extension::{
        EditorExtensionRegistry, EditorExtensionRegistryError,
    };
    use zircon_runtime::builtin::RuntimePluginId;
    use zircon_runtime::plugin::PluginModuleKind;

    use super::*;

    const TEST_CAPABILITY: &str = "editor.extension.sdk_test";
    const TEST_CAPABILITIES: &[&str] = &[TEST_CAPABILITY];

    fn register_test_extensions(
        registry: &mut EditorExtensionRegistry,
    ) -> Result<(), EditorExtensionRegistryError> {
        registry.register_view(zircon_editor::core::editor_extension::ViewDescriptor::new(
            "sdk_test.window",
            "SDK Test",
            "SDK",
        ))
    }

    crate::editor::authoring_plugin! {
        pub struct MacroEditorPlugin {
            package_id: "sdk_test",
            display_name: "SDK Test",
            crate_name: "zircon_plugin_sdk_test_editor",
            category: "sdk",
            description: "SDK test editor plugin.",
            maturity: PluginMaturity::Experimental,
            capabilities: TEST_CAPABILITIES,
            asset_root: "assets",
            content_root: "examples",
            register_extensions: register_test_extensions,
        }
    }

    fn mirrored_runtime_declaration() -> RuntimePluginDeclaration {
        RuntimePluginDeclaration::new(
            "sdk_mirror",
            "SDK Mirror",
            RuntimePluginId::Animation,
            "zircon_plugin_sdk_mirror_runtime",
        )
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capability("runtime.plugin.sdk_mirror")
    }

    crate::editor::authoring_plugin! {
        pub struct MirroredMacroEditorPlugin {
            package_id: "sdk_mirror",
            display_name: "SDK Mirror Editor",
            crate_name: "zircon_plugin_sdk_mirror_editor",
            category: "sdk",
            description: "SDK mirrored editor plugin.",
            maturity: PluginMaturity::Experimental,
            mirrors_runtime: mirrored_runtime_declaration(),
            capabilities: TEST_CAPABILITIES,
            asset_root: "editor_assets",
            content_root: "editor_examples",
            register_extensions: register_test_extensions,
        }
    }

    #[test]
    fn authoring_plugin_macro_generates_descriptor_manifest_and_registration() {
        let plugin = MacroEditorPlugin::new();
        let manifest = plugin.package_manifest();
        let report = plugin.registration_report();

        assert_eq!(plugin.descriptor().package_id, "sdk_test");
        assert_eq!(manifest.category, "sdk");
        assert_eq!(manifest.capabilities, ["editor.extension.sdk_test"]);
        assert_eq!(manifest.asset_roots, ["assets"]);
        assert_eq!(manifest.content_roots, ["examples"]);
        assert!(manifest.modules.iter().any(|module| {
            module.kind == PluginModuleKind::Editor
                && module.crate_name == "zircon_plugin_sdk_test_editor"
        }));
        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .views()
            .iter()
            .any(|view| view.id() == "sdk_test.window"));
    }

    #[test]
    fn editor_declaration_mirrors_runtime_manifest_and_keeps_editor_capabilities() {
        let plugin = MirroredMacroEditorPlugin::new();
        let declaration = plugin.declaration();
        let manifest = plugin.package_manifest();

        assert_eq!(
            declaration.mirrored_runtime_package_id(),
            Some("sdk_mirror")
        );
        assert_eq!(manifest.id, "sdk_mirror");
        assert!(manifest
            .capabilities
            .contains(&"runtime.plugin.sdk_mirror".to_string()));
        assert!(manifest
            .capabilities
            .contains(&"editor.extension.sdk_test".to_string()));
        assert!(manifest.asset_roots.contains(&"editor_assets".to_string()));
        assert!(manifest
            .content_roots
            .contains(&"editor_examples".to_string()));

        let runtime_module = manifest
            .modules
            .iter()
            .find(|module| module.kind == PluginModuleKind::Runtime)
            .expect("mirrored package keeps runtime module");
        assert_eq!(
            runtime_module.capabilities,
            ["runtime.plugin.sdk_mirror".to_string()]
        );
        let editor_module = manifest
            .modules
            .iter()
            .find(|module| module.kind == PluginModuleKind::Editor)
            .expect("mirrored package adds editor module");
        assert_eq!(
            editor_module.capabilities,
            ["editor.extension.sdk_test".to_string()]
        );
    }

    #[test]
    fn mirrored_manifest_moves_editor_root_buffers() {
        let declaration = EditorPluginDeclaration::new(
            "editor.mirror",
            "Editor Mirror",
            "zircon_plugin_editor_mirror",
        )
        .with_asset_root("editor_assets")
        .with_content_root("editor_content");
        let asset_root_buffer = declaration.base_manifest.asset_roots.as_ptr();
        let content_root_buffer = declaration.base_manifest.content_roots.as_ptr();

        let mirrored = declaration
            .mirrors_runtime_manifest(PluginPackageManifest::new("runtime.mirror", "Runtime"));

        assert_eq!(
            mirrored.base_manifest.asset_roots.as_ptr(),
            asset_root_buffer
        );
        assert_eq!(
            mirrored.base_manifest.content_roots.as_ptr(),
            content_root_buffer
        );
        assert_eq!(mirrored.base_manifest.asset_roots, ["editor_assets"]);
        assert_eq!(mirrored.base_manifest.content_roots, ["editor_content"]);
    }
}

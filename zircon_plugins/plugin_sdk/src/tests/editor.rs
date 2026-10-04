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

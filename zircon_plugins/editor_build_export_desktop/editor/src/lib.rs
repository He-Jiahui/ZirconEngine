use zircon_editor::core::editor_authoring_extension::AssetCreationTemplateDescriptor;
use zircon_editor::core::editor_extension::{
    AssetEditorDescriptor, ComponentDrawerDescriptor, EditorExtensionRegistry,
    EditorExtensionRegistryError, EditorMenuItemDescriptor, EditorUiTemplateDescriptor,
};
use zircon_editor::core::editor_operation::{
    EditorOperationDescriptor, EditorOperationPath, UndoableEditorOperation,
};
use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::PluginPackageManifest, RuntimeTargetMode,
};

pub const PLUGIN_ID: &str = "editor_build_export_desktop";
pub const CAPABILITY: &str = "editor.extension.build_export_desktop";
pub const DIAGNOSTICS_CAPABILITY: &str = "editor.extension.build_export_desktop.diagnostics";
pub const NATIVE_DYNAMIC_REPORT_CAPABILITY: &str =
    "editor.extension.build_export_desktop.native_dynamic_report";

pub const EXPORT_VIEW_ID: &str = "editor.build_export_desktop";
pub const EXPORT_DRAWER_ID: &str = "editor_build_export_desktop.drawer";
pub const EXPORT_TEMPLATE_ID: &str = "editor_build_export_desktop.panel";
pub const SOURCE_TEMPLATE_REPORT_ID: &str = "editor_build_export_desktop.source_template_report";
pub const LIBRARY_EMBED_REPORT_ID: &str = "editor_build_export_desktop.library_embed_report";
pub const NATIVE_DYNAMIC_REPORT_ID: &str = "editor_build_export_desktop.native_dynamic_report";
pub const EXPORT_PROFILE_COMPONENT: &str = "editor.build_export_desktop.ExportProfile";
pub const EXPORT_PROFILE_ASSET_KIND: &str = "DesktopExportProfile";

#[derive(Clone, Debug)]
pub struct EditorBuildExportDesktopPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl EditorBuildExportDesktopPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for EditorBuildExportDesktopPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut EditorExtensionRegistry,
    ) -> Result<(), EditorExtensionRegistryError> {
        register_authoring_extensions(
            registry,
            EditorAuthoringExtensions {
                drawer_id: EXPORT_DRAWER_ID,
                drawer_display_name: "Desktop Export Tools",
                template_id: EXPORT_TEMPLATE_ID,
                template_document: "pane.editor_build_export_desktop.body",
                surfaces: &[EditorAuthoringSurface::new(
                    EXPORT_VIEW_ID,
                    "Desktop Export",
                    "Build",
                    "Project/Export/Desktop",
                )],
            },
        )?;
        register_export_operations(registry)?;
        register_export_report_templates(registry)?;
        register_export_profile_authoring(registry)
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Desktop Build Export",
        "zircon_plugin_editor_build_export_desktop_editor",
    )
    .with_capability(CAPABILITY)
    .with_capability(DIAGNOSTICS_CAPABILITY)
    .with_capability(NATIVE_DYNAMIC_REPORT_CAPABILITY)
}

pub fn editor_plugin() -> EditorBuildExportDesktopPlugin {
    EditorBuildExportDesktopPlugin::new()
}

pub fn package_manifest() -> PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(&editor_plugin(), base_package_manifest())
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        base_package_manifest(),
    )
}

fn base_package_manifest() -> PluginPackageManifest {
    PluginPackageManifest::new(PLUGIN_ID, "Desktop Build Export")
        .with_sdk_api_version("0.1.0")
        .with_category("platform")
        .with_supported_targets([RuntimeTargetMode::EditorHost])
        .with_supported_platforms([
            ExportTargetPlatform::Windows,
            ExportTargetPlatform::Linux,
            ExportTargetPlatform::Macos,
        ])
        .with_capabilities([
            CAPABILITY,
            DIAGNOSTICS_CAPABILITY,
            NATIVE_DYNAMIC_REPORT_CAPABILITY,
        ])
        .with_asset_root("assets")
        .with_content_root("templates")
        .with_default_packaging([
            ExportPackagingStrategy::SourceTemplate,
            ExportPackagingStrategy::LibraryEmbed,
        ])
}

fn register_export_operations(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    for operation in export_operations()? {
        let menu_path = operation
            .menu_path()
            .expect("desktop export operations are menu-backed")
            .to_string();
        let path = operation.path().clone();
        registry.register_operation(operation)?;
        registry.register_menu_item(
            EditorMenuItemDescriptor::new(menu_path, path).with_required_capabilities([CAPABILITY]),
        )?;
    }
    Ok(())
}

fn export_operations() -> Result<Vec<EditorOperationDescriptor>, EditorExtensionRegistryError> {
    let generate = parse_operation("BuildExport.Desktop.GeneratePlan")?;
    let source_template = parse_operation("BuildExport.Desktop.SourceTemplate")?;
    let library_embed = parse_operation("BuildExport.Desktop.LibraryEmbed")?;
    let native_dynamic = parse_operation("BuildExport.Desktop.NativeDynamic")?;
    let diagnostics = parse_operation("BuildExport.Desktop.OpenDiagnostics")?;
    let create_profile = parse_operation("BuildExport.Desktop.CreateProfile")?;
    let open_profile = parse_operation("BuildExport.Desktop.OpenProfile")?;

    Ok(vec![
        EditorOperationDescriptor::new(generate, "Generate Desktop Export Plan")
            .with_menu_path("Project/Export/Desktop/Generate Plan")
            .with_required_capabilities([CAPABILITY]),
        EditorOperationDescriptor::new(source_template, "Export Source Template")
            .with_menu_path("Project/Export/Desktop/Source Template")
            .with_required_capabilities([CAPABILITY]),
        EditorOperationDescriptor::new(library_embed, "Export Library Embed")
            .with_menu_path("Project/Export/Desktop/Library Embed")
            .with_required_capabilities([CAPABILITY]),
        EditorOperationDescriptor::new(native_dynamic, "Export Native Dynamic")
            .with_menu_path("Project/Export/Desktop/Native Dynamic")
            .with_required_capabilities([CAPABILITY, NATIVE_DYNAMIC_REPORT_CAPABILITY]),
        EditorOperationDescriptor::new(diagnostics, "Open Export Diagnostics")
            .with_menu_path("Project/Export/Desktop/Diagnostics")
            .with_required_capabilities([CAPABILITY, DIAGNOSTICS_CAPABILITY]),
        EditorOperationDescriptor::new(create_profile, "Create Desktop Export Profile")
            .with_menu_path("Assets/Create/Desktop Export Profile")
            .with_undoable(UndoableEditorOperation::new(
                "Create Desktop Export Profile",
            ))
            .with_required_capabilities([CAPABILITY]),
        EditorOperationDescriptor::new(open_profile, "Open Desktop Export Profile")
            .with_menu_path("Assets/Open/Desktop Export Profile")
            .with_required_capabilities([CAPABILITY]),
    ])
}

fn register_export_report_templates(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    for (id, document) in [
        (
            SOURCE_TEMPLATE_REPORT_ID,
            "asset://editor_build_export_desktop/editor/source_template_report.ui.toml",
        ),
        (
            LIBRARY_EMBED_REPORT_ID,
            "asset://editor_build_export_desktop/editor/library_embed_report.ui.toml",
        ),
        (
            NATIVE_DYNAMIC_REPORT_ID,
            "asset://editor_build_export_desktop/editor/native_dynamic_report.ui.toml",
        ),
    ] {
        registry.register_ui_template(EditorUiTemplateDescriptor::new(id, document))?;
    }
    Ok(())
}

fn register_export_profile_authoring(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    let create_profile = parse_operation("BuildExport.Desktop.CreateProfile")?;
    let open_profile = parse_operation("BuildExport.Desktop.OpenProfile")?;

    registry.register_asset_creation_template(
        AssetCreationTemplateDescriptor::new(
            "editor_build_export_desktop.profile",
            "Desktop Export Profile",
            EXPORT_PROFILE_ASSET_KIND,
            create_profile,
        )
        .with_default_document(
            "asset://editor_build_export_desktop/templates/desktop_export_profile.toml",
        )
        .with_required_capabilities([CAPABILITY]),
    )?;
    registry.register_asset_editor(
        AssetEditorDescriptor::new(
            EXPORT_PROFILE_ASSET_KIND,
            EXPORT_VIEW_ID,
            "Desktop Export Profile",
            open_profile,
        )
        .with_required_capabilities([CAPABILITY]),
    )?;
    registry.register_component_drawer(
        ComponentDrawerDescriptor::new(
            EXPORT_PROFILE_COMPONENT,
            "asset://editor_build_export_desktop/editor/export_profile_drawer.ui.toml",
            "editor.build_export_desktop.ExportProfileController",
        )
        .with_binding("BuildExport.Desktop.GeneratePlan")
        .with_binding("BuildExport.Desktop.SourceTemplate")
        .with_binding("BuildExport.Desktop.LibraryEmbed")
        .with_binding("BuildExport.Desktop.NativeDynamic"),
    )
}

fn parse_operation(path: &str) -> Result<EditorOperationPath, EditorExtensionRegistryError> {
    EditorOperationPath::parse(path).map_err(EditorExtensionRegistryError::Operation)
}

#[cfg(test)]
mod tests {
    use zircon_runtime::plugin::PluginModuleKind;

    use super::*;

    #[test]
    fn desktop_export_plugin_contributes_panel_operations_and_reports() {
        let registration = plugin_registration();

        assert!(registration.is_success(), "{:?}", registration.diagnostics);
        assert_eq!(
            registration.capabilities,
            vec![
                CAPABILITY.to_string(),
                DIAGNOSTICS_CAPABILITY.to_string(),
                NATIVE_DYNAMIC_REPORT_CAPABILITY.to_string()
            ]
        );
        assert!(registration
            .extensions
            .views()
            .iter()
            .any(|view| view.id() == EXPORT_VIEW_ID));
        assert!(registration
            .extensions
            .ui_templates()
            .iter()
            .any(|template| template.id() == SOURCE_TEMPLATE_REPORT_ID));
        assert!(registration
            .extensions
            .ui_templates()
            .iter()
            .any(|template| template.id() == LIBRARY_EMBED_REPORT_ID));
        assert!(registration
            .extensions
            .ui_templates()
            .iter()
            .any(|template| template.id() == NATIVE_DYNAMIC_REPORT_ID));
        assert!(registration
            .extensions
            .operations()
            .descriptors()
            .any(|operation| operation.path().as_str() == "BuildExport.Desktop.NativeDynamic"));
        assert!(registration
            .extensions
            .menu_items()
            .iter()
            .any(|menu| menu.path() == "Project/Export/Desktop/Native Dynamic"));
        assert!(registration
            .extensions
            .asset_creation_templates()
            .iter()
            .any(|template| template.asset_kind() == EXPORT_PROFILE_ASSET_KIND));
        assert!(registration
            .extensions
            .component_drawers()
            .iter()
            .any(|drawer| drawer.component_type() == EXPORT_PROFILE_COMPONENT));
    }

    #[test]
    fn desktop_export_package_manifest_declares_editor_only_metadata() {
        let manifest = package_manifest();

        assert_eq!(manifest.category, "platform");
        assert_eq!(
            manifest.supported_targets,
            vec![RuntimeTargetMode::EditorHost]
        );
        assert!(manifest.capabilities.contains(&CAPABILITY.to_string()));
        assert!(manifest
            .modules
            .iter()
            .any(|module| module.kind == PluginModuleKind::Editor
                && module.crate_name == "zircon_plugin_editor_build_export_desktop_editor"));
        assert!(manifest
            .default_packaging
            .contains(&ExportPackagingStrategy::SourceTemplate));
    }
}

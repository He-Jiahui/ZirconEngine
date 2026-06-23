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
use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::PluginPackageManifest,
};

use crate::{
    CAPABILITY, DIAGNOSTICS_CAPABILITY, EDITOR_CAPABILITIES, EXPORT_DRAWER_ID,
    EXPORT_OPERATION_CREATE_PROFILE, EXPORT_OPERATION_GENERATE_PLAN,
    EXPORT_OPERATION_LIBRARY_EMBED, EXPORT_OPERATION_NATIVE_DYNAMIC,
    EXPORT_OPERATION_OPEN_DIAGNOSTICS, EXPORT_OPERATION_OPEN_PROFILE,
    EXPORT_OPERATION_SOURCE_TEMPLATE, EXPORT_PANEL_TEMPLATE_DOCUMENT, EXPORT_PROFILE_ASSET_KIND,
    EXPORT_PROFILE_COMPONENT, EXPORT_PROFILE_DRAWER_DOCUMENT, EXPORT_PROFILE_TEMPLATE_DOCUMENT,
    EXPORT_REPORT_TEMPLATE_DOCUMENTS, EXPORT_TEMPLATE_ID, EXPORT_VIEW_ID,
    NATIVE_DYNAMIC_REPORT_CAPABILITY, PLUGIN_ID,
};

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
                template_document: EXPORT_PANEL_TEMPLATE_DOCUMENT,
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
        .with_capabilities(EDITOR_CAPABILITIES.iter().copied())
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
    let generate = parse_operation(EXPORT_OPERATION_GENERATE_PLAN)?;
    let source_template = parse_operation(EXPORT_OPERATION_SOURCE_TEMPLATE)?;
    let library_embed = parse_operation(EXPORT_OPERATION_LIBRARY_EMBED)?;
    let native_dynamic = parse_operation(EXPORT_OPERATION_NATIVE_DYNAMIC)?;
    let diagnostics = parse_operation(EXPORT_OPERATION_OPEN_DIAGNOSTICS)?;
    let create_profile = parse_operation(EXPORT_OPERATION_CREATE_PROFILE)?;
    let open_profile = parse_operation(EXPORT_OPERATION_OPEN_PROFILE)?;

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
    for &(id, document) in EXPORT_REPORT_TEMPLATE_DOCUMENTS {
        registry.register_ui_template(EditorUiTemplateDescriptor::new(id, document))?;
    }
    Ok(())
}

fn register_export_profile_authoring(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    let create_profile = parse_operation(EXPORT_OPERATION_CREATE_PROFILE)?;
    let open_profile = parse_operation(EXPORT_OPERATION_OPEN_PROFILE)?;

    registry.register_asset_creation_template(
        AssetCreationTemplateDescriptor::new(
            "editor_build_export_desktop.profile",
            "Desktop Export Profile",
            EXPORT_PROFILE_ASSET_KIND,
            create_profile,
        )
        .with_default_document(EXPORT_PROFILE_TEMPLATE_DOCUMENT)
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
            EXPORT_PROFILE_DRAWER_DOCUMENT,
            "editor.build_export_desktop.ExportProfileController",
        )
        .with_binding(EXPORT_OPERATION_GENERATE_PLAN)
        .with_binding(EXPORT_OPERATION_SOURCE_TEMPLATE)
        .with_binding(EXPORT_OPERATION_LIBRARY_EMBED)
        .with_binding(EXPORT_OPERATION_NATIVE_DYNAMIC),
    )
}

fn parse_operation(path: &str) -> Result<EditorOperationPath, EditorExtensionRegistryError> {
    EditorOperationPath::parse(path).map_err(EditorExtensionRegistryError::Operation)
}

mod export_wizard;

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
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::PluginPackageManifest,
    RuntimeTargetMode,
};

pub const PLUGIN_ID: &str = "editor_build_export_desktop";
pub const CAPABILITY: &str = "editor.extension.build_export_desktop";
pub const DIAGNOSTICS_CAPABILITY: &str = "editor.extension.build_export_desktop.diagnostics";
pub const NATIVE_DYNAMIC_REPORT_CAPABILITY: &str =
    "editor.extension.build_export_desktop.native_dynamic_report";

pub const EXPORT_VIEW_ID: &str = "editor.build_export_desktop";
pub const EXPORT_DRAWER_ID: &str = "editor_build_export_desktop.drawer";
pub const EXPORT_TEMPLATE_ID: &str = zircon_editor::EXPORT_WIZARD_TEMPLATE_DOCUMENT_ID;
pub const SOURCE_TEMPLATE_REPORT_ID: &str = "editor_build_export_desktop.source_template_report";
pub const LIBRARY_EMBED_REPORT_ID: &str = "editor_build_export_desktop.library_embed_report";
pub const NATIVE_DYNAMIC_REPORT_ID: &str = "editor_build_export_desktop.native_dynamic_report";
pub const EXPORT_PANEL_TEMPLATE_DOCUMENT: &str =
    "asset://editor_build_export_desktop/editor/panel.v2.ui.toml";
pub const SOURCE_TEMPLATE_REPORT_DOCUMENT: &str =
    "asset://editor_build_export_desktop/editor/source_template_report.v2.ui.toml";
pub const LIBRARY_EMBED_REPORT_DOCUMENT: &str =
    "asset://editor_build_export_desktop/editor/library_embed_report.v2.ui.toml";
pub const NATIVE_DYNAMIC_REPORT_DOCUMENT: &str =
    "asset://editor_build_export_desktop/editor/native_dynamic_report.v2.ui.toml";
pub const EXPORT_PROFILE_TEMPLATE_DOCUMENT: &str =
    "asset://editor_build_export_desktop/templates/desktop_export_profile.toml";
pub const EXPORT_PROFILE_DRAWER_DOCUMENT: &str =
    "asset://editor_build_export_desktop/editor/export_profile_drawer.zui";
pub const EXPORT_OPERATION_GENERATE_PLAN: &str = "build_export.desktop.generate_plan";
pub const EXPORT_OPERATION_SOURCE_TEMPLATE: &str = "build_export.desktop.source_template";
pub const EXPORT_OPERATION_LIBRARY_EMBED: &str = "build_export.desktop.library_embed";
pub const EXPORT_OPERATION_NATIVE_DYNAMIC: &str = "build_export.desktop.native_dynamic";
pub const EXPORT_OPERATION_OPEN_DIAGNOSTICS: &str = "build_export.desktop.open_diagnostics";
pub const EXPORT_OPERATION_CREATE_PROFILE: &str = "build_export.desktop.create_profile";
pub const EXPORT_OPERATION_OPEN_PROFILE: &str = "build_export.desktop.open_profile";
pub const EXPORT_UI_TEMPLATE_DOCUMENTS: &[(&str, &str)] = &[
    (EXPORT_TEMPLATE_ID, EXPORT_PANEL_TEMPLATE_DOCUMENT),
    (SOURCE_TEMPLATE_REPORT_ID, SOURCE_TEMPLATE_REPORT_DOCUMENT),
    (LIBRARY_EMBED_REPORT_ID, LIBRARY_EMBED_REPORT_DOCUMENT),
    (NATIVE_DYNAMIC_REPORT_ID, NATIVE_DYNAMIC_REPORT_DOCUMENT),
];
pub const EXPORT_REPORT_TEMPLATE_DOCUMENTS: &[(&str, &str)] = &[
    (SOURCE_TEMPLATE_REPORT_ID, SOURCE_TEMPLATE_REPORT_DOCUMENT),
    (LIBRARY_EMBED_REPORT_ID, LIBRARY_EMBED_REPORT_DOCUMENT),
    (NATIVE_DYNAMIC_REPORT_ID, NATIVE_DYNAMIC_REPORT_DOCUMENT),
];
pub const EXPORT_PROFILE_COMPONENT: &str = "editor.build_export_desktop.ExportProfile";
pub const EXPORT_PROFILE_ASSET_KIND: &str = "DesktopExportProfile";

pub use export_wizard::{
    export_wizard_descriptor, stage_progress_kinds, ExportWizardAction, ExportWizardDescriptor,
    ExportWizardRegion, ExportWizardRegionDescriptor, ExportWizardReportViewDescriptor,
    ExportWizardStageDescriptor, BUILD_EXPORT_LAYOUT_REFERENCE, PIPELINE_REPORT_PATH,
};
pub use zircon_editor::{
    apply_export_wizard_panel_template_state, execute_export_wizard_pipeline,
    execute_export_wizard_stage, export_pipeline_stage_cli_id, export_pipeline_stage_report_name,
    export_wizard_panel_action_call, export_wizard_panel_action_for_control,
    export_wizard_panel_binding_entries, export_wizard_panel_bindings,
    export_wizard_panel_retained_projection, export_wizard_panel_template_state,
    export_wizard_pipeline_plan, project_export_wizard_panel,
    register_export_wizard_panel_bindings, register_export_wizard_panel_template,
    run_export_wizard_job, ExportWizardCancelSignal, ExportWizardCommandExecution,
    ExportWizardCommandRunner, ExportWizardControlState, ExportWizardJobController,
    ExportWizardJobEvent, ExportWizardJobEventKind, ExportWizardJobHandle, ExportWizardJobSnapshot,
    ExportWizardJobState, ExportWizardJobStatus, ExportWizardNeverCancel, ExportWizardPanelAction,
    ExportWizardPanelBinding, ExportWizardPanelControlBindingState, ExportWizardPanelEntrySeverity,
    ExportWizardPanelRequest, ExportWizardPanelSession, ExportWizardPanelSessionError,
    ExportWizardPanelSlotEntry, ExportWizardPanelSlotKind, ExportWizardPanelSlotState,
    ExportWizardPanelTemplateState, ExportWizardPanelUpdate, ExportWizardPanelViewModel,
    ExportWizardPipelineExecution, ExportWizardPipelineOptions, ExportWizardPipelinePlan,
    ExportWizardPipelineStageCommand, ExportWizardStageExecution, ExportWizardStageMissingInputs,
    ExportWizardStagePlannedArtifacts, ExportWizardStageViewRow, ProcessCommandRunner,
};
pub use zircon_editor::{
    export_pipeline_stages, parse_export_pipeline_stage, ExportStageProgressKind,
    ExportWizardProgressState, ExportWizardStageArtifactPath, ExportWizardStageProgressSnapshot,
    ExportWizardStreamEvent, DESKTOP_EXPORT_ARTIFACT_PATHS_SLOT, DESKTOP_EXPORT_CANCEL_BINDING_ID,
    DESKTOP_EXPORT_CANCEL_BUTTON, DESKTOP_EXPORT_GENERATE_PLAN_BINDING_ID,
    DESKTOP_EXPORT_GENERATE_PLAN_BUTTON, DESKTOP_EXPORT_MISSING_INPUTS_SLOT,
    DESKTOP_EXPORT_REPORT_BODY_SLOT, DESKTOP_EXPORT_STAGE_ROWS_SLOT,
    DESKTOP_EXPORT_START_BINDING_ID, DESKTOP_EXPORT_START_BUTTON,
    DESKTOP_EXPORT_TERMINAL_OUTPUT_SLOT, EXPORT_WIZARD_BINDING_SYMBOL,
    EXPORT_WIZARD_TEMPLATE_DOCUMENT_ID, EXPORT_WIZARD_VIEW_ID,
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    use zircon_runtime::plugin::PluginModuleKind;
    use zircon_runtime::ui::v2::{UiV2AssetLoader, UiZuiAssetLoader};
    use zircon_runtime_interface::ui::v2::UiV2AssetKind;

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
            .any(|operation| operation.path().as_str() == EXPORT_OPERATION_NATIVE_DYNAMIC));
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

    #[test]
    fn export_wizard_descriptor_covers_build_layout_stages_and_reports() {
        let wizard = export_wizard_descriptor();

        assert_eq!(wizard.template_id, EXPORT_TEMPLATE_ID);
        assert_eq!(
            wizard.layout_reference,
            "docs/ui-and-layout/ai-workbench-style/ai-build-export-layout.png"
        );
        assert_eq!(wizard.regions.len(), 3);
        assert!(wizard
            .regions
            .iter()
            .any(|region| region.region == ExportWizardRegion::ProfileTree));
        assert_eq!(
            wizard
                .stages
                .iter()
                .map(|stage| stage.stage)
                .collect::<Vec<_>>(),
            vec![
                zircon_runtime::plugin::ExportPipelineStage::Validate,
                zircon_runtime::plugin::ExportPipelineStage::CompileHost,
                zircon_runtime::plugin::ExportPipelineStage::SourceTemplate,
                zircon_runtime::plugin::ExportPipelineStage::CookAssets,
                zircon_runtime::plugin::ExportPipelineStage::Pack,
                zircon_runtime::plugin::ExportPipelineStage::PlatformBundle,
                zircon_runtime::plugin::ExportPipelineStage::Report,
            ]
        );
        assert_eq!(
            wizard
                .stage(zircon_runtime::plugin::ExportPipelineStage::Pack)
                .expect("pack stage descriptor")
                .report_path,
            "report.json"
        );
        assert_eq!(
            wizard
                .report_view("native_dynamic")
                .expect("native dynamic report")
                .template_id,
            NATIVE_DYNAMIC_REPORT_ID
        );
        assert_eq!(stage_progress_kinds().len(), 4);
    }

    #[test]
    fn desktop_export_private_template_assets_match_registered_documents() {
        let registration = plugin_registration();

        assert!(registration.is_success(), "{:?}", registration.diagnostics);
        let template_documents = registration
            .extensions
            .ui_templates()
            .into_iter()
            .map(|template| {
                (
                    template.id().to_string(),
                    template.ui_document().to_string(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        for (template_id, document) in EXPORT_UI_TEMPLATE_DOCUMENTS {
            assert_eq!(
                template_documents.get(*template_id).map(String::as_str),
                Some(*document),
                "registered template {template_id} should point at its private asset"
            );
            assert_view_template_asset(document, template_id);
        }

        let profile_template = registration
            .extensions
            .asset_creation_templates()
            .into_iter()
            .find(|template| template.asset_kind() == EXPORT_PROFILE_ASSET_KIND)
            .expect("desktop export profile template");
        assert_eq!(
            profile_template.default_document(),
            Some(EXPORT_PROFILE_TEMPLATE_DOCUMENT)
        );
        assert_profile_template_asset(EXPORT_PROFILE_TEMPLATE_DOCUMENT);

        let profile_drawer = registration
            .extensions
            .component_drawers()
            .into_iter()
            .find(|drawer| drawer.component_type() == EXPORT_PROFILE_COMPONENT)
            .expect("desktop export profile drawer");
        assert_eq!(profile_drawer.ui_document(), EXPORT_PROFILE_DRAWER_DOCUMENT);
        assert_component_template_asset(EXPORT_PROFILE_DRAWER_DOCUMENT, "ExportProfileDrawer");
    }

    fn assert_view_template_asset(document: &str, expected_id: &str) {
        let path = plugin_asset_path(document);
        let asset = UiV2AssetLoader::load_toml_file(&path)
            .unwrap_or_else(|error| panic!("{} should parse: {error}", path.display()));

        assert_eq!(asset.asset.kind, UiV2AssetKind::View);
        assert_eq!(asset.asset.id, expected_id);
        assert!(
            asset.root_node_id().is_some(),
            "view template {expected_id} should declare a root node"
        );
    }

    fn assert_component_template_asset(document: &str, component_name: &str) {
        let path = plugin_asset_path(document);
        let asset = UiZuiAssetLoader::load_zui_file(&path)
            .unwrap_or_else(|error| panic!("{} should parse: {error}", path.display()));

        assert_eq!(asset.asset.kind, UiV2AssetKind::Component);
        assert!(
            asset.components.contains_key(component_name),
            "component template {} should define {component_name}",
            path.display()
        );
    }

    fn assert_profile_template_asset(document: &str) {
        let path = plugin_asset_path(document);
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));

        assert!(source.contains("[export_profile]"));
        assert!(source.contains("platform = \"windows-x86_64\""));
        assert!(source.contains("path = \"library_embed\""));
        assert!(source.contains("asset_filter = \"shipping\""));
    }

    fn plugin_asset_path(document: &str) -> PathBuf {
        const PREFIX: &str = "asset://editor_build_export_desktop/";

        let relative = document
            .strip_prefix(PREFIX)
            .unwrap_or_else(|| panic!("document {document} should use {PREFIX}"));
        let plugin_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("editor crate should be inside plugin root");
        let mut path = plugin_root.to_path_buf();
        for segment in relative.split('/') {
            path.push(segment);
        }
        path
    }
}

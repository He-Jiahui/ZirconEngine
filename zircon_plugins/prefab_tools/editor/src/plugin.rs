use zircon_editor::core::asset::{
    AssetCreationTemplateDescriptor, AssetToolkitDescriptor, AssetTypeContribution, AssetTypeId,
};
use zircon_editor::core::commands::EditorCommandDescriptor;
use zircon_editor::core::editor_extension::{
    EditorExtensionRegistry, EditorMenuItemDescriptor,
};
use zircon_editor::core::extension::InspectorCustomizationDescriptor;
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_plugin_editor_support::{
    register_authoring_contribution_batch, register_authoring_extensions,
    EditorAuthoringContributionBatch, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_runtime_interface::resource::ResourceKind;

use crate::{
    CAPABILITY, PLUGIN_ID, PREFAB_AUTHORING_VIEW_ID, PREFAB_DRAWER_ID, PREFAB_TEMPLATE_ID,
};

#[derive(Clone, Debug)]
pub struct PrefabToolsEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl PrefabToolsEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for PrefabToolsEditorPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut EditorExtensionRegistry,
    ) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
        register_authoring_extensions(
            registry,
            EditorAuthoringExtensions {
                drawer_id: PREFAB_DRAWER_ID,
                drawer_display_name: "Prefab Tools",
                template_id: PREFAB_TEMPLATE_ID,
                template_document: "plugins://prefab_tools/editor/authoring.zui",
                surfaces: &[EditorAuthoringSurface::new(
                    PREFAB_AUTHORING_VIEW_ID,
                    "Prefabs",
                    "World",
                    "Plugins/Prefab Tools",
                )],
            },
        )?;
        register_authoring_contribution_batch(registry, prefab_authoring_batch())
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Prefab Tools",
        "zircon_plugin_prefab_tools_editor",
    )
    .with_capability(CAPABILITY)
}

pub fn editor_plugin() -> PrefabToolsEditorPlugin {
    PrefabToolsEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(
        &editor_plugin(),
        zircon_plugin_prefab_tools_runtime::package_manifest(),
    )
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        zircon_plugin_prefab_tools_runtime::package_manifest(),
    )
}

fn prefab_authoring_batch() -> EditorAuthoringContributionBatch {
    let create = operation("prefab_tools.authoring.create_from_selection");
    let open = operation("prefab_tools.authoring.open");
    let apply = operation("prefab_tools.authoring.apply_overrides");
    let revert = operation("prefab_tools.authoring.revert_overrides");
    let break_instance = operation("prefab_tools.authoring.break_instance");
    EditorAuthoringContributionBatch {
        commands: vec![
            EditorCommandDescriptor::operation(create.clone(), "Create Prefab From Selection")
                .with_menu_path("Plugins/Prefab Tools/Create From Selection")
                .with_payload_schema_id("prefab_tools.create_from_selection.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::operation(open.clone(), "Open Prefab")
                .with_menu_path("Plugins/Prefab Tools/Open Prefab Asset")
                .with_payload_schema_id("prefab_tools.open_asset.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::operation(apply.clone(), "Apply Prefab Overrides")
                .with_menu_path("Plugins/Prefab Tools/Apply Overrides")
                .with_payload_schema_id("prefab_tools.apply_overrides.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::operation(revert.clone(), "Revert Prefab Overrides")
                .with_menu_path("Plugins/Prefab Tools/Revert Overrides")
                .with_payload_schema_id("prefab_tools.revert_overrides.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::operation(break_instance.clone(), "Break Prefab Instance")
                .with_menu_path("Plugins/Prefab Tools/Break Instance")
                .with_payload_schema_id("prefab_tools.break_instance.v1")
                .with_required_capabilities([CAPABILITY]),
        ],
        menu_items: vec![
            menu_item("Plugins/Prefab Tools/Create From Selection", &create),
            menu_item("Plugins/Prefab Tools/Open Prefab Asset", &open),
            menu_item("Plugins/Prefab Tools/Apply Overrides", &apply),
            menu_item("Plugins/Prefab Tools/Revert Overrides", &revert),
            menu_item("Plugins/Prefab Tools/Break Instance", &break_instance),
        ],
        asset_type_contributions: vec![AssetTypeContribution::augment(
            AssetTypeId::from_resource_kind(ResourceKind::Prefab),
        )
        .with_toolkit(
            AssetToolkitDescriptor::new(PREFAB_AUTHORING_VIEW_ID, open)
                .with_required_capabilities([CAPABILITY]),
        )
        .with_creation_template(
            AssetCreationTemplateDescriptor::new("prefab_tools.template.prefab", "Prefab", create)
                .with_default_document("plugins://prefab_tools/templates/default_prefab.toml")
                .with_required_capabilities([CAPABILITY]),
        )],
        inspector_customizations: vec![InspectorCustomizationDescriptor::new(
            zircon_plugin_prefab_tools_runtime::PREFAB_INSTANCE_COMPONENT_TYPE,
            "plugins://prefab_tools/editor/prefab_instance.zui",
            "prefab_tools.editor.component",
        )],
        ..Default::default()
    }
}

fn operation(path: &str) -> EditorOperationPath {
    EditorOperationPath::parse(path).expect("valid prefab operation path")
}

fn menu_item(path: &str, operation: &EditorOperationPath) -> EditorMenuItemDescriptor {
    EditorMenuItemDescriptor::new(path, operation.clone()).with_required_capabilities([CAPABILITY])
}

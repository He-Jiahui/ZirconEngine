use zircon_editor::core::asset::{
    AssetCreationTemplateDescriptor, AssetToolkitDescriptor, AssetTypeContribution, AssetTypeId,
};
use zircon_editor::core::commands::EditorCommandDescriptor;
use zircon_editor::core::editor_authoring_extension::ViewportToolModeDescriptor;
use zircon_editor::core::editor_extension::{
    AssetImporterDescriptor, ComponentDrawerDescriptor, EditorMenuItemDescriptor,
};
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_plugin_editor_support::{
    register_authoring_contribution_batch, register_authoring_extensions,
    EditorAuthoringContributionBatch, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_runtime_interface::resource::ResourceKind;

use crate::{
    CAPABILITY, PLUGIN_ID, TILEMAP_AUTHORING_VIEW_ID, TILEMAP_DRAWER_ID, TILEMAP_TEMPLATE_ID,
};

#[derive(Clone, Debug)]
pub struct Tilemap2dEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl Tilemap2dEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for Tilemap2dEditorPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut zircon_editor::core::editor_extension::EditorExtensionRegistry,
    ) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
        register_authoring_extensions(
            registry,
            EditorAuthoringExtensions {
                drawer_id: TILEMAP_DRAWER_ID,
                drawer_display_name: "Tilemap Tools",
                template_id: TILEMAP_TEMPLATE_ID,
                template_document: "plugins://tilemap_2d/editor/authoring.zui",
                surfaces: &[EditorAuthoringSurface::new(
                    TILEMAP_AUTHORING_VIEW_ID,
                    "Tilemap 2D",
                    "World",
                    "Plugins/Tilemap 2D",
                )],
            },
        )?;
        register_authoring_contribution_batch(registry, tilemap_authoring_batch())
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Tilemap 2D",
        "zircon_plugin_tilemap_2d_editor",
    )
    .with_capability(CAPABILITY)
}

pub fn editor_plugin() -> Tilemap2dEditorPlugin {
    Tilemap2dEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(
        &editor_plugin(),
        zircon_plugin_tilemap_2d_runtime::package_manifest(),
    )
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        zircon_plugin_tilemap_2d_runtime::package_manifest(),
    )
}

fn tilemap_authoring_batch() -> EditorAuthoringContributionBatch {
    let import_tiled = operation("tilemap_2d.authoring.import_tiled");
    let create_tilemap = operation("tilemap_2d.authoring.create_tilemap");
    let create_tileset = operation("tilemap_2d.authoring.create_tileset");
    let open = operation("tilemap_2d.authoring.open");
    let paint = operation("tilemap_2d.authoring.paint");
    EditorAuthoringContributionBatch {
        commands: vec![
            EditorCommandDescriptor::pending_operation(
                import_tiled.clone(),
                "Import Tiled Tilemap",
            )
            .with_menu_path("Plugins/Tilemap 2D/Import Tiled")
            .with_payload_schema_id("tilemap_2d.import_tiled.v1")
            .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::pending_operation(create_tilemap.clone(), "Create Tilemap")
                .with_menu_path("Plugins/Tilemap 2D/Create Tilemap")
                .with_payload_schema_id("tilemap_2d.create_tilemap.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::pending_operation(create_tileset.clone(), "Create Tileset")
                .with_menu_path("Plugins/Tilemap 2D/Create Tileset")
                .with_payload_schema_id("tilemap_2d.create_tileset.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::pending_operation(open.clone(), "Open Tilemap")
                .with_menu_path("Plugins/Tilemap 2D/Open Tilemap Asset")
                .with_payload_schema_id("tilemap_2d.open_asset.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::pending_operation(paint.clone(), "Paint Tilemap")
                .with_menu_path("Plugins/Tilemap 2D/Paint")
                .with_payload_schema_id("tilemap_2d.paint.v1")
                .with_required_capabilities([CAPABILITY]),
        ],
        menu_items: vec![
            menu_item("Plugins/Tilemap 2D/Import Tiled", &import_tiled),
            menu_item("Plugins/Tilemap 2D/Create Tilemap", &create_tilemap),
            menu_item("Plugins/Tilemap 2D/Create Tileset", &create_tileset),
            menu_item("Plugins/Tilemap 2D/Open Tilemap Asset", &open),
            menu_item("Plugins/Tilemap 2D/Paint", &paint),
        ],
        asset_importers: vec![AssetImporterDescriptor::new(
            "tilemap_2d.tiled.importer",
            "Tiled Tilemap",
            import_tiled,
        )
        .with_source_extensions(["tmx", "tsx", "json"])
        .with_output_type(AssetTypeId::from_resource_kind(ResourceKind::TileMap))
        .with_required_capabilities([CAPABILITY])],
        asset_type_contributions: vec![
            AssetTypeContribution::augment(AssetTypeId::from_resource_kind(ResourceKind::TileMap))
                .with_toolkit(
                    AssetToolkitDescriptor::new(TILEMAP_AUTHORING_VIEW_ID, open)
                        .with_required_capabilities([CAPABILITY]),
                )
                .with_creation_template(
                    AssetCreationTemplateDescriptor::new(
                        "tilemap_2d.template.tilemap",
                        "Tilemap",
                        create_tilemap,
                    )
                    .with_required_capabilities([CAPABILITY]),
                ),
            AssetTypeContribution::augment(AssetTypeId::from_resource_kind(ResourceKind::TileSet))
                .with_creation_template(
                    AssetCreationTemplateDescriptor::new(
                        "tilemap_2d.template.tileset",
                        "Tileset",
                        create_tileset,
                    )
                    .with_required_capabilities([CAPABILITY]),
                ),
        ],
        component_drawers: vec![ComponentDrawerDescriptor::new(
            zircon_plugin_tilemap_2d_runtime::TILEMAP_COMPONENT_TYPE,
            "plugins://tilemap_2d/editor/tilemap_component.zui",
            "tilemap_2d.editor.component",
        )],
        viewport_tool_modes: vec![ViewportToolModeDescriptor::new(
            "tilemap_2d.tool.paint",
            "Paint Tiles",
            TILEMAP_AUTHORING_VIEW_ID,
            paint,
        )
        .with_required_capabilities([CAPABILITY])],
        ..Default::default()
    }
}

fn operation(path: &str) -> EditorOperationPath {
    EditorOperationPath::parse(path).expect("valid tilemap operation path")
}

fn menu_item(path: &str, operation: &EditorOperationPath) -> EditorMenuItemDescriptor {
    EditorMenuItemDescriptor::new(path, operation.clone()).with_required_capabilities([CAPABILITY])
}

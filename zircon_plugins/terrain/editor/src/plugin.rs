use zircon_editor::core::asset::{
    AssetCreationTemplateDescriptor, AssetToolkitDescriptor, AssetTypeContribution, AssetTypeId,
};
use zircon_editor::core::commands::{EditorCommandDescriptor, EditorCommandMenuPath};
use zircon_editor::core::editor_extension::AssetImporterDescriptor;
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_editor::core::extension::InspectorCustomizationDescriptor;
use zircon_plugin_editor_support::{
    register_authoring_contribution_batch, register_authoring_extensions,
    EditorAuthoringContributionBatch, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_runtime_interface::resource::ResourceKind;

use crate::{
    CAPABILITY, PLUGIN_ID, TERRAIN_AUTHORING_VIEW_ID, TERRAIN_DRAWER_ID, TERRAIN_TEMPLATE_ID,
};

#[derive(Clone, Debug)]
pub struct TerrainEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl TerrainEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for TerrainEditorPlugin {
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
                drawer_id: TERRAIN_DRAWER_ID,
                drawer_display_name: "Terrain Tools",
                template_id: TERRAIN_TEMPLATE_ID,
                template_document: "plugins://terrain/editor/authoring.zui",
                surfaces: &[EditorAuthoringSurface::new(
                    TERRAIN_AUTHORING_VIEW_ID,
                    "Terrain",
                    "World",
                )],
            },
        )?;
        register_authoring_contribution_batch(registry, terrain_authoring_batch())
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(PLUGIN_ID, "Terrain", "zircon_plugin_terrain_editor")
        .with_capability(CAPABILITY)
}

pub fn editor_plugin() -> TerrainEditorPlugin {
    TerrainEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(
        &editor_plugin(),
        zircon_plugin_terrain_runtime::package_manifest(),
    )
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        zircon_plugin_terrain_runtime::package_manifest(),
    )
}

fn terrain_authoring_batch() -> EditorAuthoringContributionBatch {
    let import_heightfield = operation("terrain.authoring.import_heightfield");
    let import_weightmap = operation("terrain.authoring.import_weightmap");
    let create = operation("terrain.authoring.create_heightfield");
    let open = operation("terrain.authoring.open");
    let sculpt = operation("terrain.authoring.sculpt");
    EditorAuthoringContributionBatch {
        commands: vec![
            EditorCommandDescriptor::operation(import_heightfield.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &import_heightfield,
                    "plugins",
                    &["terrain"],
                ))
                .with_payload_schema_id("terrain.import_heightfield.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::operation(import_weightmap.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &import_weightmap,
                    "plugins",
                    &["terrain"],
                ))
                .with_payload_schema_id("terrain.import_weightmap.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::operation(create.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &create,
                    "plugins",
                    &["terrain"],
                ))
                .with_payload_schema_id("terrain.create_heightfield.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::operation(open.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &open,
                    "plugins",
                    &["terrain"],
                ))
                .with_payload_schema_id("terrain.open_asset.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::operation(sculpt.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &sculpt,
                    "plugins",
                    &["terrain"],
                ))
                .with_payload_schema_id("terrain.activate_sculpt_tool.v1")
                .with_required_capabilities([CAPABILITY]),
        ],
        menu_items: Vec::new(),
        asset_importers: vec![
            AssetImporterDescriptor::new(
                "terrain.heightfield.importer",
                "Terrain Heightfield",
                import_heightfield,
            )
            .with_source_extensions(["raw", "r16", "png"])
            .with_output_type(AssetTypeId::from_resource_kind(ResourceKind::Terrain))
            .with_required_capabilities([CAPABILITY]),
            AssetImporterDescriptor::new(
                "terrain.weightmap.importer",
                "Terrain Weightmap",
                import_weightmap,
            )
            .with_source_extensions(["raw", "r16", "png"])
            .with_output_type(AssetTypeId::from_resource_kind(
                ResourceKind::TerrainLayerStack,
            ))
            .with_required_capabilities([CAPABILITY]),
        ],
        asset_type_contributions: vec![AssetTypeContribution::augment(
            AssetTypeId::from_resource_kind(ResourceKind::Terrain),
        )
        .with_toolkit(
            AssetToolkitDescriptor::new(TERRAIN_AUTHORING_VIEW_ID, open)
                .with_required_capabilities([CAPABILITY]),
        )
        .with_creation_template(
            AssetCreationTemplateDescriptor::new(
                "terrain.template.heightfield",
                "Terrain Heightfield",
                create,
            )
            .with_default_document("plugins://terrain/templates/default_heightfield.toml")
            .with_required_capabilities([CAPABILITY]),
        )],
        inspector_customizations: vec![InspectorCustomizationDescriptor::new(
            zircon_plugin_terrain_runtime::TERRAIN_COMPONENT_TYPE,
            "plugins://terrain/editor/terrain_component.zui",
            "terrain.editor.component",
        )],
        ..Default::default()
    }
}

fn operation(path: &str) -> EditorOperationPath {
    EditorOperationPath::parse(path).expect("valid terrain operation path")
}

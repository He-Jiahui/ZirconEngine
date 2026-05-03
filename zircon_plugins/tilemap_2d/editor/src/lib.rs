use zircon_editor::core::editor_authoring_extension::{
    AssetCreationTemplateDescriptor, ViewportToolModeDescriptor,
};
use zircon_editor::core::editor_extension::{
    AssetEditorDescriptor, AssetImporterDescriptor, ComponentDrawerDescriptor,
    EditorMenuItemDescriptor,
};
use zircon_editor::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};
use zircon_plugin_editor_support::{
    register_authoring_contribution_batch, register_authoring_extensions,
    EditorAuthoringContributionBatch, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_runtime::asset::{TileMapAsset, TileMapProjectionAsset};

pub const PLUGIN_ID: &str = zircon_plugin_tilemap_2d_runtime::PLUGIN_ID;
pub const CAPABILITY: &str = "editor.extension.tilemap_2d_authoring";
pub const TILEMAP_AUTHORING_VIEW_ID: &str = "tilemap_2d.authoring";
pub const TILEMAP_DRAWER_ID: &str = "tilemap_2d.drawer";
pub const TILEMAP_TEMPLATE_ID: &str = "tilemap_2d.authoring";

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
                template_document: "plugins://tilemap_2d/editor/authoring.ui.toml",
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
    let import_tiled = operation("Tilemap2d.Authoring.ImportTiled");
    let create_tilemap = operation("Tilemap2d.Authoring.CreateTilemap");
    let create_tileset = operation("Tilemap2d.Authoring.CreateTileset");
    let open = operation("Tilemap2d.Authoring.Open");
    let paint = operation("Tilemap2d.Authoring.Paint");
    EditorAuthoringContributionBatch {
        operations: vec![
            EditorOperationDescriptor::new(import_tiled.clone(), "Import Tiled Tilemap")
                .with_menu_path("Plugins/Tilemap 2D/Import Tiled")
                .with_payload_schema_id("tilemap_2d.import_tiled.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(create_tilemap.clone(), "Create Tilemap")
                .with_menu_path("Plugins/Tilemap 2D/Create Tilemap")
                .with_payload_schema_id("tilemap_2d.create_tilemap.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(create_tileset.clone(), "Create Tileset")
                .with_menu_path("Plugins/Tilemap 2D/Create Tileset")
                .with_payload_schema_id("tilemap_2d.create_tileset.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(open.clone(), "Open Tilemap")
                .with_menu_path("Plugins/Tilemap 2D/Open Tilemap Asset")
                .with_payload_schema_id("tilemap_2d.open_asset.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(paint.clone(), "Paint Tilemap")
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
        .with_output_kind("tilemap_2d.tilemap")
        .with_required_capabilities([CAPABILITY])],
        asset_editors: vec![AssetEditorDescriptor::new(
            "tilemap_2d.tilemap",
            TILEMAP_AUTHORING_VIEW_ID,
            "Tilemap 2D",
            open,
        )
        .with_required_capabilities([CAPABILITY])],
        component_drawers: vec![ComponentDrawerDescriptor::new(
            zircon_plugin_tilemap_2d_runtime::TILEMAP_COMPONENT_TYPE,
            "plugins://tilemap_2d/editor/tilemap_component.ui.toml",
            "tilemap_2d.editor.component",
        )],
        asset_creation_templates: vec![
            AssetCreationTemplateDescriptor::new(
                "tilemap_2d.template.tilemap",
                "Tilemap",
                "tilemap_2d.tilemap",
                create_tilemap,
            )
            .with_required_capabilities([CAPABILITY]),
            AssetCreationTemplateDescriptor::new(
                "tilemap_2d.template.tileset",
                "Tileset",
                "tilemap_2d.tileset",
                create_tileset,
            )
            .with_required_capabilities([CAPABILITY]),
        ],
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TilemapEditorStats {
    pub layer_count: usize,
    pub occupied_tile_count: usize,
    pub empty_tile_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TilemapPaintRequest {
    pub layer_index: usize,
    pub x: u32,
    pub y: u32,
    pub tile_id: Option<u32>,
}

pub fn validate_tilemap_for_editor(tilemap: &TileMapAsset) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if tilemap.width == 0 || tilemap.height == 0 {
        diagnostics.push("tilemap dimensions must be greater than zero".to_string());
    }
    if let Err(error) = tilemap.validate_layers() {
        diagnostics.push(error);
    }
    if !supported_projection(tilemap.projection) {
        diagnostics.push("tilemap projection is not supported".to_string());
    }
    diagnostics
}

pub fn apply_tilemap_paint(
    tilemap: &mut TileMapAsset,
    request: &TilemapPaintRequest,
) -> Result<TilemapEditorStats, Vec<String>> {
    let mut diagnostics = validate_tilemap_for_editor(tilemap);
    if request.layer_index >= tilemap.layers.len() {
        diagnostics.push(format!(
            "tilemap paint layer index {} is outside {} layers",
            request.layer_index,
            tilemap.layers.len()
        ));
    }
    if request.x >= tilemap.width || request.y >= tilemap.height {
        diagnostics.push(format!(
            "tilemap paint cell {},{} is outside {}x{} map",
            request.x, request.y, tilemap.width, tilemap.height
        ));
    }
    if !diagnostics.is_empty() {
        diagnostics.sort();
        diagnostics.dedup();
        return Err(diagnostics);
    }

    let tile_index = tilemap_cell_index(tilemap.width, request.x, request.y)
        .expect("validated tilemap paint cell has an index");
    tilemap.layers[request.layer_index].tiles[tile_index] = request.tile_id;
    Ok(tilemap_editor_stats(tilemap))
}

pub fn tilemap_editor_stats(tilemap: &TileMapAsset) -> TilemapEditorStats {
    let occupied_tile_count = tilemap
        .layers
        .iter()
        .flat_map(|layer| layer.tiles.iter())
        .filter(|tile| tile.is_some())
        .count();
    let total_tile_count = tilemap
        .layers
        .iter()
        .map(|layer| layer.tiles.len())
        .sum::<usize>();
    TilemapEditorStats {
        layer_count: tilemap.layers.len(),
        occupied_tile_count,
        empty_tile_count: total_tile_count.saturating_sub(occupied_tile_count),
    }
}

fn tilemap_cell_index(width: u32, x: u32, y: u32) -> Option<usize> {
    y.checked_mul(width)
        .and_then(|row| row.checked_add(x))
        .map(|index| index as usize)
}

pub fn supported_projection(projection: TileMapProjectionAsset) -> bool {
    matches!(
        projection,
        TileMapProjectionAsset::Orthogonal
            | TileMapProjectionAsset::IsometricDiamond
            | TileMapProjectionAsset::IsometricStaggered
            | TileMapProjectionAsset::HexagonalStaggered
    )
}

fn operation(path: &str) -> EditorOperationPath {
    EditorOperationPath::parse(path).expect("valid tilemap operation path")
}

fn menu_item(path: &str, operation: &EditorOperationPath) -> EditorMenuItemDescriptor {
    EditorMenuItemDescriptor::new(path, operation.clone()).with_required_capabilities([CAPABILITY])
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_editor::EditorPlugin;
    use zircon_runtime::asset::{AssetReference, AssetUri, TileMapLayerAsset};

    #[test]
    fn tilemap_authoring_registration_exposes_menu_items_and_payload_schemas() {
        let mut registry =
            zircon_editor::core::editor_extension::EditorExtensionRegistry::default();
        editor_plugin()
            .register_editor_extensions(&mut registry)
            .expect("tilemap authoring registration");
        let operation = operation("Tilemap2d.Authoring.Paint");
        let descriptor = registry
            .operations()
            .descriptor(&operation)
            .expect("paint operation registered");

        assert_eq!(descriptor.menu_path(), Some("Plugins/Tilemap 2D/Paint"));
        assert_eq!(descriptor.payload_schema_id(), Some("tilemap_2d.paint.v1"));
        assert!(registry.menu_items().iter().any(|item| {
            item.path() == "Plugins/Tilemap 2D/Paint" && item.operation() == &operation
        }));
    }

    #[test]
    fn tilemap_editor_validation_accepts_supported_projection_and_layer_size() {
        let tilemap = tilemap_with_layer(vec![Some(1), None, Some(2), None]);

        assert!(validate_tilemap_for_editor(&tilemap).is_empty());
        assert_eq!(
            tilemap_editor_stats(&tilemap),
            TilemapEditorStats {
                layer_count: 1,
                occupied_tile_count: 2,
                empty_tile_count: 2,
            }
        );
    }

    #[test]
    fn tilemap_editor_validation_reports_layer_size_errors() {
        let mut tilemap = tilemap_with_layer(vec![Some(1)]);
        tilemap.width = 2;
        tilemap.height = 2;

        let diagnostics = validate_tilemap_for_editor(&tilemap);

        assert!(diagnostics
            .iter()
            .any(|message| message.contains("stores 1 tiles for 2x2 map")));
    }

    #[test]
    fn tilemap_paint_updates_selected_cell_and_returns_stats() {
        let mut tilemap = tilemap_with_layer(vec![None, None, None, None]);

        let stats = apply_tilemap_paint(
            &mut tilemap,
            &TilemapPaintRequest {
                layer_index: 0,
                x: 1,
                y: 0,
                tile_id: Some(7),
            },
        )
        .expect("paint request is valid");

        assert_eq!(tilemap.layers[0].tiles, vec![None, Some(7), None, None]);
        assert_eq!(
            stats,
            TilemapEditorStats {
                layer_count: 1,
                occupied_tile_count: 1,
                empty_tile_count: 3,
            }
        );
    }

    #[test]
    fn tilemap_paint_reports_out_of_range_layer_and_cell() {
        let mut tilemap = tilemap_with_layer(vec![None, None, None, None]);

        let diagnostics = apply_tilemap_paint(
            &mut tilemap,
            &TilemapPaintRequest {
                layer_index: 2,
                x: 4,
                y: 0,
                tile_id: Some(7),
            },
        )
        .expect_err("paint request is outside layer and grid bounds");

        assert!(diagnostics
            .iter()
            .any(|message| message.contains("outside 1 layers")));
        assert!(diagnostics
            .iter()
            .any(|message| message.contains("outside 2x2 map")));
    }

    #[test]
    fn tilemap_projection_support_covers_paper2d_style_defaults() {
        assert!(supported_projection(TileMapProjectionAsset::Orthogonal));
        assert!(supported_projection(
            TileMapProjectionAsset::IsometricDiamond
        ));
        assert!(supported_projection(
            TileMapProjectionAsset::IsometricStaggered
        ));
        assert!(supported_projection(
            TileMapProjectionAsset::HexagonalStaggered
        ));
    }

    fn tilemap_with_layer(tiles: Vec<Option<u32>>) -> TileMapAsset {
        TileMapAsset {
            uri: AssetUri::parse("res://tilemaps/test.tilemap.toml").unwrap(),
            width: 2,
            height: 2,
            projection: TileMapProjectionAsset::Orthogonal,
            tile_set: asset_ref("res://tilemaps/test.tileset.toml"),
            layers: vec![TileMapLayerAsset {
                name: "Ground".to_string(),
                visible: true,
                opacity: 1.0,
                tiles,
            }],
        }
    }

    fn asset_ref(locator: &str) -> AssetReference {
        AssetReference::from_locator(AssetUri::parse(locator).unwrap())
    }
}

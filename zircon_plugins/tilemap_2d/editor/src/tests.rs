use super::*;
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_editor::EditorPlugin;
use zircon_runtime::asset::{AssetReference, AssetUri, TileMapAsset, TileMapLayerAsset};

#[test]
fn tilemap_authoring_registration_exposes_menu_items_and_payload_schemas() {
    let mut registry = zircon_editor::core::editor_extension::EditorExtensionRegistry::default();
    editor_plugin()
        .register_editor_extensions(&mut registry)
        .expect("tilemap authoring registration");
    let operation =
        EditorOperationPath::parse("tilemap_2d.authoring.paint").expect("operation path");
    let descriptor = registry
        .commands()
        .command(&operation)
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
    assert!(supported_projection(
        zircon_runtime::asset::TileMapProjectionAsset::Orthogonal
    ));
    assert!(supported_projection(
        zircon_runtime::asset::TileMapProjectionAsset::IsometricDiamond
    ));
    assert!(supported_projection(
        zircon_runtime::asset::TileMapProjectionAsset::IsometricStaggered
    ));
    assert!(supported_projection(
        zircon_runtime::asset::TileMapProjectionAsset::HexagonalStaggered
    ));
}

fn tilemap_with_layer(tiles: Vec<Option<u32>>) -> TileMapAsset {
    TileMapAsset {
        uri: AssetUri::parse("res://tilemaps/test.tilemap.toml").unwrap(),
        width: 2,
        height: 2,
        projection: zircon_runtime::asset::TileMapProjectionAsset::Orthogonal,
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

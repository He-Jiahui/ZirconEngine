use super::*;
use std::hint::black_box;
use std::time::{Duration, Instant};
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

    assert_eq!(
        descriptor
            .menu_path()
            .expect("paint command menu path")
            .stable_path(),
        "plugins/tilemap_2d/tilemap_2d.authoring.paint"
    );
    assert_eq!(descriptor.payload_schema_id(), Some("tilemap_2d.paint.v1"));
    assert!(registry.menu_items().is_empty());
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
            layer: TilemapLayerId::try_new("Ground").unwrap(),
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
            layer: TilemapLayerId::try_new("Missing").unwrap(),
            x: 4,
            y: 0,
            tile_id: Some(7),
        },
    )
    .expect_err("paint request is outside layer and grid bounds");

    assert!(diagnostics
        .iter()
        .any(|message| message.contains("layer `Missing` is not present")));
    assert!(diagnostics
        .iter()
        .any(|message| message.contains("outside 2x2 map")));
}

#[test]
fn tilemap_paint_resolves_layer_identity_after_reorder() {
    let mut tilemap = tilemap_with_dimensions(2, 2, &["Ground", "Objects"]);
    tilemap.layers.reverse();

    apply_tilemap_paint(
        &mut tilemap,
        &TilemapPaintRequest {
            layer: TilemapLayerId::try_new("Objects").unwrap(),
            x: 0,
            y: 1,
            tile_id: Some(11),
        },
    )
    .expect("layer identity survives array reorder");

    let objects = tilemap
        .layers
        .iter()
        .find(|layer| layer.name == "Objects")
        .unwrap();
    assert_eq!(objects.tiles, vec![None, None, Some(11), None]);
}

#[test]
fn tilemap_paint_stroke_failure_preserves_the_entire_asset() {
    let mut tilemap = tilemap_with_dimensions(2, 2, &["Ground", "Objects"]);
    let before = tilemap.clone();
    let duplicate_cell = TilemapPaintRequest {
        layer: TilemapLayerId::try_new("Ground").unwrap(),
        x: 0,
        y: 0,
        tile_id: Some(3),
    };
    let requests = vec![
        duplicate_cell.clone(),
        duplicate_cell,
        TilemapPaintRequest {
            layer: TilemapLayerId::try_new("Objects").unwrap(),
            x: 9,
            y: 0,
            tile_id: Some(7),
        },
    ];

    let diagnostics = apply_tilemap_paint_stroke(&mut tilemap, &requests)
        .expect_err("ambiguous and out-of-range stroke fails atomically");

    assert!(diagnostics
        .iter()
        .any(|message| message.contains("duplicate tilemap paint cell")));
    assert!(diagnostics
        .iter()
        .any(|message| message.contains("outside 2x2 map")));
    assert_eq!(tilemap, before);
}

#[test]
fn tilemap_paint_stroke_updates_stats_incrementally() {
    let mut tilemap = tilemap_with_dimensions(2, 2, &["Ground", "Objects"]);
    tilemap.layers[0].tiles[0] = Some(1);
    tilemap.layers[1].tiles[3] = Some(2);
    let requests = vec![
        TilemapPaintRequest {
            layer: TilemapLayerId::try_new("Ground").unwrap(),
            x: 0,
            y: 0,
            tile_id: None,
        },
        TilemapPaintRequest {
            layer: TilemapLayerId::try_new("Objects").unwrap(),
            x: 1,
            y: 0,
            tile_id: Some(9),
        },
    ];

    let receipt =
        apply_tilemap_paint_stroke(&mut tilemap, &requests).expect("bounded stroke is valid");

    assert_eq!(receipt.requested_cell_count, 2);
    assert_eq!(receipt.changed_cell_count, 2);
    assert_eq!(
        receipt.stats,
        TilemapEditorStats {
            layer_count: 2,
            occupied_tile_count: 2,
            empty_tile_count: 6,
        }
    );
}

#[test]
fn tilemap_paint_stroke_rejects_budget_and_duplicate_layer_identity() {
    let request = TilemapPaintRequest {
        layer: TilemapLayerId::try_new("Ground").unwrap(),
        x: 0,
        y: 0,
        tile_id: Some(1),
    };
    let mut tilemap = tilemap_with_dimensions(2, 2, &["Ground"]);
    let before = tilemap.clone();
    let oversized = vec![request; TILEMAP_PAINT_STROKE_MAX_CELLS + 1];

    let diagnostics = apply_tilemap_paint_stroke(&mut tilemap, &oversized)
        .expect_err("oversized stroke fails before scratch allocation");
    assert!(diagnostics[0].contains("exceeding the 4096-cell limit"));
    assert_eq!(tilemap, before);

    let duplicate_layer = tilemap.layers.first().expect("ground layer").clone();
    tilemap.layers.push(duplicate_layer);
    let diagnostics = validate_tilemap_for_editor(&tilemap);
    assert!(diagnostics
        .iter()
        .any(|message| message.contains("duplicate tilemap layer identity `Ground`")));
}

#[test]
#[ignore = "release performance gate"]
fn tilemap_paint_stroke_release_gate_avoids_per_cell_full_map_scans() {
    const SAMPLE_PAIRS: usize = 21;
    const WIDTH: u32 = 64;
    const HEIGHT: u32 = 64;
    const LAYERS: usize = 4;
    const STROKE_CELLS: usize = 128;
    const REQUIRED_IMPROVEMENT_PERCENT: u128 = 80;

    let layer_names = (0..LAYERS)
        .map(|index| format!("Layer{index}"))
        .collect::<Vec<_>>();
    let layer_refs = layer_names.iter().map(String::as_str).collect::<Vec<_>>();
    let base = tilemap_with_dimensions(WIDTH, HEIGHT, &layer_refs);
    let requests = (0..STROKE_CELLS)
        .map(|index| TilemapPaintRequest {
            layer: TilemapLayerId::try_new("Layer2").unwrap(),
            x: (index as u32) % WIDTH,
            y: (index as u32) / WIDTH,
            tile_id: Some((index + 1) as u32),
        })
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure_tilemap_stroke(
                &base,
                &requests,
                legacy_apply_tilemap_paint_stroke,
            ));
            optimized_samples.push(measure_tilemap_stroke(
                &base,
                &requests,
                optimized_apply_tilemap_paint_stroke,
            ));
        } else {
            optimized_samples.push(measure_tilemap_stroke(
                &base,
                &requests,
                optimized_apply_tilemap_paint_stroke,
            ));
            legacy_samples.push(measure_tilemap_stroke(
                &base,
                &requests,
                legacy_apply_tilemap_paint_stroke,
            ));
        }
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples).as_nanos();
    let optimized_p95 = nearest_rank_p95(&optimized_samples).as_nanos();
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins08_tilemap_bounded_stroke sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even width={WIDTH} height={HEIGHT} layers={LAYERS} stroke_cells={STROKE_CELLS} legacy_full_map_scans={STROKE_CELLS} optimized_full_map_scans=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={REQUIRED_IMPROVEMENT_PERCENT}",
        durations_csv(&legacy_samples),
        durations_csv(&optimized_samples)
    );
    assert!(
        improvement_percent >= REQUIRED_IMPROVEMENT_PERCENT,
        "bounded stroke must improve P95 by at least {REQUIRED_IMPROVEMENT_PERCENT}% (legacy={legacy_p95}ns optimized={optimized_p95}ns improvement={improvement_percent}%)"
    );
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

fn tilemap_with_dimensions(width: u32, height: u32, layer_names: &[&str]) -> TileMapAsset {
    let tile_count = (width as usize) * (height as usize);
    TileMapAsset {
        uri: AssetUri::parse("res://tilemaps/stroke.tilemap.toml").unwrap(),
        width,
        height,
        projection: zircon_runtime::asset::TileMapProjectionAsset::Orthogonal,
        tile_set: asset_ref("res://tilemaps/test.tileset.toml"),
        layers: layer_names
            .iter()
            .map(|name| TileMapLayerAsset {
                name: (*name).to_string(),
                visible: true,
                opacity: 1.0,
                tiles: vec![None; tile_count],
            })
            .collect(),
    }
}

fn legacy_apply_tilemap_paint_stroke(
    tilemap: &mut TileMapAsset,
    requests: &[TilemapPaintRequest],
) -> TilemapPaintStrokeReceipt {
    let layer_index = tilemap
        .layers
        .iter()
        .position(|layer| layer.name == requests[0].layer.as_str())
        .unwrap();
    let mut stats = None;
    let mut changed_cell_count = 0usize;
    for request in requests {
        assert_eq!(request.layer.as_str(), requests[0].layer.as_str());
        let tile_index = (request.y * tilemap.width + request.x) as usize;
        let tile = &mut tilemap.layers[layer_index].tiles[tile_index];
        changed_cell_count += usize::from(*tile != request.tile_id);
        *tile = request.tile_id;
        stats = Some(tilemap_editor_stats(tilemap));
    }
    TilemapPaintStrokeReceipt {
        requested_cell_count: requests.len(),
        changed_cell_count,
        stats: stats.unwrap_or_else(|| tilemap_editor_stats(tilemap)),
    }
}

fn optimized_apply_tilemap_paint_stroke(
    tilemap: &mut TileMapAsset,
    requests: &[TilemapPaintRequest],
) -> TilemapPaintStrokeReceipt {
    apply_tilemap_paint_stroke(tilemap, requests).expect("benchmark stroke remains valid")
}

fn measure_tilemap_stroke(
    base: &TileMapAsset,
    requests: &[TilemapPaintRequest],
    apply: fn(&mut TileMapAsset, &[TilemapPaintRequest]) -> TilemapPaintStrokeReceipt,
) -> Duration {
    let mut tilemap = base.clone();
    let started = Instant::now();
    let receipt = black_box(apply(black_box(&mut tilemap), black_box(requests)));
    let elapsed = started.elapsed();
    assert_eq!(receipt.changed_cell_count, requests.len());
    black_box((tilemap, receipt));
    elapsed
}

fn nearest_rank_p95(samples: &[Duration]) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[((sorted.len() * 95).div_ceil(100)).saturating_sub(1)]
}

fn durations_csv(samples: &[Duration]) -> String {
    samples
        .iter()
        .map(|sample| sample.as_nanos().to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn asset_ref(locator: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(locator).unwrap())
}

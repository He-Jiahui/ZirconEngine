use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::asset::{TileMapAsset, TileMapProjectionAsset};

pub const TILEMAP_PAINT_STROKE_MAX_CELLS: usize = 4_096;
const TILEMAP_LAYER_ID_MAX_BYTES: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TilemapLayerId(String);

impl TilemapLayerId {
    pub fn try_new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err("tilemap layer identity must not be empty".to_string());
        }
        if value.trim() != value {
            return Err(
                "tilemap layer identity must not contain surrounding whitespace".to_string(),
            );
        }
        if value.len() > TILEMAP_LAYER_ID_MAX_BYTES {
            return Err(format!(
                "tilemap layer identity exceeds {TILEMAP_LAYER_ID_MAX_BYTES} bytes"
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
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
    pub layer: TilemapLayerId,
    pub x: u32,
    pub y: u32,
    pub tile_id: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TilemapPaintStrokeReceipt {
    pub requested_cell_count: usize,
    pub changed_cell_count: usize,
    pub stats: TilemapEditorStats,
}

pub fn validate_tilemap_for_editor(tilemap: &TileMapAsset) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let mut layer_ids = BTreeSet::new();
    if tilemap.width == 0 || tilemap.height == 0 {
        diagnostics.push("tilemap dimensions must be greater than zero".to_string());
    }
    if let Err(error) = tilemap.validate_layers() {
        diagnostics.push(error.to_string());
    }
    if !supported_projection(tilemap.projection) {
        diagnostics.push("tilemap projection is not supported".to_string());
    }
    for layer in &tilemap.layers {
        match TilemapLayerId::try_new(layer.name.clone()) {
            Ok(layer_id) if layer_ids.insert(layer_id) => {}
            Ok(layer_id) => diagnostics.push(format!(
                "duplicate tilemap layer identity `{}` is ambiguous",
                layer_id.as_str()
            )),
            Err(error) => diagnostics.push(error),
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub fn apply_tilemap_paint(
    tilemap: &mut TileMapAsset,
    request: &TilemapPaintRequest,
) -> Result<TilemapEditorStats, Vec<String>> {
    apply_tilemap_paint_stroke(tilemap, std::slice::from_ref(request)).map(|receipt| receipt.stats)
}

pub fn apply_tilemap_paint_stroke(
    tilemap: &mut TileMapAsset,
    requests: &[TilemapPaintRequest],
) -> Result<TilemapPaintStrokeReceipt, Vec<String>> {
    let mut diagnostics = validate_tilemap_for_editor(tilemap);
    if requests.is_empty() {
        diagnostics.push("tilemap paint stroke must contain at least one cell".to_string());
    }
    if requests.len() > TILEMAP_PAINT_STROKE_MAX_CELLS {
        return Err(vec![format!(
            "tilemap paint stroke contains {} cells, exceeding the {}-cell limit",
            requests.len(),
            TILEMAP_PAINT_STROKE_MAX_CELLS
        )]);
    }
    let layer_indices = tilemap
        .layers
        .iter()
        .enumerate()
        .map(|(index, layer)| (layer.name.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let mut requested_cells = BTreeSet::new();
    let mut resolved_cells = Vec::with_capacity(requests.len());
    for request in requests {
        let Some(&layer_index) = layer_indices.get(request.layer.as_str()) else {
            diagnostics.push(format!(
                "tilemap paint layer `{}` is not present",
                request.layer.as_str()
            ));
            continue;
        };
        if request.x >= tilemap.width || request.y >= tilemap.height {
            diagnostics.push(format!(
                "tilemap paint cell {},{} is outside {}x{} map",
                request.x, request.y, tilemap.width, tilemap.height
            ));
            continue;
        }
        if !requested_cells.insert((layer_index, request.x, request.y)) {
            diagnostics.push(format!(
                "duplicate tilemap paint cell `{}` {},{} is ambiguous",
                request.layer.as_str(),
                request.x,
                request.y
            ));
            continue;
        }
        let tile_index = tilemap_cell_index(tilemap.width, request.x, request.y)
            .expect("validated tilemap paint cell has an index");
        resolved_cells.push((layer_index, tile_index, request.tile_id));
    }
    if !diagnostics.is_empty() {
        diagnostics.sort();
        diagnostics.dedup();
        return Err(diagnostics);
    }

    let mut stats = tilemap_editor_stats(tilemap);
    let mut changed_cell_count = 0usize;
    for (layer_index, tile_index, tile_id) in resolved_cells {
        let tile = &mut tilemap.layers[layer_index].tiles[tile_index];
        if *tile == tile_id {
            continue;
        }
        changed_cell_count += 1;
        match (tile.is_some(), tile_id.is_some()) {
            (false, true) => {
                stats.occupied_tile_count += 1;
                stats.empty_tile_count -= 1;
            }
            (true, false) => {
                stats.occupied_tile_count -= 1;
                stats.empty_tile_count += 1;
            }
            _ => {}
        }
        *tile = tile_id;
    }
    Ok(TilemapPaintStrokeReceipt {
        requested_cell_count: requests.len(),
        changed_cell_count,
        stats,
    })
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

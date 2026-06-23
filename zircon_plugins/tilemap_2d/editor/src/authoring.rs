use zircon_runtime::asset::{TileMapAsset, TileMapProjectionAsset};

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

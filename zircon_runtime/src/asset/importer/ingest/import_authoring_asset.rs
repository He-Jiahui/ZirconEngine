use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, ImportedAsset, MaterialGraphAsset,
    TerrainAsset, TileMapAsset,
};

pub(super) fn import_prefab(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    parse_typed_toml(context, "prefab toml")
        .map(|asset| AssetImportOutcome::new(ImportedAsset::Prefab(asset)))
}

pub(super) fn import_material_graph(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let graph: MaterialGraphAsset = parse_typed_toml(context, "material graph toml")?;
    graph
        .validate_output_node()
        .map_err(AssetImportError::Parse)?;
    Ok(AssetImportOutcome::new(ImportedAsset::MaterialGraph(graph)))
}

pub(super) fn import_terrain(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let terrain: TerrainAsset = parse_typed_toml(context, "terrain toml")?;
    terrain
        .validate_dimensions()
        .map_err(AssetImportError::Parse)?;
    Ok(AssetImportOutcome::new(ImportedAsset::Terrain(terrain)))
}

pub(super) fn import_terrain_layer_stack(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    parse_typed_toml(context, "terrain layer stack toml")
        .map(|asset| AssetImportOutcome::new(ImportedAsset::TerrainLayerStack(asset)))
}

pub(super) fn import_tileset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    parse_typed_toml(context, "tileset toml")
        .map(|asset| AssetImportOutcome::new(ImportedAsset::TileSet(asset)))
}

pub(super) fn import_tilemap(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let tilemap: TileMapAsset = parse_typed_toml(context, "tilemap toml")?;
    tilemap.validate_layers().map_err(AssetImportError::Parse)?;
    Ok(AssetImportOutcome::new(ImportedAsset::TileMap(tilemap)))
}

pub(super) fn import_navmesh(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    parse_typed_toml(context, "navmesh toml")
        .map(|asset| AssetImportOutcome::new(ImportedAsset::NavMesh(asset)))
}

pub(super) fn import_navigation_settings(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    parse_typed_toml(context, "navigation settings toml")
        .map(|asset| AssetImportOutcome::new(ImportedAsset::NavigationSettings(asset)))
}

fn parse_typed_toml<T: serde::de::DeserializeOwned>(
    context: &AssetImportContext,
    label: &str,
) -> Result<T, AssetImportError> {
    let document = context.source_text()?;
    toml::from_str::<T>(&document)
        .map_err(|error| AssetImportError::Parse(format!("parse {label}: {error}")))
}

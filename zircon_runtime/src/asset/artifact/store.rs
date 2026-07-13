use std::fs;
use std::path::PathBuf;

use crate::core::resource::{ResourceRecord, ResourceScheme};

use super::cache_payload::ArtifactCacheAsset;
use crate::asset::project::ProjectPaths;
use crate::asset::{
    asset_kind_for_imported_asset, AssetImportError, AssetKind, AssetUri, ImportedAsset,
};

const ARTIFACT_CACHE_EXTENSION: &str = "zasset";
const ARTIFACT_CACHE_SUFFIX: &str = ".zasset";
const ARTIFACT_CACHE_MAGIC: &[u8] = b"ZRARTZ01";
const ARTIFACT_CACHE_ZSTD_LEVEL: i32 = 1;

#[derive(Clone, Debug, Default)]
pub struct ArtifactStore;

impl ArtifactStore {
    pub fn write(
        &self,
        paths: &ProjectPaths,
        metadata: &ResourceRecord,
        asset: &ImportedAsset,
    ) -> Result<AssetUri, AssetImportError> {
        let relative_path = format!(
            "{}/{}.{}",
            asset_kind_directory(metadata.kind),
            metadata.id(),
            ARTIFACT_CACHE_EXTENSION
        );
        let artifact_uri = AssetUri::parse(&format!("lib://{relative_path}"))?;
        let artifact_path = resolve_artifact_cache_path(paths, &artifact_uri)?;
        if let Some(parent) = artifact_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let payload = serialize_asset(asset)?;
        fs::write(&artifact_path, payload)?;
        Ok(artifact_uri)
    }

    pub fn read(
        &self,
        paths: &ProjectPaths,
        artifact_uri: &AssetUri,
    ) -> Result<ImportedAsset, AssetImportError> {
        let artifact_path = resolve_artifact_cache_path(paths, artifact_uri)?;
        let payload = fs::read(artifact_path)?;
        deserialize_asset(artifact_uri.path(), &payload)
    }
}

fn resolve_artifact_cache_path(
    paths: &ProjectPaths,
    artifact_uri: &AssetUri,
) -> Result<PathBuf, AssetImportError> {
    if artifact_uri.scheme() != ResourceScheme::Library {
        return Err(AssetImportError::UnsupportedFormat(format!(
            "artifact uri must use lib:// scheme: {artifact_uri}"
        )));
    }
    Ok(paths.asset_artifact_root().join(artifact_uri.path()))
}

fn asset_kind_directory(kind: AssetKind) -> &'static str {
    match kind {
        AssetKind::Data => "data",
        AssetKind::Texture => "textures",
        AssetKind::Shader => "shaders",
        AssetKind::Material => "materials",
        AssetKind::MaterialGraph => "materials/graphs",
        AssetKind::Sound => "sound",
        AssetKind::Font => "fonts",
        AssetKind::PhysicsMaterial => "physics/materials",
        AssetKind::NavMesh => "navigation/navmeshes",
        AssetKind::NavigationSettings => "navigation/settings",
        AssetKind::Terrain => "terrain/heightfields",
        AssetKind::TerrainLayerStack => "terrain/layers",
        AssetKind::TileSet => "tilemap_2d/tilesets",
        AssetKind::TileMap => "tilemap_2d/maps",
        AssetKind::Prefab => "prefabs",
        AssetKind::Scene => "scenes",
        AssetKind::Model => "models",
        AssetKind::Mesh => "meshes",
        AssetKind::AnimationSkeleton => "animation/skeletons",
        AssetKind::AnimationClip => "animation/clips",
        AssetKind::AnimationSequence => "animation/sequences",
        AssetKind::AnimationGraph => "animation/graphs",
        AssetKind::AnimationStateMachine => "animation/state_machines",
        AssetKind::UiLayout => "ui/layouts",
        AssetKind::UiWidget => "ui/widgets",
        AssetKind::UiStyle => "ui/styles",
    }
}

fn serialize_asset(asset: &ImportedAsset) -> Result<Vec<u8>, AssetImportError> {
    let cache_asset = ArtifactCacheAsset::from_imported(asset)?;
    let bytes =
        bincode::serialize(&cache_asset).map_err(AssetImportError::ArtifactCacheSerialize)?;
    let compressed = zstd::stream::encode_all(&bytes[..], ARTIFACT_CACHE_ZSTD_LEVEL)?;
    let mut payload = Vec::with_capacity(ARTIFACT_CACHE_MAGIC.len() + compressed.len());
    payload.extend_from_slice(ARTIFACT_CACHE_MAGIC);
    payload.extend_from_slice(&compressed);
    Ok(payload)
}

fn deserialize_asset(path: &str, payload: &[u8]) -> Result<ImportedAsset, AssetImportError> {
    if !path.ends_with(ARTIFACT_CACHE_SUFFIX) {
        return Err(AssetImportError::Parse(format!(
            "unsupported artifact cache extension for {path}; expected {ARTIFACT_CACHE_SUFFIX}"
        )));
    }
    if !payload.starts_with(ARTIFACT_CACHE_MAGIC) {
        return Err(AssetImportError::Parse(
            "unsupported artifact cache format; expected compressed binary cache".to_string(),
        ));
    }
    let expected_kind = asset_kind_from_artifact_path(path);
    let bytes = zstd::stream::decode_all(&payload[ARTIFACT_CACHE_MAGIC.len()..])?;
    let cache_asset = bincode::deserialize::<ArtifactCacheAsset>(&bytes)
        .map_err(AssetImportError::ArtifactCacheDeserialize)?;
    let asset = cache_asset.into_imported()?;
    if let Some(expected_kind) = expected_kind {
        let actual_kind = asset_kind_for_imported_asset(&asset);
        if actual_kind != expected_kind {
            return Err(AssetImportError::Parse(format!(
                "artifact cache kind mismatch for {path}: path is {expected_kind:?}, payload is {actual_kind:?}"
            )));
        }
    }
    Ok(asset)
}

fn asset_kind_from_artifact_path(path: &str) -> Option<AssetKind> {
    [
        ("textures/", AssetKind::Texture),
        ("shaders/", AssetKind::Shader),
        ("data/", AssetKind::Data),
        ("physics/materials/", AssetKind::PhysicsMaterial),
        ("materials/graphs/", AssetKind::MaterialGraph),
        ("materials/", AssetKind::Material),
        ("sound/", AssetKind::Sound),
        ("fonts/", AssetKind::Font),
        ("navigation/navmeshes/", AssetKind::NavMesh),
        ("navigation/settings/", AssetKind::NavigationSettings),
        ("terrain/heightfields/", AssetKind::Terrain),
        ("terrain/layers/", AssetKind::TerrainLayerStack),
        ("tilemap_2d/tilesets/", AssetKind::TileSet),
        ("tilemap_2d/maps/", AssetKind::TileMap),
        ("prefabs/", AssetKind::Prefab),
        ("scenes/", AssetKind::Scene),
        ("meshes/", AssetKind::Mesh),
        ("models/", AssetKind::Model),
        ("animation/skeletons/", AssetKind::AnimationSkeleton),
        ("animation/clips/", AssetKind::AnimationClip),
        ("animation/sequences/", AssetKind::AnimationSequence),
        ("animation/graphs/", AssetKind::AnimationGraph),
        (
            "animation/state_machines/",
            AssetKind::AnimationStateMachine,
        ),
        ("ui/layouts/", AssetKind::UiLayout),
        ("ui/widgets/", AssetKind::UiWidget),
        ("ui/styles/", AssetKind::UiStyle),
    ]
    .into_iter()
    .find_map(|(prefix, kind)| path.starts_with(prefix).then_some(kind))
}

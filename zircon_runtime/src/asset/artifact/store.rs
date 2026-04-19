use std::fs;
use std::path::PathBuf;

use crate::core::resource::{ResourceRecord, ResourceScheme};

use crate::asset::project::ProjectPaths;
use crate::asset::{AssetImportError, AssetKind, AssetUri, ImportedAsset};

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
            artifact_extension(metadata.kind)
        );
        let artifact_uri = AssetUri::parse(&format!("lib://{relative_path}"))?;
        let artifact_path = resolve_library_path(paths, &artifact_uri)?;
        if let Some(parent) = artifact_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&artifact_path, serialize_asset(asset)?)?;
        Ok(artifact_uri)
    }

    pub fn read(
        &self,
        paths: &ProjectPaths,
        artifact_uri: &AssetUri,
    ) -> Result<ImportedAsset, AssetImportError> {
        let artifact_path = resolve_library_path(paths, artifact_uri)?;
        let payload = fs::read(artifact_path)?;
        deserialize_asset(artifact_uri.path(), &payload)
    }
}

fn resolve_library_path(
    paths: &ProjectPaths,
    artifact_uri: &AssetUri,
) -> Result<PathBuf, AssetImportError> {
    if artifact_uri.scheme() != ResourceScheme::Library {
        return Err(AssetImportError::UnsupportedFormat(format!(
            "artifact uri must use lib:// scheme: {artifact_uri}"
        )));
    }
    Ok(paths.library_root().join(artifact_uri.path()))
}

fn asset_kind_directory(kind: AssetKind) -> &'static str {
    match kind {
        AssetKind::Texture => "textures",
        AssetKind::Shader => "shaders",
        AssetKind::Material => "materials",
        AssetKind::PhysicsMaterial => "physics/materials",
        AssetKind::Scene => "scenes",
        AssetKind::Model => "models",
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

fn artifact_extension(kind: AssetKind) -> &'static str {
    match kind {
        AssetKind::AnimationSkeleton
        | AssetKind::AnimationClip
        | AssetKind::AnimationSequence
        | AssetKind::AnimationGraph
        | AssetKind::AnimationStateMachine => "bin",
        _ => "json",
    }
}

fn serialize_asset(asset: &ImportedAsset) -> Result<Vec<u8>, AssetImportError> {
    match asset {
        ImportedAsset::AnimationSkeleton(asset) => asset
            .to_bytes()
            .map_err(AssetImportError::Parse),
        ImportedAsset::AnimationClip(asset) => asset.to_bytes().map_err(AssetImportError::Parse),
        ImportedAsset::AnimationSequence(asset) => {
            asset.to_bytes().map_err(AssetImportError::Parse)
        }
        ImportedAsset::AnimationGraph(asset) => {
            asset.to_bytes().map_err(AssetImportError::Parse)
        }
        ImportedAsset::AnimationStateMachine(asset) => {
            asset.to_bytes().map_err(AssetImportError::Parse)
        }
        _ => serde_json::to_vec_pretty(asset).map_err(AssetImportError::from),
    }
}

fn deserialize_asset(path: &str, payload: &[u8]) -> Result<ImportedAsset, AssetImportError> {
    match asset_kind_from_artifact_path(path) {
        Some(AssetKind::AnimationSkeleton) => {
            crate::asset::AnimationSkeletonAsset::from_bytes(payload)
                .map(ImportedAsset::AnimationSkeleton)
                .map_err(AssetImportError::Parse)
        }
        Some(AssetKind::AnimationClip) => crate::asset::AnimationClipAsset::from_bytes(payload)
            .map(ImportedAsset::AnimationClip)
            .map_err(AssetImportError::Parse),
        Some(AssetKind::AnimationSequence) => {
            crate::asset::AnimationSequenceAsset::from_bytes(payload)
                .map(ImportedAsset::AnimationSequence)
                .map_err(AssetImportError::Parse)
        }
        Some(AssetKind::AnimationGraph) => crate::asset::AnimationGraphAsset::from_bytes(payload)
            .map(ImportedAsset::AnimationGraph)
            .map_err(AssetImportError::Parse),
        Some(AssetKind::AnimationStateMachine) => {
            crate::asset::AnimationStateMachineAsset::from_bytes(payload)
                .map(ImportedAsset::AnimationStateMachine)
                .map_err(AssetImportError::Parse)
        }
        _ => serde_json::from_slice(payload).map_err(AssetImportError::from),
    }
}

fn asset_kind_from_artifact_path(path: &str) -> Option<AssetKind> {
    [
        ("textures/", AssetKind::Texture),
        ("shaders/", AssetKind::Shader),
        ("physics/materials/", AssetKind::PhysicsMaterial),
        ("materials/", AssetKind::Material),
        ("scenes/", AssetKind::Scene),
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

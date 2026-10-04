//! 模型项目文档将资源引用编码成可迁移的项目引用；解析后仍由模型资产的依赖收集建立加载关系。

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::project::PersistedAssetReference;

use crate::asset::{AssetReference, ModelAsset, ReferenceResolutionError};

use super::codec::{decode_document, encode_document, ProjectDocumentArtifact};
use crate::asset::assets::ProjectDocumentError;

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct ModelAuthoringDocument<R> {
    primitives: Vec<ModelPrimitiveDocument<R>>,
    #[serde(flatten)]
    _rest: toml::Table,
}

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct ModelPrimitiveDocument<R> {
    #[serde(default)]
    mesh: Option<R>,
    #[serde(flatten)]
    _rest: toml::Table,
}

pub(in crate::asset::assets) fn deserialize_model(
    document: &str,
    resolver: impl FnMut(&PersistedAssetReference) -> Result<AssetReference, ReferenceResolutionError>,
) -> Result<ModelAsset, ProjectDocumentError> {
    deserialize_model_artifact(ProjectDocumentArtifact::parse(document)?, resolver)
}

pub(in crate::asset) fn deserialize_model_artifact(
    document: ProjectDocumentArtifact,
    mut resolver: impl FnMut(
        &PersistedAssetReference,
    ) -> Result<AssetReference, ReferenceResolutionError>,
) -> Result<ModelAsset, ProjectDocumentError> {
    let document = document.into_document::<ModelAuthoringDocument<PersistedAssetReference>>()?;
    let document = map_references(document, |reference| resolver(&reference))?;
    decode_document(document)
}

pub(in crate::asset::assets) fn serialize_model(
    value: &ModelAsset,
    mut resolver: impl FnMut(
        &AssetReference,
    ) -> Result<PersistedAssetReference, ReferenceResolutionError>,
) -> Result<String, ProjectDocumentError> {
    let document = encode_document::<_, ModelAuthoringDocument<AssetReference>>(value)?;
    let document = map_references(document, |reference| resolver(&reference))?;
    Ok(toml::to_string_pretty(&document)?)
}

fn map_references<A, B>(
    document: ModelAuthoringDocument<A>,
    mut map: impl FnMut(A) -> Result<B, ReferenceResolutionError>,
) -> Result<ModelAuthoringDocument<B>, ReferenceResolutionError> {
    Ok(ModelAuthoringDocument {
        primitives: document
            .primitives
            .into_iter()
            .map(|primitive| {
                Ok(ModelPrimitiveDocument {
                    mesh: primitive.mesh.map(&mut map).transpose()?,
                    _rest: primitive._rest,
                })
            })
            .collect::<Result<_, ReferenceResolutionError>>()?,
        _rest: document._rest,
    })
}

#[cfg(test)]
#[path = "tests/model.rs"]
mod tests;

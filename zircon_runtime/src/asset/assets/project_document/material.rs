//! 材质项目文档保留用户编辑的 shader、纹理槽和属性覆写；导入与写回都经过该边界，不能将运行时生成值误写为作者值。

use std::collections::BTreeMap;

use serde::de::{DeserializeOwned, Error as _};
use serde::ser::{Error as _, SerializeMap};
use serde::{Deserialize, Serialize};
use zircon_runtime_interface::project::PersistedAssetReference;

use crate::asset::{AssetReference, ReferenceResolutionError, ZMaterialDocument};

use super::codec::{decode_document, encode_document, ProjectDocumentArtifact};
use crate::asset::assets::material::validate_zmaterial_version;
use crate::asset::assets::ProjectDocumentError;

#[derive(Deserialize, Serialize)]
#[serde(bound(
    serialize = "R: FlattenedMaterialReference",
    deserialize = "R: FlattenedMaterialReference"
))]
struct MaterialAuthoringDocument<R> {
    version: u32,
    shader: R,
    #[serde(default)]
    parent: Option<R>,
    #[serde(default)]
    textures: BTreeMap<String, MaterialTextureDocument<R>>,
    #[serde(flatten)]
    _rest: toml::Table,
}

struct MaterialTextureDocument<R> {
    reference: Option<R>,
    _rest: toml::Table,
}

trait FlattenedMaterialReference: Serialize + DeserializeOwned {
    fn take_fields(fields: &mut toml::Table) -> Option<toml::Table>;
}

impl FlattenedMaterialReference for AssetReference {
    fn take_fields(fields: &mut toml::Table) -> Option<toml::Table> {
        take_named_fields(fields, &["uuid", "url"])
    }
}

impl FlattenedMaterialReference for PersistedAssetReference {
    fn take_fields(fields: &mut toml::Table) -> Option<toml::Table> {
        take_named_fields(fields, &["kind", "guid", "path_hint", "sub", "locator"])
    }
}

impl<R> Serialize for MaterialTextureDocument<R>
where
    R: FlattenedMaterialReference,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reference = self
            .reference
            .as_ref()
            .map(toml::Value::try_from)
            .transpose()
            .map_err(S::Error::custom)?;
        let reference = match reference {
            Some(toml::Value::Table(fields)) => Some(fields),
            Some(_) => {
                return Err(S::Error::custom(
                    "material texture reference must serialize as a table",
                ));
            }
            None => None,
        };
        let reference_len = reference.as_ref().map_or(0, toml::Table::len);
        let mut map = serializer.serialize_map(Some(reference_len + self._rest.len()))?;
        if let Some(reference) = &reference {
            for (name, value) in reference {
                map.serialize_entry(name, value)?;
            }
        }
        for (name, value) in &self._rest {
            map.serialize_entry(name, value)?;
        }
        map.end()
    }
}

impl<'de, R> Deserialize<'de> for MaterialTextureDocument<R>
where
    R: FlattenedMaterialReference,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut fields = toml::Table::deserialize(deserializer)?;
        let reference = R::take_fields(&mut fields)
            .map(|reference| {
                let parsed: Result<R, toml::de::Error> = toml::Value::Table(reference).try_into();
                parsed.map_err(D::Error::custom)
            })
            .transpose()?;
        Ok(Self {
            reference,
            _rest: fields,
        })
    }
}

fn take_named_fields(fields: &mut toml::Table, names: &[&str]) -> Option<toml::Table> {
    let mut reference = toml::Table::new();
    for name in names {
        if let Some(value) = fields.remove(*name) {
            reference.insert((*name).to_owned(), value);
        }
    }
    (!reference.is_empty()).then_some(reference)
}

pub(in crate::asset::assets) fn deserialize_material(
    document: &str,
    resolver: impl FnMut(&PersistedAssetReference) -> Result<AssetReference, ReferenceResolutionError>,
) -> Result<ZMaterialDocument, ProjectDocumentError> {
    deserialize_material_artifact(ProjectDocumentArtifact::parse(document)?, resolver)
}

pub(in crate::asset) fn deserialize_material_artifact(
    document: ProjectDocumentArtifact,
    mut resolver: impl FnMut(
        &PersistedAssetReference,
    ) -> Result<AssetReference, ReferenceResolutionError>,
) -> Result<ZMaterialDocument, ProjectDocumentError> {
    let document =
        document.into_document::<MaterialAuthoringDocument<PersistedAssetReference>>()?;
    let document = map_references(document, |reference| resolver(&reference))?;
    let material: ZMaterialDocument = decode_document(document)?;
    validate_zmaterial_version(material.version)?;
    Ok(material)
}

pub(in crate::asset::assets) fn serialize_material(
    value: &ZMaterialDocument,
    mut resolver: impl FnMut(
        &AssetReference,
    ) -> Result<PersistedAssetReference, ReferenceResolutionError>,
) -> Result<String, ProjectDocumentError> {
    validate_zmaterial_version(value.version)?;
    let document = encode_document::<_, MaterialAuthoringDocument<AssetReference>>(value)?;
    let document = map_references(document, |reference| resolver(&reference))?;
    Ok(toml::to_string_pretty(&document)?)
}

fn map_references<A, B>(
    document: MaterialAuthoringDocument<A>,
    mut map: impl FnMut(A) -> Result<B, ReferenceResolutionError>,
) -> Result<MaterialAuthoringDocument<B>, ReferenceResolutionError> {
    Ok(MaterialAuthoringDocument {
        version: document.version,
        shader: map(document.shader)?,
        parent: document.parent.map(&mut map).transpose()?,
        textures: document
            .textures
            .into_iter()
            .map(|(name, texture)| {
                Ok((
                    name,
                    MaterialTextureDocument {
                        reference: texture.reference.map(&mut map).transpose()?,
                        _rest: texture._rest,
                    },
                ))
            })
            .collect::<Result<_, ReferenceResolutionError>>()?,
        _rest: document._rest,
    })
}

#[cfg(test)]
#[path = "material/tests/single_pass_field_tests.rs"]
mod single_pass_field_tests;

#[cfg(test)]
#[path = "tests/material.rs"]
mod tests;

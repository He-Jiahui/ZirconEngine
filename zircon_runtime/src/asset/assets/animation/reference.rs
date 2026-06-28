use serde::{Deserialize, Serialize};

use super::error::AnimationAssetError;
use crate::asset::{AssetReference, AssetUri, AssetUuid};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct AnimationAssetReferenceBinary {
    uuid: String,
    url: String,
}

impl From<&AssetReference> for AnimationAssetReferenceBinary {
    fn from(value: &AssetReference) -> Self {
        Self {
            uuid: value.uuid.to_string(),
            url: value.locator.to_string(),
        }
    }
}

impl TryFrom<AnimationAssetReferenceBinary> for AssetReference {
    type Error = AnimationAssetError;

    fn try_from(value: AnimationAssetReferenceBinary) -> Result<Self, Self::Error> {
        let uuid = value.uuid.parse::<AssetUuid>().map_err(|source| {
            AnimationAssetError::InvalidReferenceUuid {
                value: value.uuid.clone(),
                source,
            }
        })?;
        let locator = AssetUri::parse(&value.url).map_err(|source| {
            AnimationAssetError::InvalidReferenceLocator {
                value: value.url.clone(),
                source,
            }
        })?;
        Ok(AssetReference::new(uuid, locator))
    }
}

pub(super) fn push_unique_reference(
    references: &mut Vec<AssetReference>,
    reference: AssetReference,
) {
    if !references
        .iter()
        .any(|existing| existing.uuid == reference.uuid && existing.locator == reference.locator)
    {
        references.push(reference);
    }
}

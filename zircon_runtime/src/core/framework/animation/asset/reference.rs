use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::error::AnimationAssetError;
use crate::core::resource::{AssetReference, AssetUuid, ResourceLocator};

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
        let locator = ResourceLocator::parse(&value.url).map_err(|source| {
            AnimationAssetError::InvalidReferenceLocator {
                value: value.url.clone(),
                source,
            }
        })?;
        Ok(AssetReference::new(uuid, locator))
    }
}

pub(super) struct DirectReferenceCollector<'a> {
    seen: HashSet<&'a AssetReference>,
    references: Vec<AssetReference>,
}

impl<'a> DirectReferenceCollector<'a> {
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            seen: HashSet::with_capacity(capacity),
            references: Vec::with_capacity(capacity),
        }
    }

    pub(super) fn push(&mut self, reference: &'a AssetReference) {
        if self.seen.insert(reference) {
            self.references.push(reference.clone());
        }
    }

    pub(super) fn into_references(self) -> Vec<AssetReference> {
        self.references
    }
}

#[cfg(test)]
#[path = "reference/borrowed_dedup_tests.rs"]
mod borrowed_dedup_tests;

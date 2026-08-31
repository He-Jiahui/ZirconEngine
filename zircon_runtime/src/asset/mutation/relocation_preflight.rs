use std::collections::HashSet;

use crate::asset::registry::AssetRegistryIndex;
use crate::asset::{AssetUri, AssetUuid};
use crate::core::resource::ResourceScheme;

use super::AssetMutationAsset;

/// Topology result for a root-source rename or move request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetMutationRelocationDisposition {
    Ready,
    MissingAsset,
    UnsupportedSubasset,
    UnsupportedSource,
    UnsupportedTarget,
    TargetOccupied,
    NoChange,
}

/// Immutable relocation input assembled from one Runtime registry generation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetMutationRelocationPreflight {
    disposition: AssetMutationRelocationDisposition,
    source: Option<AssetMutationAsset>,
    target: AssetUri,
    target_occupant: Option<AssetMutationAsset>,
    companions: Vec<AssetMutationAsset>,
    referencer_closure: Vec<AssetMutationAsset>,
}

impl AssetMutationRelocationPreflight {
    /// Preflights a root-source relocation without filesystem access or registry mutation.
    ///
    /// The MVP mutation path accepts only `res://` root sources and destinations. All labeled
    /// entries sharing the root source are companions and move together; only referencers outside
    /// that companion set enter the external referencer closure.
    pub fn evaluate(
        registry: &AssetRegistryIndex,
        source_uuid: AssetUuid,
        target: AssetUri,
    ) -> Self {
        let Some(source_entry) = registry.entry_by_uuid(source_uuid) else {
            return Self::without_topology(
                AssetMutationRelocationDisposition::MissingAsset,
                None,
                target,
                None,
            );
        };
        let source = AssetMutationAsset::from(source_entry);
        if source.locator().label().is_some() {
            return Self::without_topology(
                AssetMutationRelocationDisposition::UnsupportedSubasset,
                Some(source),
                target,
                None,
            );
        }
        if source.locator().scheme() != ResourceScheme::Res {
            return Self::without_topology(
                AssetMutationRelocationDisposition::UnsupportedSource,
                Some(source),
                target,
                None,
            );
        }
        if target.scheme() != ResourceScheme::Res || target.label().is_some() {
            return Self::without_topology(
                AssetMutationRelocationDisposition::UnsupportedTarget,
                Some(source),
                target,
                None,
            );
        }

        let (companions, referencer_closure) = Self::collect_topology(registry, source.locator());
        if target == *source.locator() {
            return Self {
                disposition: AssetMutationRelocationDisposition::NoChange,
                source: Some(source),
                target,
                target_occupant: None,
                companions,
                referencer_closure,
            };
        }
        if let Some(occupant) = registry.entry_by_path(&target) {
            return Self {
                disposition: AssetMutationRelocationDisposition::TargetOccupied,
                source: Some(source),
                target,
                target_occupant: Some(AssetMutationAsset::from(occupant)),
                companions,
                referencer_closure,
            };
        }
        Self {
            disposition: AssetMutationRelocationDisposition::Ready,
            source: Some(source),
            target,
            target_occupant: None,
            companions,
            referencer_closure,
        }
    }

    fn without_topology(
        disposition: AssetMutationRelocationDisposition,
        source: Option<AssetMutationAsset>,
        target: AssetUri,
        target_occupant: Option<AssetMutationAsset>,
    ) -> Self {
        Self {
            disposition,
            source,
            target,
            target_occupant,
            companions: Vec::new(),
            referencer_closure: Vec::new(),
        }
    }

    fn collect_topology(
        registry: &AssetRegistryIndex,
        source: &AssetUri,
    ) -> (Vec<AssetMutationAsset>, Vec<AssetMutationAsset>) {
        let mut companions = registry
            .source_entries(source)
            .iter()
            .map(AssetMutationAsset::from)
            .collect::<Vec<_>>();
        companions.sort_by(asset_order);
        let companion_uuids = companions
            .iter()
            .map(AssetMutationAsset::uuid)
            .collect::<HashSet<_>>();
        let mut referencer_uuids = companions
            .iter()
            .flat_map(|companion| registry.get_referencers_by_uuid(companion.uuid()))
            .filter(|uuid| !companion_uuids.contains(uuid))
            .collect::<HashSet<_>>();
        let mut referencer_closure = referencer_uuids
            .drain()
            .filter_map(|uuid| registry.entry_by_uuid(uuid))
            .map(AssetMutationAsset::from)
            .collect::<Vec<_>>();
        referencer_closure.sort_by(asset_order);
        (companions, referencer_closure)
    }

    pub fn disposition(&self) -> AssetMutationRelocationDisposition {
        self.disposition
    }

    pub fn source(&self) -> Option<&AssetMutationAsset> {
        self.source.as_ref()
    }

    pub fn target(&self) -> &AssetUri {
        &self.target
    }

    pub fn target_occupant(&self) -> Option<&AssetMutationAsset> {
        self.target_occupant.as_ref()
    }

    /// All root and labeled entries that must retain identity together during relocation.
    pub fn companions(&self) -> &[AssetMutationAsset] {
        &self.companions
    }

    /// Direct referencers outside the companion set, in canonical order.
    pub fn referencer_closure(&self) -> &[AssetMutationAsset] {
        &self.referencer_closure
    }
}

fn asset_order(left: &AssetMutationAsset, right: &AssetMutationAsset) -> std::cmp::Ordering {
    left.locator()
        .cmp(right.locator())
        .then_with(|| left.uuid().to_string().cmp(&right.uuid().to_string()))
}

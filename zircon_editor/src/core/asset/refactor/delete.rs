use zircon_runtime::asset::{
    AssetMutationAsset, AssetMutationDeleteDisposition, AssetMutationDeletePreflight,
    AssetRegistryIndex, AssetUuid,
};

use super::super::{AssetSourceAuthority, AssetSourceWritePolicy};

/// Editor admission result for a source-asset delete command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetDeleteDisposition {
    Allowed,
    MissingAsset,
    UnsupportedSubasset,
    ReadOnlySource,
    BlockedByReferencers,
}

/// Editor write-policy projection over the runtime-owned delete topology preflight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetDeletePreflight {
    disposition: AssetDeleteDisposition,
    target: Option<AssetMutationAsset>,
    referencers: Vec<AssetMutationAsset>,
}

impl AssetDeletePreflight {
    /// Evaluates write policy over immutable runtime registry topology.
    ///
    /// The later Runtime transaction re-evaluates topology after acquiring its commit generation.
    pub fn evaluate(
        registry: &AssetRegistryIndex,
        target_uuid: AssetUuid,
        write_policy: AssetSourceWritePolicy,
    ) -> Self {
        let topology = AssetMutationDeletePreflight::evaluate(registry, target_uuid);
        let blocked_by_referencers =
            topology.disposition() == AssetMutationDeleteDisposition::BlockedByReferencers;
        let (topology_disposition, target, referencers) = topology.into_parts();
        let disposition = match topology_disposition {
            AssetMutationDeleteDisposition::MissingAsset => AssetDeleteDisposition::MissingAsset,
            AssetMutationDeleteDisposition::UnsupportedSubasset => {
                AssetDeleteDisposition::UnsupportedSubasset
            }
            AssetMutationDeleteDisposition::Ready
            | AssetMutationDeleteDisposition::BlockedByReferencers => {
                let writable = target.as_ref().is_some_and(|asset| {
                    AssetSourceAuthority::from_locator(write_policy, asset.locator()).is_writable()
                });
                if !writable {
                    AssetDeleteDisposition::ReadOnlySource
                } else if blocked_by_referencers {
                    AssetDeleteDisposition::BlockedByReferencers
                } else {
                    AssetDeleteDisposition::Allowed
                }
            }
        };
        Self {
            disposition,
            target,
            referencers,
        }
    }

    pub fn disposition(&self) -> AssetDeleteDisposition {
        self.disposition
    }

    pub fn target(&self) -> Option<&AssetMutationAsset> {
        self.target.as_ref()
    }

    pub fn referencers(&self) -> &[AssetMutationAsset] {
        &self.referencers
    }
}

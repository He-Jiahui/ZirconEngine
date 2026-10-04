use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock, Weak};

use crate::text::font::{
    shared_font_collection_service, FontCollectionService, RuntimeFontAssetClaimScope,
};

/// Keeps installed engine font resources alive for an Editor or standalone UI host.
/// Core-owned Runtime surfaces use their context-bound project asset admissions.
#[derive(Debug)]
pub struct UiHostFontAssets {
    _residents: Vec<Arc<ResidentHostFontAsset>>,
    ready: Vec<UiHostFontAssetReady>,
}

#[derive(Debug)]
struct ResidentHostFontAsset {
    _claims: RuntimeFontAssetClaimScope,
    ready: UiHostFontAssetReady,
}

#[derive(Default)]
struct HostFontAssetRegistry {
    residents: Mutex<BTreeMap<String, Weak<ResidentHostFontAsset>>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiHostFontAssetReady {
    pub asset_ref: String,
    pub family: Option<String>,
    pub registered_face_count: usize,
}

#[derive(Debug, thiserror::Error)]
#[error("UI host font {asset_ref} is unavailable: {reason}")]
pub struct UiHostFontAssetError {
    pub asset_ref: String,
    pub reason: String,
}

impl UiHostFontAssets {
    /// Loads manifests through the runtime asset resolver before host text is measured.
    /// The last owner releases the fonts; no project font collection is modified.
    pub fn load_runtime(asset_refs: &[&str]) -> Result<Self, UiHostFontAssetError> {
        static REGISTRY: OnceLock<HostFontAssetRegistry> = OnceLock::new();
        REGISTRY
            .get_or_init(HostFontAssetRegistry::default)
            .load(shared_font_collection_service(), asset_refs)
    }

    pub fn ready(&self) -> &[UiHostFontAssetReady] {
        &self.ready
    }
}

impl HostFontAssetRegistry {
    fn load(
        &self,
        collection: Arc<FontCollectionService>,
        asset_refs: &[&str],
    ) -> Result<UiHostFontAssets, UiHostFontAssetError> {
        let mut asset_refs = asset_refs.to_vec();
        asset_refs.sort_unstable();
        asset_refs.dedup();
        // Serialize first admission with lookup: another window reuses the resident
        // bytes and cannot retire them by attempting to reopen an unavailable file.
        let mut registry = self
            .residents
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        registry.retain(|_, resident| resident.strong_count() != 0);
        let mut residents = Vec::with_capacity(asset_refs.len());
        for asset_ref in asset_refs {
            let resident = match registry.get(asset_ref).and_then(Weak::upgrade) {
                Some(resident) => resident,
                None => {
                    let asset_ref_arc = Arc::<str>::from(asset_ref);
                    let mut claims = collection.runtime_font_asset_claim_scope();
                    let transition = claims.load_runtime_assets(&[Arc::clone(&asset_ref_arc)]);
                    let outcome = transition
                        .admissions
                        .into_iter()
                        .next()
                        .expect("one requested font admission");
                    let report = outcome.result.map_err(|error| UiHostFontAssetError {
                        asset_ref: asset_ref.to_string(),
                        reason: error.to_string(),
                    })?;
                    let resident = Arc::new(ResidentHostFontAsset {
                        _claims: claims,
                        ready: UiHostFontAssetReady {
                            asset_ref: asset_ref.to_owned(),
                            family: report.family,
                            registered_face_count: report.registered_face_count,
                        },
                    });
                    registry.insert(asset_ref.to_owned(), Arc::downgrade(&resident));
                    resident
                }
            };
            residents.push(resident);
        }
        Ok(UiHostFontAssets {
            ready: residents
                .iter()
                .map(|resident| resident.ready.clone())
                .collect(),
            _residents: residents,
        })
    }
}

#[cfg(test)]
#[path = "tests/host_font_assets.rs"]
mod tests;

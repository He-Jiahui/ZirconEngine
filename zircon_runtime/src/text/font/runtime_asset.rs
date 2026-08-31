use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use zircon_runtime_interface::ui::surface::UiTextRenderMode;

use super::{
    DEFAULT_UI_FONT_ASSET, FontAssetUpdateReport, FontCollectionService, FontDatabaseError,
    FontLoadError, LoadedTextFontSource, load_text_font_source,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RuntimeFontAssetAdmissionReport {
    pub(crate) font_inputs_changed: bool,
    pub(crate) registered_face_count: usize,
    pub(crate) family: Option<String>,
    pub(crate) render_mode: Option<UiTextRenderMode>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RuntimeFontAssetAdmissionError {
    #[error(transparent)]
    Source(#[from] FontLoadError),
    #[error(transparent)]
    Database(#[from] FontDatabaseError),
    #[error("font asset registered no usable faces")]
    NoRegisteredFaces,
}

#[derive(Debug, Default)]
pub(super) struct RuntimeFontAssetClaimRegistry {
    claim_counts: HashMap<Arc<str>, usize>,
}

/// A non-cloneable lease over the runtime font assets used by one text consumer.
///
/// The collection owns the aggregate claim counts. Dropping the last scope that
/// names an asset retires its database owner and publishes one collection
/// generation for the complete release set.
#[derive(Debug)]
pub(crate) struct RuntimeFontAssetClaimScope {
    collection: Arc<FontCollectionService>,
    claimed_assets: HashSet<Arc<str>>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RuntimeFontAssetClaimUpdateReport {
    pub(crate) active_claim_count: usize,
    pub(crate) added_claim_count: usize,
    pub(crate) released_claim_count: usize,
    pub(crate) unclaimed_asset_count: usize,
    pub(crate) font_inputs_changed: bool,
}

#[derive(Debug)]
pub(crate) struct PreparedRuntimeFontAssetAdmission {
    asset_ref: Arc<str>,
    source: Result<LoadedTextFontSource, FontLoadError>,
}

#[derive(Debug)]
pub(crate) struct RuntimeFontAssetAdmissionOutcome {
    pub(crate) asset_ref: Arc<str>,
    pub(crate) result: Result<RuntimeFontAssetAdmissionReport, RuntimeFontAssetAdmissionError>,
}

#[derive(Debug, Default)]
pub(crate) struct RuntimeFontAssetTransitionReport {
    pub(crate) claims: RuntimeFontAssetClaimUpdateReport,
    pub(crate) admissions: Vec<RuntimeFontAssetAdmissionOutcome>,
}

impl FontCollectionService {
    pub(crate) fn runtime_font_asset_claim_scope(self: &Arc<Self>) -> RuntimeFontAssetClaimScope {
        RuntimeFontAssetClaimScope {
            collection: Arc::clone(self),
            claimed_assets: HashSet::new(),
        }
    }

    fn replace_runtime_font_asset_claims(
        self: &Arc<Self>,
        added: &[Arc<str>],
        released: &[Arc<str>],
    ) -> RuntimeFontAssetClaimUpdateReport {
        self.replace_runtime_font_asset_claims_with_admissions(added, released, Vec::new())
            .claims
    }

    fn replace_runtime_font_asset_claims_with_admissions(
        self: &Arc<Self>,
        added: &[Arc<str>],
        released: &[Arc<str>],
        admissions: Vec<PreparedRuntimeFontAssetAdmission>,
    ) -> RuntimeFontAssetTransitionReport {
        let mut registry = self
            .runtime_asset_claims
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for asset_ref in added {
            let count = registry
                .claim_counts
                .entry(Arc::clone(asset_ref))
                .or_default();
            *count = count.saturating_add(1);
        }

        let mut unclaimed = Vec::new();
        for asset_ref in released {
            if let std::collections::hash_map::Entry::Occupied(mut claim) =
                registry.claim_counts.entry(Arc::clone(asset_ref))
            {
                let remaining = claim.get().saturating_sub(1);
                *claim.get_mut() = remaining;
                if remaining == 0 {
                    claim.remove();
                    unclaimed.push(Arc::clone(asset_ref));
                }
            }
        }

        // Keep the claim ledger locked through the collection mutation so a concurrent
        // scope cannot observe a half-applied release/admission transition.
        let (font_inputs_changed, admission_outcomes) = if unclaimed.is_empty()
            && admissions.is_empty()
        {
            (false, Vec::new())
        } else {
            let (_, (font_inputs_changed, admission_outcomes)) =
                self.mutate_published_snapshot(|database| {
                    let release_report = retire_runtime_font_assets_from_database(
                        database,
                        unclaimed.iter().map(|asset_ref| asset_ref.as_ref()),
                    );
                    let mut font_inputs_changed =
                        release_report.database_changed || release_report.asset_mapping_changed;
                    let admission_outcomes = admissions
                        .into_iter()
                        .map(|admission| {
                            let (outcome, changed) =
                                apply_prepared_runtime_font_asset_admission(database, admission);
                            font_inputs_changed |= changed;
                            outcome
                        })
                        .collect();
                    (font_inputs_changed, admission_outcomes)
                });
            (font_inputs_changed, admission_outcomes)
        };
        let active_asset_count = registry.claim_counts.len();
        drop(registry);
        let report = RuntimeFontAssetClaimUpdateReport {
            active_claim_count: active_asset_count,
            added_claim_count: added.len(),
            released_claim_count: released.len(),
            unclaimed_asset_count: unclaimed.len(),
            font_inputs_changed,
        };
        crate::profile_counter!(
            "runtime",
            "text.font_asset_claim.active_asset_count",
            report.active_claim_count
        );
        crate::profile_counter!(
            "runtime",
            "text.font_asset_claim.unclaimed_asset_count",
            report.unclaimed_asset_count
        );
        RuntimeFontAssetTransitionReport {
            claims: report,
            admissions: admission_outcomes,
        }
    }
}

impl RuntimeFontAssetClaimScope {
    pub(crate) fn replace_claims<I, S>(
        &mut self,
        asset_refs: I,
    ) -> RuntimeFontAssetClaimUpdateReport
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let next = asset_refs
            .into_iter()
            .map(|asset_ref| Arc::<str>::from(asset_ref.as_ref()))
            .collect::<HashSet<_>>();
        self.replace_claim_set(next)
    }

    pub(crate) fn replace_shared_claims(
        &mut self,
        asset_refs: &[Arc<str>],
    ) -> RuntimeFontAssetClaimUpdateReport {
        if self.claimed_assets.len() == asset_refs.len()
            && asset_refs
                .iter()
                .all(|asset_ref| self.claimed_assets.contains(asset_ref.as_ref()))
        {
            return RuntimeFontAssetClaimUpdateReport {
                active_claim_count: self.claimed_assets.len(),
                ..RuntimeFontAssetClaimUpdateReport::default()
            };
        }
        self.replace_claim_set(asset_refs.iter().cloned().collect())
    }

    pub(crate) fn replace_shared_claims_with_admissions(
        &mut self,
        asset_refs: &[Arc<str>],
        admissions: Vec<PreparedRuntimeFontAssetAdmission>,
    ) -> RuntimeFontAssetTransitionReport {
        if admissions.is_empty() {
            return RuntimeFontAssetTransitionReport {
                claims: self.replace_shared_claims(asset_refs),
                admissions: Vec::new(),
            };
        }
        self.replace_claim_set_with_admissions(asset_refs.iter().cloned().collect(), admissions)
    }

    fn replace_claim_set(&mut self, next: HashSet<Arc<str>>) -> RuntimeFontAssetClaimUpdateReport {
        if self.claimed_assets == next {
            return RuntimeFontAssetClaimUpdateReport {
                active_claim_count: self.claimed_assets.len(),
                ..RuntimeFontAssetClaimUpdateReport::default()
            };
        }

        let added = next
            .difference(&self.claimed_assets)
            .cloned()
            .collect::<Vec<_>>();
        let released = self
            .claimed_assets
            .difference(&next)
            .cloned()
            .collect::<Vec<_>>();
        let mut report = self
            .collection
            .replace_runtime_font_asset_claims(&added, &released);
        self.claimed_assets = next;
        report.active_claim_count = self.claimed_assets.len();
        report
    }

    fn replace_claim_set_with_admissions(
        &mut self,
        next: HashSet<Arc<str>>,
        admissions: Vec<PreparedRuntimeFontAssetAdmission>,
    ) -> RuntimeFontAssetTransitionReport {
        let added = next
            .difference(&self.claimed_assets)
            .cloned()
            .collect::<Vec<_>>();
        let released = self
            .claimed_assets
            .difference(&next)
            .cloned()
            .collect::<Vec<_>>();
        let mut report = self
            .collection
            .replace_runtime_font_asset_claims_with_admissions(&added, &released, admissions);
        self.claimed_assets = next;
        report.claims.active_claim_count = self.claimed_assets.len();
        report
    }

    fn release_all(&mut self) {
        if self.claimed_assets.is_empty() {
            return;
        }
        let released = std::mem::take(&mut self.claimed_assets)
            .into_iter()
            .collect::<Vec<_>>();
        let _ = self
            .collection
            .replace_runtime_font_asset_claims(&[], &released);
    }
}

impl Drop for RuntimeFontAssetClaimScope {
    fn drop(&mut self) {
        self.release_all();
    }
}

pub(crate) fn prepare_runtime_font_asset_admission(
    asset_manager: &ProjectAssetManager,
    asset_ref: Arc<str>,
) -> PreparedRuntimeFontAssetAdmission {
    let source = load_text_font_source(&asset_ref, Some(asset_manager));
    PreparedRuntimeFontAssetAdmission { asset_ref, source }
}

fn apply_prepared_runtime_font_asset_admission(
    database: &mut super::FontDatabase,
    prepared: PreparedRuntimeFontAssetAdmission,
) -> (RuntimeFontAssetAdmissionOutcome, bool) {
    let asset_ref = prepared.asset_ref;
    let result = match prepared.source {
        Ok(source) => apply_loaded_runtime_font_asset(database, &asset_ref, source),
        Err(error) => Err(RuntimeFontAssetAdmissionError::Source(error)),
    };
    let mut font_inputs_changed = result
        .as_ref()
        .is_ok_and(|report| report.font_inputs_changed);
    if result.is_err() {
        let report = retire_runtime_font_assets_from_database(database, [asset_ref.as_ref()]);
        font_inputs_changed |= report.database_changed || report.asset_mapping_changed;
    }
    (
        RuntimeFontAssetAdmissionOutcome { asset_ref, result },
        font_inputs_changed,
    )
}

fn apply_loaded_runtime_font_asset(
    database: &mut super::FontDatabase,
    asset_ref: &str,
    source: LoadedTextFontSource,
) -> Result<RuntimeFontAssetAdmissionReport, RuntimeFontAssetAdmissionError> {
    let project_composite = source
        .asset
        .as_ref()
        .and_then(|asset| asset.composite_font.clone());
    let default_family = source.family.clone();
    let render_mode = source
        .asset
        .as_ref()
        .and_then(|asset| asset.effective_render_mode());
    let report = match (source.asset.as_ref(), source.cooked_blob.as_ref()) {
        (Some(asset), Some(blob)) => {
            database.replace_font_asset_blob(asset_ref, asset, &source.source_path, blob)
        }
        (Some(asset), None) => database.replace_font_asset(asset_ref, asset, &source.source_path),
        (None, _) => database.replace_font_source(
            asset_ref,
            &source.source_path,
            source.family.as_deref(),
            source.face_index,
        ),
    }?;
    if report.faces.is_empty() {
        return Err(RuntimeFontAssetAdmissionError::NoRegisteredFaces);
    }

    let projection_changed = if asset_ref == DEFAULT_UI_FONT_ASSET {
        let composite_changed = database.set_project_composite_font(project_composite);
        let family_changed = if let Some(family) = default_family.as_deref() {
            database.set_default_ui_family(family)
        } else {
            database.clear_default_ui_family()
        };
        composite_changed || family_changed
    } else {
        false
    };
    Ok(RuntimeFontAssetAdmissionReport {
        font_inputs_changed: report.database_changed
            || report.asset_mapping_changed
            || projection_changed,
        registered_face_count: report.faces.len(),
        family: source.family,
        render_mode,
    })
}

fn retire_runtime_font_assets_from_database<'a>(
    database: &mut super::FontDatabase,
    asset_refs: impl IntoIterator<Item = &'a str>,
) -> FontAssetUpdateReport {
    let mut aggregate = FontAssetUpdateReport::default();
    for asset_ref in asset_refs {
        let report = database.remove_font_asset(asset_ref);
        aggregate.faces.extend(report.faces);
        aggregate.retired_faces.extend(report.retired_faces);
        aggregate.database_changed |= report.database_changed;
        aggregate.asset_mapping_changed |= report.asset_mapping_changed;
        if asset_ref == DEFAULT_UI_FONT_ASSET {
            let composite_changed = database.set_project_composite_font(None);
            let family_changed = database.clear_default_ui_family();
            aggregate.database_changed |= composite_changed || family_changed;
        }
    }
    aggregate
}

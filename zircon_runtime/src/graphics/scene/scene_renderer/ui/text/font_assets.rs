use std::collections::HashMap;
use std::sync::Arc;

use zircon_runtime_interface::ui::surface::{UiTextRenderMode, resolve_ui_text_render_mode};

use crate::asset::ProjectAssetManager;
use crate::asset::assets::FontSourceBudgetError;
use crate::core::framework::asset::{ResourceCacheIdentity, ResourceManager as _};
use crate::text::TextRenderState;
use crate::text::font::{
    FontDatabaseError, FontLoadError, FontLoadIoFailure, RuntimeFontAssetAdmissionError,
    RuntimeFontAssetClaimScope, RuntimeFontAssetClaimUpdateReport,
    prepare_runtime_font_asset_admission,
};

#[derive(Clone, Debug, Default)]
pub(super) struct LoadedUiFontAsset {
    pub(super) family: Option<String>,
    pub(super) render_mode: Option<UiTextRenderMode>,
}

#[cfg(test)]
pub(super) struct EnsuredUiFontAsset<'a> {
    pub(super) faces_changed: bool,
    pub(super) record: Option<&'a LoadedUiFontAsset>,
    pub(super) loaded: bool,
    pub(super) cache_hit: bool,
    pub(super) status: UiFontAssetCacheStatus,
    pub(super) failure: Option<&'a UiFontAssetLoadError>,
}

pub(super) type UiFontAssetCache = HashMap<String, UiFontAssetCacheEntry>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct UiFontAssetRefreshReport {
    pub(super) claims: RuntimeFontAssetClaimUpdateReport,
    pub(super) font_collection_changed: bool,
    pub(super) font_records_reloaded: bool,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UiFontAssetCacheStatus {
    Ready,
    Missing,
    Error,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiFontAssetCacheReport {
    pub(crate) ready_count: usize,
    pub(crate) missing_count: usize,
    pub(crate) error_count: usize,
    pub(crate) source_contract_failure_count: usize,
    pub(crate) source_not_found_count: usize,
    pub(crate) source_permission_denied_count: usize,
    pub(crate) source_other_io_failure_count: usize,
    pub(crate) source_decode_failure_count: usize,
    pub(crate) source_budget_failure_count: usize,
    pub(crate) registration_failure_count: usize,
    pub(crate) no_registered_faces_count: usize,
}

#[derive(Debug)]
pub(super) struct UiFontAssetCacheEntry {
    resource_identity: Option<ResourceCacheIdentity>,
    state: UiFontAssetCacheState,
}

#[derive(Debug)]
enum UiFontAssetCacheState {
    Ready(LoadedUiFontAsset),
    Missing,
    Error(UiFontAssetLoadError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UiFontAssetLoadError {
    Source(FontLoadError),
    SourceReadFailed(FontLoadIoFailure),
    SourceDecodeFailed,
    SourceBudgetExceeded(FontSourceBudgetError),
    RegistrationFailed,
    NoRegisteredFaces,
}

impl UiFontAssetCacheReport {
    fn record_failure(&mut self, error: UiFontAssetLoadError) {
        self.error_count = self.error_count.saturating_add(1);
        match error {
            UiFontAssetLoadError::SourceReadFailed(cause)
            | UiFontAssetLoadError::Source(
                FontLoadError::ManifestReadFailed(cause)
                | FontLoadError::AllowedRootUnavailable(cause)
                | FontLoadError::ManifestSourceUnavailable(cause),
            ) => match cause {
                crate::text::font::FontLoadIoFailure::NotFound => {
                    self.source_not_found_count = self.source_not_found_count.saturating_add(1);
                }
                crate::text::font::FontLoadIoFailure::PermissionDenied => {
                    self.source_permission_denied_count =
                        self.source_permission_denied_count.saturating_add(1);
                }
                crate::text::font::FontLoadIoFailure::Other => {
                    self.source_other_io_failure_count =
                        self.source_other_io_failure_count.saturating_add(1);
                }
            },
            UiFontAssetLoadError::Source(_) => {
                self.source_contract_failure_count =
                    self.source_contract_failure_count.saturating_add(1);
            }
            UiFontAssetLoadError::SourceDecodeFailed => {
                self.source_decode_failure_count =
                    self.source_decode_failure_count.saturating_add(1);
            }
            UiFontAssetLoadError::SourceBudgetExceeded(_) => {
                self.source_budget_failure_count =
                    self.source_budget_failure_count.saturating_add(1);
            }
            UiFontAssetLoadError::RegistrationFailed => {
                self.registration_failure_count = self.registration_failure_count.saturating_add(1);
            }
            UiFontAssetLoadError::NoRegisteredFaces => {
                self.no_registered_faces_count = self.no_registered_faces_count.saturating_add(1);
            }
        }
    }
}

impl UiFontAssetLoadError {
    pub(super) fn from_database_error(error: FontDatabaseError) -> Self {
        match error {
            FontDatabaseError::ReadFailed { source, .. } => {
                Self::SourceReadFailed(source.kind().into())
            }
            FontDatabaseError::SourceDecode { .. } => Self::SourceDecodeFailed,
            FontDatabaseError::SourceBudget { source, .. } => Self::SourceBudgetExceeded(source),
            _ => Self::RegistrationFailed,
        }
    }

    fn from_admission_error(error: RuntimeFontAssetAdmissionError) -> Self {
        match error {
            RuntimeFontAssetAdmissionError::Source(error) => Self::Source(error),
            RuntimeFontAssetAdmissionError::Database(error) => Self::from_database_error(error),
            RuntimeFontAssetAdmissionError::NoRegisteredFaces => Self::NoRegisteredFaces,
        }
    }
}

impl UiFontAssetCacheEntry {
    fn new(resource_identity: Option<ResourceCacheIdentity>, state: UiFontAssetCacheState) -> Self {
        Self {
            resource_identity,
            state,
        }
    }

    pub(super) fn loaded_asset(&self) -> Option<&LoadedUiFontAsset> {
        match &self.state {
            UiFontAssetCacheState::Ready(record) => Some(record),
            UiFontAssetCacheState::Missing => None,
            UiFontAssetCacheState::Error(_) => None,
        }
    }

    #[cfg(test)]
    fn status(&self) -> UiFontAssetCacheStatus {
        match &self.state {
            UiFontAssetCacheState::Ready(_) => UiFontAssetCacheStatus::Ready,
            UiFontAssetCacheState::Missing => UiFontAssetCacheStatus::Missing,
            UiFontAssetCacheState::Error(_) => UiFontAssetCacheStatus::Error,
        }
    }

    #[cfg(test)]
    fn failure(&self) -> Option<&UiFontAssetLoadError> {
        match &self.state {
            UiFontAssetCacheState::Error(error) => Some(error),
            UiFontAssetCacheState::Ready(_) | UiFontAssetCacheState::Missing => None,
        }
    }
}

pub(super) fn font_asset_cache_report(font_assets: &UiFontAssetCache) -> UiFontAssetCacheReport {
    let mut report = UiFontAssetCacheReport::default();
    for entry in font_assets.values() {
        match &entry.state {
            UiFontAssetCacheState::Ready(_) => {
                report.ready_count = report.ready_count.saturating_add(1);
            }
            UiFontAssetCacheState::Missing => {
                report.missing_count = report.missing_count.saturating_add(1);
            }
            UiFontAssetCacheState::Error(error) => report.record_failure(*error),
        }
    }
    report
}

#[cfg(test)]
impl<'a> EnsuredUiFontAsset<'a> {
    fn cache_hit(entry: &'a UiFontAssetCacheEntry) -> Self {
        Self {
            faces_changed: false,
            record: entry.loaded_asset(),
            loaded: false,
            cache_hit: true,
            status: entry.status(),
            failure: entry.failure(),
        }
    }

    fn reloaded(entry: &'a UiFontAssetCacheEntry, loaded: bool, faces_changed: bool) -> Self {
        Self {
            faces_changed,
            record: entry.loaded_asset(),
            loaded,
            cache_hit: false,
            status: entry.status(),
            failure: entry.failure(),
        }
    }
}

pub(super) fn effective_text_render_mode(
    requested_mode: UiTextRenderMode,
    font_asset: Option<&LoadedUiFontAsset>,
) -> UiTextRenderMode {
    resolve_ui_text_render_mode(
        requested_mode,
        font_asset.and_then(|asset| asset.render_mode),
    )
}

pub(super) fn refresh_font_asset_records(
    text_state: &mut TextRenderState,
    font_assets: &mut UiFontAssetCache,
    asset_manager: &ProjectAssetManager,
    active_font_dependencies: &[Arc<str>],
    font_claim_scope: &mut RuntimeFontAssetClaimScope,
) -> UiFontAssetRefreshReport {
    let pending = active_font_dependencies
        .iter()
        .filter_map(|asset_ref| {
            let resource_identity = asset_manager.resource_cache_identity(asset_ref);
            let cache_hit = font_assets
                .get(asset_ref.as_ref())
                .is_some_and(|entry| entry.resource_identity == resource_identity);
            (!cache_hit).then(|| (Arc::clone(asset_ref), resource_identity))
        })
        .collect::<Vec<_>>();
    let admissions = pending
        .iter()
        .map(|(asset_ref, _)| {
            prepare_runtime_font_asset_admission(asset_manager, Arc::clone(asset_ref))
        })
        .collect();
    let transition = font_claim_scope
        .replace_shared_claims_with_admissions(active_font_dependencies, admissions);
    if transition.claims.released_claim_count > 0 {
        font_assets.retain(|asset_ref, _| {
            active_font_dependencies
                .iter()
                .any(|active| active.as_ref() == asset_ref)
        });
    }

    let font_records_reloaded = !pending.is_empty();
    debug_assert_eq!(pending.len(), transition.admissions.len());
    for ((asset_ref, resource_identity), outcome) in pending.into_iter().zip(transition.admissions)
    {
        debug_assert_eq!(asset_ref.as_ref(), outcome.asset_ref.as_ref());
        let state = match outcome.result {
            Ok(report) => UiFontAssetCacheState::Ready(LoadedUiFontAsset {
                family: report.family,
                render_mode: report.render_mode,
            }),
            Err(_) if resource_identity.is_none() => UiFontAssetCacheState::Missing,
            Err(error) => {
                UiFontAssetCacheState::Error(UiFontAssetLoadError::from_admission_error(error))
            }
        };
        font_assets.insert(
            asset_ref.to_string(),
            UiFontAssetCacheEntry::new(resource_identity, state),
        );
    }

    UiFontAssetRefreshReport {
        claims: transition.claims,
        font_collection_changed: text_state.refresh_font_collection(),
        font_records_reloaded,
    }
}

#[cfg(test)]
pub(super) fn ensure_font_asset_record<'a>(
    text_state: &mut TextRenderState,
    font_assets: &'a mut UiFontAssetCache,
    asset_manager: &ProjectAssetManager,
    asset_ref: &str,
    active_font_dependencies: &mut Vec<Arc<str>>,
    font_claim_scope: &mut RuntimeFontAssetClaimScope,
) -> EnsuredUiFontAsset<'a> {
    let resource_identity = asset_manager.resource_cache_identity(asset_ref);
    let cache_hit = font_assets
        .get(asset_ref)
        .is_some_and(|entry| entry.resource_identity == resource_identity);
    if !active_font_dependencies
        .iter()
        .any(|active| active.as_ref() == asset_ref)
    {
        active_font_dependencies.push(Arc::<str>::from(asset_ref));
    }
    let refresh = refresh_font_asset_records(
        text_state,
        font_assets,
        asset_manager,
        active_font_dependencies,
        font_claim_scope,
    );
    let entry = &font_assets[asset_ref];
    if cache_hit {
        EnsuredUiFontAsset::cache_hit(entry)
    } else {
        EnsuredUiFontAsset::reloaded(
            entry,
            matches!(entry.state, UiFontAssetCacheState::Ready(_)),
            refresh.font_collection_changed,
        )
    }
}

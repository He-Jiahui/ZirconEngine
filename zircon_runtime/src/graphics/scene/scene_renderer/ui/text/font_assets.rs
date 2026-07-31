use std::collections::{HashMap, hash_map::Entry};

use zircon_runtime_interface::ui::surface::{UiTextRenderMode, resolve_ui_text_render_mode};

use super::super::font_asset::load_ui_font_manifest_with_asset_manager;
use crate::asset::ProjectAssetManager;
use crate::core::framework::asset::{ResourceCacheIdentity, ResourceManager as _};
use crate::text::{CompositeFontDescriptor, TextRenderState};

#[derive(Clone, Debug, Default)]
pub(super) struct LoadedUiFontAsset {
    pub(super) family: Option<String>,
    pub(super) render_mode: Option<UiTextRenderMode>,
    pub(super) composite_font: Option<CompositeFontDescriptor>,
}

pub(super) struct EnsuredUiFontAsset<'a> {
    pub(super) faces_changed: bool,
    #[cfg(test)]
    pub(super) record: Option<&'a LoadedUiFontAsset>,
    #[cfg(test)]
    pub(super) loaded: bool,
    #[cfg(test)]
    pub(super) cache_hit: bool,
    #[cfg(test)]
    pub(super) status: UiFontAssetCacheStatus,
    #[cfg(not(test))]
    _marker: std::marker::PhantomData<&'a LoadedUiFontAsset>,
}

pub(super) type UiFontAssetCache = HashMap<String, UiFontAssetCacheEntry>;

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UiFontAssetCacheStatus {
    Ready,
    Missing,
    Error,
}

#[derive(Clone, Debug)]
pub(super) struct UiFontAssetCacheEntry {
    resource_identity: Option<ResourceCacheIdentity>,
    state: UiFontAssetCacheState,
}

#[derive(Clone, Debug)]
enum UiFontAssetCacheState {
    Ready(LoadedUiFontAsset),
    Missing,
    Error,
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
            UiFontAssetCacheState::Missing | UiFontAssetCacheState::Error => None,
        }
    }

    #[cfg(test)]
    fn status(&self) -> UiFontAssetCacheStatus {
        match &self.state {
            UiFontAssetCacheState::Ready(_) => UiFontAssetCacheStatus::Ready,
            UiFontAssetCacheState::Missing => UiFontAssetCacheStatus::Missing,
            UiFontAssetCacheState::Error => UiFontAssetCacheStatus::Error,
        }
    }
}

impl<'a> EnsuredUiFontAsset<'a> {
    fn cache_hit(entry: &'a UiFontAssetCacheEntry) -> Self {
        #[cfg(not(test))]
        let _ = entry;
        Self {
            faces_changed: false,
            #[cfg(test)]
            record: entry.loaded_asset(),
            #[cfg(test)]
            loaded: false,
            #[cfg(test)]
            cache_hit: true,
            #[cfg(test)]
            status: entry.status(),
            #[cfg(not(test))]
            _marker: std::marker::PhantomData,
        }
    }

    fn reloaded(entry: &'a UiFontAssetCacheEntry, loaded: bool, faces_changed: bool) -> Self {
        #[cfg(not(test))]
        let _ = (entry, loaded);
        Self {
            faces_changed,
            #[cfg(test)]
            record: entry.loaded_asset(),
            #[cfg(test)]
            loaded,
            #[cfg(test)]
            cache_hit: false,
            #[cfg(test)]
            status: entry.status(),
            #[cfg(not(test))]
            _marker: std::marker::PhantomData,
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

pub(super) fn load_font_asset_record(
    text_state: &mut TextRenderState,
    asset_ref: &str,
    asset_manager: &ProjectAssetManager,
) -> Option<(LoadedUiFontAsset, bool)> {
    let manifest = load_ui_font_manifest_with_asset_manager(asset_ref, Some(asset_manager))?;
    let report = text_state.replace_font_source(
        asset_ref,
        &manifest.source_path,
        manifest.asset.as_ref(),
        manifest.family.as_deref(),
        manifest.face_index,
    )?;
    let composite_font = manifest
        .asset
        .as_ref()
        .and_then(|asset| asset.composite_font.clone());
    Some((
        LoadedUiFontAsset {
            family: manifest.family,
            render_mode: manifest.render_mode,
            composite_font,
        },
        report.database_changed || report.asset_mapping_changed,
    ))
}

fn apply_default_font_asset_projection(
    text_state: &mut TextRenderState,
    record: Option<&LoadedUiFontAsset>,
) -> bool {
    let composite_changed = text_state
        .set_project_composite_font(record.and_then(|record| record.composite_font.clone()));
    let family_changed =
        text_state.set_default_ui_family_asset(record.and_then(|record| record.family.as_deref()));
    composite_changed || family_changed
}

pub(super) fn ensure_font_asset_record<'a>(
    text_state: &mut TextRenderState,
    font_assets: &'a mut UiFontAssetCache,
    asset_manager: &ProjectAssetManager,
    asset_ref: &str,
) -> EnsuredUiFontAsset<'a> {
    let resource_identity = asset_manager.resource_cache_identity(asset_ref);
    let slot = font_assets.entry(asset_ref.to_string());
    match slot {
        Entry::Occupied(slot) if slot.get().resource_identity == resource_identity => {
            let entry = slot.into_mut();
            EnsuredUiFontAsset::cache_hit(entry)
        }
        slot => {
            let loaded_record = load_font_asset_record(text_state, asset_ref, asset_manager);
            let (record, source_changed) = match loaded_record {
                Some((record, database_changed)) => (Some(record), database_changed),
                None => {
                    let report = text_state.remove_font_asset(asset_ref);
                    (
                        None,
                        report.database_changed || report.asset_mapping_changed,
                    )
                }
            };
            let projection_changed = asset_ref == super::DEFAULT_FONT_ASSET
                && apply_default_font_asset_projection(text_state, record.as_ref());
            let font_inputs_changed = source_changed || projection_changed;
            let state = match record {
                Some(record) => UiFontAssetCacheState::Ready(record),
                None if resource_identity.is_some() => UiFontAssetCacheState::Error,
                None => UiFontAssetCacheState::Missing,
            };
            let loaded = matches!(&state, UiFontAssetCacheState::Ready(_));
            let new_entry = UiFontAssetCacheEntry::new(resource_identity, state);
            let entry = match slot {
                Entry::Occupied(mut slot) => {
                    slot.insert(new_entry);
                    slot.into_mut()
                }
                Entry::Vacant(slot) => slot.insert(new_entry),
            };

            EnsuredUiFontAsset::reloaded(entry, loaded, font_inputs_changed)
        }
    }
}

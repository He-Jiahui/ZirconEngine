use std::collections::BTreeMap;

use zircon_runtime::asset::AssetUri;
use zircon_runtime_interface::resource::{ResourceKind, ResourceScheme};
use zircon_runtime_interface::ui::component::UiValue;

use crate::core::document::ScenePickerTicket;
use crate::core::project::{SceneCreateRequest, SceneOpenRequest};
use crate::ui::host::editor_asset_manager::EditorAssetCatalogGeneration;
use crate::ui::retained_host::callback_dispatch::WorkbenchCommandPaletteOpenState;

const SCENE_PICKER_WINDOW_ENTRIES: usize = 12;
const SCENE_CREATE_COMMAND_ID: &str = "scene-picker-create-confirm";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ScenePickerMode {
    Open,
    Create,
}

impl ScenePickerMode {
    pub(super) const fn command_source(self) -> &'static str {
        match self {
            Self::Open => "scene-picker-open",
            Self::Create => "scene-picker-create",
        }
    }

    pub(super) const fn placeholder(self) -> &'static str {
        match self {
            Self::Open => "Search project scenes",
            Self::Create => "Enter res://path/new.scene.toml",
        }
    }

    pub(super) const fn empty_text(self) -> &'static str {
        match self {
            Self::Open => "No scene assets found in this project",
            Self::Create => "Enter a project-owned res://*.scene.toml destination",
        }
    }

    pub(super) const fn accessibility_label(self) -> &'static str {
        match self {
            Self::Open => "Open project scene",
            Self::Create => "Create project scene",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ScenePickerEntry {
    command_id: String,
    scene_uri: String,
}

impl ScenePickerEntry {
    pub(super) fn scene_uri(&self) -> &str {
        &self.scene_uri
    }

    #[cfg(test)]
    pub(super) fn command_source(&self) -> &'static str {
        ScenePickerMode::Open.command_source()
    }

    fn matches_query(&self, normalized_query: &str) -> bool {
        normalized_query.is_empty()
            || contains_ascii_case_insensitive(&self.scene_uri, normalized_query)
    }

    fn command_value(&self) -> UiValue {
        command_value(
            &self.command_id,
            &self.scene_uri,
            ScenePickerMode::Open.command_source(),
        )
    }
}

/// Retains the catalog snapshot and capability that were current when a scene picker opened.
///
/// The host must submit the ticket returned by this session unchanged; the core document route
/// rejects it if the active project changes while the picker is visible.
pub(super) struct ScenePickerSession {
    ticket: ScenePickerTicket,
    mode: ScenePickerMode,
    entries: Vec<ScenePickerEntry>,
    catalog_generation: u64,
}

impl ScenePickerSession {
    pub(super) fn new(
        ticket: ScenePickerTicket,
        mode: ScenePickerMode,
        catalog: &EditorAssetCatalogGeneration,
    ) -> Self {
        Self {
            ticket,
            mode,
            entries: match mode {
                ScenePickerMode::Open => scene_entries_from_catalog(catalog),
                ScenePickerMode::Create => Vec::new(),
            },
            catalog_generation: catalog.publish_epoch,
        }
    }

    pub(super) fn mode(&self) -> ScenePickerMode {
        self.mode
    }

    pub(super) fn command_palette_state(
        &self,
        query: &str,
        requested_offset: usize,
        focus_last: bool,
    ) -> WorkbenchCommandPaletteOpenState {
        match self.mode {
            ScenePickerMode::Open => {
                self.open_scene_palette_state(query, requested_offset, focus_last)
            }
            ScenePickerMode::Create => self.create_scene_palette_state(query),
        }
    }

    pub(super) fn submission(
        &self,
        command_id: &str,
        query: &str,
        requested_offset: usize,
    ) -> Result<ScenePickerSubmission, String> {
        match self.mode {
            ScenePickerMode::Open => {
                let entry = scene_entry_for_open_submission(
                    &self.entries,
                    command_id,
                    query,
                    requested_offset,
                )?;
                let scene_uri = parse_project_scene_uri(entry.scene_uri())?;
                Ok(ScenePickerSubmission::Open {
                    ticket: self.ticket.clone(),
                    request: SceneOpenRequest::new(scene_uri),
                })
            }
            ScenePickerMode::Create => {
                if command_id != SCENE_CREATE_COMMAND_ID {
                    return Err("scene creation requires the current destination entry".to_string());
                }
                Ok(ScenePickerSubmission::Create {
                    ticket: self.ticket.clone(),
                    request: scene_create_request_for_query(query)?,
                })
            }
        }
    }

    fn open_scene_palette_state(
        &self,
        query: &str,
        requested_offset: usize,
        focus_last: bool,
    ) -> WorkbenchCommandPaletteOpenState {
        scene_open_palette_state_from_entries(
            &self.entries,
            self.catalog_generation,
            query,
            requested_offset,
            focus_last,
        )
    }

    fn create_scene_palette_state(&self, query: &str) -> WorkbenchCommandPaletteOpenState {
        let candidate = scene_create_request_for_query(query).ok();
        let has_candidate = candidate.is_some();
        let commands = candidate
            .as_ref()
            .map(|request| {
                UiValue::Array(vec![command_value(
                    SCENE_CREATE_COMMAND_ID,
                    &format!("Create {}", request.scene_uri()),
                    ScenePickerMode::Create.command_source(),
                )])
            })
            .unwrap_or_else(|| UiValue::Array(Vec::new()));
        let selected_command_id = candidate
            .as_ref()
            .map(|_| SCENE_CREATE_COMMAND_ID.to_string())
            .unwrap_or_default();

        WorkbenchCommandPaletteOpenState {
            query: query.to_string(),
            commands,
            filtered_commands: UiValue::Array(
                (!selected_command_id.is_empty())
                    .then_some(UiValue::String(selected_command_id.clone()))
                    .into_iter()
                    .collect(),
            ),
            selected_command_id,
            focused_index: if has_candidate { 0 } else { -1 },
            catalog_generation: self.catalog_generation,
            total_match_count: usize::from(has_candidate),
            window_offset: 0,
        }
    }
}

pub(super) enum ScenePickerSubmission {
    Open {
        ticket: ScenePickerTicket,
        request: SceneOpenRequest,
    },
    Create {
        ticket: ScenePickerTicket,
        request: SceneCreateRequest,
    },
}

pub(super) fn scene_entries_from_catalog(
    catalog: &EditorAssetCatalogGeneration,
) -> Vec<ScenePickerEntry> {
    let mut scene_uris = catalog
        .assets
        .iter()
        .filter(|asset| asset.kind == ResourceKind::Scene)
        .filter_map(|asset| {
            parse_project_scene_uri(&asset.locator)
                .ok()
                .map(|_| (asset.locator.to_lowercase(), asset.locator.as_str()))
        })
        .collect::<Vec<_>>();
    scene_uris.sort_unstable();
    scene_uris.dedup();
    scene_uris
        .into_iter()
        .enumerate()
        .map(|(index, (_, scene_uri))| ScenePickerEntry {
            command_id: format!("scene-picker-open-{index}"),
            scene_uri: scene_uri.to_string(),
        })
        .collect()
}

#[cfg(test)]
pub(super) fn scene_open_palette_state(
    catalog: &EditorAssetCatalogGeneration,
    query: &str,
    requested_offset: usize,
    focus_last: bool,
) -> WorkbenchCommandPaletteOpenState {
    let entries = scene_entries_from_catalog(catalog);
    scene_open_palette_state_from_entries(
        &entries,
        catalog.publish_epoch,
        query,
        requested_offset,
        focus_last,
    )
}

pub(super) fn scene_create_request_for_query(query: &str) -> Result<SceneCreateRequest, String> {
    Ok(SceneCreateRequest::new(parse_project_scene_uri(query)?))
}

pub(super) fn scene_entry_for_open_submission<'a>(
    entries: &'a [ScenePickerEntry],
    command_id: &str,
    query: &str,
    requested_offset: usize,
) -> Result<&'a ScenePickerEntry, String> {
    scene_open_query_window(entries, query, requested_offset)
        .entries
        .into_iter()
        .find(|entry| entry.command_id == command_id)
        .ok_or_else(|| "selected scene is no longer available in this picker".to_string())
}

fn parse_project_scene_uri(value: &str) -> Result<AssetUri, String> {
    let scene_uri = AssetUri::parse(value.trim()).map_err(|error| error.to_string())?;
    if scene_uri.scheme() != ResourceScheme::Res {
        return Err("scene assets must use a project-owned res:// URI".to_string());
    }
    if !scene_uri.path().ends_with(".scene.toml") {
        return Err("scene asset URI must end in .scene.toml".to_string());
    }
    if scene_uri.label().is_some() {
        return Err("scene asset URI cannot target a sub-asset label".to_string());
    }
    Ok(scene_uri)
}

fn normalized_window_offset(requested_offset: usize, total_match_count: usize) -> usize {
    if total_match_count == 0 {
        return 0;
    }
    let last_offset = total_match_count
        .saturating_sub(1)
        .checked_div(SCENE_PICKER_WINDOW_ENTRIES)
        .unwrap_or(0)
        .saturating_mul(SCENE_PICKER_WINDOW_ENTRIES);
    requested_offset
        .checked_div(SCENE_PICKER_WINDOW_ENTRIES)
        .unwrap_or(0)
        .saturating_mul(SCENE_PICKER_WINDOW_ENTRIES)
        .min(last_offset)
}

struct SceneOpenQueryWindow<'a> {
    total_match_count: usize,
    window_offset: usize,
    entries: Vec<&'a ScenePickerEntry>,
}

fn scene_open_query_window<'a>(
    entries: &'a [ScenePickerEntry],
    query: &str,
    requested_offset: usize,
) -> SceneOpenQueryWindow<'a> {
    let normalized_query = query.trim().to_ascii_lowercase();
    let total_match_count = entries
        .iter()
        .filter(|entry| entry.matches_query(&normalized_query))
        .count();
    let window_offset = normalized_window_offset(requested_offset, total_match_count);
    let entries = entries
        .iter()
        .filter(|entry| entry.matches_query(&normalized_query))
        .skip(window_offset)
        .take(SCENE_PICKER_WINDOW_ENTRIES)
        .collect();

    SceneOpenQueryWindow {
        total_match_count,
        window_offset,
        entries,
    }
}

fn scene_open_palette_state_from_entries(
    entries: &[ScenePickerEntry],
    catalog_generation: u64,
    query: &str,
    requested_offset: usize,
    focus_last: bool,
) -> WorkbenchCommandPaletteOpenState {
    let result_window = scene_open_query_window(entries, query, requested_offset);
    let visible_entries = result_window.entries;
    let selected_entry = if focus_last {
        visible_entries.last().copied()
    } else {
        visible_entries.first().copied()
    };

    WorkbenchCommandPaletteOpenState {
        query: query.to_string(),
        commands: UiValue::Array(
            visible_entries
                .iter()
                .map(|entry| entry.command_value())
                .collect(),
        ),
        filtered_commands: UiValue::Array(
            visible_entries
                .iter()
                .map(|entry| UiValue::String(entry.command_id.clone()))
                .collect(),
        ),
        selected_command_id: selected_entry
            .map(|entry| entry.command_id.clone())
            .unwrap_or_default(),
        focused_index: selected_entry
            .map(|_| {
                if focus_last {
                    visible_entries.len() as i64 - 1
                } else {
                    0
                }
            })
            .unwrap_or(-1),
        catalog_generation,
        total_match_count: result_window.total_match_count,
        window_offset: result_window.window_offset,
    }
}

fn contains_ascii_case_insensitive(haystack: &str, normalized_needle: &str) -> bool {
    if normalized_needle.is_empty() {
        return true;
    }
    haystack
        .as_bytes()
        .windows(normalized_needle.len())
        .any(|candidate| candidate.eq_ignore_ascii_case(normalized_needle.as_bytes()))
}

fn command_value(command_id: &str, label: &str, source: &str) -> UiValue {
    UiValue::Map(BTreeMap::from([
        ("id".to_string(), UiValue::String(command_id.to_string())),
        ("label".to_string(), UiValue::String(label.to_string())),
        ("source".to_string(), UiValue::String(source.to_string())),
        ("category".to_string(), UiValue::String("Scene".to_string())),
        (
            "keywords".to_string(),
            UiValue::Array(vec![UiValue::String("scene".to_string())]),
        ),
    ]))
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use zircon_runtime::asset::project::PreviewState;

    use super::*;
    use crate::ui::host::editor_asset_manager::{
        EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord,
    };

    #[test]
    fn optimization_wave_20260824q_editor51_scene_picker_sort_preserves_order_and_exact_dedup() {
        let catalog = EditorAssetCatalogGeneration::from_snapshot_record(
            EditorAssetCatalogSnapshotRecord {
                assets: [
                    "res://levels/alpha.scene.toml",
                    "res://levels/Alpha.scene.toml",
                    "res://levels/zebra.scene.toml",
                    "res://levels/alpha.scene.toml",
                ]
                .into_iter()
                .map(catalog_record)
                .collect(),
                ..EditorAssetCatalogSnapshotRecord::default()
            },
            1,
        );

        let entries = scene_entries_from_catalog(&catalog);
        let uris = entries
            .iter()
            .map(ScenePickerEntry::scene_uri)
            .collect::<Vec<_>>();

        assert_eq!(
            uris,
            vec![
                "res://levels/Alpha.scene.toml",
                "res://levels/alpha.scene.toml",
                "res://levels/zebra.scene.toml",
            ]
        );
    }

    #[test]
    fn optimization_wave_20260824q_editor51_scene_picker_sort_borrows_original_uris() {
        let source = include_str!("scene_picker_session.rs");
        let (production, _) = source
            .rsplit_once("#[cfg(test)]")
            .expect("scene picker optimization test module");
        let compact = production.split_whitespace().collect::<String>();

        assert!(compact.contains("(asset.locator.to_lowercase(),asset.locator.as_str())"));
        assert!(production.contains("scene_uris.sort_unstable()"));
        assert!(production.contains("scene_uris.dedup()"));
        assert!(!production.contains(
            "sort_by_cached_key(|scene_uri| (scene_uri.to_lowercase(), scene_uri.clone()))"
        ));
    }

    #[test]
    #[ignore = "managed release evidence"]
    fn optimization_wave_20260824q_editor51_scene_picker_sort_projection_evidence() {
        const URI_COUNT: usize = 20_000;
        const SAMPLE_PAIRS: usize = 11;
        const TARGET: Duration = Duration::from_millis(100);

        let locators = (0..URI_COUNT)
            .rev()
            .map(|index| {
                format!(
                    "res://world/region-{index:05}/streaming/sublevel/authoring/scene-{index:05}.scene.toml"
                )
            })
            .collect::<Vec<_>>();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_ns(|| legacy_scene_uri_projection(&locators)));
                optimized_samples.push(measure_ns(|| borrowed_scene_uri_projection(&locators)));
            } else {
                optimized_samples.push(measure_ns(|| borrowed_scene_uri_projection(&locators)));
                legacy_samples.push(measure_ns(|| legacy_scene_uri_projection(&locators)));
            }
        }

        let legacy_p95 = nearest_rank(&legacy_samples, 95);
        let optimized_p95 = nearest_rank(&optimized_samples, 95);
        let locator_bytes = locators.iter().map(String::len).sum::<usize>();
        let string_bytes_before = locator_bytes * 3;
        let string_bytes_after = locator_bytes * 2;
        let string_byte_reduction_percent =
            (1.0 - string_bytes_after as f64 / string_bytes_before as f64) * 100.0;

        assert!(
            optimized_p95 <= TARGET.as_nanos(),
            "optimized_p95_ns={optimized_p95} target_ns={}",
            TARGET.as_nanos()
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(90),
            "optimized_p95_ns={optimized_p95} legacy_p95_ns={legacy_p95}"
        );
        println!(
            "EDITOR51_SCENE_PICKER_SORT_BENCH_V1 uri_count={} locator_bytes={} string_bytes_before={} string_bytes_after={} string_byte_reduction_percent={:.4} legacy_p95_ns={} optimized_p95_ns={} target_ns={}",
            URI_COUNT,
            locator_bytes,
            string_bytes_before,
            string_bytes_after,
            string_byte_reduction_percent,
            legacy_p95,
            optimized_p95,
            TARGET.as_nanos()
        );
    }

    fn legacy_scene_uri_projection(locators: &[String]) -> u64 {
        let mut projected = locators.to_vec();
        projected.sort_by_cached_key(|scene_uri| (scene_uri.to_lowercase(), scene_uri.clone()));
        projected.dedup();
        projection_checksum(&projected)
    }

    fn borrowed_scene_uri_projection(locators: &[String]) -> u64 {
        let mut projected = locators
            .iter()
            .map(|scene_uri| (scene_uri.to_lowercase(), scene_uri.as_str()))
            .collect::<Vec<_>>();
        projected.sort_unstable();
        projected.dedup();
        let projected = projected
            .into_iter()
            .map(|(_, scene_uri)| scene_uri.to_string())
            .collect::<Vec<_>>();
        projection_checksum(&projected)
    }

    fn projection_checksum(projected: &[String]) -> u64 {
        black_box(projected.iter().fold(0_u64, |checksum, uri| {
            checksum.wrapping_add(uri.len() as u64)
        }))
    }

    fn measure_ns(measure: impl FnOnce() -> u64) -> u128 {
        let started = Instant::now();
        black_box(measure());
        started.elapsed().as_nanos()
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn catalog_record(locator: &str) -> EditorAssetCatalogRecord {
        EditorAssetCatalogRecord {
            uuid: format!("asset-{locator}"),
            id: locator.to_string(),
            locator: locator.to_string(),
            kind: ResourceKind::Scene,
            display_name: locator.to_string(),
            file_name: locator.to_string(),
            extension: "toml".to_string(),
            preview_state: PreviewState::Dirty,
            meta_path: String::new(),
            preview_artifact_path: String::new(),
            source_mtime_unix_ms: 0,
            source_hash: String::new(),
            dirty: false,
            diagnostics: Vec::new(),
            direct_reference_uuids: Vec::new(),
        }
    }
}

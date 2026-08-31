use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::view_projection::{
    build_view_template_node_projection, compose_view_template_node_model,
    view_template_resource_generation, ViewTemplateResourceGeneration,
};
use crate::ui::layouts::windows::workbench_host_window::AssetBrowserPaneViewData;
use crate::ui::retained_host::{
    primitives::ModelRc,
    runtime_text_metrics_generation,
    ui_perf::{record_current_ui_perf_counter, UiPerfCounter},
};
use crate::ui::workbench::asset_content_layout::{
    asset_browser_materialized_item_budget, asset_content_paint_metadata, AssetBrowserPaintItem,
    AssetContentPaintNodeInput, AssetContentSurface,
};
use crate::ui::workbench::snapshot::{
    AssetItemSnapshot, AssetUtilityTab, AssetViewMode, AssetWorkspaceSnapshot,
};
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::{layout::UiSize, surface::UiSurfaceFrame};

use super::ViewTemplateNodeData;
use compact_layout::{apply_asset_browser_compact_layout, apply_asset_browser_sources_layout};
use labels::asset_state_label;
use logical_paint_source::{asset_browser_logical_paint_items, selected_asset_item_indices};
use reference_nodes::{apply_asset_browser_reference_layout, sync_asset_browser_reference_nodes};
use selection_text::{
    has_asset_selection, selected_asset, selected_folder_breadcrumb, selection_diagnostics_text,
    selection_display_name, selection_identity, selection_kind_label, selection_locator,
    selection_metadata_body, selection_metadata_summary, selection_revision_label,
};
use source_tree_nodes::sync_asset_browser_source_tree_nodes;
use stack_layout::apply_asset_browser_standard_stack_layout;
use state_marks::{
    mark_panel_group_selected, mark_panel_selected, mark_toggle_state, mark_utility_tab_state,
};
use summary_nodes::sync_asset_browser_summary_nodes;
use table_nodes::{
    apply_asset_browser_list_logical_extent, apply_asset_browser_table_cells,
    mark_asset_table_rows, sync_asset_table_nodes,
};
use thumbnail_layout::apply_thumbnail_grid_logical_extent;
use thumbnail_nodes::{append_asset_browser_thumbnail_slots, trim_asset_browser_thumbnail_slots};
use toolbar_layout::apply_asset_browser_toolbar_layout;
use utility_tabs::apply_asset_browser_utility_tab_typography;

mod compact_layout;
mod compact_table_layout;
mod labels;
mod logical_paint_source;
mod name_compaction;
mod name_lines;
mod reference_nodes;
mod selection_text;
mod source_tree_nodes;
mod stack_layout;
mod state_marks;
mod summary_layout;
mod summary_nodes;
mod table_nodes;
#[cfg(test)]
mod tests;
mod thumbnail_layout;
mod thumbnail_nodes;
#[cfg(test)]
mod token_contracts;
mod toolbar_layout;
#[cfg(test)]
mod toolbar_responsiveness_tests;
mod utility_tabs;

const ASSET_BROWSER_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/asset_browser.zui";
const ASSET_BROWSER_VIRTUAL_OVERSCAN_ROWS: usize = 2;

struct AssetBrowserPaneProjectionCacheEntry {
    input: AssetBrowserProjectionInput,
    resource_generation: ViewTemplateResourceGeneration,
    text_metrics_generation: [u64; 3],
    composition_generation: u64,
    width_bits: u32,
    height_bits: u32,
    nodes: ModelRc<ViewTemplateNodeData>,
    render_source_frame: Option<Arc<UiSurfaceFrame>>,
}

struct AssetBrowserProjectionInput {
    project_root: String,
    catalog_revision: u64,
    view_mode: AssetViewMode,
    utility_tab: AssetUtilityTab,
    search_query: String,
    mesh_import_path: String,
    kind_filter: Option<ResourceKind>,
    selected_folder_id: Option<String>,
    selected_asset_uuid: Option<String>,
}

impl AssetBrowserProjectionInput {
    fn new(snapshot: &AssetWorkspaceSnapshot) -> Self {
        Self {
            project_root: snapshot.project_root.clone(),
            catalog_revision: snapshot.catalog_revision,
            view_mode: snapshot.view_mode,
            utility_tab: snapshot.utility_tab,
            search_query: snapshot.search_query.clone(),
            mesh_import_path: snapshot.mesh_import_path.clone(),
            kind_filter: snapshot.kind_filter.clone(),
            selected_folder_id: snapshot.selected_folder_id.clone(),
            selected_asset_uuid: snapshot.selected_asset_uuid.clone(),
        }
    }

    fn matches(&self, snapshot: &AssetWorkspaceSnapshot) -> bool {
        self.project_root == snapshot.project_root
            && self.catalog_revision == snapshot.catalog_revision
            && self.view_mode == snapshot.view_mode
            && self.utility_tab == snapshot.utility_tab
            && self.search_query == snapshot.search_query
            && self.mesh_import_path == snapshot.mesh_import_path
            && self.kind_filter == snapshot.kind_filter
            && self.selected_folder_id == snapshot.selected_folder_id
            && self.selected_asset_uuid == snapshot.selected_asset_uuid
    }
}

thread_local! {
    static ASSET_BROWSER_PANE_PROJECTION_CACHE: RefCell<Option<AssetBrowserPaneProjectionCacheEntry>> =
        const { RefCell::new(None) };
}

pub(crate) fn asset_browser_pane_data(
    snapshot: &AssetWorkspaceSnapshot,
    size: UiSize,
) -> AssetBrowserPaneViewData {
    let width_bits = size.width.to_bits();
    let height_bits = size.height.to_bits();
    let resource_generation =
        view_template_resource_generation(ASSET_BROWSER_LAYOUT_ASSET_PATH, &[]);
    let text_metrics_generation = runtime_text_metrics_generation();
    if let Some(data) = ASSET_BROWSER_PANE_PROJECTION_CACHE.with(|cache| {
        let cache = cache.borrow();
        cache.as_ref().and_then(|cached| {
            (cached.width_bits == width_bits
                && cached.height_bits == height_bits
                && cached.input.matches(snapshot)
                && resource_generation
                    .as_ref()
                    .is_some_and(|current| current == &cached.resource_generation)
                && cached.text_metrics_generation == text_metrics_generation)
                .then(|| AssetBrowserPaneViewData {
                    nodes: cached.nodes.clone(),
                    render_source_frame: cached.render_source_frame.clone(),
                })
        })
    }) {
        return data;
    }
    let composition_generation = ASSET_BROWSER_PANE_PROJECTION_CACHE.with(|cache| {
        cache
            .borrow()
            .as_ref()
            .map_or(1, |cached| cached.composition_generation.wrapping_add(1))
    });
    record_current_ui_perf_counter(UiPerfCounter::AssetBrowserProjectionBuildCount, 1.0);

    let mut text_overrides = BTreeMap::new();
    let selected_asset = selected_asset(snapshot);
    let project_root = if snapshot.project_root.is_empty() {
        "Project/assets".to_string()
    } else {
        snapshot.project_root.clone()
    };
    let selection_name = selection_display_name(&snapshot.selection, selected_asset);
    let selection_locator = selection_locator(&snapshot.selection, selected_asset);
    let selection_kind = selection_kind_label(&snapshot.selection, selected_asset).to_string();
    let selection_identity = selection_identity(&snapshot.selection, selected_asset);
    let selection_revision = selection_revision_label(&snapshot.selection, selected_asset);
    let selection_diagnostics = selection_diagnostics_text(&snapshot.selection, selected_asset);
    let selection_metadata_summary = selection_metadata_summary(&snapshot.selection);
    let selection_metadata_body =
        selection_metadata_body(&snapshot.selection, &selection_diagnostics);
    let catalog_summary = format!(
        "{} folders • {} assets",
        snapshot.visible_folders.len(),
        snapshot.visible_assets.len()
    );
    let logical_item_count = snapshot.visible_assets.len();
    let materialized_item_count = asset_browser_materialized_item_budget(
        snapshot.view_mode,
        size.height,
        logical_item_count,
        ASSET_BROWSER_VIRTUAL_OVERSCAN_ROWS,
    );
    let browser_paint_items = asset_browser_logical_paint_items(snapshot, text_metrics_generation);
    let selected_asset_item_indices = selected_asset_item_indices(snapshot);

    text_overrides.insert("AssetBrowserSubtitleText".to_string(), project_root.clone());
    text_overrides.insert(
        "AssetBrowserSourcesSubtitleText".to_string(),
        project_root.clone(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsHeaderSelectionText".to_string(),
        selection_locator.clone(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsPreviewNameText".to_string(),
        selection_name.clone(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsPreviewLocatorText".to_string(),
        selection_locator.clone(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsPreviewKindText".to_string(),
        selection_kind.clone(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsPreviewIdentityText".to_string(),
        selection_identity.clone(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsPreviewToolkitText".to_string(),
        if snapshot.selection.toolkit_view_id.is_empty() {
            "No toolkit".to_string()
        } else {
            snapshot.selection.toolkit_view_id.clone()
        },
    );
    text_overrides.insert(
        "AssetBrowserDetailsPreviewMetaPathText".to_string(),
        if snapshot.selection.meta_path.is_empty() {
            "No meta path".to_string()
        } else {
            snapshot.selection.meta_path.clone()
        },
    );
    text_overrides.insert(
        "AssetBrowserDetailsPreviewDiagnosticsText".to_string(),
        selection_diagnostics.clone(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsLocatorValue".to_string(),
        selection_locator.clone(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsTypeValue".to_string(),
        selection_kind.clone(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsIdentityUuidValue".to_string(),
        selection_identity,
    );
    text_overrides.insert(
        "AssetBrowserDetailsIdentityRevisionValue".to_string(),
        selection_revision,
    );
    text_overrides.insert(
        "AssetBrowserDetailsMetadataMetaPathValue".to_string(),
        if snapshot.selection.meta_path.is_empty() {
            "No meta path".to_string()
        } else {
            snapshot.selection.meta_path.clone()
        },
    );
    text_overrides.insert(
        "AssetBrowserDetailsMetadataToolkitValue".to_string(),
        selection_metadata_summary.clone(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsDiagnosticsText".to_string(),
        selection_diagnostics.clone(),
    );
    text_overrides.insert(
        "AssetBrowserSelectionLocatorText".to_string(),
        selection_locator.clone(),
    );
    text_overrides.insert("AssetBrowserPreviewNameText".to_string(), selection_name);
    text_overrides.insert(
        "AssetBrowserPreviewLocatorText".to_string(),
        selection_locator,
    );
    text_overrides.insert("AssetBrowserPreviewKindText".to_string(), selection_kind);
    text_overrides.insert(
        "AssetBrowserPreviewIdentityText".to_string(),
        snapshot
            .selection
            .uuid
            .clone()
            .unwrap_or_else(|| "No UUID".to_string()),
    );
    text_overrides.insert(
        "AssetBrowserPreviewToolkitText".to_string(),
        if snapshot.selection.toolkit_view_id.is_empty() {
            "No toolkit".to_string()
        } else {
            snapshot.selection.toolkit_view_id.clone()
        },
    );
    text_overrides.insert(
        "AssetBrowserPreviewMetaPathText".to_string(),
        if snapshot.selection.meta_path.is_empty() {
            "No meta path".to_string()
        } else {
            snapshot.selection.meta_path.clone()
        },
    );
    text_overrides.insert(
        "AssetBrowserPreviewDiagnosticsText".to_string(),
        selection_diagnostics.clone(),
    );
    text_overrides.insert(
        "AssetBrowserMetaPathValue".to_string(),
        if snapshot.selection.meta_path.is_empty() {
            "No meta path".to_string()
        } else {
            snapshot.selection.meta_path.clone()
        },
    );
    text_overrides.insert(
        "AssetBrowserToolkitValue".to_string(),
        selection_metadata_summary,
    );
    text_overrides.insert(
        "AssetBrowserToolkitLabel".to_string(),
        "Toolkit / Package".to_string(),
    );
    text_overrides.insert(
        "AssetBrowserDiagnosticsText".to_string(),
        selection_metadata_body,
    );
    text_overrides.insert(
        "AssetBrowserDiagnosticsLabel".to_string(),
        "Diagnostics / Subassets".to_string(),
    );
    text_overrides.insert(
        "AssetBrowserPluginsText".to_string(),
        format!(
            "{} • view {:?} • tab {:?}",
            catalog_summary, snapshot.view_mode, snapshot.utility_tab
        ),
    );
    text_overrides.insert(
        "AssetBrowserViewModeListButton".to_string(),
        "List".to_string(),
    );
    text_overrides.insert(
        "AssetBrowserViewModeThumbButton".to_string(),
        "Thumb".to_string(),
    );
    text_overrides.insert(
        "AssetBrowserPreviewTabButton".to_string(),
        "Preview".to_string(),
    );
    text_overrides.insert(
        "AssetBrowserReferencesTabButton".to_string(),
        "References".to_string(),
    );
    text_overrides.insert(
        "AssetBrowserMetadataTabButton".to_string(),
        "Metadata".to_string(),
    );
    text_overrides.insert(
        "AssetBrowserPluginsTabButton".to_string(),
        "Plugins".to_string(),
    );
    text_overrides.insert("SearchEdited".to_string(), snapshot.search_query.clone());
    text_overrides.insert(
        "AssetBrowserImportPathField".to_string(),
        snapshot.mesh_import_path.clone(),
    );
    text_overrides.insert(
        "AssetBrowserContentHeaderPathText".to_string(),
        selected_folder_breadcrumb(snapshot).unwrap_or(catalog_summary.clone()),
    );
    for (index, item) in browser_paint_items
        .iter()
        .take(materialized_item_count)
        .enumerate()
    {
        let AssetBrowserPaintItem::List(item) = item else {
            continue;
        };
        text_overrides.insert(
            format!("WorkbenchAssetBrowserAssetRow{:02}", index + 1),
            item.text.clone(),
        );
    }
    if let Some(asset) = selected_asset {
        text_overrides.insert(
            "AssetBrowserContentPreviewName".to_string(),
            asset.display_name.clone(),
        );
        text_overrides.insert(
            "AssetBrowserContentPreviewMeta".to_string(),
            asset_state_label(asset).to_string(),
        );
    } else {
        text_overrides.insert(
            "AssetBrowserContentPreviewName".to_string(),
            "No Asset Selected".to_string(),
        );
        text_overrides.insert(
            "AssetBrowserContentPreviewMeta".to_string(),
            "Select a table row to inspect".to_string(),
        );
    }
    let Ok(projection) = build_view_template_node_projection(
        "asset_browser.template_projection",
        ASSET_BROWSER_LAYOUT_ASSET_PATH,
        &[],
        size,
        &text_overrides,
    ) else {
        return AssetBrowserPaneViewData::default();
    };
    let render_source_frame = projection.source_frame();
    let nodes = compose_view_template_node_model(
        "asset_browser.template_composition",
        projection,
        &composition_generation,
        move |nodes| {
            sync_asset_table_nodes(nodes, snapshot.view_mode, materialized_item_count);
            sync_asset_browser_source_tree_nodes(nodes, snapshot);
            sync_asset_browser_reference_nodes(nodes, snapshot);
            let toolbar_layout = apply_asset_browser_toolbar_layout(nodes, size.width);
            apply_asset_browser_visual_state(nodes, snapshot);
            apply_asset_browser_table_cells(nodes, &browser_paint_items);
            append_asset_browser_thumbnail_slots(nodes, snapshot, materialized_item_count);
            sync_asset_browser_summary_nodes(nodes, snapshot);
            retain_active_utility_tab_nodes(nodes, snapshot.utility_tab);
            if let Some(toolbar_layout) = toolbar_layout.as_ref() {
                apply_asset_browser_standard_stack_layout(nodes, size, toolbar_layout);
            }
            apply_asset_browser_compact_layout(
                nodes,
                size,
                snapshot.view_mode,
                toolbar_layout.map(|layout| layout.main_y),
            );
            apply_asset_browser_sources_layout(nodes);
            apply_asset_browser_reference_layout(nodes);
            trim_asset_browser_thumbnail_slots(
                nodes,
                logical_item_count,
                ASSET_BROWSER_VIRTUAL_OVERSCAN_ROWS,
            );
            apply_asset_browser_list_logical_extent(nodes, logical_item_count);
            apply_thumbnail_grid_logical_extent(nodes, logical_item_count);
            let metadata = asset_content_paint_metadata(
                nodes.iter().map(|node| {
                    AssetContentPaintNodeInput::new(
                        node.control_id.as_str(),
                        node.frame.x,
                        node.frame.y,
                        node.frame.width,
                        node.frame.height,
                        node.value_number,
                    )
                }),
                AssetContentSurface::Browser,
            )
            .with_browser_virtual_items(
                browser_paint_items,
                selected_asset_item_indices,
                ASSET_BROWSER_VIRTUAL_OVERSCAN_ROWS,
            );
            #[cfg(feature = "profiling")]
            record_current_ui_perf_counter(
                UiPerfCounter::AssetContentGenerationIdentityParseCount,
                metadata.identity_parse_count() as f64,
            );
            metadata
        },
    );
    ASSET_BROWSER_PANE_PROJECTION_CACHE.with(|cache| {
        *cache.borrow_mut() =
            resource_generation.map(|resource_generation| AssetBrowserPaneProjectionCacheEntry {
                input: AssetBrowserProjectionInput::new(snapshot),
                resource_generation,
                text_metrics_generation,
                composition_generation,
                width_bits,
                height_bits,
                nodes: nodes.clone(),
                render_source_frame: render_source_frame.clone(),
            });
    });
    AssetBrowserPaneViewData {
        nodes,
        render_source_frame,
    }
}

pub(crate) fn asset_browser_pane_nodes(
    snapshot: &AssetWorkspaceSnapshot,
    size: UiSize,
) -> ModelRc<ViewTemplateNodeData> {
    asset_browser_pane_data(snapshot, size).nodes
}

#[cfg(test)]
pub(super) fn clear_asset_browser_pane_projection_cache_for_tests() {
    ASSET_BROWSER_PANE_PROJECTION_CACHE.with(|cache| *cache.borrow_mut() = None);
    logical_paint_source::clear_asset_browser_logical_paint_cache_for_tests();
}

#[cfg(test)]
#[test]
fn stable_asset_browser_snapshot_reuses_the_composed_model() {
    super::view_projection::clear_view_template_projection_caches_for_tests();
    clear_asset_browser_pane_projection_cache_for_tests();
    let snapshot = AssetWorkspaceSnapshot::default();
    let stable_snapshot = snapshot.clone();
    let size = UiSize::new(900.0, 620.0);

    let first = asset_browser_pane_nodes(&snapshot, size);
    let stable = asset_browser_pane_nodes(&stable_snapshot, size);

    assert!(first.shares_values_with(&stable));
}

#[cfg(test)]
#[test]
fn stable_asset_browser_snapshot_reuses_the_render_source_frame() {
    super::view_projection::clear_view_template_projection_caches_for_tests();
    clear_asset_browser_pane_projection_cache_for_tests();
    let snapshot = AssetWorkspaceSnapshot::default();
    let stable_snapshot = snapshot.clone();
    let size = UiSize::new(900.0, 620.0);

    let first = asset_browser_pane_data(&snapshot, size);
    let stable = asset_browser_pane_data(&stable_snapshot, size);
    let first_frame = first
        .render_source_frame
        .as_ref()
        .expect("initial projection should publish its runtime source frame");
    let stable_frame = stable
        .render_source_frame
        .as_ref()
        .expect("stable cache hit should retain its runtime source frame");

    assert!(Arc::ptr_eq(first_frame, stable_frame));
}

#[cfg(test)]
#[test]
fn catalog_generation_change_invalidates_the_pane_cache() {
    super::view_projection::clear_view_template_projection_caches_for_tests();
    clear_asset_browser_pane_projection_cache_for_tests();
    let snapshot = AssetWorkspaceSnapshot::default();
    let mut changed = snapshot.clone();
    changed.selection.display_name = "Changed selection".to_string();
    changed.catalog_revision = changed.catalog_revision.wrapping_add(1);
    let size = UiSize::new(900.0, 620.0);

    let first = asset_browser_pane_nodes(&snapshot, size);
    let next = asset_browser_pane_nodes(&changed, size);

    assert!(!first.shares_values_with(&next));
    assert!(next.iter().any(|node| node.text == "Changed selection"));
}

#[cfg(test)]
#[test]
fn mesh_import_path_change_invalidates_the_pane_cache_and_reprojects_the_field() {
    super::view_projection::clear_view_template_projection_caches_for_tests();
    clear_asset_browser_pane_projection_cache_for_tests();
    let snapshot = AssetWorkspaceSnapshot::default();
    let mut changed = snapshot.clone();
    changed.mesh_import_path = "E:/Models/cube.glb".to_string();
    let size = UiSize::new(900.0, 620.0);

    let first = asset_browser_pane_nodes(&snapshot, size);
    let next = asset_browser_pane_nodes(&changed, size);
    let import_path = next
        .iter()
        .find(|node| node.control_id == "AssetBrowserImportPathField")
        .expect("import path field");

    assert!(!first.shares_values_with(&next));
    assert_eq!(import_path.value_text.as_str(), "E:/Models/cube.glb");
}

#[cfg(test)]
#[test]
fn selection_change_reuses_the_generation_owned_logical_paint_source() {
    super::view_projection::clear_view_template_projection_caches_for_tests();
    clear_asset_browser_pane_projection_cache_for_tests();
    let mut snapshot = AssetWorkspaceSnapshot::default();
    snapshot.catalog_revision = 7;
    snapshot.view_mode = AssetViewMode::List;
    snapshot.visible_assets = vec![test_asset_item("asset-a"), test_asset_item("asset-b")].into();
    snapshot.selected_asset_uuid = Some("asset-a".to_string());
    let mut changed = snapshot.clone();
    changed.selected_asset_uuid = Some("asset-b".to_string());
    let size = UiSize::new(900.0, 620.0);

    let first = asset_browser_pane_nodes(&snapshot, size);
    let next = asset_browser_pane_nodes(&changed, size);
    let first_metadata = first
        .metadata_rc::<crate::ui::workbench::asset_content_layout::AssetContentPaintMetadata>()
        .expect("first Asset Browser paint metadata");
    let next_metadata = next
        .metadata_rc::<crate::ui::workbench::asset_content_layout::AssetContentPaintMetadata>()
        .expect("next Asset Browser paint metadata");

    assert!(first_metadata.shares_browser_logical_items_with(&next_metadata));
}

#[cfg(test)]
#[test]
fn local_asset_delta_reuses_unchanged_logical_paint_chunks_in_pane_metadata() {
    super::view_projection::clear_view_template_projection_caches_for_tests();
    clear_asset_browser_pane_projection_cache_for_tests();
    let mut snapshot = AssetWorkspaceSnapshot::default();
    snapshot.catalog_revision = 7;
    snapshot.view_mode = AssetViewMode::List;
    snapshot.visible_assets = (0..130)
        .map(|index| test_asset_item(&format!("asset-{index:03}")))
        .collect();
    let mut changed = snapshot.clone();
    let mut replacement = changed.visible_assets[65].clone();
    replacement.display_name = "changed-asset-065.mesh".to_string();
    changed.catalog_revision = changed.catalog_revision.wrapping_add(1);
    changed.visible_assets = changed
        .visible_assets
        .replace_existing_items([replacement])
        .expect("existing asset delta should preserve visible membership");
    let size = UiSize::new(900.0, 620.0);

    let first = asset_browser_pane_nodes(&snapshot, size);
    let next = asset_browser_pane_nodes(&changed, size);
    let first_metadata = first
        .metadata_rc::<crate::ui::workbench::asset_content_layout::AssetContentPaintMetadata>()
        .expect("first Asset Browser paint metadata");
    let next_metadata = next
        .metadata_rc::<crate::ui::workbench::asset_content_layout::AssetContentPaintMetadata>()
        .expect("next Asset Browser paint metadata");

    assert!(first_metadata.shares_browser_logical_item_chunk_with(0, &next_metadata));
    assert!(!first_metadata.shares_browser_logical_item_chunk_with(65, &next_metadata));
    assert!(first_metadata.shares_browser_logical_item_chunk_with(129, &next_metadata));
}

#[cfg(test)]
fn test_asset_item(uuid: &str) -> AssetItemSnapshot {
    AssetItemSnapshot {
        uuid: uuid.to_string(),
        locator: format!("res://{uuid}.mesh"),
        display_name: format!("{uuid}.mesh"),
        file_name: format!("{uuid}.mesh"),
        extension: "mesh".to_string(),
        kind: ResourceKind::Mesh,
        asset_type: crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
            ResourceKind::Mesh,
        ),
        preview_artifact_path: String::new(),
        dirty: false,
        diagnostics: Vec::new(),
        selected: false,
        resource_state: None,
        resource_revision: Some(1),
    }
}

fn apply_asset_browser_visual_state(
    nodes: &mut [ViewTemplateNodeData],
    snapshot: &AssetWorkspaceSnapshot,
) {
    apply_asset_browser_kind_filter_state(nodes, snapshot.kind_filter);
    mark_toggle_state(
        nodes,
        "AssetBrowserViewModeListButton",
        snapshot.view_mode == AssetViewMode::List,
    );
    mark_toggle_state(
        nodes,
        "AssetBrowserViewModeThumbButton",
        snapshot.view_mode == AssetViewMode::Thumbnail,
    );
    mark_utility_tab_state(
        nodes,
        "AssetBrowserPreviewTabButton",
        snapshot.utility_tab == AssetUtilityTab::Preview,
    );
    mark_utility_tab_state(
        nodes,
        "AssetBrowserReferencesTabButton",
        snapshot.utility_tab == AssetUtilityTab::References,
    );
    mark_utility_tab_state(
        nodes,
        "AssetBrowserMetadataTabButton",
        snapshot.utility_tab == AssetUtilityTab::Metadata,
    );
    mark_utility_tab_state(
        nodes,
        "AssetBrowserPluginsTabButton",
        snapshot.utility_tab == AssetUtilityTab::Plugins,
    );
    apply_asset_browser_utility_tab_typography(nodes);

    mark_asset_table_rows(nodes, snapshot);

    let has_selection = has_asset_selection(snapshot);
    let selected_asset = selected_asset(snapshot);
    let has_content_preview = selected_asset.is_some();
    update_panel_surface(
        nodes,
        "AssetBrowserContentPreviewCard",
        if has_content_preview {
            "asset-preview"
        } else {
            "asset-placeholder"
        },
        if has_content_preview { 1.0 } else { 0.0 },
    );
    update_panel_surface(
        nodes,
        "AssetBrowserContentPreviewVisual",
        if has_content_preview {
            "asset-preview-visual"
        } else {
            "asset-placeholder-visual"
        },
        if has_content_preview { 1.0 } else { 0.0 },
    );
    update_asset_preview_visual_icon(nodes, "AssetBrowserContentPreviewVisual", selected_asset);
    mark_panel_selected(nodes, "AssetBrowserContentPreviewCard", has_content_preview);
    update_panel_surface(
        nodes,
        "AssetBrowserDetailsPreviewPanel",
        "asset-placeholder",
        0.0,
    );
    update_panel_surface(
        nodes,
        "AssetBrowserDetailsPreviewVisualPanel",
        "asset-placeholder-visual",
        0.0,
    );
    update_asset_preview_visual_icon(
        nodes,
        "AssetBrowserDetailsPreviewVisualPanel",
        selected_asset,
    );
    update_panel_surface(
        nodes,
        "AssetBrowserPreviewPanel",
        if has_selection {
            "asset-preview"
        } else {
            "asset-placeholder"
        },
        if has_selection { 1.0 } else { 0.0 },
    );
    update_panel_surface(
        nodes,
        "AssetBrowserPreviewVisualPanel",
        if has_selection {
            "asset-preview-visual"
        } else {
            "asset-placeholder-visual"
        },
        if has_selection { 1.0 } else { 0.0 },
    );
    update_asset_preview_visual_icon(nodes, "AssetBrowserPreviewVisualPanel", selected_asset);
    mark_panel_selected(
        nodes,
        "AssetBrowserPreviewPanel",
        has_selection && snapshot.utility_tab == AssetUtilityTab::Preview,
    );
    mark_panel_group_selected(
        nodes,
        &[
            "AssetBrowserReferenceLeftPanel",
            "AssetBrowserReferenceRightPanel",
        ],
        snapshot.utility_tab == AssetUtilityTab::References,
    );
    mark_panel_group_selected(
        nodes,
        &[
            "AssetBrowserMetaPathPanel",
            "AssetBrowserToolkitPanel",
            "AssetBrowserDiagnosticsPanel",
        ],
        snapshot.utility_tab == AssetUtilityTab::Metadata,
    );
    mark_panel_selected(
        nodes,
        "AssetBrowserPluginsPanel",
        snapshot.utility_tab == AssetUtilityTab::Plugins,
    );

    if snapshot.selection.diagnostics.is_empty() {
        update_panel_variant(nodes, "AssetBrowserDetailsDiagnosticsPanel", "inset");
    }
}

fn apply_asset_browser_kind_filter_state(
    nodes: &mut [ViewTemplateNodeData],
    kind_filter: Option<ResourceKind>,
) {
    let (selected_label, options) = super::asset_kind_filter_options(kind_filter);

    if let Some(node) = nodes
        .iter_mut()
        .find(|node| node.control_id == super::ASSET_BROWSER_KIND_FILTER_CONTROL_ID)
    {
        node.value_text = selected_label.into();
        node.options = model_rc(options);
    }
}

fn update_panel_variant(
    nodes: &mut [ViewTemplateNodeData],
    control_id: &str,
    surface_variant: &str,
) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.surface_variant = surface_variant.into();
    }
}

fn update_panel_surface(
    nodes: &mut [ViewTemplateNodeData],
    control_id: &str,
    surface_variant: &str,
    border_width: f32,
) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.surface_variant = surface_variant.into();
        node.border_width = border_width;
    }
}

fn update_asset_preview_visual_icon(
    nodes: &mut [ViewTemplateNodeData],
    control_id: &str,
    asset: Option<&AssetItemSnapshot>,
) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        if let Some(asset) = asset {
            node.component_role = "asset-thumbnail-visual".into();
            node.component_variant = asset.asset_type.icon_name.clone().into();
            update_asset_preview_visual_image(node, asset);
        } else {
            node.component_role = "".into();
            node.component_variant = "".into();
            clear_asset_preview_visual_image(node);
        }
    }
}

fn update_asset_preview_visual_image(node: &mut ViewTemplateNodeData, asset: &AssetItemSnapshot) {
    node.media_source = asset.preview_artifact_path.clone().into();
    node.has_preview_image = !asset.preview_artifact_path.trim().is_empty();
    node.preview_image = Default::default();
}

fn clear_asset_preview_visual_image(node: &mut ViewTemplateNodeData) {
    node.media_source = "".into();
    node.has_preview_image = false;
    node.preview_image = Default::default();
}

fn retain_active_utility_tab_nodes(
    nodes: &mut Vec<ViewTemplateNodeData>,
    active_tab: AssetUtilityTab,
) {
    nodes.retain(
        |node| match utility_tab_for_control_id(node.control_id.as_str()) {
            Some(tab) => tab == active_tab,
            None => true,
        },
    );
}

fn utility_tab_for_control_id(control_id: &str) -> Option<AssetUtilityTab> {
    if matches!(
        control_id,
        "AssetBrowserPreviewPanel"
            | "AssetBrowserPreviewVisualPanel"
            | "AssetBrowserPreviewNameText"
            | "AssetBrowserPreviewLocatorText"
            | "AssetBrowserPreviewKindText"
            | "AssetBrowserPreviewIdentityText"
            | "AssetBrowserPreviewToolkitText"
            | "AssetBrowserPreviewMetaPathText"
            | "AssetBrowserPreviewDiagnosticsText"
    ) {
        return Some(AssetUtilityTab::Preview);
    }

    if control_id.starts_with("AssetBrowserReferenceLeft")
        || control_id.starts_with("AssetBrowserReferenceRight")
    {
        return Some(AssetUtilityTab::References);
    }

    if control_id.starts_with("AssetBrowserMetaPath")
        || control_id.starts_with("AssetBrowserToolkit")
        || matches!(
            control_id,
            "AssetBrowserDiagnosticsPanel"
                | "AssetBrowserDiagnosticsLabel"
                | "AssetBrowserDiagnosticsText"
        )
    {
        return Some(AssetUtilityTab::Metadata);
    }

    if matches!(
        control_id,
        "AssetBrowserPluginsPanel" | "AssetBrowserPluginsText"
    ) {
        return Some(AssetUtilityTab::Plugins);
    }

    None
}

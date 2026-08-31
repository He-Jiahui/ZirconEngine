from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
ASSET_BROWSER = ROOT / "zircon_editor/src/ui/layouts/views/asset_browser.rs"
LOGICAL_PAINT_SOURCE = ROOT / (
    "zircon_editor/src/ui/layouts/views/asset_browser/logical_paint_source.rs"
)
ASSET_WORKSPACE_STATE = ROOT / (
    "zircon_editor/src/ui/workbench/project/asset_workspace_state.rs"
)
ASSET_WORKSPACE_SNAPSHOT = ROOT / (
    "zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_snapshot.rs"
)
ASSET_ITEM_GENERATION = ROOT / (
    "zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_item_generation.rs"
)
ASSET_BROWSER_VIRTUALIZATION = ROOT / (
    "zircon_editor/src/ui/workbench/asset_content_layout/browser_virtualization.rs"
)
ASSET_ACCESS = ROOT / (
    "zircon_editor/src/ui/host/editor_event_runtime_access/asset_access.rs"
)
WORKBENCH_SHELL_STATE = ROOT / "zircon_editor/src/ui/workbench/shell_state.rs"
UI_PERF = ROOT / "zircon_editor/src/ui/retained_host/ui_perf.rs"
PROFILE_COUNTER_EVIDENCE = ROOT / "tools/ui-profile-counter-evidence.ps1"
TABLE_NODES = ROOT / (
    "zircon_editor/src/ui/layouts/views/asset_browser/table_nodes.rs"
)
TABLE_LAYOUT = ROOT / (
    "zircon_editor/src/ui/layouts/views/asset_browser/compact_table_layout.rs"
)
THUMBNAIL_LAYOUT = ROOT / (
    "zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_layout.rs"
)


def production_source(path: Path) -> str:
    return path.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]


class EditorAssetBrowserProjectionComplexityContract(unittest.TestCase):
    def test_workspace_selection_reuses_an_immutable_generation_owned_item_source(self) -> None:
        state = ASSET_WORKSPACE_STATE.read_text(encoding="utf-8")
        snapshot = ASSET_WORKSPACE_SNAPSHOT.read_text(encoding="utf-8")
        generation = ASSET_ITEM_GENERATION.read_text(encoding="utf-8")

        self.assertIn("AssetWorkspaceItemGeneration", snapshot)
        self.assertIn("chunks: Arc<[Arc<[AssetItemSnapshot]>]>", generation)
        self.assertIn("indices_by_uuid", generation)
        self.assertIn("indices_by_locator", generation)
        self.assertIn("item_generation_cache", state)
        cache_input = state.split("struct AssetWorkspaceItemProjectionInput", 1)[1].split(
            "struct AssetWorkspaceState", 1
        )[0]
        self.assertNotIn("selected_asset_uuid", cache_input)
        self.assertIn("self.visible_asset_generation(catalog", state)

    def test_registry_projection_is_cached_by_shared_item_generation(self) -> None:
        access = ASSET_ACCESS.read_text(encoding="utf-8")
        shell = WORKBENCH_SHELL_STATE.read_text(encoding="utf-8")
        generation = ASSET_ITEM_GENERATION.read_text(encoding="utf-8")

        self.assertNotIn("for item in &mut snapshot.visible_assets", access)
        self.assertIn("projected_asset_items", access)
        self.assertIn("project_items", generation)
        self.assertIn("project_items_reusing", generation)
        self.assertIn("previous_asset_item_projection", access)
        self.assertIn("Arc::clone(&self.indices_by_uuid)", generation)
        self.assertIn("cached_asset_item_projection", shell)
        self.assertIn("shares_items_with", shell)

    def test_selection_changes_reuse_the_generation_owned_logical_paint_source(self) -> None:
        source = LOGICAL_PAINT_SOURCE.read_text(encoding="utf-8")

        self.assertIn("ASSET_BROWSER_LOGICAL_PAINT_CACHE", source)
        self.assertIn("AssetBrowserLogicalPaintGeneration", source)
        cache_input = source.split("struct AssetBrowserLogicalPaintInput", 1)[1].split(
            "thread_local!", 1
        )[0]
        self.assertNotIn("selected_asset_uuid", cache_input)

    def test_sparse_selection_fallback_uses_the_generation_index_without_scanning_items(
        self,
    ) -> None:
        source = LOGICAL_PAINT_SOURCE.read_text(encoding="utf-8")
        generation = ASSET_ITEM_GENERATION.read_text(encoding="utf-8")
        selection = source.split("pub(super) fn selected_asset_item_indices", 1)[1]
        selection = selection.split("#[cfg(test)]", 1)[0]

        self.assertIn("selected_indices: Arc<[usize]>", generation)
        self.assertIn("pub(crate) fn selected_indices(&self) -> &[usize]", generation)
        self.assertIn("snapshot.visible_assets.selected_indices().to_vec()", selection)
        self.assertNotIn(".iter()", selection)
        self.assertNotIn(".enumerate()", selection)

        replacement = generation.split("pub(crate) fn replace_existing_items", 1)[1]
        replacement = replacement.split("pub(crate) fn project_items", 1)[0]
        reuse = generation.split("pub(crate) fn project_items_reusing", 1)[1]
        reuse = reuse.split("pub(crate) fn shares_item_chunk_with", 1)[0]
        self.assertIn("update_selected_index", replacement)
        self.assertIn("replace_chunk_selected_indices", reuse)
        self.assertNotIn("selected_indices_from_chunks", reuse)
        self.assertIn(
            "fn sparse_selection_index_tracks_local_item_replacements_without_a_source_scan",
            source,
        )
        self.assertIn(
            "fn sparse_selection_index_uses_the_last_duplicate_replacement",
            source,
        )
        self.assertIn(
            "fn sparse_selection_index_updates_only_reprojected_chunks",
            source,
        )
        self.assertIn(
            "fn uuid_selection_overrides_and_missing_uuid_falls_back_to_sparse_flags",
            source,
        )

    def test_local_asset_delta_reuses_unchanged_logical_paint_chunks(self) -> None:
        source = LOGICAL_PAINT_SOURCE.read_text(encoding="utf-8")
        virtualization = ASSET_BROWSER_VIRTUALIZATION.read_text(encoding="utf-8")

        self.assertIn("AssetBrowserLogicalPaintGeneration", virtualization)
        self.assertIn("chunks: Rc<[Rc<[AssetBrowserPaintItem]>]>", virtualization)
        self.assertIn("chunk_size: 1", virtualization)
        self.assertNotIn("Clone, Debug, Default", virtualization)
        self.assertIn("shares_item_chunk_with", source)
        self.assertIn("cloned_chunk", source)
        self.assertNotIn("Rc<Vec<AssetBrowserPaintItem>>", source)
        self.assertNotIn("items: Rc<Vec<AssetBrowserPaintItem>>", virtualization)

    def test_logical_paint_chunk_reuse_is_measurable_and_budget_gated(self) -> None:
        source = LOGICAL_PAINT_SOURCE.read_text(encoding="utf-8")
        ui_perf = UI_PERF.read_text(encoding="utf-8")
        gate = PROFILE_COUNTER_EVIDENCE.read_text(encoding="utf-8")

        self.assertIn("record_current_ui_perf_counter_batch", source)
        for counter in (
            "AssetBrowserLogicalPaintChunkBuildCount",
            "AssetBrowserLogicalPaintChunkReuseCount",
            "AssetBrowserLogicalPaintItemProjectionCount",
        ):
            self.assertIn(counter, source)
            self.assertIn(counter, ui_perf)
        self.assertIn("asset_browser_logical_paint_chunk_build_count", gate)
        self.assertIn("asset_browser_logical_paint_chunk_reuse_count", gate)
        self.assertIn("asset_browser_logical_paint_item_projection_count", gate)
        self.assertIn("64 * $paintChunkBuildCount", gate)

    def test_table_projection_updates_rows_in_single_node_passes(self) -> None:
        source = production_source(TABLE_NODES)

        self.assertIn("existing_row_indices", source)
        self.assertIn("vec![false; asset_count]", source)
        self.assertIn("for node in nodes.iter_mut()", source)
        self.assertNotIn("nodes.iter_mut().find", source)
        self.assertNotIn("nodes\n            .iter()\n            .any", source)

    def test_table_layout_does_not_rescan_nodes_for_each_logical_row(self) -> None:
        source = production_source(TABLE_LAYOUT)

        self.assertIn("for node in nodes.iter_mut()", source)
        self.assertNotIn("fn set_frame(", source)
        self.assertNotIn("for index in 0..row_count", source)

    def test_thumbnail_layout_indexes_each_materialized_node_once(self) -> None:
        source = THUMBNAIL_LAYOUT.read_text(encoding="utf-8").split(
            "pub(super) fn apply_compact_thumbnail_grid_layout", 1
        )[1].split("#[cfg(test)]\nmod tests", 1)[0]

        self.assertIn("ThumbnailLayoutInput", source)
        self.assertIn("for node in nodes.iter_mut()", source)
        self.assertNotIn("fn set_node_frame(", source)
        self.assertNotIn("nodes.iter_mut().find", source)
        self.assertNotIn("for index in 0..count", source)

    def test_each_mode_builds_one_logical_paint_source_without_a_duplicate_table_vector(self) -> None:
        source = ASSET_BROWSER.read_text(encoding="utf-8").split(
            "pub(crate) fn asset_browser_pane_data", 1
        )[1].split("pub(crate) fn asset_browser_pane_nodes", 1)[0]
        logical_source = LOGICAL_PAINT_SOURCE.read_text(encoding="utf-8")

        self.assertIn("asset_browser_logical_paint_items(snapshot", source)
        self.assertIn("project_paint_item(snapshot.view_mode, asset)", logical_source)
        self.assertIn("AssetBrowserLogicalPaintGeneration::from_chunks", logical_source)
        self.assertNotIn("let asset_table_rows", source)
        self.assertNotIn("asset_table_rows(snapshot)", source)


if __name__ == "__main__":
    unittest.main()

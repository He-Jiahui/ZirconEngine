import pathlib
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
ASSET_ACCESS = REPO_ROOT / "zircon_editor/src/ui/host/editor_event_runtime_access/asset_access.rs"
SNAPSHOT_SYNC = REPO_ROOT / "zircon_editor/src/ui/retained_host/app/assets/refresh/snapshots.rs"
REFRESH_APPLY = REPO_ROOT / "zircon_editor/src/ui/retained_host/app/assets/refresh/apply.rs"
REFRESH_ROOT = REPO_ROOT / "zircon_editor/src/ui/retained_host/app/assets/refresh.rs"
WORKSPACE_STATE = REPO_ROOT / "zircon_editor/src/ui/workbench/project/asset_workspace_state.rs"
ITEM_GENERATION = REPO_ROOT / "zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_item_generation.rs"


def function_body(source: str, signature: str, next_signature: str) -> str:
    return source.split(signature, 1)[1].split(next_signature, 1)[0]


class EditorAssetRefreshInvalidationAuthorityContract(unittest.TestCase):
    def test_runtime_exposes_data_only_asset_generation_publication(self) -> None:
        source = ASSET_ACCESS.read_text(encoding="utf-8")
        catalog_data = function_body(
            source,
            "pub(crate) fn sync_asset_catalog_data(",
            "pub fn sync_asset_resources(",
        )
        resource_data = function_body(
            source,
            "pub(crate) fn sync_asset_resources_data(",
            "pub fn sync_asset_details(",
        )

        self.assertIn("state.sync_asset_catalog(catalog)", catalog_data)
        self.assertNotIn("refresh_workbench", catalog_data)
        self.assertIn("state.sync_asset_resources(resources)", resource_data)
        self.assertNotIn("refresh_workbench", resource_data)

    def test_retained_snapshot_sync_does_not_publish_generic_presentation_dirty(self) -> None:
        source = SNAPSHOT_SYNC.read_text(encoding="utf-8")
        catalog_sync = function_body(
            source,
            "fn sync_asset_catalog_snapshot(",
            "pub(in crate::ui::retained_host::app) fn sync_asset_resources(",
        )
        resource_sync = function_body(
            source,
            "fn sync_asset_resources_snapshot(",
            "pub(in crate::ui::retained_host::app) fn refresh_selected_asset_details(",
        )

        self.assertIn("sync_asset_catalog_data", catalog_sync)
        self.assertNotIn(".sync_asset_catalog(", catalog_sync)
        self.assertIn("sync_asset_resources_data", resource_sync)
        self.assertNotIn(".sync_asset_resources(", resource_sync)

    def test_refresh_plan_remains_the_invalidation_authority(self) -> None:
        source = REFRESH_APPLY.read_text(encoding="utf-8")
        apply_plan = function_body(
            source,
            "fn apply_asset_refresh_plan(",
            "fn apply_asset_refresh_invalidation(",
        )

        self.assertIn("self.apply_asset_refresh_invalidation(plan)", apply_plan)

    def test_exact_asset_and_resource_changes_reach_workspace_sync(self) -> None:
        refresh = REFRESH_ROOT.read_text(encoding="utf-8")
        apply = REFRESH_APPLY.read_text(encoding="utf-8")
        snapshots = SNAPSHOT_SYNC.read_text(encoding="utf-8")

        self.assertIn("self.apply_asset_refresh_plan(&plan, &events)", refresh)
        self.assertIn("events: &AssetRefreshEvents", apply)
        self.assertIn("sync_asset_catalog_snapshot(&events.editor_asset_changes)", apply)
        self.assertIn("sync_asset_resources_snapshot(", apply)
        self.assertIn("&events.resource_changes", apply)
        self.assertIn("events.resource_generation_lagged", apply)
        self.assertIn("sync_asset_catalog_changes", snapshots)
        self.assertIn("sync_asset_resource_changes", snapshots)

    def test_item_generation_patches_immutable_chunks_without_full_payload_clone(self) -> None:
        generation = ITEM_GENERATION.read_text(encoding="utf-8")
        workspace = WORKSPACE_STATE.read_text(encoding="utf-8")

        self.assertIn("ASSET_WORKSPACE_ITEM_CHUNK_SIZE", generation)
        self.assertIn("chunks: Arc<[Arc<[AssetItemSnapshot]>]>", generation)
        self.assertIn("replace_existing_items", generation)
        replace_body = function_body(
            generation,
            "fn replace_existing_items(",
            "fn project_items(",
        )
        self.assertNotIn("self.iter().cloned().collect", replace_body)
        self.assertNotIn("self.items.to_vec", replace_body)
        self.assertIn("sync_catalog_changes", workspace)
        self.assertIn("sync_resource_changes", workspace)

    def test_visual_cache_full_invalidation_is_reserved_for_sprite_atlas_sources(self) -> None:
        source = REFRESH_ROOT.read_text(encoding="utf-8")
        selector = function_body(
            source,
            "fn visual_asset_cache_refresh(",
            "fn events_reference_sprite_atlas(",
        )

        self.assertIn("if events_reference_sprite_atlas(events)", selector)
        self.assertIn("return VisualAssetCacheRefresh::All;", selector)
        self.assertIn("VisualAssetCacheRefresh::Reconcile", selector)
        self.assertIn("VisualAssetCacheRefresh::None", selector)
        self.assertIn("VisualAssetCacheRefresh::Paths(paths)", selector)
        self.assertIn(
            "fn unlocated_runtime_texture_does_not_invalidate_file_backed_visual_assets",
            source,
        )

    def test_targeted_visual_refresh_invalidates_pixels_and_svg_trees_by_path(self) -> None:
        source = REFRESH_ROOT.read_text(encoding="utf-8")
        refresh = function_body(
            source,
            "pub(in crate::ui::retained_host::app) fn refresh_project_assets(",
            "#[derive(Debug, PartialEq, Eq)]",
        )
        targeted = refresh.split("VisualAssetCacheRefresh::Paths(paths)", 1)[1]
        targeted = targeted.split("VisualAssetCacheRefresh::Reconcile", 1)[0]

        self.assertIn("invalidate_visual_asset_pixel_paths", targeted)
        self.assertIn("invalidate_svg_tree_paths", targeted)
        self.assertNotIn("clear_visual_asset_pixels_cache", targeted)
        self.assertNotIn("clear_svg_tree_cache", targeted)


if __name__ == "__main__":
    unittest.main()

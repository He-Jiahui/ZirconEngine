from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
STATE_WRITEBACK = (
    ROOT
    / "zircon_editor/src/ui/retained_host/app/asset_content_pointer/target/state.rs"
)
PAINT_METADATA = ROOT / (
    "zircon_editor/src/ui/workbench/asset_content_layout/paint_metadata.rs"
)
PAINT_IDENTITY = ROOT / (
    "zircon_editor/src/ui/workbench/asset_content_layout/identity.rs"
)
ASSET_POINTER_ROOT = ROOT / "zircon_editor/src/ui/retained_host"
ASSET_BRIDGES = (
    ASSET_POINTER_ROOT / "asset_pointer/content/bridge.rs",
    ASSET_POINTER_ROOT / "asset_pointer/tree/bridge.rs",
    ASSET_POINTER_ROOT / "asset_pointer/reference/bridge.rs",
)
ASSET_POINTER_SYNC = (
    ASSET_POINTER_ROOT / "app/pointer_layout/asset_surfaces/sync.rs"
)
CONTENT_TARGET_DISPATCH = (
    ASSET_POINTER_ROOT / "app/asset_content_pointer/target/dispatch.rs"
)
TREE_TARGET = ASSET_POINTER_ROOT / "app/asset_tree_pointer/target.rs"
REFERENCE_TARGET_DISPATCH = (
    ASSET_POINTER_ROOT / "app/asset_reference_pointer/target/dispatch.rs"
)
HOST_RECOMPUTE = ASSET_POINTER_ROOT / "app/host_lifecycle/recompute.rs"
POINTER_SURFACES = (
    ASSET_POINTER_ROOT / "app/host_lifecycle/recompute/pointer_surfaces.rs"
)
COUNTER_CATALOG = ASSET_POINTER_ROOT / "ui_perf/counter_catalog.rs"
UI_PERF = ASSET_POINTER_ROOT / "ui_perf.rs"
COUNTER_GATE = ROOT / "tools/ui-profile-counter-evidence.ps1"


class EditorAssetContentPointerPerformanceContractTests(unittest.TestCase):
    def test_unchanged_content_pointer_state_skips_ui_property_writeback(self) -> None:
        source = STATE_WRITEBACK.read_text(encoding="utf-8")

        compare = source.index("surface.content_state == state")
        assignment = source.index("surface.content_state = state")
        writeback = source.index("self.apply_asset_pointer_state_to_ui(surface_mode)")

        self.assertLess(compare, assignment)
        self.assertLess(assignment, writeback)
        self.assertIn("return;", source[compare:assignment])

    def test_paint_metadata_parses_borrowed_control_ids_without_identity_index(self) -> None:
        source = PAINT_METADATA.read_text(encoding="utf-8")
        identity_source = PAINT_IDENTITY.read_text(encoding="utf-8")
        metadata = source.split("pub(crate) struct AssetContentPaintMetadata", 1)[1]
        build = metadata.split("fn build", 1)[1].split("fn finish", 1)[0]
        wrapper = source.split("pub(crate) fn asset_content_paint_metadata", 1)[1]

        self.assertNotIn("identities: BTreeMap<String", metadata)
        self.assertNotIn("identity_index", build)
        self.assertNotIn("control_id.to_owned()", build)
        self.assertNotIn("collect::<Vec<_>>()", wrapper)
        self.assertIn("I: Iterator<Item = AssetContentPaintNodeInput<'a>> + Clone", metadata)
        self.assertIn(
            "describe_asset_content_row(surface, node.control_id)",
            build,
        )
        self.assertIn("identity_parse_count: row_descriptors.len()", build)
        self.assertIn("pub(crate) fn parse_activity_content_identity", identity_source)
        self.assertIn("pub(crate) fn parse_browser_content_identity", identity_source)

    def test_asset_pointer_bridges_patch_size_without_reprojecting_items(self) -> None:
        for path in ASSET_BRIDGES:
            source = path.read_text(encoding="utf-8")
            pane_patch = source.split("fn sync_pane_size", 1)[1].split("fn handle_", 1)[0]

            self.assertIn("self.layout.pane_size = pane_size", pane_patch, path)
            self.assertIn("self.clamp_scroll_offset()", pane_patch, path)
            self.assertIn("self.patch_surface_geometry()", pane_patch, path)
            self.assertNotIn("from_snapshot", pane_patch, path)
            self.assertNotIn("from_references", pane_patch, path)
            self.assertIn(
                "pane_size_patch_preserves", source, f"missing retained-data regression in {path}"
            )

    def test_pointer_callback_size_changes_use_geometry_only_bridge_patches(self) -> None:
        for path in (CONTENT_TARGET_DISPATCH, TREE_TARGET, REFERENCE_TARGET_DISPATCH):
            source = path.read_text(encoding="utf-8")

            self.assertIn("sync_pane_size", source, path)
            self.assertNotIn("from_snapshot", source, path)
            self.assertNotIn("asset_reference_layout", source, path)

    def test_window_metrics_recompute_skips_asset_snapshot_projection(self) -> None:
        recompute = HOST_RECOMPUTE.read_text(encoding="utf-8")
        pointer_surfaces = POINTER_SURFACES.read_text(encoding="utf-8")
        asset_sync = ASSET_POINTER_SYNC.read_text(encoding="utf-8")

        self.assertIn("window_metrics_target", recompute)
        self.assertIn("sync_recompute_pointer_surfaces(", recompute)
        self.assertIn("window_metrics_target", pointer_surfaces)
        self.assertIn("sync_asset_pointer_geometries", pointer_surfaces)
        self.assertIn("sync_asset_pointer_layouts", pointer_surfaces)
        self.assertIn("fn sync_asset_pointer_geometries", asset_sync)
        geometry_only = asset_sync.split("fn sync_asset_pointer_geometries", 1)[1]
        self.assertNotIn("snapshot.clone()", geometry_only)
        self.assertNotIn("from_snapshot", geometry_only)
        self.assertNotIn("from_references", geometry_only)

    def test_resize_gate_rejects_asset_snapshot_projection(self) -> None:
        counter_catalog = COUNTER_CATALOG.read_text(encoding="utf-8")
        ui_perf = UI_PERF.read_text(encoding="utf-8")
        asset_sync = ASSET_POINTER_SYNC.read_text(encoding="utf-8")
        gate = COUNTER_GATE.read_text(encoding="utf-8")

        self.assertIn("AssetPointerSnapshotCloneCount", counter_catalog)
        self.assertIn(
            'concat!($prefix, ".asset_pointer_snapshot_clone_count")', ui_perf
        )
        self.assertIn(
            "UiPerfCounter::AssetPointerSnapshotCloneCount", asset_sync
        )
        self.assertIn(
            '"ui.window_resize.asset_pointer_snapshot_clone_count"', gate
        )
        self.assertIn("$assetPointerSnapshotCloneCount -eq 0", gate)


if __name__ == "__main__":
    unittest.main()

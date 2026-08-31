import importlib.util
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
TOOL = ROOT / "tools" / "editor_asset_pointer_generation_pressure.py"
LAYOUT = ROOT / "zircon_editor/src/ui/retained_host/asset_pointer/content/layout.rs"
BRIDGE = ROOT / "zircon_editor/src/ui/retained_host/asset_pointer/content/bridge.rs"
GENERATION = ROOT / (
    "zircon_editor/src/ui/workbench/snapshot/asset/"
    "asset_workspace_item_generation.rs"
)
UNREAL_ASSET_VIEW = ROOT / (
    "dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/SAssetView.cpp"
)


spec = importlib.util.spec_from_file_location("asset_pointer_pressure", TOOL)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = module
spec.loader.exec_module(module)


class AssetPointerGenerationPressureTests(unittest.TestCase):
    def test_pointer_layout_retains_the_published_item_generation(self):
        layout = LAYOUT.read_text(encoding="utf-8")
        bridge = BRIDGE.read_text(encoding="utf-8")
        self.assertIn("pub items: AssetWorkspaceItemGeneration", layout)
        self.assertIn("items: snapshot.visible_assets.clone()", layout)
        self.assertNotIn("pub item_ids: Vec<String>", layout)
        self.assertNotIn(".map(|item| item.uuid.clone())", layout)
        self.assertIn("self.layout.item_uuid(item_index)", bridge)

    def test_generation_publishes_constant_time_item_order_identity(self):
        generation = GENERATION.read_text(encoding="utf-8")
        self.assertIn("pub fn len(&self) -> usize", generation)
        self.assertIn(
            "pub fn get(&self, index: usize) -> Option<&AssetItemSnapshot>", generation
        )
        identity = generation.split("pub(crate) fn shares_item_identity_with", 1)[1].split(
            "pub(crate) fn replace_existing_items", 1
        )[0]
        self.assertIn("Arc::ptr_eq(&self.indices_by_uuid, &other.indices_by_uuid)", identity)
        self.assertNotIn(".iter()", identity)

    def test_stable_bridge_sync_refreshes_the_handle_without_rebuilding_geometry(self):
        layout = LAYOUT.read_text(encoding="utf-8")
        bridge = BRIDGE.read_text(encoding="utf-8")
        equality = layout.split("impl PartialEq for AssetContentListPointerLayout", 1)[1]
        self.assertIn("self.items.shares_item_identity_with(&other.items)", equality)
        self.assertNotIn("self.items.iter()", equality)
        unchanged = bridge.split("pub(crate) fn sync(", 1)[1].split(
            "pub(crate) fn sync_pane_size", 1
        )[0]
        compare = unchanged.index("self.layout == layout && self.state == state")
        refresh = unchanged.index("self.layout = layout")
        early_return = unchanged.index("return false")
        self.assertLess(compare, refresh)
        self.assertLess(refresh, early_return)

    def test_stable_sync_model_removes_uuid_vector_republication(self):
        report = module.pressure_report(100_000, 256, 1_000, 10_000)
        self.assertEqual(report["current"]["uuid_payload_clones"], 200_000_000)
        self.assertEqual(report["target"]["uuid_payload_clones"], 0)
        self.assertEqual(report["target"]["generation_arc_handle_clones"], 10_000)
        self.assertAlmostEqual(
            report["ratios"]["item_identity_operation_units"],
            400_000_000 / 12_000,
        )

    def test_hit_route_keeps_one_owned_uuid_per_real_hit(self):
        report = module.pressure_report(100_000, 256, 1_000, 10_000)
        self.assertEqual(
            report["current"]["routed_hit_uuid_payload_clones"],
            report["target"]["routed_hit_uuid_payload_clones"],
        )

    def test_unreal_views_share_one_filtered_item_source(self):
        unreal = UNREAL_ASSET_VIEW.read_text(encoding="utf-8")
        self.assertGreaterEqual(
            unreal.count(".ListItemsSource(&FilteredAssetItems)"), 3
        )
        self.assertIn("ListView->RequestListRefresh()", unreal)
        self.assertIn("TileView->RequestListRefresh()", unreal)


if __name__ == "__main__":
    unittest.main()

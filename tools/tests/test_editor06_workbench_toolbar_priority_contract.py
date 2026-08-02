from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PRIORITY = ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/toolbar_layout/priority.rs"
)
TEMPLATE_SURFACE = ROOT / (
    "zircon_editor/src/ui/workbench/reference/template_surface.rs"
)
COMPONENTIZED_WINDOW = ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/componentized_window.rs"
)
ASSET_CREATION_MENU = ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/asset_creation_menu.rs"
)
ASSET_TYPE_REGISTRY = ROOT / "zircon_editor/src/core/asset/type_registry/registry.rs"
WORKBENCH_SLOT = ROOT / "zircon_editor/src/core/extension/slots.rs"
VIEW_DESCRIPTOR = ROOT / "zircon_editor/src/ui/workbench/view/view_descriptor.rs"
VIEW_BUILDER = ROOT / "zircon_editor/src/ui/workbench/view/view_descriptor_builder.rs"
VIEW_MAPPING = ROOT / (
    "zircon_editor/src/ui/workbench/view/workbench_slot_to_view_host.rs"
)
LEGACY_PREFERRED_HOST = ROOT / "zircon_editor/src/ui/workbench/view/preferred_host.rs"


class Editor06WorkbenchToolbarPriorityContractTests(unittest.TestCase):
    def test_toolbar_width_resolution_indexes_control_nodes_once(self) -> None:
        source = PRIORITY.read_text(encoding="utf-8")

        self.assertIn("ToolbarControlIndex", source)
        self.assertIn("ToolbarControlIndex::new(surface)", source)
        self.assertEqual(source.count(".values()"), 1)
        self.assertNotIn("surface_control_node_id", source)

    def test_componentized_workbench_reuses_a_stable_control_node_index(self) -> None:
        source = TEMPLATE_SURFACE.read_text(encoding="utf-8")

        self.assertIn("control_nodes: HashMap<String, UiNodeId>", source)
        self.assertIn("build_control_node_index(&surface)", source)
        self.assertIn("self.control_node_id(control_id)", source)
        self.assertIn("DuplicateControl", source)
        self.assertNotIn("find_control_node_id", source)
        self.assertNotIn("fn control_frame(surface: &UiSurface, control_id: &str)", source)
        self.assertNotIn("fn visible_control_frame(surface: &UiSurface, control_id: &str)", source)

    def test_asset_menu_layout_and_property_access_use_control_slots(self) -> None:
        componentized = COMPONENTIZED_WINDOW.read_text(encoding="utf-8")
        asset_menu = ASSET_CREATION_MENU.read_text(encoding="utf-8")

        self.assertIn("self.control_node_id(control_id)", componentized)
        self.assertIn("self.control_node_id(MAIN_MENU_CONTROL_ID)", asset_menu)
        self.assertNotIn("tree.nodes.values().find_map", asset_menu)

    def test_asset_menu_uses_one_immutable_generation_and_indexed_actions(self) -> None:
        registry = ASSET_TYPE_REGISTRY.read_text(encoding="utf-8")
        asset_menu = ASSET_CREATION_MENU.read_text(encoding="utf-8")
        request = asset_menu.split("pub(crate) fn asset_creation_menu_request", 1)[1].split(
            "pub(crate) fn is_asset_creation_menu_action", 1
        )[0]

        self.assertIn("creation_menu: Arc<AssetCreationMenuGeneration>", registry)
        self.assertIn("action_index: Arc<HashMap<String, usize>>", registry)
        self.assertIn("action_index.insert(action_id, index)", registry)
        self.assertIn("self.action_index", registry)
        self.assertIn("next_suffix_by_base", registry)
        self.assertIn(".entry(label.clone()).or_insert(2)", registry)
        self.assertNotIn("while !used_labels.insert", registry)
        self.assertIn("Arc::ptr_eq(current, generation)", asset_menu)
        self.assertIn("generation.action(action_id)", request)
        self.assertNotIn("HashMap", request)
        self.assertNotIn("collect", request)

    def test_workbench_slot_is_the_only_persistable_declaration(self) -> None:
        slot = WORKBENCH_SLOT.read_text(encoding="utf-8")
        descriptor = VIEW_DESCRIPTOR.read_text(encoding="utf-8")
        builder = VIEW_BUILDER.read_text(encoding="utf-8")
        mapping = VIEW_MAPPING.read_text(encoding="utf-8")

        for variant in (
            "LeftTopDrawer",
            "LeftBottomDrawer",
            "RightTopDrawer",
            "RightBottomDrawer",
            "BottomDrawer",
            "DocumentCenter",
            "FloatingWindow",
            "ExclusiveMainPage",
        ):
            self.assertIn(variant, slot)
            self.assertIn(f"WorkbenchSlot::{variant}", mapping)
        self.assertIn("pub workbench_slot: WorkbenchSlot", descriptor)
        self.assertIn("pub default_presets: Vec<DefaultWorkbenchPreset>", descriptor)
        self.assertNotIn("preferred_drawer_slot", descriptor)
        self.assertIn("with_workbench_slot", builder)
        self.assertIn("with_default_presets", builder)
        self.assertNotIn("with_preferred", builder)
        self.assertFalse(LEGACY_PREFERRED_HOST.exists())


if __name__ == "__main__":
    unittest.main()

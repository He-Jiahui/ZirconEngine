from pathlib import Path
import unittest

from tools.runtime_ui_layout_slot_index_pressure import (
    pressure_report,
    pressure_suite,
    validate_output_path,
)


ROOT = Path(__file__).resolve().parents[2]
LAYOUT_SLOT_INDEX = ROOT / "zircon_runtime/src/ui/layout/pass/slot.rs"
UI_TREE = ROOT / "zircon_runtime_interface/src/ui/tree/node/ui_tree.rs"
VIRTUAL_LIST_POOL = ROOT / (
    "zircon_runtime/src/ui/surface/virtual_list_prototype_pool.rs"
)
UNREAL_CHILDREN = ROOT / (
    "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/Children.h"
)


def function_body(source: str, signature: str, next_signature: str) -> str:
    return source.split(signature, 1)[1].split(next_signature, 1)[0]


class RuntimeUiLayoutSlotIndexPressureTests(unittest.TestCase):
    def test_full_index_build_counts_repeated_missing_edge_fallback_scans(self):
        report = pressure_report(
            unslotted_children=10_000,
            global_slots=10_000,
            changed_children=1,
            changed_parents=1,
        )

        rejected = report["rejected_workspace_scan_full_index_build"]
        achieved = report["achieved_tree_edge_full_index_build"]
        self.assertEqual(rejected["missing_edge_global_slot_visits"], 100_000_000)
        self.assertEqual(rejected["operation_units"], 100_030_000)
        self.assertEqual(achieved["missing_edge_global_slot_visits"], 0)
        self.assertEqual(achieved["operation_units"], 30_000)

    def test_one_child_dependency_patch_is_exact_and_topology_fallback_is_parent_local(self):
        report = pressure_report(10_000, 10_000, 1, 1)

        rejected = report["rejected_workspace_scan_local_dependency_patch"]
        achieved = report["achieved_exact_child_dependency_patch"]
        topology = report["achieved_parent_topology_dependency_rebuild"]
        self.assertEqual(rejected["parent_child_visits"], 10_000)
        self.assertEqual(rejected["missing_edge_global_slot_visits"], 100_000_000)
        self.assertEqual(rejected["operation_units"], 100_010_000)
        self.assertEqual(achieved["missing_edge_global_slot_visits"], 0)
        self.assertEqual(achieved["changed_child_visits"], 1)
        self.assertEqual(achieved["operation_units"], 1)
        self.assertEqual(topology["parent_child_visits"], 10_000)
        self.assertEqual(topology["operation_units"], 10_000)

    def test_parent_order_patch_removes_the_workspace_wide_slot_scan(self):
        report = pressure_report(64, 10_000, 1, 1)

        rejected = report["rejected_workspace_scan_local_parent_order_patch"]
        achieved = report["achieved_parent_local_order_patch"]
        self.assertEqual(rejected["workspace_wide_slot_visits"], 10_000)
        self.assertEqual(rejected["missing_edge_global_slot_visits"], 640_000)
        self.assertEqual(rejected["operation_units"], 650_064)
        self.assertEqual(achieved["workspace_wide_slot_visits"], 0)
        self.assertEqual(achieved["operation_units"], 64)
        self.assertFalse(report["is_product_timing"])

    def test_suite_binds_the_model_to_exact_workspace_and_reference_sources(self):
        report = pressure_suite(10_000, ROOT)

        binding = report["source_binding"]
        self.assertEqual(len(binding["workspace_head"]), 40)
        self.assertEqual(len(binding["files"]), 4)
        for source in binding["files"]:
            self.assertEqual(len(source["sha256"]), 64)
            self.assertTrue((ROOT / source["path"]).is_file())

    def test_rejects_invalid_or_inconsistent_inputs(self):
        for values in (
            (0, 1, 1, 1),
            (1, 0, 1, 1),
            (1, 1, 0, 1),
            (1, 1, 1, 0),
            (1, 1, 2, 1),
        ):
            with self.subTest(values=values):
                with self.assertRaises(ValueError):
                    pressure_report(*values)

    def test_artifact_output_rejects_the_system_drive(self):
        with self.assertRaises(ValueError):
            validate_output_path(r"C:\zircon-profiles\slot-index.json")
        self.assertEqual(
            validate_output_path(r"E:\zircon-profiles\slot-index.json").drive.upper(),
            "E:",
        )

    def test_current_zircon_uses_tree_owned_edge_authority_without_repair_scans(self):
        slot_index = LAYOUT_SLOT_INDEX.read_text(encoding="utf-8")
        ui_tree = UI_TREE.read_text(encoding="utf-8")
        virtual_list_pool = VIRTUAL_LIST_POOL.read_text(encoding="utf-8")
        unreal_children = UNREAL_CHILDREN.read_text(encoding="utf-8")

        production = slot_index.split("#[cfg(test)]\nmod tests", 1)[0]
        order_patch = function_body(
            slot_index,
            "fn patch_layout_order_parents",
            "fn rebuild_ordered_children",
        )
        dependency_patch = function_body(
            slot_index,
            "fn patch_parent_size_dependencies",
            "fn rebuild_parent_size_dependencies",
        )
        self.assertNotIn("tree.slots.iter()", production)
        self.assertNotIn("for (index, slot)", order_patch)
        self.assertNotIn("index_for_edge_matching", production)
        self.assertNotIn("edge_indices", production)
        self.assertIn("struct UiParentSizeDependencies", production)
        self.assertIn("patch_parent_size_dependency_child", dependency_patch)
        self.assertNotIn("for parent_id in parent_ids", dependency_patch)
        self.assertNotIn("pub slots: Vec<UiSlot>", ui_tree)
        self.assertIn("slots: Vec<UiSlot>", ui_tree)
        self.assertIn("layout_slot_authority: RefCell<UiLayoutSlotAuthority>", ui_tree)
        self.assertIn("layout_slot_index_for_edge_kind", ui_tree)
        self.assertIn("pub fn push_layout_slot", ui_tree)
        self.assertIn("surface.tree.push_layout_slot(root_slot)", virtual_list_pool)
        self.assertIn("surface.tree.push_layout_slot(slot)", virtual_list_pool)

        panel_children = unreal_children.split("class TPanelChildren final", 1)[1].split(
            "class TPanelChildrenConstIterator", 1
        )[0]
        self.assertIn("TArray<TUniquePtr<SlotType>> Children", panel_children)
        self.assertIn("return *Children[ChildIndex]", panel_children)
        self.assertIn("Children.Add(MoveTemp(NewSlot))", panel_children)


if __name__ == "__main__":
    unittest.main()

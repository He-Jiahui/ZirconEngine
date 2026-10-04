import json
import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.analysis.performance.editor.editor_menu_pointer_resize_pressure import (
    SOURCE_GUARDS,
    SOURCE_PATHS,
    build_source_binding,
    run,
    validate_source_contract,
    write_result,
)


ROOT = Path(__file__).resolve().parents[2]


class EditorMenuPointerResizePressureTests(unittest.TestCase):
    def test_resize_keeps_topology_and_registration_work_stable(self) -> None:
        result = run()

        self.assertEqual(result["inputs"]["resize_step_count"], 200)
        self.assertEqual(result["legacy_full_rebuild"]["surface_build_count"], 200)
        self.assertEqual(
            result["current_retained_geometry_patch"]["surface_build_count"], 0
        )
        self.assertEqual(
            result["legacy_full_rebuild"]["dispatcher_registration_count"],
            2_200,
        )
        self.assertEqual(
            result["current_retained_geometry_patch"]["dispatcher_registration_count"],
            0,
        )
        self.assertEqual(
            result["legacy_full_rebuild"]["route_path_string_build_count"],
            2_200,
        )
        self.assertEqual(result["delta"]["avoided_hover_surface_build_count"], 10_000)
        self.assertEqual(
            result["current_retained_geometry_patch"]["hover_geometry_publication_count"],
            0,
        )
        self.assertEqual(
            result["current_retained_geometry_patch"]["hover_popup_item_comparison_count"],
            0,
        )
        self.assertEqual(result["delta"]["avoided_popup_item_projection_count"], 8_000)
        self.assertEqual(result["delta"]["avoided_semantic_layout_build_count"], 200)
        self.assertEqual(
            result["delta"]["avoided_popup_text_row_measurement_count"],
            8_000,
        )
        self.assertEqual(
            result["current_retained_geometry_patch"]["geometry_layout_build_count"],
            200,
        )
        self.assertFalse(result["interpretation"]["timing_claim"])
        self.assertEqual(result["schema"], "zircon.editor.menu_pointer_resize_pressure.v7")

    def test_window_metrics_reuses_geometry_free_semantic_pointer_receipts(self) -> None:
        result = run()
        receipts = result["semantic_pointer_receipt_resize"]

        self.assertEqual(result["derived"]["semantic_pointer_receipt_type_count"], 3)
        self.assertEqual(result["derived"]["semantic_pointer_receipt_item_count"], 13)
        self.assertEqual(
            receipts["legacy_projection"]["receipt_product_build_count"], 600
        )
        self.assertEqual(
            receipts["legacy_projection"]["receipt_item_projection_count"], 2_600
        )
        self.assertEqual(
            receipts["current_retained_receipts"]["receipt_product_build_count"], 0
        )
        self.assertEqual(
            receipts["current_retained_receipts"]["receipt_item_projection_count"], 0
        )

    def test_window_metrics_reuses_welcome_recent_semantics(self) -> None:
        result = run()
        recent = result["welcome_recent_resize"]

        self.assertEqual(recent["legacy_projection"]["recent_path_clone_count"], 200)
        self.assertEqual(recent["legacy_projection"]["recent_item_projection_count"], 1_600)
        self.assertEqual(recent["current_viewport_fast_path"]["recent_path_clone_count"], 0)
        self.assertEqual(recent["current_viewport_fast_path"]["recent_item_projection_count"], 0)
        self.assertEqual(recent["current_viewport_fast_path"]["viewport_sync_count"], 200)

    def test_activity_rail_resize_retains_surface_and_route_authority(self) -> None:
        result = run()
        activity = result["activity_rail_resize"]

        self.assertEqual(result["derived"]["activity_rail_surface_node_count"], 7)
        self.assertEqual(activity["legacy_full_rebuild"]["surface_build_count"], 200)
        self.assertEqual(
            activity["current_retained_geometry_patch"]["surface_build_count"], 0
        )
        self.assertEqual(
            activity["legacy_full_rebuild"]["dispatcher_registration_count"],
            1_200,
        )
        self.assertEqual(
            activity["current_retained_geometry_patch"][
                "dispatcher_registration_count"
            ],
            0,
        )
        self.assertEqual(
            activity["legacy_full_rebuild"]["semantic_tab_projection_count"], 800
        )
        self.assertEqual(
            activity["current_retained_geometry_patch"]["semantic_tab_projection_count"],
            0,
        )
        self.assertEqual(
            activity["current_retained_geometry_patch"]["geometry_product_build_count"],
            200,
        )
        self.assertEqual(activity["legacy_full_rebuild"]["node_domain_visit_units"], 7_000)
        self.assertEqual(
            activity["current_retained_geometry_patch"]["node_domain_visit_units"],
            5_000,
        )

    def test_geometry_patch_work_is_bounded_by_changed_nodes(self) -> None:
        result = run()

        self.assertEqual(result["derived"]["surface_node_count"], 12)
        self.assertEqual(
            result["legacy_full_rebuild"]["node_domain_visit_units"],
            12_000,
        )
        self.assertEqual(
            result["current_retained_geometry_patch"]["node_domain_visit_units"],
            3_000,
        )
        self.assertGreater(
            result["delta"]["node_domain_visit_reduction_ratio"],
            3.9,
        )

    def test_invalid_cardinalities_fail_closed(self) -> None:
        for kwargs in (
            {"resize_step_count": 0},
            {"hover_state_change_count": 0},
            {"menu_button_count": 0},
            {"open_popup_item_count": 0},
            {"open_submenu_depth": -1},
            {"changed_geometry_node_count": 0},
            {"changed_geometry_node_count": 13},
            {"activity_rail_tab_count": 0},
            {"activity_rail_changed_geometry_node_count": 0},
            {"activity_rail_changed_geometry_node_count": 8},
            {"host_page_receipt_item_count": -1},
            {"document_tab_receipt_item_count": -1},
            {"drawer_header_receipt_item_count": -1},
            {"welcome_recent_item_count": -1},
        ):
            with self.subTest(kwargs=kwargs):
                with self.assertRaises(ValueError):
                    run(**kwargs)

    def test_output_is_stable_json_and_restricted_to_profile_drives(self) -> None:
        result = run(resize_step_count=2)
        with tempfile.TemporaryDirectory(dir=Path(tempfile.gettempdir())) as directory:
            output = Path(directory) / "menu-pointer-resize.json"
            write_result(output, result)
            self.assertEqual(json.loads(output.read_text(encoding="utf-8")), result)
            self.assertTrue(output.read_text(encoding="utf-8").endswith("\n"))

        for output in (
            Path("C:/temp/menu-pointer-resize.json"),
            Path("relative/menu-pointer-resize.json"),
        ):
            with self.subTest(output=output):
                with self.assertRaises(ValueError):
                    write_result(output, result)

    def test_current_source_binding_proves_full_rebuild_and_geometry_reference(self) -> None:
        binding = build_source_binding(ROOT)

        self.assertTrue(binding["ready"], binding["blockers"])
        self.assertEqual(len(binding["git_revision"]), 40)
        self.assertEqual(
            [entry["relative_path"] for entry in binding["critical_source_files"]],
            list(SOURCE_PATHS),
        )
        self.assertTrue(
            all(len(entry["sha256"]) == 64 for entry in binding["critical_source_files"])
        )

    def test_source_guard_rejects_loss_of_resize_geometry_reference(self) -> None:
        sources = {
            relative_path: (ROOT / relative_path).read_text(encoding="utf-8")
            for relative_path in SOURCE_PATHS
        }
        self.assertTrue(validate_source_contract(sources)["ready"])

        toolbar_path = (
            "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/"
            "rebuild_surface.rs"
        )
        sources[toolbar_path] = sources[toolbar_path].replace(
            "publish_authored_geometry", "publish_full_geometry", 1
        )
        changed = validate_source_contract(sources)

        self.assertFalse(changed["ready"])
        self.assertIn(
            toolbar_path,
            {blocker.get("relative_path") for blocker in changed["blockers"]},
        )

    def test_source_binding_derives_text_size_and_hash_from_the_same_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            expected_entries = []
            for relative_path in SOURCE_PATHS:
                payload = "\n".join(SOURCE_GUARDS[relative_path]).encode("utf-8")
                path = root / relative_path
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(payload)
                expected_entries.append(
                    {
                        "relative_path": relative_path,
                        "byte_length": len(payload),
                        "sha256": hashlib.sha256(payload).hexdigest().upper(),
                    }
                )

            with mock.patch("subprocess.run") as run_git:
                run_git.return_value.stdout = "a" * 40 + "\n"
                binding = build_source_binding(root)

            self.assertTrue(binding["ready"], binding["blockers"])
            self.assertEqual(binding["critical_source_files"], expected_entries)


if __name__ == "__main__":
    unittest.main()

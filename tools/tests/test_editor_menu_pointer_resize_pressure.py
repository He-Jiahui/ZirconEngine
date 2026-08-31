import json
import tempfile
import unittest
from pathlib import Path

from tools.editor_menu_pointer_resize_pressure import (
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
        self.assertEqual(result["current_full_rebuild"]["surface_build_count"], 200)
        self.assertEqual(result["retained_geometry_patch"]["surface_build_count"], 0)
        self.assertEqual(
            result["current_full_rebuild"]["dispatcher_registration_count"],
            2_200,
        )
        self.assertEqual(
            result["retained_geometry_patch"]["dispatcher_registration_count"],
            0,
        )
        self.assertEqual(
            result["current_full_rebuild"]["route_path_string_build_count"],
            2_200,
        )
        self.assertFalse(result["interpretation"]["timing_claim"])

    def test_geometry_patch_work_is_bounded_by_changed_nodes(self) -> None:
        result = run()

        self.assertEqual(result["derived"]["surface_node_count"], 12)
        self.assertEqual(
            result["current_full_rebuild"]["node_domain_visit_units"],
            9_600,
        )
        self.assertEqual(
            result["retained_geometry_patch"]["node_domain_visit_units"],
            1_800,
        )
        self.assertGreater(
            result["delta"]["node_domain_visit_reduction_ratio"],
            5.0,
        )

    def test_invalid_cardinalities_fail_closed(self) -> None:
        for kwargs in (
            {"resize_step_count": 0},
            {"menu_button_count": 0},
            {"open_popup_item_count": 0},
            {"open_submenu_depth": -1},
            {"changed_geometry_node_count": 0},
            {"changed_geometry_node_count": 13},
        ):
            with self.subTest(kwargs=kwargs):
                with self.assertRaises(ValueError):
                    run(**kwargs)

    def test_output_is_stable_json_and_restricted_to_profile_drives(self) -> None:
        result = run(resize_step_count=2)
        with tempfile.TemporaryDirectory(dir=Path("E:/zircon-profiles")) as directory:
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


if __name__ == "__main__":
    unittest.main()

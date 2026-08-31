import json
import tempfile
import unittest
from pathlib import Path

import tools.ui_pane_button_fallback_damage_pressure as pressure


REPO_ROOT = Path(__file__).resolve().parents[2]


class PaneButtonFallbackDamagePressureTests(unittest.TestCase):
    def test_current_source_contract_counts_conservative_damage_domains(self) -> None:
        contract = pressure.source_contract(REPO_ROOT)

        self.assertEqual(contract["pane_frame_unions"], 1)
        self.assertEqual(contract["center_band_unions"], 2)
        self.assertEqual(contract["status_bar_unions"], 3)
        self.assertTrue(contract["pressed_requests_frame_update"])
        self.assertTrue(contract["release_is_pointer_local"])

    def test_default_model_separates_bounding_damage_from_typed_regions(self) -> None:
        report = pressure.pressure_report()

        self.assertFalse(report["is_product_timing"])
        self.assertEqual(report["current_fallback"]["damage_region_count"], 1)
        self.assertEqual(report["typed_action_target"]["damage_region_count"], 2)
        self.assertGreater(
            report["current_fallback"]["bounding_area_px"],
            report["typed_action_target"]["represented_area_px"],
        )
        self.assertGreater(report["comparison"]["represented_area_ratio"], 20.0)
        self.assertEqual(report["comparison"]["semantic_correctness_policy"], "fail_closed")

    def test_model_rejects_non_finite_or_non_positive_geometry(self) -> None:
        with self.assertRaises(ValueError):
            pressure.pressure_report(viewport_width=0)
        with self.assertRaises(ValueError):
            pressure.pressure_report(button_width=float("nan"))
        with self.assertRaises(ValueError):
            pressure.pressure_report(status_height=-1)

    def test_cli_payload_is_json_serializable(self) -> None:
        payload = pressure.build_payload(REPO_ROOT)
        encoded = json.dumps(payload, sort_keys=True)

        self.assertIn("zircon.editor.ui_pane_button_fallback_damage_pressure.v1", encoded)
        self.assertIn("source_contract", payload)

    def test_output_path_rejects_c_drive(self) -> None:
        with self.assertRaises(ValueError):
            pressure.validate_output_path(r"C:\\temp\\pane-button-pressure.json")

        with tempfile.TemporaryDirectory(dir=REPO_ROOT) as directory:
            output = pressure.validate_output_path(str(Path(directory) / "report.json"))
            self.assertEqual(output.name, "report.json")


if __name__ == "__main__":
    unittest.main()

import json
import tempfile
import unittest
from pathlib import Path

from tools.analysis.performance.editor import editor_overlay_presentation_pressure as pressure


ROOT = Path(__file__).resolve().parents[2]


class EditorOverlayPresentationPressureTests(unittest.TestCase):
    def test_typed_overlay_publication_skips_global_paint_model_validation(self) -> None:
        result = pressure.run(overlay_update_count=10_000, paint_model_count=64)

        self.assertEqual(
            result["legacy_generic_presentation_update"]["paint_model_identity_visit_count"],
            640_000,
        )
        self.assertEqual(
            result["current_typed_overlay_publication"]["paint_model_identity_visit_count"],
            0,
        )
        self.assertEqual(
            result["implemented_delta"]["avoided_paint_model_directory_materialization_count"],
            10_000,
        )

    def test_current_source_binding_is_ready(self) -> None:
        binding = pressure.source_binding_report(ROOT)

        self.assertTrue(binding["ready"])
        self.assertEqual(
            len(binding["critical_sources"]),
            len(pressure.CRITICAL_SOURCE_CONTRACTS),
        )
        self.assertRegex(binding["source_set_sha256"], r"^[0-9A-F]{64}$")

    def test_source_binding_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory(dir=Path("E:/zircon-profiles")) as directory:
            root = Path(directory)
            for relative_path, _ in pressure.CRITICAL_SOURCE_CONTRACTS:
                path = root / relative_path
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("authority changed\n", encoding="utf-8")

            with self.assertRaises(pressure.SourceContractError):
                pressure.source_binding_report(root)

    def test_invalid_inputs_are_rejected(self) -> None:
        with self.assertRaises(ValueError):
            pressure.run(overlay_update_count=-1, paint_model_count=1)
        with self.assertRaises(ValueError):
            pressure.run(overlay_update_count=1, paint_model_count=0)

    def test_output_is_stable_json(self) -> None:
        result = pressure.run(overlay_update_count=4, paint_model_count=8)
        with tempfile.TemporaryDirectory(dir=Path("E:/zircon-profiles")) as directory:
            output = Path(directory) / "pressure.json"
            pressure.write_result(output, result)

            self.assertEqual(json.loads(output.read_text(encoding="utf-8")), result)
            self.assertTrue(output.read_text(encoding="utf-8").endswith("\n"))


if __name__ == "__main__":
    unittest.main()

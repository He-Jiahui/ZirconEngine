from pathlib import Path
import tempfile
import unittest

from tools.editor_viewport_toolbar_pointer_surface_reuse_pressure import (
    SourceContractError,
    pressure_report,
    source_binding_report,
    validate_output_path,
)


ROOT = Path(__file__).resolve().parents[2]


class EditorViewportToolbarPointerSurfaceReuseTests(unittest.TestCase):
    def test_default_model_bounds_surface_reconstruction_to_topology_changes(self) -> None:
        result = pressure_report()

        current = result["current_full_reconstruction"]
        target = result["retained_surface_frame_patch"]
        delta = result["delta"]

        self.assertEqual(result["inputs"]["node_count"], 529)
        self.assertEqual(current["surface_object_reconstruction_count"], 1_010)
        self.assertEqual(current["authored_frame_full_pipeline_rebuild_count"], 1_010)
        self.assertEqual(current["node_allocation_count"], 534_290)
        self.assertEqual(current["route_materialization_count"], 517_120)
        self.assertEqual(target["surface_object_reconstruction_count"], 10)
        self.assertEqual(target["authored_frame_full_pipeline_rebuild_count"], 1_010)
        self.assertEqual(target["retained_frame_patch_count"], 1_000)
        self.assertEqual(target["node_allocation_count"], 5_290)
        self.assertEqual(target["route_materialization_count"], 5_120)
        self.assertEqual(delta["avoided_surface_object_reconstruction_count"], 1_000)
        self.assertEqual(delta["avoided_full_pipeline_rebuild_count"], 0)
        self.assertEqual(delta["avoided_node_allocation_count"], 529_000)
        self.assertEqual(delta["avoided_route_materialization_count"], 512_000)
        self.assertTrue(result["residual_cost"]["requires_runtime_geometry_patch_api"])
        self.assertFalse(result["is_product_timing"])

    def test_model_rejects_invalid_cardinality(self) -> None:
        for kwargs in (
            {"surface_count": 0},
            {"controls_per_surface": 0},
            {"frame_patch_count": -1},
            {"topology_change_count": -1},
        ):
            with self.subTest(kwargs=kwargs), self.assertRaises(ValueError):
                pressure_report(**kwargs)

    def test_current_source_has_retained_patch_and_typed_full_fallback(self) -> None:
        binding = source_binding_report(ROOT)

        self.assertTrue(binding["ready"])
        self.assertEqual(len(binding["critical_sources"]), 4)
        self.assertRegex(binding["source_set_sha256"], r"^[0-9A-F]{64}$")

    def test_retained_patch_validates_before_mutating_and_keeps_one_constructor(
        self,
    ) -> None:
        source = (
            ROOT
            / "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/"
            "rebuild_surface.rs"
        ).read_text(encoding="utf-8")

        validation = source.index("if !self.retained_surface_topology_matches()")
        patch = source.index("patch_retained_node_frame(", validation)
        self.assertLess(validation, patch)
        self.assertEqual(source.count("UiSurface::new("), 1)
        self.assertIn("fn rebuild_surface_from_scratch", source)
        self.assertEqual(source.count("rebuild_authored_frames("), 2)

    def test_source_binding_fails_closed_when_patch_authority_changes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            production = root / (
                "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/"
                "rebuild_surface.rs"
            )
            production.parent.mkdir(parents=True, exist_ok=True)
            production.write_text("full rebuild only\n", encoding="utf-8")

            with self.assertRaises(SourceContractError):
                source_binding_report(root)

    def test_output_artifacts_are_restricted_to_d_e_or_f(self) -> None:
        for drive in ("D:", "E:", "F:"):
            path = Path(f"{drive}/zircon-profiles/toolbar-pointer.json")
            self.assertEqual(validate_output_path(path), path)
        with self.assertRaises(ValueError):
            validate_output_path(Path("C:/zircon-profiles/toolbar-pointer.json"))


if __name__ == "__main__":
    unittest.main()

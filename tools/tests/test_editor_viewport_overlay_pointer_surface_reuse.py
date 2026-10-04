from pathlib import Path
import tempfile
import unittest

from tools.analysis.performance.editor.editor_viewport_overlay_pointer_surface_reuse_pressure import (
    SourceContractError,
    pressure_report,
    source_binding_report,
    validate_output_path,
)


ROOT = Path(__file__).resolve().parents[2]
REBUILD_SOURCE = ROOT / (
    "zircon_editor/src/scene/viewport/pointer/overlay_router/rebuild_surface.rs"
)


class EditorViewportOverlayPointerSurfaceReuseTests(unittest.TestCase):
    def test_stable_candidate_topology_avoids_surface_and_node_reconstruction(self) -> None:
        result = pressure_report()

        current = result["current_full_reconstruction"]
        retained = result["retained_candidate_authority"]
        delta = result["delta"]

        self.assertEqual(result["inputs"]["node_count"], 4_098)
        self.assertEqual(current["surface_object_reconstruction_count"], 1_010)
        self.assertEqual(current["node_allocation_count"], 4_138_980)
        self.assertEqual(retained["surface_object_reconstruction_count"], 10)
        self.assertEqual(retained["node_allocation_count"], 40_980)
        self.assertEqual(retained["topology_validation_node_visit_count"], 4_138_980)
        self.assertEqual(retained["retained_frame_patch_probe_count"], 4_098_000)
        self.assertEqual(retained["candidate_map_materialization_count"], 40_960)
        self.assertEqual(retained["candidate_map_value_patch_count"], 4_096_000)
        self.assertEqual(delta["avoided_surface_object_reconstruction_count"], 1_000)
        self.assertEqual(delta["avoided_node_allocation_count"], 4_098_000)
        self.assertEqual(delta["avoided_candidate_map_materialization_count"], 4_096_000)
        self.assertEqual(delta["avoided_full_pipeline_rebuild_count"], 0)
        self.assertTrue(result["residual_cost"]["requires_runtime_geometry_patch_api"])
        self.assertFalse(result["is_product_timing"])

    def test_model_rejects_invalid_cardinality(self) -> None:
        for kwargs in (
            {"candidate_count": 0},
            {"frame_patch_count": -1},
            {"topology_change_count": -1},
        ):
            with self.subTest(kwargs=kwargs), self.assertRaises(ValueError):
                pressure_report(**kwargs)

    def test_current_source_has_one_typed_fallback_and_delta_classifier(self) -> None:
        binding = source_binding_report(ROOT)

        self.assertTrue(binding["ready"])
        self.assertEqual(len(binding["critical_sources"]), 6)
        self.assertRegex(binding["source_set_sha256"], r"^[0-9A-F]{64}$")

        source = REBUILD_SOURCE.read_text(encoding="utf-8")
        self.assertEqual(source.count("UiSurface::new("), 1)
        self.assertEqual(source.count("self.surface = surface;"), 1)
        self.assertIn("fn classify_surface_delta", source)
        self.assertIn("fn apply_geometry_delta", source)
        self.assertIn("fn rebuild_surface_from_scratch", source)
        self.assertIn("fn publish_retained_candidates", source)
        self.assertIn("UiAuthoredGeometryPublication::FullFallback", source)
        self.assertIn("rebuild_authored_frames(", source)

    def test_delta_classifier_validates_topology_before_geometry_write(self) -> None:
        source = REBUILD_SOURCE.read_text(encoding="utf-8")

        classification = source[
            source.index("fn classify_surface_delta") : source.index(
                "fn apply_surface_delta"
            )
        ]
        for required in (
            "self.retained_candidate_ids.len() == candidates.len()",
            "classify_node_geometry(",
            "self.retained_candidate_ids.get(candidate_index) == Some(&candidate.node_id)",
            "shared.candidates.get(&candidate.node_id)",
            "current.route != candidate.candidate.route",
            "candidate_map_reusable &= topology_matches",
        ):
            self.assertIn(required, classification)

        route_guard = source[
            source.index("let shared = lock_shared_resolution_state(self.shared.as_ref())") : source.index(
                "candidate_map_reusable &= topology_matches"
            )
        ]
        for required in (
            "lock_shared_resolution_state(self.shared.as_ref())",
            ".get(&candidate.node_id)",
            "current.route != candidate.candidate.route",
        ):
            self.assertIn(required, route_guard)
        for forbidden in (".collect::<Vec", "Vec::", ".clone()"):
            self.assertNotIn(forbidden, route_guard)
        delta_application = source[
            source.index("fn apply_surface_delta") : source.index(
                "fn apply_geometry_delta"
            )
        ]
        self.assertIn(
            "release_capture_for_route_change(&mut self.surface, route_identity_changed);",
            delta_application,
        )
        self.assertIn("fn release_capture_for_route_change", source)
        self.assertIn("surface.release_pointer_capture()", source)
        self.assertNotIn(".raw()", source)
        self.assertNotIn("UiNodeId::raw", source)

    def test_source_binding_fails_closed_when_reuse_authority_disappears(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            production = root / REBUILD_SOURCE.relative_to(ROOT)
            production.parent.mkdir(parents=True, exist_ok=True)
            production.write_text("full rebuild only\n", encoding="utf-8")

            with self.assertRaises(SourceContractError):
                source_binding_report(root)

    def test_output_artifacts_are_restricted_to_d_e_or_f(self) -> None:
        for drive in ("D:", "E:", "F:"):
            path = Path(f"{drive}/zircon-profiles/viewport-overlay-pointer.json")
            self.assertEqual(validate_output_path(path), path)
        with self.assertRaises(ValueError):
            validate_output_path(
                Path("C:/zircon-profiles/viewport-overlay-pointer.json")
            )


if __name__ == "__main__":
    unittest.main()

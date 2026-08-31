from pathlib import Path
import tempfile
import unittest

from tools.runtime_ui_authored_geometry_delta_pressure import (
    SourceContractError,
    pressure_report,
    source_binding_report,
    validate_output_path,
)


ROOT = Path(__file__).resolve().parents[2]


class RuntimeUiAuthoredGeometryDeltaPressureTests(unittest.TestCase):
    def test_exact_geometry_patch_bounds_internal_domains_to_changed_nodes(self) -> None:
        result = pressure_report()

        current = result["current_authored_frame_publication"]
        target = result["runtime_exact_geometry_publication"]
        delta = result["delta"]

        self.assertEqual(result["inputs"]["node_count"], 529)
        self.assertEqual(current["full_pipeline_rebuild_count"], 1_010)
        self.assertEqual(current["arranged_node_visit_count"], 534_290)
        self.assertEqual(target["full_pipeline_rebuild_count"], 10)
        self.assertEqual(target["exact_geometry_patch_count"], 1_000)
        self.assertEqual(target["arranged_node_visit_count"], 6_290)
        self.assertEqual(target["hit_node_visit_count"], 6_290)
        self.assertEqual(target["render_node_visit_count"], 6_290)
        self.assertEqual(delta["avoided_internal_domain_node_visit_count"], 1_584_000)
        self.assertFalse(result["is_product_timing"])

    def test_runtime_persistent_authority_removes_arranged_and_hit_publication_clones(self) -> None:
        result = pressure_report()

        current = result["current_authored_frame_publication"]
        runtime_only = result["runtime_exact_geometry_publication"]

        self.assertEqual(current["published_arranged_node_clone_count"], 534_290)
        self.assertEqual(runtime_only["published_arranged_node_clone_count"], 0)
        self.assertEqual(current["published_hit_entry_clone_count"], 534_290)
        self.assertEqual(runtime_only["published_hit_entry_clone_count"], 0)
        self.assertEqual(runtime_only["persistent_arranged_item_copy_upper_bound"], 69_290)
        self.assertEqual(runtime_only["persistent_hit_entry_item_copy_upper_bound"], 69_290)
        self.assertEqual(
            runtime_only["published_snapshot_complexity"],
            "O(1) root sharing at publication; producer mutation is bounded by touched 64-item leaves, directory paths, and hit-cell leaves",
        )

    def test_persistent_published_domains_bound_logical_updates_to_changed_nodes(self) -> None:
        result = pressure_report()

        end_state = result["persistent_published_domain_end_state"]

        self.assertEqual(end_state["arranged_logical_update_count"], 6_290)
        self.assertEqual(end_state["hit_entry_logical_update_count"], 6_290)
        self.assertEqual(end_state["stable_frame_full_arranged_clone_count"], 0)
        self.assertEqual(end_state["stable_frame_full_hit_clone_count"], 0)
        self.assertEqual(end_state["arranged_item_copy_upper_bound"], 69_290)

    def test_editor_typed_delta_removes_redundant_topology_walks(self) -> None:
        result = pressure_report()

        runtime_only = result["runtime_exact_geometry_publication"]
        end_state = result["editor_typed_delta_end_state"]

        self.assertEqual(runtime_only["topology_validation_node_visit_count"], 534_290)
        self.assertEqual(end_state["topology_validation_node_visit_count"], 5_290)
        self.assertEqual(end_state["geometry_identity_check_count"], 1_000)

    def test_clip_expansion_is_explicit_and_bounded(self) -> None:
        result = pressure_report(changed_nodes_per_frame_patch=3)

        target = result["runtime_exact_geometry_publication"]
        self.assertEqual(target["arranged_node_visit_count"], 8_290)
        self.assertEqual(target["hit_node_visit_count"], 8_290)
        self.assertEqual(target["render_node_visit_count"], 8_290)

    def test_resize_capacity_envelope_avoids_repeated_full_hit_rebuilds(self) -> None:
        result = pressure_report()
        resize = result["window_resize_capacity_envelope"]

        self.assertEqual(resize["current_full_pipeline_rebuild_count"], 120)
        self.assertEqual(resize["current_hit_arranged_node_visit_count"], 63_480)
        self.assertEqual(resize["candidate_hit_capacity_regrid_count"], 2)
        self.assertEqual(resize["candidate_hit_node_visit_count"], 1_176)
        self.assertEqual(resize["avoided_hit_node_visit_count"], 62_304)

    def test_model_rejects_invalid_cardinality(self) -> None:
        for kwargs in (
            {"node_count": 0},
            {"frame_patch_count": -1},
            {"topology_change_count": -1},
            {"changed_nodes_per_frame_patch": 0},
            {"changed_nodes_per_frame_patch": 530},
        ):
            with self.subTest(kwargs=kwargs), self.assertRaises(ValueError):
                pressure_report(**kwargs)

    def test_current_source_binding_is_ready_and_content_hashed(self) -> None:
        binding = source_binding_report(ROOT)

        self.assertTrue(binding["ready"])
        self.assertEqual(len(binding["critical_sources"]), 13)
        self.assertRegex(binding["source_set_sha256"], r"^[0-9A-F]{64}$")

    def test_source_binding_fails_closed_when_geometry_authority_changes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            production = root / "zircon_runtime/src/ui/surface/arranged.rs"
            production.parent.mkdir(parents=True, exist_ok=True)
            production.write_text("full rebuild only\n", encoding="utf-8")

            with self.assertRaises(SourceContractError):
                source_binding_report(root)

    def test_output_artifacts_are_restricted_to_d_e_or_f(self) -> None:
        for drive in ("D:", "E:", "F:"):
            path = Path(f"{drive}/zircon-profiles/authored-geometry.json")
            self.assertEqual(validate_output_path(path), path)
        with self.assertRaises(ValueError):
            validate_output_path(Path("C:/zircon-profiles/authored-geometry.json"))


if __name__ == "__main__":
    unittest.main()

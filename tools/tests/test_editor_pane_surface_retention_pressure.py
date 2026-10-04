import json
import tempfile
import unittest
from pathlib import Path

from tools.analysis.performance.editor import editor_pane_surface_retention_pressure as pressure
from tools.analysis.performance.editor.editor_pane_surface_retention_pressure import run, write_result


ROOT = Path(__file__).resolve().parents[2]


class EditorPaneSurfaceRetentionPressureTests(unittest.TestCase):
    def test_stable_updates_do_not_rebuild_retained_pane_surfaces(self) -> None:
        result = run(
            pane_count=64,
            nodes_per_pane=2_048,
            stable_update_count=1_000,
            changed_update_count=0,
            changed_panes_per_update=0,
        )

        self.assertEqual(result["baseline_stable_surface_build_count"], 64_000)
        self.assertEqual(result["retained_stable_surface_build_count"], 0)
        self.assertEqual(result["retained_total_surface_build_count"], 64)
        self.assertEqual(result["stable_surface_build_avoidance_count"], 64_000)
        self.assertEqual(result["stable_surface_build_avoidance_percent"], 100.0)

    def test_single_pane_changes_keep_work_independent_of_other_panes(self) -> None:
        result = run(
            pane_count=64,
            nodes_per_pane=2_048,
            stable_update_count=1_000,
            changed_update_count=1_000,
            changed_panes_per_update=1,
            geometry_update_count=600,
        )

        self.assertEqual(
            result["schema"],
            "zircon.editor.pane_surface_retention_pressure.v8",
        )
        self.assertEqual(result["baseline_total_surface_build_count"], 128_064)
        self.assertEqual(result["retained_total_surface_build_count"], 1_064)
        self.assertEqual(result["baseline_node_stage_visit_count"], 1_049_100_288)
        self.assertEqual(result["retained_node_stage_visit_count"], 8_716_288)
        self.assertGreater(result["node_stage_visit_reduction_ratio"], 120.0)
        self.assertEqual(result["retained_unchanged_pane_rebuild_count"], 0)

        rejected = result["rejected_dual_arc_aggregate_presentation"]
        current = result["current_semantic_identity_presentation"]
        target = result["pane_owned_presentation_transaction"]
        scoped = result["current_scoped_ui_asset_pane_transaction"]
        implemented = result["implemented_semantic_identity_delta"]
        scoped_delta = result["implemented_scoped_ui_asset_transaction_delta"]
        remaining = result["pane_transaction_remaining_delta"]
        self.assertEqual(rejected["whole_presentation_pane_clone_count"], 64_000)
        self.assertEqual(current["geometry_semantic_pane_clone_count"], 4_800)
        self.assertEqual(current["state_owned_presentation_arc_count"], 1)
        self.assertEqual(current["semantic_identity_lifetime_allocation_count"], 0)
        self.assertEqual(current["semantic_identity_update_allocation_count"], 0)
        self.assertEqual(current["semantic_generation_increment_count"], 1_000)
        self.assertEqual(current["internal_bookkeeping_cow_clone_count"], 0)
        self.assertEqual(current["whole_presentation_pane_clone_count"], 0)
        self.assertEqual(current["paint_model_identity_visit_count"], 64_000)
        self.assertEqual(current["hit_node_rebuild_visit_count"], 131_072_000)
        self.assertEqual(target["geometry_semantic_pane_clone_count"], 0)
        self.assertEqual(target["whole_presentation_pane_clone_count"], 0)
        self.assertEqual(target["hit_node_rebuild_visit_count"], 2_048_000)
        self.assertEqual(target["unchanged_pane_hit_node_visit_count"], 0)
        self.assertEqual(scoped["pane_clone_count"], 1_000)
        self.assertEqual(scoped["whole_presentation_pane_clone_count"], 0)
        self.assertEqual(scoped["full_hit_index_build_count"], 0)
        self.assertEqual(scoped["paint_model_directory_visit_count"], 64_000)
        self.assertEqual(scoped["hit_node_rebuild_visit_count"], 2_048_000)
        self.assertEqual(scoped["unchanged_pane_hit_node_visit_count"], 0)
        self.assertEqual(
            implemented["avoided_internal_whole_presentation_pane_clone_count"],
            64_000,
        )
        self.assertEqual(
            scoped_delta["avoided_unchanged_pane_hit_node_visit_count"],
            129_024_000,
        )
        self.assertEqual(scoped_delta["hit_node_visit_reduction_ratio"], 64.0)
        self.assertEqual(remaining["hit_node_visit_reduction_ratio"], 64.0)
        popup_hit = result["scoped_pane_popup_hit_index"]
        self.assertEqual(
            popup_hit["rejected_frame_only_pointer_node_visit_count"],
            204_800_000,
        )
        self.assertEqual(
            popup_hit["current_publication_index_node_visit_count"],
            2_048_000,
        )
        self.assertEqual(
            popup_hit["current_pointer_candidate_visit_count"],
            400_000,
        )
        self.assertGreater(popup_hit["node_visit_reduction_ratio"], 80.0)
        toolbar = result["viewport_toolbar_pane_transaction"]
        self.assertEqual(
            toolbar["legacy_paint_model_identity_visit_count"],
            128_000,
        )
        self.assertEqual(toolbar["legacy_floating_row_clone_count"], 32_000)
        self.assertEqual(toolbar["current_stable_pane_clone_count"], 0)
        self.assertEqual(toolbar["current_stable_generation_increment_count"], 0)
        self.assertEqual(toolbar["current_paint_model_identity_visit_count"], 0)
        self.assertEqual(toolbar["current_route_changed_pane_clone_count"], 1_000)
        self.assertEqual(toolbar["current_unchanged_floating_row_clone_count"], 0)
        native = result["native_floating_presentation_transaction"]
        self.assertEqual(native["update_count"], 1_000)
        self.assertEqual(native["legacy_presentation_cow_clone_count"], 1_000)
        self.assertEqual(native["legacy_paint_model_identity_visit_count"], 64_000)
        self.assertEqual(native["legacy_workbench_hit_index_rebuild_count"], 0)
        self.assertEqual(native["legacy_structure_generation_increment_count"], 1_000)
        self.assertEqual(native["legacy_geometry_generation_increment_count"], 1_000)
        self.assertEqual(native["current_metadata_commit_count"], 1_000)
        self.assertEqual(native["current_presentation_cow_clone_count"], 0)
        self.assertEqual(native["current_paint_model_identity_visit_count"], 0)
        self.assertEqual(native["current_workbench_hit_index_rebuild_count"], 0)
        self.assertEqual(native["current_structure_generation_increment_count"], 0)
        self.assertEqual(native["current_geometry_generation_increment_count"], 1_000)
        self.assertEqual(native["stable_repeat_commit_count"], 0)

    def test_current_source_binding_covers_zircon_and_reference_authorities(self) -> None:
        self.assertTrue(hasattr(pressure, "CRITICAL_SOURCE_CONTRACTS"))
        self.assertTrue(hasattr(pressure, "source_binding_report"))
        binding = pressure.source_binding_report(ROOT)

        self.assertTrue(binding["ready"])
        self.assertEqual(
            len(binding["critical_sources"]),
            len(pressure.CRITICAL_SOURCE_CONTRACTS),
        )
        paths = {source["relative_path"] for source in binding["critical_sources"]}
        self.assertIn(
            "zircon_editor/src/ui/retained_host/host_contract/globals/state.rs",
            paths,
        )
        self.assertIn(
            "zircon_editor/src/ui/retained_host/host_contract/data/"
            "presentation_generation.rs",
            paths,
        )
        self.assertIn(
            "zircon_editor/src/ui/retained_host/host_contract/data/"
            "pane_presentation_patch.rs",
            paths,
        )
        self.assertIn(
            "zircon_editor/src/ui/retained_host/host_contract/data/"
            "presentation_paint_models.rs",
            paths,
        )
        self.assertIn(
            "zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/"
            "surface_frames.rs",
            paths,
        )
        self.assertIn(
            "zircon_editor/src/ui/retained_host/app/native_windows/presentation.rs",
            paths,
        )
        self.assertIn(
            "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/"
            "FastUpdate/SlateInvalidationRoot.cpp",
            paths,
        )
        self.assertIn("dev/Fyrox/fyrox-ui/src/widget.rs", paths)
        self.assertIn("dev/slint/internal/core/partial_renderer.rs", paths)
        self.assertRegex(binding["source_set_sha256"], r"^[0-9A-F]{64}$")

    def test_native_window_publisher_does_not_retain_generation_or_global_updater(self) -> None:
        source = (
            ROOT
            / "zircon_editor/src/ui/retained_host/app/native_windows/presentation.rs"
        ).read_text(encoding="utf-8")
        production = source.split("#[cfg(test)]", 1)[0]
        self.assertIn("ui.set_native_floating_window_presentation(", production)
        self.assertNotIn("ui.get_host_presentation_generation(", production)
        self.assertNotIn("ui.update_host_presentation(", production)
        self.assertNotIn("apply_native_floating_presentation_data(", production)

    def test_scoped_transaction_does_not_reenter_the_global_hit_index_builder(self) -> None:
        state = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/globals/state.rs"
        ).read_text(encoding="utf-8")
        transaction = state.split(
            "pub(crate) fn patch_host_presentation_panes(", 1
        )[1].split("pub(crate) fn patch_workbench_window_nodes(", 1)[0]

        self.assertIn(".rebind_paint_models(&model_replacements)", transaction)
        self.assertNotIn("HostWorkbenchHitIndex::from_presentation", transaction)
        self.assertNotIn("indexes_presentation", transaction)

        scoped = (
            ROOT / "zircon_editor/src/ui/retained_host/ui/scoped_presentation.rs"
        ).read_text(encoding="utf-8")
        production = scoped.split("#[cfg(test)]\n#[derive(Default)]", 1)[0]
        self.assertIn("ui.patch_host_presentation_panes(", production)
        self.assertNotIn("ui.update_host_presentation_if(", production)

    def test_source_binding_fails_closed_when_an_authority_changes(self) -> None:
        self.assertTrue(hasattr(pressure, "CRITICAL_SOURCE_CONTRACTS"))
        self.assertTrue(hasattr(pressure, "SourceContractError"))
        self.assertTrue(hasattr(pressure, "source_binding_report"))
        with tempfile.TemporaryDirectory(dir=Path(tempfile.gettempdir())) as directory:
            root = Path(directory)
            for relative_path, _ in pressure.CRITICAL_SOURCE_CONTRACTS:
                path = root / relative_path
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("authority intentionally changed\n", encoding="utf-8")

            with self.assertRaises(pressure.SourceContractError):
                pressure.source_binding_report(root)

    def test_invalid_inputs_are_rejected(self) -> None:
        with self.assertRaises(ValueError):
            run(
                pane_count=0,
                nodes_per_pane=1,
                stable_update_count=1,
                changed_update_count=1,
                changed_panes_per_update=1,
            )
        with self.assertRaises(ValueError):
            run(
                pane_count=2,
                nodes_per_pane=1,
                stable_update_count=1,
                changed_update_count=1,
                changed_panes_per_update=3,
            )

    def test_output_is_stable_json(self) -> None:
        result = run(
            pane_count=2,
            nodes_per_pane=8,
            stable_update_count=3,
            changed_update_count=4,
            changed_panes_per_update=1,
        )
        with tempfile.TemporaryDirectory(dir=Path(tempfile.gettempdir())) as directory:
            output = Path(directory) / "pressure.json"
            write_result(output, result)

            self.assertEqual(json.loads(output.read_text(encoding="utf-8")), result)
            self.assertTrue(output.read_text(encoding="utf-8").endswith("\n"))


if __name__ == "__main__":
    unittest.main()

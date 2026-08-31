import unittest
from pathlib import Path


class RuntimeRenderPipelinePostprocessPluginOwnerStructureTests(unittest.TestCase):
    def test_plugin_input_routes_are_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        routes_path = (
            repo_root
            / "zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests/postprocess_routes.rs"
        )
        plugin_inputs_path = (
            repo_root
            / "zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests/postprocess_routes/plugin_inputs.rs"
        )

        routes = routes_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(routes.splitlines()), 800)

        plugin_inputs = plugin_inputs_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(plugin_inputs.splitlines()), 800)
        self.assertIn("mod plugin_inputs;", routes)
        self.assertEqual(plugin_inputs.count("#[test]"), 6)

        for test_name in (
            "compile_orders_plugin_scene_velocity_load_after_temporal_velocity_producer",
            "compile_filters_plugin_scene_velocity_pass_without_post_process_stack",
            "compile_filters_plugin_scene_velocity_pass_when_stack_does_not_use_scene_velocity",
            "compile_filters_hybrid_gi_lighting_from_uber_without_stack_input",
            "compile_routes_hybrid_gi_lighting_into_uber_when_stack_requests_current_input",
            "compile_keeps_hybrid_gi_lighting_single_sample_when_graph_msaa_is_enabled",
        ):
            self.assertNotIn(f"fn {test_name}", routes)
            self.assertIn(f"fn {test_name}", plugin_inputs)

        for helper_name in (
            "particle_velocity_descriptor",
            "hybrid_gi_lighting_descriptor",
            "particle_extract",
        ):
            self.assertNotIn(f"fn {helper_name}", routes)
            self.assertIn(f"fn {helper_name}", plugin_inputs)

        for concurrent_anchor in (
            "!fxaa.flags.has_side_effects",
            "!smaa.flags.has_side_effects",
        ):
            self.assertIn(concurrent_anchor, routes)

    def test_runtime_budget_guard_reads_plugin_input_owner(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        budget_guard = (
            repo_root
            / "zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_pipeline_asset_compile_tests.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "render_pipeline_asset/compile_tests/postprocess_routes/plugin_inputs.rs",
            budget_guard,
        )
        self.assertIn("plugin_inputs.as_str()", budget_guard)

    def test_plan_and_module_docs_record_plugin_input_owner(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        child_path = "postprocess_routes/plugin_inputs.rs"
        status = (
            "runtime_01_15_render_pipeline_postprocess_plugin_input_owner_split_"
            "static_passed_cargo_deferred"
        )
        docs = (
            "docs/plans/engine-code-structure-convention.md",
            "docs/plans/engine-code-review-findings-2026-06.md",
            "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            "docs/plans/zircon_runtime/render/01/2026-07-09-render-graph-rdg-alignment-output-records.md",
        )

        for relative_path in docs:
            source = (repo_root / relative_path).read_text(encoding="utf-8")
            self.assertIn(child_path, source, relative_path)
            self.assertIn(status, source, relative_path)


if __name__ == "__main__":
    unittest.main()

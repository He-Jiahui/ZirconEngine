import unittest
from pathlib import Path

import sys as _history_sys
from pathlib import Path as _HistoryPath
_HISTORY_REPO_ROOT = _HistoryPath(__file__).resolve().parents[2]
_history_sys.path.insert(0, str(_HISTORY_REPO_ROOT / '.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts'))
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import HistoricalArchiveReader as _ContractHistoryReader
_HISTORY_PATHS = {'docs/plans/zircon_runtime/render/01/2026-07-09-render-graph-rdg-alignment-output-records.md': 'docs/plans/zircon_runtime/render/01/2026-07-09-render-graph-rdg-alignment-output-records.md'}
_HISTORY_IDENTITIES = {'docs/plans/zircon_runtime/render/01/2026-07-09-render-graph-rdg-alignment-output-records.md': (141172, '83dc0bc386b0b143e7d8a001f350a3ef9f345a15a69cea70340967ffe515f8c4')}
_CURRENT_POLICY_INPUTS = ('docs/architecture/runtime-architecture-review-m0.md', 'docs/testing-and-validation/development-conventions.md', 'docs/crates/zircon_runtime/structure/module-convention.md')

def _read_contract_input(path, *args, **kwargs):
    candidate = _HistoryPath(path)
    try:
        relative = candidate.absolute().relative_to(_HISTORY_REPO_ROOT).as_posix()
    except ValueError:
        relative = None
    key = _HISTORY_PATHS.get(relative)
    if key is None:
        if relative and relative.startswith('docs/plans/') and relative not in _CURRENT_POLICY_INPUTS:
            raise AssertionError('Unregistered current/history contract input: ' + relative)
        return candidate.read_text(*args, **kwargs)
    if args or kwargs.get('encoding', 'utf-8').lower() not in ('utf-8', 'utf8') or kwargs.get('errors') not in (None, 'strict') or set(kwargs) - {'encoding', 'errors'}:
        raise AssertionError('Historical evidence requires strict UTF-8')
    reader = _ContractHistoryReader(identities=_HISTORY_IDENTITIES)
    text = reader.read(key)
    if text is None:
        raise AssertionError(reader.records)
    return text


# 验证渲染流水线后处理插件的职责切分：插件输入路由应子节点持有，并运行时预算守卫读取插件输入归属。
class RuntimeRenderPipelinePostprocessPluginOwnerStructureTests(unittest.TestCase):
    def test_plugin_input_routes_are_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        routes_path = (
            repo_root / "zircon_runtime/src/graphics/pipeline/pipeline/tests/postprocess_routes.rs"
        )
        plugin_inputs_path = (
            repo_root / "zircon_runtime/src/graphics/pipeline/pipeline/tests/plugin_inputs.rs"
        )

        routes = _read_contract_input(routes_path, encoding="utf-8")
        self.assertLessEqual(len(routes.splitlines()), 800)

        plugin_inputs = _read_contract_input(plugin_inputs_path, encoding="utf-8")
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
        budget_guard = _read_contract_input(repo_root
            / "zircon_runtime/src/tests/structure/source_size/render_pipeline_asset_compile.rs", encoding="utf-8")

        self.assertIn(
            "render_pipeline_asset/compile_tests/postprocess_routes/tests/plugin_inputs.rs",
            budget_guard,
        )
        self.assertIn("plugin_inputs.as_str()", budget_guard)

    def test_plan_and_module_docs_record_plugin_input_owner(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        child_path = "zircon_runtime/src/graphics/pipeline/pipeline/tests/plugin_inputs.rs"
        historical_child_path = "postprocess_routes/tests/plugin_inputs.rs"
        status = (
            "runtime_01_15_render_pipeline_postprocess_plugin_input_owner_split_"
            "static_passed_cargo_deferred"
        )
        docs = (
            'docs/testing-and-validation/development-conventions.md',
            'docs/architecture/runtime-architecture-review-m0.md',
            'docs/crates/zircon_runtime/structure/module-convention.md',
            "docs/plans/zircon_runtime/render/01/2026-07-09-render-graph-rdg-alignment-output-records.md",
        )

        for relative_path in docs:
            source = _read_contract_input(repo_root / relative_path, encoding="utf-8")
            expected_owner = historical_child_path if relative_path.startswith("docs/plans/") else child_path
            self.assertIn(expected_owner, source, relative_path)
            self.assertIn(status, source, relative_path)


if __name__ == "__main__":
    unittest.main()

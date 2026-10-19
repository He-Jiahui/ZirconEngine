import unittest
from pathlib import Path

import sys as _history_sys
from pathlib import Path as _HistoryPath
_HISTORY_REPO_ROOT = _HistoryPath(__file__).resolve().parents[2]
_history_sys.path.insert(0, str(_HISTORY_REPO_ROOT / '.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts'))
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import HistoricalArchiveReader as _ContractHistoryReader
_HISTORY_PATHS = {'docs/plans/zircon_runtime/render/07/2026-07-09-postprocess-color-pipeline-output-records.md': 'docs/plans/zircon_runtime/render/07/2026-07-09-postprocess-color-pipeline-output-records.md'}
_HISTORY_IDENTITIES = {'docs/plans/zircon_runtime/render/07/2026-07-09-postprocess-color-pipeline-output-records.md': (96345, '2fdd6266cdc42e8967c18b565d0eba255d5af914a4b38affdb9ee82c086cc522')}
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


# 约束场景后 process 体积式的测试组织：体积式雾提取测试应子节点持有，并计划与模块 docs 记录体积式测试归属。
class RuntimeScenePostProcessVolumetricTestStructureTests(unittest.TestCase):
    def test_volumetric_fog_extract_tests_are_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = (
            repo_root / "zircon_runtime/src/scene/tests/render_post_process_extract.rs"
        )
        child_path = (
            repo_root
            / "zircon_runtime/src/scene/tests/render_post_process_extract/volumetric_fog.rs"
        )

        owner = _read_contract_input(owner_path, encoding="utf-8")
        self.assertLessEqual(len(owner.splitlines()), 800)

        child = _read_contract_input(child_path, encoding="utf-8")
        self.assertLessEqual(len(child.splitlines()), 800)
        self.assertIn("mod volumetric_fog;", owner)
        self.assertEqual(owner.count("#[test]"), 11)
        self.assertEqual(child.count("#[test]"), 3)

        for test_name in (
            "render_volumetric_explicit_camera_uses_culling_mask_for_local_fog_volumes",
            "render_volumetric_scene_local_profile_and_light_marker_feed_advanced_extract",
            "render_volumetric_local_volume_rejects_invalid_bounds",
        ):
            self.assertNotIn(f"fn {test_name}", owner)
            self.assertIn(f"fn {test_name}", child)

        self.assertNotIn("fn spawn_local_volumetric_box", owner)
        self.assertIn("fn spawn_local_volumetric_box", child)
        for concurrent_anchor in (
            "spawn_node(NodeKind::DirectionalLight)\n        .expect",
            "spawn_node(NodeKind::Empty)\n        .expect",
        ):
            self.assertIn(concurrent_anchor, child)

        for retained_anchor in (
            "spawn_node(NodeKind::Mesh)\n        .expect",
            "spawn_node(NodeKind::Camera)\n        .expect",
        ):
            self.assertIn(retained_anchor, owner)

    def test_plan_and_module_docs_record_volumetric_test_owner(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        child_path = "render_post_process_extract/volumetric_fog.rs"
        status = (
            "runtime_07_15_scene_post_process_volumetric_fog_test_owner_split_"
            "static_passed_cargo_deferred"
        )
        plan_docs = (
            'docs/testing-and-validation/development-conventions.md',
            'docs/architecture/runtime-architecture-review-m0.md',
            'docs/crates/zircon_runtime/structure/module-convention.md',
            "docs/plans/zircon_runtime/render/07/2026-07-09-postprocess-color-pipeline-output-records.md",
        )

        for relative_path in plan_docs:
            source = _read_contract_input(repo_root / relative_path, encoding="utf-8")
            self.assertIn(child_path, source, relative_path)
            self.assertIn(status, source, relative_path)

        for relative_path in (
            "docs/crates/zircon_runtime/scene/render_extract.md",
            "docs/crates/zircon_runtime/graphics/scene/scene_renderer/advanced_lighting/volumetric-media-inject.md",
        ):
            source = _read_contract_input(repo_root / relative_path, encoding="utf-8")
            self.assertIn(child_path, source, relative_path)


if __name__ == "__main__":
    unittest.main()

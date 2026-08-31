import unittest
from pathlib import Path


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

        owner = owner_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(owner.splitlines()), 800)

        child = child_path.read_text(encoding="utf-8")
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
            "docs/plans/engine-code-structure-convention.md",
            "docs/plans/engine-code-review-findings-2026-06.md",
            "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            "docs/plans/zircon_runtime/render/07/2026-07-09-postprocess-color-pipeline-output-records.md",
        )

        for relative_path in plan_docs:
            source = (repo_root / relative_path).read_text(encoding="utf-8")
            self.assertIn(child_path, source, relative_path)
            self.assertIn(status, source, relative_path)

        for relative_path in (
            "docs/zircon_runtime/scene/render_extract.md",
            "docs/zircon_runtime/graphics/scene/scene_renderer/advanced_lighting/volumetric-media-inject.md",
        ):
            source = (repo_root / relative_path).read_text(encoding="utf-8")
            self.assertIn(child_path, source, relative_path)


if __name__ == "__main__":
    unittest.main()

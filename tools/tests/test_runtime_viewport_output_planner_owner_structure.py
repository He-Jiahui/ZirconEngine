import unittest
from pathlib import Path


class RuntimeViewportOutputPlannerOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_render_09_15_viewport_output_planner_owner_split_"
        "static_passed_cargo_deferred"
    )

    def test_writeback_and_graph_import_planners_are_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = (
            repo_root
            / "zircon_runtime/src/graphics/types/viewport_render_output_target.rs"
        )
        owner = owner_path.read_text(encoding="utf-8")
        owner_dir = owner_path.with_suffix("")
        writeback = (owner_dir / "writeback.rs").read_text(encoding="utf-8")
        graph_import = (owner_dir / "graph_import.rs").read_text(encoding="utf-8")
        tests = (owner_dir / "tests.rs").read_text(encoding="utf-8")

        self.assertLessEqual(len(owner.splitlines()), 110)
        self.assertLessEqual(len(writeback.splitlines()), 250)
        self.assertLessEqual(len(graph_import.splitlines()), 250)
        self.assertLessEqual(len(tests.splitlines()), 300)
        self.assertIn("mod graph_import;", owner)
        self.assertIn("mod writeback;", owner)
        self.assertIn("mod tests;", owner)
        self.assertIn("pub(crate) use graph_import::", owner)
        self.assertIn("pub(crate) use writeback::", owner)
        self.assertNotIn("fn writeback_plan", owner)
        self.assertNotIn("fn graph_import_plan", owner)

        self._assert_anchors_are_ordered(
            writeback,
            (
                "impl ViewportRenderOutputTarget",
                "pub(crate) fn writeback_plan",
                "pub(crate) struct ViewportTextureWritebackPlan",
                "impl ViewportTextureWritebackPlan",
                "pub(crate) enum ViewportTextureWritebackStatus",
            ),
        )
        self._assert_anchors_are_ordered(
            writeback.split("pub(crate) fn writeback_plan", 1)[1],
            (
                "pending_descriptor",
                "blocked_prepared_format_mismatch",
                "FRAMEWORK_OUTPUT_FORMAT_LABEL",
                "ready_for_conversion",
                "blocked_format",
            ),
        )

        self._assert_anchors_are_ordered(
            graph_import,
            (
                "impl ViewportRenderOutputTarget",
                "pub(crate) fn graph_import_plan",
                "pub(crate) struct ViewportTextureGraphImportPlan",
                "impl ViewportTextureGraphImportPlan",
                "pub(crate) enum ViewportTextureGraphImportStatus",
            ),
        )
        self._assert_anchors_are_ordered(
            graph_import.split("pub(crate) fn graph_import_plan", 1)[1],
            (
                "pending_descriptor",
                "blocked_prepared_format_mismatch",
                "ready_for_direct_import",
                "requires_conversion_writeback",
                "blocked_format",
            ),
        )

        self.assertEqual(tests.count("#[test]"), 12)
        self.assertIn('include_str!("writeback.rs")', tests)
        self.assertIn('include_str!("graph_import.rs")', tests)

    def test_owner_split_status_is_mirrored(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        mirrors = (
            repo_root / "docs/plans/zircon_runtime/render/09-camera-render-ordering.md",
            repo_root
            / "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            repo_root / "docs/plans/engine-code-structure-convention.md",
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md",
        )
        for mirror_path in mirrors:
            mirror = mirror_path.read_text(encoding="utf-8")
            self.assertIn(self.STATUS, mirror, mirror_path.as_posix())

        structure_plan = mirrors[1].read_text(encoding="utf-8")
        for current_path in (
            "zircon_runtime/src/graphics/types/viewport_render_output_target.rs",
            "zircon_runtime/src/graphics/types/viewport_render_output_target/writeback.rs",
            "zircon_runtime/src/graphics/types/viewport_render_output_target/graph_import.rs",
            "zircon_runtime/src/graphics/types/viewport_render_output_target/tests.rs",
            "tools/tests/test_runtime_viewport_output_planner_owner_structure.py",
        ):
            self.assertIn(current_path, structure_plan)

    def _assert_anchors_are_ordered(self, source: str, anchors: tuple[str, ...]) -> None:
        positions = [source.index(anchor) for anchor in anchors]
        self.assertEqual(positions, sorted(positions))


if __name__ == "__main__":
    unittest.main()

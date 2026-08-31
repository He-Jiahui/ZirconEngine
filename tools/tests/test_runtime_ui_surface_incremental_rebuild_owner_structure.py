import unittest
from pathlib import Path


class RuntimeUiSurfaceIncrementalRebuildOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_09_15_ui_surface_incremental_rebuild_owner_split_"
        "static_passed_cargo_profile_deferred"
    )

    def test_incremental_rebuild_is_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = repo_root / "zircon_runtime/src/ui/surface/surface/rebuild.rs"
        child_path = (
            repo_root / "zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs"
        )

        owner = owner_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(owner.splitlines()), 800)
        self.assertIn("mod incremental;", owner)
        self.assertNotIn("pub fn rebuild_dirty", owner)
        self.assertNotIn("fn should_use_full_layout_rebuild", owner)
        self.assertNotIn("fn merge_incremental_layout_engine_report", owner)
        self.assertNotIn("fn patch_incremental_layout_engine_report", owner)

        child = child_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(child.splitlines()), 800)
        for anchor in (
            "pub fn rebuild_dirty",
            "fn should_use_full_layout_rebuild",
            "fn merge_incremental_layout_engine_report",
            "fn patch_incremental_layout_engine_report",
            "UI_LAYOUT_INCREMENTAL_MAX_DIRTY_RATIO_DENOMINATOR",
            "UI_LAYOUT_INCREMENTAL_MAX_DIRTY_NODE_COUNT",
            "invalidate_for_changed_text_font_generation",
            "ui.layout.full_rebuild_threshold_count",
            "patch_arranged_tree_geometry",
            "patch_arranged_tree_input",
            "navigation_index_patch_changed_geometry",
            "mark_surface_frame_rebuild_dirty",
            "publish_surface_frame_after_rebuild",
            "record_surface_rebuild_profile",
        ):
            self.assertIn(anchor, child)

        for concurrent_anchor in (
            "arranged_visibility",
            "rebuild_render_extract_with_text_frame",
            "pub fn compute_layout",
            "invalidate_measure",
            "publish_surface_frame_after_rebuild",
        ):
            self.assertIn(concurrent_anchor, owner)

    def test_incremental_rebuild_owner_status_is_mirrored(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        mirrors = (
            repo_root
            / "docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md",
            repo_root
            / "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            repo_root
            / "docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md",
            repo_root
            / "docs/plans/zircon_runtime/runtime/09/2026-08-07-runtime-ui-incremental-refresh.md",
            repo_root / "docs/zircon_runtime/ui/architecture.md",
            repo_root / "docs/plans/engine-code-structure-convention.md",
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md",
        )
        for mirror_path in mirrors:
            mirror = mirror_path.read_text(encoding="utf-8")
            self.assertIn(self.STATUS, mirror, mirror_path.as_posix())

        runtime_plan = mirrors[1].read_text(encoding="utf-8")
        for current_path in (
            "zircon_runtime/src/ui/surface/surface/rebuild.rs",
            "zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs",
            "tools/tests/test_runtime_ui_surface_incremental_rebuild_owner_structure.py",
        ):
            self.assertIn(current_path, runtime_plan)


if __name__ == "__main__":
    unittest.main()

import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.runtime_plan_status_boundary import (  # noqa: E402
    runtime_plan_status_boundary_audit,
)
from runtime_structure_audits.runtime_plan_status_sources import (  # noqa: E402
    markdown_repo_link_targets,
    runtime_numbered_archives,
)


class RuntimePlanStatusCanonicalArchiveSourcesTests(unittest.TestCase):
    def test_markdown_links_require_the_exact_canonical_archive_target(self) -> None:
        plan_path = (
            REPO_ROOT
            / "docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
        )
        canonical = (
            "docs/plans/_archive/zircon_runtime/runtime/05/"
            "2026-07-09-scene-editor-boundary-closeout-output-records.md"
        )
        active = (
            "docs/plans/zircon_runtime/runtime/05/"
            "2026-07-09-scene-editor-boundary-closeout-output-records.md"
        )
        file_name = "2026-07-09-scene-editor-boundary-closeout-output-records.md"

        correct_targets = markdown_repo_link_targets(
            REPO_ROOT,
            plan_path,
            f"[record](../../_archive/zircon_runtime/runtime/05/{file_name})",
        )
        wrong_targets = markdown_repo_link_targets(
            REPO_ROOT,
            plan_path,
            f"[record](05/{file_name})",
        )
        bare_targets = markdown_repo_link_targets(REPO_ROOT, plan_path, file_name)

        self.assertEqual({canonical}, correct_targets)
        self.assertEqual({active}, wrong_targets)
        self.assertNotIn(canonical, wrong_targets)
        self.assertEqual(set(), bare_targets)

    def test_numbered_sources_include_active_children_and_canonical_archives(self) -> None:
        sources = runtime_numbered_archives(REPO_ROOT)
        runtime_05_paths = [path for path, _ in sources["05"]]
        runtime_15_paths = [path for path, _ in sources["15"]]

        self.assertTrue(
            any(path.startswith("docs/plans/zircon_runtime/runtime/05/") for path in runtime_05_paths)
        )
        self.assertIn(
            "docs/plans/_archive/zircon_runtime/runtime/05/"
            "2026-07-09-scene-editor-boundary-closeout-output-records.md",
            runtime_05_paths,
        )
        self.assertIn(
            "docs/plans/_archive/zircon_runtime/runtime/15/"
            "2026-07-09-runtime-index-output-records.md",
            runtime_15_paths,
        )

    def test_plan_status_uses_canonical_archives_for_status_and_mirror_anchors(self) -> None:
        report = runtime_plan_status_boundary_audit(REPO_ROOT)

        self.assertEqual([], report["status_table_gaps"])
        self.assertEqual([], report["missing_runtime_02_generated_status_index_anchors"])
        self.assertEqual([], report["missing_runtime_10_behavior_status_doc_anchors"])


if __name__ == "__main__":
    unittest.main()

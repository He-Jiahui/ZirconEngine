import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.ui_architecture_boundary import (  # noqa: E402
    ui_architecture_boundary_audit,
)


class RuntimeUiArchitectureBoundaryTests(unittest.TestCase):
    def test_current_text_surface_leaves_and_index_route_are_mirrored(self) -> None:
        report = ui_architecture_boundary_audit(REPO_ROOT)

        self.assertEqual([], report["surface_missing_entries"])
        self.assertEqual([], report["surface_unexpected_entries"])
        self.assertEqual([], report["missing_required_doc_mentions"])
        self.assertEqual([], report["risks"])


if __name__ == "__main__":
    unittest.main()

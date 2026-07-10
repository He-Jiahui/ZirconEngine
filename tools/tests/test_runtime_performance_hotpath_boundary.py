import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.performance_hotpath_boundary import (  # noqa: E402
    performance_hotpath_boundary_audit,
)


class RuntimePerformanceHotpathBoundaryTests(unittest.TestCase):
    def test_numbered_archive_and_current_backend_command_supply_doc_evidence(self) -> None:
        report = performance_hotpath_boundary_audit(REPO_ROOT)

        self.assertEqual([], report["missing_doc_anchors"])
        self.assertEqual([], report["missing_cargo_gate_anchors"])
        self.assertEqual([], report["risks"])


if __name__ == "__main__":
    unittest.main()

import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.dynamic_runtime_api_boundary import (  # noqa: E402
    dynamic_runtime_api_boundary_audit,
)


class RuntimeDynamicApiBoundaryArchiveOwnershipTests(unittest.TestCase):
    def test_numbered_runtime_09_10_archives_supply_concrete_contract_evidence(self) -> None:
        report = dynamic_runtime_api_boundary_audit(REPO_ROOT)

        self.assertEqual([], report["missing_ui_pending_gate_anchors"])
        self.assertEqual([], report["missing_ui_contract_single_source_anchors"])
        self.assertEqual([], report["missing_ui_v2_contract_sync_anchors"])
        self.assertEqual([], report["missing_doc_anchors"])
        self.assertEqual([], report["risks"])


if __name__ == "__main__":
    unittest.main()

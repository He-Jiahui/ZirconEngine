import sys
import unittest
from pathlib import Path
from unittest.mock import patch


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.performance_hotpath_boundary import (  # noqa: E402
    performance_hotpath_boundary_audit,
)
from runtime_structure_audits import performance_hotpath_boundary  # noqa: E402


class RuntimePerformanceHotpathBoundaryTests(unittest.TestCase):
    def test_numbered_archive_and_current_backend_command_supply_doc_evidence(self) -> None:
        report = performance_hotpath_boundary_audit(REPO_ROOT)

        self.assertEqual([], report["missing_doc_anchors"])
        self.assertEqual([], report["missing_cargo_gate_anchors"])

    def test_test_inventory_count_is_derived_from_the_manifest(self) -> None:
        manifest = (
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs",
        )
        with patch.object(performance_hotpath_boundary, "RUNTIME_07_TEST_FILES", manifest):
            report = performance_hotpath_boundary_audit(REPO_ROOT)

        self.assertEqual(len(manifest), report["expected_test_file_count"])
        self.assertEqual(len(manifest), len(report["test_files"]))


if __name__ == "__main__":
    unittest.main()

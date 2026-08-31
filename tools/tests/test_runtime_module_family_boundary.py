import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.module_family_boundary import (  # noqa: E402
    MODULE_FAMILY_ROOTS,
    module_family_boundary_audit,
)


class RuntimeModuleFamilyBoundaryTests(unittest.TestCase):
    def test_navigation_family_includes_folder_backed_operation_owners(self) -> None:
        audit = module_family_boundary_audit(REPO_ROOT)
        navigation = next(
            family for family in audit["families"] if family["family"] == "navigation"
        )

        self.assertEqual(MODULE_FAMILY_ROOTS["navigation"]["expected_file_count"], 16)
        self.assertEqual(navigation["rust_file_count"], 16)
        self.assertEqual(navigation["expected_file_count"], 16)
        self.assertEqual(audit["file_count_mismatches"], [])
        self.assertEqual(audit["missing_doc_anchors"], [])
        self.assertEqual(audit["missing_guards"], [])
        self.assertEqual(audit["risks"], [])


if __name__ == "__main__":
    unittest.main()

import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.runtime_api_boundary import (  # noqa: E402
    RUNTIME_API_MODULES,
    runtime_api_boundary_audit,
)


class RuntimeApiBoundaryTests(unittest.TestCase):
    def test_runtime_api_owner_inventory_matches_the_v2_abi_domains(self) -> None:
        expected_modules = (
            "api_table",
            "constants",
            "events",
            "host_requests",
            "operation",
            "plugin_event_mirror",
            "requests",
            "viewport",
        )

        audit = runtime_api_boundary_audit(REPO_ROOT)

        self.assertEqual(expected_modules, RUNTIME_API_MODULES)
        self.assertEqual(audit["expected_module_count"], len(expected_modules))
        self.assertEqual(audit["missing_modules"], [])
        self.assertEqual(audit["unexpected_modules"], [])
        self.assertEqual(audit["missing_mod_declarations"], [])
        self.assertEqual(audit["missing_reexports"], [])
        self.assertEqual(audit["facade_forbidden_locations"], [])
        self.assertEqual(audit["oversized_modules"], [])
        self.assertEqual(audit["risks"], [])


if __name__ == "__main__":
    unittest.main()

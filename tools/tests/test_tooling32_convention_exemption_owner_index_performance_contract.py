from __future__ import annotations

import unittest
from pathlib import Path

from tools.convention_exemptions import _owning_workspace_member


SCRIPT = Path(__file__).resolve().parents[1] / "convention_exemptions.py"


class ConventionExemptionOwnerIndexPerformanceContractTests(unittest.TestCase):
    def test_owner_lookup_uses_the_nearest_indexed_parent(self) -> None:
        workspace = Path("E:/workspace")
        member_by_root = {
            workspace: ".",
            workspace / "crates": "crates",
            workspace / "crates" / "nested": "crates/nested",
        }

        self.assertEqual(
            "crates/nested",
            _owning_workspace_member(
                workspace / "crates" / "nested" / "src" / "lib.rs",
                member_by_root,
            ),
        )

    def test_candidate_filter_uses_a_root_set(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("member_root_set = {root for _, root in member_roots}", source)
        self.assertIn("parent in member_root_set for parent in candidate.parents", source)
        self.assertNotIn("member_root in candidate.parents", source)


if __name__ == "__main__":
    unittest.main()

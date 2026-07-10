from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.plans import PlanRepository


class PlanRepositoryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        self.plan = self.root / "docs/plans/zircon_runtime/frameworks/02-module-kernel.md"
        self.child = self.plan.parent / "02"
        self.child.mkdir(parents=True)
        self.plan.write_text("# Frameworks 02\n", encoding="utf-8")
        (self.child / "2026-07-11-output.md").write_text("# Output\n", encoding="utf-8")
        (self.plan.parent / "index.md").write_text("# Index\n", encoding="utf-8")
        (self.root / "docs/plans/engine-code-review.md").write_text("# Global\n", encoding="utf-8")
        legacy = self.root / ".codex/plans/legacy-plan.md"
        legacy.parent.mkdir(parents=True)
        legacy.write_text("# Legacy\n", encoding="utf-8")
        self.repository = PlanRepository(self.root)

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def test_recursive_scan_separates_formal_plans_and_legacy_documents(self) -> None:
        inventory = self.repository.scan()

        self.assertEqual(["docs/plans/zircon_runtime/frameworks/02-module-kernel.md"], [item.path for item in inventory.formal_plans])
        self.assertEqual([".codex/plans/legacy-plan.md"], list(inventory.legacy_documents))
        self.assertEqual("docs/plans/zircon_runtime/frameworks/02", inventory.formal_plans[0].child_dir)

    def test_only_registered_child_directory_is_writable(self) -> None:
        allowed = self.repository.authorize_write(self.plan, self.child / "new-output.md")
        index = self.repository.authorize_write(self.plan, self.plan.parent / "index.md")
        definition = self.repository.authorize_write(self.plan, self.plan)
        sibling = self.repository.authorize_write(
            self.plan, self.root / "docs/plans/zircon_runtime/frameworks/03/foreign.md"
        )

        self.assertTrue(allowed.allowed)
        self.assertEqual("docs/plans/zircon_runtime/frameworks/02", allowed.owner_child_dir)
        self.assertFalse(index.allowed)
        self.assertEqual("protected_global_plan", index.code)
        self.assertFalse(definition.allowed)
        self.assertEqual("protected_plan_definition", definition.code)
        self.assertFalse(sibling.allowed)
        self.assertEqual("outside_registered_child", sibling.code)

    def test_maintenance_mode_is_explicit_and_still_repo_bounded(self) -> None:
        maintained = self.repository.authorize_write(
            self.plan, self.plan.parent / "index.md", maintenance=True
        )
        outside = self.repository.authorize_write(
            self.plan, self.root.parent / "outside.md", maintenance=True
        )
        non_plan = self.repository.authorize_write(
            self.plan, self.root / "README.md", maintenance=True
        )

        self.assertTrue(maintained.allowed)
        self.assertFalse(outside.allowed)
        self.assertEqual("path_outside_repo", outside.code)
        self.assertFalse(non_plan.allowed)
        self.assertEqual("outside_plan_root", non_plan.code)


if __name__ == "__main__":
    unittest.main()

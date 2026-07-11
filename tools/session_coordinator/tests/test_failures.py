from __future__ import annotations

import tempfile
import unittest
from datetime import date
from pathlib import Path
from unittest import mock

from tools.session_coordinator.database import Database
from tools.session_coordinator.failures import FailureGraphService, FailureResolution
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.tests.failure_fixture import FailureGraphFixture


class FailureGraphTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        self.database = Database(self.root / "state/coordinator.sqlite3")
        migrate(self.database)
        self.fixture = FailureGraphFixture(self.root)
        self.service = FailureGraphService(self.database, self.root)

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def test_import_prioritizes_open_failures_for_fixing_plan(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        self.fixture.add_handoff(origin, fixing, "second", created_at="2026-07-12")
        self.fixture.add_handoff(origin, fixing, "first", created_at="2026-07-11")

        audit = self.service.import_repository()
        open_nodes = self.service.open_for_plan(fixing.path)

        self.assertEqual(2, audit.node_count)
        self.assertEqual([], [item for item in audit.diagnostics if item.code != "duplicate_plan_edge"])
        self.assertEqual(["first", "second"], [node.summary_slug for node in open_nodes])

    def test_cycle_self_edge_and_duplicate_lifecycle_are_reported(self) -> None:
        plan_a = self.fixture.add_plan("docs/plans/a/01-a.md")
        plan_b = self.fixture.add_plan("docs/plans/b/02-b.md")
        self.fixture.add_handoff(plan_a, plan_b, "a-to-b")
        self.fixture.add_handoff(plan_b, plan_a, "b-to-a")
        self.fixture.add_handoff(plan_a, plan_a, "self")
        self.fixture.add_handoff(plan_a, plan_b, "duplicate")
        self.fixture.add_handoff(plan_a, plan_b, "duplicate", kind="fixed")

        audit = self.service.import_repository()
        codes = {diagnostic.code for diagnostic in audit.diagnostics}

        self.assertIn("cycle", codes)
        self.assertIn("self_edge", codes)
        self.assertIn("duplicate_lifecycle", codes)

    def test_excessive_dependency_depth_is_reported(self) -> None:
        plans = [
            self.fixture.add_plan(f"docs/plans/depth/{index:02d}-plan-{index}.md")
            for index in range(1, 5)
        ]
        for index in range(3):
            self.fixture.add_handoff(plans[index], plans[index + 1], f"depth-{index}")
        service = FailureGraphService(self.database, self.root, max_depth=2)

        codes = {item.code for item in service.import_repository().diagnostics}

        self.assertIn("excessive_depth", codes)

    def test_invalid_artifact_status_is_diagnostic_not_graph_import_failure(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        failure = self.fixture.add_handoff(origin, fixing, "foreign-status")
        failure.write_text(
            failure.read_text(encoding="utf-8").replace("status: open", "status: resolved"),
            encoding="utf-8",
        )

        audit = self.service.import_repository()

        self.assertEqual(1, audit.node_count)
        self.assertEqual("open", audit.nodes[0].status)
        self.assertIn("schema_validation", {item.code for item in audit.diagnostics})

    def test_validator_errors_for_plan_excludes_unrelated_handoff_diagnostics(self) -> None:
        current = self.fixture.add_plan("docs/plans/current/03-current.md")
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        failure = self.fixture.add_handoff(origin, fixing, "foreign-status")
        failure.write_text(
            failure.read_text(encoding="utf-8").replace("status: open", "status: resolved"),
            encoding="utf-8",
        )

        self.assertEqual([], self.service.validator_errors_for_plan(current.path))
        self.assertNotEqual([], self.service.validator_errors_for_plan(origin.path))
        self.assertNotEqual([], self.service.validator_errors_for_plan(fixing.path))

    def test_verified_fix_moves_back_and_updates_both_relative_links(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        failure = self.fixture.add_handoff(origin, fixing, "provider")
        audit = self.service.import_repository()
        node = next(node for node in audit.nodes if node.summary_slug == "provider")

        fixed = self.service.return_fixed(
            node.lifecycle_key,
            FailureResolution(
                root_cause="provider identity came from the wrong owner",
                architecture_fix="one shared provider-key constructor owns all paths",
                validation="lower regression, reproduction, and upward gate passed",
                return_summary="origin M3 gate can resume",
            ),
            resolved_at=date(2026, 7, 12),
        )

        self.assertFalse(failure.exists())
        self.assertEqual(origin.child / "fixed-2026-07-12-provider.md", fixed)
        self.assertIn("handoff_kind: fixed", fixed.read_text(encoding="utf-8"))
        self.assertIn("fixed 已修复", origin.path.read_text(encoding="utf-8"))
        self.assertIn("fixed 已修复", fixing.path.read_text(encoding="utf-8"))
        self.assertNotIn(str(self.root), fixing.path.read_text(encoding="utf-8"))
        self.assertEqual([], self.service.validator_errors())

    def test_return_rolls_back_files_when_atomic_write_fails(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        failure = self.fixture.add_handoff(origin, fixing, "provider")
        original_failure = failure.read_text(encoding="utf-8")
        original_origin = origin.path.read_text(encoding="utf-8")
        original_fixing = fixing.path.read_text(encoding="utf-8")
        node = self.service.import_repository().nodes[0]
        original_atomic_write = self.service._atomic_write
        calls = 0

        def fail_second_write(path: Path, content: str) -> None:
            nonlocal calls
            calls += 1
            if calls == 2:
                raise OSError("injected plan write failure")
            original_atomic_write(path, content)

        with mock.patch.object(self.service, "_atomic_write", side_effect=fail_second_write):
            with self.assertRaises(OSError):
                self.service.return_fixed(
                    node.lifecycle_key,
                    FailureResolution("root", "architecture", "validation", "return"),
                    resolved_at=date(2026, 7, 12),
                )

        self.assertEqual(original_failure, failure.read_text(encoding="utf-8"))
        self.assertEqual(original_origin, origin.path.read_text(encoding="utf-8"))
        self.assertEqual(original_fixing, fixing.path.read_text(encoding="utf-8"))
        self.assertFalse((origin.child / "fixed-2026-07-12-provider.md").exists())


if __name__ == "__main__":
    unittest.main()

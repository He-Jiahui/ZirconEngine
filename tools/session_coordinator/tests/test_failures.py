from __future__ import annotations

import os
import tempfile
import unittest
from types import SimpleNamespace
from datetime import date
from pathlib import Path
from unittest import mock

from tools.session_coordinator.database import Database
from tools.session_coordinator.failures import (
    FailureGraphService,
    FailureResolution,
    failure_artifact_snapshot,
)
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator import migrations as migrations_module
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

    def test_workflow_node_filter_is_exact_for_origin_and_plan_wide_for_fixer(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        legacy = self.fixture.add_handoff(origin, fixing, "legacy")
        slice_one = self.fixture.add_handoff(origin, fixing, "slice-one")
        slice_two = self.fixture.add_handoff(origin, fixing, "slice-two")
        for artifact, node_key in ((slice_one, "M1.1"), (slice_two, "M1.2")):
            artifact.write_text(
                artifact.read_text(encoding="utf-8").replace(
                    "summary_slug:", f"origin_workflow_node: {node_key}\nsummary_slug:"
                ),
                encoding="utf-8",
            )

        audit = self.service.import_repository()
        slice_nodes = self.service.open_related_to_workflow_nodes(
            origin.path, ("M1.2",)
        )
        parent_nodes = self.service.open_related_to_workflow_nodes(
            origin.path, ("M1", "M1.1", "M1.2")
        )
        fixing_nodes = self.service.open_related_to_workflow_nodes(
            fixing.path, ("M9.9",)
        )

        imported = {node.summary_slug: node.origin_workflow_node for node in audit.nodes}
        self.assertEqual(
            {"legacy": None, "slice-one": "M1.1", "slice-two": "M1.2"},
            imported,
        )
        self.assertEqual(
            ["legacy", "slice-two"], [node.summary_slug for node in slice_nodes]
        )
        self.assertEqual(
            ["legacy", "slice-one", "slice-two"],
            [node.summary_slug for node in parent_nodes],
        )
        self.assertEqual(
            ["legacy", "slice-one", "slice-two"],
            [node.summary_slug for node in fixing_nodes],
        )

    def test_fixed_return_manifest_excludes_unrelated_fixer_failures_but_keeps_origin_scope(self) -> None:
        current = self.fixture.add_plan("docs/plans/tooling/01-tooling.md")
        origin = self.fixture.add_plan("docs/plans/editor/02-editor.md")
        other_fixer = self.fixture.add_plan("docs/plans/runtime/03-runtime.md")
        completed_return = self.fixture.add_handoff(origin, current, "completed", kind="fixed")
        unrelated = self.fixture.add_handoff(origin, current, "unrelated")
        origin_scoped = self.fixture.add_handoff(current, other_fixer, "current-slice")
        origin_scoped.write_text(
            origin_scoped.read_text(encoding="utf-8").replace(
                "summary_slug:", "origin_workflow_node: M1.1\nsummary_slug:"
            ),
            encoding="utf-8",
        )
        self.service.import_repository()

        ordinary = self.service.open_for_manifest(
            current.path,
            ("M1.1",),
            (),
        )
        fixed_return = self.service.open_for_manifest(
            current.path,
            ("M1.1",),
            (completed_return.relative_to(self.root).as_posix(),),
        )

        self.assertEqual(
            {"unrelated", "current-slice"},
            {node.summary_slug for node in ordinary},
        )
        self.assertEqual(
            ["current-slice"],
            [node.summary_slug for node in fixed_return],
        )

    def test_child_record_source_slice_requires_complete_related_code_before_commit(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/02-editor.md")
        fixing = self.fixture.add_plan("docs/plans/plugins/02-sound.md")
        failure = self.fixture.add_handoff(origin, fixing, "sound-lock-closure")
        failure.write_text(
            failure.read_text(encoding="utf-8").replace(
                "summary_slug:",
                "plan_link_mode: child_record_only\n"
                "related_code:\n"
                "  - Cargo.lock\n"
                "  - plugins/Cargo.lock\n"
                "  - plugins/sound/runtime/Cargo.toml\n"
                "summary_slug:",
            ),
            encoding="utf-8",
        )
        self.service.import_repository()

        incomplete = self.service.open_for_manifest(
            fixing.path,
            ("M1",),
            ("Cargo.lock", "plugins/Cargo.lock"),
        )
        complete = self.service.open_for_manifest(
            fixing.path,
            ("M1",),
            (
                "Cargo.lock",
                "plugins/Cargo.lock",
                "plugins/sound/runtime/Cargo.toml",
                "docs/plans/plugins/02/2026-07-17-m1-lock-closure.md",
            ),
        )
        piggybacked = self.service.open_for_manifest(
            fixing.path,
            ("M1",),
            (
                "Cargo.lock",
                "plugins/Cargo.lock",
                "plugins/sound/runtime/Cargo.toml",
                "plugins/unrelated/runtime/lib.rs",
                "docs/plans/plugins/02/2026-07-17-m1-lock-closure.md",
            ),
        )

        self.assertEqual(["sound-lock-closure"], [node.summary_slug for node in incomplete])
        self.assertEqual([], complete)
        self.assertEqual(["sound-lock-closure"], [node.summary_slug for node in piggybacked])

    def test_import_uses_one_snapshot_for_scope_metadata_and_validation(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/02-editor.md")
        fixing = self.fixture.add_plan("docs/plans/plugins/02-sound.md")
        failure = self.fixture.add_handoff(origin, fixing, "immutable-scope")
        failure.write_text(
            failure.read_text(encoding="utf-8").replace(
                "summary_slug:",
                "plan_link_mode: child_record_only\n"
                "related_code:\n"
                "  - Cargo.lock\n"
                "summary_slug:",
            ),
            encoding="utf-8",
        )
        validator = self.service._validator_module()

        def parse_then_mutate(snapshot_root: Path):
            records, errors = validator.parse_handoff_records(snapshot_root)
            failure.write_text(
                failure.read_text(encoding="utf-8").replace("Cargo.lock", "foreign.rs"),
                encoding="utf-8",
            )
            return records, errors

        self.service._validator = SimpleNamespace(
            parse_handoff_records=parse_then_mutate,
            validate_repository=validator.validate_repository,
        )
        audit = self.service.import_repository()

        self.assertEqual(("Cargo.lock",), audit.nodes[0].related_code)

    def test_pre45_failure_rows_fail_closed_until_an_import_refreshes_scope(self) -> None:
        legacy_database = Database(self.root / "state/pre45.sqlite3")
        with mock.patch.object(migrations_module, "LATEST_SCHEMA_VERSION", 44):
            migrate(legacy_database)
        origin = self.fixture.add_plan("docs/plans/editor/02-editor.md")
        fixing = self.fixture.add_plan("docs/plans/plugins/02-sound.md")
        with legacy_database.transaction() as connection:
            connection.execute(
                """INSERT INTO failure_nodes(
                    lifecycle_key, artifact_path, kind, status, created_at,
                    resolved_at, summary_slug, origin_plan, origin_workflow_node,
                    fixing_plan, origin_child_dir, fixing_child_dir, priority, imported_at
                ) VALUES (?, ?, 'failure', 'open', ?, NULL, ?, ?, 'M1', ?, ?, ?, 0, ?)""",
                (
                    "legacy",
                    "docs/plans/plugins/02/failure-legacy.md",
                    "2026-07-17",
                    "legacy",
                    origin.path.relative_to(self.root).as_posix(),
                    fixing.path.relative_to(self.root).as_posix(),
                    origin.child.relative_to(self.root).as_posix(),
                    fixing.child.relative_to(self.root).as_posix(),
                    "2026-07-17T00:00:00+00:00",
                ),
            )
        migrate(legacy_database)
        service = FailureGraphService(legacy_database, self.root)

        blocking = service.open_for_manifest(
            fixing.path,
            ("M1",),
            ("Cargo.lock", "docs/plans/plugins/02/2026-07-17-m1-lock-closure.md"),
        )

        self.assertEqual(["legacy"], [node.summary_slug for node in blocking])

    def test_artifact_snapshot_ignores_date_named_output_record_with_failure_summary(self) -> None:
        record = (
            self.root
            / "docs/plans/zircon_tooling/session_coordinator/01"
            / "2026-07-15-live-evidence-window-and-failure-chain.md"
        )
        record.parent.mkdir(parents=True, exist_ok=True)
        record.write_text(
            "---\nrecord_kind: implementation_slice\nstatus: implemented\n---\n\n"
            "# 实时证据与 Failure 链\n",
            encoding="utf-8",
        )

        self.assertEqual([], failure_artifact_snapshot(self.root))

    def test_import_rejects_failure_files_changed_after_action_preview(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        handoff = self.fixture.add_handoff(origin, fixing, "previewed")
        expected = failure_artifact_snapshot(self.root)
        handoff.write_text(
            handoff.read_text(encoding="utf-8") + "\nchanged after preview\n",
            encoding="utf-8",
        )

        with self.assertRaises(CoordinatorError) as changed:
            self.service.import_repository(expected_artifacts=expected)

        self.assertEqual("action_state_changed", changed.exception.code)

    def test_controlled_import_parses_the_same_bytes_that_were_hashed(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        handoff = self.fixture.add_handoff(origin, fixing, "approved")
        expected = failure_artifact_snapshot(self.root)
        validator = self.service._validator_module()

        def mutate_then_parse(snapshot_root: Path):
            handoff.write_text(
                handoff.read_text(encoding="utf-8").replace(
                    "summary_slug: approved", "summary_slug: unapproved"
                ),
                encoding="utf-8",
            )
            return validator.parse_handoff_records(snapshot_root)

        self.service._validator = SimpleNamespace(
            parse_handoff_records=mutate_then_parse,
            validate_repository=validator.validate_repository,
        )
        audit = self.service.import_repository(expected_artifacts=expected)

        self.assertEqual(["approved"], [node.summary_slug for node in audit.nodes])

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

    def test_live_dependency_graph_ignores_fixed_handoff_history(self) -> None:
        plan_a = self.fixture.add_plan("docs/plans/a/01-a.md")
        plan_b = self.fixture.add_plan("docs/plans/b/02-b.md")
        self.fixture.add_handoff(plan_a, plan_b, "live-a-to-b")
        self.fixture.add_handoff(plan_b, plan_a, "fixed-b-to-a", kind="fixed")

        audit = self.service.import_repository()

        self.assertEqual(
            [],
            [diagnostic for diagnostic in audit.diagnostics if diagnostic.code == "cycle"],
        )
        self.assertEqual(2, audit.node_count)

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

    def test_return_rewrites_only_source_link_tokens_inside_table_rows(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/07-editor.md")
        fixing = self.fixture.add_plan("docs/plans/tooling/01-tooling.md")
        failure = self.fixture.add_handoff(origin, fixing, "table-row")
        source_link = Path(os.path.relpath(failure, origin.path.parent)).as_posix()
        table_row = (
            "| M3 | preserve evidence | "
            f"[first]({source_link}) | [second]({source_link}) | "
            "[unrelated](../other/failure.md) |"
        )
        with origin.path.open("a", encoding="utf-8") as stream:
            stream.write(f"\n{table_row}\n")
        node = self.service.import_repository().nodes[0]

        fixed = self.service.return_fixed(
            node.lifecycle_key,
            FailureResolution("root", "architecture", "validation", "return"),
            resolved_at=date(2026, 7, 15),
        )

        destination_link = Path(os.path.relpath(fixed, origin.path.parent)).as_posix()
        origin_text = origin.path.read_text(encoding="utf-8")
        self.assertIn(f"- fixed 已修复：[table-row]({destination_link})", origin_text)
        self.assertIn("| M3 | preserve evidence |", origin_text)
        self.assertEqual(2, origin_text.count(f"[fixed 已修复：table-row]({destination_link})"))
        self.assertIn("[unrelated](../other/failure.md)", origin_text)

    def test_child_record_only_return_moves_fixed_artifact_without_writing_parent_plans(self) -> None:
        origin = self.fixture.add_plan("docs/plans/runtime/04-runtime.md")
        fixing = self.fixture.add_plan("docs/plans/tooling/01-tooling.md")
        failure = self.fixture.add_handoff(origin, fixing, "child-record-only")
        failure.write_text(
            failure.read_text(encoding="utf-8").replace(
                "summary_slug:", "plan_link_mode: child_record_only\nsummary_slug:"
            ),
            encoding="utf-8",
        )
        original_origin = origin.path.read_text(encoding="utf-8")
        original_fixing = fixing.path.read_text(encoding="utf-8")
        node = self.service.import_repository().nodes[0]

        fixed = self.service.return_fixed(
            node.lifecycle_key,
            FailureResolution("root", "architecture", "validation", "return"),
            resolved_at=date(2026, 7, 16),
        )

        receipt = fixing.child / "2026-07-16-child-record-only-return.md"
        self.assertEqual(origin.child / "fixed-2026-07-16-child-record-only.md", fixed)
        self.assertFalse(failure.exists())
        self.assertEqual(original_origin, origin.path.read_text(encoding="utf-8"))
        self.assertEqual(original_fixing, fixing.path.read_text(encoding="utf-8"))
        self.assertTrue(receipt.exists())
        self.assertIn("status: fixed", receipt.read_text(encoding="utf-8"))
        self.assertIn("fixed-2026-07-16-child-record-only.md", receipt.read_text(encoding="utf-8"))
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

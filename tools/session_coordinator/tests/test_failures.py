from __future__ import annotations

import hashlib
import os
import tempfile
import unittest
from types import SimpleNamespace
from datetime import date
from pathlib import Path
from unittest import mock

from tools.session_coordinator.database import Database
from tools.session_coordinator import failures as failures_module
from tools.session_coordinator.failures import (
    FailureGraphService,
    FailureResolution,
    failure_artifact_snapshot,
)
from tools.session_coordinator.failure_dependency_graph import (
    failure_graph_diagnostics,
)
from tools.session_coordinator.failure_snapshot_drift import failure_snapshot_drift
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

    def test_import_accepts_explicit_local_failure_scope_without_a_dependency_self_edge(self) -> None:
        plan = self.fixture.add_plan("docs/plans/tooling/01-tooling.md")
        failure = self.fixture.add_handoff(plan, plan, "validation-repair")
        failure.write_text(
            failure.read_text(encoding="utf-8").replace(
                "summary_slug:",
                "failure_scope: local\nplan_link_mode: child_record_only\nsummary_slug:",
            ),
            encoding="utf-8",
        )

        audit = self.service.import_repository()

        self.assertEqual((), audit.diagnostics)
        self.assertEqual(
            ["validation-repair"],
            [node.summary_slug for node in self.service.open_for_plan(plan.path)],
        )

    def test_materialize_local_validation_failure_creates_a_child_only_repair_record(self) -> None:
        plan = self.fixture.add_plan("docs/plans/tooling/01-tooling.md")

        artifact = self.service.materialize_local_validation_failure(
            origin_plan=plan.path,
            summary_slug="validation-repair",
            source_slice="M2 validation ticket 41",
            reproduction="python -m unittest tools.session_coordinator.tests.test_governance",
            lowest_known_cause="The coordinator did not preserve the observed validation failure.",
            acceptance_criteria=(
                "The focused validation passes after the forward repair.",
                "The returned fixed record preserves the original validation ticket.",
            ),
            related_code=("tools/session_coordinator/governance.py",),
            created_at=date(2026, 7, 31),
        )

        audit = self.service.audit()

        self.assertEqual(plan.child / "failure-2026-07-31-validation-repair.md", artifact)
        content = artifact.read_text(encoding="utf-8")
        self.assertIn("failure_scope: local", content)
        self.assertIn("plan_link_mode: child_record_only", content)
        self.assertIn("M2 validation ticket 41", content)
        self.assertEqual((), audit.diagnostics)
        self.assertEqual(
            ["validation-repair"],
            [node.summary_slug for node in self.service.open_for_plan(plan.path)],
        )

    def test_materialize_local_validation_failure_avoids_a_global_graph_rescan(self) -> None:
        plan = self.fixture.add_plan("docs/plans/tooling/01-tooling.md")
        self.service.import_repository = mock.Mock()

        artifact = self.service.materialize_local_validation_failure(
            origin_plan=plan.path,
            summary_slug="fast-validation-repair",
            source_slice="M2 ticket 42",
            reproduction="focused test failed",
            lowest_known_cause="The test result needs an owned forward repair.",
            acceptance_criteria=("The focused test passes after repair.",),
            related_code=("tools/session_coordinator/failures.py",),
            created_at=date(2026, 7, 31),
        )

        self.service.import_repository.assert_not_called()
        self.assertEqual(
            artifact.relative_to(self.root).as_posix(),
            self.service.open_for_plan(plan.path)[0].artifact_path,
        )

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

    def test_import_rejects_scope_drift_after_parsing_one_snapshot(self) -> None:
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
        with self.assertRaises(CoordinatorError) as stale:
            self.service.import_repository()

        self.assertEqual("failure_snapshot_stale", stale.exception.code)
        self.assertEqual(0, self.service.audit().node_count)

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

    def test_prepared_import_rejects_failure_snapshot_drift_inside_transaction(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        handoff = self.fixture.add_handoff(origin, fixing, "prepared")
        prepared = self.service.prepare_import_snapshot()
        handoff.write_text(
            handoff.read_text(encoding="utf-8") + "\nchanged after preparation\n",
            encoding="utf-8",
        )

        with self.assertRaises(CoordinatorError) as stale:
            with self.database.transaction() as connection:
                self.service.import_prepared_snapshot(prepared, connection=connection)

        self.assertEqual("failure_snapshot_stale", stale.exception.code)
        expected_hash = dict(prepared.artifact_manifest)[
            handoff.relative_to(self.root).as_posix()
        ]
        current_hash = hashlib.sha256(handoff.read_bytes()).hexdigest()
        self.assertEqual(
            {
                "expectedArtifactCount": 1,
                "currentArtifactCount": 1,
                "addedCount": 0,
                "removedCount": 0,
                "modifiedCount": 1,
                "changeCount": 1,
                "changes": [
                    {
                        "path": handoff.relative_to(self.root).as_posix(),
                        "kind": "modified",
                        "expectedHash": expected_hash,
                        "currentHash": current_hash,
                    }
                ],
                "truncated": False,
            },
            stale.exception.details,
        )
        self.assertEqual(0, self.service.audit().node_count)

    def test_prepared_import_reports_added_and_removed_failure_artifacts(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        removed = self.fixture.add_handoff(origin, fixing, "removed")
        prepared = self.service.prepare_import_snapshot()
        removed_hash = dict(prepared.artifact_manifest)[
            removed.relative_to(self.root).as_posix()
        ]
        removed.unlink()
        added = self.fixture.add_handoff(origin, fixing, "added")

        with self.assertRaises(CoordinatorError) as stale:
            self.service.import_prepared_snapshot(prepared)

        self.assertEqual("failure_snapshot_stale", stale.exception.code)
        self.assertEqual(
            [
                {
                    "path": added.relative_to(self.root).as_posix(),
                    "kind": "added",
                    "expectedHash": None,
                    "currentHash": hashlib.sha256(added.read_bytes()).hexdigest(),
                },
                {
                    "path": removed.relative_to(self.root).as_posix(),
                    "kind": "removed",
                    "expectedHash": removed_hash,
                    "currentHash": None,
                },
            ],
            stale.exception.details["changes"],
        )
        self.assertEqual(2, stale.exception.details["changeCount"])
        self.assertEqual(1, stale.exception.details["addedCount"])
        self.assertEqual(1, stale.exception.details["removedCount"])
        self.assertEqual(0, stale.exception.details["modifiedCount"])
        self.assertFalse(stale.exception.details["truncated"])

    def test_failure_snapshot_drift_is_deterministic_and_bounded(self) -> None:
        details = failure_snapshot_drift(
            (
                ("docs/plans/z/failure-z.md", "a" * 64),
                ("docs/plans/m/failure-m.md", "b" * 64),
            ),
            (
                ("docs/plans/m/failure-m.md", "c" * 64),
                ("docs/plans/a/failure-a.md", "d" * 64),
            ),
            limit=2,
        )

        self.assertEqual(2, details["expectedArtifactCount"])
        self.assertEqual(2, details["currentArtifactCount"])
        self.assertEqual(1, details["addedCount"])
        self.assertEqual(1, details["removedCount"])
        self.assertEqual(1, details["modifiedCount"])
        self.assertEqual(3, details["changeCount"])
        self.assertEqual(
            [
                {
                    "path": "docs/plans/a/failure-a.md",
                    "kind": "added",
                    "expectedHash": None,
                    "currentHash": "d" * 64,
                },
                {
                    "path": "docs/plans/m/failure-m.md",
                    "kind": "modified",
                    "expectedHash": "b" * 64,
                    "currentHash": "c" * 64,
                },
            ],
            details["changes"],
        )
        self.assertTrue(details["truncated"])

    def test_controlled_import_rejects_live_drift_after_parsing_captured_bytes(self) -> None:
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
        with self.assertRaises(CoordinatorError) as stale:
            self.service.import_repository(expected_artifacts=expected)

        self.assertEqual("failure_snapshot_stale", stale.exception.code)
        self.assertEqual(0, self.service.audit().node_count)

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

    def test_cycle_inventory_groups_one_scc_with_exact_edge_artifacts_durably(self) -> None:
        plan_a = self.fixture.add_plan("docs/plans/a/01-a.md")
        plan_b = self.fixture.add_plan("docs/plans/b/02-b.md")
        plan_c = self.fixture.add_plan("docs/plans/c/03-c.md")
        a_to_b = (
            self.fixture.add_handoff(plan_a, plan_b, "a-to-b-first"),
            self.fixture.add_handoff(plan_a, plan_b, "a-to-b-second"),
        )
        b_to_c = self.fixture.add_handoff(plan_b, plan_c, "b-to-c")
        c_to_a = self.fixture.add_handoff(plan_c, plan_a, "c-to-a")
        artifacts = (*a_to_b, b_to_c, c_to_a)

        imported = self.service.import_repository()
        cycle = next(item for item in imported.diagnostics if item.code == "cycle")
        persisted = next(
            item for item in self.service.audit().diagnostics if item.code == "cycle"
        )

        expected_plans = [
            plan.path.relative_to(self.root).as_posix()
            for plan in (plan_a, plan_b, plan_c)
        ]
        expected_artifacts = sorted(
            path.relative_to(self.root).as_posix() for path in artifacts
        )
        self.assertEqual(
            1, len([item for item in imported.diagnostics if item.code == "cycle"])
        )
        self.assertRegex(cycle.details["componentId"], r"^[0-9a-f]{64}$")
        self.assertEqual(expected_plans, cycle.details["plans"])
        self.assertEqual(
            [
                {
                    "originPlan": expected_plans[0],
                    "fixingPlan": expected_plans[1],
                    "artifacts": sorted(
                        path.relative_to(self.root).as_posix() for path in a_to_b
                    ),
                },
                {
                    "originPlan": expected_plans[1],
                    "fixingPlan": expected_plans[2],
                    "artifacts": [b_to_c.relative_to(self.root).as_posix()],
                },
                {
                    "originPlan": expected_plans[2],
                    "fixingPlan": expected_plans[0],
                    "artifacts": [c_to_a.relative_to(self.root).as_posix()],
                },
            ],
            cycle.details["edges"],
        )
        self.assertEqual(tuple(expected_artifacts), cycle.paths)
        self.assertEqual(cycle, persisted)

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

    def test_excessive_dependency_depth_reuses_a_shared_suffix(self) -> None:
        root_a = self.fixture.add_plan("docs/plans/depth/01-root-a.md")
        root_b = self.fixture.add_plan("docs/plans/depth/02-root-b.md")
        shared = self.fixture.add_plan("docs/plans/depth/03-shared.md")
        tail_a = self.fixture.add_plan("docs/plans/depth/04-tail-a.md")
        tail_b = self.fixture.add_plan("docs/plans/depth/05-tail-b.md")
        self.fixture.add_handoff(root_a, shared, "root-a-to-shared")
        self.fixture.add_handoff(root_b, shared, "root-b-to-shared")
        self.fixture.add_handoff(shared, tail_a, "shared-to-tail-a")
        self.fixture.add_handoff(tail_a, tail_b, "tail-a-to-tail-b")
        service = FailureGraphService(self.database, self.root, max_depth=2)

        diagnostics = service.import_repository().diagnostics
        depth_paths = {
            item.paths[0]
            for item in diagnostics
            if item.code == "excessive_depth"
        }

        self.assertEqual(
            {
                root_a.path.relative_to(self.root).as_posix(),
                root_b.path.relative_to(self.root).as_posix(),
            },
            depth_paths,
        )

    def test_deep_dependency_graph_avoids_python_recursion_limit(self) -> None:
        plan_count = 1_200
        edges = {
            f"docs/plans/depth/{index:04d}.md": {
                f"docs/plans/depth/{index + 1:04d}.md"
            }
            for index in range(plan_count - 1)
        }

        diagnostics = failure_graph_diagnostics(
            edges,
            {},
            max_depth=plan_count - 3,
        )

        self.assertEqual(
            {
                "docs/plans/depth/0000.md",
                "docs/plans/depth/0001.md",
            },
            {
                item.paths[0]
                for item in diagnostics
                if item.code == "excessive_depth"
            },
        )
        self.assertEqual([], [item for item in diagnostics if item.code == "cycle"])

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
        expected_path = failure.relative_to(self.root).as_posix()
        status_errors = [
            item
            for item in audit.diagnostics
            if item.code == "schema_validation" and "status: open" in item.message
        ]
        persisted = [
            item
            for item in self.service.audit().diagnostics
            if item.code == "schema_validation" and "status: open" in item.message
        ]
        self.assertEqual(status_errors, persisted)
        self.assertTrue(all(item.paths == (expected_path,) for item in status_errors))

    def test_schema_diagnostics_persist_the_exact_plan_path(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        self.fixture.add_handoff(origin, fixing, "missing-fixing-link")
        fixing.path.write_text("# fixing plan without handoff link\n", encoding="utf-8")
        expected_path = fixing.path.relative_to(self.root).as_posix()

        imported = [
            item
            for item in self.service.import_repository().diagnostics
            if item.code == "schema_validation"
            and item.message.startswith(f"{expected_path}:")
        ]
        persisted = [
            item
            for item in self.service.audit().diagnostics
            if item.code == "schema_validation"
            and item.message.startswith(f"{expected_path}:")
        ]

        self.assertGreaterEqual(len(imported), 1)
        self.assertEqual(imported, persisted)
        self.assertTrue(all(item.paths == (expected_path,) for item in imported))

    def test_parse_diagnostics_persist_the_exact_artifact_path(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        failure = self.fixture.add_handoff(origin, fixing, "missing-identity")
        failure.write_text(
            "\n".join(
                line
                for line in failure.read_text(encoding="utf-8").splitlines()
                if not line.startswith(("created_at:", "summary_slug:"))
            )
            + "\n",
            encoding="utf-8",
        )
        expected_path = failure.relative_to(self.root).as_posix()

        imported = [
            item
            for item in self.service.import_repository().diagnostics
            if item.code == "parse_error"
        ]
        persisted = [
            item for item in self.service.audit().diagnostics if item.code == "parse_error"
        ]

        self.assertGreaterEqual(len(imported), 2)
        self.assertEqual(imported, persisted)
        self.assertTrue(all(item.paths == (expected_path,) for item in imported))

    def test_parse_diagnostic_path_binding_rejects_a_non_prefix_mention(self) -> None:
        artifact_path = "docs/plans/runtime/02/failure-2026-08-27-example.md"
        manifest = ((artifact_path, "a" * 64),)

        self.assertEqual(
            (),
            failures_module._artifact_paths_for_diagnostic(
                f"validator note mentions {artifact_path}: malformed", manifest
            ),
        )

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
        with self.database.connect() as connection:
            lifecycle_events = connection.execute(
                """
                SELECT event_kind, artifact_path
                FROM failure_lifecycle_events
                WHERE lifecycle_key=?
                ORDER BY event_id
                """,
                (node.lifecycle_key,),
            ).fetchall()
        self.assertEqual(["added", "fixed"], [row["event_kind"] for row in lifecycle_events])
        self.assertEqual(
            [
                failure.relative_to(self.root).as_posix(),
                fixed.relative_to(self.root).as_posix(),
            ],
            [row["artifact_path"] for row in lifecycle_events],
        )

        self.service.import_repository()
        with self.database.connect() as connection:
            replayed_events = connection.execute(
                """
                SELECT event_kind, artifact_path
                FROM failure_lifecycle_events
                WHERE lifecycle_key=?
                ORDER BY event_id
                """,
                (node.lifecycle_key,),
            ).fetchall()
        self.assertEqual(
            [(row["event_kind"], row["artifact_path"]) for row in lifecycle_events],
            [(row["event_kind"], row["artifact_path"]) for row in replayed_events],
        )

    def test_return_rejects_source_schema_errors_without_moving_failure(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        failure = self.fixture.add_handoff(origin, fixing, "malformed-source")
        source_text = failure.read_text(encoding="utf-8")
        source_text = source_text.replace(
            "## 来源执行者", "## Source executor"
        ).replace(
            "## 失败现象与复现证据", "## Failure and reproduction"
        ).replace(
            "## 最低共享层根因", "## Lowest shared cause"
        ).replace(
            "## 架构修复验收", "## Architecture acceptance"
        ).replace(
            "## 禁止临时方案", "## Forbidden shortcuts"
        )
        failure.write_text(source_text, encoding="utf-8")
        node = next(
            node
            for node in self.service.import_repository().nodes
            if node.summary_slug == "malformed-source"
        )

        with self.assertRaises(CoordinatorError) as raised:
            self.service.return_fixed(
                node.lifecycle_key,
                FailureResolution("root", "architecture", "validation", "return"),
                resolved_at=date(2026, 7, 17),
            )

        self.assertEqual("invalid_handoff", raised.exception.code)
        self.assertIn("missing required heading", raised.exception.message)
        self.assertTrue(failure.exists())
        self.assertFalse(
            (origin.child / "fixed-2026-07-17-malformed-source.md").exists()
        )

    def test_source_schema_validation_uses_an_isolated_plan_snapshot(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        failure = self.fixture.add_handoff(origin, fixing, "isolated-source-check")
        validator = self.service._validator_module()
        observed_roots: list[Path] = []
        original_validate = validator.validate_repository

        def capture_validation(root: Path) -> list[str]:
            isolated_root = Path(root)
            observed_roots.append(isolated_root)
            self.assertNotEqual(self.root.resolve(), isolated_root.resolve())
            self.assertTrue(
                (isolated_root / failure.relative_to(self.root)).is_file()
            )
            return original_validate(isolated_root)

        with mock.patch.object(
            validator, "validate_repository", side_effect=capture_validation
        ):
            self.assertEqual(
                (),
                self.service._source_validation_errors(
                    failure,
                    origin_plan=origin.path,
                    fixing_plan=fixing.path,
                ),
            )

        self.assertEqual(1, len(observed_roots))

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

    def test_return_preserves_unrelated_links_in_the_same_handoff_bullet(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/08-editor.md")
        fixing = self.fixture.add_plan("docs/plans/tooling/01-tooling.md")
        peer_fixer = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        failure = self.fixture.add_handoff(origin, fixing, "provider")
        peer = self.fixture.add_handoff(origin, peer_fixer, "peer")
        source_link = Path(os.path.relpath(failure, origin.path.parent)).as_posix()
        peer_link = Path(os.path.relpath(peer, origin.path.parent)).as_posix()
        origin_text = origin.path.read_text(encoding="utf-8")
        origin_text = origin_text.replace(
            f"- open 待修复：[provider]({source_link})",
            f"- open lifecycle：[provider]({source_link}) · [peer]({peer_link})",
        ).replace(f"\n- open 待修复：[peer]({peer_link})\n", "\n")
        origin.path.write_text(origin_text, encoding="utf-8")
        node = next(
            node
            for node in self.service.import_repository().nodes
            if node.summary_slug == "provider"
        )

        fixed = self.service.return_fixed(
            node.lifecycle_key,
            FailureResolution("root", "architecture", "validation", "return"),
            resolved_at=date(2026, 7, 15),
        )

        destination_link = Path(os.path.relpath(fixed, origin.path.parent)).as_posix()
        rewritten = origin.path.read_text(encoding="utf-8")
        self.assertIn(f"[fixed 已修复：provider]({destination_link})", rewritten)
        self.assertIn(f"[peer]({peer_link})", rewritten)
        self.assertEqual([], self.service.validator_errors())

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

    def test_child_record_only_return_preserves_required_sections_after_result(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/tooling/01-tooling.md")
        failure = self.fixture.add_handoff(origin, fixing, "section-preservation")
        content = failure.read_text(encoding="utf-8")
        content = content.replace(
            "summary_slug:",
            "plan_link_mode: child_record_only\nsummary_slug:",
        ).replace(
            "## 禁止临时方案\n\n- No fallback, alias, shim, or test bypass.\n\n"
            "## 修复结果与回传\n\n待修复",
            "## 修复结果与回传\n\n待修复\n\n"
            "## 禁止临时方案\n\n- No fallback, alias, shim, or test bypass.",
        )
        content = content.replace(
            "## 修复结果与回传\n\n待修复",
            "```markdown\n## 修复结果与回传\n```\n\n"
            "- ```markdown\n  ## 修复结果与回传\n  ```\n\n"
            "> ```markdown\n> ## 修复结果与回传\n> ```\n\n"
            "    ## 修复结果与回传\n\n"
            "## 修复结果与回传\n\n待修复",
        ).replace(
            "- No fallback, alias, shim, or test bypass.",
            "- No fallback, alias, shim, or test bypass.  \n"
            "  Preserve this Markdown hard break.\n\n"
            "## 后续证据\n\n- Preserve arbitrary trailing sections.",
        )
        self.assertLess(
            content.rindex("## 修复结果与回传"),
            content.index("## 禁止临时方案"),
        )
        expected_suffix = content[content.index("## 禁止临时方案") :]
        failure.write_text(content, encoding="utf-8")
        node = self.service.import_repository().nodes[0]

        fixed = self.service.return_fixed(
            node.lifecycle_key,
            FailureResolution("root", "architecture", "validation", "return"),
            resolved_at=date(2026, 7, 16),
        )

        fixed_content = fixed.read_text(encoding="utf-8")
        self.assertEqual(
            expected_suffix,
            fixed_content[fixed_content.index("## 禁止临时方案") :],
        )
        self.assertEqual([], self.service.validator_errors())

    def test_return_rejects_duplicate_real_result_sections(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/tooling/01-tooling.md")
        failure = self.fixture.add_handoff(origin, fixing, "duplicate-result-section")
        content = failure.read_text(encoding="utf-8").replace(
            "## 修复结果与回传\n\n待修复",
            "## 修复结果与回传\n\n待修复\n\n"
            "## 修复结果与回传\n\nstale result",
        )
        failure.write_text(content, encoding="utf-8")
        node = self.service.import_repository().nodes[0]

        with self.assertRaises(CoordinatorError) as raised:
            self.service.return_fixed(
                node.lifecycle_key,
                FailureResolution("root", "architecture", "validation", "return"),
                resolved_at=date(2026, 7, 16),
            )

        self.assertEqual("invalid_handoff", raised.exception.code)
        self.assertIn("duplicate", raised.exception.message)
        self.assertTrue(failure.exists())

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

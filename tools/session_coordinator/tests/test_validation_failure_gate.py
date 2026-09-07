from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace

from tools.session_coordinator.database import Database
from tools.session_coordinator.failures import FailureGraphService
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.tests.failure_fixture import FailureGraphFixture
from tools.session_coordinator.validation_failure_gate import blockers, record_blockers


class ValidationFailureGateTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.database = Database(self.root / "coordinator.sqlite3")
        migrate(self.database)
        self.fixture = FailureGraphFixture(self.root)

    def _ticket(
        self,
        plan_path: str,
        *,
        paths: tuple[str, ...] = (),
        workflow_nodes: tuple[str, ...] = (),
        command: tuple[str, ...] = (),
        toolchain: dict[str, object] | None = None,
        full_coverage: bool = False,
    ) -> SimpleNamespace:
        coverage: dict[str, object] = {"kind": "focused"}
        if workflow_nodes:
            coverage["workflowNodeKeys"] = list(workflow_nodes)
        if full_coverage:
            coverage["fullCoverage"] = True
        return SimpleNamespace(
            plan_path=plan_path,
            source_manifest={path: "0" * 64 for path in paths},
            coverage=coverage,
            command=command,
            toolchain=toolchain or {},
        )

    def _import(self) -> None:
        FailureGraphService(self.database, self.root).import_repository()

    def _insert_ticket(self, ticket_id: str = "ticket-a") -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO sessions(
                    session_id, status, base_head, write_scope_json,
                    created_at, updated_at, last_heartbeat_at
                ) VALUES ('session-a', 'active', '', '[]', '2026-09-05',
                          '2026-09-05', '2026-09-05')"""
            )
            connection.execute(
                """INSERT INTO validation_tickets(
                    ticket_id, session_id, plan_path, status, dedupe_key,
                    source_manifest_hash, source_manifest_json, command_json,
                    toolchain_json, coverage_json, created_at, updated_at
                ) VALUES (?, 'session-a', 'docs/plans/editor/01-editor.md',
                          'queued', 'dedupe', 'manifest', '{}', '[]', '{}', '{}',
                          '2026-09-05', '2026-09-05')""",
                (ticket_id,),
            )

    def test_affected_origin_scope_returns_structured_blocker(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        artifact = self.fixture.add_handoff(origin, fixing, "shared-identity")
        artifact.write_text(
            artifact.read_text(encoding="utf-8").replace(
                "summary_slug:",
                "origin_workflow_node: M2.1\n"
                "plan_link_mode: child_record_only\n"
                "related_code:\n"
                "  - zircon_editor/src/identity.rs\n"
                "summary_slug:",
            ),
            encoding="utf-8",
        )
        self._import()

        ticket = self._ticket(
            origin.path.relative_to(self.root).as_posix(),
            paths=("zircon_editor/src/identity.rs",),
            workflow_nodes=("M2.1",),
        )
        with self.database.connect() as connection:
            result = blockers(connection, ticket)

        self.assertEqual(1, len(result))
        expected_lifecycle = "|".join(
            (
                origin.path.resolve().as_posix().casefold(),
                fixing.path.resolve().as_posix().casefold(),
                "shared-identity",
            )
        )
        self.assertEqual(
            {
                "code": "validation_dependency_failed",
                "message": "Validation depends on open Failure shared-identity",
                "repairCondition": (
                    "Fixing plan docs/plans/runtime/02-runtime.md must resolve and "
                    "import the fixed lifecycle for "
                    "docs/plans/runtime/02/failure-2026-07-11-shared-identity.md"
                ),
                "artifactPath": artifact.relative_to(self.root).as_posix(),
                "lifecycleKey": expected_lifecycle,
                "summarySlug": "shared-identity",
                "originPlan": origin.path.relative_to(self.root).as_posix(),
                "originWorkflowNode": "M2.1",
                "fixingPlan": fixing.path.relative_to(self.root).as_posix(),
                "relatedPaths": ["zircon_editor/src/identity.rs"],
                "dependencyPath": [
                    origin.path.relative_to(self.root).as_posix(),
                    fixing.path.relative_to(self.root).as_posix(),
                ],
                "priority": 0,
            },
            result[0],
        )

    def test_degraded_baseline_does_not_block_unrelated_plan_or_path(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        unrelated = self.fixture.add_plan("docs/plans/plugins/03-sound.md")
        artifact = self.fixture.add_handoff(origin, fixing, "shared-identity")
        artifact.write_text(
            artifact.read_text(encoding="utf-8").replace(
                "summary_slug:",
                "plan_link_mode: child_record_only\n"
                "related_code:\n  - zircon_editor/src/identity.rs\nsummary_slug:",
            ),
            encoding="utf-8",
        )
        self._import()
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO baseline_epochs(
                    head_commit, index_tree, health, manifest_json, created_at,
                    degraded_at, degraded_reason
                ) VALUES ('head', 'tree', 'degraded', '{}', '2026-09-05',
                          '2026-09-05', 'unrelated global failure')"""
            )

        tickets = (
            self._ticket(
                unrelated.path.relative_to(self.root).as_posix(),
                paths=("plugins/sound/src/lib.rs",),
            ),
            self._ticket(
                origin.path.relative_to(self.root).as_posix(),
                paths=("zircon_editor/src/unrelated.rs",),
            ),
        )
        with self.database.connect() as connection:
            self.assertEqual(((), ()), tuple(blockers(connection, item) for item in tickets))

    def test_plan_identity_ignores_case_and_separator_spelling(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        self.fixture.add_handoff(origin, fixing, "mixed-case-plan")
        self._import()
        origin_path = origin.path.relative_to(self.root).as_posix()
        ticket = self._ticket(origin_path.upper().replace("/", "\\"))

        with self.database.connect() as connection:
            result = blockers(connection, ticket)

        self.assertEqual(["mixed-case-plan"], [item["summarySlug"] for item in result])
        self.assertEqual(origin_path, result[0]["originPlan"])
        self.assertEqual(origin_path.upper(), result[0]["dependencyPath"][0])

    def test_cargo_upper_package_is_blocked_when_lower_failure_is_outside_overlay(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        artifact = self.fixture.add_handoff(origin, fixing, "lower-package")
        artifact.write_text(
            artifact.read_text(encoding="utf-8").replace(
                "summary_slug:",
                "plan_link_mode: child_record_only\n"
                "related_code:\n  - zircon_runtime/src/lower.rs\nsummary_slug:",
            ),
            encoding="utf-8",
        )
        self._import()
        ticket = self._ticket(
            origin.path.relative_to(self.root).as_posix(),
            paths=("zircon_editor/src/upper.rs",),
            command=("cargo", "test", "-p", "zircon_editor", "--locked"),
        )

        with self.database.connect() as connection:
            result = blockers(connection, ticket)

        self.assertEqual(["lower-package"], [item["summarySlug"] for item in result])

    def test_legacy_cargo_wrapper_uses_worker_lane_contract(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        artifact = self.fixture.add_handoff(origin, fixing, "wrapped-lower-package")
        artifact.write_text(
            artifact.read_text(encoding="utf-8").replace(
                "summary_slug:",
                "plan_link_mode: child_record_only\n"
                "related_code:\n  - zircon_runtime/src/lower.rs\nsummary_slug:",
            ),
            encoding="utf-8",
        )
        self._import()
        cases = (
            ({"cargo_jobs": 1, "rust": "1.94.1"}, True),
            ({"cargo": "not-required", "cargo_jobs": 1, "rust": "1.94.1"}, True),
            ({"cargo_jobs": 0, "rust": "1.94.1"}, False),
            ({"cargo_jobs": -1, "rust": "1.94.1"}, False),
            ({"cargo_jobs": True, "rust": "1.94.1"}, False),
            ({"cargo_jobs": "1", "rust": "1.94.1"}, False),
            ({"cargo_jobs": 1, "rust": "not-required"}, False),
            ({"cargo_jobs": 1, "rust": 1}, False),
        )
        with self.database.connect() as connection:
            for toolchain, expected in cases:
                with self.subTest(toolchain=toolchain):
                    ticket = self._ticket(
                        origin.path.relative_to(self.root).as_posix(),
                        paths=("zircon_editor/src/upper.rs",),
                        command=("pwsh.exe", "-NoProfile", "-File", "validation.ps1"),
                        toolchain=toolchain,
                    )
                    result = blockers(connection, ticket)
                    self.assertEqual(
                        ["wrapped-lower-package"] if expected else [],
                        [item["summarySlug"] for item in result],
                    )

    def test_different_workflow_scope_is_unrelated_even_when_paths_overlap(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        artifact = self.fixture.add_handoff(origin, fixing, "other-workflow")
        artifact.write_text(
            artifact.read_text(encoding="utf-8").replace(
                "summary_slug:",
                "origin_workflow_node: M2.2\n"
                "related_code:\n  - zircon_editor/src/identity.rs\nsummary_slug:",
            ),
            encoding="utf-8",
        )
        self._import()
        ticket = self._ticket(
            origin.path.relative_to(self.root).as_posix(),
            paths=("zircon_editor/src/identity.rs",),
            workflow_nodes=("M2.1",),
            command=("cargo", "test", "-p", "zircon_editor", "--locked"),
            full_coverage=True,
        )

        with self.database.connect() as connection:
            self.assertEqual((), blockers(connection, ticket))

    def test_fixing_plan_repair_is_allowed_while_failure_is_open(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        self.fixture.add_handoff(origin, fixing, "shared-identity")
        self._import()

        repair = self._ticket(
            fixing.path.relative_to(self.root).as_posix(),
            paths=("zircon_runtime/src/identity.rs",),
        )
        with self.database.connect() as connection:
            self.assertEqual((), blockers(connection, repair))

    def test_fixed_lifecycle_no_longer_blocks(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        self.fixture.add_handoff(origin, fixing, "shared-identity", kind="fixed")
        self._import()

        with self.database.connect() as connection:
            result = blockers(
                connection,
                self._ticket(origin.path.relative_to(self.root).as_posix()),
            )

        self.assertEqual((), result)

    def test_reopened_database_uses_durable_failure_graph(self) -> None:
        origin = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        fixing = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        self.fixture.add_handoff(origin, fixing, "durable")
        self._import()

        restarted_database = Database(self.database.path)
        with restarted_database.transaction() as connection:
            result = blockers(
                connection,
                self._ticket(origin.path.relative_to(self.root).as_posix()),
            )

        self.assertEqual(["durable"], [item["summarySlug"] for item in result])

    def test_cross_plan_dependency_closure_reports_transitive_blocker(self) -> None:
        upstream = self.fixture.add_plan("docs/plans/editor/01-editor.md")
        middle = self.fixture.add_plan("docs/plans/runtime/02-runtime.md")
        root = self.fixture.add_plan("docs/plans/framework/03-framework.md")
        self.fixture.add_handoff(upstream, middle, "editor-needs-runtime")
        self.fixture.add_handoff(middle, root, "runtime-needs-framework")
        self._import()

        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE failure_nodes SET origin_plan=? WHERE summary_slug=?",
                (
                    middle.path.relative_to(self.root).as_posix().upper().replace("/", "\\"),
                    "runtime-needs-framework",
                ),
            )

        with self.database.connect() as connection:
            result = blockers(
                connection,
                self._ticket(upstream.path.relative_to(self.root).as_posix()),
            )

        self.assertEqual(
            ["editor-needs-runtime", "runtime-needs-framework"],
            [item["summarySlug"] for item in result],
        )
        self.assertEqual(
            [
                upstream.path.relative_to(self.root).as_posix(),
                middle.path.relative_to(self.root).as_posix(),
                root.path.relative_to(self.root).as_posix(),
            ],
            result[1]["dependencyPath"],
        )

    def test_blocker_event_is_emitted_only_when_blockers_change(self) -> None:
        self._insert_ticket()
        first = ({"artifactPath": "docs/plans/runtime/02/failure-a.md"},)
        second = ({"artifactPath": "docs/plans/runtime/02/failure-b.md"},)

        with self.database.transaction() as connection:
            self.assertFalse(
                record_blockers(connection, "ticket-a", (), "2026-09-05T01:00:00Z")
            )
            self.assertTrue(
                record_blockers(
                    connection, "ticket-a", first, "2026-09-05T01:01:00Z"
                )
            )
            self.assertFalse(
                record_blockers(
                    connection, "ticket-a", first, "2026-09-05T01:02:00Z"
                )
            )
            self.assertTrue(
                record_blockers(
                    connection, "ticket-a", second, "2026-09-05T01:03:00Z"
                )
            )
            self.assertTrue(record_blockers(connection, "ticket-a", (), "2026-09-05T01:04:00Z"))
            self.assertFalse(record_blockers(connection, "ticket-a", (), "2026-09-05T01:05:00Z"))
            rows = connection.execute(
                """SELECT payload_json, created_at FROM validation_ticket_events
                   WHERE ticket_id='ticket-a'
                     AND event_type='validation.ticket_dependency_blocked'
                   ORDER BY event_id"""
            ).fetchall()

        self.assertEqual(
            [
                (
                    '{"blockers":[{"artifactPath":"docs/plans/runtime/02/failure-a.md"}]}',
                    "2026-09-05T01:01:00Z",
                ),
                (
                    '{"blockers":[{"artifactPath":"docs/plans/runtime/02/failure-b.md"}]}',
                    "2026-09-05T01:03:00Z",
                ),
                ('{"blockers":[]}', "2026-09-05T01:04:00Z"),
            ],
            [(str(row["payload_json"]), str(row["created_at"])) for row in rows],
        )


if __name__ == "__main__":
    unittest.main()

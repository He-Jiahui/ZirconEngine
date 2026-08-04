from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import LATEST_SCHEMA_VERSION, MIGRATIONS, migrate
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.workflows.plan_import import TopologyImporter
from tools.session_coordinator.workflows.store import WorkflowStore
from tools.session_coordinator.workflows.topology import TopologyParser


class WorkflowTopologyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.repo = init_repo(root / "repo")
        self.plan_dir = self.repo / "docs/plans/runtime"
        self.plan_dir.mkdir(parents=True)
        self.plan = self.plan_dir / "01-control.md"

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def _write(self, body: str) -> None:
        self.plan.write_text(body, encoding="utf-8")

    def test_schema_16_upgrades_through_current_schema(self) -> None:
        database = Database(Path(self.temporary.name) / "state.sqlite3")
        with database.transaction() as connection:
            connection.execute(
                "CREATE TABLE schema_version(version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)"
            )
            for version in range(1, 17):
                MIGRATIONS[version](connection)
                connection.execute(
                    "INSERT INTO schema_version VALUES (?, 'now')", (version,)
                )

        self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))
        with database.connect() as connection:
            tables = {
                row[0]
                for row in connection.execute(
                    "SELECT name FROM sqlite_master WHERE type='table'"
                )
            }
        self.assertTrue(
            {
                "workflow_topology_versions",
                "workflow_gate_evidence",
                "workflow_review_evidence",
                "notification_attempts",
                "workflow_milestone_manifests",
            }
            <= tables
        )

    def test_frozen_schema_18_validation_binding_upgrades_in_schema_19(self) -> None:
        database = Database(Path(self.temporary.name) / "frozen-v18.sqlite3")
        with database.transaction() as connection:
            connection.execute(
                "CREATE TABLE schema_version(version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)"
            )
            for version in range(1, 19):
                MIGRATIONS[version](connection)
                connection.execute("INSERT INTO schema_version VALUES (?, 'now')", (version,))
            before = {
                row[1] for row in connection.execute(
                    "PRAGMA table_info(workflow_validation_bindings)"
                )
            }
        self.assertNotIn("source_manifest_hash", before)

        self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))
        with database.connect() as connection:
            after = {
                row[1] for row in connection.execute(
                    "PRAGMA table_info(workflow_validation_bindings)"
                )
            }
        self.assertTrue(
            {"source_manifest_hash", "paths_json", "terminal_status", "terminal_code"}
            <= after
        )

    def test_parses_exactly_one_fenced_topology_and_rejects_cycles(self) -> None:
        payload = {
            "schema": 1,
            "workflow_id": "runtime-control",
            "goal": "Runtime control",
            "milestones": [
                {"id": "M1", "title": "base", "depends_on": ["M2"]},
                {"id": "M2", "title": "upper", "depends_on": ["M1"]},
            ],
        }
        self._write(f"```zircon-workflow\n{json.dumps(payload)}\n```\n")

        with self.assertRaises(CoordinatorError) as rejected:
            TopologyParser(self.repo).parse("docs/plans/runtime/01-control.md")
        self.assertEqual("workflow_topology_cycle", rejected.exception.code)

        self._write(
            f"```zircon-workflow\n{json.dumps(payload)}\n```\n"
            f"```zircon-workflow\n{json.dumps(payload)}\n```\n"
        )
        with self.assertRaises(CoordinatorError) as duplicate:
            TopologyParser(self.repo).parse("docs/plans/runtime/01-control.md")
        self.assertEqual("workflow_topology_fence_count", duplicate.exception.code)

    def test_validates_duplicate_and_missing_dependency_ids(self) -> None:
        for milestones, code in (
            (
                [
                    {"id": "M1", "title": "one", "depends_on": []},
                    {"id": "M1", "title": "again", "depends_on": []},
                ],
                "workflow_topology_duplicate_id",
            ),
            (
                [{"id": "M1", "title": "one", "depends_on": ["M9"]}],
                "workflow_topology_missing_dependency",
            ),
        ):
            with self.subTest(code=code):
                payload = {
                    "schema": 1,
                    "workflow_id": "runtime-control",
                    "goal": "Runtime control",
                    "milestones": milestones,
                }
                self._write(f"```zircon-workflow\n{json.dumps(payload)}\n```\n")
                with self.assertRaises(CoordinatorError) as rejected:
                    TopologyParser(self.repo).parse("docs/plans/runtime/01-control.md")
                self.assertEqual(code, rejected.exception.code)

    def test_rejects_malformed_fence_and_child_output_as_plan_owner(self) -> None:
        self._write("```zircon-workflow\n{}\n")
        with self.assertRaises(CoordinatorError) as malformed:
            TopologyParser(self.repo).parse("docs/plans/runtime/01-control.md")
        self.assertEqual("workflow_topology_malformed_fence", malformed.exception.code)

        child = self.plan_dir / "01/2026-07-12-record.md"
        child.parent.mkdir(parents=True)
        child.write_text("## Milestone M1: not a plan definition\n", encoding="utf-8")
        with self.assertRaises(CoordinatorError) as protected:
            TopologyParser(self.repo).parse("docs/plans/runtime/01/2026-07-12-record.md")
        self.assertEqual("not_numbered_plan", protected.exception.code)

    def test_fallback_imports_milestones_and_checkbox_slices(self) -> None:
        self._write(
            "# Runtime Plan\n\n"
            "## Milestone M1: Base\n\n"
            "- [ ] **M1.1 Add storage.** details\n"
            "- [x] **M1.2 Add checks.** details\n\n"
            "## Milestone M2: Upper\n\n"
            "**Dependencies:** M1 accepted.\n"
        )

        topology = TopologyParser(self.repo).parse(
            "docs/plans/runtime/01-control.md"
        )

        self.assertEqual(["M1", "M2"], [item.node_id for item in topology.milestones])
        self.assertEqual(["M1.1", "M1.2"], [item.node_id for item in topology.slices])
        self.assertEqual(("M1",), topology.milestones[1].depends_on)

    def test_fallback_imports_legacy_numbered_plan_headings_without_rewriting_plan(self) -> None:
        self._write(
            "# Shader module imports\n\n"
            "### SH03-M1 Unified registry\n\n"
            "Existing implementation record.\n\n"
            "### SH03-M2 Redirect contract closure\n\n"
            "Current closeout evidence.\n"
        )

        topology = TopologyParser(self.repo).parse(
            "docs/plans/runtime/01-control.md"
        )

        self.assertEqual("headings", topology.source)
        self.assertEqual(
            [("M1", "Unified registry"), ("M2", "Redirect contract closure")],
            [(node.node_id, node.title) for node in topology.milestones],
        )

    def test_fallback_imports_plain_numbered_milestone_headings(self) -> None:
        self._write(
            "# Sound plan\n\n"
            "### M1 Kira integration\n\n"
            "Initial dependency closure.\n\n"
            "### M2 Effects mapping\n\n"
            "**Dependencies:** M1 accepted.\n"
        )

        topology = TopologyParser(self.repo).parse(
            "docs/plans/runtime/01-control.md"
        )

        self.assertEqual("headings", topology.source)
        self.assertEqual(
            [("M1", "Kira integration"), ("M2", "Effects mapping")],
            [(node.node_id, node.title) for node in topology.milestones],
        )
        self.assertEqual(("M1",), topology.milestones[1].depends_on)

    def test_changed_plan_hash_creates_version_without_rewriting_running_graph(self) -> None:
        self._write(
            "```zircon-workflow\n"
            '{"schema":1,"workflow_id":"runtime-control","goal":"Runtime",'
            '"milestones":[{"id":"M1","title":"Base","depends_on":[]}]}\n'
            "```\n"
        )
        database = Database(Path(self.temporary.name) / "state.sqlite3")
        migrate(database)
        sessions = SessionService(database, self.repo)
        sessions.register(
            session_id="session-a", plan_path="docs/plans/runtime/01-control.md"
        )
        importer = TopologyImporter(database, self.repo)

        first = importer.import_plan("session-a", "docs/plans/runtime/01-control.md")
        self._write(self.plan.read_text(encoding="utf-8").replace("Base", "Foundation"))
        second = importer.import_plan("session-a", "docs/plans/runtime/01-control.md")

        self.assertEqual(1, first.version_number)
        self.assertTrue(first.activated)
        self.assertEqual(2, second.version_number)
        self.assertFalse(second.activated)
        with database.connect() as connection:
            run = connection.execute("SELECT * FROM workflow_runs").fetchone()
            titles = [
                row[0]
                for row in connection.execute(
                    "SELECT title FROM workflow_nodes WHERE run_id=? AND kind='milestone'",
                    (run["run_id"],),
                )
            ]
        with database.connect() as connection:
            active_version = connection.execute(
                "SELECT * FROM workflow_topology_versions WHERE topology_version_id=?",
                (run["current_topology_version_id"],),
            ).fetchone()
        self.assertEqual(first.content_hash, active_version["content_hash"])
        self.assertNotEqual(second.content_hash, active_version["content_hash"])
        self.assertEqual(["Base"], titles)

    def test_content_only_change_updates_metadata_without_splitting_topology_identity(self) -> None:
        body = (
            "```zircon-workflow\n"
            '{"schema":1,"workflow_id":"runtime-control","goal":"Runtime",'
            '"milestones":[{"id":"M1","title":"Base","depends_on":[]}]}\n'
            "```\n\nImplementation notes.\n"
        )
        self._write(body)
        database = Database(Path(self.temporary.name) / "state.sqlite3")
        migrate(database)
        SessionService(database, self.repo).register(
            session_id="session-a", plan_path="docs/plans/runtime/01-control.md"
        )
        importer = TopologyImporter(database, self.repo)

        first = importer.import_plan("session-a", "docs/plans/runtime/01-control.md")
        self._write(body.replace("Implementation notes.", "Revised implementation notes."))
        second = importer.import_plan("session-a", "docs/plans/runtime/01-control.md")

        self.assertEqual(first.topology_version_id, second.topology_version_id)
        self.assertEqual(1, second.version_number)
        self.assertNotEqual(first.content_hash, second.content_hash)
        with database.connect() as connection:
            count = connection.execute(
                "SELECT COUNT(*) FROM workflow_topology_versions"
            ).fetchone()[0]
        self.assertEqual(1, count)

    def test_controlled_refresh_activates_content_only_candidate(self) -> None:
        body = (
            "```zircon-workflow\n"
            '{"schema":1,"workflow_id":"runtime-control","goal":"Runtime",'
            '"milestones":[{"id":"M1","title":"Base","depends_on":[]}]}\n'
            "```\n\nNotes A.\n"
        )
        self._write(body)
        database = Database(Path(self.temporary.name) / "state.sqlite3")
        migrate(database)
        SessionService(database, self.repo).register(
            session_id="session-a", plan_path="docs/plans/runtime/01-control.md"
        )
        importer = TopologyImporter(database, self.repo)
        first = importer.import_plan("session-a", "docs/plans/runtime/01-control.md")
        self._write(body.replace("Notes A.", "Notes B."))

        second = importer.import_plan(
            "session-a",
            "docs/plans/runtime/01-control.md",
            activate_candidate=True,
        )

        self.assertTrue(second.activated)
        self.assertEqual(first.topology_version_id, second.topology_version_id)
        self.assertEqual(1, second.version_number)
        self.assertNotEqual(first.content_hash, second.content_hash)
        with database.connect() as connection:
            active = connection.execute(
                "SELECT current_topology_version_id FROM workflow_runs WHERE run_id=?",
                (first.run_id,),
            ).fetchone()[0]
            version_count = connection.execute(
                "SELECT COUNT(*) FROM workflow_topology_versions WHERE run_id=?",
                (first.run_id,),
            ).fetchone()[0]
        self.assertEqual(second.topology_version_id, active)
        self.assertEqual(1, version_count)

    def test_structural_candidate_refuses_to_rewrite_progressed_graph(self) -> None:
        self._write(
            "```zircon-workflow\n"
            '{"schema":1,"workflow_id":"runtime-control","goal":"Runtime",'
            '"milestones":[{"id":"M1","title":"Base","depends_on":[]}]}\n'
            "```\n"
        )
        database = Database(Path(self.temporary.name) / "state.sqlite3")
        migrate(database)
        SessionService(database, self.repo).register(
            session_id="session-a", plan_path="docs/plans/runtime/01-control.md"
        )
        importer = TopologyImporter(database, self.repo)
        first = importer.import_plan("session-a", "docs/plans/runtime/01-control.md")
        store = WorkflowStore(database)
        milestone = next(node for node in store.nodes(first.run_id) if node.node_key == "M1")
        from tools.session_coordinator.models import WorkflowNodeState
        store.append_attempt(milestone.node_id, WorkflowNodeState.RUNNING, {})
        self._write(
            "```zircon-workflow\n"
            '{"schema":1,"workflow_id":"runtime-control","goal":"Runtime",'
            '"milestones":['
            '{"id":"M1","title":"Base","depends_on":[]},'
            '{"id":"M2","title":"Upper","depends_on":["M1"]}]}\n'
            "```\n"
        )

        with self.assertRaises(CoordinatorError) as rejected:
            importer.import_plan(
                "session-a",
                "docs/plans/runtime/01-control.md",
                activate_candidate=True,
            )
        self.assertEqual(
            "workflow_topology_activation_requires_pristine_run", rejected.exception.code
        )

    def test_append_only_candidate_preserves_accepted_and_pending_milestones(self) -> None:
        self._write(
            "```zircon-workflow\n"
            '{"schema":1,"workflow_id":"runtime-control","goal":"Runtime",'
            '"milestones":['
            '{"id":"M1","title":"Foundation","depends_on":[]},'
            '{"id":"M2","title":"Bindings","depends_on":["M1"]},'
            '{"id":"M3","title":"Artifacts","depends_on":[]},'
            '{"id":"M4","title":"Product","depends_on":["M3"]}]}\n'
            "```\n"
        )
        database = Database(Path(self.temporary.name) / "state.sqlite3")
        migrate(database)
        SessionService(database, self.repo).register(
            session_id="session-a", plan_path="docs/plans/runtime/01-control.md"
        )
        importer = TopologyImporter(database, self.repo)
        first = importer.import_plan("session-a", "docs/plans/runtime/01-control.md")
        store = WorkflowStore(database)
        nodes = {node.node_key: node for node in store.nodes(first.run_id)}
        from tools.session_coordinator.models import WorkflowNodeState

        store.append_attempt(nodes["M1"].node_id, WorkflowNodeState.SUCCEEDED, {})
        store.append_attempt(nodes["M2"].node_id, WorkflowNodeState.SUCCEEDED, {})
        self._write(
            "```zircon-workflow\n"
            '{"schema":1,"workflow_id":"runtime-control","goal":"Runtime",'
            '"milestones":['
            '{"id":"M1","title":"Foundation","depends_on":[]},'
            '{"id":"M2","title":"Bindings","depends_on":["M1"]},'
            '{"id":"M3","title":"Artifacts","depends_on":[]},'
            '{"id":"M4","title":"Product","depends_on":["M3"]},'
            '{"id":"M5","title":"Viewer","depends_on":["M1","M2","M3","M4"]}]}\n'
            "```\n"
        )

        second = importer.import_plan(
            "session-a", "docs/plans/runtime/01-control.md", activate_candidate=True
        )

        self.assertTrue(second.activated)
        updated = {node.node_key: node for node in store.nodes(first.run_id)}
        self.assertEqual(nodes["M1"].node_id, updated["M1"].node_id)
        self.assertEqual(nodes["M2"].node_id, updated["M2"].node_id)
        self.assertEqual(WorkflowNodeState.SUCCEEDED, updated["M1"].state)
        self.assertEqual(WorkflowNodeState.SUCCEEDED, updated["M2"].state)
        self.assertEqual(WorkflowNodeState.PENDING, updated["M3"].state)
        self.assertEqual(WorkflowNodeState.PENDING, updated["M4"].state)
        self.assertEqual(WorkflowNodeState.PENDING, updated["M5"].state)
        with database.connect() as connection:
            edges = {
                (row["from_node_id"], row["to_node_id"])
                for row in connection.execute(
                    "SELECT from_node_id, to_node_id FROM workflow_edges WHERE run_id=?",
                    (first.run_id,),
                )
            }
        self.assertEqual(
            {
                (updated["M1"].node_id, updated["M2"].node_id),
                (updated["M3"].node_id, updated["M4"].node_id),
                (updated["M1"].node_id, updated["M5"].node_id),
                (updated["M2"].node_id, updated["M5"].node_id),
                (updated["M3"].node_id, updated["M5"].node_id),
                (updated["M4"].node_id, updated["M5"].node_id),
            },
            edges,
        )

    def test_rejects_oversized_plan_before_parsing(self) -> None:
        self.plan.write_bytes(b"# Runtime\n" + b"x" * (2 * 1024 * 1024))

        with self.assertRaises(CoordinatorError) as rejected:
            TopologyParser(self.repo).parse("docs/plans/runtime/01-control.md")

        self.assertEqual("workflow_topology_too_large", rejected.exception.code)

    def test_rejects_excessive_milestone_count(self) -> None:
        milestones = [
            {"id": f"M{index}", "title": f"step {index}", "depends_on": []}
            for index in range(1, 202)
        ]
        payload = {
            "schema": 1,
            "workflow_id": "runtime-control",
            "goal": "Runtime control",
            "milestones": milestones,
        }
        self._write(f"```zircon-workflow\n{json.dumps(payload)}\n```\n")

        with self.assertRaises(CoordinatorError) as rejected:
            TopologyParser(self.repo).parse("docs/plans/runtime/01-control.md")

        self.assertEqual("workflow_topology_too_many_nodes", rejected.exception.code)


if __name__ == "__main__":
    unittest.main()

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.control_plane.snapshot import ControlSnapshotService
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.workflows.projections import WorkflowProjectionService
from tools.session_coordinator.workflows.store import WorkflowStore


class ControlSnapshotTests(unittest.TestCase):
    def test_codex_projection_is_bounded_ordered_and_path_free(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            with database.transaction() as connection:
                rows = []
                for index in range(1005):
                    state = "active" if index == 1004 else "archived"
                    location = "active" if state == "active" else "archived"
                    rows.append(
                        (
                            f"thread-{index:04d}",
                            f"C:/private/rollout-{index}.jsonl",
                            location,
                            state,
                            "E:/Git/ZirconEngine",
                            "Codex Desktop",
                            "0.test",
                            "user",
                            "task_started" if state == "active" else "task_completed",
                            f"turn-{index}",
                            "safe_diagnostic" if index == 1004 else None,
                            "2026-07-13T00:00:00+00:00",
                            f"2026-07-13T00:{index % 60:02d}:00+00:00",
                            "2026-07-13T01:00:00+00:00",
                            index,
                            index,
                        )
                    )
                connection.executemany(
                    """
                    INSERT INTO codex_sessions(
                        thread_id, rollout_path, source_location, state, cwd,
                        originator, cli_version, thread_source, last_event,
                        last_turn_id, diagnostic_code, first_seen_at,
                        last_activity_at, last_synced_at, source_mtime_ns,
                        source_size
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    rows,
                )
                connection.execute(
                    """
                    INSERT INTO codex_sync_runs(
                        run_id, trigger_kind, status, scanned_count,
                        changed_count, diagnostic_count, unavailable_count,
                        duration_ms, source_revision, error_code, created_at,
                        completed_at
                    ) VALUES (
                        'run-latest', 'periodic', 'succeeded', 1005, 1, 1, 0,
                        12, 'revision', NULL, '2026-07-13T01:00:00+00:00',
                        '2026-07-13T01:00:01+00:00'
                    )
                    """
                )
            service = ControlSnapshotService(
                database,
                WorkflowProjectionService(),
                lambda _connection: {
                    "status": "ok",
                    "codexSync": {"queueDepth": 7, "lastErrorCode": None},
                },
            )

            projection = service.build()["codexSessions"]

            self.assertEqual(1005, projection["total"])
            self.assertTrue(projection["truncated"])
            self.assertEqual(1000, len(projection["rows"]))
            self.assertEqual("thread-1004", projection["rows"][0]["threadId"])
            self.assertEqual(1, projection["stateCounts"]["active"])
            self.assertEqual(1004, projection["stateCounts"]["archived"])
            self.assertEqual(7, projection["queueDepth"])
            self.assertEqual("run-latest", projection["lastRun"]["runId"])
            serialized = json.dumps(projection)
            self.assertNotIn("rollout", serialized.casefold())
            self.assertNotIn("E:/Git/ZirconEngine", serialized)

    def test_snapshot_contains_consistent_cursor_and_domain_sections(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            sessions = SessionService(database, repo)
            session = sessions.register(session_id="session-a")
            WorkflowStore(database).synchronize_session(session)
            service = ControlSnapshotService(
                database,
                WorkflowProjectionService(),
                lambda connection: {
                    "status": "ok",
                    "instanceId": "instance-a",
                    "eventCount": connection.execute(
                        "SELECT COUNT(*) FROM events"
                    ).fetchone()[0],
                },
            )

            snapshot = service.build()

            self.assertEqual(1, snapshot["projectionVersion"])
            self.assertGreaterEqual(snapshot["eventCursor"], 1)
            self.assertEqual("session-a", snapshot["sessions"][0]["sessionId"])
            self.assertEqual(1, len(snapshot["workflows"]))
            self.assertIsNone(snapshot["workflows"][0]["topologyHash"])
            with database.connect() as detail_connection:
                detail = WorkflowProjectionService().workflow_detail(
                    detail_connection, snapshot["workflows"][0]["runId"]
                )
            self.assertIsNone(detail["topologyHash"])
            self.assertEqual("goal", detail["nodes"][0]["stage"])
            self.assertEqual(snapshot["eventCursor"], snapshot["service"]["eventCount"])
            self.assertEqual(
                {
                    "service",
                    "workflows",
                    "sessions",
                    "codexSessions",
                    "failures",
                    "collaboration",
                    "validation",
                    "git",
                    "audit",
                },
                set(snapshot) - {"projectionVersion", "eventCursor"},
            )

    def test_snapshot_projects_legacy_oversized_event_payloads(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            sessions = SessionService(database, repo)
            session = sessions.register(session_id="session-a")
            WorkflowStore(database).synchronize_session(session)
            marker = "must-not-cross-control-boundary"
            oversized = json.dumps({"paths": [marker * 1024]})
            with database.transaction() as connection:
                connection.execute(
                    "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, 'now')",
                    ("legacy.oversized", oversized),
                )
            service = ControlSnapshotService(
                database,
                WorkflowProjectionService(),
                lambda _connection: {"status": "ok"},
            )

            snapshot = service.build()

            event = snapshot["audit"][-1]
            self.assertEqual("legacy.oversized", event["type"])
            self.assertEqual(True, event["payload"]["truncated"])
            self.assertGreater(event["payload"]["originalBytes"], 16 * 1024)
            self.assertNotIn(marker, json.dumps(snapshot))

    def test_snapshot_omits_heavy_internal_manifests_and_patch_objects(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            sessions = SessionService(database, repo)
            session = sessions.register(session_id="session-a")
            WorkflowStore(database).synchronize_session(session)
            marker = "internal-content-must-not-cross-control-boundary"
            large_value = marker * 1024
            manifest = json.dumps([large_value])
            patch_objects = json.dumps({"README.md": large_value})
            with database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO baseline_epochs(
                        head_commit, index_tree, health, manifest_json, created_at
                    ) VALUES ('head', 'tree', 'healthy', ?, 'now')
                    """,
                    (json.dumps({"README.md": large_value}),),
                )
                connection.execute(
                    "INSERT INTO objects(object_hash, byte_count, compressed_byte_count, created_at) "
                    "VALUES ('object-a', 1, 1, 'now')"
                )
                connection.execute(
                    """
                    INSERT INTO patches(
                        session_id, patch_object_hash, targets_json, base_hashes_json,
                        base_objects_json, current_objects_json, status,
                        created_at, updated_at
                    ) VALUES ('session-a', 'object-a', '["README.md"]', '{}', ?, ?,
                              'needs_rebase', 'now', 'now')
                    """,
                    (patch_objects, patch_objects),
                )
                connection.execute(
                    """
                    INSERT INTO validation_copies(
                        job_id, session_id, job_root, source_root, target_root,
                        head_commit, manifest_json, status, created_at
                    ) VALUES ('copy-a', 'session-a', 'job', 'source', 'target',
                              'head', ?, 'planned', 'now')
                    """,
                    (manifest,),
                )
            service = ControlSnapshotService(
                database,
                WorkflowProjectionService(),
                lambda _connection: {"status": "ok"},
            )

            snapshot = service.build()

            baseline = snapshot["collaboration"]["baseline"]
            patch = snapshot["collaboration"]["patches"][0]
            copy = snapshot["validation"]["validationCopies"][0]
            self.assertGreater(baseline["manifest_bytes"], 16 * 1024)
            self.assertGreater(patch["content_bytes"], 16 * 1024)
            self.assertEqual(1, patch["has_current_objects"])
            self.assertGreater(copy["manifest_bytes"], 16 * 1024)
            self.assertNotIn("manifest_json", baseline)
            self.assertNotIn("base_objects", patch)
            self.assertNotIn("current_objects", patch)
            self.assertNotIn("manifest", copy)
            self.assertNotIn(marker, json.dumps(snapshot))


if __name__ == "__main__":
    unittest.main()

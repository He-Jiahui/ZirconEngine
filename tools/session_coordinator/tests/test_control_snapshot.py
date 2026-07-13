from __future__ import annotations

import json
import shutil
import sqlite3
import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.cargo_jobs import (
    CargoCompatibility,
    CargoJobService,
    CargoLaneKind,
    TargetPathPolicy,
)
from tools.session_coordinator.control_plane.snapshot import ControlSnapshotService
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.workflows.projections import WorkflowProjectionService
from tools.session_coordinator.workflows.store import WorkflowStore


class ControlSnapshotTests(unittest.TestCase):
    def test_validation_lifecycle_summary_counts_only_existing_latest_targets(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            target_root = root / "cargo-targets"
            target_root.mkdir()
            database = Database(root / "state.sqlite3")
            migrate(database)
            SessionService(database, repo).register(session_id="session-a")
            jobs = CargoJobService(
                database,
                TargetPathPolicy([target_root]),
                repo_root=repo,
                free_space=lambda _path: 200 * 1024**3,
            )
            reusable = jobs.acquire(
                "session-a",
                CargoLaneKind.CHECK,
                compatibility=CargoCompatibility(
                    platform="windows",
                    toolchain="stable-x86_64-pc-windows-msvc",
                    target_architecture="x86_64-pc-windows-msvc",
                    workspace="Cargo.toml",
                    build_config="profile=dev",
                ),
            )
            pending = jobs.acquire("session-a", CargoLaneKind.TEST)
            failed = jobs.acquire("session-a", CargoLaneKind.WORKSPACE)
            historical = jobs.acquire("session-a", CargoLaneKind.GPU)
            with database.transaction() as connection:
                connection.execute(
                    "UPDATE cargo_jobs SET cleanup_status='failed' WHERE job_id=?",
                    (failed.job_id,),
                )
                connection.execute(
                    "UPDATE cargo_jobs SET cleanup_status='failed' WHERE job_id=?",
                    (historical.job_id,),
                )
            shutil.rmtree(historical.target_dir)

            with database.connect() as connection:
                projection = ControlSnapshotService._validation(connection)

        self.assertEqual(
            {
                "reusablePools": 1,
                "ephemeralTargets": 2,
                "pendingCleanup": 1,
                "failedCleanup": 1,
            },
            projection["artifactLifecycle"],
        )
        self.assertEqual(
            {reusable.target_dir, pending.target_dir, failed.target_dir},
            {item["target_dir"] for item in projection["currentCargoTargets"]},
        )
        self.assertTrue(Path(reusable.target_dir).exists() is False)
        self.assertTrue(Path(pending.target_dir).exists() is False)

    def test_git_projection_never_reads_internal_index_snapshot(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            database = Database(root / "state.sqlite3")
            migrate(database)
            reads: set[tuple[str | None, str | None]] = set()

            def authorize(
                action: int,
                table: str | None,
                column: str | None,
                _database: str | None,
                _trigger: str | None,
            ) -> int:
                if action == sqlite3.SQLITE_READ:
                    reads.add((table, column))
                    if (table, column) == ("finalize_requests", "index_snapshot"):
                        return sqlite3.SQLITE_DENY
                return sqlite3.SQLITE_OK

            with database.connect() as connection:
                connection.set_authorizer(authorize)
                projection = ControlSnapshotService._git(connection)

            self.assertEqual([], projection["finalizeRequests"])
            self.assertNotIn(("finalize_requests", "index_snapshot"), reads)

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

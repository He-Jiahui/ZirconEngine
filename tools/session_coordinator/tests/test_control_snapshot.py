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
from tools.session_coordinator.models import SessionStatus
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.workflows.projections import WorkflowProjectionService
from tools.session_coordinator.workflows.store import WorkflowStore


class ControlSnapshotTests(unittest.TestCase):
    def test_waiting_session_gets_one_same_plan_implementation_continuation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            plan = repo / "docs" / "plans" / "tooling" / "01-workflow.md"
            plan.parent.mkdir(parents=True)
            plan.write_text(
                "# Workflow\n\n"
                "## M1 — Main work\n\n"
                "### Implementation slices\n\n"
                "- [x] Complete the primary edit.\n"
                "- [ ] Write the remaining module documentation.\n\n"
                "### Testing stage\n\n"
                "- [ ] Run the deferred verification.\n\n"
                "## M2 — Later work\n\n"
                "### Implementation slices\n\n"
                "- [ ] Start a later milestone.\n",
                encoding="utf-8",
            )
            database = Database(root / "state.sqlite3")
            migrate(database)
            sessions = SessionService(database, repo)
            sessions.register(
                session_id="waiting-owner",
                plan_path="docs/plans/tooling/01-workflow.md",
            )
            sessions.set_status("waiting-owner", SessionStatus.ACTIVE)
            sessions.set_status("waiting-owner", SessionStatus.WAITING_VALIDATION)

            snapshot = ControlSnapshotService(
                database,
                WorkflowProjectionService(),
                lambda _connection: {"status": "ok"},
                repo_root=repo,
            ).build()

        self.assertEqual(
            [
                {
                    "sessionId": "waiting-owner",
                    "planPath": "docs/plans/tooling/01-workflow.md",
                    "waitKind": "validation",
                    "candidate": {
                        "milestone": "M1",
                        "title": "Write the remaining module documentation.",
                    },
                    "scopeClaimRequired": True,
                    "returnToPrimary": True,
                }
            ],
            snapshot["experience"]["continuations"],
        )

    def test_continuations_skip_testing_tasks_and_untrusted_plan_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            sessions = SessionService(database, repo)
            sessions.register(session_id="untrusted", plan_path="notes/queue.md")
            sessions.set_status("untrusted", SessionStatus.ACTIVE)
            sessions.set_status("untrusted", SessionStatus.WAITING_LEASE)

            snapshot = ControlSnapshotService(
                database,
                WorkflowProjectionService(),
                lambda _connection: {"status": "ok"},
                repo_root=repo,
            ).build()

        self.assertEqual([], snapshot["experience"]["continuations"])

    def test_snapshot_bounds_terminal_history_without_hiding_live_rows(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            sessions = SessionService(database, repo)
            sessions.register(session_id="active-session")
            for index in range(60):
                sessions.register(session_id=f"terminal-session-{index:02d}")

            with database.transaction() as connection:
                connection.execute(
                    "UPDATE sessions SET status='archived' WHERE session_id LIKE 'terminal-session-%'"
                )
                for index in range(60):
                    stamp = f"2026-07-16T00:{index:02d}:00+00:00"
                    connection.execute(
                        """
                        INSERT INTO workflow_runs(
                            run_id, session_id, workflow_key, plan_path, state,
                            created_at, updated_at
                        ) VALUES (?, 'active-session', ?, 'plan.md', 'archived', ?, ?)
                        """,
                        (f"terminal-run-{index:02d}", f"terminal-workflow-{index:02d}", stamp, stamp),
                    )
                    connection.execute(
                        """
                        INSERT INTO finalize_requests(
                            request_id, session_id, message, paths_json, categories_json,
                            untracked_json, validation_json, maintenance, status, created_at
                        ) VALUES (?, 'active-session', 'terminal', '[]', '{}', '[]', '[]', 0,
                                  'committed', ?)
                        """,
                        (f"terminal-finalize-{index:02d}", stamp),
                    )
                    connection.execute(
                        """
                        INSERT INTO cargo_jobs(
                            job_id, session_id, lane_kind, target_dir, target_key, status,
                            command_json, created_at, last_heartbeat_at, cleanup_policy,
                            cleanup_status
                        ) VALUES (?, 'active-session', 'test', ?, ?, 'released', '[]', ?, ?,
                                  'retained', 'retained')
                        """,
                        (
                            f"terminal-cargo-{index:02d}",
                            f"target-{index:02d}",
                            f"target-{index:02d}",
                            stamp,
                            stamp,
                        ),
                    )
                    connection.execute(
                        """
                        INSERT INTO validation_copies(
                            job_id, session_id, job_root, source_root, target_root,
                            head_commit, manifest_json, status, created_at, removed_at
                        ) VALUES (?, 'active-session', 'job', 'source', 'target', 'head', '[]',
                                  'removed', ?, ?)
                        """,
                        (f"terminal-copy-{index:02d}", stamp, stamp),
                    )
                connection.execute(
                    """
                    INSERT INTO workflow_runs(
                        run_id, session_id, workflow_key, plan_path, state, created_at, updated_at
                    ) VALUES ('active-run', 'active-session', 'active-workflow', 'plan.md',
                              'active', '2026-07-16T02:00:00+00:00', '2026-07-16T02:00:00+00:00')
                    """
                )
                connection.execute(
                    """
                    INSERT INTO finalize_requests(
                        request_id, session_id, message, paths_json, categories_json,
                        untracked_json, validation_json, maintenance, status, created_at
                    ) VALUES ('active-finalize', 'active-session', 'active', '[]', '{}', '[]',
                              '[]', 0, 'previewed', '2026-07-16T02:00:00+00:00')
                    """
                )
                connection.execute(
                    """
                    INSERT INTO cargo_jobs(
                        job_id, session_id, lane_kind, target_dir, target_key, status,
                        command_json, created_at, last_heartbeat_at, cleanup_policy,
                        cleanup_status
                    ) VALUES ('active-cargo', 'active-session', 'test', 'active-target',
                              'active-target', 'running', '[]', '2026-07-16T02:00:00+00:00',
                              '2026-07-16T02:00:00+00:00', 'retained', 'retained')
                    """
                )
                connection.execute(
                    """
                    INSERT INTO validation_copies(
                        job_id, session_id, job_root, source_root, target_root,
                        head_commit, manifest_json, status, created_at
                    ) VALUES ('active-copy', 'active-session', 'job', 'source', 'target', 'head',
                              '[]', 'running', '2026-07-16T02:00:00+00:00')
                    """
                )

            snapshot = ControlSnapshotService(
                database, WorkflowProjectionService(), lambda _connection: {"status": "ok"}
            ).build()

        self.assertEqual(51, len(snapshot["sessions"]))
        self.assertIn("active-session", {row["sessionId"] for row in snapshot["sessions"]})
        self.assertEqual(51, len(snapshot["workflows"]))
        self.assertIn("active-run", {row["runId"] for row in snapshot["workflows"]})
        self.assertEqual(51, len(snapshot["git"]["finalizeRequests"]))
        self.assertIn(
            "active-finalize",
            {row["request_id"] for row in snapshot["git"]["finalizeRequests"]},
        )
        self.assertEqual(51, len(snapshot["validation"]["cargoJobs"]))
        self.assertIn(
            "active-cargo", {row["job_id"] for row in snapshot["validation"]["cargoJobs"]}
        )
        self.assertEqual(51, len(snapshot["validation"]["validationCopies"]))
        self.assertIn(
            "active-copy",
            {row["job_id"] for row in snapshot["validation"]["validationCopies"]},
        )

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
            {reusable.job_id, pending.job_id, failed.job_id},
            {item["job_id"] for item in projection["currentCargoTargets"]},
        )
        self.assertTrue(Path(reusable.target_dir).exists() is False)
        self.assertTrue(Path(pending.target_dir).exists() is False)

    def test_validation_projection_exposes_only_safe_lane_fields_to_browser(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            target_root = root / "cargo-targets"
            target_root.mkdir()
            database = Database(root / "state.sqlite3")
            migrate(database)
            SessionService(database, repo).register(session_id="lane-owner")
            jobs = CargoJobService(
                database,
                TargetPathPolicy([target_root]),
                repo_root=repo,
                free_space=lambda _path: 200 * 1024**3,
            )
            jobs.acquire("lane-owner", CargoLaneKind.TEST)

            with database.connect() as connection:
                projection = ControlSnapshotService._validation(connection)

        lane_fields = {
            "job_id",
            "session_id",
            "lane_kind",
            "status",
            "created_at",
            "started_at",
            "finished_at",
            "released_at",
            "cleanup_policy",
            "cleanup_status",
            "process_observation",
        }
        sensitive_fields = {
            "command",
            "target_dir",
            "target_key",
            "compatibility_json",
            "compatibility_key",
            "pid",
            "exit_code",
            "cleanup_error",
        }
        for rows in (projection["cargoJobs"], projection["currentCargoTargets"]):
            self.assertEqual(1, len(rows))
            self.assertEqual(lane_fields, set(rows[0]))
            self.assertTrue(sensitive_fields.isdisjoint(rows[0]))

    def test_validation_projection_explains_running_process_observation_without_pid(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            target_root = root / "cargo-targets"
            target_root.mkdir()
            database = Database(root / "state.sqlite3")
            migrate(database)
            SessionService(database, repo).register(session_id="observed-owner")
            jobs = CargoJobService(
                database,
                TargetPathPolicy([target_root]),
                repo_root=repo,
                free_space=lambda _path: 200 * 1024**3,
            )
            job = jobs.acquire("observed-owner", CargoLaneKind.TEST)
            with database.transaction() as connection:
                connection.execute(
                    """
                    UPDATE cargo_jobs
                    SET status='running', process_tree_observed_at=?,
                        process_tree_live_pids_json='[12345]', process_tree_exited_at=NULL
                    WHERE job_id=?
                    """,
                    ("2026-07-17T12:00:00+00:00", job.job_id),
                )
            with database.connect() as connection:
                observed = ControlSnapshotService._validation(connection)
            with database.transaction() as connection:
                connection.execute(
                    "UPDATE cargo_jobs SET process_tree_exited_at=? WHERE job_id=?",
                    ("2026-07-17T12:01:00+00:00", job.job_id),
                )
            with database.connect() as connection:
                reconciling = ControlSnapshotService._validation(connection)

        self.assertEqual(
            "observed",
            observed["cargoJobs"][0]["process_observation"],
        )
        self.assertEqual(
            "observed",
            observed["currentCargoTargets"][0]["process_observation"],
        )
        self.assertEqual(
            "reconciling",
            reconciling["cargoJobs"][0]["process_observation"],
        )
        self.assertNotIn("process_tree_live_pids_json", observed["cargoJobs"][0])

    def test_validation_projection_exposes_a_bounded_safe_lane_queue(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            sessions = SessionService(database, repo)
            for session_id in ("running-owner", "cpu-next", "burst-owner", "gpu-next"):
                sessions.register(session_id=session_id)
            with database.transaction() as connection:
                connection.executemany(
                    """
                    INSERT INTO cargo_lane_reservations(
                        reservation_id, session_id, lane_scope, compatibility_key,
                        compatibility_json, target_dir, command_fingerprint, status,
                        execution_mode, burst_eligible, created_at, expires_at
                    ) VALUES (?, ?, ?, 'compatibility', '{}', NULL, 'command', ?, ?, ?, ?, ?)
                    """,
                    (
                        ("cpu-running", "running-owner", "cpu", "running", "warm", 0, "2026-07-17T00:00:00+00:00", "2026-07-17T01:00:00+00:00"),
                        ("cpu-pending", "cpu-next", "cpu", "pending", "warm", 1, "2026-07-17T00:01:00+00:00", "2026-07-17T01:01:00+00:00"),
                        ("cpu-burst-running", "burst-owner", "cpu", "running", "burst", 1, "2026-07-17T00:00:30+00:00", "2026-07-17T01:00:30+00:00"),
                        ("gpu-pending", "gpu-next", "gpu", "pending", "warm", 0, "2026-07-17T00:02:00+00:00", "2026-07-17T01:02:00+00:00"),
                    ),
                )
                connection.execute(
                    "UPDATE cargo_lane_reservations SET started_at='2026-07-17T00:00:00+00:00' "
                    "WHERE reservation_id='cpu-running'"
                )

            with database.connect() as connection:
                projection = ControlSnapshotService._validation(connection)

        self.assertEqual(
            [
                {
                    "reservationId": "cpu-running",
                    "sessionId": "running-owner",
                    "laneScope": "cpu",
                    "executionMode": "warm",
                    "burstEligible": False,
                    "status": "running",
                    "queuePosition": 1,
                    "createdAt": "2026-07-17T00:00:00+00:00",
                    "expiresAt": "2026-07-17T01:00:00+00:00",
                },
                {
                    "reservationId": "cpu-pending",
                    "sessionId": "cpu-next",
                    "laneScope": "cpu",
                    "executionMode": "warm",
                    "burstEligible": True,
                    "status": "pending",
                    "queuePosition": 2,
                    "createdAt": "2026-07-17T00:01:00+00:00",
                    "expiresAt": "2026-07-17T01:01:00+00:00",
                },
                {
                    "reservationId": "cpu-burst-running",
                    "sessionId": "burst-owner",
                    "laneScope": "cpu",
                    "executionMode": "burst",
                    "burstEligible": True,
                    "status": "running",
                    "queuePosition": 1,
                    "createdAt": "2026-07-17T00:00:30+00:00",
                    "expiresAt": "2026-07-17T01:00:30+00:00",
                },
                {
                    "reservationId": "gpu-pending",
                    "sessionId": "gpu-next",
                    "laneScope": "gpu",
                    "executionMode": "warm",
                    "burstEligible": False,
                    "status": "pending",
                    "queuePosition": 1,
                    "createdAt": "2026-07-17T00:02:00+00:00",
                    "expiresAt": "2026-07-17T01:02:00+00:00",
                },
            ],
            projection["cargoReservations"],
        )
        self.assertEqual(
            {"capacity": 1, "active": 1, "eligiblePending": 1},
            projection["cpuBurst"],
        )

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
                    "experience",
                    "git",
                    "audit",
                },
                set(snapshot) - {"projectionVersion", "eventCursor"},
            )

    def test_experience_projection_summarizes_quiet_sync_and_live_resource_owner(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            SessionService(database, repo).register(session_id="owner-session")
            with database.transaction() as connection:
                connection.executemany(
                    """
                    INSERT INTO codex_sync_runs(
                        run_id, trigger_kind, status, scanned_count, changed_count,
                        diagnostic_count, unavailable_count, duration_ms, source_revision,
                        error_code, created_at, completed_at
                    ) VALUES (?, 'periodic', 'succeeded', 12, ?, 0, 0, ?, 'revision', NULL,
                              datetime('now'), datetime('now'))
                    """,
                    (("quiet-run", 0, 20), ("visible-run", 3, 40)),
                )
                connection.execute(
                    """
                    INSERT INTO cargo_jobs(
                        job_id, session_id, lane_kind, target_dir, target_key, status,
                        command_json, created_at, last_heartbeat_at, cleanup_policy,
                        cleanup_status
                    ) VALUES ('running-job', 'owner-session', 'test', 'D:/cargo-targets/live',
                              'live', 'running', '[]', datetime('now'), datetime('now'),
                              'retained', 'retained')
                    """
                )
            snapshot = ControlSnapshotService(
                database, WorkflowProjectionService(), lambda _connection: {"status": "ok"}
            ).build()

        experience = snapshot["experience"]
        self.assertEqual(2, experience["sync"]["runs"])
        self.assertEqual(1, experience["sync"]["quietRuns"])
        self.assertEqual(3, experience["sync"]["visibleChanges"])
        self.assertEqual(30, experience["sync"]["averageDurationMs"])
        self.assertEqual(
            [{
                "kind": "cargo",
                "ownerSessionId": "owner-session",
                "laneKind": "test",
                "status": "running",
                "createdAt": experience["blockers"][0]["createdAt"],
            }],
            experience["blockers"],
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
                        head_commit, manifest_json, status, created_at,
                        materialization_started_at
                    ) VALUES ('copy-a', 'session-a', 'job', 'source', 'target',
                              'head', ?, 'planned', 'now', 'now')
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
            self.assertEqual("materializing", copy["status"])
            self.assertNotIn("manifest_json", baseline)
            self.assertNotIn("base_objects", patch)
            self.assertNotIn("current_objects", patch)
            self.assertNotIn("manifest", copy)
            self.assertNotIn(marker, json.dumps(snapshot))


if __name__ == "__main__":
    unittest.main()

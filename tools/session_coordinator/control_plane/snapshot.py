from __future__ import annotations

import json
from collections.abc import Callable
from pathlib import Path

from ..database import Database
from ..event_payloads import (
    CONTROL_EVENT_PAYLOAD_COLUMNS,
    project_control_event_payload,
)
from ..workflows.projections import WorkflowProjectionService
from ..work_continuations import WorkContinuationService


class ControlSnapshotService:
    """Reads all persisted control domains from one SQLite snapshot transaction."""

    _TERMINAL_HISTORY_LIMIT = 50
    _AUDIT_EVENT_LIMIT = 200

    def __init__(
        self,
        database: Database,
        workflows: WorkflowProjectionService,
        service_state: Callable[[object], dict[str, object]],
        *,
        repo_root: str | Path | None = None,
    ):
        self.database = database
        self.workflows = workflows
        self.service_state = service_state
        self.continuations = WorkContinuationService(repo_root)

    def build(self) -> dict[str, object]:
        with self.database.connect() as connection:
            connection.execute("BEGIN")
            try:
                cursor = int(
                    connection.execute("SELECT COALESCE(MAX(event_id), 0) FROM events").fetchone()[0]
                )
                service = self.service_state(connection)
                snapshot = {
                    "projectionVersion": 1,
                    "eventCursor": cursor,
                    "service": service,
                    "workflows": self.workflows.workflow_summaries(
                        connection, terminal_history_limit=self._TERMINAL_HISTORY_LIMIT
                    ),
                    "sessions": self._sessions(connection),
                    "codexSessions": self._codex_sessions(connection, service),
                    "failures": self._failures(connection),
                    "collaboration": self._collaboration(connection),
                    "validation": self._validation(connection),
                    "experience": {
                        **self._experience(connection),
                        "continuations": self.continuations.project(connection),
                    },
                    "git": self._git(connection),
                    "audit": self._audit(connection),
                }
                connection.commit()
            except BaseException:
                connection.rollback()
                raise
        return snapshot

    @staticmethod
    def _codex_sessions(connection, service: dict[str, object]) -> dict[str, object]:
        limit = 1000
        rows = connection.execute(
            """
            SELECT thread_id, source_location, state, originator, cli_version,
                   thread_source, last_event, last_turn_id, bound_session_id,
                   diagnostic_code, first_seen_at, last_activity_at, last_synced_at
            FROM codex_sessions
            ORDER BY CASE state
                WHEN 'active' THEN 0 WHEN 'idle' THEN 1
                WHEN 'archived' THEN 2 ELSE 3 END,
                last_activity_at DESC, thread_id
            LIMIT ?
            """,
            (limit,),
        ).fetchall()
        state_counts = {state: 0 for state in ("active", "idle", "archived", "unavailable")}
        for row in connection.execute(
            "SELECT state, COUNT(*) AS count FROM codex_sessions GROUP BY state"
        ):
            state_counts[str(row["state"])] = int(row["count"])
        source_counts = {location: 0 for location in ("active", "archived", "missing")}
        for row in connection.execute(
            "SELECT source_location, COUNT(*) AS count FROM codex_sessions GROUP BY source_location"
        ):
            source_counts[str(row["source_location"])] = int(row["count"])
        total = sum(state_counts.values())
        latest = connection.execute(
            "SELECT * FROM codex_sync_runs ORDER BY created_at DESC, run_id DESC LIMIT 1"
        ).fetchone()
        successful = connection.execute(
            """
            SELECT completed_at FROM codex_sync_runs
            WHERE status='succeeded'
            ORDER BY created_at DESC, run_id DESC LIMIT 1
            """
        ).fetchone()
        worker = service.get("codexSync")
        worker_state = worker if isinstance(worker, dict) else {}
        return {
            "rows": [
                {
                    "threadId": row["thread_id"],
                    "sourceLocation": row["source_location"],
                    "state": row["state"],
                    "originator": row["originator"],
                    "cliVersion": row["cli_version"],
                    "threadSource": row["thread_source"],
                    "lastEvent": row["last_event"],
                    "lastTurnId": row["last_turn_id"],
                    "boundSessionId": row["bound_session_id"],
                    "diagnosticCode": row["diagnostic_code"],
                    "firstSeenAt": row["first_seen_at"],
                    "lastActivityAt": row["last_activity_at"],
                    "lastSyncedAt": row["last_synced_at"],
                }
                for row in rows
            ],
            "total": total,
            "truncated": total > limit,
            "stateCounts": state_counts,
            "sourceCounts": source_counts,
            "queueDepth": int(worker_state.get("queueDepth", 0)),
            "lastSuccessfulAt": successful["completed_at"] if successful else None,
            "lastTerminalCode": (
                worker_state.get("lastErrorCode")
                or (latest["error_code"] if latest is not None else None)
                or (latest["status"] if latest is not None else None)
            ),
            "lastRun": (
                {
                    "runId": latest["run_id"],
                    "trigger": latest["trigger_kind"],
                    "status": latest["status"],
                    "scannedCount": latest["scanned_count"],
                    "changedCount": latest["changed_count"],
                    "diagnosticCount": latest["diagnostic_count"],
                    "unavailableCount": latest["unavailable_count"],
                    "durationMs": latest["duration_ms"],
                    "errorCode": latest["error_code"],
                    "createdAt": latest["created_at"],
                    "completedAt": latest["completed_at"],
                }
                if latest is not None
                else None
            ),
        }

    @staticmethod
    def _sessions(connection) -> list[dict[str, object]]:
        return [
            {
                "sessionId": row["session_id"],
                "displayName": row["display_name"],
                "planPath": row["plan_path"],
                "status": row["status"],
                "statusReason": row["status_reason"],
                "baseHead": row["base_head"],
                "baselineEpoch": row["baseline_epoch"],
                "writeScope": json.loads(row["write_scope_json"]),
                "updatedAt": row["updated_at"],
                "lastHeartbeatAt": row["last_heartbeat_at"],
            }
            for row in connection.execute(
                """
                WITH recent_terminal AS (
                    SELECT session_id FROM sessions
                    WHERE status IN ('completed', 'stale', 'archived', 'cancelled')
                    ORDER BY updated_at DESC, session_id DESC LIMIT ?
                )
                SELECT * FROM sessions
                WHERE status NOT IN ('completed', 'stale', 'archived', 'cancelled')
                   OR session_id IN (SELECT session_id FROM recent_terminal)
                ORDER BY updated_at DESC, session_id
                """,
                (ControlSnapshotService._TERMINAL_HISTORY_LIMIT,),
            )
        ]

    @staticmethod
    def _failures(connection) -> dict[str, object]:
        nodes = [
            dict(row)
            for row in connection.execute(
                """
                WITH recent_fixed AS (
                    SELECT node_id FROM failure_nodes
                    WHERE status='fixed'
                    ORDER BY resolved_at DESC, created_at DESC, node_id DESC LIMIT ?
                )
                SELECT * FROM failure_nodes
                WHERE status='open' OR node_id IN (SELECT node_id FROM recent_fixed)
                ORDER BY priority, created_at, node_id
                """,
                (ControlSnapshotService._TERMINAL_HISTORY_LIMIT,),
            )
        ]
        diagnostics = [
            {
                "diagnosticId": row["diagnostic_id"],
                "code": row["code"],
                "message": row["message"],
                "paths": json.loads(row["paths_json"]),
                "createdAt": row["created_at"],
            }
            for row in connection.execute(
                "SELECT * FROM failure_diagnostics ORDER BY diagnostic_id DESC LIMIT ?",
                (ControlSnapshotService._AUDIT_EVENT_LIMIT,),
            )
        ]
        return {"nodes": nodes, "diagnostics": diagnostics}

    @staticmethod
    def _collaboration(connection) -> dict[str, object]:
        leases = [dict(row) for row in connection.execute("SELECT * FROM leases ORDER BY display_path")]
        patches = []
        for row in connection.execute(
            """
            SELECT patch_id, session_id, patch_object_hash, targets_json, status,
                   error_text, created_at, updated_at, applied_at,
                   LENGTH(CAST(base_hashes_json AS BLOB))
                       + LENGTH(CAST(base_objects_json AS BLOB))
                       + COALESCE(LENGTH(CAST(current_objects_json AS BLOB)), 0)
                       AS content_bytes,
                   CASE WHEN current_objects_json IS NULL THEN 0 ELSE 1 END
                       AS has_current_objects
            FROM patches ORDER BY patch_id DESC LIMIT 500
            """
        ):
            item = dict(row)
            item["targets"] = json.loads(item.pop("targets_json"))
            patches.append(item)
        baseline = connection.execute(
            """
            SELECT epoch_id, head_commit, index_tree, health, created_at,
                   degraded_at, degraded_reason,
                   LENGTH(CAST(manifest_json AS BLOB)) AS manifest_bytes
            FROM baseline_epochs ORDER BY epoch_id DESC LIMIT 1
            """
        ).fetchone()
        return {"baseline": dict(baseline) if baseline else None, "leases": leases, "patches": patches}

    @staticmethod
    def _validation(connection) -> dict[str, object]:
        jobs = []
        for row in connection.execute(
            """
            WITH recent_terminal AS (
                SELECT job_id FROM cargo_jobs
                WHERE status NOT IN ('leased', 'running')
                ORDER BY created_at DESC, job_id DESC LIMIT ?
            )
            SELECT job_id, session_id, lane_kind, status, created_at,
                   started_at, finished_at, released_at, cleanup_policy, cleanup_status,
                   process_tree_observed_at, process_tree_exited_at
            FROM cargo_jobs
            WHERE status IN ('leased', 'running')
               OR job_id IN (SELECT job_id FROM recent_terminal)
            ORDER BY created_at DESC, job_id DESC
            """,
            (ControlSnapshotService._TERMINAL_HISTORY_LIMIT,),
        ):
            jobs.append(ControlSnapshotService._cargo_lane_projection(row))
        copies = []
        for row in connection.execute(
            """
            SELECT job_id, session_id, job_root, source_root, target_root,
                   head_commit,
                   CASE
                       WHEN status = 'planned' AND materialization_started_at IS NOT NULL
                           THEN 'materializing'
                       ELSE status
                   END AS status,
                   created_at, removed_at,
                   LENGTH(CAST(manifest_json AS BLOB)) AS manifest_bytes
            FROM validation_copies
            WHERE status IN ('planned', 'materialized', 'running', 'cleanup_pending')
               OR job_id IN (
                    SELECT job_id FROM validation_copies
                    WHERE status NOT IN ('planned', 'materialized', 'running', 'cleanup_pending')
                    ORDER BY created_at DESC, job_id DESC LIMIT ?
               )
            ORDER BY created_at DESC, job_id DESC
            """,
            (ControlSnapshotService._TERMINAL_HISTORY_LIMIT,),
        ):
            copies.append(dict(row))
        current_targets = ControlSnapshotService._current_cargo_targets(connection)
        cpu_burst = connection.execute(
            """
            SELECT
                CASE WHEN EXISTS(
                    SELECT 1 FROM cargo_lane_reservations
                    WHERE lane_scope='cpu' AND execution_mode='burst'
                      AND status IN ('leased', 'running', 'finished')
                ) THEN 1 ELSE 0 END AS active,
                COALESCE(SUM(CASE
                    WHEN lane_scope='cpu' AND execution_mode='warm'
                     AND burst_eligible=1 AND status='pending' THEN 1
                    ELSE 0
                END), 0) AS eligible_pending
            FROM cargo_lane_reservations
            """
        ).fetchone()
        reservations = []
        for row in connection.execute(
            """
            WITH active_reservations AS (
                SELECT reservation_id, session_id, lane_scope, execution_mode, burst_eligible,
                       status, created_at, expires_at,
                       ROW_NUMBER() OVER (
                           PARTITION BY lane_scope, execution_mode
                           ORDER BY created_at, reservation_id
                       ) AS queue_position
                FROM cargo_lane_reservations
                WHERE status IN ('pending', 'leased', 'running')
            )
            SELECT reservation_id, session_id, lane_scope, execution_mode, burst_eligible,
                   status, queue_position,
                   created_at, expires_at
            FROM active_reservations
            ORDER BY CASE lane_scope WHEN 'cpu' THEN 0 ELSE 1 END,
                     CASE execution_mode WHEN 'warm' THEN 0 ELSE 1 END,
                     queue_position, reservation_id
            LIMIT 20
            """
        ):
            reservations.append(
                {
                    "reservationId": row["reservation_id"],
                    "sessionId": row["session_id"],
                    "laneScope": row["lane_scope"],
                    "executionMode": row["execution_mode"],
                    "burstEligible": bool(row["burst_eligible"]),
                    "status": row["status"],
                    "queuePosition": int(row["queue_position"]),
                    "createdAt": row["created_at"],
                    "expiresAt": row["expires_at"],
                }
            )
        return {
            "cargoJobs": jobs,
            "validationCopies": copies,
            "currentCargoTargets": current_targets,
            "cargoReservations": reservations,
            "cpuBurst": {
                "capacity": 1,
                "active": int(cpu_burst["active"]),
                "eligiblePending": int(cpu_burst["eligible_pending"]),
            },
            "artifactLifecycle": ControlSnapshotService._artifact_lifecycle(current_targets),
        }

    @staticmethod
    def _experience(connection) -> dict[str, object]:
        """Project small operator-facing flow metrics without exposing raw history."""
        sync = connection.execute(
            """
            SELECT COUNT(*) AS runs,
                   COALESCE(SUM(CASE WHEN changed_count=0 THEN 1 ELSE 0 END), 0) AS quiet_runs,
                   COALESCE(SUM(changed_count), 0) AS visible_changes,
                   COALESCE(ROUND(AVG(duration_ms)), 0) AS average_duration_ms
            FROM codex_sync_runs
            WHERE julianday(created_at) >= julianday('now', '-1 day')
            """,
        ).fetchone()
        blockers = [
            {
                "kind": "cargo",
                "ownerSessionId": row["session_id"],
                "laneKind": row["lane_kind"],
                "status": row["status"],
                "createdAt": row["created_at"],
            }
            for row in connection.execute(
                """
                SELECT session_id, lane_kind, status, created_at
                FROM cargo_jobs
                WHERE status IN ('leased', 'running')
                ORDER BY CASE status WHEN 'running' THEN 0 ELSE 1 END, created_at, job_id
                LIMIT 20
                """
            )
        ]
        return {
            "sync": {
                "runs": int(sync["runs"]),
                "quietRuns": int(sync["quiet_runs"]),
                "visibleChanges": int(sync["visible_changes"]),
                "averageDurationMs": int(sync["average_duration_ms"]),
            },
            "blockers": blockers,
        }

    @staticmethod
    def _current_cargo_targets(connection) -> list[dict[str, object]]:
        """Project one live row per Cargo target directory for the control UI."""
        rows = connection.execute(
            """
            WITH latest_target AS (
                SELECT *,
                       ROW_NUMBER() OVER (
                           PARTITION BY target_dir
                           ORDER BY created_at DESC, job_id DESC
                       ) AS row_number
                FROM cargo_jobs
                WHERE target_dir <> ''
            )
            SELECT job_id, session_id, lane_kind, status, created_at,
                   started_at, finished_at, released_at, cleanup_policy, cleanup_status,
                   process_tree_observed_at, process_tree_exited_at, target_dir
            FROM latest_target
            WHERE row_number=1 AND cleanup_status <> 'deleted'
            ORDER BY created_at DESC, job_id DESC
            """
        ).fetchall()
        targets = []
        for row in rows:
            try:
                exists = Path(row["target_dir"]).exists()
            except OSError:
                exists = False
            if not exists:
                continue
            targets.append(ControlSnapshotService._cargo_lane_projection(row))
        return targets

    @staticmethod
    def _cargo_lane_projection(row) -> dict[str, object]:
        """Expose only the lane state needed to explain a local validation wait."""
        process_observation = "not_applicable"
        if row["status"] == "running":
            if row["process_tree_exited_at"] is not None:
                process_observation = "reconciling"
            elif row["process_tree_observed_at"] is not None:
                process_observation = "observed"
            else:
                process_observation = "awaiting_observation"
        return {
            "job_id": row["job_id"],
            "session_id": row["session_id"],
            "lane_kind": row["lane_kind"],
            "status": row["status"],
            "created_at": row["created_at"],
            "started_at": row["started_at"],
            "finished_at": row["finished_at"],
            "released_at": row["released_at"],
            "cleanup_policy": row["cleanup_policy"],
            "cleanup_status": row["cleanup_status"],
            "process_observation": process_observation,
        }

    @staticmethod
    def _artifact_lifecycle(current_targets: list[dict[str, object]]) -> dict[str, int]:
        """Count the live target projection without historical job retries."""
        counts = {
            "reusablePools": 0,
            "ephemeralTargets": 0,
            "pendingCleanup": 0,
            "failedCleanup": 0,
        }
        for row in current_targets:
            policy = row["cleanup_policy"]
            status = row["cleanup_status"]
            if policy == "retained" and status == "retained":
                counts["reusablePools"] += 1
            if policy == "delete_on_release" and status != "deleted":
                counts["ephemeralTargets"] += 1
            if status == "pending":
                counts["pendingCleanup"] += 1
            if status == "failed":
                counts["failedCleanup"] += 1
        return counts

    @staticmethod
    def _git(connection) -> dict[str, object]:
        requests = []
        for row in connection.execute(
            """
            WITH recent_terminal AS (
                SELECT request_id FROM finalize_requests
                WHERE status IN ('committed', 'failed')
                ORDER BY created_at DESC, request_id DESC LIMIT ?
            )
            SELECT request_id, session_id, message, paths_json, categories_json,
                   untracked_json, validation_json, maintenance, status,
                   commit_sha, error_text, created_at, completed_at, start_head,
                   index_existed, ref_updated_sha
            FROM finalize_requests
            WHERE status NOT IN ('committed', 'failed')
               OR request_id IN (SELECT request_id FROM recent_terminal)
            ORDER BY created_at DESC, request_id DESC
            """,
            (ControlSnapshotService._TERMINAL_HISTORY_LIMIT,),
        ):
            item = dict(row)
            for key in ("paths_json", "categories_json", "untracked_json", "validation_json"):
                item[key.removesuffix("_json")] = json.loads(item.pop(key))
            requests.append(item)
        return {"finalizeRequests": requests}

    @staticmethod
    def _audit(connection) -> list[dict[str, object]]:
        rows = connection.execute(
            f"""
            SELECT event_id, session_id, event_type, created_at,
                   {CONTROL_EVENT_PAYLOAD_COLUMNS}
            FROM events ORDER BY event_id DESC LIMIT ?
            """,
            (ControlSnapshotService._AUDIT_EVENT_LIMIT,),
        ).fetchall()
        return [
            {
                "eventId": int(row["event_id"]),
                "sessionId": row["session_id"],
                "type": row["event_type"],
                "payload": project_control_event_payload(row),
                "createdAt": row["created_at"],
            }
            for row in reversed(rows)
        ]

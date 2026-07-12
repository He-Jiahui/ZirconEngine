from __future__ import annotations

import json
from collections.abc import Callable

from ..database import Database
from ..event_payloads import (
    CONTROL_EVENT_PAYLOAD_COLUMNS,
    project_control_event_payload,
)
from ..workflows.projections import WorkflowProjectionService


class ControlSnapshotService:
    """Reads all persisted control domains from one SQLite snapshot transaction."""

    def __init__(
        self,
        database: Database,
        workflows: WorkflowProjectionService,
        service_state: Callable[[object], dict[str, object]],
    ):
        self.database = database
        self.workflows = workflows
        self.service_state = service_state

    def build(self) -> dict[str, object]:
        with self.database.connect() as connection:
            connection.execute("BEGIN")
            try:
                cursor = int(
                    connection.execute("SELECT COALESCE(MAX(event_id), 0) FROM events").fetchone()[0]
                )
                snapshot = {
                    "projectionVersion": 1,
                    "eventCursor": cursor,
                    "service": self.service_state(connection),
                    "workflows": self.workflows.workflow_summaries(connection),
                    "sessions": self._sessions(connection),
                    "failures": self._failures(connection),
                    "collaboration": self._collaboration(connection),
                    "validation": self._validation(connection),
                    "git": self._git(connection),
                    "audit": self._audit(connection),
                }
                connection.commit()
            except BaseException:
                connection.rollback()
                raise
        return snapshot

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
                "SELECT * FROM sessions ORDER BY updated_at DESC, session_id"
            )
        ]

    @staticmethod
    def _failures(connection) -> dict[str, object]:
        nodes = [dict(row) for row in connection.execute("SELECT * FROM failure_nodes ORDER BY priority, created_at")]
        diagnostics = [
            {
                "diagnosticId": row["diagnostic_id"],
                "code": row["code"],
                "message": row["message"],
                "paths": json.loads(row["paths_json"]),
                "createdAt": row["created_at"],
            }
            for row in connection.execute(
                "SELECT * FROM failure_diagnostics ORDER BY diagnostic_id DESC LIMIT 500"
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
        for row in connection.execute("SELECT * FROM cargo_jobs ORDER BY created_at DESC LIMIT 500"):
            item = dict(row)
            item["command"] = json.loads(item.pop("command_json"))
            jobs.append(item)
        copies = []
        for row in connection.execute(
            """
            SELECT job_id, session_id, job_root, source_root, target_root,
                   head_commit, status, created_at, removed_at,
                   LENGTH(CAST(manifest_json AS BLOB)) AS manifest_bytes
            FROM validation_copies ORDER BY created_at DESC LIMIT 500
            """
        ):
            copies.append(dict(row))
        return {"cargoJobs": jobs, "validationCopies": copies}

    @staticmethod
    def _git(connection) -> dict[str, object]:
        requests = []
        for row in connection.execute(
            "SELECT * FROM finalize_requests ORDER BY created_at DESC LIMIT 500"
        ):
            item = dict(row)
            for key in ("paths_json", "categories_json", "untracked_json", "validation_json"):
                item[key.removesuffix("_json")] = json.loads(item.pop(key))
            item.pop("index_snapshot", None)
            requests.append(item)
        return {"finalizeRequests": requests}

    @staticmethod
    def _audit(connection) -> list[dict[str, object]]:
        rows = connection.execute(
            f"""
            SELECT event_id, session_id, event_type, created_at,
                   {CONTROL_EVENT_PAYLOAD_COLUMNS}
            FROM events ORDER BY event_id DESC LIMIT 200
            """
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

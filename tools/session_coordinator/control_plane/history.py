from __future__ import annotations

import json
from collections import defaultdict

from ..database import Database


_VALIDATION_STATUSES = (
    "queued",
    "materializing",
    "running",
    "passed",
    "failed",
    "snapshot_stale",
)
_MAX_HISTORY_ROWS = 200
_MAX_EVENTS_PER_TICKET = 64
_MAX_COMMAND_ARGUMENTS = 24
_MAX_COMMAND_ARGUMENT_CHARS = 160


class ControlHistoryService:
    """Projects bounded operator history without copying durable evidence logs."""

    def __init__(self, database: Database):
        self.database = database

    def validation(self, *, limit: int = 50) -> dict[str, object]:
        bounded_limit = self._limit(limit)
        with self.database.connect() as connection:
            rows = connection.execute(
                """
                SELECT ticket_id, session_id, plan_path, status,
                       source_manifest_hash, command_json, created_at, updated_at
                FROM validation_tickets
                ORDER BY updated_at DESC, ticket_id DESC
                LIMIT ?
                """,
                (bounded_limit + 1,),
            ).fetchall()
            selected = rows[:bounded_limit]
            events = self._validation_events(
                connection,
                tuple(str(row["ticket_id"]) for row in selected),
            )
            status_counts = {status: 0 for status in _VALIDATION_STATUSES}
            for row in connection.execute(
                """
                SELECT status, COUNT(*) AS count
                FROM validation_tickets
                GROUP BY status
                """
            ):
                status = str(row["status"])
                if status in status_counts:
                    status_counts[status] = int(row["count"])

        tickets = []
        for row in selected:
            ticket_id = str(row["ticket_id"])
            command, command_truncated = self._command(row["command_json"])
            ticket_events, event_count = events.get(ticket_id, ([], 0))
            tickets.append(
                {
                    "ticketId": ticket_id,
                    "sessionId": str(row["session_id"]),
                    "planPath": str(row["plan_path"]),
                    "status": str(row["status"]),
                    "sourceManifestHash": str(row["source_manifest_hash"]),
                    "command": command,
                    "commandTruncated": command_truncated,
                    "createdAt": str(row["created_at"]),
                    "updatedAt": str(row["updated_at"]),
                    "events": ticket_events,
                    "eventsTruncated": event_count > len(ticket_events),
                }
            )
        return {
            "tickets": tickets,
            "statusCounts": status_counts,
            "truncated": len(rows) > bounded_limit,
        }

    def failures(self, *, limit: int = 100) -> dict[str, object]:
        bounded_limit = self._limit(limit)
        with self.database.connect() as connection:
            status_counts = {"open": 0, "fixed": 0}
            for row in connection.execute(
                "SELECT status, COUNT(*) AS count FROM failure_nodes GROUP BY status"
            ):
                status = str(row["status"])
                if status in status_counts:
                    status_counts[status] = int(row["count"])
            fixed_reserve = (
                min(status_counts["fixed"], max(1, bounded_limit // 4))
                if bounded_limit > 1
                else 0
            )
            open_limit = min(status_counts["open"], bounded_limit - fixed_reserve)
            fixed_limit = min(status_counts["fixed"], bounded_limit - open_limit)
            if open_limit + fixed_limit < bounded_limit:
                open_limit = min(
                    status_counts["open"], bounded_limit - fixed_limit
                )
            open_rows = connection.execute(
                """
                SELECT lifecycle_key, artifact_path, status, created_at, resolved_at,
                       summary_slug, origin_plan, fixing_plan, priority
                FROM failure_nodes
                WHERE status='open'
                ORDER BY priority, node_id
                LIMIT ?
                """,
                (open_limit,),
            ).fetchall()
            fixed_rows = connection.execute(
                """
                SELECT lifecycle_key, artifact_path, status, created_at, resolved_at,
                       summary_slug, origin_plan, fixing_plan, priority
                FROM failure_nodes
                WHERE status='fixed'
                ORDER BY resolved_at DESC, created_at DESC, node_id DESC
                LIMIT ?
                """,
                (fixed_limit,),
            ).fetchall()
            rows = [*open_rows, *fixed_rows]
            events = self._failure_events(
                connection,
                tuple(str(row["lifecycle_key"]) for row in rows),
            )

        chains = []
        for row in rows:
            artifact_path = str(row["artifact_path"])
            created_at = str(row["created_at"])
            resolved_at = (
                str(row["resolved_at"]) if row["resolved_at"] is not None else None
            )
            chains.append(
                {
                    "lifecycleKey": str(row["lifecycle_key"]),
                    "summarySlug": str(row["summary_slug"]),
                    "status": str(row["status"]),
                    "priority": int(row["priority"]),
                    "originPlan": str(row["origin_plan"]),
                    "fixingPlan": str(row["fixing_plan"]),
                    "artifactPath": artifact_path,
                    "createdAt": created_at,
                    "resolvedAt": resolved_at,
                    "events": events.get(str(row["lifecycle_key"]), []),
                }
            )
        return {
            "chains": chains,
            "statusCounts": status_counts,
            "truncated": sum(status_counts.values()) > len(rows),
        }

    @staticmethod
    def _limit(limit: int) -> int:
        if isinstance(limit, bool) or not isinstance(limit, int):
            raise ValueError("history limit must be an integer")
        return min(_MAX_HISTORY_ROWS, max(1, limit))

    @staticmethod
    def _validation_events(connection, ticket_ids: tuple[str, ...]):
        if not ticket_ids:
            return {}
        placeholders = ",".join("?" for _ in ticket_ids)
        rows = connection.execute(
            f"""
            WITH ranked AS (
                SELECT event_id, ticket_id, event_type, payload_json, created_at,
                       COUNT(*) OVER (PARTITION BY ticket_id) AS event_count,
                       ROW_NUMBER() OVER (
                           PARTITION BY ticket_id ORDER BY event_id DESC
                       ) AS event_rank
                FROM validation_ticket_events
                WHERE ticket_id IN ({placeholders})
            )
            SELECT event_id, ticket_id, event_type, payload_json, created_at, event_count
            FROM ranked
            WHERE event_rank <= ?
            ORDER BY ticket_id, event_id
            """,
            (*ticket_ids, _MAX_EVENTS_PER_TICKET),
        ).fetchall()
        grouped: dict[str, list[dict[str, object]]] = defaultdict(list)
        counts: dict[str, int] = {}
        for row in rows:
            ticket_id = str(row["ticket_id"])
            grouped[ticket_id].append(ControlHistoryService._validation_event(row))
            counts[ticket_id] = int(row["event_count"])
        return {
            ticket_id: (grouped[ticket_id], counts[ticket_id])
            for ticket_id in grouped
        }

    @staticmethod
    def _failure_events(connection, lifecycle_keys: tuple[str, ...]):
        if not lifecycle_keys:
            return {}
        placeholders = ",".join("?" for _ in lifecycle_keys)
        rows = connection.execute(
            f"""
            SELECT lifecycle_key, event_kind, artifact_path, created_at
            FROM failure_lifecycle_events
            WHERE lifecycle_key IN ({placeholders})
            ORDER BY lifecycle_key, event_id
            """,
            lifecycle_keys,
        ).fetchall()
        grouped: dict[str, list[dict[str, object]]] = defaultdict(list)
        for row in rows:
            grouped[str(row["lifecycle_key"])].append(
                {
                    "kind": str(row["event_kind"]),
                    "createdAt": str(row["created_at"]),
                    "artifactPath": str(row["artifact_path"]),
                }
            )
        return dict(grouped)

    @staticmethod
    def _validation_event(row) -> dict[str, object]:
        try:
            payload = json.loads(str(row["payload_json"]))
        except (TypeError, ValueError, json.JSONDecodeError):
            payload = {}
        if not isinstance(payload, dict):
            payload = {}
        evidence = payload.get("evidence")
        if not isinstance(evidence, dict):
            evidence = {}
        event_type = str(row["event_type"])
        from_status = ControlHistoryService._optional_text(payload.get("from"))
        to_status = ControlHistoryService._optional_text(payload.get("to"))
        if event_type == "validation.ticket_submitted":
            to_status = "queued"
        phase = ControlHistoryService._optional_text(
            evidence.get("phase", payload.get("phase"))
        )
        error_code = ControlHistoryService._optional_text(
            evidence.get("errorCode", payload.get("errorCode"))
        )
        job_id = ControlHistoryService._optional_text(
            evidence.get("jobId", payload.get("jobId"))
        )
        run_id = ControlHistoryService._optional_text(
            evidence.get("runId", payload.get("runId"))
        )
        exit_code = evidence.get("exitCode", payload.get("exitCode"))
        if isinstance(exit_code, bool) or not isinstance(exit_code, int):
            exit_code = None
        return {
            "eventId": int(row["event_id"]),
            "type": event_type,
            "createdAt": str(row["created_at"]),
            "fromStatus": from_status,
            "toStatus": to_status,
            "phase": phase,
            "errorCode": error_code,
            "jobId": job_id,
            "runId": run_id,
            "exitCode": exit_code,
        }

    @staticmethod
    def _command(raw: object) -> tuple[list[str], bool]:
        try:
            parsed = json.loads(str(raw))
        except (TypeError, ValueError, json.JSONDecodeError):
            return [], True
        if not isinstance(parsed, list):
            return [], True
        command: list[str] = []
        truncated = len(parsed) > _MAX_COMMAND_ARGUMENTS
        for value in parsed[:_MAX_COMMAND_ARGUMENTS]:
            text = str(value)
            if len(text) > _MAX_COMMAND_ARGUMENT_CHARS:
                text = text[: _MAX_COMMAND_ARGUMENT_CHARS - 3] + "..."
                truncated = True
            command.append(text)
        return command, truncated

    @staticmethod
    def _optional_text(value: object) -> str | None:
        if not isinstance(value, str) or not value:
            return None
        return value[:160]

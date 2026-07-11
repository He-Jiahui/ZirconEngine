from __future__ import annotations

import json
import uuid
from collections.abc import Iterable

from ..database import Database
from ..models import (
    SessionRecord,
    SessionStatus,
    WorkflowNodeKind,
    WorkflowNodeState,
    WorkflowState,
    parse_utc,
    utc_text,
)
from .models import WorkflowAttemptRecord, WorkflowNodeRecord, WorkflowRunRecord


_SESSION_STATE_MAP: dict[SessionStatus, tuple[WorkflowState, WorkflowNodeState]] = {
    SessionStatus.REGISTERED: (WorkflowState.REGISTERED, WorkflowNodeState.PENDING),
    SessionStatus.ACTIVE: (WorkflowState.ACTIVE, WorkflowNodeState.RUNNING),
    SessionStatus.WAITING_LEASE: (WorkflowState.WAITING_LEASE, WorkflowNodeState.WAITING_EXTERNAL),
    SessionStatus.RESOLVING_FAILURE: (
        WorkflowState.RESOLVING_FAILURE,
        WorkflowNodeState.WAITING_EXTERNAL,
    ),
    SessionStatus.WAITING_VALIDATION: (
        WorkflowState.WAITING_VALIDATION,
        WorkflowNodeState.WAITING_EXTERNAL,
    ),
    SessionStatus.FINALIZING: (WorkflowState.FINALIZING, WorkflowNodeState.RUNNING),
    SessionStatus.COMPLETED: (WorkflowState.SUCCEEDED, WorkflowNodeState.SUCCEEDED),
    SessionStatus.STALE: (WorkflowState.STALE, WorkflowNodeState.WAITING_EXTERNAL),
    SessionStatus.ARCHIVED: (WorkflowState.ARCHIVED, WorkflowNodeState.SUCCEEDED),
    SessionStatus.CANCELLED: (WorkflowState.CANCELLED, WorkflowNodeState.CANCELLED),
}


class WorkflowStore:
    """Owns durable workflow read models and immutable node attempts."""

    def __init__(self, database: Database):
        self.database = database

    def ensure_session_run(
        self, session_id: str, plan_path: str | None
    ) -> WorkflowRunRecord:
        with self.database.transaction() as connection:
            return self.ensure_session_run_in_connection(
                connection, session_id, plan_path
            )

    def ensure_session_run_in_connection(
        self, connection, session_id: str, plan_path: str | None
    ) -> WorkflowRunRecord:
        workflow_key = plan_path or f"session:{session_id}"
        now = utc_text()
        row = connection.execute(
            "SELECT * FROM workflow_runs WHERE session_id = ? AND workflow_key = ?",
            (session_id, workflow_key),
        ).fetchone()
        if row is None:
            run_id = uuid.uuid4().hex
            connection.execute(
                """
                INSERT INTO workflow_runs(
                    run_id, session_id, workflow_key, plan_path, state,
                    created_at, updated_at
                ) VALUES (?, ?, ?, ?, 'registered', ?, ?)
                """,
                (run_id, session_id, workflow_key, plan_path, now, now),
            )
            connection.execute(
                """
                INSERT INTO workflow_nodes(
                    node_id, run_id, node_key, kind, title, stage, state,
                    owner_session_id, created_at, updated_at
                ) VALUES (?, ?, 'goal', 'goal', 'Session Goal', 'goal',
                          'pending', ?, ?, ?)
                """,
                (f"{run_id}:goal", run_id, session_id, now, now),
            )
            row = connection.execute(
                "SELECT * FROM workflow_runs WHERE run_id = ?", (run_id,)
            ).fetchone()
        return self._run_from_row(row)

    def synchronize_session(self, session: SessionRecord) -> WorkflowRunRecord:
        with self.database.transaction() as connection:
            return self.synchronize_session_in_connection(connection, session)

    def synchronize_session_in_connection(
        self, connection, session: SessionRecord
    ) -> WorkflowRunRecord:
        run = self.ensure_session_run_in_connection(
            connection, session.session_id, session.plan_path
        )
        workflow_state, node_state = _SESSION_STATE_MAP[session.status]
        now = utc_text()
        completed_at = now if workflow_state in {
            WorkflowState.SUCCEEDED,
            WorkflowState.FAILED,
            WorkflowState.CANCELLED,
            WorkflowState.ARCHIVED,
        } else None
        connection.execute(
                """
                UPDATE workflow_runs
                SET state = ?, status_reason = ?, updated_at = ?,
                    completed_at = COALESCE(?, completed_at)
                WHERE run_id = ?
                """,
                (workflow_state.value, session.status_reason, now, completed_at, run.run_id),
            )
        connection.execute(
                """
                UPDATE workflow_nodes
                SET state = CASE WHEN EXISTS (
                        SELECT 1 FROM workflow_attempts attempt
                        WHERE attempt.node_id = workflow_nodes.node_id
                          AND attempt.accepted = 1
                    ) THEN state ELSE ? END,
                    status_reason = ?, updated_at = ?
                WHERE run_id = ? AND node_key = 'goal'
                """,
                (node_state.value, session.status_reason, now, run.run_id),
            )
        row = connection.execute(
            "SELECT * FROM workflow_runs WHERE run_id = ?", (run.run_id,)
        ).fetchone()
        return self._run_from_row(row)

    def synchronize_sessions(self, sessions: Iterable[SessionRecord]) -> tuple[WorkflowRunRecord, ...]:
        return tuple(self.synchronize_session(session) for session in sessions)

    def nodes(self, run_id: str) -> tuple[WorkflowNodeRecord, ...]:
        with self.database.connect() as connection:
            rows = connection.execute(
                "SELECT * FROM workflow_nodes WHERE run_id = ? ORDER BY stage, node_key",
                (run_id,),
            ).fetchall()
        return tuple(self._node_from_row(row) for row in rows)

    def append_attempt(
        self,
        node_id: str,
        state: WorkflowNodeState,
        evidence: dict[str, object],
        *,
        accepted: bool = True,
    ) -> WorkflowAttemptRecord:
        now = utc_text()
        attempt_id = uuid.uuid4().hex
        with self.database.transaction() as connection:
            row = connection.execute(
                """
                SELECT node.run_id, COALESCE(MAX(attempt.attempt_number), 0)
                FROM workflow_nodes node
                LEFT JOIN workflow_attempts attempt ON attempt.node_id = node.node_id
                WHERE node.node_id = ?
                GROUP BY node.run_id
                """,
                (node_id,),
            ).fetchone()
            if row is None:
                raise KeyError(node_id)
            run_id = row["run_id"]
            attempt_number = int(row[1]) + 1
            completed_at = now if state not in {
                WorkflowNodeState.PENDING,
                WorkflowNodeState.READY,
                WorkflowNodeState.RUNNING,
                WorkflowNodeState.WAITING_EXTERNAL,
            } else None
            connection.execute(
                """
                INSERT INTO workflow_attempts(
                    attempt_id, run_id, node_id, attempt_number, state, accepted,
                    evidence_json, started_at, completed_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    attempt_id,
                    run_id,
                    node_id,
                    attempt_number,
                    state.value,
                    int(accepted),
                    json.dumps(evidence, sort_keys=True),
                    now,
                    completed_at,
                ),
            )
            connection.execute(
                """
                UPDATE workflow_nodes
                SET state = CASE WHEN ? = 1 THEN ? ELSE state END,
                    attempt_count = ?, updated_at = ?
                WHERE node_id = ?
                """,
                (int(accepted), state.value, attempt_number, now, node_id),
            )
            inserted = connection.execute(
                "SELECT * FROM workflow_attempts WHERE attempt_id = ?", (attempt_id,)
            ).fetchone()
        return self._attempt_from_row(inserted)

    def attempt_history(self, node_id: str) -> tuple[WorkflowAttemptRecord, ...]:
        with self.database.connect() as connection:
            rows = connection.execute(
                """
                SELECT * FROM workflow_attempts
                WHERE node_id = ? ORDER BY attempt_number
                """,
                (node_id,),
            ).fetchall()
        return tuple(self._attempt_from_row(row) for row in rows)

    def current_attempts(self, run_id: str) -> dict[str, WorkflowAttemptRecord]:
        with self.database.connect() as connection:
            rows = connection.execute(
                """
                SELECT attempt.*
                FROM workflow_attempts attempt
                JOIN workflow_nodes node ON node.node_id = attempt.node_id
                WHERE node.run_id = ? AND attempt.accepted = 1
                  AND attempt.attempt_number = (
                    SELECT MAX(latest.attempt_number)
                    FROM workflow_attempts latest
                    WHERE latest.node_id = attempt.node_id AND latest.accepted = 1
                  )
                """,
                (run_id,),
            ).fetchall()
        return {row["node_id"]: self._attempt_from_row(row) for row in rows}

    @staticmethod
    def _run_from_row(row) -> WorkflowRunRecord:
        return WorkflowRunRecord(
            run_id=row["run_id"],
            session_id=row["session_id"],
            workflow_key=row["workflow_key"],
            plan_path=row["plan_path"],
            topology_hash=row["topology_hash"],
            state=WorkflowState(row["state"]),
            status_reason=row["status_reason"],
            created_at=parse_utc(row["created_at"]),
            updated_at=parse_utc(row["updated_at"]),
            completed_at=parse_utc(row["completed_at"]) if row["completed_at"] else None,
        )

    @staticmethod
    def _node_from_row(row) -> WorkflowNodeRecord:
        return WorkflowNodeRecord(
            node_id=row["node_id"],
            run_id=row["run_id"],
            node_key=row["node_key"],
            kind=WorkflowNodeKind(row["kind"]),
            title=row["title"],
            stage=row["stage"],
            state=WorkflowNodeState(row["state"]),
            owner_session_id=row["owner_session_id"],
            status_reason=row["status_reason"],
            attempt_count=int(row["attempt_count"]),
            created_at=parse_utc(row["created_at"]),
            updated_at=parse_utc(row["updated_at"]),
        )

    @staticmethod
    def _attempt_from_row(row) -> WorkflowAttemptRecord:
        return WorkflowAttemptRecord(
            attempt_id=row["attempt_id"],
            node_id=row["node_id"],
            attempt_number=int(row["attempt_number"]),
            state=WorkflowNodeState(row["state"]),
            accepted=bool(row["accepted"]),
            evidence=json.loads(row["evidence_json"]),
            started_at=parse_utc(row["started_at"]),
            completed_at=parse_utc(row["completed_at"]) if row["completed_at"] else None,
        )

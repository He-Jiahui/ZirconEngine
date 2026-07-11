from __future__ import annotations

import json
import subprocess
from collections.abc import Callable
from datetime import timedelta
from pathlib import Path
from sqlite3 import Connection, Row

from .database import Database
from .models import (
    ALLOWED_STATUS_TRANSITIONS,
    CoordinatorError,
    InvalidStatusTransition,
    SessionRecord,
    SessionStatus,
    parse_utc,
    utc_now,
    utc_text,
)


class SessionService:
    def __init__(
        self,
        database: Database,
        repo_root: str | Path,
        *,
        session_change_hook: Callable[[Connection, SessionRecord], None] | None = None,
    ):
        self.database = database
        self.repo_root = Path(repo_root).resolve()
        self.session_change_hook = session_change_hook

    def register(
        self,
        *,
        session_id: str,
        display_name: str | None = None,
        plan_path: str | None = None,
        write_scope: list[str] | tuple[str, ...] | None = None,
    ) -> SessionRecord:
        if not session_id.strip():
            raise ValueError("session_id cannot be empty")
        normalized_scope = tuple(dict.fromkeys(write_scope or ()))
        now = utc_text()
        base_head = self._head_commit()
        with self.database.transaction() as connection:
            existing = connection.execute(
                "SELECT * FROM sessions WHERE session_id = ?", (session_id,)
            ).fetchone()
            if existing is None:
                epoch_row = connection.execute(
                    "SELECT MAX(epoch_id) FROM baseline_epochs"
                ).fetchone()
                baseline_epoch = epoch_row[0] if epoch_row else None
                connection.execute(
                    """
                    INSERT INTO sessions(
                        session_id, display_name, plan_path, status, base_head,
                        baseline_epoch, write_scope_json, created_at, updated_at,
                        last_heartbeat_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (
                        session_id,
                        display_name,
                        plan_path,
                        SessionStatus.REGISTERED.value,
                        base_head,
                        baseline_epoch,
                        json.dumps(normalized_scope),
                        now,
                        now,
                        now,
                    ),
                )
                self._event(connection, session_id, "session.registered", {"base_head": base_head})
            else:
                connection.execute(
                    """
                    UPDATE sessions
                    SET display_name = COALESCE(?, display_name),
                        plan_path = COALESCE(?, plan_path),
                        write_scope_json = CASE WHEN ? <> '[]' THEN ? ELSE write_scope_json END,
                        updated_at = ?, last_heartbeat_at = ?
                    WHERE session_id = ?
                    """,
                    (
                        display_name,
                        plan_path,
                        json.dumps(normalized_scope),
                        json.dumps(normalized_scope),
                        now,
                        now,
                        session_id,
                    ),
                )
            session = self._changed_session(connection, session_id)
        return session

    def get(self, session_id: str) -> SessionRecord:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM sessions WHERE session_id = ?", (session_id,)
            ).fetchone()
        if row is None:
            raise CoordinatorError("session_not_found", f"Unknown Session {session_id}")
        return self._from_row(row)

    def list(self, *, include_archived: bool = False) -> list[SessionRecord]:
        query = "SELECT * FROM sessions"
        parameters: tuple[object, ...] = ()
        if not include_archived:
            query += " WHERE status <> ?"
            parameters = (SessionStatus.ARCHIVED.value,)
        query += " ORDER BY updated_at DESC, session_id"
        with self.database.connect() as connection:
            rows = connection.execute(query, parameters).fetchall()
        return [self._from_row(row) for row in rows]

    def heartbeat(self, session_id: str) -> SessionRecord:
        now = utc_text()
        with self.database.transaction() as connection:
            cursor = connection.execute(
                "UPDATE sessions SET last_heartbeat_at = ?, updated_at = ? WHERE session_id = ?",
                (now, now, session_id),
            )
            if cursor.rowcount != 1:
                raise CoordinatorError("session_not_found", f"Unknown Session {session_id}")
            session = self._changed_session(connection, session_id)
        return session

    def set_status(
        self,
        session_id: str,
        status: SessionStatus,
        *,
        reason: str | None = None,
    ) -> SessionRecord:
        if not isinstance(status, SessionStatus):
            raise ValueError("status must be a SessionStatus enum value")
        now = utc_text()
        with self.database.transaction() as connection:
            row = connection.execute(
                "SELECT status FROM sessions WHERE session_id = ?", (session_id,)
            ).fetchone()
            if row is None:
                raise CoordinatorError("session_not_found", f"Unknown Session {session_id}")
            current = SessionStatus(row["status"])
            if status != current and status not in ALLOWED_STATUS_TRANSITIONS[current]:
                raise InvalidStatusTransition(current, status)
            completed_at = now if status is SessionStatus.COMPLETED else None
            archived_at = now if status is SessionStatus.ARCHIVED else None
            connection.execute(
                """
                UPDATE sessions
                SET status = ?, status_reason = ?, updated_at = ?, last_heartbeat_at = ?,
                    completed_at = COALESCE(?, completed_at),
                    archived_at = COALESCE(?, archived_at)
                WHERE session_id = ?
                """,
                (status.value, reason, now, now, completed_at, archived_at, session_id),
            )
            self._event(
                connection,
                session_id,
                "session.status_changed",
                {"from": current.value, "to": status.value, "reason": reason},
            )
            session = self._changed_session(connection, session_id)
        return session

    def mark_stale(
        self, *, older_than_seconds: int, excluded_session_ids: set[str] | None = None
    ) -> list[str]:
        cutoff = utc_text(utc_now() - timedelta(seconds=older_than_seconds))
        excluded = excluded_session_ids or set()
        eligible_statuses = (
            SessionStatus.REGISTERED.value,
            SessionStatus.ACTIVE.value,
            SessionStatus.WAITING_LEASE.value,
            SessionStatus.RESOLVING_FAILURE.value,
            SessionStatus.WAITING_VALIDATION.value,
        )
        now = utc_text()
        marked: list[str] = []
        with self.database.transaction() as connection:
            rows = connection.execute(
                """
                SELECT session_id, status, last_heartbeat_at
                FROM sessions
                WHERE status IN (?, ?, ?, ?, ?)
                  AND last_heartbeat_at < ?
                ORDER BY session_id
                """,
                (*eligible_statuses, cutoff),
            ).fetchall()
            for row in rows:
                session_id = row["session_id"]
                if session_id in excluded:
                    continue
                cursor = connection.execute(
                    """
                    UPDATE sessions
                    SET status = 'stale', status_reason = ?, updated_at = ?
                    WHERE session_id = ?
                      AND status = ?
                      AND last_heartbeat_at = ?
                      AND last_heartbeat_at < ?
                    """,
                    (
                        "heartbeat expired",
                        now,
                        session_id,
                        row["status"],
                        row["last_heartbeat_at"],
                        cutoff,
                    ),
                )
                if cursor.rowcount != 1:
                    continue
                self._event(
                    connection,
                    session_id,
                    "session.status_changed",
                    {
                        "from": row["status"],
                        "to": SessionStatus.STALE.value,
                        "reason": "heartbeat expired",
                    },
                )
                self._changed_session(connection, session_id)
                marked.append(session_id)
        return marked

    def archive_stale(
        self,
        *,
        older_than_seconds: int = 86400,
        excluded_session_ids: set[str] | None = None,
    ) -> list[str]:
        cutoff = utc_text(utc_now() - timedelta(seconds=older_than_seconds))
        now = utc_text()
        archived: list[str] = []
        with self.database.transaction() as connection:
            rows = connection.execute(
                """
                SELECT session_id FROM sessions
                WHERE status = 'stale' AND updated_at < ?
                  AND NOT EXISTS (
                      SELECT 1 FROM leases WHERE leases.session_id = sessions.session_id
                  )
                  AND NOT EXISTS (
                      SELECT 1 FROM patches
                      WHERE patches.session_id = sessions.session_id
                        AND patches.status IN ('queued', 'applying', 'needs_rebase')
                  )
                  AND NOT EXISTS (
                      SELECT 1 FROM failure_nodes
                      WHERE failure_nodes.fixing_plan = sessions.plan_path
                        AND failure_nodes.kind = 'failure'
                        AND failure_nodes.status = 'open'
                  )
                ORDER BY session_id
                """,
                (cutoff,),
            ).fetchall()
            for row in rows:
                session_id = row["session_id"]
                if session_id in (excluded_session_ids or set()):
                    continue
                connection.execute(
                    """
                    UPDATE sessions
                    SET status = 'archived', status_reason = ?, updated_at = ?, archived_at = ?
                    WHERE session_id = ? AND status = 'stale'
                    """,
                    ("stale retention elapsed", now, now, session_id),
                )
                self._event(
                    connection,
                    session_id,
                    "session.status_changed",
                    {
                        "from": SessionStatus.STALE.value,
                        "to": SessionStatus.ARCHIVED.value,
                        "reason": "stale retention elapsed",
                    },
                )
                self._changed_session(connection, session_id)
                archived.append(session_id)
        return archived

    def _changed_session(
        self, connection: Connection, session_id: str
    ) -> SessionRecord:
        row = connection.execute(
            "SELECT * FROM sessions WHERE session_id = ?", (session_id,)
        ).fetchone()
        if row is None:
            raise CoordinatorError("session_not_found", f"Unknown Session {session_id}")
        session = self._from_row(row)
        if self.session_change_hook is not None:
            self.session_change_hook(connection, session)
        return session

    def _head_commit(self) -> str:
        result = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
            text=True,
        )
        return result.stdout.strip()

    @staticmethod
    def _event(connection, session_id: str, event_type: str, payload: dict[str, object]) -> None:
        connection.execute(
            "INSERT INTO events(session_id, event_type, payload_json, created_at) VALUES (?, ?, ?, ?)",
            (session_id, event_type, json.dumps(payload, sort_keys=True), utc_text()),
        )

    @staticmethod
    def _from_row(row: Row) -> SessionRecord:
        return SessionRecord(
            session_id=row["session_id"],
            status=SessionStatus(row["status"]),
            display_name=row["display_name"],
            plan_path=row["plan_path"],
            write_scope=tuple(json.loads(row["write_scope_json"])),
            status_reason=row["status_reason"],
            base_head=row["base_head"],
            baseline_epoch=row["baseline_epoch"],
            created_at=parse_utc(row["created_at"]),
            updated_at=parse_utc(row["updated_at"]),
            last_heartbeat_at=parse_utc(row["last_heartbeat_at"]),
        )

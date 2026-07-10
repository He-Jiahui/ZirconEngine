from __future__ import annotations

import json
import subprocess
from datetime import timedelta
from pathlib import Path
from sqlite3 import Row

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
    def __init__(self, database: Database, repo_root: str | Path):
        self.database = database
        self.repo_root = Path(repo_root).resolve()

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
        return self.get(session_id)

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
        return self.get(session_id)

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
        return self.get(session_id)

    def mark_stale(self, *, older_than_seconds: int) -> list[str]:
        cutoff = utc_now() - timedelta(seconds=older_than_seconds)
        marked: list[str] = []
        for session in self.list():
            if session.status in {SessionStatus.ACTIVE, SessionStatus.WAITING_LEASE}:
                if session.last_heartbeat_at < cutoff:
                    self.set_status(session.session_id, SessionStatus.STALE, reason="heartbeat expired")
                    marked.append(session.session_id)
        return marked

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

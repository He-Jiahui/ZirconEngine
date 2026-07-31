from __future__ import annotations

import json
import subprocess
from collections.abc import Callable
from datetime import timedelta
from pathlib import Path
from sqlite3 import Connection, Row

from .cargo_reservations import expire_unstarted_cpu_reservations_for_session
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
from .plan_wip import PlanWipGate


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
        self.plan_wip = PlanWipGate(self.repo_root)

    def register(
        self,
        *,
        session_id: str,
        display_name: str | None = None,
        plan_path: str | None = None,
        write_scope: list[str] | tuple[str, ...] | None = None,
        session_role: str | None = None,
        parent_session_id: str | None = None,
        requested_status: SessionStatus | None = None,
        status_reason: str | None = None,
    ) -> SessionRecord:
        if not session_id.strip():
            raise ValueError("session_id cannot be empty")
        if requested_status is not None and not isinstance(requested_status, SessionStatus):
            raise ValueError("requested_status must be a SessionStatus enum value")
        normalized_scope = tuple(dict.fromkeys(write_scope or ()))
        now = utc_text()
        base_head = self._head_commit()
        with self.database.transaction() as connection:
            existing = connection.execute(
                "SELECT * FROM sessions WHERE session_id = ?", (session_id,)
            ).fetchone()
            if existing is None:
                target_status = requested_status or SessionStatus.REGISTERED
                admission = self.plan_wip.admit_in_connection(
                    connection,
                    session_id=session_id,
                    plan_path=plan_path,
                    session_role=session_role or "primary",
                    parent_session_id=parent_session_id,
                    write_scope=normalized_scope,
                    existing=None,
                )
                epoch_row = connection.execute(
                    "SELECT MAX(epoch_id) FROM baseline_epochs"
                ).fetchone()
                baseline_epoch = epoch_row[0] if epoch_row else None
                cursor = connection.execute(
                    """
                    INSERT INTO sessions(
                        session_id, display_name, plan_path, status, base_head,
                        baseline_epoch, plan_family_key, session_role, parent_session_id,
                        write_scope_json, created_at, updated_at,
                        last_heartbeat_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (
                        session_id,
                        display_name,
                        plan_path,
                        target_status.value,
                        base_head,
                        baseline_epoch,
                        admission.plan_family_key,
                        admission.session_role,
                        admission.parent_session_id,
                        json.dumps(normalized_scope),
                        now,
                        now,
                        now,
                    ),
                )
                self._event(connection, session_id, "session.registered", {"base_head": base_head})
                if target_status is not SessionStatus.REGISTERED:
                    self._event(
                        connection,
                        session_id,
                        "session.status_changed",
                        {
                            "from": SessionStatus.REGISTERED.value,
                            "to": target_status.value,
                            "reason": status_reason,
                        },
                    )
            else:
                existing_scope = tuple(json.loads(existing["write_scope_json"]))
                if normalized_scope and normalized_scope != existing_scope:
                    raise CoordinatorError(
                        "session_write_scope_immutable",
                        "An existing Session write scope is immutable; use an explicit "
                        "audited scope-transfer operation",
                    )
                current_status = SessionStatus(existing["status"])
                target_status = requested_status or current_status
                if (
                    target_status is not current_status
                    and target_status not in ALLOWED_STATUS_TRANSITIONS[current_status]
                ):
                    raise InvalidStatusTransition(current_status, target_status)
                admission = self.plan_wip.admit_in_connection(
                    connection,
                    session_id=session_id,
                    plan_path=plan_path or existing["plan_path"],
                    session_role=session_role or str(existing["session_role"]),
                    parent_session_id=(
                        parent_session_id
                        if parent_session_id is not None
                        else existing["parent_session_id"]
                    ),
                    write_scope=existing_scope,
                    existing=existing,
                )
                connection.execute(
                    """
                    UPDATE sessions
                    SET display_name = COALESCE(?, display_name),
                        plan_path = COALESCE(?, plan_path),
                        plan_family_key = ?,
                        session_role = ?,
                        parent_session_id = ?,
                        status = ?,
                        status_reason = ?,
                        updated_at = ?, last_heartbeat_at = ?
                    WHERE session_id = ?
                    """,
                    (
                        display_name,
                        plan_path,
                        admission.plan_family_key,
                        admission.session_role,
                        admission.parent_session_id,
                        target_status.value,
                        status_reason if target_status is not current_status else existing["status_reason"],
                        now,
                        now,
                        session_id,
                    ),
                )
                if target_status is not current_status:
                    self._event(
                        connection,
                        session_id,
                        "session.status_changed",
                        {
                            "from": current_status.value,
                            "to": target_status.value,
                            "reason": status_reason,
                        },
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
            row = connection.execute(
                "SELECT status FROM sessions WHERE session_id = ?", (session_id,)
            ).fetchone()
            if row is None:
                raise CoordinatorError("session_not_found", f"Unknown Session {session_id}")
            current = SessionStatus(row["status"])
            if current is SessionStatus.STALE:
                connection.execute(
                    """
                    UPDATE sessions
                    SET status = ?, status_reason = ?, last_heartbeat_at = ?, updated_at = ?
                    WHERE session_id = ?
                    """,
                    (
                        SessionStatus.ACTIVE.value,
                        "heartbeat resumed active work",
                        now,
                        now,
                        session_id,
                    ),
                )
                self._event(
                    connection,
                    session_id,
                    "session.status_changed",
                    {
                        "from": SessionStatus.STALE.value,
                        "to": SessionStatus.ACTIVE.value,
                        "reason": "heartbeat resumed active work",
                    },
                )
            else:
                connection.execute(
                    "UPDATE sessions SET last_heartbeat_at = ?, updated_at = ? WHERE session_id = ?",
                    (now, now, session_id),
                )
            session = self._changed_session(connection, session_id)
        return session

    def extend_write_scope_in_connection(
        self,
        connection: Connection,
        session_id: str,
        paths: tuple[str, ...],
        *,
        transfer_fingerprint: str,
    ) -> SessionRecord:
        """Append an audited exact-path scope during an ownership transfer."""
        row = connection.execute(
            "SELECT write_scope_json FROM sessions WHERE session_id=?", (session_id,)
        ).fetchone()
        if row is None:
            raise CoordinatorError("session_not_found", f"Unknown Session {session_id}")
        scope = tuple(dict.fromkeys((*json.loads(row["write_scope_json"]), *paths)))
        connection.execute(
            "UPDATE sessions SET write_scope_json=?, updated_at=? WHERE session_id=?",
            (json.dumps(scope), utc_text(), session_id),
        )
        self._event(
            connection,
            session_id,
            "session.write_scope_transferred",
            {"paths": list(paths), "transferFingerprint": transfer_fingerprint},
        )
        return self._changed_session(connection, session_id)

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
            if status is SessionStatus.STALE:
                expire_unstarted_cpu_reservations_for_session(
                    connection, session_id, completed_at=now
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
                  AND NOT EXISTS (
                      SELECT 1 FROM leases
                      WHERE leases.session_id = sessions.session_id
                        AND leases.expires_at > ?
                  )
                  AND NOT EXISTS (
                      SELECT 1 FROM patches
                      WHERE patches.session_id = sessions.session_id
                        AND patches.status IN ('queued', 'applying', 'needs_rebase')
                  )
                  AND NOT EXISTS (
                      SELECT 1 FROM cargo_jobs
                      WHERE cargo_jobs.session_id = sessions.session_id
                        AND cargo_jobs.status IN ('leased', 'running')
                  )
                  AND NOT EXISTS (
                      SELECT 1 FROM cargo_lane_reservations
                      WHERE cargo_lane_reservations.session_id = sessions.session_id
                        AND cargo_lane_reservations.status IN ('pending', 'leased', 'running')
                  )
                  AND NOT EXISTS (
                      SELECT 1 FROM validation_copies
                      WHERE validation_copies.session_id = sessions.session_id
                        AND validation_copies.status IN ('planned', 'materialized', 'running', 'cleanup_pending')
                  )
                ORDER BY session_id
                """,
                (*eligible_statuses, cutoff, now),
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
                      AND NOT EXISTS (
                          SELECT 1 FROM leases
                          WHERE leases.session_id = sessions.session_id
                            AND leases.expires_at > ?
                      )
                      AND NOT EXISTS (
                          SELECT 1 FROM patches
                          WHERE patches.session_id = sessions.session_id
                            AND patches.status IN ('queued', 'applying', 'needs_rebase')
                      )
                      AND NOT EXISTS (
                          SELECT 1 FROM cargo_jobs
                          WHERE cargo_jobs.session_id = sessions.session_id
                            AND cargo_jobs.status IN ('leased', 'running')
                      )
                      AND NOT EXISTS (
                          SELECT 1 FROM cargo_lane_reservations
                          WHERE cargo_lane_reservations.session_id = sessions.session_id
                            AND cargo_lane_reservations.status IN ('pending', 'leased', 'running')
                      )
                      AND NOT EXISTS (
                          SELECT 1 FROM validation_copies
                          WHERE validation_copies.session_id = sessions.session_id
                            AND validation_copies.status IN ('planned', 'materialized', 'running', 'cleanup_pending')
                      )
                    """,
                    (
                        "heartbeat expired",
                        now,
                        session_id,
                        row["status"],
                        row["last_heartbeat_at"],
                        cutoff,
                        now,
                    ),
                )
                if cursor.rowcount != 1:
                    continue
                expire_unstarted_cpu_reservations_for_session(
                    connection, session_id, completed_at=now
                )
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
                      SELECT 1 FROM leases
                      WHERE leases.session_id = sessions.session_id
                        AND leases.expires_at > ?
                  )
                   AND NOT EXISTS (
                       SELECT 1 FROM patches
                       WHERE patches.session_id = sessions.session_id
                         AND patches.status IN ('queued', 'applying', 'needs_rebase')
                   )
                  AND NOT EXISTS (
                      SELECT 1 FROM cargo_jobs
                      WHERE cargo_jobs.session_id = sessions.session_id
                        AND cargo_jobs.status IN ('leased', 'running')
                  )
                  AND NOT EXISTS (
                      SELECT 1 FROM cargo_lane_reservations
                      WHERE cargo_lane_reservations.session_id = sessions.session_id
                        AND cargo_lane_reservations.status IN ('pending', 'leased', 'running')
                  )
                  AND NOT EXISTS (
                      SELECT 1 FROM validation_copies
                      WHERE validation_copies.session_id = sessions.session_id
                        AND validation_copies.status IN ('planned', 'materialized', 'running', 'cleanup_pending')
                  )
                ORDER BY session_id
                """,
                (cutoff, now),
            ).fetchall()
            for row in rows:
                session_id = row["session_id"]
                if session_id in (excluded_session_ids or set()):
                    continue
                cursor = connection.execute(
                    """
                    UPDATE sessions
                    SET status = 'archived', status_reason = ?, updated_at = ?, archived_at = ?
                    WHERE session_id = ? AND status = 'stale' AND updated_at < ?
                      AND NOT EXISTS (
                          SELECT 1 FROM leases
                          WHERE leases.session_id = sessions.session_id
                            AND leases.expires_at > ?
                      )
                      AND NOT EXISTS (
                          SELECT 1 FROM patches
                          WHERE patches.session_id = sessions.session_id
                            AND patches.status IN ('queued', 'applying', 'needs_rebase')
                      )
                      AND NOT EXISTS (
                          SELECT 1 FROM cargo_jobs
                          WHERE cargo_jobs.session_id = sessions.session_id
                            AND cargo_jobs.status IN ('leased', 'running')
                      )
                      AND NOT EXISTS (
                          SELECT 1 FROM cargo_lane_reservations
                          WHERE cargo_lane_reservations.session_id = sessions.session_id
                            AND cargo_lane_reservations.status IN ('pending', 'leased', 'running')
                      )
                      AND NOT EXISTS (
                          SELECT 1 FROM validation_copies
                          WHERE validation_copies.session_id = sessions.session_id
                            AND validation_copies.status IN ('planned', 'materialized', 'running', 'cleanup_pending')
                      )
                    """,
                    ("stale retention elapsed", now, now, session_id, cutoff, now),
                ).rowcount
                if not cursor:
                    continue
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
            plan_family_key=row["plan_family_key"],
            session_role=row["session_role"],
            parent_session_id=row["parent_session_id"],
            write_scope=tuple(json.loads(row["write_scope_json"])),
            status_reason=row["status_reason"],
            base_head=row["base_head"],
            baseline_epoch=row["baseline_epoch"],
            created_at=parse_utc(row["created_at"]),
            updated_at=parse_utc(row["updated_at"]),
            last_heartbeat_at=parse_utc(row["last_heartbeat_at"]),
        )

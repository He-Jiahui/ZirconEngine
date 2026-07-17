from __future__ import annotations

import json
import uuid
from collections.abc import Callable

from ..database import Database
from ..models import utc_text
from .models import (
    CodexDiscoveryResult,
    CodexReconcileResult,
    CodexSessionState,
    CodexSourceLocation,
    CodexSyncTrigger,
)


class CodexSessionStore:
    """Transactionally project read-only Codex source metadata into SQLite."""

    def __init__(self, database: Database, *, clock: Callable[[], str] = utc_text):
        self.database = database
        self.clock = clock

    def reconcile(
        self,
        discovery: CodexDiscoveryResult,
        *,
        trigger: CodexSyncTrigger,
        duration_ms: int = 0,
    ) -> CodexReconcileResult:
        run_id = uuid.uuid4().hex
        now = self.clock()
        changed_count = 0
        unavailable_count = 0
        discovered_ids = {item.thread_id for item in discovery.sessions}
        with self.database.transaction() as connection:
            for item in discovery.sessions:
                existing = connection.execute(
                    "SELECT * FROM codex_sessions WHERE thread_id=?", (item.thread_id,)
                ).fetchone()
                binding = connection.execute(
                    "SELECT session_id FROM sessions WHERE session_id=?", (item.thread_id,)
                ).fetchone()
                bound_session_id = binding[0] if binding is not None else None
                values = (
                    item.rollout_path,
                    item.source_location.value,
                    item.state.value,
                    item.cwd,
                    item.originator,
                    item.cli_version,
                    item.thread_source,
                    item.last_event.value,
                    item.last_turn_id,
                    bound_session_id,
                    item.diagnostic_code,
                    item.first_seen_at,
                    item.last_activity_at,
                    now,
                    item.source_revision.mtime_ns,
                    item.source_revision.size,
                )
                if existing is None:
                    connection.execute(
                        """
                        INSERT INTO codex_sessions(
                            thread_id, rollout_path, source_location, state, cwd,
                            originator, cli_version, thread_source, last_event,
                            last_turn_id, bound_session_id, diagnostic_code,
                            first_seen_at, last_activity_at, last_synced_at,
                            source_mtime_ns, source_size, missing_scan_count
                        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0)
                        """,
                        (item.thread_id, *values),
                    )
                    changed_count += 1
                    self._event(
                        connection,
                        "codex.session.discovered",
                        {"state": item.state.value, "threadId": item.thread_id},
                        now,
                    )
                else:
                    visible_changed = self._visible_changed(existing, values)
                    metadata_changed = self._metadata_changed(existing, values) or int(
                        existing["missing_scan_count"]
                    ) != 0
                    if visible_changed or metadata_changed:
                        connection.execute(
                            """
                            UPDATE codex_sessions SET
                                rollout_path=?, source_location=?, state=?, cwd=?,
                                originator=?, cli_version=?, thread_source=?, last_event=?,
                                last_turn_id=?, bound_session_id=?, diagnostic_code=?,
                                first_seen_at=?, last_activity_at=?, last_synced_at=?,
                                source_mtime_ns=?, source_size=?, missing_scan_count=0
                            WHERE thread_id=?
                            """,
                            (*values, item.thread_id),
                        )
                        if visible_changed:
                            changed_count += 1
                            event_type = self._change_event(
                                existing, item.source_location, item.state
                            )
                            self._event(
                                connection,
                                event_type,
                                {"state": item.state.value, "threadId": item.thread_id},
                                now,
                            )

            if discovery.membership_complete:
                rows = connection.execute(
                    "SELECT thread_id, missing_scan_count FROM codex_sessions"
                ).fetchall()
                for row in rows:
                    thread_id = str(row["thread_id"])
                    if thread_id in discovered_ids:
                        continue
                    previous_missing_count = int(row["missing_scan_count"])
                    if previous_missing_count >= 2:
                        continue
                    missing_count = previous_missing_count + 1
                    if missing_count < 2:
                        connection.execute(
                            "UPDATE codex_sessions SET missing_scan_count=?, last_synced_at=? "
                            "WHERE thread_id=?",
                            (missing_count, now, thread_id),
                        )
                        continue
                    updated = connection.execute(
                        """
                        UPDATE codex_sessions
                        SET source_location='missing', state='unavailable',
                            missing_scan_count=?, last_synced_at=?
                        WHERE thread_id=? AND state<>'unavailable'
                        """,
                        (missing_count, now, thread_id),
                    )
                    if updated.rowcount:
                        changed_count += 1
                        unavailable_count += 1
                        self._event(
                            connection,
                            "codex.session.unavailable",
                            {"state": "unavailable", "threadId": thread_id},
                            now,
                        )
                    else:
                        connection.execute(
                            "UPDATE codex_sessions SET missing_scan_count=?, last_synced_at=? "
                            "WHERE thread_id=?",
                            (missing_count, now, thread_id),
                        )

            status = "partial" if not discovery.membership_complete else "succeeded"
            connection.execute(
                """
                INSERT INTO codex_sync_runs(
                    run_id, trigger_kind, status, scanned_count, changed_count,
                    diagnostic_count, unavailable_count, duration_ms,
                    source_revision, error_code, created_at, completed_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?)
                """,
                (
                    run_id,
                    trigger.value,
                    status,
                    discovery.scanned_count,
                    changed_count,
                    len(discovery.diagnostics),
                    unavailable_count,
                    max(0, duration_ms),
                    discovery.source_revision,
                    now,
                    now,
                ),
            )
            # Periodic source metadata refreshes are telemetry, not operator work.
            # Keep their compact run row for diagnostics, but reserve the live audit
            # stream for a visible lifecycle change, a diagnostic, or an explicit sync.
            if (
                trigger is not CodexSyncTrigger.PERIODIC
                or changed_count
                or discovery.diagnostics
                or unavailable_count
            ):
                self._event(
                    connection,
                    "codex.sync.completed",
                    {
                        "changedCount": changed_count,
                        "diagnosticCount": len(discovery.diagnostics),
                        "runId": run_id,
                        "scannedCount": discovery.scanned_count,
                        "status": status,
                    },
                    now,
                )
        return CodexReconcileResult(
            run_id=run_id,
            scanned_count=discovery.scanned_count,
            changed_count=changed_count,
            diagnostic_count=len(discovery.diagnostics),
            unavailable_count=unavailable_count,
        )

    @staticmethod
    def _visible_changed(existing, values: tuple[object, ...]) -> bool:
        columns = (
            "rollout_path",
            "source_location",
            "state",
            "cwd",
            "originator",
            "cli_version",
            "thread_source",
            "last_event",
            "last_turn_id",
            "bound_session_id",
            "diagnostic_code",
            "first_seen_at",
        )
        return any(
            existing[column] != value
            for column, value in zip(columns, values, strict=False)
        )

    @staticmethod
    def _metadata_changed(existing, values: tuple[object, ...]) -> bool:
        metadata = {
            "last_activity_at": values[12],
            "source_mtime_ns": values[14],
            "source_size": values[15],
        }
        return any(existing[column] != value for column, value in metadata.items())

    @staticmethod
    def _change_event(existing, location: CodexSourceLocation, state: CodexSessionState) -> str:
        if existing["source_location"] != location.value and location is CodexSourceLocation.ARCHIVED:
            return "codex.session.archived"
        if existing["state"] != state.value:
            return "codex.session.state_changed"
        return "codex.session.updated"

    @staticmethod
    def _event(connection, event_type: str, payload: dict[str, object], created_at: str) -> None:
        connection.execute(
            "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
            (event_type, json.dumps(payload, sort_keys=True, separators=(",", ":")), created_at),
        )

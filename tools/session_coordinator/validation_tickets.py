"""Durable, immutable validation requests that never block a business Session."""

from __future__ import annotations

import hashlib
import json
import re
from dataclasses import dataclass
from typing import Mapping
from uuid import uuid4

from .database import Database
from .models import CoordinatorError, utc_text


_SAFE_PATH = re.compile(r"^(?!/)(?![A-Za-z]:)[^\\]+$")
_SHA256 = re.compile(r"^[0-9a-f]{64}$")
_NONTERMINAL = frozenset({"queued", "materializing", "running"})
_TERMINAL = frozenset({"passed", "failed", "snapshot_stale"})
_TRANSITIONS = {
    "queued": frozenset({"materializing", "running", "passed", "failed", "snapshot_stale"}),
    "materializing": frozenset({"running", "passed", "failed", "snapshot_stale"}),
    "running": frozenset({"passed", "failed", "snapshot_stale"}),
}


@dataclass(frozen=True, slots=True)
class ValidationTicket:
    ticket_id: str
    session_id: str
    plan_path: str
    status: str
    source_manifest_hash: str
    source_manifest: Mapping[str, str]
    command: tuple[str, ...]
    toolchain: Mapping[str, object]
    coverage: Mapping[str, object]


@dataclass(frozen=True, slots=True)
class ValidationTicketReceipt:
    ticket: ValidationTicket
    request_id: str
    reused: bool


class ValidationTicketService:
    """Persist validation work separately from business Session lifecycle.

    Coalescing is intentionally limited to exact sealed inputs.  A later
    worktree edit produces a different source manifest and therefore a new
    ticket rather than silently changing the work another caller submitted.
    """

    def __init__(self, database: Database):
        self.database = database

    def submit(
        self,
        *,
        session_id: str,
        request_id: str,
        source_manifest: Mapping[str, str],
        command: tuple[str, ...] | list[str],
        toolchain: Mapping[str, object],
        coverage: Mapping[str, object],
    ) -> ValidationTicketReceipt:
        normalized_session = self._require_text("session_id", session_id)
        normalized_request = self._require_text("request_id", request_id)
        manifest = self._manifest(source_manifest)
        normalized_command = self._command(command)
        normalized_toolchain = self._mapping("toolchain", toolchain)
        normalized_coverage = self._mapping("coverage", coverage)
        manifest_json = self._canonical(manifest)
        command_json = self._canonical(normalized_command)
        toolchain_json = self._canonical(normalized_toolchain)
        coverage_json = self._canonical(normalized_coverage)
        manifest_hash = hashlib.sha256(manifest_json.encode("utf-8")).hexdigest()
        dedupe_key = hashlib.sha256(
            "\n".join((manifest_hash, command_json, toolchain_json, coverage_json)).encode("utf-8")
        ).hexdigest()
        now = utc_text()

        with self.database.transaction() as connection:
            existing_request = connection.execute(
                """
                SELECT ticket_id FROM validation_ticket_requests WHERE request_id=?
                """,
                (normalized_request,),
            ).fetchone()
            if existing_request is not None:
                ticket = self._get_in_connection(connection, str(existing_request["ticket_id"]))
                return ValidationTicketReceipt(ticket, normalized_request, reused=False)

            owner = connection.execute(
                "SELECT plan_path FROM sessions WHERE session_id=?", (normalized_session,)
            ).fetchone()
            if owner is None:
                raise CoordinatorError("session_not_found", f"Unknown Session {normalized_session}")
            plan_path = str(owner["plan_path"] or "")
            if not plan_path:
                raise CoordinatorError(
                    "validation_ticket_plan_missing",
                    "Validation ticket owner must be registered to a numbered Plan",
                )

            reusable = connection.execute(
                """
                SELECT ticket_id FROM validation_tickets
                WHERE dedupe_key=? AND status IN ('queued', 'materializing', 'running')
                ORDER BY created_at, ticket_id LIMIT 1
                """,
                (dedupe_key,),
            ).fetchone()
            if reusable is not None:
                ticket_id = str(reusable["ticket_id"])
                reused = True
            else:
                ticket_id = uuid4().hex
                reused = False
                connection.execute(
                    """
                    INSERT INTO validation_tickets(
                        ticket_id, session_id, plan_path, status, dedupe_key,
                        source_manifest_hash, source_manifest_json, command_json,
                        toolchain_json, coverage_json, created_at, updated_at
                    ) VALUES (?, ?, ?, 'queued', ?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (
                        ticket_id,
                        normalized_session,
                        plan_path,
                        dedupe_key,
                        manifest_hash,
                        manifest_json,
                        command_json,
                        toolchain_json,
                        coverage_json,
                        now,
                        now,
                    ),
                )
                self._event(
                    connection,
                    ticket_id,
                    "validation.ticket_submitted",
                    {"sessionId": normalized_session, "sourceManifestHash": manifest_hash},
                    now,
                )
            connection.execute(
                """
                INSERT INTO validation_ticket_requests(request_id, ticket_id, session_id, created_at)
                VALUES (?, ?, ?, ?)
                """,
                (normalized_request, ticket_id, normalized_session, now),
            )
            return ValidationTicketReceipt(
                self._get_in_connection(connection, ticket_id), normalized_request, reused
            )

    def transition(self, ticket_id: str, status: str, *, evidence: Mapping[str, object] | None = None) -> ValidationTicket:
        normalized_ticket = self._require_text("ticket_id", ticket_id)
        if status not in _NONTERMINAL | _TERMINAL:
            raise CoordinatorError("validation_ticket_status_invalid", f"Unsupported ticket status: {status}")
        now = utc_text()
        with self.database.transaction() as connection:
            ticket = self._get_in_connection(connection, normalized_ticket)
            if ticket.status == status:
                return ticket
            if status not in _TRANSITIONS.get(ticket.status, frozenset()):
                raise CoordinatorError(
                    "validation_ticket_transition_invalid",
                    f"Cannot transition validation ticket from {ticket.status} to {status}",
                )
            connection.execute(
                "UPDATE validation_tickets SET status=?, updated_at=? WHERE ticket_id=?",
                (status, now, normalized_ticket),
            )
            self._event(
                connection,
                normalized_ticket,
                "validation.ticket_status_changed",
                {"from": ticket.status, "to": status, "evidence": dict(evidence or {})},
                now,
            )
            return self._get_in_connection(connection, normalized_ticket)

    def record_result(
        self,
        ticket_id: str,
        status: str,
        *,
        evidence: Mapping[str, object] | None = None,
    ) -> ValidationTicket:
        """Persist a terminal worker result without making the owner wait.

        A queue worker can report a terminal result directly from ``queued`` or
        ``materializing``.  Those states mean the coordinator accepted the
        request; they must not force a caller to poll or manufacture a separate
        ``running`` acknowledgement before a real result can be recorded.
        """
        if status not in _TERMINAL:
            raise CoordinatorError(
                "validation_ticket_result_invalid",
                "Validation result status must be passed, failed, or snapshot_stale",
            )
        return self.transition(ticket_id, status, evidence=evidence)

    def get(self, ticket_id: str) -> ValidationTicket:
        with self.database.connect() as connection:
            return self._get_in_connection(connection, self._require_text("ticket_id", ticket_id))

    def claim_next(self) -> ValidationTicket | None:
        """Atomically reserve the oldest queued ticket for one worker."""
        now = utc_text()
        with self.database.transaction() as connection:
            row = connection.execute(
                """
                SELECT ticket_id FROM validation_tickets
                WHERE status='queued'
                ORDER BY created_at, ticket_id
                LIMIT 1
                """
            ).fetchone()
            if row is None:
                return None
            ticket_id = str(row["ticket_id"])
            cursor = connection.execute(
                """
                UPDATE validation_tickets SET status='materializing', updated_at=?
                WHERE ticket_id=? AND status='queued'
                """,
                (now, ticket_id),
            )
            if cursor.rowcount != 1:
                return None
            self._event(
                connection,
                ticket_id,
                "validation.ticket_status_changed",
                {"from": "queued", "to": "materializing", "evidence": {"phase": "claimed"}},
                now,
            )
            return self._get_in_connection(connection, ticket_id)

    def active_ticket(self) -> ValidationTicket | None:
        with self.database.connect() as connection:
            row = connection.execute(
                """
                SELECT ticket_id FROM validation_tickets
                WHERE status IN ('materializing', 'running')
                ORDER BY updated_at, ticket_id
                LIMIT 1
                """
            ).fetchone()
            if row is None:
                return None
            return self._get_in_connection(connection, str(row["ticket_id"]))

    def record_worker_event(
        self, ticket_id: str, event_type: str, payload: Mapping[str, object]
    ) -> None:
        normalized_ticket = self._require_text("ticket_id", ticket_id)
        normalized_event = self._require_text("event_type", event_type)
        now = utc_text()
        with self.database.transaction() as connection:
            self._get_in_connection(connection, normalized_ticket)
            self._event(connection, normalized_ticket, normalized_event, payload, now)

    def latest_worker_event(
        self, ticket_id: str, event_type: str
    ) -> Mapping[str, object] | None:
        normalized_ticket = self._require_text("ticket_id", ticket_id)
        normalized_event = self._require_text("event_type", event_type)
        with self.database.connect() as connection:
            row = connection.execute(
                """
                SELECT payload_json FROM validation_ticket_events
                WHERE ticket_id=? AND event_type=?
                ORDER BY event_id DESC
                LIMIT 1
                """,
                (normalized_ticket, normalized_event),
            ).fetchone()
        if row is None:
            return None
        payload = json.loads(str(row["payload_json"]))
        return payload if isinstance(payload, dict) else None

    @staticmethod
    def _require_text(field: str, value: object) -> str:
        if not isinstance(value, str) or not value.strip():
            raise CoordinatorError("validation_ticket_input_invalid", f"{field} must be non-empty text")
        return value.strip()

    def _manifest(self, value: Mapping[str, str]) -> dict[str, str]:
        if not isinstance(value, Mapping) or not value:
            raise CoordinatorError("validation_ticket_manifest_invalid", "source_manifest must be non-empty")
        normalized: dict[str, str] = {}
        for raw_path, raw_hash in value.items():
            if not isinstance(raw_path, str) or not _SAFE_PATH.fullmatch(raw_path.replace("\\", "/")):
                raise CoordinatorError("validation_ticket_manifest_invalid", "source_manifest path is unsafe")
            path = raw_path.replace("\\", "/")
            if any(part in {"", ".", ".."} for part in path.split("/")):
                raise CoordinatorError("validation_ticket_manifest_invalid", "source_manifest path is unsafe")
            if not isinstance(raw_hash, str) or _SHA256.fullmatch(raw_hash.casefold()) is None:
                raise CoordinatorError("validation_ticket_manifest_invalid", "source_manifest hashes must be SHA-256")
            normalized[path] = raw_hash.casefold()
        return dict(sorted(normalized.items(), key=lambda item: item[0].casefold()))

    def _command(self, value: tuple[str, ...] | list[str]) -> tuple[str, ...]:
        if not isinstance(value, (tuple, list)) or not value:
            raise CoordinatorError("validation_ticket_command_invalid", "command must be a non-empty string sequence")
        command = tuple(self._require_text("command", item) for item in value)
        return command

    @staticmethod
    def _mapping(field: str, value: Mapping[str, object]) -> dict[str, object]:
        if not isinstance(value, Mapping):
            raise CoordinatorError("validation_ticket_input_invalid", f"{field} must be an object")
        result = {str(key): item for key, item in value.items()}
        try:
            json.dumps(result, sort_keys=True, separators=(",", ":"))
        except (TypeError, ValueError) as error:
            raise CoordinatorError("validation_ticket_input_invalid", f"{field} must be JSON serializable") from error
        return result

    @staticmethod
    def _canonical(value: object) -> str:
        return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)

    def _get_in_connection(self, connection, ticket_id: str) -> ValidationTicket:
        row = connection.execute("SELECT * FROM validation_tickets WHERE ticket_id=?", (ticket_id,)).fetchone()
        if row is None:
            raise CoordinatorError("validation_ticket_not_found", f"Unknown validation ticket {ticket_id}")
        return ValidationTicket(
            ticket_id=str(row["ticket_id"]),
            session_id=str(row["session_id"]),
            plan_path=str(row["plan_path"]),
            status=str(row["status"]),
            source_manifest_hash=str(row["source_manifest_hash"]),
            source_manifest=json.loads(str(row["source_manifest_json"])),
            command=tuple(json.loads(str(row["command_json"]))),
            toolchain=json.loads(str(row["toolchain_json"])),
            coverage=json.loads(str(row["coverage_json"])),
        )

    @staticmethod
    def _event(connection, ticket_id: str, event_type: str, payload: Mapping[str, object], created_at: str) -> None:
        connection.execute(
            """
            INSERT INTO validation_ticket_events(ticket_id, event_type, payload_json, created_at)
            VALUES (?, ?, ?, ?)
            """,
            (ticket_id, event_type, json.dumps(dict(payload), sort_keys=True), created_at),
        )

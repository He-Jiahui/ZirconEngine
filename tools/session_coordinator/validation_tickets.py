"""Durable, immutable validation requests that never block a business Session."""

from __future__ import annotations

import hashlib
import json
import os
import re
from collections.abc import Callable
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from typing import Mapping
from uuid import uuid4

from .database import Database
from .cargo_command_policy import normalize_cargo_ticket_command
from .models import CoordinatorError, utc_text
from .portable_paths import normalize_portable_relative_path, portable_path_key
from .snapshots import ObjectStore
from .validation_copy_external import ExternalGitSource
from .validation_external_pins import (
    discover_pinned_external_sources,
    external_sources_from_coverage,
    merge_external_sources_into_coverage,
    seal_pinned_external_sources,
)


_SAFE_PATH = re.compile(r"^(?!/)(?![A-Za-z]:)[^\\]+$")
_SHA256 = re.compile(r"^[0-9a-f]{64}$")
_NONTERMINAL = frozenset({"queued", "materializing", "running"})
_TERMINAL = frozenset({"passed", "failed", "snapshot_stale"})
_TRANSITIONS = {
    "queued": frozenset({"materializing", "running", "passed", "failed", "snapshot_stale"}),
    "materializing": frozenset({"running", "passed", "failed", "snapshot_stale"}),
    "running": frozenset({"passed", "failed", "snapshot_stale"}),
}
_CARGO_TOOLCHAIN_NOT_REQUIRED = frozenset(
    {
        "",
        "disabled",
        "false",
        "none",
        "not required",
        "not-required",
        "not_required",
        "off",
    }
)
_SOURCE_SEALED_EVENT = "validation.ticket_source_sealed"
_SOURCE_SNAPSHOT_PREFIX = "validation-ticket-source:"
_EXTERNAL_SNAPSHOT_PREFIX = "validation-ticket-external:"


def validation_uses_cargo_lane(
    command: tuple[str, ...], toolchain: Mapping[str, object]
) -> bool:
    if command:
        executable = command[0].replace("\\", "/").rsplit("/", 1)[-1].casefold()
        if executable in {"cargo", "cargo.exe"}:
            return True
    cargo_identity = toolchain.get("cargo")
    if (
        isinstance(cargo_identity, str)
        and cargo_identity.strip().casefold() not in _CARGO_TOOLCHAIN_NOT_REQUIRED
    ):
        return True
    cargo_jobs = toolchain.get("cargo_jobs")
    rust_identity = toolchain.get("rust")
    return (
        isinstance(cargo_jobs, int)
        and not isinstance(cargo_jobs, bool)
        and cargo_jobs > 0
        and isinstance(rust_identity, str)
        and rust_identity.strip().casefold() not in _CARGO_TOOLCHAIN_NOT_REQUIRED
    )


def validation_dependency_roots(coverage: Mapping[str, object]) -> tuple[str, ...]:
    roots = coverage.get("dependencyRoots")
    if roots is None:
        roots = coverage.get("dependency_roots")
    if roots is None:
        raise CoordinatorError(
            "validation_ticket_dependency_roots_missing",
            "Non-Cargo validation tickets must declare coverage.dependencyRoots or coverage.dependency_roots",
        )
    if not isinstance(roots, (list, tuple)) or not roots:
        raise CoordinatorError(
            "validation_ticket_dependency_roots_invalid",
            "Validation dependency roots must be a non-empty string array",
        )
    normalized: list[str] = []
    for root in roots:
        if not isinstance(root, str) or not root.strip():
            raise CoordinatorError(
                "validation_ticket_dependency_roots_invalid",
                "Validation dependency roots must be a non-empty string array",
            )
        normalized.append(root)
    return tuple(dict.fromkeys(normalized))


@dataclass(frozen=True, slots=True)
class ValidationTicket:
    ticket_id: str
    session_id: str
    plan_path: str
    status: str
    baseline_epoch: int | None
    base_head: str | None
    source_manifest_hash: str
    source_manifest: Mapping[str, str | None]
    command: tuple[str, ...]
    toolchain: Mapping[str, object]
    coverage: Mapping[str, object]


@dataclass(frozen=True, slots=True)
class ValidationTicketReceipt:
    ticket: ValidationTicket
    request_id: str
    reused: bool


@dataclass(frozen=True, slots=True)
class _SubmissionContext:
    plan_path: str
    baseline_epoch: int | None
    base_head: str | None


class ValidationTicketService:
    """Persist validation work separately from business Session lifecycle.

    Coalescing is intentionally limited to exact sealed inputs.  A later
    worktree edit produces a different source manifest and therefore a new
    ticket rather than silently changing the work another caller submitted.
    """

    def __init__(
        self,
        database: Database,
        *,
        repo_root: str | Path | None = None,
        object_store: ObjectStore | None = None,
    ):
        if (repo_root is None) != (object_store is None):
            raise ValueError("repo_root and object_store must be configured together")
        self.database = database
        self.repo_root = Path(repo_root).resolve() if repo_root is not None else None
        self.object_store = object_store

    def submit(
        self,
        *,
        session_id: str,
        request_id: str,
        source_manifest: Mapping[str, str | None],
        command: tuple[str, ...] | list[str],
        toolchain: Mapping[str, object],
        coverage: Mapping[str, object],
        overlay_ownership_preflight: (
            Callable[[str, tuple[str, ...]], object] | None
        ) = None,
    ) -> ValidationTicketReceipt:
        normalized_session = self._require_text("session_id", session_id)
        normalized_request = self._require_text("request_id", request_id)
        with self.database.connect() as connection:
            existing = self._request_receipt(connection, normalized_request)
            if existing is not None:
                return existing
        manifest = self._manifest(source_manifest)
        normalized_command = self._command(command)
        normalized_toolchain = self._mapping("toolchain", toolchain)
        normalized_coverage = self._mapping("coverage", coverage)
        if validation_uses_cargo_lane(normalized_command, normalized_toolchain):
            normalized_command = normalize_cargo_ticket_command(
                normalized_command, self.repo_root
            )
        now = utc_text()

        captured_sources: tuple[tuple[str, str, bytes], ...] = ()
        captured_external_sources: tuple[tuple[str, str, bytes], ...] = ()
        captured_context: _SubmissionContext | None = None
        if self.object_store is not None:
            # Source I/O can cover hundreds of paths. Keep it outside the global
            # SQLite writer transaction, then repeat mutable admission below.
            with self.database.connect() as connection:
                existing = self._request_receipt(connection, normalized_request)
                if existing is not None:
                    return existing
                captured_context = self._submission_preflight(
                    connection,
                    session_id=normalized_session,
                    command=normalized_command,
                    toolchain=normalized_toolchain,
                    coverage=normalized_coverage,
                    manifest=manifest,
                    overlay_ownership_preflight=overlay_ownership_preflight,
                )
            captured_sources = self._capture_source_contents(manifest)

            if (
                validation_uses_cargo_lane(normalized_command, normalized_toolchain)
                and captured_context.base_head
                and (self.repo_root / ".git").exists()
            ):
                overlay_files = {
                    path: None for path, expected_hash in manifest.items() if expected_hash is None
                }
                overlay_files.update(
                    {
                        path: content
                        for path, _expected_hash, content in captured_sources
                    }
                )
                discovered = discover_pinned_external_sources(
                    self.repo_root,
                    baseline_commit=captured_context.base_head,
                    overlay_files=overlay_files,
                    command=normalized_command,
                )
                discovered, captured_external_sources = seal_pinned_external_sources(
                    list(discovered)
                )
                normalized_coverage = merge_external_sources_into_coverage(
                    normalized_coverage, discovered
                )

        manifest_json = self._canonical(manifest)
        command_json = self._canonical(normalized_command)
        toolchain_json = self._canonical(normalized_toolchain)
        coverage_json = self._canonical(normalized_coverage)
        manifest_hash = hashlib.sha256(manifest_json.encode("utf-8")).hexdigest()

        captured_objects = (*captured_sources, *captured_external_sources)
        with self._submission_transaction(captured_objects) as connection:
            existing = self._request_receipt(connection, normalized_request)
            if existing is not None:
                return existing
            submission = self._submission_preflight(
                connection,
                session_id=normalized_session,
                command=normalized_command,
                toolchain=normalized_toolchain,
                coverage=normalized_coverage,
                manifest=manifest,
                overlay_ownership_preflight=overlay_ownership_preflight,
            )
            if captured_context is not None and captured_context != submission:
                raise CoordinatorError(
                    "validation_ticket_baseline_changed",
                    "Session baseline changed while validation sources were sealed; submit again",
                    details={
                        "capturedBaseHead": captured_context.base_head,
                        "currentBaseHead": submission.base_head,
                    },
                )
            dedupe_key = hashlib.sha256(
                "\n".join(
                    (
                        manifest_hash,
                        command_json,
                        toolchain_json,
                        coverage_json,
                        str(submission.baseline_epoch or ""),
                        submission.base_head or "",
                    )
                ).encode("utf-8")
            ).hexdigest()

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
                if not self._source_is_sealed_in_connection(connection, ticket_id):
                    object_count, byte_count = self._store_source_objects(
                        connection, captured_objects
                    )
                    self._pin_source_objects(
                        connection,
                        ticket_id=ticket_id,
                        session_id=str(
                            connection.execute(
                                "SELECT session_id FROM validation_tickets WHERE ticket_id=?",
                                (ticket_id,),
                            ).fetchone()[0]
                        ),
                        manifest=manifest,
                        manifest_hash=manifest_hash,
                        baseline_epoch=submission.baseline_epoch,
                        object_count=object_count,
                        byte_count=byte_count,
                        created_at=now,
                    )
            else:
                object_count, byte_count = self._store_source_objects(
                    connection, captured_objects
                )
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
                        submission.plan_path,
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
                    {
                        "sessionId": normalized_session,
                        "sourceManifestHash": manifest_hash,
                        "baselineEpoch": submission.baseline_epoch,
                        "baseHead": submission.base_head,
                    },
                    now,
                )
                self._pin_source_objects(
                    connection,
                    ticket_id=ticket_id,
                    session_id=normalized_session,
                    manifest=manifest,
                    manifest_hash=manifest_hash,
                    baseline_epoch=submission.baseline_epoch,
                    object_count=object_count,
                    byte_count=byte_count,
                    created_at=now,
                )
            self._pin_external_objects(
                connection,
                ticket_id=ticket_id,
                session_id=str(
                    connection.execute(
                        "SELECT session_id FROM validation_tickets WHERE ticket_id=?",
                        (ticket_id,),
                    ).fetchone()[0]
                ),
                coverage=normalized_coverage,
                baseline_epoch=submission.baseline_epoch,
                created_at=now,
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

    def _request_receipt(
        self, connection, request_id: str
    ) -> ValidationTicketReceipt | None:
        row = connection.execute(
            "SELECT ticket_id FROM validation_ticket_requests WHERE request_id=?",
            (request_id,),
        ).fetchone()
        if row is None:
            return None
        ticket = self._get_in_connection(connection, str(row["ticket_id"]))
        return ValidationTicketReceipt(ticket, request_id, reused=False)

    def _submission_preflight(
        self,
        connection,
        *,
        session_id: str,
        command: tuple[str, ...],
        toolchain: Mapping[str, object],
        coverage: Mapping[str, object],
        manifest: Mapping[str, str | None],
        overlay_ownership_preflight: (
            Callable[[str, tuple[str, ...]], object] | None
        ),
    ) -> _SubmissionContext:
        owner = connection.execute(
            "SELECT plan_path, baseline_epoch, base_head FROM sessions WHERE session_id=?",
            (session_id,),
        ).fetchone()
        if owner is None:
            raise CoordinatorError("session_not_found", f"Unknown Session {session_id}")
        plan_path = str(owner["plan_path"] or "")
        if not plan_path:
            raise CoordinatorError(
                "validation_ticket_plan_missing",
                "Validation ticket owner must be registered to a numbered Plan",
            )
        if not validation_uses_cargo_lane(command, toolchain):
            validation_dependency_roots(coverage)
        if overlay_ownership_preflight is not None:
            overlay_ownership_preflight(session_id, tuple(manifest))
        return _SubmissionContext(
            plan_path=plan_path,
            baseline_epoch=(
                int(owner["baseline_epoch"])
                if owner["baseline_epoch"] is not None
                else None
            ),
            base_head=(str(owner["base_head"]) if owner["base_head"] else None),
        )

    def transition(self, ticket_id: str, status: str, *, evidence: Mapping[str, object] | None = None) -> ValidationTicket:
        normalized_ticket = self._require_text("ticket_id", ticket_id)
        if status not in _NONTERMINAL | _TERMINAL:
            raise CoordinatorError("validation_ticket_status_invalid", f"Unsupported ticket status: {status}")
        normalized_evidence = self._mapping("evidence", {} if evidence is None else evidence)
        now = utc_text()
        with self.database.transaction() as connection:
            ticket = self._get_in_connection(connection, normalized_ticket)
            if ticket.status == status:
                if status in _TERMINAL:
                    self._release_source_pin(connection, normalized_ticket)
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
                {"from": ticket.status, "to": status, "evidence": normalized_evidence},
                now,
            )
            if status in _TERMINAL:
                self._release_source_pin(connection, normalized_ticket)
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
        tickets = self.active_tickets(limit=1)
        return tickets[0] if tickets else None

    def active_tickets(self, *, limit: int | None = None) -> tuple[ValidationTicket, ...]:
        if limit is not None and limit < 1:
            raise ValueError("limit must be positive")
        with self.database.connect() as connection:
            query = """
                SELECT ticket_id FROM validation_tickets
                WHERE status IN ('materializing', 'running')
                ORDER BY updated_at, ticket_id
            """
            parameters: tuple[int, ...] = ()
            if limit is not None:
                query += " LIMIT ?"
                parameters = (limit,)
            rows = connection.execute(query, parameters).fetchall()
            return tuple(
                self._get_in_connection(connection, str(row["ticket_id"]))
                for row in rows
            )

    def record_worker_event(
        self, ticket_id: str, event_type: str, payload: Mapping[str, object]
    ) -> None:
        normalized_ticket = self._require_text("ticket_id", ticket_id)
        normalized_event = self._require_text("event_type", event_type)
        normalized_payload = self._mapping("payload", payload)
        now = utc_text()
        with self.database.transaction() as connection:
            self._get_in_connection(connection, normalized_ticket)
            self._event(connection, normalized_ticket, normalized_event, normalized_payload, now)

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

    def source_is_sealed(self, ticket_id: str) -> bool:
        normalized_ticket = self._require_text("ticket_id", ticket_id)
        with self.database.connect() as connection:
            self._get_in_connection(connection, normalized_ticket)
            return self._source_is_sealed_in_connection(
                connection, normalized_ticket
            )

    @staticmethod
    def _source_is_sealed_in_connection(connection, ticket_id: str) -> bool:
        purpose = f"{_SOURCE_SNAPSHOT_PREFIX}{ticket_id}"
        return (
            connection.execute(
                """
                SELECT 1 FROM validation_ticket_events AS sealed
                WHERE sealed.ticket_id=? AND sealed.event_type=?
                  AND EXISTS (
                      SELECT 1 FROM snapshots AS pin WHERE pin.purpose=?
                  )
                LIMIT 1
                """,
                (ticket_id, _SOURCE_SEALED_EVENT, purpose),
            ).fetchone()
            is not None
        )

    def _capture_source_contents(
        self, manifest: Mapping[str, str | None]
    ) -> tuple[tuple[str, str, bytes], ...]:
        if self.object_store is None or self.repo_root is None:
            return ()
        captured: list[tuple[str, str, bytes]] = []
        for path, expected_hash in manifest.items():
            source = self._source_path(path)
            if expected_hash is None:
                if os.path.lexists(source):
                    raise CoordinatorError(
                        "validation_ticket_source_snapshot_stale",
                        f"Deleted validation source exists at submit time: {path}",
                        details={"path": path},
                    )
                continue
            try:
                if source.is_symlink() or not source.is_file():
                    raise OSError("source is not a regular file")
                content = source.read_bytes()
            except OSError as error:
                raise CoordinatorError(
                    "validation_ticket_source_snapshot_stale",
                    f"Validation source is unavailable at submit time: {path}",
                    details={"path": path},
                ) from error
            actual_hash = hashlib.sha256(content).hexdigest()
            if actual_hash != expected_hash:
                raise CoordinatorError(
                    "validation_ticket_source_snapshot_stale",
                    f"Validation source changed before it could be sealed: {path}",
                    details={"path": path},
                )
            captured.append((path, expected_hash, content))
        return tuple(captured)

    def _store_source_objects(
        self, connection, captured: tuple[tuple[str, str, bytes], ...]
    ) -> tuple[int, int]:
        if self.object_store is None:
            return 0, 0
        # Validate the whole submission before ObjectStore.put() creates files.
        # A later stale path must not leave earlier objects orphaned by rollback.
        created_paths = {
            expected_hash: self.object_store.path_for_hash(expected_hash)
            for _path, expected_hash, _content in captured
            if not self.object_store.path_for_hash(expected_hash).exists()
        }
        try:
            for _path, expected_hash, content in captured:
                stored_hash = self.object_store.put(content, connection=connection)
                if stored_hash != expected_hash:
                    raise AssertionError("content-addressed validation source hash changed")
        except BaseException:
            for target in created_paths.values():
                target.unlink(missing_ok=True)
            raise
        return len(captured), sum(len(content) for _path, _hash, content in captured)

    @contextmanager
    def _submission_transaction(
        self, captured: tuple[tuple[str, str, bytes], ...]
    ):
        new_hashes: set[str] = set()
        if self.object_store is not None:
            new_hashes = {
                expected_hash
                for _path, expected_hash, _content in captured
                if not self.object_store.path_for_hash(expected_hash).exists()
            }
        try:
            with self.database.transaction() as connection:
                yield connection
        except BaseException:
            if self.object_store is not None and new_hashes:
                with self.database.transaction() as cleanup_connection:
                    for object_hash in new_hashes:
                        exists = cleanup_connection.execute(
                            "SELECT 1 FROM objects WHERE object_hash=?",
                            (object_hash,),
                        ).fetchone()
                        if exists is None:
                            self.object_store.path_for_hash(object_hash).unlink(
                                missing_ok=True
                            )
            raise

    def _source_path(self, relative_path: str) -> Path:
        if self.repo_root is None:
            raise AssertionError("source sealing requires a repository root")
        candidate = self.repo_root.joinpath(*relative_path.split("/"))
        current = self.repo_root
        for part in relative_path.split("/"):
            current = current / part
            if current.is_symlink():
                raise CoordinatorError(
                    "validation_ticket_source_link_forbidden",
                    "Validation source sealing does not follow filesystem links",
                    details={"path": relative_path},
                )
        try:
            candidate.resolve(strict=False).relative_to(self.repo_root)
        except ValueError as error:
            raise CoordinatorError(
                "validation_ticket_manifest_invalid",
                "source_manifest path is unsafe",
                details={"path": relative_path},
            ) from error
        return candidate

    def _pin_source_objects(
        self,
        connection,
        *,
        ticket_id: str,
        session_id: str,
        manifest: Mapping[str, str | None],
        manifest_hash: str,
        baseline_epoch: int | None,
        object_count: int,
        byte_count: int,
        created_at: str,
    ) -> None:
        if self.object_store is None:
            return
        purpose = f"{_SOURCE_SNAPSHOT_PREFIX}{ticket_id}"
        existing = connection.execute(
            "SELECT manifest_json FROM snapshots WHERE purpose=? LIMIT 1",
            (purpose,),
        ).fetchone()
        manifest_json = self._canonical(manifest)
        if existing is None:
            connection.execute(
                """
                INSERT INTO snapshots(
                    session_id, baseline_epoch, manifest_json, purpose, created_at
                ) VALUES (?, ?, ?, ?, ?)
                """,
                (session_id, baseline_epoch, manifest_json, purpose, created_at),
            )
        elif str(existing["manifest_json"]) != manifest_json:
            raise CoordinatorError(
                "validation_ticket_source_pin_conflict",
                "Validation ticket source pin does not match its immutable manifest",
                details={"ticketId": ticket_id},
            )
        if not self._source_is_sealed_in_connection(connection, ticket_id):
            self._event(
                connection,
                ticket_id,
                _SOURCE_SEALED_EVENT,
                {
                    "sourceManifestHash": manifest_hash,
                    "objectCount": object_count,
                    "byteCount": byte_count,
                },
                created_at,
            )

    def _pin_external_objects(
        self,
        connection,
        *,
        ticket_id: str,
        session_id: str,
        coverage: Mapping[str, object],
        baseline_epoch: int | None,
        created_at: str,
    ) -> None:
        if self.object_store is None:
            return
        manifest = {
            f"external/{source.mount_path}.tar": source.archive_hash
            for source in (
                ExternalGitSource.from_payload(payload)
                for payload in external_sources_from_coverage(coverage)
            )
            if source.archive_hash is not None
        }
        if not manifest:
            return
        purpose = f"{_EXTERNAL_SNAPSHOT_PREFIX}{ticket_id}"
        manifest_json = self._canonical(manifest)
        existing = connection.execute(
            "SELECT manifest_json FROM snapshots WHERE purpose=? LIMIT 1",
            (purpose,),
        ).fetchone()
        if existing is None:
            connection.execute(
                """
                INSERT INTO snapshots(
                    session_id, baseline_epoch, manifest_json, purpose, created_at
                ) VALUES (?, ?, ?, ?, ?)
                """,
                (session_id, baseline_epoch, manifest_json, purpose, created_at),
            )
        elif str(existing["manifest_json"]) != manifest_json:
            raise CoordinatorError(
                "validation_ticket_external_pin_conflict",
                "Validation ticket external pin does not match its immutable archive",
                details={"ticketId": ticket_id},
            )

    @staticmethod
    def _release_source_pin(connection, ticket_id: str) -> None:
        connection.execute(
            "DELETE FROM snapshots WHERE purpose IN (?, ?)",
            (
                f"{_SOURCE_SNAPSHOT_PREFIX}{ticket_id}",
                f"{_EXTERNAL_SNAPSHOT_PREFIX}{ticket_id}",
            ),
        )

    @staticmethod
    def _require_text(field: str, value: object) -> str:
        if not isinstance(value, str) or not value.strip():
            raise CoordinatorError("validation_ticket_input_invalid", f"{field} must be non-empty text")
        return value.strip()

    def _manifest(
        self, value: Mapping[str, str | None]
    ) -> dict[str, str | None]:
        if not isinstance(value, Mapping) or not value:
            raise CoordinatorError("validation_ticket_manifest_invalid", "source_manifest must be non-empty")
        normalized: dict[str, str | None] = {}
        path_keys: set[str] = set()
        for raw_path, raw_hash in value.items():
            path = normalize_portable_relative_path(
                raw_path,
                code="validation_ticket_manifest_invalid",
                message="source_manifest path is unsafe",
            )
            folded = path.casefold()
            path_key = portable_path_key(path)
            protected = (
                folded == ".git"
                or folded.startswith(".git/")
                or folded == "target"
                or folded.startswith("target/")
                or folded == ".codex/state"
                or folded.startswith(".codex/state/")
            )
            if (
                protected
                or path_key in path_keys
            ):
                raise CoordinatorError("validation_ticket_manifest_invalid", "source_manifest path is unsafe")
            path_keys.add(path_key)
            if raw_hash is None:
                normalized[path] = None
            elif isinstance(raw_hash, str) and _SHA256.fullmatch(raw_hash.casefold()):
                normalized[path] = raw_hash.casefold()
            else:
                raise CoordinatorError(
                    "validation_ticket_manifest_invalid",
                    "source_manifest values must be SHA-256 or null deletion tombstones",
                )
        return dict(sorted(normalized.items(), key=lambda item: item[0].casefold()))

    def _command(self, value: tuple[str, ...] | list[str]) -> tuple[str, ...]:
        if not isinstance(value, (tuple, list)) or not value:
            raise CoordinatorError("validation_ticket_command_invalid", "command must be a non-empty string sequence")
        command = tuple(self._require_text("command", item) for item in value)
        return command

    @classmethod
    def _mapping(cls, field: str, value: Mapping[str, object]) -> dict[str, object]:
        if not isinstance(value, Mapping):
            raise CoordinatorError("validation_ticket_input_invalid", f"{field} must be an object")
        result = cls._json_value(field, value, set())
        if not isinstance(result, dict):
            raise AssertionError("mapping normalization must preserve object shape")
        return result

    @classmethod
    def _json_value(cls, field: str, value: object, active: set[int]) -> object:
        if isinstance(value, Mapping):
            identity = id(value)
            if identity in active:
                raise CoordinatorError(
                    "validation_ticket_input_invalid",
                    f"{field} must not contain a circular JSON value",
                )
            active.add(identity)
            try:
                result: dict[str, object] = {}
                for key, item in value.items():
                    if not isinstance(key, str):
                        raise CoordinatorError(
                            "validation_ticket_input_invalid",
                            f"{field} object keys must be strings",
                        )
                    result[key] = cls._json_value(field, item, active)
                return result
            finally:
                active.remove(identity)
        if isinstance(value, (list, tuple)):
            identity = id(value)
            if identity in active:
                raise CoordinatorError(
                    "validation_ticket_input_invalid",
                    f"{field} must not contain a circular JSON value",
                )
            active.add(identity)
            try:
                return [cls._json_value(field, item, active) for item in value]
            finally:
                active.remove(identity)
        try:
            json.dumps(value, allow_nan=False)
        except (TypeError, ValueError) as error:
            raise CoordinatorError("validation_ticket_input_invalid", f"{field} must be JSON serializable") from error
        return value

    @staticmethod
    def _canonical(value: object) -> str:
        return json.dumps(
            value,
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=True,
            allow_nan=False,
        )

    def _get_in_connection(self, connection, ticket_id: str) -> ValidationTicket:
        row = connection.execute("SELECT * FROM validation_tickets WHERE ticket_id=?", (ticket_id,)).fetchone()
        if row is None:
            raise CoordinatorError("validation_ticket_not_found", f"Unknown validation ticket {ticket_id}")
        submitted = connection.execute(
            """
            SELECT payload_json FROM validation_ticket_events
            WHERE ticket_id=? AND event_type='validation.ticket_submitted'
            ORDER BY event_id LIMIT 1
            """,
            (ticket_id,),
        ).fetchone()
        submission_payload: Mapping[str, object] = {}
        if submitted is not None:
            decoded = json.loads(str(submitted["payload_json"]))
            if isinstance(decoded, dict):
                submission_payload = decoded
        raw_epoch = submission_payload.get("baselineEpoch")
        baseline_epoch = (
            int(raw_epoch)
            if isinstance(raw_epoch, int) and not isinstance(raw_epoch, bool)
            else None
        )
        raw_head = submission_payload.get("baseHead")
        return ValidationTicket(
            ticket_id=str(row["ticket_id"]),
            session_id=str(row["session_id"]),
            plan_path=str(row["plan_path"]),
            status=str(row["status"]),
            baseline_epoch=baseline_epoch,
            base_head=str(raw_head) if isinstance(raw_head, str) and raw_head else None,
            source_manifest_hash=str(row["source_manifest_hash"]),
            source_manifest=json.loads(str(row["source_manifest_json"])),
            command=tuple(json.loads(str(row["command_json"]))),
            toolchain=json.loads(str(row["toolchain_json"])),
            coverage=json.loads(str(row["coverage_json"])),
        )

    @staticmethod
    def _event(connection, ticket_id: str, event_type: str, payload: Mapping[str, object], created_at: str) -> None:
        normalized_payload = ValidationTicketService._mapping("event payload", payload)
        connection.execute(
            """
            INSERT INTO validation_ticket_events(ticket_id, event_type, payload_json, created_at)
            VALUES (?, ?, ?, ?)
            """,
            (
                ticket_id,
                event_type,
                ValidationTicketService._canonical(normalized_payload),
                created_at,
            ),
        )

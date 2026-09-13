"""Durable, immutable validation requests that never block a business Session."""

from __future__ import annotations

import hashlib
import json
import os
import time
from collections.abc import Callable
from contextlib import contextmanager
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Mapping
from uuid import uuid4

from .database import Database
from .cargo_command_policy import normalize_cargo_ticket_command
from .models import CoordinatorError, utc_text
from . import validation_ticket_inputs as ticket_inputs
from . import validation_ticket_identity as ticket_identity
from .snapshots import ObjectStore
from .validation_copy_external import ExternalGitSource
from .validation_external_pins import (
    discover_and_seal_pinned_external_sources,
    external_sources_from_coverage,
    merge_external_sources_into_coverage,
)
from .validation_ticket_policy import (
    FAILURE_REUSABLE_EVENT,
    FAILURE_REUSED_EVENT,
    FailureReusePolicy,
    deterministic_failure,
    execution_environment_identity,
    reuse_blocker,
    submission_details,
    terminal_diagnostic,
    validator_identity,
)


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
    blockers: tuple[Mapping[str, object], ...] = ()
    original_failure_ticket_id: str | None = None
    reuse_reason: str | None = None
    timings: Mapping[str, object] | None = None
    diagnostic: Mapping[str, object] | None = None


@dataclass(frozen=True, slots=True)
class ValidationTicketReceipt:
    ticket: ValidationTicket
    request_id: str
    reused: bool
    original_failure_ticket_id: str | None = None
    reuse_reason: str | None = None


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
        validator_version: str | None = None,
        runtime_identity: Callable[[tuple[str, ...], Path], str] | None = None,
        failure_reuse_policy: FailureReusePolicy | None = None,
    ):
        if (repo_root is None) != (object_store is None):
            raise ValueError("repo_root and object_store must be configured together")
        self.database = database
        self.repo_root = Path(repo_root).resolve() if repo_root is not None else None
        self.object_store = object_store
        self.validator_version = validator_version or validator_identity()
        self.runtime_identity = runtime_identity
        self.failure_reuse_policy = failure_reuse_policy or FailureReusePolicy()

    def submit(
        self,
        *,
        session_id: str,
        request_id: str,
        source_manifest: Mapping[str, str | None],
        command: tuple[str, ...] | list[str],
        toolchain: Mapping[str, object],
        coverage: Mapping[str, object],
        overlay_ownership_preflight: Callable[[str, tuple[str, ...]], object] | None = None,
        force_rerun_reason: str | None = None,
    ) -> ValidationTicketReceipt:
        from .validation_preflight import enrich_admission_error

        try:
            return self._submit(
                session_id=session_id, request_id=request_id,
                source_manifest=source_manifest, command=command,
                toolchain=toolchain, coverage=coverage,
                overlay_ownership_preflight=overlay_ownership_preflight,
                force_rerun_reason=force_rerun_reason,
            )
        except CoordinatorError as error:
            raise enrich_admission_error(error) from error

    def _submit(
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
        force_rerun_reason: str | None = None,
    ) -> ValidationTicketReceipt:
        started = time.monotonic()
        normalized_session = self._require_text("session_id", session_id)
        normalized_request = self._require_text("request_id", request_id)
        force_reason = (
            self._require_text("force_rerun_reason", force_rerun_reason)
            if force_rerun_reason is not None else None
        )
        if force_reason is not None and len(force_reason) > 1024:
            raise CoordinatorError("validation_force_rerun_reason_invalid", "Force rerun reason exceeds 1024 characters")
        with self.database.connect() as connection:
            existing = self._request_receipt(connection, normalized_request, normalized_session)
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
        runtime_identity: str | None = None
        if self.object_store is not None:
            # Source I/O can cover hundreds of paths. Keep it outside the global
            # SQLite writer transaction, then repeat mutable admission below.
            with self.database.connect() as connection:
                existing = self._request_receipt(connection, normalized_request, normalized_session)
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
                from .build_policy import pinned_development_command

                normalized_command, link_mode = pinned_development_command(
                    normalized_command, self.repo_root, captured_context.base_head,
                    overlay_files, str(normalized_toolchain.get("linkMode", "auto")),
                )
                normalized_toolchain = {**normalized_toolchain, "linkMode": link_mode}
                discovered, captured_external_sources = (
                    discover_and_seal_pinned_external_sources(
                        self.repo_root,
                        baseline_commit=captured_context.base_head,
                        overlay_files=overlay_files,
                        command=normalized_command,
                    )
                )
                normalized_coverage = merge_external_sources_into_coverage(
                    normalized_coverage, discovered
                )
                from .validation_preflight import preflight_pinned_cargo

                archives = {digest: content for _path, digest, content in captured_external_sources}
                runtime_identity = preflight_pinned_cargo(
                    self.repo_root,
                    baseline_commit=captured_context.base_head,
                    overlay_files=overlay_files,
                    command=normalized_command,
                    external_sources=tuple(ExternalGitSource.from_payload(item) for item in discovered),
                    external_archive_loader=archives.__getitem__,
                    runtime_identity=self.runtime_identity,
                    planner_parent=self.object_store.root.parent,
                )

        failure_reuse_eligible = runtime_identity is not None or (
            captured_context is not None and bool(captured_context.base_head)
            and not validation_uses_cargo_lane(normalized_command, normalized_toolchain)
        )

        manifest_json = self._canonical(manifest)
        command_json = self._canonical(normalized_command)
        toolchain_json = self._canonical(normalized_toolchain)
        coverage_json = self._canonical(normalized_coverage)
        manifest_hash = hashlib.sha256(manifest_json.encode("utf-8")).hexdigest()

        captured_objects = (*captured_sources, *captured_external_sources)
        now = utc_text()
        with self._submission_transaction(captured_objects) as connection:
            existing = self._request_receipt(connection, normalized_request, normalized_session)
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
            identity = ticket_identity.create_identity(
                baseline_epoch=submission.baseline_epoch,
                base_head=submission.base_head,
                validator_version=self.validator_version,
                runtime_identity=runtime_identity,
                execution_environment_hash=execution_environment_identity(),
            )
            dedupe_key = ticket_identity.dedupe_key(
                identity=identity,
                source_manifest_hash=manifest_hash,
                command=normalized_command,
                toolchain=normalized_toolchain,
                coverage=normalized_coverage,
            )

            reusable = connection.execute(
                """
                SELECT ticket_id FROM validation_tickets
                WHERE dedupe_key=? AND status IN ('queued', 'materializing', 'running')
                ORDER BY created_at, ticket_id LIMIT 1
                """,
                (dedupe_key,),
            ).fetchone()
            prior_failure = (
                self.failure_reuse_policy.lookup(connection, dedupe_key)
                if reusable is None and force_reason is None and failure_reuse_eligible
                else None
            )
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
                        "validatorVersion": self.validator_version,
                        "failureReuseEligible": failure_reuse_eligible,
                        "runtimeIdentityHash": identity["runtimeIdentityHash"],
                        "identity": identity,
                        "originalFailureTicketId": prior_failure[0] if prior_failure else None,
                        "reuseReason": prior_failure[1] if prior_failure else None,
                        "forceRerunReason": force_reason,
                        "admissionMs": max(0, int((time.monotonic() - started) * 1000)),
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
            if prior_failure is not None:
                original_id, reason = prior_failure
                self._event(connection, ticket_id, FAILURE_REUSED_EVENT, {
                    "originalFailureTicketId": original_id, "reuseReason": reason,
                    "requestId": normalized_request,
                }, now)
                connection.execute(
                    "UPDATE validation_tickets SET status='failed', updated_at=? WHERE ticket_id=?",
                    (now, ticket_id),
                )
                self._event(connection, ticket_id, "validation.ticket_status_changed", {
                    "from": "queued", "to": "failed", "evidence": {
                        "phase": "failure_reuse", "originalFailureTicketId": original_id,
                        "reuseReason": reason, "blockers": [reuse_blocker(original_id, reason)],
                    },
                }, now)
                self._release_source_pin(connection, ticket_id)
                reused = True
            if force_reason is not None:
                self._event(connection, ticket_id, "validation.ticket_rerun_requested", {
                    "requestId": normalized_request, "sessionId": normalized_session,
                    "forceRerunReason": force_reason, "coalesced": reused,
                }, now)
            return ValidationTicketReceipt(
                self._get_in_connection(connection, ticket_id), normalized_request, reused,
                prior_failure[0] if prior_failure else None,
                prior_failure[1] if prior_failure else None,
            )

    def _request_receipt(
        self, connection, request_id: str, session_id: str
    ) -> ValidationTicketReceipt | None:
        row = connection.execute(
            "SELECT ticket_id, session_id FROM validation_ticket_requests WHERE request_id=?",
            (request_id,),
        ).fetchone()
        if row is None:
            return None
        if str(row["session_id"]) != session_id:
            raise CoordinatorError("validation_request_owner_mismatch", "Request ID belongs to a different Session")
        ticket = self._get_in_connection(connection, str(row["ticket_id"]))
        return ValidationTicketReceipt(
            ticket, request_id, reused=ticket.original_failure_ticket_id is not None,
            original_failure_ticket_id=ticket.original_failure_ticket_id,
            reuse_reason=ticket.reuse_reason,
        )

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
                submitted = submission_details(connection, normalized_ticket)
                reason = deterministic_failure(
                    normalized_evidence,
                    compiler_identity_verified=bool(submitted.get("runtimeIdentityHash")),
                ) if status == "failed" else None
                if reason and submitted.get("failureReuseEligible"):
                    self._event(connection, normalized_ticket, FAILURE_REUSABLE_EVENT, {"reason": reason}, now)
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

    def verify_runtime_identity(self, ticket: ValidationTicket, source_root: Path) -> None:
        if self.runtime_identity is None:
            return
        with self.database.connect() as connection:
            expected = submission_details(connection, ticket.ticket_id).get("runtimeIdentityHash")
        if expected is None:
            return
        actual = self.runtime_identity(ticket.command, source_root)
        if hashlib.sha256(actual.encode()).hexdigest() != expected:
            raise CoordinatorError(
                "validation_ticket_toolchain_changed",
                "The managed toolchain changed after submission; submit a fresh validation ticket",
            )

    def claim_next(self) -> ValidationTicket | None:
        """Atomically reserve the oldest queued ticket for one worker."""
        now = utc_text()
        from .validation_failure_gate import record_blockers

        with self.database.transaction() as connection:
            rows = connection.execute(
                """
                SELECT ticket_id FROM validation_tickets
                WHERE status='queued'
                ORDER BY created_at, ticket_id
                """
            ).fetchall()
            ticket_id = None
            for row in rows:
                candidate = self._get_in_connection(connection, str(row["ticket_id"]))
                record_blockers(connection, candidate.ticket_id, candidate.blockers, now)
                if not candidate.blockers:
                    ticket_id = candidate.ticket_id
                    break
            if ticket_id is None:
                return None
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

    def defer_for_dependencies(self, ticket_id: str) -> bool:
        from .validation_failure_gate import record_blockers

        with self.database.transaction() as connection:
            ticket = self._get_in_connection(connection, ticket_id)
            if ticket.status != "materializing" or not ticket.blockers:
                return False
            now = utc_text()
            record_blockers(connection, ticket_id, ticket.blockers, now)
            connection.execute(
                "UPDATE validation_tickets SET status='queued', updated_at=? WHERE ticket_id=?",
                (now, ticket_id),
            )
            self._event(connection, ticket_id, "validation.ticket_status_changed", {
                "from": "materializing", "to": "queued",
                "evidence": {"phase": "dependency_wait", "blockers": list(ticket.blockers)},
            }, now)
            return True

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
        for _path, expected_hash, content in captured:
            stored_hash = self.object_store.put(content, connection=connection)
            if stored_hash != expected_hash:
                raise AssertionError("content-addressed validation source hash changed")
        return len(captured), sum(len(content) for _path, _hash, content in captured)

    @contextmanager
    def _submission_transaction(
        self, captured: tuple[tuple[str, str, bytes], ...]
    ):
        if self.object_store is None:
            with self.database.transaction() as connection:
                yield connection
            return
        with self.object_store.transaction() as connection:
            yield connection

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

    _require_text = staticmethod(ticket_inputs.require_text)
    _manifest = staticmethod(ticket_inputs.manifest)

    _command = staticmethod(ticket_inputs.command)
    _mapping = staticmethod(ticket_inputs.mapping)
    _json_value = staticmethod(ticket_inputs.json_value)

    _canonical = staticmethod(ticket_inputs.canonical)

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
        ticket = ValidationTicket(
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
            original_failure_ticket_id=submission_payload.get("originalFailureTicketId"),
            reuse_reason=submission_payload.get("reuseReason"),
        )
        from .validation_failure_gate import blockers
        from .validation_timings import ticket_timings

        original_id = ticket.original_failure_ticket_id
        return replace(
            ticket,
            blockers=(
                (reuse_blocker(original_id, ticket.reuse_reason or ""),)
                if original_id else tuple(blockers(connection, ticket))
                if ticket.status in {"queued", "materializing"} else ()
            ),
            timings=ticket_timings(connection, ticket_id),
            diagnostic=terminal_diagnostic(connection, original_id or ticket_id),
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

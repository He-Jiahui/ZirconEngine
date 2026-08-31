"""Bounded worker that consumes validation tickets through immutable copies."""

from __future__ import annotations

import hashlib
import os
from collections.abc import Callable, Mapping
from pathlib import Path
from typing import Protocol

from .database import Database
from .models import CoordinatorError
from .validation_tickets import (
    ValidationTicket,
    ValidationTicketService,
    validation_dependency_roots,
    validation_uses_cargo_lane,
)
from .validation_external_pins import external_sources_from_coverage


_COPY_LINK_EVENT = "validation.ticket_copy_linked"
_RUN_LINK_EVENT = "validation.ticket_run_linked"
_ACTIVE_COPY_STATES = frozenset({"planned", "materializing"})
_SNAPSHOT_STALE_COPY_ERRORS = frozenset(
    {
        "validation_copy_attribution_stale",
        "validation_copy_owned_source_missing",
        "validation_copy_owned_source_reappeared",
    }
)


class ValidationCopyExecutor(Protocol):
    def require_overlay_ownership(
        self, session_id: str, overlay_paths: tuple[str, ...]
    ) -> tuple[str, ...]: ...

    def materialize_validation_async(
        self,
        session_id: str,
        *,
        dependency_roots: tuple[str, ...],
        overlay_paths: tuple[str, ...],
        sealed_overlay_manifest: Mapping[str, str | None] | None = None,
        baseline_commit: str | None = None,
    ): ...

    def materialize_cargo_async(
        self,
        session_id: str,
        *,
        command: tuple[str, ...],
        overlay_paths: tuple[str, ...],
        discover_external_sources: bool,
        external_sources: tuple[Mapping[str, object], ...] = (),
        sealed_overlay_manifest: Mapping[str, str | None] | None = None,
        baseline_commit: str | None = None,
    ): ...

    def status(self, session_id: str, job_id: str): ...

    def cleanup_terminal_ticket_copy(self, ticket_id: str, job_id: str) -> bool: ...

    def start(
        self,
        session_id: str,
        job_id: str,
        *,
        command: tuple[str, ...],
        run_id: str,
    ) -> Mapping[str, object]: ...


class ValidationTicketWorker:
    """Advance a bounded set of isolated validations and preserve FIFO admission."""

    def __init__(
        self,
        database: Database,
        repo_root: str | Path,
        tickets: ValidationTicketService,
        workspace_copy: ValidationCopyExecutor,
        *,
        run_result_lookup: Callable[[str], Mapping[str, object] | None] | None = None,
        max_active: int = 2,
    ) -> None:
        if max_active < 1:
            raise ValueError("max_active must be positive")
        self.database = database
        self.repo_root = Path(repo_root).resolve()
        self.tickets = tickets
        self.workspace_copy = workspace_copy
        self.run_result_lookup = run_result_lookup or self._run_result
        self.max_active = max_active

    def tick(self, *, stale_batch_size: int = 16) -> dict[str, int]:
        if stale_batch_size < 1:
            raise ValueError("stale_batch_size must be positive")
        result = {
            "snapshot_stale": 0,
            "materializing": 0,
            "running": 0,
            "passed": 0,
            "failed": 0,
        }
        for ticket in self.tickets.active_tickets():
            result[self._advance(ticket)] += 1

        active_count = len(self.tickets.active_tickets())
        for _ in range(stale_batch_size):
            if active_count >= self.max_active:
                break
            ticket = self.tickets.claim_next()
            if ticket is None:
                break
            sealed_manifest = (
                ticket.source_manifest
                if self.tickets.source_is_sealed(ticket.ticket_id)
                else None
            )
            if sealed_manifest is None:
                try:
                    self.workspace_copy.require_overlay_ownership(
                        ticket.session_id, tuple(ticket.source_manifest)
                    )
                except Exception as error:
                    self._terminal_error(ticket, "queue_claim", error)
                    result["failed"] += 1
                    continue
                drift = self._manifest_drift(self.repo_root, ticket.source_manifest)
                if drift:
                    self._record_terminal(
                        ticket,
                        "snapshot_stale",
                        evidence={"phase": "queue_claim", "driftPaths": drift},
                    )
                    result["snapshot_stale"] += 1
                    continue
            try:
                if self._uses_cargo_lane(ticket):
                    external_sources = external_sources_from_coverage(ticket.coverage)
                    record = self.workspace_copy.materialize_cargo_async(
                        ticket.session_id,
                        command=ticket.command,
                        overlay_paths=tuple(ticket.source_manifest),
                        external_sources=external_sources,
                        discover_external_sources=False,
                        sealed_overlay_manifest=sealed_manifest,
                        baseline_commit=ticket.base_head,
                    )
                else:
                    record = self.workspace_copy.materialize_validation_async(
                        ticket.session_id,
                        dependency_roots=self._dependency_roots(ticket),
                        overlay_paths=tuple(ticket.source_manifest),
                        sealed_overlay_manifest=sealed_manifest,
                        baseline_commit=ticket.base_head,
                    )
                self.tickets.record_worker_event(
                    ticket.ticket_id,
                    _COPY_LINK_EVENT,
                    {"jobId": str(record.job_id)},
                )
            except Exception as error:
                self._terminal_error(ticket, "materialization_submit", error)
                result["failed"] += 1
            else:
                result["materializing"] += 1
                active_count += 1
        return result

    def _advance(self, ticket: ValidationTicket) -> str:
        if ticket.status == "materializing":
            return self._advance_materializing(ticket)
        if ticket.status == "running":
            return self._advance_running(ticket)
        raise CoordinatorError(
            "validation_ticket_worker_state_invalid",
            f"Worker cannot advance validation ticket from {ticket.status}",
        )

    def _advance_materializing(self, ticket: ValidationTicket) -> str:
        link = self.tickets.latest_worker_event(ticket.ticket_id, _COPY_LINK_EVENT)
        job_id = str((link or {}).get("jobId") or "")
        if not job_id:
            self._record_terminal(
                ticket,
                "failed",
                evidence={
                    "phase": "materializing_recovery",
                    "errorCode": "validation_ticket_copy_link_missing",
                },
            )
            return "failed"
        try:
            record = self.workspace_copy.status(ticket.session_id, job_id)
        except Exception as error:
            self._terminal_error(ticket, "materialization_status", error, job_id=job_id)
            return "failed"

        status = str(record.status)
        if status in _ACTIVE_COPY_STATES:
            return "materializing"
        copy_failed = status == "failed" or (
            status == "removed"
            and (
                str(getattr(record, "materialization_phase", "")) == "failed"
                or bool(getattr(record, "error_code", None))
            )
        )
        if copy_failed:
            terminal_status = (
                "snapshot_stale"
                if str(record.error_code) in _SNAPSHOT_STALE_COPY_ERRORS
                else "failed"
            )
            self._record_terminal(
                ticket,
                terminal_status,
                evidence=self._copy_failure(record, job_id),
                job_id=job_id,
            )
            return terminal_status
        if status == "removed":
            run_link = self.tickets.latest_worker_event(ticket.ticket_id, _RUN_LINK_EVENT)
            uses_cargo_lane = self._uses_cargo_lane(ticket)
            if run_link is None and self.run_result_lookup(ticket.ticket_id) is None:
                materialization_kind = getattr(record, "materialization_kind", None)
                if not uses_cargo_lane or materialization_kind is None:
                    return self._restart_removed_copy(
                        ticket,
                        job_id,
                        uses_cargo_lane=uses_cargo_lane,
                    )
            return self._finish_from_run(ticket, job_id)
        if status == "running":
            self._link_running(ticket, job_id)
            return "running"
        if status != "materialized":
            self._record_terminal(
                ticket,
                "failed",
                evidence={
                    "phase": "materialization_status",
                    "errorCode": "validation_ticket_copy_state_invalid",
                    "jobId": job_id,
                    "copyStatus": status,
                },
                job_id=job_id,
            )
            return "failed"

        drift = self._manifest_drift(Path(record.source_root), ticket.source_manifest)
        if drift:
            self._record_terminal(
                ticket,
                "snapshot_stale",
                evidence={"phase": "materialized_copy", "jobId": job_id, "driftPaths": drift},
                job_id=job_id,
            )
            return "snapshot_stale"
        try:
            progress = self.workspace_copy.start(
                ticket.session_id,
                job_id,
                command=ticket.command,
                run_id=ticket.ticket_id,
            )
        except Exception as error:
            self._terminal_error(ticket, "run_start", error, job_id=job_id)
            return "failed"
        if str(progress.get("status") or "") == "waiting":
            return "materializing"
        self._link_running(ticket, job_id)
        return "running"

    def _restart_removed_copy(
        self,
        ticket: ValidationTicket,
        previous_job_id: str,
        *,
        uses_cargo_lane: bool,
    ) -> str:
        sealed_manifest = (
            ticket.source_manifest
            if self.tickets.source_is_sealed(ticket.ticket_id)
            else None
        )
        drift = (
            []
            if sealed_manifest is not None
            else self._manifest_drift(self.repo_root, ticket.source_manifest)
        )
        if drift:
            self._record_terminal(
                ticket,
                "snapshot_stale",
                evidence={
                    "phase": "materialization_recovery",
                    "jobId": previous_job_id,
                    "driftPaths": drift,
                },
                job_id=previous_job_id,
            )
            return "snapshot_stale"
        try:
            if uses_cargo_lane:
                external_sources = external_sources_from_coverage(ticket.coverage)
                record = self.workspace_copy.materialize_cargo_async(
                    ticket.session_id,
                    command=ticket.command,
                    overlay_paths=tuple(ticket.source_manifest),
                    external_sources=external_sources,
                    discover_external_sources=False,
                    sealed_overlay_manifest=sealed_manifest,
                    baseline_commit=ticket.base_head,
                )
            else:
                record = self.workspace_copy.materialize_validation_async(
                    ticket.session_id,
                    dependency_roots=self._dependency_roots(ticket),
                    overlay_paths=tuple(ticket.source_manifest),
                    sealed_overlay_manifest=sealed_manifest,
                    baseline_commit=ticket.base_head,
                )
        except Exception as error:
            self._terminal_error(
                ticket,
                "materialization_recovery",
                error,
                job_id=previous_job_id,
            )
            return "failed"
        self.tickets.record_worker_event(
            ticket.ticket_id,
            _COPY_LINK_EVENT,
            {"jobId": str(record.job_id), "recoveredFromJobId": previous_job_id},
        )
        return "materializing"

    def _advance_running(self, ticket: ValidationTicket) -> str:
        link = self.tickets.latest_worker_event(ticket.ticket_id, _RUN_LINK_EVENT)
        job_id = str((link or {}).get("jobId") or "")
        if not job_id:
            self._record_terminal(
                ticket,
                "failed",
                evidence={
                    "phase": "run_recovery",
                    "errorCode": "validation_ticket_run_link_missing",
                },
            )
            return "failed"
        return self._finish_from_run(ticket, job_id)

    def _finish_from_run(self, ticket: ValidationTicket, job_id: str) -> str:
        run = self.run_result_lookup(ticket.ticket_id)
        if run is not None:
            exit_code = int(run.get("exitCode", run.get("exit_code", 1)))
            status = "passed" if exit_code == 0 else "failed"
            self._record_terminal(
                ticket,
                status,
                evidence=self._run_evidence(run, ticket.ticket_id, job_id, exit_code),
                job_id=job_id,
            )
            return status
        try:
            record = self.workspace_copy.status(ticket.session_id, job_id)
        except Exception as error:
            self._terminal_error(ticket, "run_status", error, job_id=job_id)
            return "failed"
        if str(record.status) == "running":
            return "running"
        if str(record.status) == "materialized":
            try:
                progress = self.workspace_copy.start(
                    ticket.session_id,
                    job_id,
                    command=ticket.command,
                    run_id=ticket.ticket_id,
                )
            except Exception as error:
                self._terminal_error(ticket, "run_restart", error, job_id=job_id)
                return "failed"
            if str(progress.get("status") or "") == "waiting":
                # This branch recovers a ticket already durably projected as running;
                # preserve that projection until its exact reservation can start.
                return "running"
            return "running"
        self._record_terminal(
            ticket,
            "failed",
            evidence={
                "phase": "run_status",
                "errorCode": "validation_ticket_run_terminal_missing",
                "jobId": job_id,
                "copyStatus": str(record.status),
            },
            job_id=job_id,
        )
        return "failed"

    def _link_running(self, ticket: ValidationTicket, job_id: str) -> None:
        self.tickets.record_worker_event(
            ticket.ticket_id,
            _RUN_LINK_EVENT,
            {"jobId": job_id, "runId": ticket.ticket_id},
        )
        self.tickets.transition(
            ticket.ticket_id,
            "running",
            evidence={"jobId": job_id, "runId": ticket.ticket_id},
        )

    def _run_result(self, run_id: str) -> Mapping[str, object] | None:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM validation_copy_runs WHERE run_id=?", (run_id,)
            ).fetchone()
        return dict(row) if row is not None else None

    def _terminal_error(
        self,
        ticket: ValidationTicket,
        phase: str,
        error: Exception,
        *,
        job_id: str | None = None,
    ) -> None:
        evidence: dict[str, object] = {
            "phase": phase,
            "errorCode": getattr(error, "code", type(error).__name__),
            "error": str(error)[-4096:],
        }
        details = getattr(error, "details", None)
        if isinstance(details, Mapping) and details:
            evidence["errorDetails"] = dict(details)
        if job_id:
            evidence["jobId"] = job_id
        self._record_terminal(ticket, "failed", evidence=evidence, job_id=job_id)

    def _record_terminal(
        self,
        ticket: ValidationTicket,
        status: str,
        *,
        evidence: Mapping[str, object],
        job_id: str | None = None,
    ) -> None:
        self.tickets.record_result(ticket.ticket_id, status, evidence=evidence)
        if not job_id:
            return
        try:
            self.workspace_copy.cleanup_terminal_ticket_copy(ticket.ticket_id, job_id)
        except Exception:
            # Cargo release can trail ticket projection; periodic recovery retries
            # only terminal tickets with an explicit copy link.
            return

    @staticmethod
    def _manifest_drift(
        root: Path, manifest: Mapping[str, str | None]
    ) -> list[str]:
        drift: list[str] = []
        for relative, expected in manifest.items():
            source = root / relative
            if expected is None:
                # A tombstone matches only an absent directory entry, including a dangling link.
                drifted = os.path.lexists(source)
            else:
                actual = hashlib.sha256(source.read_bytes()).hexdigest() if source.is_file() else None
                drifted = actual != expected
            if drifted:
                drift.append(relative)
                if len(drift) == 64:
                    break
        return drift

    @classmethod
    def _uses_cargo_lane(cls, ticket: ValidationTicket) -> bool:
        return validation_uses_cargo_lane(ticket.command, ticket.toolchain)

    @staticmethod
    def _dependency_roots(ticket: ValidationTicket) -> tuple[str, ...]:
        return validation_dependency_roots(ticket.coverage)

    @staticmethod
    def _copy_failure(record, job_id: str) -> dict[str, object]:
        details = getattr(record, "error_details", None)
        return {
            "phase": "materialization",
            "jobId": job_id,
            "errorCode": str(record.error_code or "validation_copy_failed"),
            "errorStage": record.error_stage,
            "errorPath": record.error_path,
            "errorDetails": dict(details) if isinstance(details, Mapping) else {},
        }

    @staticmethod
    def _run_evidence(
        run: Mapping[str, object], run_id: str, job_id: str, exit_code: int
    ) -> dict[str, object]:
        stdout = str(run.get("stdout", run.get("stdout_text", "")) or "")
        stderr = str(run.get("stderr", run.get("stderr_text", "")) or "")
        return {
            "phase": "run",
            "jobId": job_id,
            "runId": run_id,
            "exitCode": exit_code,
            "stdoutTail": stdout[-4096:],
            "stderrTail": stderr[-4096:],
        }

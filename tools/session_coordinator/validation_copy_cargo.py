from __future__ import annotations

import hashlib
import json
import os
import platform
from pathlib import Path
from typing import Callable, Mapping

from .cargo_jobs import CargoCompatibility, CargoJobService, CargoLaneKind
from .cargo_runner import CargoJobRunner
from .database import Database
from .models import CoordinatorError

_RETRYABLE_ADMISSION_ERRORS = frozenset(
    {
        "cargo_cpu_reservation_not_fifo_head",
        "cargo_cpu_lane_reserved",
        "cargo_lane_occupied",
        "cargo_reuse_pool_busy",
        "cargo_process_tree_alive",
        "cargo_lane_cleanup_reserved",
    }
)


class ValidationCopyCargoExecution:
    """Bind one materialized validation copy to the managed Cargo lane."""

    def __init__(
        self,
        database: Database,
        cargo_jobs: CargoJobService,
        cargo_runner: CargoJobRunner,
        *,
        preflight: Callable[[], None] | None = None,
        reservation_lookup: Callable[[str, str], Mapping[str, object] | None]
        | None = None,
        source_manifest_lookup: Callable[[str, str], Mapping[str, str] | None]
        | None = None,
    ) -> None:
        self.database = database
        self.cargo_jobs = cargo_jobs
        self.cargo_runner = cargo_runner
        self.preflight = preflight
        self.reservation_lookup = reservation_lookup or self._latest_reservation
        self._source_manifest_required = (
            source_manifest_lookup is not None or reservation_lookup is None
        )
        if source_manifest_lookup is not None:
            self.source_manifest_lookup = source_manifest_lookup
        elif reservation_lookup is None:
            self.source_manifest_lookup = self._validation_source_manifest
        else:
            # Existing state-machine tests inject reservation state without a DB.
            # The real service never replaces only this half of the durable lookup.
            self.source_manifest_lookup = lambda _session_id, _run_id: None

    def advance(
        self,
        *,
        session_id: str,
        copy_job_id: str,
        source_root: Path,
        input_manifest_hash: str | None,
        command: tuple[str, ...],
        validation_run_id: str,
    ) -> dict[str, object]:
        source_manifest = self.source_manifest_lookup(session_id, validation_run_id)
        if not source_manifest and self._source_manifest_required:
            raise CoordinatorError(
                "validation_copy_cargo_source_manifest_missing",
                "Managed Cargo execution requires the validation ticket source manifest",
            )
        compatibility = self._compatibility(
            copy_job_id=copy_job_id,
            input_manifest_hash=input_manifest_hash,
            source_manifest=source_manifest,
            command=command,
            validation_run_id=validation_run_id,
        )
        reservation = self.reservation_lookup(session_id, copy_job_id)
        if reservation is None or (
            not reservation.get("jobId")
            and str(reservation.get("status") or "") != "pending"
        ):
            reservation = self.cargo_jobs.reserve_cpu(
                session_id,
                compatibility=compatibility,
                command=command,
                ttl_seconds=900,
                burst_eligible=False,
            )
        reservation_id = str(reservation.get("reservationId") or "")
        if not reservation_id:
            raise CoordinatorError(
                "validation_copy_cargo_reservation_invalid",
                "Managed Cargo reservation did not return an identity",
            )
        cargo_job_id = str(reservation.get("jobId") or "")
        if not cargo_job_id:
            if self.preflight is not None:
                self.preflight()
            try:
                job = self.cargo_jobs.acquire(
                    session_id,
                    CargoLaneKind.WORKSPACE,
                    compatibility=compatibility,
                    expected_cpu_reservation_id=reservation_id,
                )
            except CoordinatorError as error:
                if error.code not in _RETRYABLE_ADMISSION_ERRORS:
                    raise
                return self._progress(
                    "waiting", reservation_id=reservation_id, blocker=error.code
                )
            cargo_job_id = str(job.job_id)
        else:
            job = self.cargo_jobs.get(cargo_job_id)

        job_status = self._status_value(job.status)
        if job_status == "leased":
            run = self.cargo_runner.start(
                session_id=session_id,
                job_id=cargo_job_id,
                command=command,
                working_directory=source_root,
            )
            return self._progress(
                "running",
                reservation_id=reservation_id,
                cargo_job_id=cargo_job_id,
                cargo_run_id=str(run.run_id),
                pid=int(run.pid) if run.pid is not None else None,
            )
        if job_status not in {
            "running",
            "succeeded",
            "failed",
            "released",
            "orphaned",
        }:
            raise CoordinatorError(
                "validation_copy_cargo_job_state_invalid",
                f"Managed Cargo job {cargo_job_id} is {job_status}",
            )

        run = self.cargo_runner.status(cargo_job_id, session_id=session_id)
        run_status = str(run.get("status") or "")
        if run_status == "running":
            return self._progress(
                "running",
                reservation_id=reservation_id,
                cargo_job_id=cargo_job_id,
                cargo_run_id=str(run.get("runId") or ""),
                pid=int(run["pid"]) if run.get("pid") is not None else None,
            )
        if run_status not in {"completed", "finish_blocked", "launch_failed"}:
            raise CoordinatorError(
                "validation_copy_cargo_run_state_invalid",
                f"Managed Cargo run for {cargo_job_id} is {run_status or 'unknown'}",
            )
        exit_code = run.get("exitCode")
        return {
            **self._progress(
                "completed",
                reservation_id=reservation_id,
                cargo_job_id=cargo_job_id,
                cargo_run_id=str(run.get("runId") or ""),
            ),
            "exitCode": int(exit_code) if exit_code is not None else -1,
            "stdoutTail": str(run.get("stdoutTail") or ""),
            "stderrTail": str(run.get("stderrTail") or ""),
            "startedAt": str(run.get("startedAt") or ""),
            "completedAt": str(run.get("completedAt") or ""),
            "cargoRunStatus": run_status,
            "cargoErrorCode": run.get("errorCode"),
        }

    def _latest_reservation(
        self, session_id: str, copy_job_id: str
    ) -> Mapping[str, object] | None:
        with self.database.connect() as connection:
            row = connection.execute(
                """SELECT reservation_id, status, job_id
                   FROM cargo_lane_reservations
                   WHERE lane_scope='cpu' AND session_id=? AND source_copy_job_id=?
                   ORDER BY created_at DESC, reservation_id DESC LIMIT 1""",
                (session_id, copy_job_id),
            ).fetchone()
        if row is None:
            return None
        return {
            "reservationId": str(row["reservation_id"]),
            "status": str(row["status"]),
            "jobId": str(row["job_id"]) if row["job_id"] else None,
        }

    def _validation_source_manifest(
        self, session_id: str, validation_run_id: str
    ) -> Mapping[str, str] | None:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT source_manifest_json FROM validation_tickets "
                "WHERE ticket_id=? AND session_id=?",
                (validation_run_id, session_id),
            ).fetchone()
        if row is None:
            return None
        try:
            manifest = json.loads(str(row["source_manifest_json"]))
        except (TypeError, ValueError, json.JSONDecodeError) as error:
            raise CoordinatorError(
                "validation_copy_cargo_source_manifest_invalid",
                "Validation ticket source manifest is not valid JSON",
            ) from error
        if not isinstance(manifest, dict) or not manifest:
            raise CoordinatorError(
                "validation_copy_cargo_source_manifest_invalid",
                "Validation ticket source manifest must be a non-empty object",
            )
        return manifest

    @staticmethod
    def _compatibility(
        *,
        copy_job_id: str,
        input_manifest_hash: str | None,
        source_manifest: Mapping[str, str] | None,
        command: tuple[str, ...],
        validation_run_id: str,
    ) -> CargoCompatibility:
        digest = str(input_manifest_hash or "").strip()
        if len(digest) != 64 or any(character not in "0123456789abcdefABCDEF" for character in digest):
            raise CoordinatorError(
                "validation_copy_cargo_manifest_invalid",
                "Managed Cargo execution requires the immutable copy manifest identity",
            )
        command_payload = json.dumps(command, separators=(",", ":"), ensure_ascii=True)
        build_config = json.dumps(
            {
                "command_sha256": hashlib.sha256(command_payload.encode("utf-8")).hexdigest(),
                "validation_run_id": validation_run_id,
            },
            sort_keys=True,
            separators=(",", ":"),
        )
        return CargoCompatibility(
            platform="windows" if os.name == "nt" else "wsl",
            toolchain="managed-validation-copy",
            target_architecture=platform.machine() or "unknown",
            workspace="validation-copy",
            build_config=build_config,
            source_manifest=source_manifest,
            source_copy_job_id=copy_job_id,
            source_copy_manifest_hash=digest,
        )

    @staticmethod
    def _status_value(status: object) -> str:
        return str(getattr(status, "value", status))

    @staticmethod
    def _progress(
        status: str,
        *,
        reservation_id: str,
        cargo_job_id: str | None = None,
        cargo_run_id: str | None = None,
        pid: int | None = None,
        blocker: str | None = None,
    ) -> dict[str, object]:
        return {
            "status": status,
            "cargoReservationId": reservation_id,
            "cargoJobId": cargo_job_id,
            "cargoRunId": cargo_run_id,
            "pid": pid,
            "blockerCode": blocker,
        }

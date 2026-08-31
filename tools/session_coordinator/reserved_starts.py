from __future__ import annotations

import json
import threading
from collections.abc import Callable
from datetime import UTC, datetime, timedelta
from sqlite3 import Connection, IntegrityError

from .cargo_jobs import CargoJobService, reservation_code
from .cargo_reservations import require_executable_cargo_session
from .cargo_runner import CargoJobRunner
from .database import Database
from .models import CoordinatorError, parse_utc, utc_text


ProofGuard = Callable[[Connection, str, str, str], None]
AdmissionGuard = Callable[[Connection, str, str], None]
Scheduler = Callable[[Callable[[], None]], object]


def _thread_scheduler(callback: Callable[[], None]) -> threading.Thread:
    worker = threading.Thread(target=callback, name="zircon-cargo-start", daemon=True)
    worker.start()
    return worker


class ReservedCargoStartService:
    """Own the durable interval between exact-pair admission and process spawn."""

    def __init__(
        self,
        database: Database,
        cargo_jobs: CargoJobService,
        runner: CargoJobRunner,
        *,
        proof_guard: ProofGuard,
        admission_guard: AdmissionGuard | None = None,
        scheduler: Scheduler = _thread_scheduler,
        start_deadline_seconds: int = 900,
    ):
        if start_deadline_seconds <= 300:
            raise ValueError("Reserved Cargo start deadline must exceed the leased watchdog")
        self.database = database
        self.cargo_jobs = cargo_jobs
        self.runner = runner
        self.proof_guard = proof_guard
        self.admission_guard = admission_guard
        self.scheduler = scheduler
        self.start_deadline_seconds = start_deadline_seconds
        self._launch_transition_lock = threading.RLock()

    def accept(
        self,
        *,
        request_id: str,
        reservation_id: str,
        job_id: str,
        session_id: str,
        command: tuple[str, ...] | list[str],
        admission_checkpoint: str | None = None,
    ) -> dict[str, object]:
        with self.database.transaction() as connection:
            row, created = self.admit_in_connection(
                connection,
                request_id=request_id,
                reservation_id=reservation_id,
                job_id=job_id,
                session_id=session_id,
                command=command,
                admission_checkpoint=admission_checkpoint,
            )
        if created:
            self.schedule(request_id)
        return row

    def admit_in_connection(
        self,
        connection: Connection,
        *,
        request_id: str,
        reservation_id: str,
        job_id: str,
        session_id: str,
        command: tuple[str, ...] | list[str],
        admission_checkpoint: str | None = None,
    ) -> tuple[dict[str, object], bool]:
        command_tuple = tuple(str(part) for part in command)
        if not command_tuple or any(not part for part in command_tuple):
            raise CoordinatorError("cargo_run_command_empty", "Managed Cargo command cannot be empty")
        self.cargo_jobs._reject_coordinator_output_flags(command_tuple)
        acknowledged_at = utc_text()
        deadline_at = utc_text(
            parse_utc(acknowledged_at) + timedelta(seconds=self.start_deadline_seconds)
        )
        existing = connection.execute(
            "SELECT * FROM cargo_start_requests WHERE request_id=?", (request_id,)
        ).fetchone()
        if existing is not None:
            self._require_same_request(
                existing,
                reservation_id=reservation_id,
                job_id=job_id,
                session_id=session_id,
                command=command_tuple,
            )
            return self._to_dict(existing), False
        other = connection.execute(
            "SELECT request_id FROM cargo_start_requests WHERE job_id=? OR reservation_id=?",
            (job_id, reservation_id),
        ).fetchone()
        if other is not None:
            raise CoordinatorError(
                "cargo_start_request_exists",
                "The reserved Cargo pair already has a durable start request",
                details={"requestId": other["request_id"], "jobId": job_id},
            )
        self.cargo_jobs.require_cargo_start_admission_in_connection(
            connection, f"cargo.run_reserved@{session_id}"
        )
        require_executable_cargo_session(connection, session_id)
        reservation = connection.execute(
            "SELECT * FROM cargo_lane_reservations WHERE reservation_id=?",
            (reservation_id,),
        ).fetchone()
        if reservation is None:
            raise CoordinatorError(
                "cargo_reservation_not_found", f"Unknown Cargo reservation {reservation_id}"
            )
        lane_scope = str(reservation["lane_scope"])
        job = connection.execute("SELECT * FROM cargo_jobs WHERE job_id=?", (job_id,)).fetchone()
        if (
            reservation["session_id"] != session_id
            or reservation["status"] != "leased"
            or reservation["job_id"] != job_id
            or job is None
            or job["session_id"] != session_id
            or job["status"] != "leased"
            or job["pid"] is not None
            or job["started_at"] is not None
        ):
            raise CoordinatorError(
                reservation_code(lane_scope, "binding_invalid"),
                f"{lane_scope.upper()} reservation is not bound to the requested unstarted job",
                details={"reservationId": reservation_id, "jobId": job_id},
            )
        if reservation["command_fingerprint"] != self.cargo_jobs._command_fingerprint(command_tuple):
            raise CoordinatorError(
                reservation_code(lane_scope, "command_mismatch"),
                f"The reserved {lane_scope.upper()} job must run its exact approved command",
                details={"reservationId": reservation_id},
            )
        if admission_checkpoint is not None and self.admission_guard is not None:
            self.admission_guard(
                connection,
                f"cargo.run_reserved@{session_id}",
                admission_checkpoint,
            )
        self.proof_guard(connection, reservation_id, session_id, job_id)
        try:
            connection.execute(
                """
                INSERT INTO cargo_start_requests(
                    request_id, reservation_id, job_id, session_id, command_json,
                    status, acknowledged_at, deadline_at
                ) VALUES (?, ?, ?, ?, ?, 'start_pending', ?, ?)
                """,
                (
                    request_id,
                    reservation_id,
                    job_id,
                    session_id,
                    json.dumps(command_tuple, ensure_ascii=False),
                    acknowledged_at,
                    deadline_at,
                ),
            )
        except IntegrityError as error:
            raise CoordinatorError(
                "cargo_start_request_exists",
                "The reserved Cargo pair already has a durable start request",
                details={"requestId": request_id, "jobId": job_id},
            ) from error
        self._record_event(
            connection,
            session_id,
            "cargo.start_pending",
            {
                "requestId": request_id,
                "reservationId": reservation_id,
                "jobId": job_id,
                "deadlineAt": deadline_at,
            },
        )
        row = connection.execute(
            "SELECT * FROM cargo_start_requests WHERE request_id=?", (request_id,)
        ).fetchone()
        return self._to_dict(row), True

    def schedule(self, request_id: str) -> dict[str, object]:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM cargo_start_requests WHERE request_id=?", (request_id,)
            ).fetchone()
        if row is None:
            raise CoordinatorError(
                "cargo_start_request_not_found", f"Unknown Cargo start request {request_id}"
            )
        if row["status"] != "start_pending":
            return self._to_dict(row)
        command = tuple(json.loads(row["command_json"]))
        try:
            self.scheduler(
                lambda: self._launch(
                    request_id=request_id,
                    reservation_id=str(row["reservation_id"]),
                    job_id=str(row["job_id"]),
                    session_id=str(row["session_id"]),
                    command=command,
                )
            )
        except BaseException as error:
            self._mark_launch_failed(request_id, error)
        return self.status(request_id)

    def status(self, request_id: str) -> dict[str, object]:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM cargo_start_requests WHERE request_id=?", (request_id,)
            ).fetchone()
        if row is None:
            raise CoordinatorError(
                "cargo_start_request_not_found", f"Unknown Cargo start request {request_id}"
            )
        return self._to_dict(row)

    def reconcile_expired(self, *, now: datetime | None = None) -> tuple[str, ...]:
        current = now or datetime.now(UTC)
        now_text = utc_text(current)
        with self.database.connect() as connection:
            request_ids = tuple(
                row["request_id"]
                for row in connection.execute(
                    """
                    SELECT request_id FROM cargo_start_requests
                    WHERE status='start_pending' AND deadline_at<=?
                    ORDER BY deadline_at, request_id
                    """,
                    (now_text,),
                )
            )
        expired: list[str] = []
        for request_id in request_ids:
            if self._mark_launch_failed(
                request_id,
                CoordinatorError(
                    "cargo_launch_deadline_exceeded",
                    "Reserved Cargo launch did not register a process before its deadline",
                ),
                now=current,
                recover_registered=True,
            ):
                expired.append(request_id)
        return tuple(expired)

    def reconcile_interrupted(self, *, now: datetime | None = None) -> tuple[str, ...]:
        """Close predecessor-owned pending acknowledgements without replaying Cargo."""
        current = now or datetime.now(UTC)
        with self.database.connect() as connection:
            request_ids = tuple(
                str(row["request_id"])
                for row in connection.execute(
                    """
                    SELECT request_id FROM cargo_start_requests
                    WHERE status='start_pending' AND deadline_at>?
                    ORDER BY acknowledged_at, request_id
                    """,
                    (utc_text(current),),
                )
            )
        interrupted: list[str] = []
        for request_id in request_ids:
            if self._mark_launch_failed(
                request_id,
                CoordinatorError(
                    "cargo_launch_interrupted_before_spawn",
                    "Coordinator restarted before the acknowledged Cargo launch registered a process",
                ),
                now=current,
                recover_registered=True,
            ):
                interrupted.append(request_id)
        return tuple(interrupted)

    def _launch(
        self,
        *,
        request_id: str,
        reservation_id: str,
        job_id: str,
        session_id: str,
        command: tuple[str, ...],
    ) -> None:
        try:
            context = self.cargo_jobs.reserved_run_context(
                reservation_id,
                session_id=session_id,
                job_id=job_id,
                command=command,
            )
        except BaseException as error:
            self._mark_launch_failed(request_id, error)
            return
        with self._launch_transition_lock:
            with self.database.connect() as connection:
                pending = connection.execute(
                    "SELECT status, deadline_at FROM cargo_start_requests WHERE request_id=?",
                    (request_id,),
                ).fetchone()
            if pending is None or pending["status"] != "start_pending":
                return
            current = datetime.now(UTC)
            if parse_utc(pending["deadline_at"]) <= current:
                self._mark_launch_failed(
                    request_id,
                    CoordinatorError(
                        "cargo_launch_deadline_exceeded",
                        "Reserved Cargo launch did not register a process before its deadline",
                    ),
                    now=current,
                    recover_registered=True,
                )
                return
            try:
                run = self.runner.start(
                    session_id=session_id,
                    job_id=job_id,
                    command=command,
                    environment=context.environment,
                    working_directory=context.working_directory,
                )
            except BaseException as error:
                self._mark_launch_failed(request_id, error)
                return
            completed_at = utc_text()
            with self.database.transaction() as connection:
                cursor = connection.execute(
                    """
                    UPDATE cargo_start_requests
                    SET status='started', run_id=?, completed_at=?
                    WHERE request_id=? AND status='start_pending'
                    """,
                    (run.run_id, completed_at, request_id),
                )
                if cursor.rowcount == 1:
                    self._record_event(
                        connection,
                        session_id,
                        "cargo.start_registered",
                        {"requestId": request_id, "jobId": job_id, "runId": run.run_id},
                    )

    def _mark_launch_failed(
        self,
        request_id: str,
        error: BaseException,
        *,
        now: datetime | None = None,
        recover_registered: bool = False,
    ) -> bool:
        with self._launch_transition_lock:
            completed_at = utc_text(now)
            code = error.code if isinstance(error, CoordinatorError) else "cargo_launch_failed"
            message = error.message if isinstance(error, CoordinatorError) else str(error)
            with self.database.transaction() as connection:
                row = connection.execute(
                    "SELECT * FROM cargo_start_requests WHERE request_id=?", (request_id,)
                ).fetchone()
                if row is None or row["status"] != "start_pending":
                    return False
                registered_run = connection.execute(
                    """
                    SELECT run_id, status, error_code FROM cargo_job_runs
                    WHERE job_id=? AND session_id=?
                    ORDER BY started_at DESC, run_id DESC LIMIT 1
                    """,
                    (row["job_id"], row["session_id"]),
                ).fetchone()
                job = connection.execute(
                    "SELECT status, pid, started_at FROM cargo_jobs WHERE job_id=?",
                    (row["job_id"],),
                ).fetchone()
                if (
                    recover_registered
                    and registered_run is not None
                    and registered_run["status"] != "launch_failed"
                    and registered_run["error_code"]
                    != "cargo_run_suspended_before_resume"
                    and job is not None
                    and job["pid"] is not None
                    and job["started_at"] is not None
                ):
                    connection.execute(
                        """
                        UPDATE cargo_start_requests
                        SET status='started', run_id=?, completed_at=?
                        WHERE request_id=? AND status='start_pending'
                        """,
                        (registered_run["run_id"], completed_at, request_id),
                    )
                    self._record_event(
                        connection,
                        str(row["session_id"]),
                        "cargo.start_registered",
                        {
                            "requestId": request_id,
                            "jobId": row["job_id"],
                            "runId": registered_run["run_id"],
                            "reconciled": True,
                        },
                    )
                    return False
                interrupted_after_spawn_identity = (
                    registered_run is None
                    and job is not None
                    and job["status"] == "running"
                    and job["pid"] is not None
                )
                if interrupted_after_spawn_identity:
                    code = "cargo_launch_interrupted_after_spawn_identity"
                    message = (
                        "Coordinator restarted after a Cargo process identity was registered "
                        "but before its managed run projection was committed"
                    )
                cleanup_unproven = (
                    code == "cargo_launch_cleanup_unproven"
                    or interrupted_after_spawn_identity
                )
                if registered_run is not None and not cleanup_unproven:
                    connection.execute(
                        """
                        UPDATE cargo_job_runs
                        SET status='launch_failed', error_code=?, completed_at=?
                        WHERE run_id=? AND status='running'
                        """,
                        (code, completed_at, registered_run["run_id"]),
                    )
                connection.execute(
                    """
                    UPDATE cargo_start_requests
                    SET status='launch_failed', error_code=?, error_message=?, completed_at=?
                    WHERE request_id=? AND status='start_pending'
                    """,
                    (code, message, completed_at, request_id),
                )
                if not cleanup_unproven:
                    connection.execute(
                        """
                        UPDATE cargo_jobs
                        SET status='released', finished_at=COALESCE(finished_at, ?),
                            released_at=COALESCE(released_at, ?), last_heartbeat_at=?
                        WHERE job_id=? AND status IN ('leased', 'running')
                        """,
                        (completed_at, completed_at, completed_at, row["job_id"]),
                    )
                    connection.execute(
                        """
                        UPDATE cargo_lane_reservations
                        SET status='expired', completed_at=COALESCE(completed_at, ?)
                        WHERE reservation_id=? AND status IN ('leased', 'running')
                        """,
                        (completed_at, row["reservation_id"]),
                    )
                self._record_event(
                    connection,
                    str(row["session_id"]),
                    "cargo.launch_failed",
                    {
                        "requestId": request_id,
                        "reservationId": row["reservation_id"],
                        "jobId": row["job_id"],
                        "errorCode": code,
                        "errorMessage": message,
                        "cleanupProven": not cleanup_unproven,
                    },
                )
            return True

    @staticmethod
    def _require_same_request(
        row,
        *,
        reservation_id: str,
        job_id: str,
        session_id: str,
        command: tuple[str, ...],
    ) -> None:
        if (
            row["reservation_id"] != reservation_id
            or row["job_id"] != job_id
            or row["session_id"] != session_id
            or tuple(json.loads(row["command_json"])) != command
        ):
            raise CoordinatorError(
                "cargo_start_request_conflict",
                "request_id is already bound to a different reserved Cargo start",
                details={"requestId": row["request_id"]},
            )

    @staticmethod
    def _record_event(
        connection: Connection,
        session_id: str,
        event_type: str,
        payload: dict[str, object],
    ) -> None:
        connection.execute(
            "INSERT INTO events(session_id, event_type, payload_json, created_at) VALUES (?, ?, ?, ?)",
            (session_id, event_type, json.dumps(payload, sort_keys=True), utc_text()),
        )

    @staticmethod
    def _to_dict(row) -> dict[str, object]:
        return {
            "requestId": row["request_id"],
            "reservationId": row["reservation_id"],
            "jobId": row["job_id"],
            "sessionId": row["session_id"],
            "status": row["status"],
            "acknowledgedAt": row["acknowledged_at"],
            "deadlineAt": row["deadline_at"],
            "runId": row["run_id"],
            "errorCode": row["error_code"],
            "errorMessage": row["error_message"],
            "completedAt": row["completed_at"],
        }

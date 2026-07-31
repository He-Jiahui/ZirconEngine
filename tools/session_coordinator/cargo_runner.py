from __future__ import annotations

import json
import os
import subprocess
import threading
import time
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Mapping

from .cargo_jobs import CargoJobService
from .database import Database
from .models import CoordinatorError, utc_text


MAX_LOG_TAIL_BYTES = 64 * 1024
_ALLOWED_ENVIRONMENT_KEYS = frozenset({"RUSTFLAGS", "CARGO_INCREMENTAL", "CARGO_BUILD_JOBS"})


@dataclass(frozen=True, slots=True)
class CargoRun:
    run_id: str
    job_id: str
    session_id: str
    status: str
    pid: int | None
    stdout_path: str
    stderr_path: str

    def to_dict(self) -> dict[str, object]:
        return {
            "runId": self.run_id,
            "jobId": self.job_id,
            "sessionId": self.session_id,
            "status": self.status,
            "pid": self.pid,
            "stdoutPath": self.stdout_path,
            "stderrPath": self.stderr_path,
        }


class CargoJobRunner:
    """Owns managed Cargo child processes so caller lifetime cannot orphan them."""

    def __init__(
        self,
        database: Database,
        cargo_jobs: CargoJobService,
        *,
        repo_root: str | Path,
        log_root: str | Path,
        popen: Callable[..., subprocess.Popen] = subprocess.Popen,
    ):
        self.database = database
        self.cargo_jobs = cargo_jobs
        self.repo_root = Path(repo_root).resolve()
        self.log_root = Path(log_root).resolve()
        self.popen = popen
        self._running_lock = threading.Lock()
        self._running: dict[str, subprocess.Popen] = {}

    def start(
        self,
        *,
        session_id: str,
        job_id: str,
        command: tuple[str, ...] | list[str],
        environment: Mapping[str, str] | None = None,
        working_directory: str | Path | None = None,
    ) -> CargoRun:
        command_tuple = tuple(str(part) for part in command if str(part))
        if not command_tuple:
            raise CoordinatorError("cargo_run_command_empty", "Managed Cargo command cannot be empty")
        environment_values = self._validate_environment(environment)
        working_root = (
            Path(working_directory).resolve()
            if working_directory is not None
            else self.repo_root
        )
        if not working_root.is_dir():
            raise CoordinatorError(
                "cargo_run_source_root_invalid",
                "Managed Cargo source root must be an existing directory",
                details={"sourceRoot": str(working_root)},
            )
        job = self.cargo_jobs.get(job_id)
        if job.session_id != session_id:
            raise CoordinatorError("cargo_job_owner_mismatch", f"Cargo job {job_id} belongs to another Session")
        if job.status.value != "leased":
            raise CoordinatorError(
                "invalid_cargo_job_status",
                f"Cargo job {job_id} is {job.status.value}; expected ['leased']",
            )
        run_id = uuid.uuid4().hex
        run_root = self.log_root / job_id / run_id
        run_root.mkdir(parents=True, exist_ok=False)
        stdout_path = run_root / "stdout.log"
        stderr_path = run_root / "stderr.log"
        started_at = utc_text()
        process: subprocess.Popen | None = None
        try:
            with stdout_path.open("w", encoding="utf-8", errors="replace") as stdout_file, stderr_path.open(
                "w", encoding="utf-8", errors="replace"
            ) as stderr_file:
                environment = os.environ.copy()
                environment["CARGO_TARGET_DIR"] = job.target_dir
                environment.update(environment_values)
                process = self.popen(
                    command_tuple,
                    cwd=working_root,
                    env=environment,
                    stdout=stdout_file,
                    stderr=stderr_file,
                    text=True,
                )
            self.cargo_jobs.start(
                job_id,
                session_id=session_id,
                pid=int(process.pid),
                command=command_tuple,
                root_is_supervisor=True,
            )
            with self.database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO cargo_job_runs(
                        run_id, job_id, session_id, command_json, environment_json, status,
                        stdout_path, stderr_path, started_at
                    ) VALUES (?, ?, ?, ?, ?, 'running', ?, ?, ?)
                    """,
                    (
                        run_id,
                        job_id,
                        session_id,
                        json.dumps(command_tuple),
                        json.dumps(environment_values, sort_keys=True),
                        str(stdout_path),
                        str(stderr_path),
                        started_at,
                    ),
                )
            with self._running_lock:
                self._running[job_id] = process
            threading.Thread(
                target=self._finish,
                args=(run_id, job_id, session_id, process, stdout_path, stderr_path),
                name=f"zircon-cargo-{job_id[:12]}",
                daemon=True,
            ).start()
        except BaseException:
            cleanup_error: BaseException | None = None
            if process is not None:
                try:
                    if process.poll() is None:
                        process.kill()
                        process.wait(timeout=5)
                except BaseException as error:
                    cleanup_error = error
            if cleanup_error is not None and process is not None:
                with self._running_lock:
                    self._running[job_id] = process
                raise CoordinatorError(
                    "cargo_launch_cleanup_unproven",
                    "Spawned Cargo process could not be confirmed stopped after launch setup failed",
                    details={"jobId": job_id, "pid": int(process.pid)},
                ) from cleanup_error
            with self._running_lock:
                self._running.pop(job_id, None)
            raise
        return CargoRun(run_id, job_id, session_id, "running", process.pid, str(stdout_path), str(stderr_path))

    def status(self, job_id: str, *, session_id: str) -> dict[str, object]:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM cargo_job_runs WHERE job_id=?", (job_id,)
            ).fetchone()
        if row is None:
            raise CoordinatorError("cargo_run_not_found", f"No managed run for Cargo job {job_id}")
        if row["session_id"] != session_id:
            raise CoordinatorError("cargo_job_owner_mismatch", f"Cargo job {job_id} belongs to another Session")
        if row["status"] == "running":
            self.reconcile_terminal_runs(job_id=job_id)
            with self.database.connect() as connection:
                row = connection.execute(
                    "SELECT * FROM cargo_job_runs WHERE job_id=?", (job_id,)
                ).fetchone()
            if row is None:
                raise CoordinatorError("cargo_run_not_found", f"No managed run for Cargo job {job_id}")
        return {
            "runId": row["run_id"],
            "jobId": row["job_id"],
            "sessionId": row["session_id"],
            "status": row["status"],
            "exitCode": row["exit_code"],
            "stdoutPath": row["stdout_path"],
            "stderrPath": row["stderr_path"],
            "stdoutTail": row["stdout_tail"],
            "stderrTail": row["stderr_tail"],
            "environment": json.loads(row["environment_json"]),
            "errorCode": row["error_code"],
            "startedAt": row["started_at"],
            "completedAt": row["completed_at"],
        }

    def reconcile_terminal_runs(self, *, job_id: str | None = None) -> tuple[str, ...]:
        """Close stale run projections after their Cargo job reached a proven terminal state.

        The raw stdout/stderr files remain immutable.  This only repairs the
        database projection when a process supervisor vanished after the job
        itself was safely finished and released.
        """

        with self.database.connect() as connection:
            rows = connection.execute(
                """SELECT run.run_id, run.stdout_path, run.stderr_path,
                          job.status AS job_status, job.exit_code, job.finished_at,
                          job.released_at, job.process_tree_live_pids_json
                   FROM cargo_job_runs AS run
                   JOIN cargo_jobs AS job ON job.job_id=run.job_id
                   WHERE run.status='running'
                     AND (
                        (job.status IN ('succeeded', 'failed', 'released')
                         AND job.exit_code IS NOT NULL)
                        OR (job.status='orphaned'
                            AND job.process_tree_live_pids_json='[]')
                        OR (job.status='released' AND job.exit_code IS NULL
                            AND job.process_tree_live_pids_json='[]')
                     )
                     AND (? IS NULL OR run.job_id=?)
                   ORDER BY run.started_at, run.run_id""",
                (job_id, job_id),
            ).fetchall()
        if not rows:
            return ()
        completed_at = utc_text()
        reconciled: list[str] = []
        with self.database.transaction() as connection:
            for row in rows:
                job_status = str(row["job_status"])
                if job_status == "orphaned":
                    error_code = "cargo_run_reconciled_from_orphaned_job"
                elif row["exit_code"] is None:
                    error_code = "cargo_run_reconciled_from_released_job_missing_exit_code"
                else:
                    error_code = "cargo_run_reconciled_from_terminal_job"
                updated = connection.execute(
                    """UPDATE cargo_job_runs
                       SET status='completed', exit_code=?, stdout_tail=?, stderr_tail=?,
                           error_code=?,
                           completed_at=?
                       WHERE run_id=? AND status='running'""",
                    (
                        int(row["exit_code"]) if row["exit_code"] is not None else None,
                        self._read_tail(Path(row["stdout_path"])),
                        self._read_tail(Path(row["stderr_path"])),
                        error_code,
                        row["released_at"] or row["finished_at"] or completed_at,
                        row["run_id"],
                    ),
                ).rowcount
                if updated:
                    reconciled.append(str(row["run_id"]))
        return tuple(reconciled)

    def _finish(
        self,
        run_id: str,
        job_id: str,
        session_id: str,
        process: subprocess.Popen,
        stdout_path: Path,
        stderr_path: Path,
    ) -> None:
        exit_code = int(process.wait())
        error_code: str | None = None
        status = "completed"
        try:
            # A wrapper can return before a Cargo child exits. Keep the runner,
            # not the originating shell, responsible for the final transition.
            finished = False
            for _ in range(120):
                try:
                    if not finished:
                        self.cargo_jobs.finish(job_id, session_id=session_id, exit_code=exit_code)
                        finished = True
                    self.cargo_jobs.release(job_id, session_id=session_id)
                    break
                except CoordinatorError as error:
                    if error.code != "cargo_process_tree_alive":
                        raise
                    if not finished:
                        self.cargo_jobs.heartbeat(job_id, session_id=session_id)
                    time.sleep(1)
            else:
                status = "finish_blocked"
                error_code = "cargo_process_tree_alive"
        except CoordinatorError as error:
            status = "finish_blocked"
            error_code = error.code
        finally:
            with self.database.transaction() as connection:
                connection.execute(
                    """
                    UPDATE cargo_job_runs
                    SET status=?, exit_code=?, stdout_tail=?, stderr_tail=?,
                        error_code=?, completed_at=?
                    WHERE run_id=?
                    """,
                    (
                        status,
                        exit_code,
                        self._read_tail(stdout_path),
                        self._read_tail(stderr_path),
                        error_code,
                        utc_text(),
                        run_id,
                    ),
                )
            with self._running_lock:
                self._running.pop(job_id, None)

    @staticmethod
    def _read_tail(path: Path) -> str:
        try:
            with path.open("rb") as stream:
                stream.seek(0, 2)
                size = stream.tell()
                stream.seek(max(0, size - MAX_LOG_TAIL_BYTES))
                return stream.read().decode("utf-8", errors="replace")
        except OSError:
            return ""

    @staticmethod
    def _validate_environment(environment: Mapping[str, str] | None) -> dict[str, str]:
        if environment is None:
            return {}
        values = dict(environment)
        unsupported = sorted(set(values) - _ALLOWED_ENVIRONMENT_KEYS)
        if unsupported:
            raise CoordinatorError(
                "cargo_run_environment_forbidden",
                f"Managed Cargo environment is limited to: {', '.join(sorted(_ALLOWED_ENVIRONMENT_KEYS))}",
                details={"keys": unsupported},
            )
        normalized: dict[str, str] = {}
        for key, value in values.items():
            if not isinstance(key, str) or not isinstance(value, str) or not value:
                raise CoordinatorError(
                    "cargo_run_environment_invalid",
                    "Managed Cargo environment values must be non-empty strings",
                )
            if len(value) > 4096 or any(character in value for character in ("\0", "\r", "\n")):
                raise CoordinatorError(
                    "cargo_run_environment_invalid",
                    f"Managed Cargo environment value for {key} is invalid",
                )
            normalized[key] = value
        return normalized

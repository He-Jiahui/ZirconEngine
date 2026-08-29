from __future__ import annotations

import json
import io
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
from .processes import popen_process_creation_time
from .windows_job_process import (
    close_process_job,
    create_atomic_kill_on_close_process,
    resume_popen_process,
    terminate_and_close_process_job,
    wait_for_process_job_terminal,
)


MAX_LOG_TAIL_BYTES = 64 * 1024
CARGO_JOB_HEARTBEAT_INTERVAL_SECONDS = 5.0
_STREAM_READER_JOIN_TIMEOUT_SECONDS = 5.0
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


@dataclass(slots=True)
class _PipeReaderGroup:
    threads: tuple[threading.Thread, ...]
    streams: tuple[TextIO, ...]
    errors: list[tuple[str, BaseException]]
    error_lock: threading.Lock
    read_failed: threading.Event


class CargoJobRunner:
    """Owns managed Cargo child processes so caller lifetime cannot orphan them."""

    def __init__(
        self,
        database: Database,
        cargo_jobs: CargoJobService,
        *,
        repo_root: str | Path,
        log_root: str | Path,
        popen: Callable[..., subprocess.Popen] | None = None,
        atomic_popen: Callable[..., tuple[subprocess.Popen, int]] | None = None,
        resume_process: Callable[[subprocess.Popen], None] = resume_popen_process,
        terminate_process_job: Callable[[int | None], None] = terminate_and_close_process_job,
        wait_process_job: Callable[..., None] = wait_for_process_job_terminal,
    ):
        self.database = database
        self.cargo_jobs = cargo_jobs
        self.repo_root = Path(repo_root).resolve()
        self.log_root = Path(log_root).resolve()
        self.popen = popen
        self.atomic_popen = atomic_popen or create_atomic_kill_on_close_process
        self._atomic_popen_injected = atomic_popen is not None
        self.resume_process = resume_process
        self.terminate_process_job = terminate_process_job
        self.wait_process_job = wait_process_job
        self._running_lock = threading.Lock()
        self._running: dict[str, subprocess.Popen] = {}
        self._collecting: set[str] = set()

    @staticmethod
    def _managed_cargo_environment(target_directory: str | Path) -> dict[str, str]:
        target_root = Path(target_directory).resolve()
        temporary = target_root / "temporary"
        cargo_home = target_root / "cargo-home"
        sccache = target_root / "sccache"
        for directory in (target_root, temporary, cargo_home, sccache):
            directory.mkdir(parents=True, exist_ok=True)
        return {
            "CARGO_TARGET_DIR": str(target_root),
            "CARGO_HOME": str(cargo_home),
            "SCCACHE_DIR": str(sccache),
            "TEMP": str(temporary),
            "TMP": str(temporary),
            "TMPDIR": str(temporary),
        }

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
        process_job: int | None = None
        reader_group: _PipeReaderGroup | None = None
        collector_decision = threading.Event()
        collector_authorized = threading.Event()
        authorized = False
        process_registered = False
        with self.cargo_jobs.managed_start_registration():
            try:
                self.cargo_jobs.authorize_managed_start(
                    job_id,
                    session_id=session_id,
                    command=command_tuple,
                )
                authorized = True
                child_environment = os.environ.copy()
                child_environment.update(environment_values)
                child_environment.update(self._managed_cargo_environment(job.target_dir))
                use_atomic_launch = self.popen is None and (
                    os.name == "nt" or self._atomic_popen_injected
                )
                if use_atomic_launch:
                    process, process_job = self.atomic_popen(
                        command_tuple,
                        cwd=working_root,
                        env=child_environment,
                    )
                    root_creation_time = popen_process_creation_time(process)
                else:
                    popen = self.popen or subprocess.Popen
                    with stdout_path.open(
                        "w", encoding="utf-8", errors="replace"
                    ) as stdout_file, stderr_path.open(
                        "w", encoding="utf-8", errors="replace"
                    ) as stderr_file:
                        process = popen(
                            command_tuple,
                            cwd=working_root,
                            env=child_environment,
                            stdout=stdout_file,
                            stderr=stderr_file,
                            text=True,
                        )
                    root_creation_time = None
                self.cargo_jobs.register_authorized_managed_run(
                    job_id,
                    session_id=session_id,
                    pid=int(process.pid),
                    command=command_tuple,
                    run_id=run_id,
                    environment=environment_values,
                    stdout_path=stdout_path,
                    stderr_path=stderr_path,
                    started_at=started_at,
                    root_process_creation_time=root_creation_time,
                )
                process_registered = True
                if process_job is not None:
                    reader_group = self._start_pipe_readers(
                        process,
                        stdout_path=stdout_path,
                        stderr_path=stderr_path,
                    )
                    with reader_group.error_lock:
                        setup_error = next(
                            (
                                error
                                for kind, error in reader_group.errors
                                if kind == "setup"
                            ),
                            None,
                        )
                    if setup_error is not None:
                        raise CoordinatorError(
                            "cargo_atomic_launch_log_open_failed",
                            "Cargo log output could not be opened before process resume",
                            details={"errorType": type(setup_error).__name__},
                        ) from setup_error
                with self._running_lock:
                    self._running[job_id] = process
                    self._collecting.add(job_id)
                try:
                    self.cargo_jobs.register_managed_collector(job_id)

                    def finish_after_launch_decision() -> None:
                        collector_decision.wait()
                        if collector_authorized.is_set():
                            self._finish(
                                run_id,
                                job_id,
                                session_id,
                                process,
                                process_job,
                                reader_group,
                                stdout_path,
                                stderr_path,
                            )

                    threading.Thread(
                        target=finish_after_launch_decision,
                        name=f"zircon-cargo-{job_id[:12]}",
                        daemon=True,
                    ).start()
                    if process_job is not None:
                        self.resume_process(process)
                        self.cargo_jobs.mark_authorized_managed_run_resumed(
                            run_id, job_id=job_id, session_id=session_id
                        )
                    collector_authorized.set()
                    collector_decision.set()
                except BaseException:
                    collector_decision.set()
                    with self._running_lock:
                        self._collecting.discard(job_id)
                    self.cargo_jobs.unregister_managed_collector(job_id)
                    raise
            except BaseException as launch_error:
                cleanup_error: BaseException | None = None
                if process is not None:
                    try:
                        if process_job is not None:
                            self.terminate_process_job(process_job)
                            process_job = None
                            process.wait(timeout=5)
                        elif process.poll() is None:
                            process.kill()
                            process.wait(timeout=5)
                    except BaseException as error:
                        cleanup_error = error
                if cleanup_error is not None and process is not None:
                    rejection_code = (
                        launch_error.code
                        if isinstance(launch_error, CoordinatorError)
                        else "cargo_launch_failed"
                    )
                    try:
                        self.cargo_jobs.record_cleanup_unproven_spawn(
                            run_id=run_id,
                            job_id=job_id,
                            session_id=session_id,
                            command=command_tuple,
                            environment=environment_values,
                            stdout_path=stdout_path,
                            stderr_path=stderr_path,
                            started_at=started_at,
                            pid=int(process.pid),
                            rejection_code=rejection_code,
                        )
                    except BaseException as registration_error:
                        durable_registration_error = type(registration_error).__name__
                    else:
                        durable_registration_error = None
                    with self._running_lock:
                        self._running[job_id] = process
                    raise CoordinatorError(
                        "cargo_launch_cleanup_unproven",
                        "Spawned Cargo process could not be confirmed stopped after launch setup failed",
                        details={
                            "jobId": job_id,
                            "pid": int(process.pid),
                            "durableRegistrationError": durable_registration_error,
                        },
                    ) from cleanup_error
                if authorized and not process_registered:
                    self.cargo_jobs.rollback_managed_start_authorization(
                        job_id,
                        session_id=session_id,
                        command=command_tuple,
                    )
                with self._running_lock:
                    self._running.pop(job_id, None)
                    self._collecting.discard(job_id)
                for reader in reader_group.threads if reader_group is not None else ():
                    reader.join(timeout=5)
                close_process = getattr(process, "close", None)
                if close_process is not None:
                    close_process()
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
            with self._running_lock:
                locally_collected = job_id in self._collecting
            if not locally_collected:
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
                """SELECT run.run_id, run.job_id, run.stdout_path, run.stderr_path,
                          run.error_code AS run_error_code,
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
                suspended_before_resume = (
                    row["run_error_code"] == "cargo_run_suspended_before_resume"
                )
                job_status = str(row["job_status"])
                if suspended_before_resume:
                    projected_status = "launch_failed"
                    error_code = "cargo_launch_interrupted_before_resume"
                elif job_status == "orphaned":
                    projected_status = "completed"
                    error_code = "cargo_run_reconciled_from_orphaned_job"
                elif row["exit_code"] is None:
                    projected_status = "completed"
                    error_code = "cargo_run_reconciled_from_released_job_missing_exit_code"
                else:
                    projected_status = "completed"
                    error_code = "cargo_run_reconciled_from_terminal_job"
                with self._running_lock:
                    if str(row["job_id"]) in self._collecting:
                        continue
                    updated = connection.execute(
                        """UPDATE cargo_job_runs
                           SET status=?, exit_code=?, stdout_tail=?, stderr_tail=?,
                               error_code=?,
                               completed_at=?
                           WHERE run_id=? AND status='running'""",
                        (
                            projected_status,
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

    def _terminate_job_tree(self, process_job: int) -> tuple[int | None, str | None]:
        try:
            self.terminate_process_job(process_job)
        except BaseException:
            try:
                close_process_job(process_job)
            except BaseException:
                # Keep the handle for the final kill-on-close retry.
                return process_job, "cargo_process_job_close_failed"
            return None, "cargo_process_job_termination_failed"
        return None, None

    def _finish(
        self,
        run_id: str,
        job_id: str,
        session_id: str,
        process: subprocess.Popen,
        process_job: int | None,
        reader_group: _PipeReaderGroup | None,
        stdout_path: Path,
        stderr_path: Path,
    ) -> None:
        try:
            job_tree_error: str | None = None
            job_tree_terminal = False
            if process_job is not None:
                next_heartbeat_at = time.monotonic()
                while process_job is not None:
                    if reader_group is not None and reader_group.read_failed.is_set():
                        process_job, termination_error = self._terminate_job_tree(process_job)
                        if termination_error is not None:
                            job_tree_error = termination_error
                        break
                    try:
                        self.wait_process_job(process_job, timeout_seconds=0.1)
                    except TimeoutError:
                        # Cancellation, shutdown, and the Job Object own termination;
                        # a cold Cargo build must not be rejected by a local collector timer.
                        now = time.monotonic()
                        if now >= next_heartbeat_at:
                            try:
                                self.cargo_jobs.heartbeat(job_id, session_id=session_id)
                            except CoordinatorError:
                                job_tree_error = "cargo_process_heartbeat_failed"
                                process_job, termination_error = self._terminate_job_tree(
                                    process_job
                                )
                                if termination_error is not None:
                                    job_tree_error = termination_error
                                break
                            else:
                                next_heartbeat_at = (
                                    now + CARGO_JOB_HEARTBEAT_INTERVAL_SECONDS
                                )
                    except OSError:
                        job_tree_error = "cargo_process_job_wait_failed"
                        try:
                            close_process_job(process_job)
                        except BaseException:
                            job_tree_error = "cargo_process_job_close_failed"
                        else:
                            process_job = None
                        break
                    else:
                        try:
                            close_process_job(process_job)
                        except BaseException:
                            job_tree_error = "cargo_process_job_close_failed"
                        else:
                            job_tree_terminal = True
                            process_job = None
                        break
            try:
                exit_code = int(
                    process.wait(timeout=10 if job_tree_error is not None else None)
                )
            except (subprocess.TimeoutExpired, TimeoutError):
                exit_code = -1
                job_tree_error = "cargo_process_root_termination_failed"
            if process_job is not None:
                # Defensive fallback for a future Job wait implementation that
                # can return without consuming the handle.
                if reader_group is not None and reader_group.read_failed.is_set():
                    process_job, termination_error = self._terminate_job_tree(process_job)
                    if termination_error is not None:
                        job_tree_error = termination_error
                else:
                    try:
                        close_process_job(process_job)
                    except BaseException:
                        job_tree_error = "cargo_process_job_close_failed"
                    else:
                        process_job = None
            reader_timeout = False
            if reader_group is not None:
                for reader in reader_group.threads:
                    reader.join(timeout=_STREAM_READER_JOIN_TIMEOUT_SECONDS)
                    if reader.is_alive():
                        reader_timeout = True
                if reader_timeout:
                    # A descendant may have inherited the pipe after the root
                    # exited. Close our handles to unblock the reader without
                    # waiting indefinitely on an unrelated process.
                    for stream in getattr(reader_group, "streams", ()):
                        try:
                            stream.close()
                        except OSError:
                            pass
            if process_job is not None:
                try:
                    close_process_job(process_job)
                except BaseException:
                    job_tree_error = "cargo_process_job_close_failed"
                else:
                    process_job = None
            error_code: str | None = None
            status = "completed"
            if job_tree_error is not None:
                status = "finish_blocked"
                error_code = job_tree_error
            if reader_group is not None and job_tree_error != "cargo_process_job_close_failed":
                with reader_group.error_lock:
                    reader_error_kinds = {kind for kind, _error in reader_group.errors}
                if "read" in reader_error_kinds:
                    status = "finish_blocked"
                    error_code = "cargo_run_log_read_failed"
                elif "write" in reader_error_kinds:
                    status = "finish_blocked"
                    error_code = "cargo_run_log_write_failed"
            if reader_timeout:
                status = "finish_blocked"
                error_code = "cargo_run_log_reader_timeout"
            try:
                # A wrapper can return before a Cargo child exits. Keep the runner,
                # not the originating shell, responsible for the final transition.
                finished = False
                if job_tree_error is None:
                    if job_tree_terminal:
                        self.cargo_jobs.finish_from_atomic_job_terminal(
                            job_id, session_id=session_id, exit_code=exit_code
                        )
                    else:
                        while True:
                            try:
                                if not finished:
                                    self.cargo_jobs.finish(
                                        job_id, session_id=session_id, exit_code=exit_code
                                    )
                                    finished = True
                                self.cargo_jobs.release(job_id, session_id=session_id)
                                break
                            except CoordinatorError as error:
                                if error.code != "cargo_process_tree_alive":
                                    raise
                                if not finished:
                                    self.cargo_jobs.heartbeat(job_id, session_id=session_id)
                                time.sleep(1)
            except CoordinatorError as error:
                status = "finish_blocked"
                error_code = error.code
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
        finally:
            try:
                close_process_job(process_job)
            except BaseException:
                pass
            close_process = getattr(process, "close", None)
            if close_process is not None:
                close_process()
            self.cargo_jobs.unregister_managed_collector(job_id)
            with self._running_lock:
                self._running.pop(job_id, None)
                self._collecting.discard(job_id)

    def _start_pipe_readers(
        self,
        process: subprocess.Popen,
        *,
        stdout_path: Path,
        stderr_path: Path,
    ) -> _PipeReaderGroup:
        readers: list[threading.Thread] = []
        streams: list[TextIO] = []
        ready_events: list[threading.Event] = []
        reader_errors: list[tuple[str, BaseException]] = []
        error_lock = threading.Lock()
        read_failed = threading.Event()
        for stream, path, label in (
            (getattr(process, "stdout", None), stdout_path, "stdout"),
            (getattr(process, "stderr", None), stderr_path, "stderr"),
        ):
            if stream is None:
                raise CoordinatorError(
                    "cargo_atomic_launch_pipe_missing",
                    f"Atomic Cargo launch did not retain its {label} pipe",
                )
            ready = threading.Event()
            reader = threading.Thread(
                target=self._drain_stream,
                args=(
                    stream,
                    path,
                    ready,
                    reader_errors,
                    error_lock,
                    read_failed,
                ),
                name=f"zircon-cargo-{label}",
                daemon=True,
            )
            reader.start()
            readers.append(reader)
            streams.append(stream)
            ready_events.append(ready)
        for ready in ready_events:
            if not ready.wait(timeout=5):
                raise CoordinatorError(
                    "cargo_atomic_launch_pipe_reader_timeout",
                    "Cargo log reader did not become ready before process resume",
                )
        return _PipeReaderGroup(
            tuple(readers), tuple(streams), reader_errors, error_lock, read_failed
        )

    @staticmethod
    def _drain_stream(
        stream: io.TextIOBase,
        path: Path,
        ready: threading.Event,
        errors: list[tuple[str, BaseException]],
        error_lock: threading.Lock,
        read_failed: threading.Event,
    ) -> None:
        output: io.TextIOBase | None = None
        try:
            try:
                output = path.open("w", encoding="utf-8", errors="replace")
            except BaseException as error:
                with error_lock:
                    errors.append(("setup", error))
                ready.set()
                return
            else:
                ready.set()
            while True:
                try:
                    chunk = stream.read(8192)
                except BaseException as error:
                    with error_lock:
                        errors.append(("read", error))
                    read_failed.set()
                    return
                if not chunk:
                    return
                if output is not None:
                    try:
                        output.write(chunk)
                    except BaseException as error:
                        with error_lock:
                            errors.append(("write", error))
                        try:
                            output.close()
                        except BaseException:
                            pass
                        output = None
        finally:
            ready.set()
            if output is not None:
                try:
                    output.close()
                except BaseException as error:
                    with error_lock:
                        errors.append(("write", error))

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

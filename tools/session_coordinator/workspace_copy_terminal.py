from __future__ import annotations

import codecs
import io
import json
import os
import subprocess
import threading
import time
from collections import deque
from contextlib import nullcontext
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, ContextManager, TextIO

from .database import Database
from .models import CoordinatorError, utc_text

_CAPTURED_STREAM_CHARACTER_LIMIT = 65_536
_STREAM_READ_CHARACTER_COUNT = 8_192
_STREAM_READER_JOIN_TIMEOUT_SECONDS = 5.0
_STREAM_READER_CANCEL_GRACE_SECONDS = 1.0
_STREAM_POLL_INTERVAL_SECONDS = 0.01


def _cancel_synchronous_reader_io(reader: threading.Thread) -> None:
    if os.name != "nt" or reader.native_id is None:
        return
    import ctypes

    thread_terminate = 0x0001
    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel32.OpenThread.argtypes = (
        ctypes.c_uint32,
        ctypes.c_int,
        ctypes.c_uint32,
    )
    kernel32.OpenThread.restype = ctypes.c_void_p
    kernel32.CancelSynchronousIo.argtypes = (ctypes.c_void_p,)
    kernel32.CancelSynchronousIo.restype = ctypes.c_int
    kernel32.CloseHandle.argtypes = (ctypes.c_void_p,)
    kernel32.CloseHandle.restype = ctypes.c_int
    handle = kernel32.OpenThread(thread_terminate, False, reader.native_id)
    if not handle:
        return
    try:
        kernel32.CancelSynchronousIo(handle)
    finally:
        kernel32.CloseHandle(handle)


class _BoundedTextTail:
    def __init__(self, limit: int) -> None:
        self._limit = limit
        self._chunks: deque[str] = deque()
        self._length = 0

    def append(self, value: str) -> None:
        if not value:
            return
        self._chunks.append(value)
        self._length += len(value)
        while self._length > self._limit:
            excess = self._length - self._limit
            first = self._chunks[0]
            if len(first) <= excess:
                self._chunks.popleft()
                self._length -= len(first)
            else:
                self._chunks[0] = first[excess:]
                self._length -= excess

    def value(self) -> str:
        return "".join(self._chunks)


@dataclass(frozen=True, slots=True)
class ValidationRunEvidence:
    run_id: str
    job_id: str
    command: tuple[str, ...]
    exit_code: int
    stdout: str
    stderr: str

    def to_dict(self) -> dict[str, object]:
        return {
            "run_id": self.run_id,
            "job_id": self.job_id,
            "command": list(self.command),
            "exit_code": self.exit_code,
            "stdout": self.stdout,
            "stderr": self.stderr,
        }


class ValidationCopyTerminalLifecycle:
    def __init__(
        self,
        database: Database,
        mutation_gate: Callable[[], ContextManager[None]] | None,
    ) -> None:
        self._database = database
        self._mutation_gate = mutation_gate

    def collect(
        self,
        process: subprocess.Popen[str],
        *,
        after_root_exit: Callable[[], None] | None = None,
    ) -> tuple[int, str, str]:
        stdout_tail = _BoundedTextTail(_CAPTURED_STREAM_CHARACTER_LIMIT)
        stderr_tail = _BoundedTextTail(_CAPTURED_STREAM_CHARACTER_LIMIT)
        errors: list[BaseException] = []
        error_lock = threading.Lock()
        root_exited = threading.Event()

        def drain(stream: TextIO, tail: _BoundedTextTail) -> None:
            try:
                try:
                    descriptor = stream.fileno()
                    os.set_blocking(descriptor, False)
                except (AttributeError, OSError):
                    while chunk := stream.read(_STREAM_READ_CHARACTER_COUNT):
                        tail.append(chunk)
                else:
                    character_decoder = codecs.getincrementaldecoder(
                        getattr(stream, "encoding", None) or "utf-8"
                    )(errors=getattr(stream, "errors", None) or "replace")
                    decoder = io.IncrementalNewlineDecoder(
                        character_decoder, translate=True
                    )
                    while True:
                        try:
                            chunk = os.read(descriptor, _STREAM_READ_CHARACTER_COUNT)
                        except BlockingIOError:
                            if root_exited.is_set():
                                break
                            root_exited.wait(_STREAM_POLL_INTERVAL_SECONDS)
                            continue
                        if not chunk:
                            break
                        tail.append(decoder.decode(chunk))
                    tail.append(decoder.decode(b"", final=True))
            except BaseException as error:
                if not root_exited.is_set():
                    with error_lock:
                        errors.append(error)
                    try:
                        if process.poll() is None:
                            process.kill()
                    except BaseException:
                        pass
            finally:
                try:
                    stream.close()
                except OSError:
                    pass

        readers = [
            (
                threading.Thread(
                    target=drain,
                    args=(stream, tail),
                    name=f"zircon-validation-{name}-drain",
                    daemon=True,
                ),
                stream,
                name,
            )
            for name, stream, tail in (
                ("stdout", process.stdout, stdout_tail),
                ("stderr", process.stderr, stderr_tail),
            )
            if stream is not None
        ]
        for reader, _stream, _name in readers:
            reader.start()
        waited_exit_code = process.wait()
        timed_out: list[tuple[threading.Thread, TextIO, str]] = []
        try:
            if after_root_exit is not None:
                after_root_exit()
        finally:
            root_exited.set()
            deadline = time.monotonic() + _STREAM_READER_JOIN_TIMEOUT_SECONDS
            for reader, _stream, _name in readers:
                reader.join(max(0.0, deadline - time.monotonic()))
            timed_out = [
                (reader, stream, name)
                for reader, stream, name in readers
                if reader.is_alive()
            ]
            for reader, _stream, _name in timed_out:
                _cancel_synchronous_reader_io(reader)
            cancel_deadline = time.monotonic() + _STREAM_READER_CANCEL_GRACE_SECONDS
            for reader, _stream, _name in timed_out:
                reader.join(max(0.0, cancel_deadline - time.monotonic()))
            timed_out = [item for item in timed_out if item[0].is_alive()]
        if timed_out:
            raise CoordinatorError(
                "validation_copy_stream_capture_timeout",
                "Validation process output streams remained open after root exit",
                details={
                    "streams": [name for _reader, _stream, name in timed_out],
                    "timeoutSeconds": _STREAM_READER_JOIN_TIMEOUT_SECONDS,
                },
            )
        if errors:
            error = errors[0]
            raise CoordinatorError(
                "validation_copy_stream_capture_failed",
                "Validation process output stream could not be captured",
                details={"errorType": type(error).__name__},
            ) from error
        exit_code = process.returncode
        if exit_code is None:
            exit_code = waited_exit_code
        if exit_code is None:
            raise CoordinatorError(
                "validation_copy_process_not_terminal",
                "Validation process wait returned without a terminal exit code",
            )
        return int(exit_code), stdout_tail.value(), stderr_tail.value()

    def latest_for_job(
        self, *, session_id: str, job_id: str
    ) -> ValidationRunEvidence | None:
        with self._database.connect() as connection:
            row = connection.execute(
                """SELECT run_id, job_id, command_json, exit_code,
                          stdout_text, stderr_text
                   FROM validation_copy_runs
                   WHERE session_id = ? AND job_id = ?
                   ORDER BY completed_at DESC, run_id DESC
                   LIMIT 1""",
                (session_id, job_id),
            ).fetchone()
        if row is None:
            return None
        try:
            command = json.loads(str(row["command_json"]))
        except (TypeError, ValueError) as error:
            raise CoordinatorError(
                "validation_copy_terminal_evidence_invalid",
                "Validation-copy terminal evidence contains invalid command JSON",
                details={"runId": str(row["run_id"])},
            ) from error
        if not isinstance(command, list) or not all(
            isinstance(part, str) for part in command
        ):
            raise CoordinatorError(
                "validation_copy_terminal_evidence_invalid",
                "Validation-copy terminal evidence command must be a string array",
                details={"runId": str(row["run_id"])},
            )
        return ValidationRunEvidence(
            run_id=str(row["run_id"]),
            job_id=str(row["job_id"]),
            command=tuple(command),
            exit_code=int(row["exit_code"]),
            stdout=str(row["stdout_text"]),
            stderr=str(row["stderr_text"]),
        )

    def persist(
        self,
        *,
        run_id: str,
        job_id: str,
        session_id: str,
        command: tuple[str, ...],
        exit_code: int,
        stdout: str,
        stderr: str,
        started_at: str,
    ) -> ValidationRunEvidence:
        gate = (
            self._mutation_gate()
            if self._mutation_gate is not None
            else nullcontext()
        )
        with gate, self._database.transaction() as connection:
            connection.execute(
                """INSERT INTO validation_copy_runs(
                       run_id, job_id, session_id, command_json, exit_code,
                       stdout_text, stderr_text, started_at, completed_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    run_id,
                    job_id,
                    session_id,
                    json.dumps(command),
                    exit_code,
                    stdout,
                    stderr,
                    started_at,
                    utc_text(),
                ),
            )
        return ValidationRunEvidence(
            run_id, job_id, command, exit_code, stdout, stderr
        )

    def finalize_success(self, job_id: str) -> None:
        gate = (
            self._mutation_gate()
            if self._mutation_gate is not None
            else nullcontext()
        )
        with gate, self._database.transaction() as connection:
            cursor = connection.execute(
                "UPDATE validation_copies SET status = 'materialized', run_pid = NULL "
                "WHERE job_id = ? AND status = 'running'",
                (job_id,),
            )
            if cursor.rowcount != 1:
                raise CoordinatorError(
                    "validation_copy_terminal_state_changed",
                    "Validation copy changed state before terminal completion",
                )

    def notify_completion(
        self, completion_hook: Callable[[str], None] | None, run_id: str
    ) -> None:
        if completion_hook is None:
            return
        try:
            completion_hook(run_id)
        except BaseException as error:
            raise CoordinatorError(
                "validation_copy_completion_hook_failed",
                "Validation evidence was persisted, but its completion hook failed",
                details={"runId": run_id},
            ) from error

    def preserve_completion_failure(
        self,
        *,
        error: BaseException,
        run_id: str,
        job_id: str,
        session_id: str,
    ) -> None:
        if (
            not isinstance(error, CoordinatorError)
            or error.code != "validation_copy_completion_hook_failed"
        ):
            return
        try:
            self._record_completion_failure(
                run_id=run_id,
                job_id=job_id,
                session_id=session_id,
                error_code=error.code,
            )
        except BaseException:
            # The typed hook error remains authoritative. The caller still
            # suppresses cleanup, preserving the immutable source for recovery.
            return

    def _record_completion_failure(
        self,
        *,
        run_id: str,
        job_id: str,
        session_id: str,
        error_code: str,
    ) -> None:
        gate = (
            self._mutation_gate()
            if self._mutation_gate is not None
            else nullcontext()
        )
        with gate, self._database.transaction() as connection:
            connection.execute(
                "UPDATE validation_copies SET status = 'failed', run_pid = NULL "
                "WHERE job_id = ? AND status = 'running'",
                (job_id,),
            )
            connection.execute(
                "INSERT INTO events(session_id, event_type, payload_json, created_at) "
                "VALUES (?, ?, ?, ?)",
                (
                    session_id,
                    "validation_copy.completion_hook_failed",
                    json.dumps(
                        {
                            "errorCode": error_code,
                            "jobId": job_id,
                            "runId": run_id,
                        },
                        sort_keys=True,
                    ),
                    utc_text(),
                ),
            )

    def restore_after_failure(
        self,
        job_id: str,
        *,
        process_started: bool,
        evidence_persisted: bool,
    ) -> None:
        if evidence_persisted:
            return
        status = "failed" if process_started else "materialized"
        gate = (
            self._mutation_gate()
            if self._mutation_gate is not None
            else nullcontext()
        )
        try:
            with gate, self._database.transaction() as connection:
                connection.execute(
                    "UPDATE validation_copies SET status = ?, run_pid = NULL "
                    "WHERE job_id = ? AND status = 'running'",
                    (status, job_id),
                )
        except BaseException:
            # Keep the original terminal-evidence failure authoritative and
            # preserve the materialized copy for coordinator recovery.
            return

    def recover_missing_roots(
        self,
        *,
        validate_cleanup_root: Callable[[Path], None],
        running_lock: ContextManager[None],
        active_run_jobs: Callable[[], frozenset[str]],
    ) -> int:
        """Converge idle durable copies after their managed root disappears."""
        with self._database.connect() as connection:
            rows = connection.execute(
                """SELECT job_id, session_id, job_root, status
                   FROM validation_copies
                   WHERE status IN ('materialized', 'failed') AND run_pid IS NULL"""
            ).fetchall()

        missing: list[tuple[str, str, str, str]] = []
        for row in rows:
            job_id = str(row["job_id"])
            try:
                raw_candidate = Path(str(row["job_root"]))
                validate_cleanup_root(raw_candidate.resolve())
            except Exception:
                continue
            if raw_candidate.exists() or raw_candidate.is_symlink():
                continue
            missing.append(
                (
                    job_id,
                    str(row["session_id"]),
                    str(row["job_root"]),
                    str(row["status"]),
                )
            )

        if not missing:
            return 0
        recovered = 0
        gate = (
            self._mutation_gate()
            if self._mutation_gate is not None
            else nullcontext()
        )
        with gate, running_lock:
            locally_active = active_run_jobs()
            still_missing = [
                row
                for row in missing
                if not Path(row[2]).exists() and not Path(row[2]).is_symlink()
            ]
            if not still_missing:
                return 0
            with self._database.transaction() as connection:
                for job_id, session_id, job_root, prior_status in still_missing:
                    if job_id in locally_active:
                        continue
                    cursor = connection.execute(
                        """UPDATE validation_copies
                           SET status='removed', removed_at=?
                           WHERE job_id=? AND job_root=? AND status=? AND run_pid IS NULL""",
                        (utc_text(), job_id, job_root, prior_status),
                    )
                    if cursor.rowcount != 1:
                        continue
                    connection.execute(
                        """INSERT INTO events(session_id, event_type, payload_json, created_at)
                           VALUES (?, 'validation_copy.missing_root_recovered', ?, ?)""",
                        (
                            session_id,
                            json.dumps(
                                {
                                    "jobId": job_id,
                                    "jobRoot": job_root,
                                    "priorStatus": prior_status,
                                },
                                sort_keys=True,
                            ),
                            utc_text(),
                        ),
                    )
                    recovered += 1
        return recovered

    @staticmethod
    def _capture_stream_tail(value: str | None) -> str:
        return (value or "")[-_CAPTURED_STREAM_CHARACTER_LIMIT:]

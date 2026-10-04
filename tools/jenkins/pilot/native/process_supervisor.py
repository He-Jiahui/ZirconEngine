"""Worker-owned process trees and bounded, cursor based output capture."""

from __future__ import annotations

import os
import subprocess
import threading
import time
from collections import deque
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable, Iterable, Mapping

from .process_identity import popen_process_creation_time
from .windows_job_process import (
    AtomicJobProcess,
    close_process_job,
    create_atomic_kill_on_close_process,
    resume_popen_process,
    terminate_and_close_process_job,
    wait_for_process_job_terminal,
)


DEFAULT_OUTPUT_LIMIT_BYTES = 256 * 1024
DEFAULT_OUTPUT_CHUNK_BYTES = 8192


@dataclass(frozen=True, slots=True)
class OutputBatch:
    cursor: int
    next_cursor: int
    data: bytes
    truncated: bool


class BoundedOutput:
    """A byte bounded ring retaining a monotonically increasing cursor."""

    def __init__(self, maximum_bytes: int = DEFAULT_OUTPUT_LIMIT_BYTES) -> None:
        if maximum_bytes <= 0:
            raise ValueError("maximum_bytes must be positive")
        self.maximum_bytes = int(maximum_bytes)
        self._chunks: deque[tuple[int, bytes]] = deque()
        self._start = 0
        self._end = 0
        self._size = 0
        self._lock = threading.Lock()

    @property
    def cursor(self) -> int:
        with self._lock:
            return self._end

    def append(self, data: bytes) -> int:
        if not data:
            return self.cursor
        raw = bytes(data)
        if len(raw) > self.maximum_bytes:
            raw = raw[-self.maximum_bytes :]
        with self._lock:
            offset = self._end
            self._chunks.append((offset, raw))
            self._end += len(raw)
            self._size += len(raw)
            while self._size > self.maximum_bytes and self._chunks:
                first_offset, first = self._chunks[0]
                excess = self._size - self.maximum_bytes
                if excess >= len(first):
                    self._chunks.popleft()
                    self._size -= len(first)
                    self._start = first_offset + len(first)
                else:
                    self._chunks[0] = (first_offset + excess, first[excess:])
                    self._size -= excess
                    self._start = first_offset + excess
            if not self._chunks:
                self._start = self._end
            return self._end

    def read(self, cursor: int, maximum_bytes: int = DEFAULT_OUTPUT_CHUNK_BYTES) -> OutputBatch:
        if cursor < 0 or maximum_bytes <= 0:
            raise ValueError("cursor and maximum_bytes must be non-negative/positive")
        with self._lock:
            requested = int(cursor)
            start = min(max(requested, self._start), self._end)
            output = bytearray()
            for offset, chunk in self._chunks:
                chunk_end = offset + len(chunk)
                if chunk_end <= start:
                    continue
                begin = max(0, start - offset)
                available = chunk[begin : begin + maximum_bytes - len(output)]
                output.extend(available)
                if len(output) >= maximum_bytes:
                    break
            next_cursor = start + len(output)
            return OutputBatch(
                cursor=requested,
                next_cursor=next_cursor,
                data=bytes(output),
                truncated=requested < self._start,
            )


@dataclass(frozen=True, slots=True)
class ManagedProcessSnapshot:
    owner: tuple[str, str, int, int]
    pid: int
    creation_time: str
    returncode: int | None
    stdout_cursor: int
    stderr_cursor: int
    started_at: float
    ended_at: float | None
    terminal_scope: str | None = None


class ManagedProcess:
    """A retained root and atomically owned Windows Job Object."""

    def __init__(
        self,
        process: subprocess.Popen | AtomicJobProcess,
        *,
        owner: tuple[str, str, int, int],
        creation_time: str,
        job_handle: int | None,
        maximum_output_bytes: int,
        output_callback: Callable[[tuple[str, str, int, int], str, bytes], None] | None = None,
        job_terminal_timeout: float = 8.0,
    ) -> None:
        self.process = process
        self.owner = owner
        self.pid = int(process.pid)
        self.creation_time = creation_time
        self.job_handle = job_handle
        self.stdout = BoundedOutput(maximum_output_bytes)
        self.stderr = BoundedOutput(maximum_output_bytes)
        self._callback = output_callback
        self._job_terminal_timeout = job_terminal_timeout
        self._termination_lock = threading.Lock()
        self._terminated = False
        self._closed = False
        self._job_terminal_proven = False
        self.started_at = time.monotonic()
        self.ended_at: float | None = None
        self.terminal_scope: str | None = None
        self._pipe_eof = {"stdout": False, "stderr": False}
        self._pipe_errors: dict[str, BaseException] = {}
        self._readers = tuple(
            threading.Thread(target=self._drain, args=(name,), daemon=True, name=f"worker-{name}-{self.pid}")
            for name in ("stdout", "stderr")
        )
        for reader in self._readers:
            reader.start()

    def _drain(self, name: str) -> None:
        stream = getattr(self.process, name, None)
        if stream is None:
            self._pipe_errors[name] = OSError("owned process pipe unavailable")
            return
        raw_stream = getattr(stream, "buffer", stream)
        read = getattr(raw_stream, "read1", raw_stream.read)
        while True:
            try:
                data = read(DEFAULT_OUTPUT_CHUNK_BYTES)
            except (OSError, ValueError) as error:
                self._pipe_errors[name] = error
                break
            if not data:
                self._pipe_eof[name] = True
                break
            encoded = data if isinstance(data, bytes) else data.encode("utf-8", errors="replace")
            destination = self.stdout if name == "stdout" else self.stderr
            destination.append(encoded)
            if self._callback is not None:
                try:
                    self._callback(self.owner, name, encoded)
                except Exception as error:
                    # Continue draining the owned pipe to avoid blocking the
                    # child, but lost durable output forbids acceptance.
                    self._pipe_errors[name] = error

    def wait(self, timeout: float | None = None) -> int:
        try:
            result = int(self.process.wait(timeout=timeout))
        except subprocess.TimeoutExpired:
            raise
        if self.job_handle is not None:
            try:
                wait_for_process_job_terminal(
                    self.job_handle,
                    timeout_seconds=self._job_terminal_timeout,
                )
                self._job_terminal_proven = True
            except TimeoutError:
                self.terminate(reason="job_descendants_did_not_exit")
                raise
        elif not self._job_terminal_proven:
            raise OSError("retained Windows Job unavailable for terminal proof")
        for reader in self._readers:
            reader.join(timeout=self._job_terminal_timeout)
        if self._pipe_errors or not all(self._pipe_eof.values()):
            raise OSError("owned output pipes did not both reach actual EOF")
        self.terminal_scope = "windows_job"
        self.ended_at = time.monotonic()
        close_process_job(self.job_handle)
        self.job_handle = None
        return result

    def terminate(self, *, reason: str = "cancelled") -> None:
        del reason  # Kept for audit hooks and diagnostics at the caller boundary.
        with self._termination_lock:
            if self._terminated:
                return
            if self.job_handle is not None:
                handle, self.job_handle = self.job_handle, None
                terminate_and_close_process_job(handle)
                self._job_terminal_proven = True
            else:
                if not self._job_terminal_proven:
                    raise OSError("retained Windows Job unavailable for termination")
            # Job accounting can reach zero before the retained root handle
            # signals. Preserve its real exit code before any handle close.
            self.process.wait(timeout=self._job_terminal_timeout)
            self._terminated = True

    def output(self, channel: str, cursor: int, maximum_bytes: int = DEFAULT_OUTPUT_CHUNK_BYTES) -> OutputBatch:
        if channel == "stdout":
            return self.stdout.read(cursor, maximum_bytes)
        if channel == "stderr":
            return self.stderr.read(cursor, maximum_bytes)
        raise ValueError("channel must be stdout or stderr")

    def snapshot(self) -> ManagedProcessSnapshot:
        return ManagedProcessSnapshot(
            owner=self.owner,
            pid=self.pid,
            creation_time=self.creation_time,
            returncode=self.process.poll(),
            stdout_cursor=self.stdout.cursor,
            stderr_cursor=self.stderr.cursor,
            started_at=self.started_at,
            ended_at=self.ended_at,
            terminal_scope=self.terminal_scope,
        )

    def close(self) -> None:
        if self._closed:
            return
        self._closed = True
        for reader in self._readers:
            reader.join(timeout=0.25)
        if self.job_handle is not None:
            close_process_job(self.job_handle)
            self.job_handle = None
        closer = getattr(self.process, "close", None)
        if callable(closer):
            closer()
        else:
            for stream in (self.process.stdout, self.process.stderr):
                if stream is not None:
                    stream.close()


class ProcessSupervisor:
    """Own only child trees started by this worker and bind each to its lease."""

    def __init__(
        self,
        *,
        maximum_output_bytes: int = DEFAULT_OUTPUT_LIMIT_BYTES,
        popen: Callable[..., subprocess.Popen] = subprocess.Popen,
        atomic_popen: Callable[..., tuple[subprocess.Popen | AtomicJobProcess, int]] = create_atomic_kill_on_close_process,
        resume_process: Callable[[Any], None] = resume_popen_process,
        output_callback: Callable[[tuple[str, str, int, int], str, bytes], None] | None = None,
        job_terminal_timeout: float = 8.0,
    ) -> None:
        self.maximum_output_bytes = maximum_output_bytes
        self.popen = popen
        self.atomic_popen = atomic_popen
        self.resume_process = resume_process
        self.output_callback = output_callback
        self.job_terminal_timeout = job_terminal_timeout
        self._lock = threading.RLock()
        self._processes: dict[tuple[str, str, int, int], ManagedProcess] = {}
        self._closed = False

    def start(
        self,
        command: Iterable[str],
        *,
        cwd: Path,
        env: Mapping[str, str],
        owner: tuple[str, str, int, int],
    ) -> ManagedProcess:
        # Keep admission, atomic creation, and registration one ownership
        # operation so close() cannot finish while a root remains unregistered.
        with self._lock:
            return self._start_locked(command, cwd=cwd, env=env, owner=owner)

    def _start_locked(
        self,
        command: Iterable[str],
        *,
        cwd: Path,
        env: Mapping[str, str],
        owner: tuple[str, str, int, int],
    ) -> ManagedProcess:
        argv = tuple(str(value) for value in command)
        if os.name != "nt":
            raise OSError("Jenkins process supervision requires atomic Windows Jobs")
        if not argv or any(not part or "\x00" in part for part in argv):
            raise ValueError("process command must contain non-empty argv elements")
        if len(owner) != 4 or not owner[0] or not owner[1] or owner[2] < 0 or owner[3] < 0:
            raise ValueError("process owner must contain TaskId, AttemptId, generation and lease generation")
        with self._lock:
            if self._closed:
                raise RuntimeError("process supervisor is closed")
            if owner in self._processes:
                previous = self._processes[owner]
                if previous.terminal_scope is None:
                    raise RuntimeError("this task attempt already owns an active process")
                previous.close()
                del self._processes[owner]
        job_handle: int | None = None
        process, job_handle = self.atomic_popen(argv, cwd=Path(cwd), env=dict(env))
        try:
            if not job_handle:
                raise OSError("atomic launcher did not retain a Windows Job")
            creation_time = popen_process_creation_time(process)
            self.resume_process(process)
        except BaseException:
            if job_handle:
                terminate_and_close_process_job(job_handle)
            process.close()
            raise
        managed = ManagedProcess(
            process,
            owner=owner,
            creation_time=creation_time,
            job_handle=job_handle,
            maximum_output_bytes=self.maximum_output_bytes,
            output_callback=self.output_callback,
            job_terminal_timeout=self.job_terminal_timeout,
        )
        with self._lock:
            self._processes[owner] = managed
        return managed

    def cancel_owner(self, owner: tuple[str, str, int, int], *, reason: str) -> bool:
        with self._lock:
            process = self._processes.get(owner)
        if process is None:
            return False
        if process.terminal_scope is None:
            process.terminate(reason=reason)
        return True

    def cancel_lease(self, task_id: str, attempt_id: str, lease_generation: int, *, reason: str) -> int:
        with self._lock:
            owners = [
                owner
                for owner in self._processes
                if owner[0] == task_id and owner[1] == attempt_id and owner[3] == lease_generation
            ]
        stopped = 0
        for owner in owners:
            stopped += int(self.cancel_owner(owner, reason=reason))
        return stopped

    def disconnect(self) -> int:
        with self._lock:
            active = tuple(
                (owner, process)
                for owner, process in self._processes.items()
                if process.terminal_scope is None
            )
        return sum(
            int(self.cancel_owner(owner, reason="worker_disconnect"))
            for owner, _process in active
        )

    def snapshots(self) -> tuple[ManagedProcessSnapshot, ...]:
        with self._lock:
            processes = tuple(self._processes.values())
        return tuple(process.snapshot() for process in processes)

    def close(self) -> None:
        with self._lock:
            if self._closed:
                return
            self._closed = True
            processes = tuple(self._processes.values())
        for process in processes:
            if process.terminal_scope is None:
                process.terminate(reason="worker_shutdown")
            process.close()


__all__ = [
    "BoundedOutput",
    "ManagedProcess",
    "ManagedProcessSnapshot",
    "OutputBatch",
    "ProcessSupervisor",
]

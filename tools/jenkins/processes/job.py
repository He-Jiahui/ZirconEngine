from __future__ import annotations

import os
import subprocess
import time
import threading
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Mapping

from tools.jenkins.contracts import JenkinsError

from .identity import ProcessIdentity, current_identity, identity_from_popen, identity_matches, process_alive
from . import native


@dataclass(frozen=True, slots=True)
class TerminationProof:
    """Evidence that an owned execution reached a terminal state."""

    identity: ProcessIdentity
    requested: bool
    process_exit_code: int | None
    active_processes: int | None
    stdout_eof: bool
    stderr_eof: bool
    children_gone: bool

    @property
    def complete(self) -> bool:
        return (
            self.children_gone
            and self.stdout_eof
            and self.stderr_eof
            and self.active_processes in (None, 0)
        )

    def to_dict(self) -> dict[str, object]:
        return {
            "identity": self.identity.to_dict(),
            "requested": self.requested,
            "processExitCode": self.process_exit_code,
            "activeProcesses": self.active_processes,
            "stdoutEof": self.stdout_eof,
            "stderrEof": self.stderr_eof,
            "childrenGone": self.children_gone,
            "complete": self.complete,
        }


class ManagedProcess:
    """Popen-like process plus immutable identity and optional Job handle."""

    def __init__(self, process: object, identity: ProcessIdentity, job_handle: int | None = None, *, log_dir: Path | None = None):
        self.process = process
        self.identity = identity
        self.job_handle = job_handle
        self._stdout_eof = False
        self._stderr_eof = False
        self._pump_threads: list[threading.Thread] = []
        self._log_handles: list[object] = []
        if log_dir is not None:
            log_dir.mkdir(parents=True, exist_ok=True)
            self._start_log_pumps(log_dir)

    def _start_log_pumps(self, log_dir: Path) -> None:
        for name in ("stdout", "stderr"):
            stream = getattr(self.process, name, None)
            if stream is None:
                continue
            handle = (log_dir / f"{name}.log").open("ab")
            self._log_handles.append(handle)
            thread = threading.Thread(target=self._pump, args=(stream, handle, name), daemon=True)
            thread.start()
            self._pump_threads.append(thread)

    def _pump(self, stream: object, handle: object, name: str) -> None:
        try:
            raw = getattr(stream, "buffer", stream)
            read = getattr(raw, "read1", raw.read)
            while True:
                chunk = read(65536)
                if not chunk:
                    setattr(self, f"_{name}_eof", True)
                    return
                if isinstance(chunk, str):
                    chunk = chunk.encode("utf-8", "replace")
                handle.write(chunk)
                handle.flush()
        except (OSError, ValueError):
            return

    @property
    def pid(self) -> int:
        return self.identity.pid

    @property
    def returncode(self) -> int | None:
        return getattr(self.process, "returncode", None)

    def poll(self) -> int | None:
        return self.process.poll()

    def wait(self, timeout: float | None = None) -> int:
        return int(self.process.wait(timeout=timeout))

    def communicate(self, timeout: float | None = None):
        if self._pump_threads:
            for thread in self._pump_threads:
                thread.join(timeout=timeout)
            self._stdout_eof = self._stdout_eof or not any(t.is_alive() for t in self._pump_threads)
            self._stderr_eof = self._stderr_eof or self._stdout_eof
            return (b"", b"")
        if hasattr(self.process, "communicate"):
            result = self.process.communicate(timeout=timeout)
        else:
            self.process.wait(timeout=timeout)
            result = (self.process.stdout.read(), self.process.stderr.read())
        self._stdout_eof = self._stderr_eof = True
        return result

    def close(self) -> None:
        for thread in self._pump_threads:
            thread.join(timeout=2)
        for handle in self._log_handles:
            try: handle.close()
            except OSError: pass
        close = getattr(self.process, "close", None)
        if close is not None:
            close()


class NativeJob:
    """Own a process tree and provide identity-checked termination evidence."""

    def __init__(self, managed: ManagedProcess):
        self.process = managed
        self.identity = managed.identity
        self.job_handle = managed.job_handle
        self._terminated = False
        self._job_terminal_proven = False
        self._terminal_active: int | None = None

    @classmethod
    def launch(
        cls,
        args: Iterable[str],
        *,
        cwd: str | os.PathLike[str],
        env: Mapping[str, str] | None = None,
        text: bool = True,
        stdout: object | None = None,
        stderr: object | None = None,
        log_dir: str | os.PathLike[str] | None = None,
        suspended: bool = False,
        job_name: str | None = None,
    ) -> "NativeJob":
        command = tuple(str(value) for value in args)
        if not command:
            raise JenkinsError("invalid_process_command", "process command cannot be empty")
        workdir = Path(cwd).resolve()
        if not workdir.is_dir():
            raise JenkinsError("invalid_process_cwd", f"process working directory does not exist: {workdir}")
        environment = dict(env or os.environ)
        if os.name == "nt":
            try:
                process, handle = native.create_atomic_kill_on_close_process(command, cwd=workdir, env=environment, job_name=job_name)
                # The atomic launcher deliberately returns suspended so Job
                # membership is established before the first instruction.
                identity = current_identity(process.pid, executable=command[0], command_line=command)
                job = cls(NativeJob._managed(process, identity, handle, log_dir=Path(log_dir) if log_dir else None))
                job.job_name = job_name
                job._suspended = True
                if not suspended:
                    job.resume()
                return job
            except (OSError, JenkinsError):
                raise
        if suspended:
            raise JenkinsError("suspended_process_unsupported", "atomic suspended launch requires Windows")
        process = subprocess.Popen(command, cwd=str(workdir), env=environment, stdin=subprocess.DEVNULL,
                                   stdout=subprocess.PIPE if stdout is None else stdout,
                                   stderr=subprocess.PIPE if stderr is None else stderr, text=text)
        identity = identity_from_popen(process, command)
        return cls(NativeJob._managed(process, identity, None, log_dir=Path(log_dir) if log_dir else None))

    def resume(self) -> None:
        if getattr(self, "_suspended", False):
            self.ensure_owned()
            native.resume_popen_process(self.process.process)
            self._suspended = False

    @staticmethod
    def _managed(process: object, identity: ProcessIdentity, handle: int | None, *, log_dir: Path | None = None) -> ManagedProcess:
        return ManagedProcess(process, identity, handle, log_dir=log_dir)

    def ensure_owned(self) -> None:
        if not identity_matches(self.identity):
            raise JenkinsError("process_identity_mismatch", f"PID {self.identity.pid} is not the owned process")

    def terminate(self, *, timeout_seconds: float = 10.0) -> TerminationProof:
        if self._terminated:
            return self.proof(requested=True)
        if self.job_handle and os.name == "nt":
            # The retained Job handle remains authoritative after its root
            # exits. Requiring a live root would strand surviving children.
            if self.poll() is None:
                self.ensure_owned()
            native.terminate_process_job(self.job_handle, timeout_seconds=timeout_seconds)
            self._job_terminal_proven = True
            self._terminal_active = 0
            self.process.wait(timeout=timeout_seconds)
            self.process.communicate(timeout=timeout_seconds)
        else:
            self.ensure_owned()
            self.process.process.terminate()
            try:
                self.process.wait(timeout=timeout_seconds)
            except subprocess.TimeoutExpired as exc:
                self.process.process.kill()
                self.process.wait(timeout=timeout_seconds)
        self._terminated = True
        return self.proof(requested=True)

    def poll(self) -> int | None:
        """Return the owned root exit code without inferring Job completion."""
        return self.process.poll()

    def wait(self, *, timeout_seconds: float | None = None, cleanup_descendants: bool = False) -> int:
        result = self.process.wait(timeout_seconds)
        if self.job_handle and os.name == "nt":
            if cleanup_descendants:
                active = native.query_job_active_processes(self.job_handle)
                self.descendant_cleanup = {"rootExitCode": result,
                    "activeProcessesAfterRootExit": active, "requested": bool(active)}
                if active:
                    self.descendant_cleanup["membersBeforeCleanup"] = native.describe_job_members(self.job_handle)
                    # A phase owns its entire subprocess tree. After its
                    # command exits, stop retained helpers before observing
                    # EOF or publishing any output closure.
                    self.terminate(timeout_seconds=timeout_seconds or 10.0)
            native.wait_for_process_job_terminal(self.job_handle, timeout_seconds=timeout_seconds or 120.0)
            self._job_terminal_proven = True
            self._terminal_active = 0
        elif not self.job_handle:
            self._job_terminal_proven = True
        if not self.process._pump_threads:
            self.process.communicate(timeout=timeout_seconds)
        else:
            for thread in self.process._pump_threads:
                thread.join(timeout=timeout_seconds)
        self._terminated = True
        return result

    def proof(self, *, requested: bool = False) -> TerminationProof:
        code = self.process.poll()
        alive = process_alive(self.identity.pid)
        active = native.query_job_active_processes(self.job_handle) if self.job_handle and os.name == "nt" else (self._terminal_active if self._job_terminal_proven else (0 if alive is False and os.name != "nt" else None))
        children_gone = active == 0 and self._job_terminal_proven
        return TerminationProof(self.identity, requested, code, active,
                                self.process._stdout_eof,
                                self.process._stderr_eof,
                                children_gone)

    def close(self) -> None:
        if self.job_handle and os.name == "nt":
            native.close_process_job(self.job_handle)
            self.job_handle = None
        self.process.close()

    def __enter__(self) -> "NativeJob":
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        self.close()

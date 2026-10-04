from __future__ import annotations

import os
import subprocess
import hashlib
import ctypes
from ctypes import wintypes
from dataclasses import dataclass
from pathlib import Path

from tools.jenkins.contracts import JenkinsError


@dataclass(frozen=True, slots=True)
class ProcessIdentity:
    """Stable identity captured at launch; PID alone is never sufficient."""

    pid: int
    creation_time: str
    executable: str = ""
    command_digest: str = ""

    def __post_init__(self) -> None:
        if self.pid <= 0:
            raise JenkinsError("invalid_process_identity", "process PID must be positive")
        if not self.creation_time:
            raise JenkinsError("invalid_process_identity", "process creation time is required")

    def to_dict(self) -> dict[str, object]:
        return {
            "pid": self.pid,
            "creationTime": self.creation_time,
            "executable": self.executable,
            "commandDigest": self.command_digest,
        }


_QUERY_LIMITED = 0x1000
_STILL_ACTIVE = 259


def _kernel32():
    api = ctypes.WinDLL("kernel32", use_last_error=True)
    api.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    api.OpenProcess.restype = wintypes.HANDLE
    api.CloseHandle.argtypes = [wintypes.HANDLE]
    api.CloseHandle.restype = wintypes.BOOL
    api.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
    api.GetProcessTimes.restype = wintypes.BOOL
    api.GetExitCodeProcess.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.DWORD)]
    api.GetExitCodeProcess.restype = wintypes.BOOL
    api.QueryFullProcessImageNameW.argtypes = [wintypes.HANDLE, wintypes.DWORD, wintypes.LPWSTR, ctypes.POINTER(wintypes.DWORD)]
    api.QueryFullProcessImageNameW.restype = wintypes.BOOL
    return api


def _open_query(pid: int):
    api = _kernel32()
    handle = api.OpenProcess(_QUERY_LIMITED, False, pid)
    if not handle:
        raise OSError(ctypes.get_last_error(), "OpenProcess failed")
    return api, handle


def _creation_time(pid: int) -> str:
    if os.name == "nt":
        try:
            kernel, handle = _open_query(pid)
        except OSError as exc:
            code = getattr(exc, "winerror", None) or getattr(exc, "errno", None)
            if code == 87:
                raise JenkinsError("process_identity_unavailable", f"process {pid} is not available") from exc
            raise JenkinsError("process_identity_inconclusive", f"cannot read process birth time for {pid}", retryable=True) from exc
        try:
            values = [wintypes.FILETIME() for _ in range(4)]
            if not kernel.GetProcessTimes(handle, *(ctypes.byref(value) for value in values)):
                raise OSError(ctypes.get_last_error(), "GetProcessTimes failed")
            return str((values[0].dwHighDateTime << 32) | values[0].dwLowDateTime)
        finally:
            kernel.CloseHandle(handle)
    try:
        return str(os.stat(f"/proc/{pid}").st_ctime_ns)
    except (OSError, ValueError, IndexError) as exc:
        raise JenkinsError("process_identity_unavailable", f"cannot read process identity for {pid}") from exc


def current_identity(pid: int, *, executable: str = "", command_line: tuple[str, ...] = ()) -> ProcessIdentity:
    command_digest = hashlib.sha256("\0".join(command_line).encode("utf-8")).hexdigest() if command_line else ""
    return ProcessIdentity(pid, _creation_time(pid), _executable_path(pid) if os.name == "nt" else executable, command_digest)


def _executable_path(pid: int) -> str:
    api, handle = _open_query(pid)
    try:
        size = wintypes.DWORD(32768)
        buffer = ctypes.create_unicode_buffer(size.value)
        if not api.QueryFullProcessImageNameW(handle, 0, buffer, ctypes.byref(size)):
            raise OSError(ctypes.get_last_error(), "QueryFullProcessImageNameW failed")
        return buffer.value
    finally:
        api.CloseHandle(handle)


def process_alive(pid: int) -> bool | None:
    if os.name == "nt":
        try:
            kernel, handle = _open_query(pid)
        except OSError as exc:
            code = getattr(exc, "winerror", None) or getattr(exc, "errno", None)
            if code == 87:
                return False
            return None
        try:
            code = wintypes.DWORD()
            if not kernel.GetExitCodeProcess(handle, ctypes.byref(code)): return None
            return int(code.value) == _STILL_ACTIVE
        except OSError as exc:
            if getattr(exc, "winerror", None) == 87:
                return False
            return None
        finally: kernel.CloseHandle(handle)
    try: os.kill(pid, 0); return True
    except ProcessLookupError: return False
    except PermissionError: return True
    except OSError: return None


def identity_matches(expected: ProcessIdentity, *, allow_absent: bool = False) -> bool:
    """Check PID and birth time, returning false for a reused PID."""
    alive = process_alive(expected.pid)
    if alive is False:
        return bool(allow_absent)
    if alive is None:
        raise JenkinsError("process_identity_inconclusive", f"cannot determine liveness for {expected.pid}", retryable=True)
    observed = _creation_time(expected.pid)
    if observed != expected.creation_time:
        return False
    if expected.executable:
        return os.path.normcase(_executable_path(expected.pid)) == os.path.normcase(expected.executable)
    return True


def identity_from_popen(process: subprocess.Popen[object], args: tuple[str, ...] = ()) -> ProcessIdentity:
    executable = str(args[0]) if args else ""
    return current_identity(int(process.pid), executable=executable, command_line=args)

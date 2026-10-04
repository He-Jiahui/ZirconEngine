"""Exact Windows PID/FILETIME identity; uncertainty never proves termination."""
from __future__ import annotations

import ctypes
import os
from ctypes import wintypes


def _kernel():
    if os.name != "nt":
        raise OSError("Jenkins process identity requires Windows")
    api = ctypes.WinDLL("kernel32", use_last_error=True)
    api.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    api.OpenProcess.restype = wintypes.HANDLE
    api.CloseHandle.argtypes = [wintypes.HANDLE]
    api.CloseHandle.restype = wintypes.BOOL
    api.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
    api.GetProcessTimes.restype = wintypes.BOOL
    api.GetProcessId.argtypes = [wintypes.HANDLE]
    api.GetProcessId.restype = wintypes.DWORD
    api.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
    api.WaitForSingleObject.restype = wintypes.DWORD
    return api


def _open(api, pid):
    handle = api.OpenProcess(0x1000 | 0x100000, False, pid)
    if not handle and ctypes.get_last_error() == 5:  # ERROR_ACCESS_DENIED
        # Creation time needs QUERY_LIMITED_INFORMATION alone. A reused PID
        # can therefore be rejected even when its new owner denies SYNCHRONIZE.
        # Matching identities still pass through the native wait below, which
        # must raise if this restricted handle cannot prove process exit.
        handle = api.OpenProcess(0x1000, False, pid)
    if not handle:
        error = ctypes.get_last_error()
        if error == 87:  # ERROR_INVALID_PARAMETER: PID does not exist.
            return None
        raise OSError(error, "cannot query exact process identity")
    return handle


def _birth(api, handle):
    times = [wintypes.FILETIME() for _ in range(4)]
    if not api.GetProcessTimes(handle, *(ctypes.byref(value) for value in times)):
        raise OSError(ctypes.get_last_error(), "GetProcessTimes failed")
    return str((times[0].dwHighDateTime << 32) | times[0].dwLowDateTime)


def _alive(api, handle):
    result = api.WaitForSingleObject(handle, 0)
    if result == 258:
        return True
    if result == 0:
        return False
    raise OSError(ctypes.get_last_error(), "cannot prove process exit")


def process_creation_time(pid: int) -> str:
    if pid <= 0:
        raise ValueError("PID must be positive")
    api = _kernel()
    handle = _open(api, pid)
    if handle is None:
        raise ProcessLookupError("process no longer exists")
    try:
        return _birth(api, handle)
    finally:
        api.CloseHandle(handle)


def popen_process_creation_time(process) -> str:
    api = _kernel()
    handle = int(getattr(process, "_handle", 0) or 0)
    if process.pid <= 0 or not handle:
        raise OSError("process must retain its original Windows handle")
    if api.GetProcessId(handle) != process.pid:
        raise ProcessLookupError("retained handle does not match process PID")
    return _birth(api, handle)


def process_matches_creation_time(pid: int, expected_creation_time: str) -> bool:
    if pid <= 0 or not expected_creation_time:
        raise ValueError("positive PID and exact creation time are required")
    api = _kernel()
    handle = _open(api, pid)
    if handle is None:
        return False
    try:
        return _birth(api, handle) == expected_creation_time and _alive(api, handle)
    finally:
        api.CloseHandle(handle)


def process_is_alive(pid: int) -> bool:
    if pid <= 0:
        return False
    api = _kernel()
    handle = _open(api, pid)
    if handle is None:
        return False
    try:
        return _alive(api, handle)
    finally:
        api.CloseHandle(handle)


def wait_for_process_exit(pid: int, expected_creation_time: str, *, timeout_seconds: float = 15) -> None:
    """Wait on one exact retained identity, never on a reused PID."""
    if pid <= 0 or not expected_creation_time:
        raise ValueError("positive PID and exact creation time are required")
    api = _kernel()
    handle = _open(api, pid)
    if handle is None:
        return
    try:
        if _birth(api, handle) != expected_creation_time:
            return
        result = api.WaitForSingleObject(handle, max(0, int(timeout_seconds * 1000)))
        if result == 258:
            raise TimeoutError("exact owned process has not signaled exit")
        if result != 0:
            raise OSError(ctypes.get_last_error(), "cannot prove exact owned process exit")
    finally:
        api.CloseHandle(handle)

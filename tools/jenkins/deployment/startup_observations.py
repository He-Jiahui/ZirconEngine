"""Read-only Windows observations for an abandoned controller startup."""
from __future__ import annotations

import ctypes
from ctypes import wintypes
import os
from pathlib import Path

from ..contracts import JenkinsError
from ..processes.identity import current_identity, process_alive
from .driver import _plain


def observe_departed_identity(expected: dict) -> dict:
    """An inaccessible PID is inconclusive, never evidence of its departure."""
    pid = expected["pid"]
    alive = process_alive(pid)
    if alive is False:
        return {"pid": pid, "expectedCreationTime": expected["creationTime"], "status": "absent"}
    if alive is None:
        raise JenkinsError("deployment_identity_inconclusive", "Cannot establish the old owner identity", retryable=True)
    try:
        actual = current_identity(pid).to_dict()
    except (JenkinsError, OSError) as error:
        if process_alive(pid) is False:
            return {"pid": pid, "expectedCreationTime": expected["creationTime"], "status": "absent"}
        raise JenkinsError("deployment_identity_inconclusive", "Cannot query the old owner identity", retryable=True) from error
    if str(actual["creationTime"]) == str(expected["creationTime"]):
        raise JenkinsError("deployment_owner_live", "The old lifecycle component is still alive", retryable=True)
    return {"pid": pid, "expectedCreationTime": expected["creationTime"],
            "status": "pid-reused", "observedIdentity": actual}


def runtime_owner_inventory(repository: Path) -> list[dict]:
    """Enumerate all processes without WMI or exposing process arguments."""
    if os.name != "nt":
        raise JenkinsError("deployment_inventory_unavailable", "This recovery observation requires Windows")
    class ProcessEntry(ctypes.Structure):
        _fields_ = [("size", wintypes.DWORD), ("usage", wintypes.DWORD),
                    ("pid", wintypes.DWORD), ("heap", ctypes.c_size_t),
                    ("module", wintypes.DWORD), ("threads", wintypes.DWORD),
                    ("parentPid", wintypes.DWORD), ("priority", wintypes.LONG),
                    ("flags", wintypes.DWORD), ("executableName", wintypes.WCHAR * 260)]
    api = ctypes.WinDLL("kernel32", use_last_error=True)
    api.CreateToolhelp32Snapshot.argtypes = [wintypes.DWORD, wintypes.DWORD]
    api.CreateToolhelp32Snapshot.restype = wintypes.HANDLE
    for name in ("Process32FirstW", "Process32NextW"):
        function = getattr(api, name)
        function.argtypes = [wintypes.HANDLE, ctypes.POINTER(ProcessEntry)]
        function.restype = wintypes.BOOL
    api.CloseHandle.argtypes = [wintypes.HANDLE]
    api.CloseHandle.restype = wintypes.BOOL
    snapshot = api.CreateToolhelp32Snapshot(2, 0)
    if not snapshot or snapshot == ctypes.c_void_p(-1).value:
        raise JenkinsError("deployment_inventory_unavailable", "Windows process inventory is unavailable", retryable=True)
    owners = []
    runtime = (repository / ".jenkins/runtime").absolute()
    try:
        entry = ProcessEntry()
        entry.size = ctypes.sizeof(entry)
        available = api.Process32FirstW(snapshot, ctypes.byref(entry))
        while available:
            if entry.pid != os.getpid() and entry.executableName.casefold() in {"java.exe", "javaw.exe", "python.exe", "pythonw.exe"}:
                try:
                    actual = current_identity(int(entry.pid)).to_dict()
                except (JenkinsError, OSError) as error:
                    if process_alive(int(entry.pid)) is not False:
                        raise JenkinsError("deployment_inventory_inconclusive", "A relevant process identity is inaccessible", retryable=True) from error
                else:
                    executable = Path(actual["executable"]).absolute()
                    if executable.is_relative_to(runtime):
                        owners.append(actual)
            available = api.Process32NextW(snapshot, ctypes.byref(entry))
        if ctypes.get_last_error() != 18:
            raise JenkinsError("deployment_inventory_inconclusive", "Process enumeration did not finish", retryable=True)
    finally:
        api.CloseHandle(snapshot)
    return owners


def observe_home_available(home: Path) -> dict:
    """Probe existing controller files without deleting stale owner markers."""
    _plain(home)
    if os.name != "nt" or not (home / "config.xml").is_file():
        raise JenkinsError("deployment_home_unproven", "The existing Jenkins Home is unavailable")
    api = ctypes.WinDLL("kernel32", use_last_error=True)
    api.CreateFileW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD,
                              ctypes.c_void_p, wintypes.DWORD, wintypes.DWORD, wintypes.HANDLE]
    api.CreateFileW.restype = wintypes.HANDLE
    api.CloseHandle.argtypes = [wintypes.HANDLE]
    api.CloseHandle.restype = wintypes.BOOL
    observations = []
    for name in ("config.xml", "queue.xml", ".owner"):
        path = home / name
        _plain(path)
        if not path.exists():
            continue
        handle = api.CreateFileW(str(path), 0x80000000, 0, None, 3, 0x80, None)
        if not handle or handle == ctypes.c_void_p(-1).value:
            raise JenkinsError("deployment_home_busy", "A Jenkins Home file is occupied or inaccessible", retryable=True,
                               details={"file": name, "osError": ctypes.get_last_error()})
        try:
            stat = path.stat()
            observations.append({"file": name, "bytes": stat.st_size, "modifiedNs": stat.st_mtime_ns})
        finally:
            api.CloseHandle(handle)
    return {"exclusiveReadProbes": observations, "staleMarkersDeleted": False}

"""Transfer runtime Job Object lifetime to the exact storage lease owner."""
from __future__ import annotations

import ctypes
import os
import sys
import json
import time
import secrets
from ctypes import wintypes
from pathlib import Path
from typing import Callable, Mapping

from tools.jenkins.pilot.native.windows_job_process import (
    create_atomic_kill_on_close_process, close_process_job, resume_popen_process,
    terminate_and_close_process_job,
)
from tools.jenkins.pilot.native.process_identity import (
    popen_process_creation_time, process_matches_creation_time, wait_for_process_exit,
)
from .storage import require_managed_root


def _kernel():
    if os.name != "nt":
        raise OSError("pilot runtime lifetime requires Windows Job Objects")
    dll = ctypes.WinDLL("kernel32", use_last_error=True)
    dll.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    dll.OpenProcess.restype = wintypes.HANDLE
    dll.CloseHandle.argtypes = [wintypes.HANDLE]
    dll.CloseHandle.restype = wintypes.BOOL
    dll.GetCurrentProcess.restype = wintypes.HANDLE
    dll.GetProcessTimes.argtypes = [wintypes.HANDLE, *[ctypes.POINTER(wintypes.FILETIME)] * 4]
    dll.GetProcessTimes.restype = wintypes.BOOL
    dll.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
    dll.WaitForSingleObject.restype = wintypes.DWORD
    dll.DuplicateHandle.argtypes = [wintypes.HANDLE, wintypes.HANDLE, wintypes.HANDLE,
                                   ctypes.POINTER(wintypes.HANDLE), wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    dll.DuplicateHandle.restype = wintypes.BOOL
    dll.CreateFileW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD, wintypes.LPVOID,
                               wintypes.DWORD, wintypes.DWORD, wintypes.HANDLE]
    dll.CreateFileW.restype = wintypes.HANDLE
    return dll


def _open_owner(dll, pid: int, expected_creation: str) -> int:
    if pid <= 0 or not expected_creation:
        raise ValueError("exact positive keeper PID and creation time are required")
    handle = dll.OpenProcess(0x0040 | 0x1000 | 0x100000, False, pid)
    if not handle:
        error = ctypes.get_last_error()
        if error == 87:
            raise ProcessLookupError("exact storage owner PID no longer exists")
        raise OSError(error, "cannot retain the exact storage owner process")
    creation, exit_time, kernel, user = [wintypes.FILETIME() for _ in range(4)]
    if not dll.GetProcessTimes(handle, ctypes.byref(creation), ctypes.byref(exit_time), ctypes.byref(kernel), ctypes.byref(user)):
        dll.CloseHandle(handle)
        raise OSError("cannot read the retained storage owner identity")
    actual = str((creation.dwHighDateTime << 32) | creation.dwLowDateTime)
    if actual != expected_creation:
        dll.CloseHandle(handle)
        raise ProcessLookupError("storage owner PID was reused")
    result = dll.WaitForSingleObject(handle, 0)
    if result != 258:
        error = ctypes.get_last_error()
        dll.CloseHandle(handle)
        if result == 0:
            raise ProcessLookupError("exact storage owner exited")
        raise OSError(error, "cannot prove exact storage owner liveness")
    return int(handle)


def _duplicate(dll, source_process, handle, target_process, *, close_source=False) -> int:
    target = wintypes.HANDLE()
    if not dll.DuplicateHandle(source_process, handle, target_process, ctypes.byref(target), 0, False, 2 | int(close_source)):
        raise OSError(ctypes.get_last_error(), "cannot transfer the owned runtime Job Object")
    return int(target.value)


def launch_owned(args: list[str], *, cwd: Path, env: Mapping[str, str], stream, lease: Mapping[str, object],
                 on_record: Callable[[dict[str, object]], None] | None = None,
                 log_path: Path | None = None,
                 runtime_assets: Mapping[str, object] | None = None) -> dict[str, object]:
    log_path = require_managed_root(Path(log_path if log_path is not None else stream.name), allow_root=False)
    host_argv = [sys.executable, "-B", str(Path(__file__).with_name("runtime_host.py"))]
    start_path = None
    if runtime_assets is not None:
        from .runtime_host import _validated_runtime_assets
        fixed_assets = _validated_runtime_assets(dict(runtime_assets))
        host_argv.extend(["--runtime-assets", json.dumps(fixed_assets, sort_keys=True)])
        start_path = log_path.parent / ("runtime-start-" + secrets.token_hex(16) + ".json")
        host_argv.extend(["--runtime-start-file", str(start_path)])
    dll = _kernel()
    owner = _open_owner(dll, lease["owner_pid"], lease["owner_process_creation_time"])
    job, process, remote = None, None, None
    try:
        # The existing atomic launcher establishes Job membership in CreateProcess.
        # A small host sends descendant output straight to a managed file, so no
        # pipe reader needs to survive this short-lived CLI.
        host_env = dict(env)
        host_env["JENKINS_PILOT_HOST_ARGS"] = json.dumps(args)
        host_env["JENKINS_PILOT_HOST_LOG"] = str(log_path)
        process, job = create_atomic_kill_on_close_process(
            tuple(host_argv),
            cwd=cwd, env=host_env,
        )
        remote = _duplicate(dll, dll.GetCurrentProcess(), job, owner)
        record = {"pid": process.pid, "creationTime": popen_process_creation_time(process),
                  "keeperPid": lease["owner_pid"], "keeperCreationTime": lease["owner_process_creation_time"],
                  "keeperJobHandle": remote, "logPath": str(log_path)}
        if start_path is not None:
            record.update(runtimeStartPath=str(start_path), runtimeAssets=fixed_assets)
        if on_record is not None:
            # The bootstrap has already saved a fail-closed launch intent. Save
            # the exact Job binding before this host can execute any instruction.
            on_record(record)
        # The keeper becomes the sole job-handle owner before first instruction.
        close_process_job(job)
        job = None
        if dll.WaitForSingleObject(owner, 0) != 258:
            raise ProcessLookupError("storage owner exited before runtime resume")
        resume_popen_process(process)
        return record
    except BaseException:
        if job:
            terminate_and_close_process_job(job)
        if remote:
            try:
                # A persisted record may already refer to this remote handle.
                # Retain its terminal Job until keeper exit to prevent reuse.
                borrowed = _duplicate(dll, owner, remote, dll.GetCurrentProcess())
                terminate_and_close_process_job(borrowed)
            except OSError:
                pass
        if process is not None:
            if process.poll() is None:
                process.kill()
            process.wait(timeout=15)
        raise
    finally:
        if process is not None:
            process.close()
        dll.CloseHandle(owner)


def wait_runtime_started(record: Mapping[str, object], *, preparation_seconds: float = 180) -> dict[str, object]:
    """Separate bounded host preparation from the Java readiness budget."""
    from .storage import ManagedStorage
    path = require_managed_root(Path(record["runtimeStartPath"]), allow_root=False)
    log = require_managed_root(Path(record["logPath"]), allow_root=False)
    if path.parent != log.parent or not path.name.startswith("runtime-start-"):
        raise ValueError("runtime start marker is outside its exact launch log directory")
    storage = ManagedStorage(path.parent)
    deadline = time.monotonic() + preparation_seconds
    while True:
        if not process_matches_creation_time(record["pid"], record["creationTime"]):
            raise ProcessLookupError("runtime host exited before verified Java startup")
        with storage.backend() as backend:
            if backend.exists(path.name):
                value = json.loads(backend.read_bytes(path.name, max_bytes=64 * 1024))
                break
        if time.monotonic() >= deadline:
            raise TimeoutError("runtime host asset preparation is still pending")
        time.sleep(0.1)
    if (not isinstance(value, dict) or set(value) != {"schemaVersion", "hostPid", "hostCreationTime", "childPid", "childCreationTime", "runtimeAssets"}
            or value.get("schemaVersion") != 1 or value.get("hostPid") != record["pid"]
            or value.get("hostCreationTime") != record["creationTime"]
            or value.get("runtimeAssets") != record["runtimeAssets"]):
        raise ValueError("runtime Java start marker does not match its exact launch binding")
    dll = _kernel()
    owner = _open_owner(dll, record["keeperPid"], record["keeperCreationTime"])
    child, query = None, wintypes.HANDLE()
    try:
        if not dll.DuplicateHandle(owner, record["keeperJobHandle"], dll.GetCurrentProcess(), ctypes.byref(query), 4, False, 0):
            raise OSError(ctypes.get_last_error(), "cannot query the exact runtime start Job")
        child = dll.OpenProcess(0x1000 | 0x100000, False, value["childPid"])
        if not child:
            raise OSError(ctypes.get_last_error(), "cannot retain started Java identity")
        from .native.process_identity import _birth
        if _birth(dll, child) != value["childCreationTime"] or dll.WaitForSingleObject(child, 0) != 258:
            raise ProcessLookupError("Java start marker child exited or its PID was reused")
        dll.IsProcessInJob.argtypes = [wintypes.HANDLE, wintypes.HANDLE, ctypes.POINTER(wintypes.BOOL)]
        dll.IsProcessInJob.restype = wintypes.BOOL
        member = wintypes.BOOL()
        if not dll.IsProcessInJob(child, query, ctypes.byref(member)):
            raise OSError(ctypes.get_last_error(), "cannot prove started Java Job membership")
        if not member.value:
            raise ProcessLookupError("Java start marker child is outside the exact owned Job")
        return value
    finally:
        if child:
            dll.CloseHandle(child)
        if query.value:
            dll.CloseHandle(query)
        dll.CloseHandle(owner)


def _wait_log_release(dll, record: Mapping[str, object], timeout_seconds: float = 30) -> None:
    """Check DELETE sharing without deleting or changing the owned runtime log."""
    path = require_managed_root(Path(record["logPath"]), allow_root=False)
    deadline = time.monotonic() + timeout_seconds
    while True:
        handle = dll.CreateFileW(str(path), 0x10000, 7, None, 3, 0, None)
        if handle != ctypes.c_void_p(-1).value:
            dll.CloseHandle(handle)
            return
        error = ctypes.get_last_error()
        if error == 2:
            return
        if error not in {32, 33}:
            raise OSError(error, "cannot prove runtime log release")
        if time.monotonic() >= deadline:
            raise TimeoutError("runtime log remains occupied; preserve the storage lease")
        time.sleep(0.1)


def stop_owned(record: Mapping[str, object]) -> bool:
    dll = _kernel()
    try:
        owner = _open_owner(dll, record["keeperPid"], record["keeperCreationTime"])
    except ProcessLookupError:
        # A dead keeper closes the sole kill-on-close job handle. Never act on
        # a reused PID; the exact owned root must still be checked as terminal.
        wait_for_process_exit(record["pid"], record["creationTime"])
        terminal = not process_matches_creation_time(record["pid"], record["creationTime"])
        if terminal:
            _wait_log_release(dll, record)
        return terminal
    try:
        borrowed = _duplicate(dll, owner, record["keeperJobHandle"], dll.GetCurrentProcess())
        terminate_and_close_process_job(borrowed)
        # Retain the terminal remote handle until keeper exit, so repeated stop
        # requests cannot act on a recycled numeric handle in the owner process.
        wait_for_process_exit(record["pid"], record["creationTime"])
        terminal = not process_matches_creation_time(record["pid"], record["creationTime"])
        if terminal:
            _wait_log_release(dll, record)
        return terminal
    finally:
        dll.CloseHandle(owner)

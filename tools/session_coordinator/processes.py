from __future__ import annotations

import ctypes
import os
import signal
import subprocess
import sys
import time
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True, slots=True)
class ProcessIdentity:
    pid: int
    creation_time: str
    executable: str
    command_line: tuple[str, ...]

    def to_public_dict(self) -> dict[str, object]:
        return {
            "pid": self.pid,
            "process_creation_time": self.creation_time,
            "executable": self.executable,
            "command_line": list(self.command_line),
        }


class _FileTime(ctypes.Structure):
    _fields_ = (("low", ctypes.c_uint32), ("high", ctypes.c_uint32))

    def integer(self) -> int:
        return (int(self.high) << 32) | int(self.low)


def process_is_alive(pid: int) -> bool:
    if pid <= 0:
        return False
    if os.name == "nt":
        process_query_limited_information = 0x1000
        still_active = 259
        kernel32 = ctypes.windll.kernel32
        kernel32.OpenProcess.argtypes = [ctypes.c_uint32, ctypes.c_bool, ctypes.c_uint32]
        kernel32.OpenProcess.restype = ctypes.c_void_p
        kernel32.CloseHandle.argtypes = [ctypes.c_void_p]
        kernel32.CloseHandle.restype = ctypes.c_bool
        kernel32.GetExitCodeProcess.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint32)]
        kernel32.GetExitCodeProcess.restype = ctypes.c_bool
        handle = kernel32.OpenProcess(process_query_limited_information, False, pid)
        if not handle:
            return False
        try:
            exit_code = ctypes.c_uint32()
            if not kernel32.GetExitCodeProcess(handle, ctypes.byref(exit_code)):
                return False
            return int(exit_code.value) == still_active
        finally:
            kernel32.CloseHandle(handle)
    try:
        os.kill(pid, 0)
    except OSError:
        return False
    return True


class _ProcessEntry32W(ctypes.Structure):
    _fields_ = [
        ("dwSize", ctypes.c_uint32),
        ("cntUsage", ctypes.c_uint32),
        ("th32ProcessID", ctypes.c_uint32),
        ("th32DefaultHeapID", ctypes.c_size_t),
        ("th32ModuleID", ctypes.c_uint32),
        ("cntThreads", ctypes.c_uint32),
        ("th32ParentProcessID", ctypes.c_uint32),
        ("pcPriClassBase", ctypes.c_long),
        ("dwFlags", ctypes.c_uint32),
        ("szExeFile", ctypes.c_wchar * 260),
    ]


def live_process_tree_pids(root_pid: int) -> tuple[int, ...]:
    """Return live root/descendant PIDs, retaining descendants after root exit."""
    if root_pid <= 0:
        return ()
    try:
        parents = _windows_process_parent_ids() if os.name == "nt" else _posix_process_parent_ids()
    except OSError:
        return (root_pid,) if process_is_alive(root_pid) else ()
    return _descendant_pids(root_pid, parents)


def live_cargo_process_tree_pids(root_pid: int) -> tuple[int, ...]:
    """Return live Cargo/rustc descendants plus their tool children, not control clients."""
    if root_pid <= 0:
        return ()
    try:
        if os.name == "nt":
            parents, executable_names = _windows_process_entries()
        else:
            parents = _posix_process_parent_ids()
            executable_names = _posix_process_executable_names(parents)
    except OSError:
        return ()
    candidates = _descendant_pids(root_pid, parents)
    cargo_roots = [
        pid
        for pid in candidates
        if _is_cargo_or_rustc(executable_names.get(pid, ""))
    ]
    tracked: set[int] = set()
    for cargo_root in cargo_roots:
        tracked.update(_descendant_pids(cargo_root, parents))
    return tuple(sorted(tracked))


def live_process_ids_named(*executable_names: str) -> tuple[int, ...]:
    """Return live PIDs whose executable stem exactly matches a requested name."""
    normalized = {
        Path(name).stem.casefold()
        for name in executable_names
        if str(name).strip()
    }
    if not normalized:
        return ()
    if os.name == "nt":
        _, observed_names = _windows_process_entries()
    else:
        parents = _posix_process_parent_ids()
        observed_names = _posix_process_executable_names(parents)
    return tuple(
        sorted(
            pid
            for pid, name in observed_names.items()
            if Path(name).stem.casefold() in normalized
        )
    )


class _RmUniqueProcess(ctypes.Structure):
    _fields_ = [
        ("process_id", ctypes.c_uint32),
        ("process_start_time", _FileTime),
    ]


class _RmProcessInfo(ctypes.Structure):
    _fields_ = [
        ("process", _RmUniqueProcess),
        ("app_name", ctypes.c_wchar * 256),
        ("service_short_name", ctypes.c_wchar * 64),
        ("application_type", ctypes.c_uint32),
        ("app_status", ctypes.c_ulong),
        ("terminal_session_id", ctypes.c_uint32),
        ("restartable", ctypes.c_int32),
    ]


def file_owner_process_ids(path: Path) -> tuple[int, ...]:
    """Return processes holding one file, using Windows Restart Manager."""
    if os.name != "nt":
        return live_process_ids_named("git")
    restart_manager = ctypes.WinDLL("rstrtmgr")
    session_handle = ctypes.c_uint32()
    session_key = ctypes.create_unicode_buffer(33)
    start_session = restart_manager.RmStartSession
    start_session.argtypes = [
        ctypes.POINTER(ctypes.c_uint32),
        ctypes.c_uint32,
        ctypes.c_wchar_p,
    ]
    start_session.restype = ctypes.c_uint32
    register_resources = restart_manager.RmRegisterResources
    register_resources.argtypes = [
        ctypes.c_uint32,
        ctypes.c_uint32,
        ctypes.POINTER(ctypes.c_wchar_p),
        ctypes.c_uint32,
        ctypes.c_void_p,
        ctypes.c_uint32,
        ctypes.c_void_p,
    ]
    register_resources.restype = ctypes.c_uint32
    get_list = restart_manager.RmGetList
    get_list.argtypes = [
        ctypes.c_uint32,
        ctypes.POINTER(ctypes.c_uint32),
        ctypes.POINTER(ctypes.c_uint32),
        ctypes.POINTER(_RmProcessInfo),
        ctypes.POINTER(ctypes.c_uint32),
    ]
    get_list.restype = ctypes.c_uint32
    end_session = restart_manager.RmEndSession
    end_session.argtypes = [ctypes.c_uint32]
    end_session.restype = ctypes.c_uint32
    result = int(start_session(ctypes.byref(session_handle), 0, session_key))
    if result != 0:
        raise OSError(result, "RmStartSession failed")
    try:
        resources = (ctypes.c_wchar_p * 1)(str(path.resolve()))
        result = int(
            register_resources(
                session_handle.value,
                1,
                resources,
                0,
                None,
                0,
                None,
            )
        )
        if result != 0:
            raise OSError(result, "RmRegisterResources failed")
        needed = ctypes.c_uint32()
        count = ctypes.c_uint32()
        reboot_reasons = ctypes.c_uint32()
        result = int(
            get_list(
                session_handle.value,
                ctypes.byref(needed),
                ctypes.byref(count),
                None,
                ctypes.byref(reboot_reasons),
            )
        )
        if result == 0:
            return ()
        if result != 234:
            raise OSError(result, "RmGetList sizing failed")
        entries = (_RmProcessInfo * needed.value)()
        count.value = needed.value
        result = int(
            get_list(
                session_handle.value,
                ctypes.byref(needed),
                ctypes.byref(count),
                entries,
                ctypes.byref(reboot_reasons),
            )
        )
        if result != 0:
            raise OSError(result, "RmGetList failed")
        return tuple(
            sorted(
                {
                    int(entries[index].process.process_id)
                    for index in range(count.value)
                    if int(entries[index].process.process_id) > 0
                }
            )
        )
    finally:
        end_session(session_handle.value)


def _descendant_pids(root_pid: int, parents: dict[int, int]) -> tuple[int, ...]:
    children: dict[int, list[int]] = defaultdict(list)
    for pid, parent_pid in parents.items():
        children[parent_pid].append(pid)
    live: set[int] = set()
    pending = [root_pid]
    while pending:
        current = pending.pop()
        if current in parents:
            live.add(current)
        pending.extend(child for child in children.get(current, ()) if child not in live)
    return tuple(sorted(live))


def _windows_process_parent_ids() -> dict[int, int]:
    return _windows_process_entries()[0]


def _windows_process_entries() -> tuple[dict[int, int], dict[int, str]]:
    snapshot_flag = 0x00000002
    invalid_handle_value = ctypes.c_void_p(-1).value
    kernel32 = ctypes.windll.kernel32
    kernel32.CreateToolhelp32Snapshot.argtypes = [ctypes.c_uint32, ctypes.c_uint32]
    kernel32.CreateToolhelp32Snapshot.restype = ctypes.c_void_p
    kernel32.Process32FirstW.argtypes = [ctypes.c_void_p, ctypes.POINTER(_ProcessEntry32W)]
    kernel32.Process32FirstW.restype = ctypes.c_bool
    kernel32.Process32NextW.argtypes = [ctypes.c_void_p, ctypes.POINTER(_ProcessEntry32W)]
    kernel32.Process32NextW.restype = ctypes.c_bool
    kernel32.CloseHandle.argtypes = [ctypes.c_void_p]
    kernel32.CloseHandle.restype = ctypes.c_bool
    handle = kernel32.CreateToolhelp32Snapshot(snapshot_flag, 0)
    if not handle or handle == invalid_handle_value:
        raise OSError(ctypes.get_last_error(), "CreateToolhelp32Snapshot failed")
    try:
        entry = _ProcessEntry32W()
        entry.dwSize = ctypes.sizeof(_ProcessEntry32W)
        if not kernel32.Process32FirstW(handle, ctypes.byref(entry)):
            raise OSError(ctypes.get_last_error(), "Process32FirstW failed")
        parents: dict[int, int] = {}
        executable_names: dict[int, str] = {}
        while True:
            parents[int(entry.th32ProcessID)] = int(entry.th32ParentProcessID)
            executable_names[int(entry.th32ProcessID)] = str(entry.szExeFile)
            if not kernel32.Process32NextW(handle, ctypes.byref(entry)):
                return parents, executable_names
    finally:
        kernel32.CloseHandle(handle)


def _posix_process_parent_ids() -> dict[int, int]:
    parents: dict[int, int] = {}
    proc = Path("/proc")
    if not proc.exists():
        return parents
    for entry in proc.iterdir():
        if not entry.name.isdecimal():
            continue
        try:
            payload = (entry / "stat").read_text(encoding="utf-8")
            closing_parenthesis = payload.rfind(")")
            fields = payload[closing_parenthesis + 2 :].split()
            parents[int(entry.name)] = int(fields[1])
        except (OSError, ValueError, IndexError):
            continue
    return parents


def _posix_process_executable_names(parents: dict[int, int]) -> dict[int, str]:
    names: dict[int, str] = {}
    for pid in parents:
        try:
            names[pid] = Path(f"/proc/{pid}/comm").read_text(encoding="utf-8").strip()
        except OSError:
            continue
    return names


def _is_cargo_or_rustc(executable_name: str) -> bool:
    return Path(executable_name).stem.casefold() in {"cargo", "rustc"}


def process_creation_time(pid: int) -> str:
    if pid <= 0:
        raise ValueError("PID must be positive")
    if os.name != "nt":
        stat = Path(f"/proc/{pid}/stat")
        if stat.exists():
            return stat.read_text(encoding="utf-8").split()[21]
        return "unknown"
    query = 0x1000
    kernel32 = ctypes.windll.kernel32
    kernel32.OpenProcess.argtypes = [ctypes.c_uint32, ctypes.c_bool, ctypes.c_uint32]
    kernel32.OpenProcess.restype = ctypes.c_void_p
    kernel32.GetProcessTimes.argtypes = [
        ctypes.c_void_p,
        ctypes.POINTER(_FileTime),
        ctypes.POINTER(_FileTime),
        ctypes.POINTER(_FileTime),
        ctypes.POINTER(_FileTime),
    ]
    kernel32.GetProcessTimes.restype = ctypes.c_bool
    kernel32.CloseHandle.argtypes = [ctypes.c_void_p]
    handle = kernel32.OpenProcess(query, False, pid)
    if not handle:
        raise OSError(ctypes.get_last_error(), "OpenProcess failed")
    try:
        created = _FileTime()
        exited = _FileTime()
        kernel = _FileTime()
        user = _FileTime()
        if not kernel32.GetProcessTimes(
            handle,
            ctypes.byref(created),
            ctypes.byref(exited),
            ctypes.byref(kernel),
            ctypes.byref(user),
        ):
            raise OSError(ctypes.get_last_error(), "GetProcessTimes failed")
        return str(created.integer())
    finally:
        kernel32.CloseHandle(handle)


def popen_process_creation_time(process: subprocess.Popen[object]) -> str:
    """Read creation identity from Popen's retained Windows process handle."""
    if process.pid <= 0:
        raise ValueError("Popen PID must be positive")
    if os.name != "nt":
        return process_creation_time(process.pid)
    handle = int(getattr(process, "_handle", 0) or 0)
    if not handle:
        raise OSError("Popen did not retain a Windows process handle")
    kernel32 = ctypes.windll.kernel32
    kernel32.GetProcessId.argtypes = [ctypes.c_void_p]
    kernel32.GetProcessId.restype = ctypes.c_uint32
    if int(kernel32.GetProcessId(handle)) != process.pid:
        raise ProcessLookupError("Popen process handle does not match its recorded PID")
    return _windows_handle_creation_time(handle)


def confirm_kill_on_close_job_terminated(
    root_pid: int, expected_creation_time: str, *, timeout_seconds: float = 10.0
) -> None:
    """Confirm that closing the Coordinator-owned job ended its original root."""
    if os.name != "nt":
        terminate_process_tree(
            root_pid,
            expected_creation_time,
            timeout_seconds=timeout_seconds,
        )
        return
    handle = _open_windows_process_handle(root_pid)
    if not handle:
        return
    try:
        if _windows_handle_creation_time(handle) != expected_creation_time:
            return
        deadline = time.monotonic() + timeout_seconds
        while _windows_handle_is_alive(handle):
            if time.monotonic() >= deadline:
                raise TimeoutError(
                    f"Kill-on-close job root PID {root_pid} remained alive"
                )
            time.sleep(0.05)
    finally:
        _close_windows_process_handle(handle)


def terminate_process_tree(
    root_pid: int, expected_creation_time: str, *, timeout_seconds: float = 10.0
) -> None:
    """Terminate one process tree only while its durable root identity still matches."""
    if root_pid <= 0 or not expected_creation_time:
        raise ValueError("A positive PID and creation time are required")
    if os.name == "nt":
        _terminate_windows_process_tree(
            root_pid,
            expected_creation_time,
            timeout_seconds=timeout_seconds,
        )
        return
    if not process_is_alive(root_pid):
        return
    pids = live_process_tree_pids(root_pid)
    if process_creation_time(root_pid) != expected_creation_time:
        raise ProcessLookupError(
            f"PID {root_pid} changed identity before termination"
        )
    for pid in reversed(pids):
        try:
            os.kill(pid, signal.SIGTERM)
        except ProcessLookupError:
            continue
    deadline = time.monotonic() + timeout_seconds
    while process_is_alive(root_pid) and time.monotonic() < deadline:
        time.sleep(0.05)
    if process_is_alive(root_pid):
        raise TimeoutError(f"Process tree rooted at PID {root_pid} did not terminate")


def _terminate_windows_process_tree(
    root_pid: int, expected_creation_time: str, *, timeout_seconds: float
) -> None:
    deadline = time.monotonic() + timeout_seconds
    root_handle = _open_windows_process_handle(root_pid)
    if not root_handle:
        if not live_process_tree_pids(root_pid):
            return
        raise ProcessLookupError(
            f"PID {root_pid} exited while descendants still require identity recovery"
        )
    handles: dict[int, int] = {root_pid: root_handle}
    suspended_handles: list[int] = []
    termination_started = False
    try:
        observed_creation_time = _windows_handle_creation_time(root_handle)
        if observed_creation_time != expected_creation_time:
            raise ProcessLookupError(
                f"PID {root_pid} no longer matches its recorded process creation time"
            )
        if _suspend_windows_handle_if_alive(root_handle):
            suspended_handles.append(root_handle)
        while True:
            first_snapshot = _windows_process_parent_ids()
            descendant_pids = tuple(
                pid
                for pid in _descendant_pids(root_pid, first_snapshot)
                if pid not in handles
            )
            if not descendant_pids:
                break
            opened_handles = {
                pid: _open_windows_process_handle(pid) for pid in descendant_pids
            }
            try:
                confirmed = set(
                    _descendant_pids(root_pid, _windows_process_parent_ids())
                )
                for pid, handle in opened_handles.items():
                    if not handle:
                        if pid in confirmed:
                            raise OSError(
                                ctypes.get_last_error(),
                                f"OpenProcess failed for benchmark descendant {pid}",
                            )
                        continue
                    if pid in confirmed and _suspend_windows_handle_if_alive(handle):
                        handles[pid] = handle
                        suspended_handles.append(handle)
                        opened_handles[pid] = 0
            finally:
                for handle in opened_handles.values():
                    if handle:
                        _close_windows_process_handle(handle)
            if time.monotonic() >= deadline:
                raise TimeoutError(
                    f"Process tree rooted at PID {root_pid} could not be frozen"
                )
        termination_started = True
        for pid, handle in reversed(tuple(handles.items())):
            _terminate_windows_handle(handle)
        while any(_windows_handle_is_alive(handle) for handle in handles.values()):
            if time.monotonic() >= deadline:
                raise TimeoutError(
                    f"Process tree rooted at PID {root_pid} did not terminate"
                )
            time.sleep(0.05)
    except BaseException:
        if termination_started:
            for handle in reversed(tuple(handles.values())):
                try:
                    _terminate_windows_handle(handle)
                except OSError:
                    continue
        else:
            for handle in reversed(suspended_handles):
                try:
                    _resume_windows_handle(handle)
                except OSError:
                    continue
        raise
    finally:
        for handle in reversed(tuple(handles.values())):
            _close_windows_process_handle(handle)


def _open_windows_process_handle(pid: int) -> int:
    process_terminate = 0x0001
    process_suspend_resume = 0x0800
    process_query_limited_information = 0x1000
    synchronize = 0x00100000
    kernel32 = ctypes.windll.kernel32
    kernel32.OpenProcess.argtypes = [ctypes.c_uint32, ctypes.c_bool, ctypes.c_uint32]
    kernel32.OpenProcess.restype = ctypes.c_void_p
    return int(
        kernel32.OpenProcess(
            process_terminate
            | process_suspend_resume
            | process_query_limited_information
            | synchronize,
            False,
            pid,
        )
        or 0
    )


def _windows_handle_creation_time(handle: int) -> str:
    kernel32 = ctypes.windll.kernel32
    kernel32.GetProcessTimes.argtypes = [
        ctypes.c_void_p,
        ctypes.POINTER(_FileTime),
        ctypes.POINTER(_FileTime),
        ctypes.POINTER(_FileTime),
        ctypes.POINTER(_FileTime),
    ]
    kernel32.GetProcessTimes.restype = ctypes.c_bool
    created = _FileTime()
    exited = _FileTime()
    kernel = _FileTime()
    user = _FileTime()
    if not kernel32.GetProcessTimes(
        handle,
        ctypes.byref(created),
        ctypes.byref(exited),
        ctypes.byref(kernel),
        ctypes.byref(user),
    ):
        raise OSError(ctypes.get_last_error(), "GetProcessTimes failed")
    return str(created.integer())


def _windows_handle_is_alive(handle: int) -> bool:
    still_active = 259
    kernel32 = ctypes.windll.kernel32
    kernel32.GetExitCodeProcess.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint32)]
    kernel32.GetExitCodeProcess.restype = ctypes.c_bool
    exit_code = ctypes.c_uint32()
    if not kernel32.GetExitCodeProcess(handle, ctypes.byref(exit_code)):
        raise OSError(ctypes.get_last_error(), "GetExitCodeProcess failed")
    return int(exit_code.value) == still_active


def _terminate_windows_handle(handle: int) -> None:
    if not _windows_handle_is_alive(handle):
        return
    kernel32 = ctypes.windll.kernel32
    kernel32.TerminateProcess.argtypes = [ctypes.c_void_p, ctypes.c_uint32]
    kernel32.TerminateProcess.restype = ctypes.c_bool
    if not kernel32.TerminateProcess(handle, 1) and _windows_handle_is_alive(handle):
        raise OSError(ctypes.get_last_error(), "TerminateProcess failed")


def _suspend_windows_handle(handle: int) -> None:
    ntdll = ctypes.windll.ntdll
    ntdll.NtSuspendProcess.argtypes = [ctypes.c_void_p]
    ntdll.NtSuspendProcess.restype = ctypes.c_long
    status = int(ntdll.NtSuspendProcess(handle))
    if status != 0:
        raise OSError(status, "NtSuspendProcess failed")


def _suspend_windows_handle_if_alive(handle: int) -> bool:
    if not _windows_handle_is_alive(handle):
        return False
    try:
        _suspend_windows_handle(handle)
    except OSError:
        if not _windows_handle_is_alive(handle):
            return False
        raise
    return True


def _resume_windows_handle(handle: int) -> None:
    ntdll = ctypes.windll.ntdll
    ntdll.NtResumeProcess.argtypes = [ctypes.c_void_p]
    ntdll.NtResumeProcess.restype = ctypes.c_long
    status = int(ntdll.NtResumeProcess(handle))
    if status != 0:
        raise OSError(status, "NtResumeProcess failed")


def _close_windows_process_handle(handle: int) -> None:
    kernel32 = ctypes.windll.kernel32
    kernel32.CloseHandle.argtypes = [ctypes.c_void_p]
    kernel32.CloseHandle.restype = ctypes.c_bool
    kernel32.CloseHandle(handle)


def current_process_identity() -> ProcessIdentity:
    return ProcessIdentity(
        pid=os.getpid(),
        creation_time=process_creation_time(os.getpid()),
        executable=str(Path(sys.executable).resolve()),
        command_line=tuple(str(value) for value in sys.argv),
    )


def process_command_line(pid: int) -> tuple[str, ...]:
    """Best-effort read used only for identity verification, never execution."""
    if pid == os.getpid():
        return tuple(str(value) for value in sys.argv)
    if os.name == "nt":
        escaped = str(pid)
        result = subprocess.run(
            [
                "powershell.exe",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                f"(Get-CimInstance Win32_Process -Filter 'ProcessId={escaped}').CommandLine",
            ],
            check=False,
            capture_output=True,
            text=True,
            timeout=5,
        )
        value = result.stdout.strip()
        return (value,) if result.returncode == 0 and value else ()
    path = Path(f"/proc/{pid}/cmdline")
    if not path.exists():
        return ()
    return tuple(part.decode("utf-8", errors="replace") for part in path.read_bytes().split(b"\0") if part)

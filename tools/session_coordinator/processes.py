from __future__ import annotations

import ctypes
import os
import subprocess
import sys
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

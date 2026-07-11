from __future__ import annotations

import ctypes
import os
import subprocess
import sys
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
        kernel32 = ctypes.windll.kernel32
        kernel32.OpenProcess.argtypes = [ctypes.c_uint32, ctypes.c_bool, ctypes.c_uint32]
        kernel32.OpenProcess.restype = ctypes.c_void_p
        kernel32.CloseHandle.argtypes = [ctypes.c_void_p]
        kernel32.CloseHandle.restype = ctypes.c_bool
        handle = kernel32.OpenProcess(process_query_limited_information, False, pid)
        if not handle:
            return False
        kernel32.CloseHandle(handle)
        return True
    try:
        os.kill(pid, 0)
    except OSError:
        return False
    return True


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

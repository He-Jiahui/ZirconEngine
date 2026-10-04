"""Fail-closed process identity helpers for the formal tray runtime."""
from __future__ import annotations

import os
from pathlib import Path


def identity(pid: int) -> dict | None:
    if os.name != "nt":
        return None
    import ctypes
    from ctypes import wintypes
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    PROCESS_QUERY_LIMITED_INFORMATION = 0x1000
    handle = kernel.OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, False, pid)
    if not handle: return None
    try:
        created = wintypes.FILETIME(); exited = wintypes.FILETIME(); kernel_time = wintypes.FILETIME(); user = wintypes.FILETIME()
        if not kernel.GetProcessTimes(handle, ctypes.byref(created), ctypes.byref(exited), ctypes.byref(kernel_time), ctypes.byref(user)): return None
        code = wintypes.DWORD()
        if not kernel.GetExitCodeProcess(handle, ctypes.byref(code)) or code.value != 259: return None
        size = wintypes.DWORD(32768); buf = ctypes.create_unicode_buffer(size.value)
        if not kernel.QueryFullProcessImageNameW(handle, 0, buf, ctypes.byref(size)): return None
        birth = (created.dwHighDateTime << 32) | created.dwLowDateTime
        return {"pid": pid, "birth": str(birth), "executable": str(Path(buf.value).resolve())}
    finally: kernel.CloseHandle(handle)


def terminate_owned(record: dict) -> bool:
    if os.name != "nt": return False
    current = identity(int(record.get("pid", -1)))
    if not current or current.get("birth") != record.get("birth") or current.get("executable") != record.get("executable"):
        return False
    import ctypes
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    handle = kernel.OpenProcess(0x0001 | 0x1000, False, int(record["pid"]))
    if not handle: return False
    try: return bool(kernel.TerminateProcess(handle, 1))
    finally: kernel.CloseHandle(handle)

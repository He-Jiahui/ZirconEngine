"""Read-only OS observations used by deployment recovery.

The helpers never persist state and never accept caller-supplied command lines.
"""
from __future__ import annotations

import ctypes
import hashlib
import os
import socket
from ctypes import wintypes
from pathlib import Path

from ..contracts import JenkinsError
from ..processes.identity import current_identity


def _fail(message: str, *, retryable: bool = True) -> JenkinsError:
    return JenkinsError("live_observation_failed", message, retryable=retryable)


def _windows_command_line(pid: int) -> tuple[str, ...]:
    if os.name != "nt":
        raw = Path(f"/proc/{pid}/cmdline").read_bytes().rstrip(b"\0")
        if not raw:
            raise _fail("process command line is unavailable")
        return tuple(x.decode("utf-8", "strict") for x in raw.split(b"\0"))
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    kernel.OpenProcess.restype = wintypes.HANDLE
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel.CloseHandle.restype = wintypes.BOOL
    kernel.LocalFree.argtypes = [wintypes.HLOCAL]
    kernel.LocalFree.restype = wintypes.HLOCAL
    if pid == os.getpid():
        get_command_line = kernel.GetCommandLineW
        get_command_line.restype = wintypes.LPWSTR
        text = get_command_line()
        shell = ctypes.WinDLL("shell32", use_last_error=True)
        shell.CommandLineToArgvW.argtypes = [wintypes.LPCWSTR, ctypes.POINTER(ctypes.c_int)]
        shell.CommandLineToArgvW.restype = ctypes.POINTER(wintypes.LPWSTR)
        argc = ctypes.c_int()
        argv = shell.CommandLineToArgvW(text, ctypes.byref(argc))
        if not argv:
            raise _fail("process command line cannot be parsed")
        try:
            values = ctypes.cast(argv, ctypes.POINTER(wintypes.LPWSTR))
            return tuple(values[i] for i in range(argc.value))
        finally:
            kernel.LocalFree(argv)
    handle = kernel.OpenProcess(0x1000, False, wintypes.DWORD(pid))
    if not handle:
        raise _fail(f"cannot open process {pid}")
    try:
        # NtQueryInformationProcess(ProcessCommandLineInformation=60) returns
        # a UNICODE_STRING allocated in the target process address space.
        ntdll = ctypes.WinDLL("ntdll")
        query = ntdll.NtQueryInformationProcess
        query.argtypes = [wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p,
                          wintypes.ULONG, ctypes.POINTER(wintypes.ULONG)]
        query.restype = wintypes.LONG
        class UnicodeString(ctypes.Structure):
            _fields_ = [("Length", wintypes.USHORT), ("MaximumLength", wintypes.USHORT),
                        ("Buffer", wintypes.LPWSTR)]
        size = wintypes.ULONG(4096)
        while size.value <= 1024 * 1024:
            buffer = ctypes.create_string_buffer(size.value)
            returned = wintypes.ULONG()
            status = query(handle, 60, ctypes.byref(buffer), size,
                           ctypes.byref(returned))
            if status == 0:
                value = ctypes.cast(ctypes.byref(buffer), ctypes.POINTER(UnicodeString)).contents
                header = ctypes.sizeof(UnicodeString)
                if value.Length == 0 or value.Length > size.value - header:
                    break
                # ProcessCommandLineInformation returns the UNICODE_STRING
                # followed by the command line in the caller's buffer.  Do
                # not dereference an untrusted target pointer.
                text = ctypes.wstring_at(ctypes.addressof(buffer) + ctypes.sizeof(UnicodeString),
                                         value.Length // ctypes.sizeof(wintypes.WCHAR))
                shell = ctypes.WinDLL("shell32", use_last_error=True)
                shell.CommandLineToArgvW.argtypes = [wintypes.LPCWSTR, ctypes.POINTER(ctypes.c_int)]
                shell.CommandLineToArgvW.restype = ctypes.POINTER(wintypes.LPWSTR)
                argc = ctypes.c_int()
                argv = shell.CommandLineToArgvW(text, ctypes.byref(argc))
                if not argv:
                    raise _fail("process command line cannot be parsed")
                try:
                    values = ctypes.cast(argv, ctypes.POINTER(wintypes.LPWSTR))
                    return tuple(values[i] for i in range(argc.value))
                finally:
                    kernel.LocalFree(argv)
            # STATUS_INFO_LENGTH_MISMATCH; grow the private buffer.
            if status != -1073741820:
                break
            size.value *= 2
        raise _fail("process command line query failed")
    finally:
        kernel.CloseHandle(handle)


def _digest_args(args: tuple[str, ...]) -> str:
    return hashlib.sha256("\0".join(args).encode("utf-8")).hexdigest()


def observe_live_identity(expected: dict) -> dict:
    """Observe and verify a live process without exposing its argv."""
    if not isinstance(expected, dict) or type(expected.get("pid")) is not int or expected["pid"] <= 0:
        raise _fail("expected process identity is invalid", retryable=False)
    pid = expected["pid"]
    try:
        before = current_identity(pid).to_dict()
        args = _windows_command_line(pid)
        command_digest = _digest_args(args)
        after = current_identity(pid).to_dict()
    except JenkinsError:
        raise
    except Exception as exc:
        raise _fail("process identity is inaccessible") from exc
    if before.get("pid") != after.get("pid") or before.get("creationTime") != after.get("creationTime") \
            or os.path.normcase(str(before.get("executable"))) != os.path.normcase(str(after.get("executable"))):
        raise _fail("process identity changed during observation")
    observed = {"pid": after["pid"], "creationTime": after["creationTime"],
                "executable": after.get("executable", ""), "commandDigest": command_digest}
    if (str(expected.get("creationTime")) != str(observed["creationTime"])
            or os.path.normcase(str(expected.get("executable", ""))) != os.path.normcase(str(observed["executable"]))
            or str(expected.get("commandDigest", "")) != command_digest):
        raise _fail("process identity does not match expected birth, executable, or command digest", retryable=False)
    return observed


class _TcpRowOwnerPid(ctypes.Structure):
    _fields_ = [("state", wintypes.DWORD), ("localAddr", wintypes.DWORD),
                ("localPort", wintypes.DWORD), ("remoteAddr", wintypes.DWORD),
                ("remotePort", wintypes.DWORD), ("owningPid", wintypes.DWORD)]


def observe_endpoint_owner(address: str, port: int, expected_pid: int) -> dict:
    """Prove the unique IPv4 listener owner for the local endpoint."""
    if address != "127.0.0.1" or type(port) is not int or not 1 <= port <= 65535:
        raise _fail("only the loopback endpoint is permitted", retryable=False)
    if type(expected_pid) is not int or expected_pid <= 0:
        raise _fail("expected listener PID is invalid", retryable=False)
    if os.name != "nt":
        raise _fail("listener ownership observation requires Windows")
    ip = ctypes.WinDLL("iphlpapi", use_last_error=True)
    fn = ip.GetExtendedTcpTable
    fn.argtypes = [ctypes.c_void_p, ctypes.POINTER(wintypes.ULONG), wintypes.BOOL,
                   wintypes.ULONG, wintypes.ULONG, wintypes.ULONG]
    fn.restype = wintypes.ULONG
    size = wintypes.ULONG(0)
    af = 2  # AF_INET
    table_class = 3  # TCP_TABLE_OWNER_PID_LISTENER
    err = fn(None, ctypes.byref(size), False, af, table_class, 0)
    if err not in (0, 122):
        raise _fail("listener ownership table is unavailable")
    buffer = ctypes.create_string_buffer(size.value)
    err = fn(ctypes.byref(buffer), ctypes.byref(size), False, af, table_class, 0)
    if err != 0 or size.value < ctypes.sizeof(wintypes.DWORD):
        raise _fail("listener ownership table query failed")
    count = ctypes.cast(ctypes.byref(buffer), ctypes.POINTER(wintypes.DWORD)).contents.value
    header = ctypes.sizeof(wintypes.DWORD)
    row_size = ctypes.sizeof(_TcpRowOwnerPid)
    if count > (size.value - header) // row_size:
        raise _fail("listener ownership table count exceeds returned buffer")
    rows = ctypes.cast(ctypes.addressof(buffer) + header,
                       ctypes.POINTER(_TcpRowOwnerPid * count)).contents
    matches = []
    for row in rows:
        local_port = socket.ntohs(row.localPort & 0xFFFF)
        local_addr = socket.inet_ntoa(int(row.localAddr).to_bytes(4, "little"))
        if local_port == port and local_addr == address:
            matches.append(int(row.owningPid))
    if len(matches) != 1 or matches[0] != expected_pid:
        raise _fail("listener owner is missing, ambiguous, or belongs to another process", retryable=False)
    return {"address": address, "port": port, "pid": expected_pid, "ownerCount": 1}


__all__ = ["observe_live_identity", "observe_endpoint_owner"]

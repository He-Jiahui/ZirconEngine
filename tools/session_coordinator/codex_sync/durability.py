from __future__ import annotations

import os
from pathlib import Path


def flush_directory(path: str | Path) -> None:
    """Flush one existing directory's namespace metadata or fail closed."""
    directory = Path(path).resolve(strict=True)
    if not directory.is_dir():
        raise NotADirectoryError(str(directory))
    if os.name == "nt":
        _flush_windows_directory(directory)
        return

    flags = os.O_RDONLY
    flags |= getattr(os, "O_DIRECTORY", 0)
    flags |= getattr(os, "O_CLOEXEC", 0)
    descriptor = os.open(directory, flags)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def _flush_windows_directory(directory: Path) -> None:
    import ctypes
    from ctypes import wintypes

    generic_write = 0x40000000
    share_all = 0x00000001 | 0x00000002 | 0x00000004
    open_existing = 3
    backup_semantics = 0x02000000

    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    create_file = kernel32.CreateFileW
    create_file.argtypes = (
        wintypes.LPCWSTR,
        wintypes.DWORD,
        wintypes.DWORD,
        wintypes.LPVOID,
        wintypes.DWORD,
        wintypes.DWORD,
        wintypes.HANDLE,
    )
    create_file.restype = wintypes.HANDLE
    flush_file_buffers = kernel32.FlushFileBuffers
    flush_file_buffers.argtypes = (wintypes.HANDLE,)
    flush_file_buffers.restype = wintypes.BOOL
    close_handle = kernel32.CloseHandle
    close_handle.argtypes = (wintypes.HANDLE,)
    close_handle.restype = wintypes.BOOL

    handle = create_file(
        str(directory),
        generic_write,
        share_all,
        None,
        open_existing,
        backup_semantics,
        None,
    )
    if handle == wintypes.HANDLE(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())

    error: OSError | None = None
    try:
        if not flush_file_buffers(handle):
            error = ctypes.WinError(ctypes.get_last_error())
    finally:
        if not close_handle(handle) and error is None:
            error = ctypes.WinError(ctypes.get_last_error())
    if error is not None:
        raise error

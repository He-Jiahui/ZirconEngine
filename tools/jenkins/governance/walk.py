"""Windows metadata traversal with retained non-reparse ancestor handles."""
from __future__ import annotations

from contextlib import contextmanager
import ctypes
from ctypes import wintypes
import os
from pathlib import Path
import stat
import time

from .paths import linked, physical


@contextmanager
def directory_handle(path):
    api = ctypes.WinDLL("kernel32", use_last_error=True)
    api.CreateFileW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD, ctypes.c_void_p,
                               wintypes.DWORD, wintypes.DWORD, ctypes.c_void_p]
    api.CreateFileW.restype = wintypes.HANDLE
    api.CloseHandle.argtypes = [wintypes.HANDLE]
    api.CloseHandle.restype = wintypes.BOOL
    api.GetFileInformationByHandleEx.argtypes = [wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD]
    api.GetFileInformationByHandleEx.restype = wintypes.BOOL
    # READ sharing only prevents both a rename and conversion to a reparse
    # point while scandir uses this pathname. Ancestors stay held through DFS.
    handle = api.CreateFileW("\\\\?\\" + str(path), 0x81, 1, None, 3, 0x02200000, None)
    if handle == wintypes.HANDLE(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        attributes = (wintypes.DWORD * 2)()
        if not api.GetFileInformationByHandleEx(handle, 9, attributes, ctypes.sizeof(attributes)):
            raise ctypes.WinError(ctypes.get_last_error())
        if attributes[0] & 0x400 or not attributes[0] & 0x10:
            raise ValueError("scan_directory_reparse_or_type_changed")
        yield handle
    finally:
        api.CloseHandle(handle)



class WalkBudgetExceeded(RuntimeError):
    pass


class WalkBudget:
    def __init__(self, *, entries=100000, directories=10000, root_seconds=120):
        self.limits = {"entries": entries, "directories": directories, "rootSeconds": root_seconds}

    def begin_root(self):
        self.counts = {"entries": 0, "directories": 0}
        self.deadline = time.monotonic() + self.limits["rootSeconds"]

    def check(self, kind=None):
        if time.monotonic() >= self.deadline:
            raise WalkBudgetExceeded("root-time-budget")
        if kind:
            if self.counts[kind] >= self.limits[kind]:
                raise WalkBudgetExceeded("root-" + kind + "-budget")
            self.counts[kind] += 1


def walk(roots, *, error, link, budget=None, descend=None):
    def visit(directory):
        if budget:
            budget.check()
        if descend is not None and not descend(directory):
            return
        if budget:
            budget.check("directories")
        try:
            with directory_handle(directory):
                with os.scandir(directory) as iterator:
                    entries = []
                    for entry in iterator:
                        if budget:
                            budget.check("entries")
                        entries.append(entry)
                yield directory, entries
                for entry in entries:
                    if budget:
                        budget.check()
                    try:
                        info = entry.stat(follow_symlinks=False)
                        if linked(info):
                            link(Path(entry.path))
                        elif stat.S_ISDIR(info.st_mode):
                            yield from visit(Path(entry.path))
                    except (OSError, ValueError, RecursionError) as failure:
                        error(Path(entry.path), failure)
        except (OSError, ValueError, RecursionError) as failure:
            error(directory, failure)
    for root in roots:
        try:
            yield from visit(physical(root))
        except (OSError, ValueError) as failure:
            error(root, failure)

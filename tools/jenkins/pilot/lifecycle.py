"""Serialize mutations of one pilot runtime across Windows processes."""
from __future__ import annotations

import ctypes
import hashlib
import os
from contextlib import contextmanager
from functools import wraps
from pathlib import Path

from .contracts import PilotError
from .storage import DEFAULT_PILOT_ROOT, ManagedStorage, require_managed_root

LIFECYCLE_WAIT_SECONDS = 150


@contextmanager
def runtime_lock(root: Path, *, timeout_seconds: float = LIFECYCLE_WAIT_SECONDS):
    root = require_managed_root(root, allow_root=False)
    if not root.is_dir():
        raise PilotError("pilot runtime must already have allocated storage")
    name = "Global\\ZirconEngine-JenkinsPilot-" + hashlib.sha256(
        os.path.normcase(str(root.resolve())).encode("utf-8")
    ).hexdigest()
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.CreateMutexW.argtypes = [ctypes.c_void_p, ctypes.c_bool, ctypes.c_wchar_p]
    kernel.CreateMutexW.restype = ctypes.c_void_p
    kernel.WaitForSingleObject.argtypes = [ctypes.c_void_p, ctypes.c_uint32]
    kernel.WaitForSingleObject.restype = ctypes.c_uint32
    kernel.ReleaseMutex.argtypes = [ctypes.c_void_p]
    kernel.ReleaseMutex.restype = ctypes.c_bool
    kernel.CloseHandle.argtypes = [ctypes.c_void_p]
    kernel.CloseHandle.restype = ctypes.c_bool
    with ManagedStorage(root).backend():
        handle = kernel.CreateMutexW(None, False, name)
        if not handle:
            raise ctypes.WinError(ctypes.get_last_error())
        acquired = False
        try:
            result = kernel.WaitForSingleObject(handle, int(timeout_seconds * 1000))
            if result in (0, 0x80):  # Normal ownership or a terminated previous owner.
                acquired = True
            elif result == 0x102:
                raise PilotError("pilot lifecycle operation remains busy; reconcile its current state")
            else:
                raise ctypes.WinError(ctypes.get_last_error())
            yield
        finally:
            try:
                if acquired and not kernel.ReleaseMutex(handle):
                    raise ctypes.WinError(ctypes.get_last_error())
            finally:
                kernel.CloseHandle(handle)


def serialized_lifecycle(function):
    @wraps(function)
    def guarded(root=DEFAULT_PILOT_ROOT, *arguments, **options):
        with runtime_lock(Path(root)):
            return function(root, *arguments, **options)
    return guarded

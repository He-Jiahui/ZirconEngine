"""Windows shell helpers and the tray's single-instance guard."""
from __future__ import annotations
import ctypes
import hashlib
import os
from ctypes import wintypes
from . import native_windows as nw


class SingleInstance:
    def __init__(self, key: str):
        self.handle = None
        self.name = "Local\\ZirconJenkinsTray-" + hashlib.sha256(key.encode()).hexdigest()[:32]

    def acquire(self) -> bool:
        if not nw.IS_WINDOWS: return True
        nw.kernel32.CreateMutexW.restype = wintypes.HANDLE
        self.handle = nw.kernel32.CreateMutexW(None, False, self.name)
        if not self.handle:
            return False
        if ctypes.get_last_error() == 183:
            nw.kernel32.CloseHandle(self.handle)
            self.handle = None
            return False
        return True

    def close(self) -> None:
        if self.handle and nw.IS_WINDOWS:
            nw.kernel32.CloseHandle(self.handle)
            self.handle = None


def identity_key(config) -> str:
    return f"{getattr(config, 'repo_root', '')}|{getattr(config, 'pilot_root', '')}"

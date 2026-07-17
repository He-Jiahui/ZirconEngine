from __future__ import annotations

import ctypes
import math
import os
import time
from collections.abc import Callable, Sequence
from dataclasses import dataclass


BURST_MIN_FREE_BYTES = 100 * 1024**3
BURST_MAX_CPU_PERCENT = 80.0
BURST_MIN_FREE_MEMORY_BYTES = 12 * 1024**3
BURST_SAMPLE_COUNT = 3


@dataclass(frozen=True)
class SystemTimes:
    """Cumulative Windows system-time counters measured in 100 ns units."""

    idle: int
    kernel: int
    user: int


@dataclass(frozen=True)
class ResourceSample:
    cpu_percent: float
    free_memory_bytes: int


@dataclass(frozen=True)
class BurstDecision:
    allowed: bool
    reason: str


class _FileTime(ctypes.Structure):
    _fields_ = [
        ("dwLowDateTime", ctypes.c_ulong),
        ("dwHighDateTime", ctypes.c_ulong),
    ]


class _MemoryStatusEx(ctypes.Structure):
    _fields_ = [
        ("dwLength", ctypes.c_ulong),
        ("dwMemoryLoad", ctypes.c_ulong),
        ("ullTotalPhys", ctypes.c_ulonglong),
        ("ullAvailPhys", ctypes.c_ulonglong),
        ("ullTotalPageFile", ctypes.c_ulonglong),
        ("ullAvailPageFile", ctypes.c_ulonglong),
        ("ullTotalVirtual", ctypes.c_ulonglong),
        ("ullAvailVirtual", ctypes.c_ulonglong),
        ("ullAvailExtendedVirtual", ctypes.c_ulonglong),
    ]


def _filetime_value(value: _FileTime) -> int:
    return (int(value.dwHighDateTime) << 32) | int(value.dwLowDateTime)


def read_windows_system_times() -> SystemTimes:
    if os.name != "nt":
        raise OSError("Windows resource probe is unavailable on this platform")
    idle = _FileTime()
    kernel = _FileTime()
    user = _FileTime()
    if not ctypes.windll.kernel32.GetSystemTimes(
        ctypes.byref(idle), ctypes.byref(kernel), ctypes.byref(user)
    ):
        raise OSError(ctypes.get_last_error(), "GetSystemTimes failed")
    return SystemTimes(
        idle=_filetime_value(idle),
        kernel=_filetime_value(kernel),
        user=_filetime_value(user),
    )


def read_windows_free_memory() -> int:
    if os.name != "nt":
        raise OSError("Windows resource probe is unavailable on this platform")
    memory = _MemoryStatusEx()
    memory.dwLength = ctypes.sizeof(_MemoryStatusEx)
    if not ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(memory)):
        raise OSError(ctypes.get_last_error(), "GlobalMemoryStatusEx failed")
    return int(memory.ullAvailPhys)


class WindowsResourceProbe:
    """Collect one bounded CPU/memory sample without starting a worker process."""

    def __init__(
        self,
        *,
        interval_seconds: float = 0.2,
        read_system_times: Callable[[], SystemTimes] = read_windows_system_times,
        read_free_memory: Callable[[], int] = read_windows_free_memory,
        sleep: Callable[[float], None] = time.sleep,
    ):
        self.interval_seconds = interval_seconds
        self.read_system_times = read_system_times
        self.read_free_memory = read_free_memory
        self.sleep = sleep

    def sample(self) -> ResourceSample:
        before = self.read_system_times()
        self.sleep(self.interval_seconds)
        after = self.read_system_times()
        total_delta = (after.kernel + after.user) - (before.kernel + before.user)
        idle_delta = after.idle - before.idle
        if total_delta <= 0 or idle_delta < 0 or idle_delta > total_delta:
            cpu_percent = 100.0
        else:
            cpu_percent = max(0.0, min(100.0, 100.0 * (1.0 - idle_delta / total_delta)))
        return ResourceSample(
            cpu_percent=cpu_percent,
            free_memory_bytes=max(0, int(self.read_free_memory())),
        )


def burst_decision(
    samples: Sequence[ResourceSample], *, free_bytes: int, burst_active: bool
) -> BurstDecision:
    """Keep burst admission explicit, bounded, and independent from Session admission."""

    if burst_active:
        return BurstDecision(False, "burst_active")
    if free_bytes < BURST_MIN_FREE_BYTES:
        return BurstDecision(False, "disk_headroom")
    if len(samples) != BURST_SAMPLE_COUNT or any(
        not math.isfinite(sample.cpu_percent) or sample.cpu_percent > BURST_MAX_CPU_PERCENT
        for sample in samples
    ):
        return BurstDecision(False, "cpu_headroom")
    if any(sample.free_memory_bytes < BURST_MIN_FREE_MEMORY_BYTES for sample in samples):
        return BurstDecision(False, "memory_headroom")
    return BurstDecision(True, "allowed")

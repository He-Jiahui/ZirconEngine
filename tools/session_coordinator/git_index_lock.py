from __future__ import annotations

import stat
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

from .processes import file_owner_process_ids


@dataclass(frozen=True, slots=True)
class _LockIdentity:
    device: int
    inode: int
    size: int
    modified_ns: int
    changed_ns: int


@dataclass(frozen=True, slots=True)
class IndexLockRecovery:
    device: int
    inode: int
    size: int
    modified_ns: int
    age_seconds: float

    def to_event_payload(self) -> dict[str, int | float]:
        return {
            "device": self.device,
            "inode": self.inode,
            "size": self.size,
            "modified_ns": self.modified_ns,
            "age_seconds": self.age_seconds,
        }


class IndexLockRecoveryRefused(RuntimeError):
    def __init__(
        self,
        reason: str,
        *,
        active_pids: tuple[int, ...] = (),
        details: dict[str, int | float | str] | None = None,
    ) -> None:
        super().__init__(reason)
        self.reason = reason
        self.active_pids = active_pids
        self.details = details or {}


def recover_stale_index_lock(
    lock_path: Path,
    *,
    minimum_age_seconds: float = 30.0,
    observation_seconds: float = 0.05,
    now_ns: Callable[[], int] = time.time_ns,
    sleep: Callable[[float], None] = time.sleep,
    lock_owner_process_ids: Callable[[], tuple[int, ...]] | None = None,
) -> IndexLockRecovery | None:
    """Remove one abandoned index lock only after two fail-closed observations."""
    process_ids = lock_owner_process_ids or (lambda: file_owner_process_ids(lock_path))
    first = _identity(lock_path)
    if first is None:
        return None
    age_seconds = max(0.0, (now_ns() - first.modified_ns) / 1_000_000_000)
    _require_recoverable(first, age_seconds, minimum_age_seconds)
    first_active = _active_process_ids(process_ids)
    if first_active:
        raise IndexLockRecoveryRefused(
            "active_lock_owner", active_pids=first_active
        )
    sleep(observation_seconds)
    second = _identity(lock_path)
    if second is None:
        return None
    if second != first:
        raise IndexLockRecoveryRefused("identity_changed")
    second_active = _active_process_ids(process_ids)
    if second_active:
        raise IndexLockRecoveryRefused(
            "active_lock_owner", active_pids=second_active
        )
    try:
        lock_path.unlink()
    except FileNotFoundError:
        return None
    return IndexLockRecovery(
        device=first.device,
        inode=first.inode,
        size=first.size,
        modified_ns=first.modified_ns,
        age_seconds=age_seconds,
    )


def _identity(lock_path: Path) -> _LockIdentity | None:
    try:
        observed = lock_path.lstat()
    except FileNotFoundError:
        return None
    except OSError as error:
        raise IndexLockRecoveryRefused(
            "inspection_failed", details={"error": str(error)}
        ) from error
    if not stat.S_ISREG(observed.st_mode):
        raise IndexLockRecoveryRefused("not_regular_file")
    return _LockIdentity(
        device=int(observed.st_dev),
        inode=int(observed.st_ino),
        size=int(observed.st_size),
        modified_ns=int(observed.st_mtime_ns),
        changed_ns=int(observed.st_ctime_ns),
    )


def _require_recoverable(
    identity: _LockIdentity,
    age_seconds: float,
    minimum_age_seconds: float,
) -> None:
    if identity.size != 0:
        raise IndexLockRecoveryRefused(
            "nonzero", details={"size": identity.size}
        )
    if age_seconds < minimum_age_seconds:
        raise IndexLockRecoveryRefused(
            "too_young",
            details={
                "age_seconds": age_seconds,
                "minimum_age_seconds": minimum_age_seconds,
            },
        )


def _active_process_ids(
    process_ids: Callable[[], tuple[int, ...]],
) -> tuple[int, ...]:
    try:
        return tuple(sorted({int(pid) for pid in process_ids() if int(pid) > 0}))
    except OSError as error:
        raise IndexLockRecoveryRefused(
            "process_inspection_failed", details={"error": str(error)}
        ) from error

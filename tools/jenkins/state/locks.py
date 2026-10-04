"""Process-scoped locks; the operating system releases ownership on exit."""
from contextlib import contextmanager
import os
from pathlib import Path
import time

from tools.jenkins.contracts import JenkinsError


@contextmanager
def process_lock(path: str | Path, *, reason: str = "operation_busy", timeout: float = 0):
    path = Path(path).absolute()
    for parent in (path, *path.parents):
        if parent.exists() and (parent.is_symlink() or getattr(parent.lstat(), "st_file_attributes", 0) & 0x400):
            raise JenkinsError("lock_alias_rejected", "A process lock requires a physical path")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a+b") as handle:
        if path.stat().st_size == 0:
            handle.write(b"\0")
            handle.flush()
        deadline = time.monotonic() + timeout
        locked = False
        try:
            while not locked:
                try:
                    handle.seek(0)
                    if os.name == "nt":
                        import msvcrt
                        msvcrt.locking(handle.fileno(), msvcrt.LK_NBLCK, 1)
                    else:
                        import fcntl
                        fcntl.flock(handle.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                    locked = True
                except OSError as exc:
                    if time.monotonic() >= deadline:
                        raise JenkinsError(reason, "Another process owns this operation", retryable=True) from exc
                    time.sleep(min(.05, max(0, deadline - time.monotonic())))
            yield handle
        finally:
            if locked:
                handle.seek(0)
                if os.name == "nt":
                    import msvcrt
                    msvcrt.locking(handle.fileno(), msvcrt.LK_UNLCK, 1)
                else:
                    import fcntl
                    fcntl.flock(handle.fileno(), fcntl.LOCK_UN)

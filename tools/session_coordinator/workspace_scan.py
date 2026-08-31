from __future__ import annotations

import threading
from typing import Protocol


class WorkspaceScanWatcher(Protocol):
    def prepare_scan(self) -> object: ...


class WorkspaceScanSingleFlight:
    """Prepare at most one workspace observation off-loop; apply stays serialized."""

    _CLOSE_JOIN_TIMEOUT_SECONDS = 0.1

    def __init__(self, watcher: WorkspaceScanWatcher) -> None:
        self._watcher = watcher
        self._request = threading.Event()
        self._completed = threading.Event()
        self._shutdown = threading.Event()
        self._lock = threading.Lock()
        self._in_flight = False
        self._outcome: tuple[object | None, Exception | None] | None = None
        self._thread = threading.Thread(
            target=self._run,
            name="zircon-workspace-scan",
            daemon=True,
        )
        self._thread.start()

    def request(self) -> bool:
        with self._lock:
            if self._in_flight or self._shutdown.is_set():
                return False
            self._in_flight = True
        self._request.set()
        return True

    def poll(self) -> tuple[object | None, Exception | None] | None:
        if not self._completed.is_set():
            return None
        with self._lock:
            outcome = self._outcome
            self._outcome = None
            self._in_flight = False
            self._completed.clear()
        return outcome

    def close(self) -> None:
        self._shutdown.set()
        self._request.set()
        self._thread.join(timeout=self._CLOSE_JOIN_TIMEOUT_SECONDS)

    def _run(self) -> None:
        while True:
            self._request.wait()
            self._request.clear()
            if self._shutdown.is_set():
                return
            try:
                outcome = (self._watcher.prepare_scan(), None)
            except Exception as error:  # pragma: no cover - surfaced by maintenance
                outcome = (None, error)
            with self._lock:
                self._outcome = outcome
            self._completed.set()

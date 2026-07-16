from __future__ import annotations

import threading
import time
from collections.abc import Callable

from .models import CodexDiscoveryResult, CodexReconcileResult, CodexSyncTrigger


class CodexSyncWorker:
    """One repository-scoped reconciliation thread with coalesced wakeups."""

    def __init__(
        self,
        *,
        discover: Callable[[bool], CodexDiscoveryResult],
        store,
        spool,
        writable: Callable[[], bool],
        project: Callable[[CodexReconcileResult, bool], object] | None = None,
        membership_interval_seconds: float = 30.0,
        full_interval_seconds: float = 15 * 60.0,
        monotonic: Callable[[], float] = time.monotonic,
    ):
        self._discover = discover
        self._store = store
        self._spool = spool
        self._writable = writable
        self._project = project
        self._membership_interval = max(0.01, membership_interval_seconds)
        self._full_interval = max(self._membership_interval, full_interval_seconds)
        self._monotonic = monotonic
        self._wake = threading.Event()
        self._stop = threading.Event()
        self._state_lock = threading.Lock()
        self._requested_trigger = CodexSyncTrigger.STARTUP
        self._thread: threading.Thread | None = None
        self._successful_runs = 0
        self._failed_runs = 0
        self._suppressed_runs = 0
        self._last_error_code: str | None = None
        self._last_run_id: str | None = None
        self._running = False

    def start(self) -> None:
        with self._state_lock:
            if self._thread is not None and self._thread.is_alive():
                return
            self._stop.clear()
            self._wake.set()
            self._thread = threading.Thread(
                target=self._run,
                name="zircon-codex-session-sync",
                daemon=True,
            )
            self._thread.start()

    def wake(self, reason: str = "hook") -> bool:
        trigger = {
            "startup": CodexSyncTrigger.STARTUP,
            "periodic": CodexSyncTrigger.PERIODIC,
            "hook": CodexSyncTrigger.HOOK,
            "controlled": CodexSyncTrigger.CONTROLLED,
        }.get(reason, CodexSyncTrigger.HOOK)
        with self._state_lock:
            self._requested_trigger = trigger
        self._wake.set()
        return True

    def stop(self, timeout: float = 5.0) -> None:
        self._stop.set()
        self._wake.set()
        thread = self._thread
        if thread is not None:
            thread.join(timeout=timeout)

    def is_alive(self) -> bool:
        return self._thread is not None and self._thread.is_alive()

    def snapshot(self) -> dict[str, object]:
        with self._state_lock:
            return {
                "state": "running" if self._running else ("healthy" if self.is_alive() else "stopped"),
                "successfulRuns": self._successful_runs,
                "failedRuns": self._failed_runs,
                "suppressedRuns": self._suppressed_runs,
                "lastErrorCode": self._last_error_code,
                "lastRunId": self._last_run_id,
                "pendingWake": self._wake.is_set(),
            }

    def _run(self) -> None:
        started_at = self._monotonic()
        next_membership = started_at + self._membership_interval
        next_full = started_at + self._full_interval
        while not self._stop.is_set():
            now = self._monotonic()
            timeout = max(0.0, min(next_membership, next_full) - now)
            self._wake.wait(timeout)
            if self._stop.is_set():
                break
            now = self._monotonic()
            membership_due = now >= next_membership
            full_due = now >= next_full
            periodic_due = membership_due or full_due
            if not self._wake.is_set() and not periodic_due:
                continue
            self._wake.clear()
            with self._state_lock:
                trigger = (
                    CodexSyncTrigger.PERIODIC
                    if periodic_due
                    else self._requested_trigger
                )
                self._running = True
            try:
                if not self._writable():
                    with self._state_lock:
                        self._suppressed_runs += 1
                    continue
                started = self._monotonic()
                items = self._spool.validated_pending()
                include_history = full_due or trigger is CodexSyncTrigger.STARTUP
                discovery = self._discover(include_history)
                result = self._store.reconcile(
                    discovery,
                    trigger=trigger,
                    duration_ms=max(0, int((self._monotonic() - started) * 1000)),
                )
                self._spool.acknowledge_committed(items, run_id=result.run_id)
                if self._project is not None:
                    self._project(result, include_history)
                with self._state_lock:
                    self._successful_runs += 1
                    self._last_run_id = result.run_id
                    self._last_error_code = None
            except Exception:
                with self._state_lock:
                    self._failed_runs += 1
                    self._last_error_code = "codex_sync_failed"
            finally:
                with self._state_lock:
                    self._running = False
                now = self._monotonic()
                if periodic_due or now >= next_membership:
                    next_membership = now + self._membership_interval
                if now >= next_full:
                    next_full = now + self._full_interval

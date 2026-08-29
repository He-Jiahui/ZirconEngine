from __future__ import annotations

import threading
import time
import unittest

from tools.session_coordinator.codex_sync.models import (
    CodexDiscoveryResult,
    CodexReconcileResult,
)
from tools.session_coordinator.codex_sync.worker import CodexSyncWorker


EMPTY_DISCOVERY = CodexDiscoveryResult((), (), True, 0, "empty-revision")


class _Spool:
    def __init__(self) -> None:
        self.items = (object(),)
        self.acknowledged: list[tuple[tuple[object, ...], str]] = []
        self.overflow = {"markerStatus": "absent"}

    def validated_pending(self):
        return self.items

    def acknowledge_committed(self, items, *, run_id: str) -> None:
        self.acknowledged.append((items, run_id))
        self.items = ()

    def overflow_status(self):
        return self.overflow


class _Store:
    def __init__(self) -> None:
        self.calls = 0
        self.active = 0
        self.max_active = 0
        self.entered = threading.Event()
        self.release = threading.Event()
        self.block_first = False
        self.fail_first = False

    def reconcile(self, _discovery, *, trigger, duration_ms=0):
        self.calls += 1
        self.active += 1
        self.max_active = max(self.max_active, self.active)
        try:
            if self.calls == 1 and self.block_first:
                self.entered.set()
                self.release.wait(2)
            if self.calls == 1 and self.fail_first:
                raise RuntimeError("private failure detail must not surface")
            return CodexReconcileResult(f"run-{self.calls}", 0, 0, 0, 0)
        finally:
            self.active -= 1


class CodexSyncWorkerTests(unittest.TestCase):
    def _worker(self, store, spool, **overrides) -> CodexSyncWorker:
        return CodexSyncWorker(
            discover=lambda _full: EMPTY_DISCOVERY,
            store=store,
            spool=spool,
            writable=overrides.pop("writable", lambda: True),
            membership_interval_seconds=overrides.pop("membership", 60),
            full_interval_seconds=overrides.pop("full", 120),
            **overrides,
        )

    @staticmethod
    def _wait_for(predicate, timeout: float = 2.0) -> None:
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if predicate():
                return
            time.sleep(0.01)
        raise AssertionError("worker condition was not reached")

    def test_startup_commit_acknowledges_pending_trigger(self) -> None:
        store = _Store()
        spool = _Spool()
        pending = spool.items
        worker = self._worker(store, spool)

        worker.start()
        self.addCleanup(worker.stop)
        self._wait_for(lambda: store.calls == 1)

        self.assertEqual([(pending, "run-1")], spool.acknowledged)
        self.assertEqual(1, worker.snapshot()["successfulRuns"])

    def test_snapshot_projects_durable_spool_overflow_status(self) -> None:
        store = _Store()
        spool = _Spool()
        spool.overflow = {
            "markerStatus": "valid",
            "firstDetectedAt": "2026-08-29T00:00:00+00:00",
            "lastDetectedAt": "2026-08-29T00:00:00+00:00",
            "maxPending": 3,
            "pendingCount": 3,
        }
        worker = self._worker(store, spool)

        self.assertEqual(spool.overflow, worker.snapshot()["spoolOverflow"])

    def test_wakes_during_run_coalesce_to_one_follow_up(self) -> None:
        store = _Store()
        store.block_first = True
        spool = _Spool()
        worker = self._worker(store, spool)
        worker.start()
        self.addCleanup(worker.stop)
        self.assertTrue(store.entered.wait(1))

        for _ in range(20):
            worker.wake("hook")
        store.release.set()
        self._wait_for(lambda: store.calls >= 2)
        time.sleep(0.1)

        self.assertEqual(2, store.calls)
        self.assertEqual(1, store.max_active)

    def test_periodic_membership_and_full_deadlines_reconcile(self) -> None:
        store = _Store()
        spool = _Spool()
        worker = self._worker(store, spool, membership=0.04, full=0.09)
        worker.start()
        self.addCleanup(worker.stop)

        self._wait_for(lambda: store.calls >= 4)

        self.assertEqual(1, store.max_active)

    def test_read_only_suppresses_reconcile_without_acknowledging(self) -> None:
        store = _Store()
        spool = _Spool()
        worker = self._worker(store, spool, writable=lambda: False, membership=0.03)
        worker.start()
        self.addCleanup(worker.stop)
        time.sleep(0.12)

        self.assertEqual(0, store.calls)
        self.assertEqual([], spool.acknowledged)
        self.assertGreater(worker.snapshot()["suppressedRuns"], 0)

    def test_failure_is_sanitized_and_next_wake_recovers(self) -> None:
        store = _Store()
        store.fail_first = True
        spool = _Spool()
        worker = self._worker(store, spool)
        worker.start()
        self.addCleanup(worker.stop)
        self._wait_for(lambda: worker.snapshot()["failedRuns"] == 1)

        failed_snapshot = worker.snapshot()
        self.assertEqual("codex_sync_failed", failed_snapshot["lastErrorCode"])
        self.assertNotIn("private failure", repr(failed_snapshot))

        worker.wake("controlled")
        self._wait_for(lambda: store.calls == 2)

        snapshot = worker.snapshot()
        self.assertEqual(1, snapshot["failedRuns"])
        self.assertIsNone(snapshot["lastErrorCode"])

    def test_stop_joins_worker_after_inflight_run(self) -> None:
        store = _Store()
        store.block_first = True
        spool = _Spool()
        worker = self._worker(store, spool)
        worker.start()
        self.assertTrue(store.entered.wait(1))

        stopper = threading.Thread(target=worker.stop)
        stopper.start()
        time.sleep(0.05)
        self.assertTrue(stopper.is_alive())
        store.release.set()
        stopper.join(2)

        self.assertFalse(stopper.is_alive())
        self.assertFalse(worker.is_alive())


if __name__ == "__main__":
    unittest.main()

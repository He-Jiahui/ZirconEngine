from __future__ import annotations

import json
import os
import tempfile
import threading
import unittest
from pathlib import Path

from tools.session_coordinator.client import CoordinatorClientError
from tools.session_coordinator.offline_queue import OfflineCommandSpool


class OfflineCommandSpoolTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.spool = OfflineCommandSpool(self.root, repository_key="a" * 64)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def test_enqueue_persists_an_exact_repository_bound_command(self) -> None:
        queued = self.spool.enqueue(
            command="session.heartbeat",
            arguments={"session_id": "session-a"},
        )

        self.assertEqual("session.heartbeat", queued.command)
        self.assertEqual("a" * 64, queued.repository_key)
        self.assertEqual((queued,), self.spool.validated_pending())
        payload = json.loads(queued.path.read_text(encoding="utf-8"))
        self.assertEqual(
            {
                "arguments",
                "command",
                "createdAt",
                "queueId",
                "repositoryKey",
                "schemaVersion",
            },
            set(payload),
        )

    def test_replay_preserves_fifo_order_and_deletes_only_acknowledged_items(self) -> None:
        first = self.spool.enqueue("session.heartbeat", {"session_id": "first"})
        second = self.spool.enqueue("session.heartbeat", {"session_id": "second"})
        delivered: list[str] = []

        result = self.spool.replay(
            lambda command, arguments: delivered.append(
                f"{command}:{arguments['session_id']}"
            )
            or {"ok": True}
        )

        self.assertEqual(["session.heartbeat:first", "session.heartbeat:second"], delivered)
        self.assertEqual(2, result.acknowledged)
        self.assertFalse(first.path.exists())
        self.assertFalse(second.path.exists())
        self.assertEqual(0, self.spool.snapshot().pending)

    def test_transport_loss_retains_head_and_stops_replay_without_reordering(self) -> None:
        first = self.spool.enqueue("session.heartbeat", {"session_id": "first"})
        second = self.spool.enqueue("session.heartbeat", {"session_id": "second"})

        result = self.spool.replay(
            lambda _command, _arguments: (_ for _ in ()).throw(
                CoordinatorClientError("offline", "service offline")
            )
        )

        self.assertEqual(0, result.acknowledged)
        self.assertEqual(2, result.retained)
        self.assertTrue(first.path.exists())
        self.assertTrue(second.path.exists())
        self.assertEqual((first, second), self.spool.validated_pending())

    def test_terminal_rejection_moves_the_head_to_failed_and_preserves_later_fifo_work(self) -> None:
        failed = self.spool.enqueue("session.register", {"session_id": "failed"})
        completed = self.spool.enqueue("session.heartbeat", {"session_id": "completed"})

        result = self.spool.replay(
            lambda _command, arguments: (
                (_ for _ in ()).throw(CoordinatorClientError("session_not_found", "missing"))
                if arguments["session_id"] == "failed"
                else {"ok": True}
            )
        )

        self.assertEqual(1, result.failed)
        self.assertEqual(0, result.acknowledged)
        self.assertEqual(1, result.retained)
        self.assertFalse(failed.path.exists())
        self.assertTrue(completed.path.exists())
        self.assertEqual(1, self.spool.snapshot().failed)
        self.assertEqual((completed,), self.spool.validated_pending())

    def test_foreign_or_malformed_items_are_quarantined_without_touching_valid_work(self) -> None:
        valid = self.spool.enqueue("session.heartbeat", {"session_id": "valid"})
        self.spool.pending_root.mkdir(parents=True, exist_ok=True)
        foreign = self.spool.pending_root / "00000000000000000000-foreign.json"
        foreign.write_text(
            json.dumps(
                {
                    "schemaVersion": 1,
                    "queueId": "foreign",
                    "repositoryKey": "b" * 64,
                    "command": "session.heartbeat",
                    "arguments": {"session_id": "foreign"},
                    "createdAt": "2026-07-16T00:00:00+00:00",
                }
            ),
            encoding="utf-8",
        )

        self.assertEqual((valid,), self.spool.validated_pending())
        self.assertTrue(valid.path.exists())
        self.assertFalse(foreign.exists())
        self.assertEqual(1, self.spool.snapshot().quarantined)

    def test_queue_limit_rejects_new_work_without_discarding_pending_items(self) -> None:
        spool = OfflineCommandSpool(self.root, repository_key="a" * 64, max_pending=1)
        first = spool.enqueue("session.heartbeat", {"session_id": "first"})

        with self.assertRaises(ValueError):
            spool.enqueue("session.heartbeat", {"session_id": "second"})

        self.assertEqual((first,), spool.validated_pending())

    def test_spool_rejects_unsafe_commands_even_when_called_without_the_cli(self) -> None:
        with self.assertRaises(ValueError):
            self.spool.enqueue("cargo.acquire", {"session_id": "session-a"})
        with self.assertRaises(ValueError):
            self.spool.enqueue("service.restart", {})
        self.assertEqual(0, self.spool.snapshot().pending)

    def test_repository_bound_unsafe_envelope_is_quarantined_before_replay(self) -> None:
        self.spool.pending_root.mkdir(parents=True, exist_ok=True)
        unsafe = self.spool.pending_root / "00000000000000000000-unsafe.json"
        unsafe.write_text(
            json.dumps(
                {
                    "schemaVersion": 1,
                    "queueId": "0" * 32,
                    "repositoryKey": "a" * 64,
                    "command": "finalize.commit",
                    "arguments": {"session_id": "session-a"},
                    "createdAt": "2026-07-16T00:00:00+00:00",
                }
            ),
            encoding="utf-8",
        )

        result = self.spool.replay(lambda _command, _arguments: self.fail("unsafe replay"))

        self.assertEqual(1, result.quarantined)
        self.assertEqual(0, result.acknowledged)
        self.assertEqual(1, self.spool.snapshot().quarantined)

    def test_second_replayer_returns_without_waiting_or_executing_the_pending_head(self) -> None:
        queued = self.spool.enqueue("session.heartbeat", {"session_id": "session-a"})
        self.spool.root.mkdir(parents=True, exist_ok=True)
        self.spool.replay_lock_path.write_text(
            json.dumps(self.spool._lock_descriptor()),
            encoding="utf-8",
        )

        result = self.spool.replay(
            lambda _command, _arguments: self.fail("concurrent replayer executed work")
        )

        self.assertEqual(1, result.retained)
        self.assertTrue(queued.path.exists())

    def test_dead_replay_lock_is_recovered_before_replaying(self) -> None:
        self.spool.enqueue("session.heartbeat", {"session_id": "session-a"})
        self.spool.root.mkdir(parents=True, exist_ok=True)
        self.spool.replay_lock_path.write_text(
            json.dumps({"pid": 2_147_483_647, "createdAt": "2026-07-16T00:00:00+00:00"}),
            encoding="utf-8",
        )
        delivered: list[str] = []

        result = self.spool.replay(
            lambda command, _arguments: delivered.append(command) or {"ok": True}
        )

        self.assertEqual(["session.heartbeat"], delivered)
        self.assertEqual(1, result.acknowledged)
        self.assertFalse(self.spool.replay_lock_path.exists())

    def test_concurrent_enqueues_never_exceed_the_configured_capacity(self) -> None:
        spool = OfflineCommandSpool(self.root, repository_key="a" * 64, max_pending=1)
        barrier = threading.Barrier(2)
        outcomes: list[str] = []

        def enqueue(session_id: str) -> None:
            barrier.wait()
            try:
                spool.enqueue("session.heartbeat", {"session_id": session_id})
            except ValueError:
                outcomes.append("rejected")
            else:
                outcomes.append("queued")

        workers = [threading.Thread(target=enqueue, args=(f"session-{index}",)) for index in range(2)]
        for worker in workers:
            worker.start()
        for worker in workers:
            worker.join()

        self.assertEqual(["queued", "rejected"], sorted(outcomes))
        self.assertEqual(1, spool.snapshot().pending)

    def test_corrupt_replay_lock_is_recovered_without_stranding_pending_work(self) -> None:
        self.spool.enqueue("session.heartbeat", {"session_id": "session-a"})
        self.spool.root.mkdir(parents=True, exist_ok=True)
        self.spool.replay_lock_path.write_text("not-json", encoding="utf-8")
        delivered: list[str] = []

        result = self.spool.replay(
            lambda command, _arguments: delivered.append(command) or {"ok": True}
        )

        self.assertEqual(["session.heartbeat"], delivered)
        self.assertEqual(1, result.acknowledged)


if __name__ == "__main__":
    unittest.main()

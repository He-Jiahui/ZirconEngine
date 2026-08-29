from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.codex_sync.spool import (
    CodexHookEvent,
    CodexTrigger,
    CodexTriggerSpool,
)


class CodexTriggerSpoolTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.spool = CodexTriggerSpool(
            self.root / "spool", "a" * 64, max_pending=3
        )

    def _trigger(self, session_id: str) -> CodexTrigger:
        return CodexTrigger(
            event=CodexHookEvent.STOP,
            session_id=session_id,
            cwd=str(self.root),
            created_at="2026-07-13T00:00:00+00:00",
            turn_id="turn-one",
            model="gpt-5-codex",
            permission_mode="default",
        )

    def test_round_trip_is_bounded_and_contains_only_schema_fields(self) -> None:
        path = self.spool.enqueue(self._trigger("session-one"))

        raw = path.read_text(encoding="utf-8")
        items = self.spool.validated_pending()

        self.assertLessEqual(len(raw.encode("utf-8")), 4096)
        self.assertEqual(1, len(items))
        self.assertEqual("session-one", items[0].trigger.session_id)
        self.assertEqual(
            {
                "agentId",
                "agentType",
                "createdAt",
                "cwd",
                "eventName",
                "model",
                "permissionMode",
                "repositoryKey",
                "schemaVersion",
                "sessionId",
                "source",
                "turnId",
            },
            set(json.loads(raw)),
        )

    def test_queue_cap_rejects_new_trigger_and_preserves_pending_work(self) -> None:
        for index in range(3):
            self.spool.enqueue(self._trigger(f"session-{index}"))

        with self.assertRaises(OverflowError):
            self.spool.enqueue(self._trigger("session-rejected"))

        items = self.spool.validated_pending()
        overflow = self.spool.overflow_status()

        self.assertEqual(3, len(items))
        self.assertEqual(
            {"session-0", "session-1", "session-2"},
            {item.trigger.session_id for item in items},
        )
        self.assertEqual("valid", overflow["markerStatus"])
        self.assertEqual(3, overflow["maxPending"])
        self.assertEqual(3, overflow["pendingCount"])
        self.assertNotIn("session-rejected", repr(overflow))

        reopened = CodexTriggerSpool(self.root / "spool", "a" * 64, max_pending=3)
        self.assertEqual(overflow, reopened.overflow_status())

    def test_corrupt_item_is_quarantined_without_blocking_valid_items(self) -> None:
        self.spool.enqueue(self._trigger("valid-session"))
        self.spool.pending_root.mkdir(parents=True, exist_ok=True)
        (self.spool.pending_root / "corrupt.json").write_text(
            '{"prompt":"must-not-surface"', encoding="utf-8"
        )

        items = self.spool.validated_pending()

        self.assertEqual(("valid-session",), tuple(item.trigger.session_id for item in items))
        self.assertEqual(1, len(tuple(self.spool.quarantine_root.glob("*.json"))))

    def test_corrupt_overflow_marker_is_projected_fail_closed(self) -> None:
        marker = self.spool.repository_root / "overflow.json"
        marker.parent.mkdir(parents=True, exist_ok=True)
        marker.write_text('{"sessionId":"must-not-surface"}', encoding="utf-8")

        overflow = self.spool.overflow_status()

        self.assertEqual({"markerStatus": "invalid"}, overflow)
        self.assertNotIn("must-not-surface", repr(overflow))

    def test_overflow_replaces_corrupt_marker_before_rejecting_new_trigger(self) -> None:
        self.spool.overflow_path.parent.mkdir(parents=True, exist_ok=True)
        self.spool.overflow_path.write_text(
            '{"sessionId":"must-not-surface"}', encoding="utf-8"
        )
        for index in range(3):
            self.spool.enqueue(self._trigger(f"session-{index}"))

        with self.assertRaises(OverflowError):
            self.spool.enqueue(self._trigger("session-rejected"))

        self.assertEqual("valid", self.spool.overflow_status()["markerStatus"])
        self.assertEqual(
            {"session-0", "session-1", "session-2"},
            {item.trigger.session_id for item in self.spool.validated_pending()},
        )

    def test_overflow_marker_write_failure_does_not_accept_new_trigger(self) -> None:
        for index in range(3):
            self.spool.enqueue(self._trigger(f"session-{index}"))

        original = self.spool._record_overflow
        self.spool._record_overflow = lambda _count: (_ for _ in ()).throw(
            OSError("marker storage unavailable")
        )
        self.addCleanup(lambda: setattr(self.spool, "_record_overflow", original))

        with self.assertRaises(OSError):
            self.spool.enqueue(self._trigger("session-rejected"))

        self.assertEqual(3, self.spool.pending_count())
        self.assertEqual(
            {"session-0", "session-1", "session-2"},
            {item.trigger.session_id for item in self.spool.validated_pending()},
        )

    def test_acknowledgement_requires_committed_run_and_owned_item(self) -> None:
        item = self.spool.validated_pending()[0] if self.spool.pending_count() else None
        if item is None:
            self.spool.enqueue(self._trigger("ack-session"))
            item = self.spool.validated_pending()[0]

        with self.assertRaises(ValueError):
            self.spool.acknowledge_committed((item,), run_id="")
        outside = self.root / "outside.json"
        outside.write_text("{}", encoding="utf-8")
        forged = item.__class__(outside, item.trigger)
        with self.assertRaises(ValueError):
            self.spool.acknowledge_committed((forged,), run_id="run-one")

        self.spool.acknowledge_committed((item,), run_id="run-one")

        self.assertFalse(item.path.exists())


if __name__ == "__main__":
    unittest.main()

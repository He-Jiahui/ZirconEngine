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

    def test_queue_cap_removes_oldest_pending_trigger(self) -> None:
        for index in range(5):
            self.spool.enqueue(self._trigger(f"session-{index}"))

        items = self.spool.validated_pending()

        self.assertEqual(3, len(items))
        self.assertEqual(
            {"session-2", "session-3", "session-4"},
            {item.trigger.session_id for item in items},
        )

    def test_corrupt_item_is_quarantined_without_blocking_valid_items(self) -> None:
        self.spool.enqueue(self._trigger("valid-session"))
        self.spool.pending_root.mkdir(parents=True, exist_ok=True)
        (self.spool.pending_root / "corrupt.json").write_text(
            '{"prompt":"must-not-surface"', encoding="utf-8"
        )

        items = self.spool.validated_pending()

        self.assertEqual(("valid-session",), tuple(item.trigger.session_id for item in items))
        self.assertEqual(1, len(tuple(self.spool.quarantine_root.glob("*.json"))))

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

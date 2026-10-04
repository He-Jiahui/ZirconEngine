from __future__ import annotations

import tempfile
import threading
import unittest
from pathlib import Path

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.notifications import NotificationOutbox
from tools.jenkins.state import State


class NotificationOutboxTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.state = State(Path(self.temp.name) / "state.sqlite3")

    def tearDown(self) -> None:
        self.temp.cleanup()

    def test_event_and_destination_are_idempotent(self) -> None:
        outbox = NotificationOutbox(self.state)
        first = outbox.enqueue("event-1", "tray", {"text": "done"})
        duplicate = outbox.enqueue("event-1", "tray", {"text": "done"})
        self.assertEqual(first, duplicate)
        with self.assertRaisesRegex(JenkinsError, "payload"):
            outbox.enqueue("event-1", "tray", {"text": "changed"})
        other = outbox.enqueue("event-1", "wecom", {"text": "done"})
        self.assertNotEqual(first.destination_id, other.destination_id)

    def test_success_is_sent_once(self) -> None:
        calls: list[dict] = []
        outbox = NotificationOutbox(self.state, lambda payload: calls.append(payload) or True)
        outbox.enqueue("event-2", "tray", {"text": "done"})
        first = outbox.deliver("event-2", "tray")
        second = outbox.deliver("event-2", "tray")
        self.assertEqual("delivered", first.status)
        self.assertEqual(first, second)
        self.assertEqual(1, len(calls))

    def test_ambiguous_error_requires_reconcile_before_retry(self) -> None:
        calls: list[dict] = []

        def sender(payload):
            calls.append(payload)
            raise TimeoutError("connection lost after provider accepted")

        outbox = NotificationOutbox(self.state, sender)
        outbox.enqueue("event-3", "tray", {"text": "done"})
        unknown = outbox.deliver("event-3", "tray")
        self.assertEqual("unknown", unknown.status)
        with self.assertRaisesRegex(JenkinsError, "Reconcile"):
            outbox.deliver("event-3", "tray")
        delivered = outbox.reconcile("event-3", "tray", lambda payload: True)
        self.assertEqual("delivered", delivered.status)
        self.assertEqual(1, len(calls))

    def test_reconcile_can_authorize_an_independent_retry(self) -> None:
        outcomes = iter([RuntimeError("ambiguous"), True])
        calls = 0

        def sender(payload):
            nonlocal calls
            calls += 1
            outcome = next(outcomes)
            if isinstance(outcome, Exception):
                raise outcome
            return outcome

        outbox = NotificationOutbox(self.state, sender)
        outbox.enqueue("event-4", "tray", {"text": "done"})
        self.assertEqual("unknown", outbox.deliver("event-4", "tray").status)
        self.assertEqual("retryable", outbox.reconcile("event-4", "tray", lambda payload: False).status)
        self.assertEqual("delivered", outbox.deliver("event-4", "tray").status)
        self.assertEqual(2, calls)

    def test_git_event_does_not_get_resent_by_notification_retry(self) -> None:
        calls = []
        outbox = NotificationOutbox(self.state, lambda payload: calls.append(payload) or False)
        outbox.enqueue("git-commit-1", "tray", {"commit": "abc"})
        result = outbox.deliver("git-commit-1", "tray")
        self.assertEqual("retryable", result.status)
        self.assertEqual("abc", result.payload["commit"])
        self.assertEqual(1, len(calls))

    def test_concurrent_delivery_has_one_sender(self) -> None:
        entered = threading.Event()
        release = threading.Event()
        calls = []

        def sender(payload):
            calls.append(payload)
            entered.set()
            self.assertTrue(release.wait(timeout=10))
            return True

        outbox = NotificationOutbox(self.state, sender)
        outbox.enqueue("event-concurrent", "tray", {"text": "done"})
        results, errors = [], []

        def run():
            try:
                results.append(outbox.deliver("event-concurrent", "tray"))
            except Exception as error:
                errors.append(error)

        first = threading.Thread(target=run)
        second = threading.Thread(target=run)
        first.start()
        try:
            self.assertTrue(entered.wait(timeout=5))
            second.start()
            second.join(timeout=5)
            self.assertFalse(second.is_alive())
        finally:
            release.set()
            first.join(timeout=10)
            if second.ident is not None:
                second.join(timeout=10)
        self.assertEqual(1, len(calls))
        self.assertEqual(1, len(results))
        self.assertEqual(1, len(errors))
        self.assertIsInstance(errors[0], JenkinsError)
        self.assertEqual("delivered", outbox.deliver("event-concurrent", "tray").status)
        self.assertEqual(1, len(calls))

    def test_recovery_marks_claim_unknown_and_late_finish_cannot_overwrite(self) -> None:
        outbox = NotificationOutbox(self.state, lambda payload: True)
        outbox.enqueue("event-recover", "tray", {"text": "done"})
        record = outbox.get("event-recover", "tray")
        key = outbox._key("event-recover", "tray")
        with self.state.transaction() as connection:
            value = {**record.as_dict(), "status": "delivering", "claimEpoch": "old-claim"}
            self.state.put("notification_outbox", key, value, expected_version=self.state.get("notification_outbox", key, connection=connection)["version"], connection=connection)
        self.assertEqual(("event-recover",), outbox.recover_delivering())
        recovered = outbox.get("event-recover", "tray")
        self.assertEqual("unknown", recovered.status)
        with self.assertRaisesRegex(JenkinsError, "stale"):
            outbox._finish(key, "old-claim", "delivered", None)
        self.assertEqual("unknown", outbox.get("event-recover", "tray").status)

    def test_delivery_exception_is_sanitized(self) -> None:
        secret = "https://example.test/hook?token=topsecret"
        outbox = NotificationOutbox(self.state, lambda payload: (_ for _ in ()).throw(RuntimeError(secret)))
        outbox.enqueue("event-secret", "tray", {"text": "done"})
        result = outbox.deliver("event-secret", "tray")
        self.assertEqual("unknown", result.status)
        self.assertNotIn("topsecret", result.last_error or "")
        self.assertNotIn("https://", result.last_error or "")


if __name__ == "__main__":
    unittest.main()

from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.notifications import WeComNotificationService


class NotificationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.database = Database(Path(self.temporary.name) / "state.sqlite3")
        migrate(self.database)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def test_formats_exactly_four_server_derived_lines(self) -> None:
        message = WeComNotificationService.format_message(
            module="session_coordinator",
            summary="完成 M4 工作流门禁",
            commit_time="2026-07-12T02:00:00+08:00",
            shortstat="5 files changed, 10 insertions(+)",
            commit_content="881600cc feat(workflow): complete M4",
        )

        self.assertEqual(4, len(message.splitlines()))
        self.assertTrue(message.startswith("核心内容摘要：【session_coordinator】"))
        self.assertIn("\n提交时间：", message)
        self.assertIn("\n修改情况统计：", message)
        self.assertIn(
            "\n提交的commit内容：881600cc feat(workflow): complete M4", message
        )
        self.assertNotIn("【session_coordinator】feat(workflow)", message)

    def test_rejects_unsafe_notification_module(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            WeComNotificationService.format_message(
                module="runtime】\n伪造字段：值",
                summary="完成提交",
                commit_time="2026-07-12T02:00:00+08:00",
                shortstat="1 file changed",
                commit_content="abc1234 fix(runtime): repair gate",
            )

        self.assertEqual("notification_module_invalid", rejected.exception.code)

    def test_reserves_before_one_call_and_refuses_retry(self) -> None:
        calls: list[list[str]] = []

        def runner(command: list[str]) -> subprocess.CompletedProcess[str]:
            with self.database.connect() as connection:
                status = connection.execute(
                    "SELECT status FROM notification_attempts"
                ).fetchone()[0]
            self.assertEqual("reserved", status)
            calls.append(command)
            return subprocess.CompletedProcess(command, 0, '{"errcode":0}', "")

        service = WeComNotificationService(
            self.database,
            script_path=Path(self.temporary.name) / "send.ps1",
            runner=runner,
        )
        first = service.notify_once(commit_sha="a" * 40, message="four lines")

        self.assertEqual("succeeded", first.status)
        self.assertEqual("0", first.provider_errcode)
        self.assertEqual(1, len(calls))
        self.assertFalse(any("webhook" in item.casefold() for item in calls[0]))
        duplicate = service.notify_once(commit_sha="a" * 40, message="four lines")
        self.assertEqual(first.notification_attempt_id, duplicate.notification_attempt_id)
        self.assertEqual(1, len(calls))

    def test_failure_is_sanitized_and_never_retried(self) -> None:
        secret_url = "https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=secret"

        def runner(command: list[str]) -> subprocess.CompletedProcess[str]:
            return subprocess.CompletedProcess(
                command, 1, "", f"request failed {secret_url} errcode=93000"
            )

        service = WeComNotificationService(
            self.database, runner=runner, script_path="send.ps1"
        )
        result = service.notify_once(commit_sha="b" * 40, message="four lines")

        self.assertEqual("failed", result.status)
        self.assertEqual("93000", result.provider_errcode)
        self.assertNotIn("secret", result.sanitized_error or "")
        self.assertNotIn("qyapi", result.sanitized_error or "")

    def test_post_commit_preparation_failure_is_recorded_once_without_delivery(self) -> None:
        calls: list[list[str]] = []
        service = WeComNotificationService(
            self.database,
            script_path="send.ps1",
            runner=lambda command: calls.append(command),
        )

        first = service.record_post_commit_failure(
            commit_sha="e" * 40,
            error=CoordinatorError("notification_content_invalid", "format failed"),
        )
        duplicate = service.record_post_commit_failure(
            commit_sha="e" * 40,
            error=CoordinatorError("notification_content_invalid", "format failed"),
        )

        self.assertEqual("unknown", first.status)
        self.assertEqual(first.notification_attempt_id, duplicate.notification_attempt_id)
        self.assertIn("post-commit", first.sanitized_error or "")
        self.assertEqual([], calls)

    def test_startup_marks_abandoned_reservation_unknown_without_retry(self) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO notification_attempts(
                       notification_attempt_id, commit_sha, channel, status,
                       message_hash, attempted_at
                   ) VALUES ('lost', ?, 'wecom', 'reserved', ?, '2026-07-12T00:00:00Z')""",
                ("c" * 40, "d" * 64),
            )
        calls: list[list[str]] = []
        service = WeComNotificationService(
            self.database,
            script_path="send.ps1",
            runner=lambda command: calls.append(command),
        )

        recovered = service.recover_reserved()

        self.assertEqual(("lost",), recovered)
        self.assertEqual([], calls)
        with self.database.connect() as connection:
            status = connection.execute(
                "SELECT status FROM notification_attempts WHERE notification_attempt_id='lost'"
            ).fetchone()[0]
        self.assertEqual("unknown", status)


if __name__ == "__main__":
    unittest.main()

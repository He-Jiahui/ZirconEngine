from __future__ import annotations

import json
import os
import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.codex_sync.discovery import CodexSessionDiscovery
from tools.session_coordinator.codex_sync.models import (
    CodexLifecycleEvent,
    CodexSessionState,
    CodexSourceLocation,
)
from tools.session_coordinator.tests.codex_rollout_fixture import write_rollout


class CodexDiscoveryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.repo = self.root / "ZirconEngine"
        self.repo.mkdir()
        self.codex_home = self.root / "codex"

    def test_discovers_only_safe_metadata_for_zircon_sessions(self) -> None:
        secret = "super-secret-hook-value"
        write_rollout(
            self.codex_home,
            thread_id="thread-active",
            cwd=self.repo,
            lifecycle=("task_started",),
            secret_marker=secret,
        )
        write_rollout(
            self.codex_home,
            thread_id="thread-foreign",
            cwd=self.root / "ZirconEngine-other",
            secret_marker=secret,
        )

        result = CodexSessionDiscovery(self.codex_home, self.repo).discover()

        self.assertEqual(1, len(result.sessions))
        session = result.sessions[0]
        self.assertEqual("thread-active", session.thread_id)
        self.assertEqual(CodexSessionState.ACTIVE, session.state)
        self.assertEqual(CodexLifecycleEvent.TASK_STARTED, session.last_event)
        self.assertNotIn(secret, repr(session))
        self.assertNotIn(secret, repr(result.diagnostics))

    def test_archived_location_overrides_tail_activity(self) -> None:
        write_rollout(
            self.codex_home,
            thread_id="thread-archived",
            cwd=self.repo,
            archived=True,
            lifecycle=("task_started",),
        )

        session = CodexSessionDiscovery(self.codex_home, self.repo).discover().sessions[0]

        self.assertEqual(CodexSourceLocation.ARCHIVED, session.source_location)
        self.assertEqual(CodexSessionState.ARCHIVED, session.state)

    def test_terminal_tail_event_derives_idle_state(self) -> None:
        write_rollout(
            self.codex_home,
            thread_id="thread-idle",
            cwd=self.repo,
            lifecycle=("task_started", "task_complete"),
        )

        session = CodexSessionDiscovery(self.codex_home, self.repo).discover().sessions[0]

        self.assertEqual(CodexSessionState.IDLE, session.state)
        self.assertEqual(CodexLifecycleEvent.TASK_COMPLETED, session.last_event)
        self.assertEqual("turn-2", session.last_turn_id)

    def test_malformed_rollout_is_diagnostic_not_global_failure(self) -> None:
        valid = write_rollout(
            self.codex_home,
            thread_id="thread-valid",
            cwd=self.repo,
        )
        malformed = valid.with_name("rollout-malformed.jsonl")
        malformed.write_text("not-json\n", encoding="utf-8")

        result = CodexSessionDiscovery(self.codex_home, self.repo).discover()

        self.assertEqual(("thread-valid",), tuple(item.thread_id for item in result.sessions))
        self.assertIn("codex_rollout_unreadable", {item.code for item in result.diagnostics})

    def test_file_limit_marks_membership_incomplete(self) -> None:
        for index in range(3):
            write_rollout(
                self.codex_home,
                thread_id=f"thread-{index}",
                cwd=self.repo,
            )

        result = CodexSessionDiscovery(self.codex_home, self.repo, max_files=2).discover()

        self.assertFalse(result.membership_complete)
        self.assertEqual(2, result.scanned_count)
        self.assertIn("codex_rollout_limit_exceeded", {item.code for item in result.diagnostics})

    def test_alternate_drive_and_malformed_cwd_are_rejected(self) -> None:
        write_rollout(
            self.codex_home,
            thread_id="thread-other-drive",
            cwd=Path("Z:/unrelated/ZirconEngine"),
        )
        malformed = write_rollout(
            self.codex_home,
            thread_id="thread-malformed-cwd",
            cwd=self.repo,
        )
        records = malformed.read_text(encoding="utf-8").splitlines()
        first = json.loads(records[0])
        first["payload"]["cwd"] = "invalid\x00cwd"
        records[0] = json.dumps(first, separators=(",", ":"))
        malformed.write_text("\n".join(records) + "\n", encoding="utf-8")

        result = CodexSessionDiscovery(self.codex_home, self.repo).discover()

        self.assertEqual((), result.sessions)
        self.assertIn("codex_session_meta_invalid", {item.code for item in result.diagnostics})

    def test_symlink_cwd_escape_is_rejected_when_supported(self) -> None:
        outside = self.root / "outside"
        outside.mkdir()
        link = self.repo / "linked-outside"
        try:
            link.symlink_to(outside, target_is_directory=True)
        except OSError as error:
            self.skipTest(f"directory symlink is unavailable: {error}")
        write_rollout(
            self.codex_home,
            thread_id="thread-symlink-escape",
            cwd=link,
        )

        result = CodexSessionDiscovery(self.codex_home, self.repo).discover()

        self.assertEqual((), result.sessions)

    def test_partial_final_tail_line_preserves_last_complete_event(self) -> None:
        rollout = write_rollout(
            self.codex_home,
            thread_id="thread-partial-tail",
            cwd=self.repo,
            lifecycle=("task_started",),
        )
        with rollout.open("ab") as handle:
            handle.write(b'{"type":"event_msg","payload":{"type":"task_complete"')

        session = CodexSessionDiscovery(self.codex_home, self.repo).discover().sessions[0]

        self.assertEqual(CodexSessionState.ACTIVE, session.state)
        self.assertEqual(CodexLifecycleEvent.TASK_STARTED, session.last_event)

    def test_untrusted_optional_metadata_is_discarded(self) -> None:
        secret = "metadata-secret"
        write_rollout(
            self.codex_home,
            thread_id="thread-safe-metadata",
            cwd=self.repo,
            originator=f"https://example.invalid/?key={secret}",
            cli_version=f"0.test\n{secret}",
            thread_source=f"source={secret}",
        )

        session = CodexSessionDiscovery(self.codex_home, self.repo).discover().sessions[0]

        self.assertIsNone(session.originator)
        self.assertIsNone(session.cli_version)
        self.assertIsNone(session.thread_source)
        self.assertNotIn(secret, repr(session))

    def test_incremental_scan_reuses_unchanged_parse_and_full_scan_refreshes(self) -> None:
        rollout = write_rollout(
            self.codex_home,
            thread_id="thread-incremental",
            cwd=self.repo,
            lifecycle=("task_started",),
        )
        discovery = CodexSessionDiscovery(self.codex_home, self.repo)
        first = discovery.discover(full=True)
        original = rollout.read_text(encoding="utf-8")
        replacement = original.replace("task_started", "turn_aborted")
        self.assertEqual(len(original), len(replacement))
        stat = rollout.stat()
        rollout.write_text(replacement, encoding="utf-8")
        os.utime(rollout, ns=(stat.st_atime_ns, stat.st_mtime_ns))

        incremental = discovery.discover(full=False)
        refreshed = discovery.discover(full=True)

        self.assertEqual(CodexSessionState.ACTIVE, first.sessions[0].state)
        self.assertEqual(CodexSessionState.ACTIVE, incremental.sessions[0].state)
        self.assertEqual(CodexSessionState.IDLE, refreshed.sessions[0].state)


if __name__ == "__main__":
    unittest.main()

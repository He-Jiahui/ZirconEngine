from __future__ import annotations

import io
import json
import tempfile
import threading
import time
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from unittest.mock import patch

from tools.session_coordinator.codex_sync.hook import (
    MAX_HOOK_STDIN_BYTES,
    run_hook,
    signal_coordinator,
)
from tools.session_coordinator.codex_sync.spool import CodexTriggerSpool
from tools.session_coordinator.processes import current_process_identity
from tools.session_coordinator.supervision.repository_identity import repository_identity
from tools.session_coordinator.supervision.runtime_descriptor import RuntimeDescriptor


class CodexHookTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.repo = self.root / "ZirconEngine"
        self.repo.mkdir()
        self.spool_root = self.root / "spool"
        self.secret = "hook-secret-must-not-persist"

    def _payload(self, event: str, **extra) -> bytes:
        payload = {
            "session_id": "thread-one",
            "transcript_path": str(self.root / f"{self.secret}.jsonl"),
            "cwd": str(self.repo),
            "hook_event_name": event,
            "model": "gpt-5-codex",
            "permission_mode": "default",
            **extra,
        }
        return json.dumps(payload).encode("utf-8")

    def _run(self, event: str, raw: bytes, signaler=None):
        stdout = io.StringIO()
        result = run_hook(
            event,
            io.BytesIO(raw),
            stdout,
            repo_root=self.repo,
            spool_base=self.spool_root,
            signaler=signaler or (lambda *_: False),
            clock=lambda: "2026-07-13T00:00:00+00:00",
        )
        return result, stdout.getvalue()

    def _spool(self) -> CodexTriggerSpool:
        return CodexTriggerSpool(
            self.spool_root, repository_identity(self.repo).key
        )

    def _write_runtime(self, port: int, *, creation_time: str | None = None) -> dict:
        identity = current_process_identity()
        descriptor = RuntimeDescriptor(
            "127.0.0.1",
            port,
            "fixture-runtime-token",
            self.repo,
            repository_identity(self.repo),
            "fixture-instance",
            "2026-07-13T00:00:00+00:00",
            identity,
        ).to_payload()
        if creation_time is not None:
            descriptor["process_creation_time"] = creation_time
        runtime = self.repo / ".codex" / "state" / "session-coordinator" / "runtime.json"
        runtime.parent.mkdir(parents=True)
        runtime.write_text(json.dumps(descriptor), encoding="utf-8")
        (runtime.parent / "coordinator.lock").write_text(
            json.dumps({"pid": descriptor["pid"]}), encoding="utf-8"
        )
        return descriptor

    def test_prompt_and_assistant_content_are_never_spooled(self) -> None:
        self._run(
            "UserPromptSubmit",
            self._payload(
                "UserPromptSubmit",
                turn_id="turn-one",
                prompt=f"prompt {self.secret}",
                tool_input={"token": self.secret},
            ),
        )
        _, stop_output = self._run(
            "Stop",
            self._payload(
                "Stop",
                turn_id="turn-one",
                last_assistant_message=f"assistant {self.secret}",
            ),
        )

        raw = "\n".join(
            item.path.read_text(encoding="utf-8")
            for item in self._spool().validated_pending()
        )
        self.assertNotIn(self.secret, raw)
        self.assertEqual({"continue": True}, json.loads(stop_output))

    def test_all_supported_events_reduce_to_closed_fields_and_stdout_shapes(self) -> None:
        events = {
            "SessionStart": {"source": "resume"},
            "UserPromptSubmit": {"turn_id": "turn-prompt", "prompt": self.secret},
            "Stop": {"turn_id": "turn-stop", "last_assistant_message": self.secret},
            "SubagentStart": {
                "turn_id": "turn-agent",
                "agent_id": "agent-one",
                "agent_type": "reviewer",
            },
            "SubagentStop": {
                "turn_id": "turn-agent",
                "agent_id": "agent-one",
                "agent_type": "reviewer",
                "agent_transcript_path": str(self.root / self.secret),
                "last_assistant_message": self.secret,
            },
        }

        outputs = {
            event: self._run(event, self._payload(event, **fields))[1]
            for event, fields in events.items()
        }

        self.assertEqual({"continue": True}, json.loads(outputs.pop("Stop")))
        self.assertEqual({"SessionStart": "", "UserPromptSubmit": "", "SubagentStart": "", "SubagentStop": ""}, outputs)
        self.assertEqual(set(events), {item.trigger.event.codex_name for item in self._spool().validated_pending()})

    def test_invalid_input_still_returns_valid_stop_output_without_spooling(self) -> None:
        for raw in (b"", b"not-json", b"{}", b"x" * (MAX_HOOK_STDIN_BYTES + 1)):
            result, output = self._run("Stop", raw)
            self.assertEqual(0, result)
            self.assertEqual({"continue": True}, json.loads(output))

        self.assertEqual(0, self._spool().pending_count())

    def test_event_mismatch_and_unsupported_session_source_are_ignored(self) -> None:
        self._run("Stop", self._payload("UserPromptSubmit"))
        self._run("SessionStart", self._payload("SessionStart", source="unknown"))

        self.assertEqual(0, self._spool().pending_count())

    def test_repository_boundary_rejects_foreign_and_sibling_prefix_cwd(self) -> None:
        for cwd in (self.root / "foreign", self.root / "ZirconEngine-other"):
            payload = json.loads(self._payload("Stop", turn_id="turn-one"))
            payload["cwd"] = str(cwd)
            self._run("Stop", json.dumps(payload).encode("utf-8"))

        self.assertEqual(0, self._spool().pending_count())

    def test_trigger_is_durable_before_best_effort_signal(self) -> None:
        observed = []

        def signaler(repo_root: Path, repository_key: str) -> bool:
            observed.append(
                CodexTriggerSpool(self.spool_root, repository_key).pending_count()
            )
            return True

        self._run("Stop", self._payload("Stop", turn_id="turn-one"), signaler)

        self.assertEqual([1], observed)

    def test_authenticated_online_signal_uses_only_repository_identity(self) -> None:
        observed: dict[str, object] = {}

        class Handler(BaseHTTPRequestHandler):
            def do_POST(self) -> None:
                observed["path"] = self.path
                observed["authorization"] = self.headers.get("Authorization")
                length = int(self.headers.get("Content-Length", "0"))
                observed["body"] = json.loads(self.rfile.read(length))
                self.send_response(202)
                self.end_headers()

            def log_message(self, *_args) -> None:
                return

        server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        self.addCleanup(server.server_close)
        self.addCleanup(server.shutdown)
        self._write_runtime(server.server_port)

        signaled = signal_coordinator(self.repo, repository_identity(self.repo).key)

        self.assertTrue(signaled)
        self.assertEqual("/control/v1/codex-sync/wake", observed["path"])
        self.assertEqual("Bearer fixture-runtime-token", observed["authorization"])
        self.assertEqual(
            {"repositoryKey": repository_identity(self.repo).key, "schemaVersion": 1},
            observed["body"],
        )

    def test_stale_process_identity_fails_before_network(self) -> None:
        self._write_runtime(43123, creation_time="stale-creation-time")

        with patch("tools.session_coordinator.codex_sync.hook.urllib.request.urlopen") as open_url:
            signaled = signal_coordinator(self.repo, repository_identity(self.repo).key)

        self.assertFalse(signaled)
        open_url.assert_not_called()

    def test_runtime_and_lock_pid_mismatch_fails_before_network(self) -> None:
        self._write_runtime(43123)
        lock = self.repo / ".codex" / "state" / "session-coordinator" / "coordinator.lock"
        lock.write_text(json.dumps({"pid": 999999}), encoding="utf-8")

        with patch("tools.session_coordinator.codex_sync.hook.urllib.request.urlopen") as open_url:
            signaled = signal_coordinator(self.repo, repository_identity(self.repo).key)

        self.assertFalse(signaled)
        open_url.assert_not_called()

    def test_slow_daemon_timeout_keeps_signal_below_half_second(self) -> None:
        class SlowHandler(BaseHTTPRequestHandler):
            def do_POST(self) -> None:
                time.sleep(0.8)
                try:
                    self.send_response(202)
                    self.end_headers()
                except OSError:
                    pass

            def log_message(self, *_args) -> None:
                return

        server = ThreadingHTTPServer(("127.0.0.1", 0), SlowHandler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        self.addCleanup(server.server_close)
        self.addCleanup(server.shutdown)
        self._write_runtime(server.server_port)

        started = time.perf_counter()
        signaled = signal_coordinator(self.repo, repository_identity(self.repo).key)
        elapsed = time.perf_counter() - started

        self.assertFalse(signaled)
        self.assertLess(elapsed, 0.5)


if __name__ == "__main__":
    unittest.main()

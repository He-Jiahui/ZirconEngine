from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from importlib.util import module_from_spec, spec_from_file_location


def _guard_module():
    path = Path(__file__).resolve().parents[3] / ".codex/hooks/pre_tool_use_cargo_guard.py"
    spec = spec_from_file_location("pre_tool_use_cargo_guard", path)
    assert spec is not None and spec.loader is not None
    module = module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class CargoGuardTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.repo = Path(self.temporary.name) / "repo"
        self.repo.mkdir()
        self.guard = _guard_module()

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def _payload(self, command: str) -> dict[str, object]:
        return {
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "cwd": str(self.repo),
            "session_id": "session-a",
            "turn_id": "turn-a",
            "tool_input": {"command": command},
        }

    def test_rejects_direct_artifact_producing_cargo_and_logs_no_command_text(self) -> None:
        decision = self.guard.evaluate_pre_tool_use(self._payload("& cargo.exe test -p zircon_runtime"), self.repo)

        self.assertFalse(decision.allowed)
        self.assertEqual("test", decision.subcommand)
        log = self.repo / ".codex/state/session-coordinator/logs/blocked-cargo.jsonl"
        record = json.loads(log.read_text(encoding="utf-8").strip())
        self.assertEqual("test", record["subcommand"])
        self.assertNotIn("zircon_runtime", log.read_text(encoding="utf-8"))
        self.assertNotIn("command", record)

    def test_allows_read_only_or_coordinator_aware_commands(self) -> None:
        for command in (
            "cargo metadata --no-deps",
            "cargo tree -p zircon_runtime",
            "cargo fmt --check",
            "& .\\.codex\\skills\\zircon-dev\\scripts\\validate-matrix.ps1 -Package zircon_runtime",
            ".\\tools\\zircon-session.ps1 cargo acquire check --session-id session-a",
        ):
            with self.subTest(command=command):
                decision = self.guard.evaluate_pre_tool_use(self._payload(command), self.repo)
                self.assertTrue(decision.allowed)

    def test_ignores_non_bash_or_external_working_directory(self) -> None:
        payload = self._payload("cargo check")
        payload["tool_name"] = "apply_patch"
        self.assertTrue(self.guard.evaluate_pre_tool_use(payload, self.repo).allowed)
        payload = self._payload("cargo check")
        payload["cwd"] = str(self.repo.parent)
        self.assertTrue(self.guard.evaluate_pre_tool_use(payload, self.repo).allowed)


if __name__ == "__main__":
    unittest.main()

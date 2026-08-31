from __future__ import annotations

import ast
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "tools" / "session_coordinator" / "offline_queue.py"


class OfflineQueueSingleSnapshotPerformanceContractTests(unittest.TestCase):
    def test_replay_locked_validates_pending_queue_once(self) -> None:
        tree = ast.parse(SOURCE.read_text(encoding="utf-8"))
        replay = next(
            node
            for node in ast.walk(tree)
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
            and node.name == "_replay_locked"
        )
        calls = [
            node
            for node in ast.walk(replay)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Attribute)
            and node.func.attr in {"validated_pending", "_validated_pending"}
        ]

        self.assertEqual(1, len(calls))

    def test_replay_locked_does_not_rescan_queue_directories(self) -> None:
        tree = ast.parse(SOURCE.read_text(encoding="utf-8"))
        replay = next(
            node
            for node in ast.walk(tree)
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
            and node.name == "_replay_locked"
        )
        ordered_calls = [
            node
            for node in ast.walk(replay)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Attribute)
            and node.func.attr == "_ordered"
        ]

        self.assertEqual([], ordered_calls)


if __name__ == "__main__":
    unittest.main()

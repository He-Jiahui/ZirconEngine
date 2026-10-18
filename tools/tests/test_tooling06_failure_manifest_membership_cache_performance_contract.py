from __future__ import annotations

import ast
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "tools/session_coordinator/failures.py"


class FailureManifestMembershipCachePerformanceContract(unittest.TestCase):
    def test_open_for_manifest_builds_one_membership_index(self) -> None:
        tree = ast.parse(SOURCE.read_text(encoding="utf-8"))
        service = next(
            node
            for node in tree.body
            if isinstance(node, ast.ClassDef) and node.name == "FailureGraphService"
        )
        function = next(
            node
            for node in service.body
            if isinstance(node, ast.FunctionDef) and node.name == "open_for_manifest"
        )
        frozen_calls = [
            node
            for node in ast.walk(function)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Name)
            and node.func.id == "frozenset"
        ]
        self.assertEqual(len(frozen_calls), 1)

    def test_child_slice_helpers_do_not_rebuild_manifest_sets(self) -> None:
        tree = ast.parse(SOURCE.read_text(encoding="utf-8"))
        service = next(
            node
            for node in tree.body
            if isinstance(node, ast.ClassDef) and node.name == "FailureGraphService"
        )
        helper_names = {
            "_complete_child_record_source_slice",
            "_has_exact_child_record_evidence",
        }
        helpers = [
            node
            for node in service.body
            if isinstance(node, ast.FunctionDef) and node.name in helper_names
        ]
        self.assertEqual({node.name for node in helpers}, helper_names)
        rebuilding_calls = [
            node
            for helper in helpers
            for node in ast.walk(helper)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Name)
            and node.func.id in {"set", "frozenset"}
        ]
        self.assertEqual(rebuilding_calls, [])


if __name__ == "__main__":
    unittest.main()

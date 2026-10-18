from __future__ import annotations

import ast
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "tools/session_coordinator/artifact_receipts.py"


class ArtifactReceiptManifestSetSingleProjectionPerformanceContract(unittest.TestCase):
    def test_copy_manifest_builds_one_immutable_membership_projection(self) -> None:
        tree = ast.parse(SOURCE.read_text(encoding="utf-8"))
        service = next(
            node
            for node in tree.body
            if isinstance(node, ast.ClassDef) and node.name == "ManagedArtifactReceiptService"
        )
        function = next(
            node
            for node in service.body
            if isinstance(node, ast.FunctionDef) and node.name == "_copy_manifest"
        )
        constructors = [
            node.func.id
            for node in ast.walk(function)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Name)
            and node.func.id in {"set", "frozenset"}
        ]
        self.assertEqual(constructors, ["frozenset"])


if __name__ == "__main__":
    unittest.main()

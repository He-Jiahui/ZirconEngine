"""Read-only behavior checks for the Codex directory inventory."""
from __future__ import annotations

import contextlib
import hashlib
import io
import json
from pathlib import Path
from types import SimpleNamespace
import stat
import unittest
from unittest.mock import patch

from tools.dev import artifact_inventory as inventory


class ArtifactInventoryTests(unittest.TestCase):
    def test_existing_source_observation_does_not_modify_inputs(self) -> None:
        path = Path(inventory.__file__).resolve()
        before = path.stat().st_mtime_ns, hashlib.sha256(path.read_bytes()).hexdigest()
        fact = inventory.tree_fact(path, inspect_source=True)
        after = path.stat().st_mtime_ns, hashlib.sha256(path.read_bytes()).hexdigest()
        self.assertEqual(before, after)
        self.assertEqual(1, fact["files"])
        self.assertEqual(path.stat().st_size, fact["logicalBytes"])
        self.assertEqual(1, fact["sourceFiles"])
        self.assertTrue(fact["complete"])

    def test_hardlinks_are_logical_bytes_without_reclamation_claims(self) -> None:
        info = SimpleNamespace(st_mode=stat.S_IFREG, st_mtime=1, st_size=6, st_nlink=2)
        with patch.object(Path, "lstat", return_value=info):
            fact = inventory.tree_fact(Path("E:/cargo-targets/zircon-local/synthetic.bin"))
        self.assertEqual(6, fact["logicalBytes"])
        self.assertEqual(1, fact["hardlinkedFiles"])
        self.assertNotIn("reclaimedBytes", fact)

    def test_reparse_directory_is_not_followed(self) -> None:
        path = Path("E:/cargo-targets/zircon-local/synthetic-link")
        info = SimpleNamespace(st_mode=stat.S_IFDIR, st_file_attributes=inventory.REPARSE)
        with patch.object(Path, "lstat", return_value=info), patch.object(
                inventory.os, "scandir", side_effect=AssertionError("followed reparse point")):
            fact = inventory.tree_fact(path)
            self.assertEqual(0, fact["files"])
            self.assertEqual([str(path)], fact["reparsePoints"])
            self.assertFalse(fact["complete"])
            with self.assertRaises(ValueError):
                inventory.physical_literal(path / "anything.bin")

    def test_missing_input_is_incomplete(self) -> None:
        with patch.object(Path, "lstat", side_effect=FileNotFoundError):
            fact = inventory.tree_fact(Path("E:/cargo-targets/zircon-local/synthetic-missing"))
        self.assertFalse(fact["complete"])
        self.assertTrue(fact["errors"])
        self.assertEqual(0, fact["files"])

    def test_narrow_read_exception_does_not_become_write_ownership(self) -> None:
        root = Path("E:/cargo-targets/zircon-local/synthetic")
        grants = {str(root): "write", str(root / "protected"): "read"}
        self.assertTrue(inventory.writable(root / "allowed/file", grants))
        self.assertFalse(inventory.writable(root / "protected/file", grants))
        self.assertFalse(inventory.writable(root.parent / "outside", grants))

    def test_relative_and_traversal_paths_are_rejected(self) -> None:
        for value in ("relative/directory", "E:/cargo-targets/zircon-local/../outside"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                inventory.physical_literal(Path(value))

    def test_process_match_retains_birth_without_emitting_arguments(self) -> None:
        root = Path("E:/cargo-targets/zircon-local/synthetic")
        row = {"ProcessId": 7, "Name": "synthetic.exe", "CreationDate": "observed-birth",
               "CommandLine": f'synthetic --input "{root}" --secret never-export-this',
               "ExecutablePath": None}
        matches = inventory.process_matches(root, [row])
        self.assertEqual([{"ProcessId": 7, "Name": "synthetic.exe",
                           "CreationDate": "observed-birth"}], matches)

    def test_source_build_and_control_state_remain_protected(self) -> None:
        repo = Path("E:/Git/ZirconEngine")
        self.assertEqual("canonical-source-or-docs", inventory.classify(repo / "zircon_runtime/build", repo))
        self.assertEqual("protected-control", inventory.classify(repo / ".git", repo))
        self.assertEqual("protected-control", inventory.classify(repo / ".jenkins", repo))
        self.assertEqual("protected-jenkins", inventory.classify(
            Path("D:/cargo-targets/zircon-local/zircon-jenkins"), repo))

    def test_cli_emits_json_without_file_writes(self) -> None:
        repo = Path(inventory.__file__).resolve().parents[2]
        result = {"summary": {"files": 1}, "inventory": []}
        with patch.object(inventory, "scan", return_value=result), patch.object(
                Path, "mkdir", side_effect=AssertionError("created directory")), patch.object(
                Path, "write_text", side_effect=AssertionError("wrote file")), contextlib.redirect_stdout(io.StringIO()) as out:
            self.assertEqual(0, inventory.main(["--repo", str(repo)]))
        self.assertEqual(result, json.loads(out.getvalue()))

    def test_cli_has_no_report_file_writer_option(self) -> None:
        with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit) as raised:
            inventory.main(["--report", "E:/cargo-targets/zircon-local/synthetic-report.json"])
        self.assertEqual(2, raised.exception.code)


if __name__ == "__main__":
    unittest.main()

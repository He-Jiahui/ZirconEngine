from __future__ import annotations

import os
import stat
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator import windows_tree_delete


@unittest.skipUnless(os.name == "nt", "Windows handle-bound deletion semantics")
class WindowsTreeDeleteTests(unittest.TestCase):
    def test_entry_handle_requests_only_delete_and_metadata_access(self) -> None:
        kernel32 = mock.Mock()
        kernel32.CreateFileW.return_value = 123

        with mock.patch.object(windows_tree_delete, "_kernel32", return_value=kernel32):
            entry = windows_tree_delete._open_entry(Path("candidate"))

        requested_access = kernel32.CreateFileW.call_args.args[1]
        self.assertEqual(
            windows_tree_delete._DELETE | windows_tree_delete._FILE_READ_ATTRIBUTES,
            requested_access,
        )
        entry.value = 0

    def test_non_windows_fallback_refuses_unbound_tree_deletion(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "candidate"
            root.mkdir()

            with mock.patch.object(windows_tree_delete.os, "name", "posix"):
                with self.assertRaises(OSError):
                    windows_tree_delete.remove_tree(root, expected_identity="durable")

            self.assertTrue(root.exists())

    def test_rejects_dangling_junction_recreated_after_root_handle_closes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            root = base / "candidate"
            root.mkdir()
            missing_target = base / "missing-target"
            original_close = windows_tree_delete._EntryHandle.close
            recreated = False

            def close_and_recreate(entry) -> None:
                nonlocal recreated
                original_close(entry)
                if entry.path == root and not recreated:
                    recreated = True
                    subprocess.run(
                        ["cmd.exe", "/c", "mklink", "/J", str(root), str(missing_target)],
                        check=True,
                        capture_output=True,
                        text=True,
                    )

            try:
                with mock.patch.object(
                    windows_tree_delete._EntryHandle,
                    "close",
                    autospec=True,
                    side_effect=close_and_recreate,
                ):
                    with self.assertRaises(OSError):
                        windows_tree_delete.remove_tree(root)
            finally:
                if root.is_junction():
                    os.rmdir(root)

            self.assertTrue(recreated)

    def test_removes_readonly_single_link_tree(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "candidate"
            readonly = root / ".git" / "objects" / "aa" / "object"
            readonly.parent.mkdir(parents=True)
            readonly.write_bytes(b"object")
            readonly.chmod(stat.S_IREAD)

            windows_tree_delete.remove_tree(root)

            self.assertFalse(root.exists())

    def test_rejects_child_replaced_by_outside_junction_during_traversal(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            root = base / "candidate"
            inner = root / "inner"
            outside = base / "outside"
            inner.mkdir(parents=True)
            outside.mkdir()
            marker = outside / "keep.txt"
            marker.write_text("keep", encoding="utf-8")
            original_open = windows_tree_delete._open_entry
            replaced = False

            def replace_before_open(path: Path):
                nonlocal replaced
                if path == inner and not replaced:
                    replaced = True
                    os.rmdir(inner)
                    subprocess.run(
                        ["cmd.exe", "/c", "mklink", "/J", str(inner), str(outside)],
                        check=True,
                        capture_output=True,
                        text=True,
                    )
                return original_open(path)

            try:
                with mock.patch.object(
                    windows_tree_delete, "_open_entry", side_effect=replace_before_open
                ):
                    with self.assertRaises(OSError):
                        windows_tree_delete.remove_tree(root)
            finally:
                if inner.exists() and inner.is_junction():
                    os.rmdir(inner)

            self.assertTrue(replaced)
            self.assertEqual("keep", marker.read_text(encoding="utf-8"))

    def test_readonly_hardlink_fails_without_changing_outside_attribute(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            outside = base / "outside.bin"
            outside.write_bytes(b"shared")
            root = base / "candidate"
            root.mkdir()
            inside = root / "inside.bin"
            os.link(outside, inside)
            outside.chmod(stat.S_IREAD)

            try:
                with self.assertRaises(PermissionError):
                    windows_tree_delete.remove_tree(root)

                self.assertTrue(outside.stat().st_file_attributes & stat.FILE_ATTRIBUTE_READONLY)
                self.assertEqual(b"shared", outside.read_bytes())
                self.assertTrue(inside.exists())
            finally:
                outside.chmod(stat.S_IWRITE)


if __name__ == "__main__":
    unittest.main()

from pathlib import Path
import os
import unittest
import uuid
from contextlib import contextmanager

from tools.jenkins.pilot.storage import ManagedStorage, StoragePathError, require_managed_root, physical_identity


@contextmanager
def temporary_directory():
    # Preserve actual evidence; do not acquire retired coordinator ownership.
    root = Path(r"D:\cargo-targets\zircon-jenkins") / f"tests-storage-{uuid.uuid4().hex}"
    with ManagedStorage(root).backend():
        yield root


class StorageTests(unittest.TestCase):
    def test_rejects_alias_roots_and_traversal(self):
        for value in (
            r"C:\cargo-targets\pilot", r"D:\targets\pilot", r"E:\Git\cargo-targets\pilot",
            r"D:\cargo-targets\..\elsewhere", r"D:\cargo-targets\pilot.\x",
            r"\\?\D:\cargo-targets\pilot", r"D:\cargo-targets\CON\x",
            r"D:\ZirconBuilds\pilot", r"D:\cargo-targets\nested\..\pilot",
        ):
            with self.subTest(value=value), self.assertRaises(StoragePathError):
                require_managed_root(value)

    def test_rejects_hard_link_and_unsafe_child_before_write(self):
        with temporary_directory() as temporary:
            root = Path(temporary)
            storage = ManagedStorage(root)
            original = root / "original"
            original.write_bytes(b"foreign")
            os.link(original, root / "alias")
            with self.assertRaises(StoragePathError):
                storage.atomic_write("alias", b"overwrite")
            for child in ("../escape", "nested/../escape", "D:relative", "folder./x"):
                with self.subTest(child=child), self.assertRaises(StoragePathError):
                    storage.atomic_write(child, b"no")
            self.assertEqual(original.read_bytes(), b"foreign")

    def test_managed_storage_uses_pinned_backend(self):
        with temporary_directory() as temporary:
            storage = ManagedStorage(Path(temporary))
            target = storage.atomic_write("nested/evidence.json", b"{}")
            self.assertEqual(target.read_bytes(), b"{}")
            self.assertTrue(storage.path("nested/evidence.json").is_relative_to(storage.root))

    def test_physical_identity_is_stable_after_children_change(self):
        with temporary_directory() as temporary:
            storage = ManagedStorage(temporary)
            before = physical_identity(storage.root)
            storage.atomic_write("evidence.json", b"{}")
            self.assertEqual(physical_identity(storage.root), before)
            self.assertEqual(len(before["fileId"]), 32)
            self.assertGreater(before["creationTime"], 0)

    def test_temporary_directory_has_exclusive_native_creation(self):
        with temporary_directory() as temporary:
            storage = ManagedStorage(temporary)
            first, second = storage.temporary_directory(), storage.temporary_directory()
            self.assertNotEqual(first, second)
            self.assertTrue(first.is_dir())
            self.assertTrue(second.is_dir())


if __name__ == "__main__":
    unittest.main()

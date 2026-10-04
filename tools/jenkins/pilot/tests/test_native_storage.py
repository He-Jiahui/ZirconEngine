"""Real Windows native storage guards independent of the retired coordinator."""
import hashlib
import os
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch
import uuid

from tools.jenkins.pilot.contracts import PilotError
from tools.jenkins.pilot.native.paths import (
    WorkerStorage, StorageConfigurationError, StorageSecurityError,
    FileIdentityMismatch, _validate_local_drive_target,
    _verify_local_drive_mapping, _verify_pinned_local_storage_mapping,
)
from tools.jenkins.pilot.native.portable_paths import normalize_portable_relative_path


class NativeStorageTests(unittest.TestCase):
    def root(self):
        return Path(r"D:\cargo-targets\zircon-jenkins") / f"tests-native-{uuid.uuid4().hex}"

    def test_portable_paths_reject_aliases_without_normalizing_them_away(self):
        for name in ("a//b", "a/./b", "../x", "folder./x", "a ", "NUL.txt", "COM¹.txt", "C:x", "a:b", "a\x00b"):
            with self.subTest(name=name), self.assertRaises(PilotError):
                normalize_portable_relative_path(name, code="path", message="unsafe")

    def test_physical_mapping_rejects_subst_network_and_duplicate_volume(self):
        for target in (r"\??\E:\cargo-targets", r"\Device\Mup\server\share", r"\Device\HarddiskVolume5\nested"):
            with self.subTest(target=target), self.assertRaises(StorageConfigurationError):
                _validate_local_drive_target("D:", target)
        with patch("tools.jenkins.pilot.native.paths._query_dos_device", return_value=r"\Device\HarddiskVolume5"):
            with self.assertRaises(StorageConfigurationError):
                _verify_local_drive_mapping("D:")
        for alias in ("C:", "G:"):
            def mapping(drive, *, required=True):
                return r"\Device\HarddiskVolume5" if drive in {"D:", alias} else None
            with self.subTest(alias=alias), patch("tools.jenkins.pilot.native.paths._query_dos_device", side_effect=mapping):
                with self.assertRaises(StorageConfigurationError):
                    _verify_local_drive_mapping("D:")

    def test_native_storage_rejects_unapproved_roots_and_policy_bypass(self):
        for root in (r"C:\cargo-targets\x", r"D:\targets\x", r"D:\ZirconBuilds\x",
                     r"E:\Git\cargo-targets\x", r"D:\cargo-targets\..\x",
                     r"\\?\D:\cargo-targets\x", r"D:\cargo-targets\x.\nested"):
            with self.subTest(root=root), self.assertRaises(PilotError):
                WorkerStorage(root)
        with self.assertRaises(StorageConfigurationError):
            WorkerStorage(self.root(), storage_class="remote")
        with self.assertRaises(TypeError):
            WorkerStorage(self.root(), test_policy=True)

    def test_identity_checked_atomic_replace_and_bounded_read(self):
        with WorkerStorage(self.root()) as storage:
            identity = storage.write_bytes("artifacts/proof", b"before")
            self.assertEqual(storage.read_bytes("artifacts/proof", expected_identity=identity), b"before")
            storage.write_bytes("artifacts/proof", b"after", expected_identity=identity)
            with self.assertRaises(FileIdentityMismatch):
                storage.read_bytes("artifacts/proof", expected_identity=identity)
            with self.assertRaises(StorageSecurityError):
                storage.read_bytes("artifacts/proof", max_bytes=2)
            with self.assertRaises(FileIdentityMismatch):
                storage.write_bytes("artifacts/proof", b"overwrite", expected_identity=identity)
            self.assertEqual(storage.read_bytes("artifacts/proof"), b"after")

    def test_hard_link_and_junction_never_expose_or_modify_foreign_data(self):
        root = self.root()
        with WorkerStorage(root) as storage:
            storage.write_bytes("artifacts/foreign", b"foreign")
            os.link(root / "artifacts/foreign", root / "artifacts/alias")
            for relative in ("artifacts/foreign", "artifacts/alias"):
                with self.subTest(relative=relative), self.assertRaises(StorageSecurityError):
                    storage.read_bytes(relative)
                with self.subTest(relative=relative), self.assertRaises(StorageSecurityError):
                    storage.write_bytes(relative, b"damage")
            # This creates only a new test junction; all test paths remain as evidence.
            subprocess.run(["cmd.exe", "/c", "mklink", "/J", str(root / "tmp/junction"), str(root / "artifacts")],
                           check=True, capture_output=True)
            with self.assertRaises(StorageSecurityError):
                storage.write_bytes("tmp/junction/foreign", b"damage")
            self.assertEqual((root / "artifacts/foreign").read_bytes(), b"foreign")

    def test_stream_failure_preserves_destination_and_pinned_parent_blocks_race(self):
        root = self.root()
        with WorkerStorage(root) as storage:
            storage.write_bytes("artifacts/proof", b"old")
            def chunks():
                with self.assertRaises(PermissionError):
                    os.rename(root / "artifacts", root / "renamed-artifacts")
                yield b"new"
            with self.assertRaises(StorageSecurityError):
                storage.atomic_write_stream("artifacts/proof", chunks(), expected_size=3,
                                            expected_sha256=hashlib.sha256(b"bad").hexdigest())
            self.assertEqual(storage.read_bytes("artifacts/proof"), b"old")
            self.assertEqual(storage.list_files("artifacts"), ["artifacts/proof"])
            with self.assertRaises(PermissionError):
                os.rename(root, root.with_name(root.name + "-replaced"))

    def test_mapping_change_after_pin_is_rejected(self):
        with WorkerStorage(self.root()) as storage:
            with patch("tools.jenkins.pilot.native.paths._verify_local_drive_mapping", return_value=r"\Device\HarddiskVolume999"):
                with self.assertRaises(StorageConfigurationError):
                    _verify_pinned_local_storage_mapping(storage._windows, storage.root, r"\Device\HarddiskVolume5")

    def test_wrong_volume_is_rejected_before_root_creation(self):
        root = self.root()
        with patch("tools.jenkins.pilot.native.paths._verify_local_drive_mapping", return_value=r"\Device\HarddiskVolume999"):
            with self.assertRaises(FileIdentityMismatch):
                WorkerStorage(root)
        self.assertFalse(root.exists())

    def test_lock_stream_pins_parent_and_file_and_contenders_share_byte_lock(self):
        import msvcrt
        root = self.root()
        with WorkerStorage(root) as first, WorkerStorage(root) as second:
            with first.open_lock_file("locks/writer.lock") as writer:
                writer.write(b"0")
                writer.seek(0)
                msvcrt.locking(writer.fileno(), msvcrt.LK_NBLCK, 1)
                try:
                    with self.assertRaises(PermissionError):
                        os.rename(root / "locks", root / "swapped-locks")
                    with self.assertRaises(PermissionError):
                        os.rename(root / "locks/writer.lock", root / "locks/swapped.lock")
                    with second.open_lock_file("locks/writer.lock") as contender:
                        contender.seek(0)
                        with self.assertRaises(OSError):
                            msvcrt.locking(contender.fileno(), msvcrt.LK_NBLCK, 1)
                finally:
                    writer.seek(0)
                    msvcrt.locking(writer.fileno(), msvcrt.LK_UNLCK, 1)
            with second.open_lock_file("locks/writer.lock") as next_writer:
                self.assertEqual(next_writer.read(), b"0")
                next_writer.seek(0)
                msvcrt.locking(next_writer.fileno(), msvcrt.LK_NBLCK, 1)
                next_writer.seek(0)
                msvcrt.locking(next_writer.fileno(), msvcrt.LK_UNLCK, 1)

    def test_lock_stream_rejects_hardlink_and_junction_before_modifying(self):
        root = self.root()
        with WorkerStorage(root) as storage:
            storage.write_bytes("artifacts/writer.lock", b"foreign")
            os.link(root / "artifacts/writer.lock", root / "artifacts/alias.lock")
            with self.assertRaises(StorageSecurityError):
                with storage.open_lock_file("artifacts/alias.lock"):
                    self.fail("hard-linked lock was exposed")
            subprocess.run(["cmd.exe", "/c", "mklink", "/J", str(root / "tmp/lock-junction"), str(root / "artifacts")],
                           check=True, capture_output=True)
            with self.assertRaises(StorageSecurityError):
                with storage.open_lock_file("tmp/lock-junction/new.lock"):
                    self.fail("junction lock was exposed")
            self.assertFalse((root / "artifacts/new.lock").exists())
            self.assertEqual((root / "artifacts/writer.lock").read_bytes(), b"foreign")

    def test_compiled_internal_aliases_are_proven_by_native_complete_link_names(self):
        root = self.root()
        with WorkerStorage(root) as storage:
            storage.write_bytes("build/build/proc-macro/build-script-hashed.exe", b"compiled-exe")
            os.link(root / "build/build/proc-macro/build-script-hashed.exe",
                    root / "build/build/proc-macro/build-script-build.exe")
            storage.write_bytes("build/deps/libmetadata.rmeta", b"compiled-metadata")
            storage.ensure_directory("build/incremental/session")
            os.link(root / "build/deps/libmetadata.rmeta", root / "build/incremental/session/metadata.rmeta")
            self.assertEqual(storage.list_compiled_products(), [
                "build/build/proc-macro/build-script-build.exe", "build/build/proc-macro/build-script-hashed.exe",
                "build/deps/libmetadata.rmeta", "build/incremental/session/metadata.rmeta"])
            self.assertIn("build/incremental/session", storage.list_compiled_directories())
            with self.assertRaises(StorageSecurityError):
                storage.list_files("build")
            with self.assertRaises(StorageSecurityError):
                storage.read_bytes("build/deps/libmetadata.rmeta")

    def test_compiled_aliases_to_external_or_state_names_are_rejected(self):
        for external in (True, False):
            root = self.root()
            outside = self.root()
            with self.subTest(external=external), WorkerStorage(root) as storage, WorkerStorage(outside):
                storage.write_bytes("build/deps/product.rmeta", b"must-preserve")
                alias = outside / "external.rmeta" if external else root / "state.json"
                os.link(root / "build/deps/product.rmeta", alias)
                with self.assertRaises(StorageSecurityError):
                    storage.list_compiled_products()
                with self.assertRaises(StorageSecurityError):
                    storage.list_compiled_directories()
                self.assertEqual(alias.read_bytes(), b"must-preserve")

    def test_compiled_alias_enumeration_failure_or_incomplete_set_is_rejected(self):
        root = self.root()
        with WorkerStorage(root) as storage:
            storage.write_bytes("build/deps/original.rmeta", b"compiled")
            os.link(root / "build/deps/original.rmeta", root / "build/deps/alias.rmeta")
            with patch("tools.jenkins.pilot.native.compiled_products._hardlink_names", return_value=["one-name"]):
                with self.assertRaises(FileIdentityMismatch):
                    storage.list_compiled_products()
            from tools.jenkins.pilot.native.windows_storage import WindowsStorageError
            with patch("tools.jenkins.pilot.native.compiled_products._hardlink_names", side_effect=WindowsStorageError(50, "unsupported")):
                with self.assertRaises(StorageSecurityError):
                    storage.list_compiled_products()
            with self.assertRaises(StorageSecurityError):
                storage.list_compiled_products("artifacts", allowed_directories=("artifacts",))


if __name__ == "__main__":
    unittest.main()

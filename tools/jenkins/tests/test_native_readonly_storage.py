"""Native read-only root pins keep all identity guards and forbid writes."""
from contextlib import ExitStack, contextmanager
import hashlib
import io
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.pilot.native import windows_storage as native
from tools.jenkins.workflow import artifact_protection as protection


class NativeReadOnlyStorageTests(unittest.TestCase):
    root = Path(r"D:\cargo-targets")
    device = r"\Device\HarddiskVolume5"

    def native_calls(self):
        stack = ExitStack()
        self.addCleanup(stack.close)
        self.handles, self.files = {10: "volume"}, set()
        self.missing, self.reparse = None, None
        stack.enter_context(patch.object(native, "is_available", return_value=True))
        self.volume = stack.enter_context(patch.object(native, "_open_volume_root", return_value=10))
        stack.enter_context(patch.object(native, "_final_nt_path", return_value=self.device + "\\"))
        def opened(parent, name, **kwargs):
            if name == self.missing:
                raise native.WindowsStorageError(2, "missing observed component")
            handle = 11 + len(self.handles)
            self.handles[handle] = name
            if kwargs.get("options", 0) & native._FILE_NON_DIRECTORY_FILE:
                self.files.add(handle)
            return handle
        self.opened = stack.enter_context(patch.object(native, "_open_relative_handle", side_effect=opened))
        stack.enter_context(patch.object(native, "_attributes", side_effect=lambda handle: SimpleNamespace(
            FileAttributes=(native._FILE_ATTRIBUTE_NORMAL if handle in self.files else native._FILE_ATTRIBUTE_DIRECTORY)
                           | (native._FILE_ATTRIBUTE_REPARSE_POINT if self.handles[handle] == self.reparse else 0))))
        stack.enter_context(patch.object(native, "identity_from_handle", side_effect=lambda handle:
            native.WindowsFileIdentity(123, handle.to_bytes(16, "little"), 0, 1000, 1)))
        self.closed = stack.enter_context(patch.object(native, "_close_handle"))
        @contextmanager
        def file_stream(handle, mode):
            self.assertEqual("rb", mode)
            try:
                yield io.BytesIO(b"")
            finally:
                native._close_handle(handle)
        stack.enter_context(patch.object(native.WindowsStorage, "_file_stream", side_effect=file_stream))
        return stack

    def test_readonly_root_requests_only_read_access_and_default_writer_root_is_unchanged(self):
        self.native_calls()
        for readonly in (True, False):
            with self.subTest(readonly=readonly):
                self.opened.reset_mock()
                with native.WindowsStorage(self.root, expected_device=self.device, readonly_root=readonly) as storage:
                    self.assertTrue(storage.verify_root())
                    self.assertEqual(native._DIRECTORY_READ_ACCESS if readonly else native._DIRECTORY_MUTATE_ACCESS,
                                     self.opened.call_args.kwargs["access"])
                    self.assertEqual(native._FILE_OPEN_REPARSE_POINT | native._FILE_DIRECTORY_FILE
                                     | native._FILE_SYNCHRONOUS_IO_NONALERT,
                                     self.opened.call_args.kwargs["options"])
                    self.assertFalse(self.opened.call_args.kwargs["share"] & 4)  # no FILE_SHARE_DELETE

    def test_readonly_mode_rejects_creation_before_opening_any_native_handle(self):
        self.native_calls()
        with self.assertRaises(ValueError):
            native.WindowsStorage(self.root, expected_device=self.device, create=True, readonly_root=True)
        self.volume.assert_not_called()
        self.opened.assert_not_called()

    def test_all_mutable_apis_are_denied_before_mutable_native_open(self):
        self.native_calls()
        def enter(context):
            with context:
                self.fail("read-only storage yielded a writable resource")
        with native.WindowsStorage(self.root, expected_device=self.device, readonly_root=True) as storage:
            self.opened.reset_mock()
            expected = storage.root_identity
            operations = (
                lambda: enter(storage.open_relative("evidence.bin", "r+b")),
                lambda: enter(storage.open_lock_file("locks/writer.lock")),
                lambda: storage.write_bytes("evidence.bin", b"x"),
                lambda: storage.atomic_write_stream("evidence.bin", (b"x",), expected_size=1,
                                                    expected_sha256=hashlib.sha256(b"x").hexdigest()),
                lambda: storage.delete("evidence.bin"),
                lambda: storage.ensure_directory("new/child"),
                lambda: storage.create_directory("new/child"),
                lambda: storage.delete_empty_directory("existing/child"),
                lambda: storage.set_mtime("evidence.bin", 100, expected_identity=expected),
                lambda: storage._open_leaf(10, "evidence.bin", "w+b", create=True),
            )
            for number, operation in enumerate(operations):
                with self.subTest(operation=number), self.assertRaises(native.WindowsStorageError) as raised:
                    operation()
                self.assertEqual(5, raised.exception.errno)
                self.opened.assert_not_called()

    def test_readonly_root_still_rejects_native_reparse_point_and_closes_pins(self):
        self.native_calls()
        self.reparse = "cargo-targets"
        with self.assertRaises(native.WindowsPathError):
            native.WindowsStorage(self.root, expected_device=self.device, readonly_root=True)
        self.assertEqual(set(self.handles), {call.args[0] for call in self.closed.call_args_list})

    def test_readonly_root_still_rejects_wrong_volume_before_component_open(self):
        self.native_calls()
        with self.assertRaises(native.WindowsIdentityMismatch):
            native.WindowsStorage(self.root, expected_device=r"\Device\HarddiskVolume999", readonly_root=True)
        self.opened.assert_not_called()
        self.closed.assert_called_once_with(10)

    def observe(self, stack):
        approved = SimpleNamespace(path=self.root, volume_identity=self.device)
        stack.enter_context(patch.object(protection, "canonical_build_root", return_value=approved))
        stack.enter_context(patch.object(protection, "physical_path_under"))
        return protection._observe_missing(approved, protection.OLD_PATH)

    def test_missing_blob_observation_uses_only_read_pins_without_creating_missing_component(self):
        stack = self.native_calls()
        self.missing = "artifacts"
        observed = self.observe(stack)
        self.assertEqual("missing", observed["status"])
        self.assertEqual(self.device, observed["volumeIdentity"])
        for call in self.opened.call_args_list:
            self.assertNotIn("disposition", call.kwargs)
            self.assertEqual(native._DIRECTORY_READ_ACCESS, call.kwargs["access"])
        self.assertEqual(set(self.handles), {call.args[0] for call in self.closed.call_args_list})

    def test_reappeared_blob_and_unqueryable_parent_never_count_as_absent(self):
        for case in ("present", "access-denied", "reparse"):
            with self.subTest(case=case):
                stack = self.native_calls()
                if case == "access-denied":
                    opened = self.opened.side_effect
                    def inaccessible(parent, name, **kwargs):
                        if name == "artifacts":
                            raise native.WindowsStorageError(5, "parent inaccessible")
                        return opened(parent, name, **kwargs)
                    self.opened.side_effect = inaccessible
                elif case == "reparse":
                    self.reparse = "zircon-jenkins"
                with self.assertRaises((JenkinsError, native.WindowsStorageError, native.WindowsPathError)):
                    self.observe(stack)
                self.assertEqual(set(self.handles), {call.args[0] for call in self.closed.call_args_list})
                stack.close()


if __name__ == "__main__":
    unittest.main()

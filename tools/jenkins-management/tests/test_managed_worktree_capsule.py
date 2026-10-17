"""Capsule-owned managed input readers retain the original reader contract."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import ntpath
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.resources import canonical_build_root
from tools.jenkins.tests import test_managed_worktree_preparation as baseline

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location(
    "managed_capsule_input_test_helper", ROOT / "tools/jenkins-management/source_preparation.py")
helper = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(helper)


@unittest.skipUnless(os.name == "nt", "Native management input pins require Windows")
class ManagedInputStageTests(unittest.TestCase):
    def setUp(self):
        area = canonical_build_root(r"E:\cargo-targets").namespace() / "tmp/managed-worktree-tests"
        area.mkdir(parents=True, exist_ok=True)
        temporary = tempfile.TemporaryDirectory(prefix="mgi-", dir=area)
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        (self.root / "shared").mkdir()
        (self.root / "shared/a.rs").write_bytes(b"abc")
        (self.root / "shared/b.rs").write_bytes(b"def")

    def test_directory_proof_is_reused_only_while_native_pin_is_held(self):
        native = helper._managed_native()
        with patch.object(native, "_open_relative_handle", wraps=native._open_relative_handle) as opened:
            for phase in ("first", "fresh"):
                with helper.managed_input_stage(self.root, phase) as stage:
                    stage.canonical("shared/a.rs")
                    stage.canonical("shared/b.rs")
            shared = [call for call in opened.call_args_list
                      if call.args[1] == "shared" and call.kwargs["options"] & native._FILE_DIRECTORY_FILE]
            self.assertEqual(2, len(shared))
            self.assertTrue(all(call.kwargs["share"] == native._FILE_SHARE_READ for call in shared))

    def test_bytes_and_absence_are_never_reused_within_a_stage(self):
        path = self.root / "shared/a.rs"
        before = path.stat()
        with helper.managed_input_stage(self.root, "fresh-bytes") as stage:
            self.assertEqual(hashlib.sha256(b"abc").hexdigest(), stage.checksum("shared/a.rs"))
            path.write_bytes(b"xyz")
            os.utime(path, ns=(before.st_atime_ns, before.st_mtime_ns))
            self.assertEqual(hashlib.sha256(b"xyz").hexdigest(), stage.checksum("shared/a.rs"))
            self.assertIsNone(stage.checksum("new/input.rs"))
            (self.root / "new").mkdir()
            (self.root / "new/input.rs").write_bytes(b"new")
            self.assertEqual(hashlib.sha256(b"new").hexdigest(), stage.checksum("new/input.rs"))

    def test_held_parent_denies_rename_and_reparse_capable_write_access(self):
        native = helper._managed_native()
        with helper.managed_input_stage(self.root, "pinned-parent") as stage:
            stage.canonical("shared/a.rs")
            with self.assertRaises(OSError):
                (self.root / "shared").rename(self.root / "moved")
            parent = stage.directories[ntpath.normcase(str(self.root))][0]
            with self.assertRaises(OSError):
                native._open_relative_handle(
                    parent, "shared",
                    access=native._FILE_WRITE_DATA | native._FILE_READ_ATTRIBUTES | native._SYNCHRONIZE,
                    share=native._FILE_SHARE_READ | native._FILE_SHARE_WRITE,
                    options=native._FILE_DIRECTORY_FILE | native._FILE_OPEN_REPARSE_POINT
                            | native._FILE_SYNCHRONOUS_IO_NONALERT)
        (self.root / "shared").rename(self.root / "moved")
        (self.root / "moved").rename(self.root / "shared")

    def test_junction_is_rejected_before_any_foreign_byte_is_read(self):
        alias, target = self.root / "alias", self.root / "shared"
        result = subprocess.run(["cmd.exe", "/d", "/c", "mklink", "/J", str(alias), str(target)],
                                capture_output=True, check=False)
        self.assertEqual(0, result.returncode, result.stderr.decode(errors="replace"))
        try:
            with helper.managed_input_stage(self.root, "junction") as stage:
                with self.assertRaises((JenkinsError, ValueError)):
                    stage.checksum("alias/a.rs")
                self.assertEqual(0, stage.byte_count)
        finally:
            os.rmdir(alias)

    def test_access_denied_never_becomes_absent(self):
        native = helper._managed_native()
        with helper.managed_input_stage(self.root, "access-denied") as stage:
            original = native._open_relative_handle
            def denied(parent, name, **kwargs):
                if name == "a.rs":
                    raise native.WindowsStorageError(5, "fixture access denied")
                return original(parent, name, **kwargs)
            with patch.object(native, "_open_relative_handle", side_effect=denied):
                with self.assertRaises(JenkinsError) as observed:
                    stage.checksum("shared/a.rs")
            self.assertEqual("managed_source_unavailable", observed.exception.code)

    def test_drift_diagnostic_names_changed_and_added_paths(self):
        with self.assertRaises(JenkinsError) as observed:
            helper._managed_drift({"docs/a.md": "old"}, {"docs/a.md": "new", "docs/b.md": "new"},
                                  "managed_source_changed", "fixture source drift")
        self.assertEqual(["docs/a.md"], observed.exception.details["changedPaths"])
        self.assertEqual(["docs/b.md"], observed.exception.details["addedPaths"])


@unittest.skipUnless(os.name == "nt", "Native management input pins require Windows")
class CapsuleManagedWorktreePreparationTests(baseline.ManagedWorktreePreparationTests):
    """Run the existing full Gitlink, authority, lifetime and reader fixtures."""

    def inventory(self, request, target_source):
        self.use({**self.payload("inspect-worktree", request, "w"), "targetSourceOperationKey": target_source,
                  "externalRoots": [str(self.external)]})
        result = helper.managed_inspect_worktree(self.management, preparation_helper=helper)
        key = self.complete(self.management.payload, result)
        self.inputs = json.loads(Path(result["preparationInputRef"]["path"]).read_bytes())["preparationInput"]
        self.reader = self.state.authorize_session(self.repo_id, "r", "user",
            sorted([*self.inputs["currentHashes"],
                    *["external/zr_vm/" + name for name in self.inputs["externalInputs"][0]["currentHashes"]]]),
            ["validation"], {"source": "user", "task": "fixture complete inspected reader"})
        return key, result["preparationInputRef"]

    def prepare(self, request, target_source):
        inspection, reference = self.inventory("inspect-" + request, target_source)
        self.use({**self.payload("prepare-worktree", request, "r"),
                  "inspectionOperationKey": inspection, "preparationInputRef": reference})
        with baseline.lifetime.writer_hold(self.management):
            result = helper.managed_prepare_worktree(self.management, preparation_helper=helper)
        return self.complete(self.management.payload, result), result

    def test_one_worktree_full_gitlinks_seal_terminal_retire_then_remove_prior_added_path(self):
        return super().test_one_worktree_full_gitlinks_seal_terminal_retire_then_remove_prior_added_path()

    def test_target_conflict_and_incomplete_native_proof_keep_target_or_reader_blocked(self):
        return super().test_target_conflict_and_incomplete_native_proof_keep_target_or_reader_blocked()

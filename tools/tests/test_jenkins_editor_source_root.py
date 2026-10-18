"""Managed source CLI admission and immutable recovery use the selected sealed declaration."""
import hashlib
import json
from pathlib import Path
import unittest
from unittest.mock import patch

from tools.dev import jenkins_editor as entry
from tools.dev.jenkins_validate import EntryError
from tools.tests import test_jenkins_editor_entry as lifecycle


class ManagedEditorSourceTests(unittest.TestCase):
    def setUp(self):
        self.fixture = lifecycle.EditorOperatorLifecycleTests("test_dry_check_does_not_dispatch_or_create_lifecycle")
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.root = self.fixture.root
        self.source = self.root / ".jenkins/worktrees/writer-task"
        self.source.mkdir(parents=True)

    def args(self, action="submit", *extra):
        return self.fixture.args(action, "--source-root", str(self.source), "--worktree-id", "writer-task", *extra)

    def driver(self, declaration):
        contents = {"launcher.py": b"# synthetic launcher", "tools/jenkins/current_source.py": declaration}
        files = {name: {"sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}
                 for name, data in contents.items()}
        checksum = entry.digest({"schemaVersion": 1, "files": files})
        root = self.root / ".jenkins/runtime/drivers" / checksum
        for name, data in contents.items():
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        (root / "driver-manifest.json").write_text(json.dumps(
            {"schemaVersion": 1, "driverDigest": checksum, "files": files}), encoding="utf-8")
        return {"digest": checksum, "runtimeOperationId": "start-synthetic", "launcher": root / "launcher.py"}

    def test_missing_pair_and_outside_managed_child_reject_before_protocol(self):
        cases = [self.fixture.args("submit", "--source-root", str(self.source)),
                 self.fixture.args("submit", "--worktree-id", "writer-task"),
                 self.fixture.args("submit", "--source-root", str(self.source), "--worktree-id", "writer-other")]
        for args in cases:
            with self.subTest(args=args), self.assertRaises(EntryError):
                self.fixture.execute(args)
        self.assertEqual([], self.fixture.calls)
        folder = self.root / ".jenkins/state/requests/operator-editor" / entry.digest(
            [entry.repository_id(self.root), "session", "request"])
        self.assertFalse((folder / "intent.json").exists())
        self.assertFalse((folder / "lifecycle.json").exists())

    def test_only_exact_new_literal_in_verified_selected_manifest_is_admitted(self):
        declarations = [b"SOURCE_ROOT_CAPABILITY = 'same-repository-worktree-v1'",
                        b"# SOURCE_ROOT_CAPABILITY = 'managed-worktree-reference-v1'",
                        b"SOURCE_ROOT_CAPABILITY = 'managed-worktree-' + 'reference-v1'",
                        b"SOURCE_ROOT_CAPABILITY: str = 'managed-worktree-reference-v1'",
                        b"SOURCE_ROOT_CAPABILITY = 'managed-worktree-reference-v1'\nSOURCE_ROOT_CAPABILITY = 'other'"]
        for declaration in declarations:
            driver = self.driver(declaration)
            with self.subTest(declaration=declaration), self.assertRaises(EntryError) as caught:
                self.fixture.execute(self.args(), driver=driver)
            self.assertEqual("source_root_unsupported", caught.exception.code)
        self.assertEqual([], self.fixture.calls)
        driver = self.driver(b"SOURCE_ROOT_CAPABILITY = 'managed-worktree-reference-v1'")
        result = self.fixture.execute(self.args(), driver=driver)
        self.assertEqual("submitted", result["status"])
        self.assertEqual("writer-task", self.fixture.calls[0][2]["worktreeId"])
        self.assertEqual(str(self.source), self.fixture.calls[0][2]["sourceRoot"])

    def test_source_reparse_ancestor_and_tampered_sealed_bytes_reject_before_protocol(self):
        original = Path.lstat
        def lstat(path, *args, **kwargs):
            if path == self.source.parent:
                from types import SimpleNamespace
                return SimpleNamespace(st_mode=0o040755, st_file_attributes=0x400)
            return original(path, *args, **kwargs)
        with patch.object(Path, "lstat", lstat), self.assertRaises(EntryError):
            self.fixture.execute(self.args())
        driver = self.driver(b"SOURCE_ROOT_CAPABILITY = 'managed-worktree-reference-v1'")
        (Path(driver["launcher"]).parent / "tools/jenkins/current_source.py").write_bytes(b"tampered")
        with self.assertRaises(EntryError) as caught:
            self.fixture.execute(self.args(), driver=driver)
        self.assertEqual("driver_integrity_failed", caught.exception.code)
        self.assertEqual([], self.fixture.calls)

    def test_replay_revalidates_capability_and_refuses_a_changed_worktree_id(self):
        driver = self.driver(b"SOURCE_ROOT_CAPABILITY = 'managed-worktree-reference-v1'")
        self.fixture.execute(self.args(), driver=driver)
        original = list(self.fixture.calls)
        with self.assertRaises(EntryError):
            self.fixture.execute(self.fixture.args("query", "--worktree-id", "writer-other"), driver=driver)
        self.assertEqual(original, self.fixture.calls)
        with patch.object(entry, "_require_source_root_capability",
                          side_effect=EntryError("source_root_unsupported", "old selected implementation")):
            with self.assertRaises(EntryError):
                self.fixture.execute(self.fixture.args("query"), driver=driver)
        self.assertEqual(original, self.fixture.calls)

    def test_resume_reuses_the_saved_pair_after_not_found_without_rebuilding_intent(self):
        driver = self.driver(b"SOURCE_ROOT_CAPABILITY = 'managed-worktree-reference-v1'")
        def interrupted(root, selected, folder, domain, action, payload, timeout):
            self.fixture.calls.append((domain, action, payload))
            raise EntryError("protocol_delivery_unknown", "synthetic unknown", retryable=True)
        with self.assertRaises(EntryError):
            self.fixture.execute(self.args(), driver=driver, call=interrupted)
        args = self.fixture.args("resume-registration")
        with patch.object(entry, "_terminal_registration", return_value={"path": "synthetic", "sha256": "e" * 64}), \
                patch.object(entry, "intent_payload", side_effect=AssertionError("immutable intent must be reused")):
            result = self.fixture.execute(args, driver=driver, call=self.fixture.resumed_call)
        self.assertEqual("submitted", result["status"])
        self.assertEqual(["register", "query", "register", "submit"], [item[1] for item in self.fixture.calls])
        self.assertEqual(self.fixture.calls[0][2], self.fixture.calls[2][2])
        self.assertEqual("writer-task", self.fixture.calls[2][2]["worktreeId"])
        self.assertEqual(str(self.source), self.fixture.calls[2][2]["sourceRoot"])


if __name__ == "__main__":
    unittest.main()

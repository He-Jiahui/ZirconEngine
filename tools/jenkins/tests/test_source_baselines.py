from __future__ import annotations

import hashlib
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.source import repository_id
from tools.jenkins.source import baselines
from tools.jenkins.source.unified_patch import render_patch_bytes


def sha(data):
    return hashlib.sha256(data).hexdigest()


class SourceBaselineTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "src").mkdir()
        self.original = b"old\r\nlast"
        (self.root / "src/a.rs").write_bytes(self.original)
        (self.root / "delete.rs").write_bytes(b"delete\n")
        self.git("init", "-q")
        self.git("config", "core.autocrlf", "false")
        self.git("add", "src/a.rs", "delete.rs")
        self.git("commit", "-q", "-m", "immutable fixture")
        self.head = self.git("rev-parse", "HEAD").decode().strip()
        self.repo = repository_id(self.root)

    def git(self, *args):
        # Only test fixture setup writes this disposable repository. The public
        # resolver below uses read commands and never touches the real checkout.
        env = {k: v for k, v in os.environ.items() if not k.upper().startswith("GIT_")}
        env.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull)
        result = subprocess.run(["git", "-C", str(self.root), "-c", "user.name=Fixture",
                                 "-c", "user.email=fixture@example.invalid", *args],
                                env=env, capture_output=True, check=True)
        return result.stdout

    def resolve(self, hashes=None, **changes):
        arguments = {"repository_id_value": self.repo, "base_head": self.head,
                     "before_hashes": hashes or {"src/a.rs": sha(self.original)}}
        return baselines.resolve_git_baseline(self.root, **{**arguments, **changes})

    def test_dirty_checkout_does_not_supply_before_and_git_or_source_bytes_are_not_written(self):
        after = b"new\r\nlast"
        (self.root / "src/a.rs").write_bytes(after)
        before_index = (self.root / ".git/index").read_bytes()
        trusted = self.resolve()
        self.assertEqual({"src/a.rs": self.original}, dict(trusted.before_bytes))
        self.assertEqual(self.repo, trusted.provenance.repository_id)
        self.assertEqual(self.head, trusted.provenance.base_head)
        self.assertEqual("100644", trusted.provenance.paths[0].mode)
        diff = ("diff --git a/src/a.rs b/src/a.rs\n--- a/src/a.rs\n+++ b/src/a.rs\n@@ -1,2 +1,2 @@\n"
                "-old\n+new\n last\n\\ No newline at end of file\n")
        self.assertEqual({"src/a.rs": after}, render_patch_bytes(diff, trusted.before_bytes,
                                                                {"src/a.rs": sha(self.original)}))
        self.assertEqual(after, (self.root / "src/a.rs").read_bytes())
        self.assertEqual(before_index, (self.root / ".git/index").read_bytes())
        self.assertEqual(self.head, self.git("rev-parse", "HEAD").decode().strip())
        with self.assertRaises(TypeError):
            trusted.before_bytes["src/a.rs"] = after
        self.assertNotIn("b'old", repr(trusted))

    def test_foreign_dirty_preimage_hash_is_not_accepted_as_git_authority(self):
        foreign = b"another author's before\n"
        (self.root / "src/a.rs").write_bytes(foreign)
        with self.assertRaises(JenkinsError) as error:
            self.resolve({"src/a.rs": sha(foreign)})
        self.assertEqual("before_hash_mismatch", error.exception.code)
        self.assertEqual(foreign, (self.root / "src/a.rs").read_bytes())

    def test_addition_absence_and_deleted_worktree_file_use_immutable_commit(self):
        (self.root / "delete.rs").unlink()
        (self.root / "new.rs").write_bytes(b"already added\n")
        trusted = self.resolve({"new.rs": None, "delete.rs": sha(b"delete\n")})
        self.assertEqual({"new.rs": None, "delete.rs": b"delete\n"}, dict(trusted.before_bytes))
        with self.assertRaises(JenkinsError):
            self.resolve({"src/a.rs": None})

    def test_forged_repository_ref_and_path_fail_closed(self):
        for changes in [{"repository_id_value": "0" * 32}, {"base_head": "HEAD"},
                        {"base_head": "f" * 40}, {"before_hashes": {"../outside": None}},
                        {"before_hashes": {".git/config": None}},
                        {"before_hashes": {"src/a.rs": sha(self.original), "SRC/A.rs": sha(self.original)}}]:
            with self.subTest(changes=changes), self.assertRaises(JenkinsError):
                self.resolve(**changes)

    def test_head_drift_during_resolution_rejects_complete_result(self):
        original_git = baselines._git
        reads = 0

        def changing_head(root, *args, **kwargs):
            nonlocal reads
            if args == ("rev-parse", "--verify", "HEAD^{commit}"):
                reads += 1
                if reads == 2:
                    return b"f" * 40 + b"\n"
            return original_git(root, *args, **kwargs)

        with patch.object(baselines, "_git", side_effect=changing_head), self.assertRaises(JenkinsError) as error:
            self.resolve()
        self.assertEqual("baseline_base_changed", error.exception.code)

    def test_git_executable_blob_keeps_exact_mode_and_never_reads_dirty_before(self):
        self.git("update-index", "--chmod=+x", "src/a.rs")
        self.git("commit", "-q", "-m", "unsupported mode")
        self.head = self.git("rev-parse", "HEAD").decode().strip()
        dirty = b"dirty executable postimage\n"
        (self.root / "src/a.rs").write_bytes(dirty)
        trusted = self.resolve()
        self.assertEqual("100755", trusted.provenance.paths[0].mode)
        self.assertEqual(sha(self.original), trusted.provenance.paths[0].sha256)
        self.assertEqual(self.original, trusted.before_bytes["src/a.rs"])
        self.assertEqual(dirty, (self.root / "src/a.rs").read_bytes())

    def test_git_object_corruption_is_rejected(self):
        original_git = baselines._git

        def corrupt_blob(root, *args, **kwargs):
            if args[:2] == ("cat-file", "blob"):
                return b"corrupt"
            return original_git(root, *args, **kwargs)

        with patch.object(baselines, "_git", side_effect=corrupt_blob), self.assertRaises(JenkinsError) as error:
            self.resolve()
        self.assertEqual("baseline_object_corrupt", error.exception.code)

    def test_multi_file_mismatch_or_unknown_reference_returns_no_baseline(self):
        with self.assertRaises(JenkinsError):
            self.resolve({"delete.rs": sha(b"delete\n"), "src/a.rs": "0" * 64})
        for reference in ["unissued-baseline", ""]:
            with self.subTest(reference=reference), patch.object(baselines, "_git") as command:
                with self.assertRaises(JenkinsError) as error:
                    baselines.resolve_baseline(self.root, repository_id_value=self.repo,
                                               before_hashes={"src/a.rs": sha(self.original)},
                                               base_head=self.head, baseline_ref=reference)
                self.assertEqual("baseline_required", error.exception.code)
                command.assert_not_called()

    def test_partial_clone_is_rejected_before_any_object_read(self):
        self.git("config", "remote.origin.promisor", "true")
        with patch.object(baselines, "_object") as read:
            with self.assertRaises(JenkinsError) as error:
                self.resolve()
        self.assertEqual("baseline_required", error.exception.code)
        read.assert_not_called()

    def test_reparse_path_is_rejected_even_if_it_resolves_inside_checkout(self):
        original = Path.lstat

        def reparse(path, *args, **kwargs):
            result = original(path, *args, **kwargs)
            if path == self.root / "src":
                class Reparse:
                    st_file_attributes = 0x400

                    def __getattr__(self, name):
                        return getattr(result, name)
                return Reparse()
            return result

        with patch.object(Path, "lstat", reparse), self.assertRaises(JenkinsError):
            self.resolve()


if __name__ == "__main__":
    unittest.main()

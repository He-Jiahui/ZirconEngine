from pathlib import Path
import subprocess
import tempfile
import unittest
import zipfile

from tools.jenkins.pilot.contracts import PilotError
from tools.jenkins.pilot.snapshot import capture, load_snapshot, materialize, verify_materialized
from tools.jenkins.pilot.storage import ManagedStorage
from tools.jenkins.pilot.tests.fixtures import temporary_directory
from tools.jenkins.pilot.source_manifest import manifest_digest, tree_manifest


class SnapshotTests(unittest.TestCase):
    def setUp(self):
        self.temporary = temporary_directory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        self.git("init", "--quiet")
        self.git("config", "user.name", "Pilot fixture")
        self.git("config", "user.email", "fixture@localhost")
        self.git("config", "core.autocrlf", "false")
        (self.repo / "kept.py").write_text("x = 1\n", encoding="utf-8")
        (self.repo / "modified.py").write_text("x = 2\n", encoding="utf-8")
        (self.repo / "deleted.py").write_text("x = 3\n", encoding="utf-8")
        self.git("add", ".")
        self.git("commit", "--quiet", "-m", "fixture base")
        self.storage = ManagedStorage(self.root / "storage")

    def git(self, *args):
        return subprocess.check_output(["git", "-C", str(self.repo), *args], stderr=subprocess.PIPE)

    def test_seals_modified_deleted_new_and_unchanged_inputs(self):
        (self.repo / "modified.py").write_text("x = 20\n", encoding="utf-8")
        (self.repo / "deleted.py").unlink()
        (self.repo / "new.py").write_text("x = 4\n", encoding="utf-8")
        sealed = capture(self.repo, self.storage, paths=["."], untracked_allowlist=["new.py"])
        states = {entry["path"]: entry["status"] for entry in sealed.manifest["entries"]}
        self.assertEqual(states, {"kept.py": "base", "modified.py": "modified", "deleted.py": "deleted", "new.py": "new"})
        (self.repo / "modified.py").write_text("contamination\n", encoding="utf-8")
        destination = self.root / "materialized"
        materialize(sealed.bundle, destination, repo_root=self.repo, expected_input_hash=sealed.input_hash)
        self.assertEqual((destination / "modified.py").read_text(), "x = 20\n")
        self.assertFalse((destination / "deleted.py").exists())
        self.assertEqual(verify_materialized(sealed, destination), sealed.input_hash)
        (destination / "kept.py").write_text("drift")
        with self.assertRaises(PilotError):
            verify_materialized(sealed, destination)

    def test_requires_explicit_new_file_admission(self):
        (self.repo / "new.py").write_text("new")
        with self.assertRaises(PilotError):
            capture(self.repo, self.storage, paths=["."])

    def test_rejects_secret_paths_and_case_collisions(self):
        (self.repo / ".env").write_text("PASSWORD=secret")
        with self.assertRaises(PilotError):
            capture(self.repo, self.storage, paths=["."], untracked_allowlist=[".env"])

    def test_rejects_extra_archive_member_and_wrong_input_identity(self):
        sealed = capture(self.repo, self.storage, paths=["."])
        with self.assertRaises(PilotError):
            load_snapshot(sealed.bundle, expected_input_hash="0" * 64)
        with zipfile.ZipFile(sealed.bundle, "a") as bundle:
            bundle.writestr("../escape", b"bad")
        with self.assertRaises(PilotError):
            load_snapshot(sealed.bundle)

    def test_extra_materialized_input_cannot_escape_digest(self):
        sealed = capture(self.repo, self.storage, paths=["."])
        destination = self.root / "materialized"
        materialize(sealed.bundle, destination, repo_root=self.repo, expected_input_hash=sealed.input_hash)
        (destination / "hidden.py").write_text("unexpected")
        with self.assertRaises(PilotError):
            verify_materialized(sealed, destination)

    def test_canonical_manifest_rejects_path_aliases_and_digest_matches_records(self):
        import hashlib
        import json
        manifest = tree_manifest(self.repo, exclude=frozenset({".git"}))
        records = [{"path": name, "sha256": manifest[name]} for name in sorted(manifest, key=str.casefold)]
        expected = hashlib.sha256(json.dumps(records, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
        self.assertEqual(manifest_digest(manifest), expected)
        for name in ("../escape", "a/../escape", "CON.rs", "a:stream", "trailing."):
            with self.subTest(name=name), self.assertRaises(PilotError):
                manifest_digest({name: "a" * 64})

    def test_linked_source_root_rejected_before_capture(self):
        alias = self.root / "alias"
        try:
            alias.symlink_to(self.repo, target_is_directory=True)
        except OSError:
            self.skipTest("Windows symbolic link privilege unavailable")
        with self.assertRaises(PilotError):
            capture(alias, self.storage, paths=["."])


if __name__ == "__main__":
    unittest.main()

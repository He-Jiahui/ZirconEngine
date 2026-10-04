from pathlib import Path
import hashlib
import io
import json
import subprocess
import unittest
from unittest.mock import patch
import zipfile

from tools.jenkins.pilot.contracts import PilotError, canonical_json, digest
from tools.jenkins.pilot.snapshot import capture, load_snapshot, materialize, verify_materialized
from tools.jenkins.pilot.storage import ManagedStorage
from tools.jenkins.pilot.tests.fixtures import temporary_directory


class ExternalSnapshotTests(unittest.TestCase):
    def setUp(self):
        self.temporary = temporary_directory(prefix="external-fixture-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.repo = self.root / "repo"
        self.external = self.root / "zr_vm"
        for repo in (self.repo, self.external):
            repo.mkdir()
            self.git(repo, "init", "--quiet")
            self.git(repo, "config", "user.name", "Pilot fixture")
            self.git(repo, "config", "user.email", "fixture@localhost")
            self.git(repo, "config", "core.autocrlf", "false")
            for name in ("kept", "dirty", "deleted"):
                (repo / name).write_bytes(name.encode())
            self.git(repo, "add", ".")
            self.git(repo, "commit", "--quiet", "-m", "base")
            (repo / "dirty").write_bytes(b"changed")
            (repo / "new").write_bytes(b"new")
            (repo / "deleted").unlink()
        self.storage = ManagedStorage(self.root / "storage")
        self.destination = self.root / "run" / "sealed" / "source"

    def git(self, repo, *args):
        return subprocess.check_output(["git", "--no-optional-locks", "-C", str(repo), *args],
                                       stderr=subprocess.PIPE,
                                       creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))

    def inputs(self):
        return {self.external: {name: hashlib.sha256((self.external / name).read_bytes()).hexdigest()
                                if (self.external / name).exists() else None
                                for name in ("kept", "dirty", "deleted", "new")}}

    def seal(self, external=None):
        return capture(self.repo, self.storage, paths=["."], untracked_allowlist=["new"],
                       external=self.inputs() if external is None else external)

    def restore(self, sealed):
        return materialize(sealed.bundle, self.destination, repo_root=self.repo,
                           expected_input_hash=sealed.input_hash)

    def rewrite(self, sealed, change, *, extra=None):
        with zipfile.ZipFile(sealed.bundle) as archive:
            members = {name: archive.read(name) for name in archive.namelist()}
        manifest = json.loads(members["manifest.json"])
        change(manifest)
        manifest["inputHash"] = digest({key: value for key, value in manifest.items() if key != "inputHash"})
        members["manifest.json"] = canonical_json(manifest)
        if extra:
            members.update(extra)
        output = io.BytesIO()
        with zipfile.ZipFile(output, "w") as archive:
            for name, data in members.items():
                archive.writestr(name, data)
        return self.storage.atomic_write("snapshots/tampered.zip", output.getvalue())

    def test_all_states_reconstruct_without_live_sibling(self):
        sealed = self.seal()
        self.assertEqual(sealed.manifest["schemaVersion"], 2)
        entries = sealed.manifest["external"][0]["entries"]
        self.assertEqual({e["path"]: e["status"] for e in entries},
                         {"kept": "base", "dirty": "modified", "new": "new", "deleted": "deleted"})
        self.assertTrue(all(e["payload"] == e["sha256"] for e in entries))
        for repo in (self.repo, self.external):
            (repo / "dirty").write_bytes(b"contamination")
            (repo / "deleted").write_bytes(b"late")
        # Live sibling HEAD also changes; materialization does not read it.
        self.git(self.external, "add", ".")
        self.git(self.external, "commit", "--quiet", "-m", "later")
        self.restore(sealed)
        for root in (self.destination, self.destination.parent / "zr_vm"):
            self.assertEqual((root / "dirty").read_bytes(), b"changed")
            self.assertEqual((root / "kept").read_bytes(), b"kept")
            self.assertEqual((root / "new").read_bytes(), b"new")
            self.assertFalse((root / "deleted").exists())
        self.assertEqual(verify_materialized(sealed, self.destination), sealed.input_hash)

    def test_hash_covers_external_content_and_planner_drift_fails(self):
        first = self.seal()
        inputs = self.inputs()
        (self.external / "dirty").write_bytes(b"later")
        with self.assertRaises(PilotError):
            self.seal(inputs)
        second = self.seal()
        self.assertNotEqual(first.input_hash, second.input_hash)

    def test_changes_between_capture_scans_fail(self):
        from tools.jenkins.pilot import external_snapshot
        original = external_snapshot.capture
        calls = []
        def changed(sources):
            result = original(sources)
            calls.append(True)
            if len(calls) == 1:
                (self.external / "dirty").write_bytes(b"drift")
            return result
        with patch.object(external_snapshot, "capture", side_effect=changed):
            with self.assertRaises(PilotError):
                self.seal()

    def test_bad_mounts_and_archive_mismatches_fail(self):
        sealed = self.seal()
        for mount in ("../escape", "source", "target", "Source", "a/b", ".git"):
            with self.subTest(mount=mount):
                bundle = self.rewrite(sealed, lambda m: m["external"][0].update(mount=mount))
                with self.assertRaises(PilotError):
                    load_snapshot(bundle)
        bundle = self.rewrite(sealed, lambda m: None, extra={"payloads/extra": b"extra"})
        with self.assertRaises(PilotError):
            load_snapshot(bundle)

    def test_external_head_changes_between_capture_scans_fail(self):
        from tools.jenkins.pilot import external_snapshot
        original = external_snapshot.capture
        calls = []
        def changed(sources):
            result = original(sources)
            calls.append(True)
            if len(calls) == 1:
                self.git(self.external, "commit", "--quiet", "--allow-empty", "-m", "HEAD drift")
            return result
        with patch.object(external_snapshot, "capture", side_effect=changed):
            with self.assertRaises(PilotError):
                self.seal()

    def test_case_colliding_mounts_and_malformed_entries_fail(self):
        sealed = self.seal()
        def duplicate(m):
            sibling = dict(m["external"][0])
            sibling["mount"] = sibling["mount"].upper()
            m["external"].append(sibling)
        with self.assertRaises(PilotError):
            load_snapshot(self.rewrite(sealed, duplicate))
        with self.assertRaises(PilotError):
            load_snapshot(self.rewrite(sealed, lambda m: m["external"][0]["entries"][0].update(status=[])))
        bundle = self.rewrite(sealed, lambda m: m["external"][0]["entries"][0].update(size=10))
        with self.assertRaises(PilotError):
            load_snapshot(bundle)

    def test_case_and_parent_collisions_fail(self):
        sealed = self.seal()
        def duplicate(m):
            entry = dict(m["external"][0]["entries"][0])
            entry["path"] = entry["path"].upper()
            m["external"][0]["entries"].append(entry)
            m["external"][0]["entries"].sort(key=lambda e: e["path"])
        with self.assertRaises(PilotError):
            load_snapshot(self.rewrite(sealed, duplicate))
        def parent(m):
            entries = m["external"][0]["entries"]
            entry = dict(entries[0])
            entry["path"] = entries[0]["path"] + "/nested/file"
            entries.append(entry)
            entries.sort(key=lambda e: e["path"])
        with self.assertRaises(PilotError):
            load_snapshot(self.rewrite(sealed, parent))
        def directory_case(m):
            entries = m["external"][0]["entries"]
            entries[0]["path"] = "Folder/a"
            entries[1]["path"] = "folder/b"
            entries.sort(key=lambda e: e["path"])
        with self.assertRaises(PilotError):
            load_snapshot(self.rewrite(sealed, directory_case))

    def test_external_transfer_limit_is_enforced(self):
        with patch("tools.jenkins.pilot.snapshot.MAX_BUNDLE_BYTES", 1):
            with self.assertRaises(PilotError):
                self.seal()

    def test_extra_tampered_and_late_deleted_sibling_files_fail(self):
        sealed = self.seal()
        self.restore(sealed)
        sibling = self.destination.parent / "zr_vm"
        for name, data in (("unexpected", b"extra"), ("deleted", b"late"), ("dirty", b"tamper")):
            with self.subTest(name=name):
                (sibling / name).write_bytes(data)
                with self.assertRaises(PilotError):
                    verify_materialized(sealed, self.destination)
                if name == "dirty":
                    (sibling / name).write_bytes(b"changed")
                else:
                    (sibling / name).unlink()
        (self.destination.parent / "extra").mkdir()
        with self.assertRaises(PilotError):
            verify_materialized(sealed, self.destination)

    def test_external_private_key_and_hardlink_fail(self):
        (self.external / "dirty").write_bytes(b"-----BEGIN PRIVATE KEY-----\n")
        with self.assertRaises(PilotError):
            self.seal()
        (self.external / "dirty").write_bytes(b"changed")
        (self.external / "linked").hardlink_to(self.external / "dirty")
        with self.assertRaises(PilotError):
            self.seal()

    def test_nonempty_sealed_parent_rejected_before_primary_writes(self):
        sealed = self.seal()
        self.destination.parent.mkdir(parents=True)
        (self.destination.parent / "foreign").write_bytes(b"foreign")
        with self.assertRaises(PilotError):
            self.restore(sealed)
        self.assertFalse(self.destination.exists())


if __name__ == "__main__":
    unittest.main()

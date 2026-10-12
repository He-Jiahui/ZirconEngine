"""One reviewed missing single-file blob; no source copies or native execution."""
import copy
import hashlib
import json
from pathlib import Path
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.tests import test_build_cache_gc as cache_fixture
from tools.jenkins.workflow import artifact_protection as protection
from tools.jenkins.workflow import build_cache_gc as gc


class ArtifactProtectionTests(unittest.TestCase):
    def setUp(self):
        self.fixture = cache_fixture.BuildCacheMaintenanceTests("test_unindexed_cache_cannot_skip_intake")
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.repo, self.state = self.fixture.repo, self.fixture.state
        self.candidate = self.fixture.candidate
        self.old_path = self.fixture.root / "cargo-targets" / "zircon-jenkins" / "artifacts" / "artifacts" / "objects" / protection.BLOB_DIGEST
        self.old_path.parent.mkdir(parents=True)
        self.fixture.stack.enter_context(patch.object(protection, "CANDIDATE", str(self.candidate)))
        self.fixture.stack.enter_context(patch.object(protection, "OLD_PATH", str(self.old_path)))
        self.observer = self.fixture.stack.enter_context(patch.object(protection, "_observe_missing", side_effect=self._observe_missing))
        reference = {"digest": protection.BLOB_DIGEST, "owner": "gc-fixture", "status": "active"}
        obj = {"fixture": protection.FIXTURE, "size": 3800, "status": "ready"}
        receipt = {"operationId": protection.OP_KEY, "buildRoot": r"D:\cargo-targets", "repositoryId": "maintenance-fixture",
                   "sessionId": "gc-" + protection.FIXTURE, "protectedDigests": [protection.BLOB_DIGEST],
                   "removed": [{"digest": "1" * 64, "bytes": 3800}], "reclaimedBytes": 3800}
        inventory = {"operationId": protection.OP_KEY, "buildRoot": r"D:\cargo-targets",
                     "activeDigests": [protection.BLOB_DIGEST], "objectCount": 17}
        self.legacy_rows = [("artifact_ref", protection.REF_KEY, reference), ("artifact_object", protection.BLOB_DIGEST, obj),
                            ("maintenance_gc", protection.OP_KEY, receipt), ("maintenance_inventory", protection.OP_KEY, inventory)]
        pins = []
        for domain, key, value in self.legacy_rows:
            row = self.state.put(domain, key, value)
            pins.append((domain, key, row["version"], digest(value)))
        self.fixture.stack.enter_context(patch.object(protection, "ROW_PINS", tuple(pins)))
        provenance = self.repo / ".jenkins" / "state" / "migration-baseline"; provenance.mkdir(parents=True)
        producer_path, archive_path = provenance / "run_gc_acceptance.py", provenance / "gc-runtime-acceptance.json"
        producer = b"# Isolated single-file blob provenance fixture; never executed.\n"
        producer_path.write_bytes(producer)
        archive = {"operationId": protection.OP_KEY,
                   "identity": {"repositoryId": "maintenance-fixture", "sessionId": "gc-" + protection.FIXTURE,
                                "requestId": "gc-request-" + protection.FIXTURE},
                   "objects": {"active": {"digest": protection.BLOB_DIGEST, "size": 3800, "path": str(self.old_path)}},
                   "receipt": receipt, "accepted": True}
        archive_raw = json.dumps(archive, sort_keys=True).encode(); archive_path.write_bytes(archive_raw)
        self.fixture.stack.enter_context(patch.object(protection, "PRODUCER", (protection.PRODUCER[0], hashlib.sha256(producer).hexdigest())))
        self.fixture.stack.enter_context(patch.object(protection, "ARCHIVE", (protection.ARCHIVE[0], hashlib.sha256(archive_raw).hexdigest())))
        self.review = protection.review_payload(self.repo, "Explicit test user review of exactly one historical blob")
        self.fixture.identity["requestId"] = "protection-request"
        self.fixture.scope["maxBytes"] = 10000
        self.fixture.intent = {**self.fixture.intent, **self.fixture.identity,
                               "cleanupScope": self.fixture.scope, "reviewedProtection": self.review}
        self.state.submit_request(self.fixture.intent)
        self.original = {(domain, key): copy.deepcopy(self.state.get(domain, key)) for domain, key, _ in self.legacy_rows}
        self.original_files = {producer_path: producer_path.read_bytes(), archive_path: archive_path.read_bytes()}

    def _observe_missing(self, approved, old_path):
        path = Path(old_path)
        if any(parent.is_symlink() for parent in (path, *path.parents)):
            raise JenkinsError("build_root_reparse_point", "fixture alias")
        if path.exists():
            raise JenkinsError("artifact_protection_presence_changed", "fixture old blob appeared")
        return {"status": "missing", "volumeIdentity": "fixture-volume", "rootDevicePath": "fixture-device",
                "rootFileId": "f" * 32, "device": 123}

    def _call(self, phase, **changes):
        return self.fixture._call(phase, **changes)

    def _assert_old_unchanged(self):
        for (domain, key), original in self.original.items():
            self.assertEqual(self.state.get(domain, key), original)
        for path, raw in self.original_files.items(): self.assertEqual(path.read_bytes(), raw)

    def test_public_reviewed_intake_restores_protection_and_gc_preserves_old_rows(self):
        intake = self._call("intake")
        self.assertEqual(intake["status"], "accepted")
        self.assertEqual(len(intake["inventoryDigest"]), 64)
        index = self.state.list(protection.DOMAIN)[0]["payload"]
        self.assertEqual(index["status"], "missing-unlocated")
        self.assertTrue(index["protectOnly"])
        self.assertFalse(index["deletionAllowed"])
        self.assertFalse(index["JenkinsAcceptance"])
        self.assertEqual(self._call("inventory")["status"], "accepted")
        self.assertEqual(self._call("gc")["result"]["status"], "complete")
        self.assertFalse(self.old_path.exists())
        self._assert_old_unchanged()

    def test_no_explicit_review_does_not_exclude_unknown_reference(self):
        self.fixture.identity["requestId"] = "unreviewed-request"
        self.fixture.intent = {k: v for k, v in {**self.fixture.intent, **self.fixture.identity}.items() if k != "reviewedProtection"}
        self.state.submit_request(self.fixture.intent)
        with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "cache_ownership_unknown")
        self.assertEqual(self.state.list(protection.DOMAIN), [])

    def test_caller_cannot_invent_or_change_reviewed_provenance(self):
        changed = {**self.review, "blobDigest": "e" * 64}
        with self.assertRaises(JenkinsError) as error: self._call("intake", reviewedProtection=changed)
        self.assertEqual(error.exception.code, "request_payload_mismatch")
        self.assertEqual(self.state.list(protection.DOMAIN), [])

    def test_archive_self_hash_does_not_authorize_different_rows(self):
        self.state.put("artifact_object", protection.BLOB_DIGEST, {"fixture": protection.FIXTURE, "size": 3800, "status": "ready", "physicalPath": str(self.candidate)})
        with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "artifact_protection_row_changed")
        self.assertEqual(self.state.list(protection.DOMAIN), [])

    def test_producer_or_archive_pin_tamper_denies_registration(self):
        producer = Path(self.review["producer"]["path"])
        producer.write_bytes(producer.read_bytes() + b"changed")
        with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "artifact_protection_provenance_changed")
        self.assertEqual(self.state.list(protection.DOMAIN), [])

    def test_missing_producer_is_typed_invalid_provenance(self):
        Path(self.review["producer"]["path"]).unlink()
        with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "artifact_protection_provenance_invalid")
        self.assertEqual(self.state.list(protection.DOMAIN), [])

    def test_whole_build_native_hardlink_gate_is_typed_and_writes_no_intake_rows(self):
        from tools.jenkins.pilot.native.windows_storage import WindowsPathError
        with patch.object(gc, "_file_fact", side_effect=WindowsPathError("worker file has a hard-link alias")):
            with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "cache_alias_rejected")
        for domain in (protection.DOMAIN, "maintenance_build_cache", "maintenance_cache_inventory", "maintenance_cache_gc"):
            self.assertEqual(self.state.list(domain), [])
        self.assertEqual(self.fixture.storage.deleted, [])
        self._assert_old_unchanged()

    def _select_fixed_subtree(self):
        self.review = protection.review_payload(self.repo, "Explicit user review of the fixed incremental subtree",
                                                protection.RELATIVE_SUBTREE)
        return self.fixture._select_fixed_subtree(self.review)

    def test_reviewed_fixed_subtree_intake_inventory_chunks_preserve_old_blob_and_siblings(self):
        selected = self._select_fixed_subtree()
        untouched = {path: path.read_bytes() for path in self.fixture.audit.rglob("*")
                     if path.is_file() and not path.is_relative_to(selected)}
        intake = self._call("intake")
        self.assertEqual(intake["status"], "accepted")
        index = self.state.list(protection.DOMAIN)[0]["payload"]
        self.assertEqual(index["candidatePath"], str(selected))
        stored = self.state.list("maintenance_cache_inventory")[0]["payload"]
        self.assertEqual(stored["candidatePath"], str(selected))
        protection.assert_inventory(stored, selected)
        self.assertEqual(self._call("inventory")["status"], "accepted")
        self.assertEqual(self.fixture._yield_after_one_delete()["status"], "waiting")
        self.assertEqual(self._call("reconcile")["status"], "waiting")
        complete = self._call("gc")
        self.assertEqual(complete["status"], "accepted")
        self.assertEqual(complete["result"]["candidatePath"], str(selected))
        self.assertEqual(complete["result"]["reclaimedBytes"], len(b"selected-a") + len(b"selected-bbb"))
        for path, raw in untouched.items(): self.assertEqual(path.read_bytes(), raw)
        self._assert_old_unchanged()

    def test_fixed_subtree_review_cannot_use_whole_build_or_foreign_candidate(self):
        selected = self._select_fixed_subtree()
        reviews = (protection.review_payload(self.repo, "Explicit but wrong whole build review"),
                   {**self.review, "candidatePath": str(self.candidate / "debug" / "foreign")})
        for number, review in enumerate(reviews):
            with self.subTest(review=review), self.assertRaises(JenkinsError) as error:
                protection.normalize_review(review, self.repo, str(selected))
            self.assertEqual(error.exception.code, "artifact_protection_scope_invalid")
            self.fixture.identity["requestId"] = "wrong-subtree-review-" + str(number)
            self.fixture.intent = {**self.fixture.intent, **self.fixture.identity, "reviewedProtection": review}
            self.state.submit_request(self.fixture.intent)
            with self.assertRaises(JenkinsError) as error: self._call("intake")
            self.assertEqual(error.exception.code, "artifact_protection_scope_invalid")
            self.assertEqual(self.state.list(protection.DOMAIN), [])
        inventory = gc._inventory(self.fixture.storage, selected, {"maxSeconds": 300, **self.fixture.scope})
        inventory["files"][0]["path"] = "build/a.bin"
        inventory["inventoryDigest"] = digest({k: v for k, v in inventory.items() if k != "inventoryDigest"})
        with self.assertRaises(JenkinsError) as error: protection.assert_inventory(inventory, selected)
        self.assertEqual(error.exception.code, "artifact_protection_inventory_invalid")

    def test_fixed_subtree_native_alias_rejects_before_any_protection_or_intake_row(self):
        self._select_fixed_subtree()
        from tools.jenkins.pilot.native.windows_storage import WindowsPathError
        with patch.object(gc, "_file_fact", side_effect=WindowsPathError("worker file has a hard-link alias")):
            with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "cache_alias_rejected")
        for domain in (protection.DOMAIN, "maintenance_build_cache", "maintenance_cache_inventory", "maintenance_cache_gc"):
            self.assertEqual(self.state.list(domain), [])
        self.assertEqual(self.fixture.storage.deleted, [])

    def test_fixed_subtree_protected_content_in_selected_file_denies_registration(self):
        selected = self._select_fixed_subtree()
        (selected / "moved.bin").write_bytes(("active-acceptance-" + protection.FIXTURE + "\n").encode() * 100)
        with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "artifact_protection_content_overlap")
        self.assertEqual(self.state.list(protection.DOMAIN), [])
        self.assertEqual(self.fixture.storage.deleted, [])

    def test_moved_protected_blob_in_candidate_denies_content_exclusion(self):
        raw = ("active-acceptance-" + protection.FIXTURE + "\n").encode() * 100
        (self.candidate / "moved.bin").write_bytes(raw)
        with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "artifact_protection_content_overlap")
        self.assertEqual(self.state.list(protection.DOMAIN), [])
        self.assertEqual(self.fixture.storage.deleted, [])

    def test_hardlink_or_unknown_inventory_shape_cannot_exclude_blob(self):
        inventory = gc._inventory(self.fixture.storage, self.candidate, {"maxSeconds": 300, **self.fixture.scope})
        for changed in ({"linkCount": 2}, {"fileId": None}, {"unexpected": "trusted"}):
            value = copy.deepcopy(inventory)
            value["files"][0].update(changed)
            value["inventoryDigest"] = digest({k: v for k, v in value.items() if k != "inventoryDigest"})
            with self.subTest(changed=changed), self.assertRaises(JenkinsError) as error:
                protection.assert_inventory(value, self.candidate)
            self.assertEqual(error.exception.code, "artifact_protection_inventory_invalid")
        value = {**inventory, "unknownProtection": True}
        value["inventoryDigest"] = digest({k: v for k, v in value.items() if k != "inventoryDigest"})
        with self.assertRaises(JenkinsError) as error: protection.assert_inventory(value, self.candidate)
        self.assertEqual(error.exception.code, "artifact_protection_inventory_invalid")

    def test_physical_alias_or_reappeared_old_blob_denies_exclusion(self):
        self.observer.side_effect = JenkinsError("build_root_reparse_point", "fixture path alias")
        with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "build_root_reparse_point")
        self.observer.side_effect = self._observe_missing
        self.old_path.write_bytes(b"reappeared")
        with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "artifact_protection_presence_changed")
        self.assertEqual(self.state.list(protection.DOMAIN), [])

    def test_additional_unknown_reference_is_not_covered_by_exact_legacy_review(self):
        self.state.put("artifact_ref", "another-active-reference", {"digest": protection.BLOB_DIGEST, "owner": "foreign", "status": "active"})
        with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "cache_ownership_unknown")
        self.assertEqual(self.state.list(protection.DOMAIN), [])

    def test_index_replay_tamper_or_changed_old_ref_denies_next_phase(self):
        self._call("intake")
        index = self.state.list(protection.DOMAIN)[0]
        self.state.put(protection.DOMAIN, index["key"], {**index["payload"], "candidatePath": str(self.fixture.target)})
        with self.assertRaises(JenkinsError) as error: self._call("inventory")
        self.assertEqual(error.exception.code, "artifact_protection_index_missing")
        self.assertEqual(self.fixture.storage.deleted, [])

    def test_changed_legacy_ref_cannot_bypass_review_by_becoming_terminal(self):
        self._call("intake"); self._call("inventory")
        self.state.put("artifact_ref", protection.REF_KEY,
                       {"digest": protection.BLOB_DIGEST, "owner": "gc-fixture", "status": "released"})
        with self.assertRaises(JenkinsError) as error: self._call("gc")
        self.assertEqual(error.exception.code, "artifact_protection_row_changed")
        self.assertEqual(self.fixture.storage.deleted, [])

    def test_relocated_protected_content_between_chunks_denies_resume(self):
        self._call("intake"); self._call("inventory")
        yielded = self.fixture._yield_after_one_delete()
        self.assertEqual(yielded["status"], "waiting")
        raw = ("active-acceptance-" + protection.FIXTURE + "\n").encode() * 100
        (self.candidate / "moved.bin").write_bytes(raw)
        with self.assertRaises(JenkinsError) as error: self._call("gc")
        self.assertEqual(error.exception.code, "cache_inventory_changed")
        self.assertEqual(self.fixture.storage.deleted, ["build/a.bin"])
        self._assert_old_unchanged()

    def test_replacement_with_protected_bytes_during_delete_is_not_unlinked(self):
        self._call("intake"); self._call("inventory")
        raw = ("active-acceptance-" + protection.FIXTURE + "\n").encode() * 100
        fact = self.fixture._fact
        calls = {"number": 0}
        def replace_before_delete(storage, relative, maximum):
            calls["number"] += 1
            if calls["number"] == 3:  # two full inventory reads, then first deletion fact
                (storage.root / relative).write_bytes(raw)
            return fact(storage, relative, maximum)
        with patch.object(gc, "_file_fact", side_effect=replace_before_delete):
            result = self._call("gc")
        self.assertEqual(result["status"], "blocked")
        self.assertEqual(self.fixture.storage.deleted, [])
        self.assertEqual((self.candidate / "a.bin").read_bytes(), raw)


if __name__ == "__main__":
    unittest.main()

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.source import (canonical_path, claim_paths, release_paths,
                                  seal_manifest, apply_owned_patch, seal_candidate,
                                  materialize_manifest, register_sealed_consumer,
                                  persist_cargo_evidence, require_sealed_consumer,
                                  get_sealed_manifest)
from tools.jenkins.source import _thaw
from tools.jenkins.state import State


class SourceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.root.joinpath("src").mkdir()
        self.source = self.root / "src" / "main.rs"
        self.source.write_text("fn main() {}\n", encoding="utf-8")
        self.state = State(self.root / "state.sqlite3")

    def tearDown(self):
        self.temp.cleanup()

    def test_canonical_path_rejects_escape_and_normalizes_case(self):
        self.assertEqual("src/main.rs", canonical_path(self.root, "SRC\\Main.rs"))
        with self.assertRaises(JenkinsError):
            canonical_path(self.root, "../outside.rs")

    def test_ancestor_claim_conflicts_atomically(self):
        first = claim_paths(self.state, self.root, "owner-a", ["src/main.rs"])
        with self.assertRaises(JenkinsError) as error:
            claim_paths(self.state, self.root, "owner-b", ["src"])
        self.assertEqual("path_claim_conflict", error.exception.code)
        self.assertEqual(1, len(self.state.list("path_claims")))
        release_paths(self.state, "owner-a", first)

    def test_before_hash_change_is_rejected(self):
        claims = claim_paths(self.state, self.root, "owner-a", ["src/main.rs"])
        self.source.write_text("changed\n", encoding="utf-8")
        with self.assertRaisesRegex(JenkinsError, "source changed"):
            seal_manifest(self.root, claims)

    def test_manifest_is_stable_and_is_immutable(self):
        claims = claim_paths(self.state, self.root, "owner-a", ["src/main.rs"])
        sealed = seal_manifest(self.root, claims, base_head="abc")
        self.assertEqual("present", sealed.entries[0]["status"])
        self.assertEqual(64, len(sealed.source_digest))
        self.assertEqual(sealed.source_digest, seal_manifest(self.root, claims, base_head="abc").source_digest)
        with self.assertRaises(TypeError):
            sealed.payload["baseHead"] = "changed"

    def test_candidate_rejects_source_changed_between_manifest_and_object_read(self):
        claims = claim_paths(self.state, self.root, "owner-a", ["src/main.rs"])
        store = self.root / "objects"
        changed = b"fn main() { println!(\"changed\"); }\n"
        original_seal_manifest = seal_manifest

        def seal_then_change(*args, **kwargs):
            sealed = original_seal_manifest(*args, **kwargs)
            # Model an edit after the manifest's own stability checks but
            # before seal_candidate copies the object bytes.
            self.source.write_bytes(changed)
            return sealed

        with patch("tools.jenkins.source.seal_manifest", side_effect=seal_then_change):
            with self.assertRaisesRegex(JenkinsError, "source changed"):
                seal_candidate(self.root, claims, object_root=store)
        manifests = list((store / "inputs").glob("*.json")) if (store / "inputs").exists() else []
        self.assertEqual([], manifests)

    def test_owned_patch_records_operation_and_materializes_content_addressed_input(self):
        before = self.source.read_bytes()
        before_hash = __import__("hashlib").sha256(before).hexdigest()
        op = apply_owned_patch(self.state, self.root, "owner-a", {"src/main.rs": "fn main() { println!(\"ok\"); }\n"},
                               {"src/main.rs": before_hash}, operation_id="patch-1")
        self.assertEqual("complete", op["status"])
        claims = claim_paths(self.state, self.root, "owner-a", ["src/main.rs"])
        store = self.root / "objects"
        sealed = seal_candidate(self.root, claims, object_root=store)
        destination = self.root / "materialized"
        materialize_manifest(sealed, store, destination)
        self.assertEqual(self.source.read_bytes(), (destination / "src/main.rs").read_bytes())

    def test_patch_rejects_foreign_before_hash(self):
        with self.assertRaises(JenkinsError) as error:
            apply_owned_patch(self.state, self.root, "owner-a", {"src/main.rs": "changed"},
                              {"src/main.rs": "0" * 64}, operation_id="patch-2")
        self.assertEqual("before_hash_mismatch", error.exception.code)

    def test_materialize_rejects_jenkins_home_object_root_and_bad_object(self):
        claims = claim_paths(self.state, self.root, "owner-a", ["src/main.rs"])
        with self.assertRaises(JenkinsError) as error:
            seal_candidate(self.root, claims, object_root=self.root / ".jenkins")
        self.assertEqual("object_root_rejected", error.exception.code)
        store = self.root / "objects"
        sealed = seal_candidate(self.root, claims, object_root=store,
                                declared_dependencies=["src/main.rs"])
        object_path = store / "inputs" / "objects" / sealed.entries[0]["objectDigest"]
        object_path.write_bytes(b"corrupt")
        with self.assertRaisesRegex(JenkinsError, "corrupt"):
            materialize_manifest(sealed, store, self.root / "materialized-bad")

    def test_materialize_rejects_changed_existing_input(self):
        claims = claim_paths(self.state, self.root, "owner-a", ["src/main.rs"])
        store = self.root / "objects"
        sealed = seal_candidate(self.root, claims, object_root=store)
        destination = self.root / "materialized"
        materialize_manifest(sealed, store, destination)
        (destination / "src/main.rs").write_text("foreign\n", encoding="utf-8")
        with self.assertRaisesRegex(JenkinsError, "differs"):
            materialize_manifest(sealed, store, destination)

    def test_sealed_consumer_is_scoped_and_immutable(self):
        first = register_sealed_consumer(self.state, repository_id_value="repo-a", session_id="s1",
                                         owner="owner-a", source_digest="d" * 64, manifest_digest="d" * 64)
        self.assertEqual(first["payload"]["sessionId"], "s1")
        self.assertIsNotNone(self.state.get("sealed_consumer", first["key"]))
        with self.assertRaisesRegex(JenkinsError, "immutable"):
            register_sealed_consumer(self.state, repository_id_value="repo-a", session_id="s1",
                                     owner="owner-a", source_digest="d" * 64, manifest_digest="e" * 64)
        with self.assertRaisesRegex(JenkinsError, "not registered"):
            require_sealed_consumer(self.state, repository_id_value="repo-a", session_id="s2",
                                    owner="owner-a", source_digest="d" * 64)

    def test_cargo_evidence_must_match_sealed_digest(self):
        claims = claim_paths(self.state, self.root, "owner-a", ["src/main.rs"])
        sealed = seal_candidate(self.root, claims, object_root=self.root / "objects")
        with self.assertRaisesRegex(JenkinsError, "not bound"):
            persist_cargo_evidence(self.state, {"sourceDigest": "0" * 64},
                                   repository_id_value="repo", session_id="s", owner="o", manifest=sealed)

    def test_cargo_evidence_is_shared_without_losing_consumer_binding(self):
        claims = claim_paths(self.state, self.root, "owner-a", ["src/main.rs"])
        sealed = seal_candidate(self.root, claims, object_root=self.root / "objects")
        evidence = {"sourceDigest": sealed.source_digest, "metadataDigest": "1" * 64}
        for session, owner in [("a", "owner-a"), ("b", "owner-b")]:
            register_sealed_consumer(self.state, repository_id_value="repo", session_id=session,
                                     owner=owner, source_digest=sealed.source_digest,
                                     manifest_digest=sealed.source_digest)
            record = persist_cargo_evidence(self.state, evidence, repository_id_value="repo",
                                             session_id=session, owner=owner, manifest=sealed)
            self.assertNotIn("sessionId", record["payload"])
            self.assertNotIn("owner", record["payload"])
        self.assertEqual(1, len(self.state.list("sealed_cargo_evidence")))
        with self.assertRaisesRegex(JenkinsError, "not registered"):
            persist_cargo_evidence(self.state, evidence, repository_id_value="repo",
                                   session_id="foreign", owner="foreign", manifest=sealed)

    def test_formal_object_root_does_not_resolve_alias_before_admission(self):
        from tools.jenkins.source import _approved_object_root
        with patch("tools.jenkins.resources.canonical_build_root",
                   side_effect=JenkinsError("build_root_alias", "aliased root")) as admission:
            with self.assertRaisesRegex(JenkinsError, "aliased"):
                _approved_object_root(r"E:\\nested\\cargo-targets")
            admission.assert_called_once_with(r"E:\\nested\\cargo-targets")

    def test_manifest_lookup_requires_consumer_binding(self):
        claims = claim_paths(self.state, self.root, "owner-a", ["src/main.rs"])
        sealed = seal_candidate(self.root, claims, object_root=self.root / "objects")
        plain = _thaw(sealed.payload)
        self.state.put("sealed_input", sealed.source_digest,
                       {"sourceManifest": plain, "manifest": plain})
        with self.assertRaisesRegex(JenkinsError, "not registered"):
            get_sealed_manifest(self.state, repository_id_value="repo", session_id="s",
                                owner="owner-a", source_digest=sealed.source_digest)
        register_sealed_consumer(self.state, repository_id_value="repo", session_id="s",
                                 owner="owner-a", source_digest=sealed.source_digest,
                                 manifest_digest=sealed.source_digest)
        self.assertEqual(sealed.source_digest,
                         get_sealed_manifest(self.state, repository_id_value="repo", session_id="s",
                                             owner="owner-a", source_digest=sealed.source_digest).source_digest)


if __name__ == "__main__":
    unittest.main()

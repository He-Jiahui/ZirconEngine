"""Synthetic sibling preparation; no real external repository is copied."""
from __future__ import annotations

import json
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, digest, file_digest
from tools.jenkins.source import materialize_manifest, collect_cargo_evidence
from tools.jenkins.source import external_inputs as external
from tools.jenkins import current_source as intake, editor_product as product
from tools.jenkins.tests import test_current_source as intake_fixture
from tools.jenkins.resources.paths import build_root_containing
from tools.jenkins.workflow import pools
from tools.jenkins.workflow.planning import _normalize_metadata


class ExternalInputTests(unittest.TestCase):
    def setUp(self):
        self.fixture = intake_fixture.CurrentSourceTests("test_registration_is_side_effect_free_and_replay_exact")
        self.fixture.setUp()
        self.root, self.state = self.fixture.root, self.fixture.state
        self.addCleanup(self.fixture.doCleanups)
        self.sibling = self.root.parent / "zr_vm"
        self.sibling.mkdir()
        self.path = "zr_vm_rust_binding/rust/zr_vm_rust_binding/cargo.toml"
        target = self.sibling / self.path
        target.parent.mkdir(parents=True)
        target.write_text("[package]\nname = 'synthetic-binding'\n", encoding="utf-8")
        self.data = {"name": "zr_vm", "repositoryRoot": str(self.sibling), "baseHead": "a" * 40,
                     "currentHashes": {self.path: file_digest(target), "deleted.rs": None}, "untrackedPaths": []}
        self.identity = {**self.fixture.identity, "sessionId": "external-session"}
        engine_hashes = dict(self.fixture.hashes)
        for name, data in (("Cargo.toml", b"[workspace]\n"), ("Cargo.lock", b"# synthetic lock\n")):
            (self.root / name).write_bytes(data)
            engine_hashes[name.casefold()] = file_digest(self.root / name)
        self.payload = {**self.fixture.payload, "identity": self.identity, "externalInputs": [self.data]}
        self.payload["currentHashes"] = engine_hashes
        self.state.authorize_session(self.identity["repositoryId"], self.identity["sessionId"], "synthetic-owner",
                                     ["crate", "deps", "cargo.toml", "cargo.lock", "external/zr_vm"], ["validation"],
                                     {"source": "user", "task": "synthetic external test"})
        selection = self.fixture.mocks[1].target._git_selection
        selection.side_effect = lambda root, paths: ({self.path, "deleted.rs"}, set()) if root == self.sibling else (
            {"crate/main.rs", "deps/helper.py", "crate/deleted.rs", "cargo.toml", "cargo.lock"}, {"crate/new.rs"})

    def prepare(self):
        registered = intake.register_current_source(self.state, self.root, self.payload, trusted_driver_digest="b" * 64)
        with patch.object(intake, "_ensure_patch_metadata"):
            prepared = intake.prepare_current_source(self.state, self.root,
                {"identity": self.identity, "currentSourceRef": registered["currentSourceRef"]}, trusted_driver_digest="b" * 64)
        return prepared, self.state.get("sealed_input", prepared["sealedInputRef"])["payload"]

    def test_registration_requires_exact_sibling_and_scoped_authorization(self):
        wrong = {**self.data, "repositoryRoot": str(self.root)}
        with self.assertRaises(JenkinsError):
            intake.register_current_source(self.state, self.root, {**self.payload, "externalInputs": [wrong]}, trusted_driver_digest="b" * 64)
        with self.assertRaises(JenkinsError):
            intake.register_current_source(self.state, self.root, {**self.payload, "identity": self.fixture.identity}, trusted_driver_digest="b" * 64)
        with self.assertRaises(JenkinsError):
            external.normalize_current(self.root, [{**self.data, "ownedPaths": [self.path]}])

    def test_external_hash_drift_blocks_prepare_without_canonical_writes(self):
        ref = intake.register_current_source(self.state, self.root, self.payload, trusted_driver_digest="b" * 64)["currentSourceRef"]
        (self.sibling / self.path).write_text("foreign concurrent edit", encoding="utf-8")
        with self.assertRaises(JenkinsError):
            intake.prepare_current_source(self.state, self.root,
                {"identity": self.identity, "currentSourceRef": ref}, trusted_driver_digest="b" * 64)
        self.assertFalse(self.fixture.cas.exists())
        self.assertEqual((self.sibling / self.path).read_text(encoding="utf-8"), "foreign concurrent edit")

    def test_exact_bundle_materialization_preserves_cargo_sibling_resolution(self):
        prepared, sealed = self.prepare()
        source = self.root.parent / "metadata-bundle/source"
        materialize_manifest(sealed["sourceManifest"], self.fixture.cas, source)
        engine_manifest = source / "zircon_plugins/zr_vm_language/runtime/Cargo.toml"
        resolved = engine_manifest.parent / "../../../../zr_vm" / self.path
        self.assertEqual(resolved.resolve(), (source.parent / "zr_vm" / self.path).resolve())
        self.assertEqual(resolved.read_bytes(), (self.sibling / self.path).read_bytes())
        self.assertFalse((source.parent / "zr_vm/deleted.rs").exists())
        self.assertEqual(self.state.list("path_claims"), [])
        recipe = {"sourceDigest": prepared["sealedInputRef"], "buildRoot": str(build_root_containing(source).path)}
        product._verify_source(recipe, sealed, source)
        (source.parent / "zr_vm/foreign.rs").write_text("foreign", encoding="utf-8")
        with self.assertRaises(JenkinsError):
            product._verify_source(recipe, sealed, source)

    def test_bundle_pool_uses_one_writer_and_rejects_corrupt_external_object(self):
        prepared, sealed = self.prepare()
        source = self.root.parent / "pool/inputs/source"
        eid, key = "synthetic-external-execution", "synthetic-external-preparation"
        self.state.put("execution", eid, {"preparationKey": key, "preparationRoot": str(source),
                       "sealedInputRef": prepared["sealedInputRef"], "reservationKey": "synthetic-reservation"})
        self.state.put("resource_reservation", "synthetic-reservation", {"status": "active", "owner": eid})
        self.state.put("pool_writer_hold", key, {"status": "active", "owner": eid, "reservationKey": "synthetic-reservation"})
        approved = SimpleNamespace(path=Path("D:/cargo-targets"), namespace=lambda: self.fixture.cas)
        with patch.object(pools, "_root", return_value=approved), patch.object(pools, "physical_path_under", side_effect=lambda root, path, **kwargs: Path(path)):
            result = pools.materialize_preparation(self.state, approved, eid)
            self.assertEqual(result["physicalPath"], str(source))
            record = self.state.get("preparation_source", key)["payload"]
            self.assertIn("source/crate/main.rs", record["ownedFiles"])
            self.assertIn("zr_vm/" + self.path, record["ownedFiles"])
            self.assertTrue(pools.materialize_preparation(self.state, approved, eid)["reused"])
            ext_manifest = external.sources(sealed["sourceManifest"])[0]["manifest"]
            obj = next(entry["objectDigest"] for entry in ext_manifest["entries"] if entry["status"] == "present")
            (self.fixture.cas / "inputs/objects" / obj).write_bytes(b"corrupt sibling object")
            with self.assertRaises(JenkinsError):
                pools.materialize_preparation(self.state, approved, eid)

    def test_metadata_paths_allow_only_the_sealed_sibling(self):
        metadata = {"packages": [{"manifest_path": "D:/cargo-targets/test/source/Cargo.toml"},
                                  {"manifest_path": "D:/cargo-targets/test/zr_vm/binding/Cargo.toml"}]}
        normalized = _normalize_metadata(metadata, "D:/cargo-targets/test/source", external_names=["zr_vm"])
        self.assertEqual(normalized["packages"][0]["manifest_path"], "Cargo.toml")
        self.assertEqual(normalized["packages"][1]["manifest_path"], "../zr_vm/binding/Cargo.toml")
        with self.assertRaises(JenkinsError):
            _normalize_metadata(metadata, "D:/cargo-targets/test/source")
        escaped = {"packages": [{"manifest_path": "D:/cargo-targets/test/foreign/Cargo.toml"}]}
        with self.assertRaises(JenkinsError):
            _normalize_metadata(escaped, "D:/cargo-targets/test/source", external_names=["zr_vm"])

    def test_joint_input_materializes_the_same_verified_external_closure(self):
        _, sealed = self.prepare()
        manifest = {**sealed["sourceManifest"], "preparationKind": "joint-input"}
        source = self.root.parent / "joint-metadata-bundle/source"
        materialize_manifest(manifest, self.fixture.cas, source)
        self.assertEqual((source.parent / "zr_vm" / self.path).read_bytes(),
                         (self.sibling / self.path).read_bytes())
        self.assertFalse((source.parent / "zr_vm/deleted.rs").exists())
        self.assertTrue(any(entry["path"].startswith("zr_vm/") for entry in external.flattened(manifest)["entries"]))
        corrupted = {**manifest, "external": [{**manifest["external"][0], "sourceDigest": "e" * 64}]}
        with self.assertRaises(JenkinsError) as raised:
            external.sources(corrupted)
        self.assertEqual("external_manifest_invalid", raised.exception.code)
        entry = next(item for item in external.sources(manifest)[0]["manifest"]["entries"]
                     if item["status"] == "present")
        (self.fixture.cas / "inputs/objects" / entry["objectDigest"]).write_bytes(b"foreign bytes")
        with self.assertRaises(JenkinsError) as raised:
            external.verify_objects(manifest, self.fixture.cas)
        self.assertEqual("external_object_corrupt", raised.exception.code)

    def test_metadata_collector_uses_isolated_bundle_and_exact_external_identity(self):
        prepared, sealed = self.prepare()
        approved = SimpleNamespace(path=Path("D:/cargo-targets"), namespace=lambda: self.fixture.cas)
        outputs = [SimpleNamespace(stdout=json.dumps({"packages": []})),
                   SimpleNamespace(stdout="synthetic cargo"), SimpleNamespace(stdout="synthetic rustc")]
        with patch("tools.jenkins.resources.canonical_build_root", return_value=approved), \
                patch("tools.jenkins.resources.physical_path_under", side_effect=lambda root, path, **kwargs: Path(path)), \
                patch("tools.jenkins.source._approved_object_root", return_value=self.fixture.cas), \
                patch("tools.jenkins.source.subprocess.run", side_effect=outputs) as run:
            evidence = collect_cargo_evidence(sealed["sourceManifest"], self.fixture.cas, "D:/cargo-targets",
                                             cargo="synthetic-cargo", rustc="synthetic-rustc")
        workspace = Path(evidence["workspaceRoot"])
        self.assertEqual(workspace, self.fixture.cas / "inputs/runs" / prepared["sealedInputRef"] / "source")
        self.assertTrue((workspace.parent / "zr_vm" / self.path).is_file())
        self.assertEqual(evidence["externalSourceDigests"], {"zr_vm": external.sources(sealed["sourceManifest"])[0]["sourceDigest"]})
        self.assertTrue(all(Path(call.kwargs["cwd"]) == workspace for call in run.call_args_list))


if __name__ == "__main__":
    unittest.main()

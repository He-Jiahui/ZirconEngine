"""Focused ordinary-management contracts using only tiny synthetic fixtures."""
from __future__ import annotations

import copy
from contextlib import contextmanager
import hashlib
import importlib.util
import json
import os
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest
from unittest.mock import Mock, patch


ROOT = Path(__file__).resolve().parents[3]
MANAGEMENT = ROOT / "tools/jenkins-management"


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


entry = load("ordinary_management_test_entry", MANAGEMENT / "entry.py")
runner = load("ordinary_management_test_runner", MANAGEMENT / "runner.py")
preparation = load("ordinary_management_test_preparation", MANAGEMENT / "source_preparation.py")


class MemoryState:
    def __init__(self):
        self.rows = {}
        self.authorizations = []

    def get(self, domain, key, *, connection=None):
        return copy.deepcopy(self.rows.get((domain, key)))

    def put(self, domain, key, payload, expected_version=None, *, connection=None):
        old = self.rows.get((domain, key))
        version = old["version"] if old else 0
        if expected_version is not None and expected_version != version:
            raise AssertionError("Fixture version conflict")
        value = {"key": key, "payload": copy.deepcopy(payload), "version": version + 1}
        self.rows[(domain, key)] = value
        return copy.deepcopy(value)

    @contextmanager
    def transaction(self):
        yield None

    def list(self, domain, *, connection=None):
        return [copy.deepcopy(value) for (name, _key), value in self.rows.items() if name == domain]

    def check_authorization(self, repository, session, action, paths):
        self.authorizations.append((repository, session, action, tuple(paths)))
        return {"status": "active"}


def snapshot(root, head):
    files = {path.relative_to(root).as_posix(): runner.sha(path.read_bytes())
             for path in root.rglob("*") if path.is_file()}
    value = {"head": head, "files": files, "diffSha256": runner.sha(runner.encoded(files))}
    return {**value, "sourceDigest": runner.sha(runner.encoded(value))}


class OrdinaryFlowTests(unittest.TestCase):
    def test_preparation_identity_shares_producers_and_separates_build_configuration(self):
        args = ["repository", {"cargoSha256": "a" * 64, "rustcSha256": "b" * 64},
                {"PATH": "fixed", "CARGO_BUILD_TARGET": "x86_64-pc-windows-msvc"},
                ["zr_math"], [], "dev", False, {"home": "F:/cache", "configSha256": {}}, ["zr_vm"]]
        identity = preparation.preparation_key(*args)
        # There is deliberately no worktree/session input in the API.
        self.assertEqual(identity, preparation.preparation_key(*copy.deepcopy(args)))
        changed = copy.deepcopy(args)
        changed[6] = True
        self.assertNotEqual(identity, preparation.preparation_key(*changed))
        changed = copy.deepcopy(args)
        changed[1]["rustcSha256"] = "c" * 64
        self.assertNotEqual(identity, preparation.preparation_key(*changed))
        changed = copy.deepcopy(args)
        changed[2]["CARGO_BUILD_TARGET"] = "aarch64-pc-windows-msvc"
        self.assertNotEqual(identity, preparation.preparation_key(*changed))

    def test_shared_pool_updates_only_changed_bytes_and_preserves_sibling_mtimes(self):
        from tools.jenkins.workflow import pools
        with tempfile.TemporaryDirectory(prefix="ordinary-preparation-") as temporary:
            root = Path(temporary)
            namespace = root / "namespace"
            store = namespace / "inputs/objects"
            store.mkdir(parents=True)
            source = namespace / "pool/inputs/source"
            approved = SimpleNamespace(path=root, namespace=lambda: namespace)
            state = MemoryState()
            def entry(name, content):
                checksum = hashlib.sha256(content).hexdigest()
                (store / checksum).write_bytes(content)
                return {"path": name, "status": "present", "sha256": checksum,
                        "objectDigest": checksum, "size": len(content), "beforeHash": checksum}
            external_manifest = {"schemaVersion": 1, "baseHead": "a" * 40,
                                 "entries": [entry("binding/Cargo.toml", b"[package]\nname='binding'\n")],
                                 "external": []}
            manifest = {"schemaVersion": 1, "baseHead": "a" * 40, "repositoryId": "repository",
                        "preparationKind": "current-source",
                        "entries": [entry("crate/lib.rs", b"original\n"), entry("Cargo.toml", b"[workspace]\n")],
                        "external": [{"kind": "sealed-external-source", "name": "zr_vm",
                                      "repositoryRoot": str(root / "zr_vm"), "manifest": external_manifest,
                                      "sourceDigest": preparation.digest(external_manifest)}]}
            def invoke(owner, value):
                sealed_digest = preparation.digest(value)
                state.put("sealed_input", sealed_digest,
                          {"sourceManifest": value, "objectRoot": str(namespace)})
                reservation = owner + "-reservation"
                state.put("resource_reservation", reservation, {"owner": owner, "status": "active"})
                state.put("pool_writer_hold", "shared-preparation",
                          {"owner": owner, "status": "active", "reservationKey": reservation})
                sealed = {"sealedInputRef": sealed_digest, "sourceManifest": value,
                          "sourceRoot": "registered-worktree-" + owner, "capturedSourceDigest": owner}
                return preparation.materialize_source(state, approved=approved, owner=owner,
                    preparation_key_value="shared-preparation", source_path=source,
                    reservation_key=reservation, sealed=sealed, identity={"sessionId": owner},
                    command=["cargo", "test"], environment_digest="e" * 64)
            physical = lambda approved, value, **kwargs: Path(value)
            with patch("tools.jenkins.resources.paths.physical_path_under", side_effect=physical), \
                    patch.object(pools, "physical_path_under", side_effect=physical):
                first = invoke("first", manifest)
                self.assertFalse(first["reused"])
                untouched = source / "Cargo.toml"
                sibling = source.parent / "zr_vm/binding/Cargo.toml"
                fixed_time = 1_700_000_000_000_000_000
                for path in (untouched, sibling):
                    os.utime(path, ns=(fixed_time, fixed_time))
                original_times = (untouched.stat().st_mtime_ns, sibling.stat().st_mtime_ns)
                second = invoke("second", manifest)
                self.assertTrue(second["reused"])
                self.assertEqual(0, second["changedFileCount"])
                updated = copy.deepcopy(manifest)
                updated["entries"][0] = entry("crate/lib.rs", b"changed\n")
                third = invoke("third", updated)
                self.assertTrue(third["incremental"])
                self.assertEqual(1, third["changedFileCount"])
                self.assertEqual(original_times, (untouched.stat().st_mtime_ns, sibling.stat().st_mtime_ns))
                self.assertEqual(b"changed\n", (source / "crate/lib.rs").read_bytes())
                self.assertEqual(3, preparation.verify_source(updated, source, approved)["fileCount"])
                deleted = copy.deepcopy(updated)
                deleted["entries"][0] = {"path": "crate/lib.rs", "status": "deleted", "sha256": None, "size": 0}
                fourth = invoke("fourth", deleted)
                self.assertEqual(1, fourth["changedFileCount"])
                self.assertEqual(2, fourth["preservedFileCount"])
                self.assertFalse((source / "crate").exists())
                self.assertEqual(original_times, (untouched.stat().st_mtime_ns, sibling.stat().st_mtime_ns))
                self.assertEqual(2, preparation.verify_source(deleted, source, approved)["fileCount"])
                # Unexpected files are retained and block acceptance.
                (source / "foreign.rs").write_bytes(b"foreign\n")
                with self.assertRaises(ValueError):
                    preparation.verify_source(deleted, source, approved)
                self.assertTrue((source / "foreign.rs").is_file())

    def test_materialization_rejects_corrupt_objects_and_invalid_closure_manifests(self):
        from tools.jenkins.contracts import JenkinsError
        from tools.jenkins.workflow import pools
        cases = (("engine-object", "input_object_missing"),
                 ("external-object", "external_object_corrupt"),
                 ("external-identity", "external_manifest_invalid"),
                 ("engine-schema", "sealed_manifest_missing"),
                 ("engine-entry", "manifest_entry_invalid"),
                 ("engine-absolute-path", "manifest_path_invalid"),
                 ("engine-empty-path", "manifest_path_invalid"))
        for case, code in cases:
            with self.subTest(case=case), tempfile.TemporaryDirectory(prefix="ordinary-invalid-closure-") as temporary:
                root = Path(temporary)
                namespace = root / "namespace"
                store = namespace / "inputs/objects"
                store.mkdir(parents=True)
                source = namespace / "pool/inputs/source"
                approved = SimpleNamespace(path=root, namespace=lambda: namespace)
                state = MemoryState()
                def entry(path, content):
                    checksum = hashlib.sha256(content).hexdigest()
                    (store / checksum).write_bytes(content)
                    return {"path": path, "status": "present", "sha256": checksum,
                            "objectDigest": checksum, "size": len(content), "beforeHash": checksum}
                external_manifest = {"schemaVersion": 1, "baseHead": "a" * 40,
                                     "entries": [entry("vm.rs", b"dependency\n")], "external": []}
                manifest = {"schemaVersion": 1, "baseHead": "a" * 40, "repositoryId": "repository",
                            "preparationKind": "current-source",
                            "entries": [entry("owned.rs", b"engine\n")],
                            "external": [{"kind": "sealed-external-source", "name": "zr_vm",
                                          "repositoryRoot": str(root / "zr_vm"), "manifest": external_manifest,
                                          "sourceDigest": preparation.digest(external_manifest)}]}
                if case == "engine-object":
                    (store / manifest["entries"][0]["objectDigest"]).write_bytes(b"corrupt engine\n")
                elif case == "external-object":
                    (store / external_manifest["entries"][0]["objectDigest"]).write_bytes(b"corrupt dependency\n")
                elif case == "external-identity":
                    manifest["external"][0]["sourceDigest"] = "0" * 64
                elif case == "engine-schema":
                    manifest["entries"] = tuple(manifest["entries"])
                elif case == "engine-entry":
                    manifest["entries"] = [None]
                elif case == "engine-absolute-path":
                    manifest["entries"][0]["path"] = "/owned.rs"
                elif case == "engine-empty-path":
                    manifest["entries"][0]["path"] = ""
                owner, key = "invalid-" + case, "shared-preparation"
                reservation = owner + "-reservation"
                sealed_digest = preparation.digest(manifest)
                state.put("sealed_input", sealed_digest,
                          {"sourceManifest": manifest, "objectRoot": str(namespace)})
                state.put("resource_reservation", reservation, {"owner": owner, "status": "active"})
                state.put("pool_writer_hold", key,
                          {"owner": owner, "status": "active", "reservationKey": reservation})
                sealed = {"sealedInputRef": sealed_digest, "sourceManifest": manifest,
                          "sourceRoot": "registered-worktree", "capturedSourceDigest": "c" * 64}
                physical = lambda approved, value, **kwargs: Path(value)
                with patch("tools.jenkins.resources.paths.physical_path_under", side_effect=physical), \
                        patch.object(pools, "physical_path_under", side_effect=physical), \
                        self.assertRaises(JenkinsError) as rejected:
                    preparation.materialize_source(state, approved=approved, owner=owner,
                        preparation_key_value=key, source_path=source, reservation_key=reservation,
                        sealed=sealed, identity={"sessionId": owner}, command=["cargo", "check"],
                        environment_digest="e" * 64)
                self.assertEqual(code, rejected.exception.code)
                self.assertIsNone(state.get("preparation_update", key))
                self.assertIsNone(state.get("preparation_source", key))
                self.assertFalse(source.parent.exists())

    def test_sealed_dependency_isolated_from_foreign_edits_and_owned_changes_still_block(self):
        from tools.jenkins import source as formal_source
        from tools.jenkins.workflow import pools
        with tempfile.TemporaryDirectory(prefix="ordinary-sealed-isolation-") as temporary:
            root, state = Path(temporary), MemoryState()
            repo, dependency, namespace = root / "repository", root / "zr_vm", root / "namespace"
            repo.mkdir()
            dependency.mkdir()
            (repo / "owned.rs").write_bytes(b"owned source\n")
            dependency_file = dependency / "vm.rs"
            dependency_file.write_bytes(b"declared dependency\n")
            approved = SimpleNamespace(path=root, namespace=lambda: namespace)
            engine_head, dependency_head = "a" * 40, "b" * 40
            engine, external = snapshot(repo, engine_head), snapshot(dependency, dependency_head)
            cache = {"home": str(namespace / "cargo-home"), "configSha256": {}}
            value = {"sourceView": "worktree", "sourceRoot": str(repo), "source": engine,
                     "externalRoots": [{"path": str(dependency), "source": external}], "dependencyCache": cache}
            before = {**value, "sourceDigest": runner.sha(runner.encoded(value))}
            management = runner.Management.__new__(runner.Management)
            management.repo, management.state = repo, state
            management.payload = {"sourceView": "worktree", "worktreeId": "owned",
                "externalRoots": [str(dependency)], "expectedSourceDigest": before["sourceDigest"]}
            management.wt = lambda name: ({}, repo)
            management.text = lambda path, *args: str(path)
            management.source = lambda path, **kwargs: snapshot(
                path, engine_head if path == repo else dependency_head)
            management.dependency_cache = lambda **kwargs: copy.deepcopy(cache)
            self.assertEqual(before, management.validation_with_cache()[1])
            owner, writer, reservation = "execution", "shared-preparation", "reservation"
            state.put("resource_reservation", reservation, {"owner": owner, "status": "active"})
            state.put("pool_writer_hold", writer,
                      {"owner": owner, "status": "active", "reservationKey": reservation})
            execution_root = namespace / "pool/inputs/source"
            real_seal, order = formal_source.seal_candidate, []
            def seal_with_foreign_edit(path, claims, **kwargs):
                order.append(Path(path))
                if Path(path) == repo:
                    dependency_file.write_bytes(b"foreign edit during engine sealing\n")
                return real_seal(path, claims, **kwargs)
            physical = lambda approved, path, **kwargs: Path(path)
            with patch("tools.jenkins.source._check_object_store_domain"), \
                    patch("tools.jenkins.source._git_base_tree", return_value={}), \
                    patch("tools.jenkins.source.seal_candidate", side_effect=seal_with_foreign_edit), \
                    patch("tools.jenkins.resources.paths.physical_path_under", side_effect=physical), \
                    patch.object(pools, "physical_path_under", side_effect=physical):
                sealed = preparation.seal_source(state, source_root=repo, snapshot=before,
                    repository_root=repo, repository_id="repository", session_id="author", owner=owner,
                    approved=approved, coverage={"packages": ["scoped-package"], "profile": "dev"})
                self.assertEqual([dependency, repo], order)
                self.assertTrue(sealed["capturedInputVerified"])
                prepared = preparation.materialize_source(state, approved=approved, owner=owner,
                    preparation_key_value=writer, source_path=execution_root, reservation_key=reservation,
                    sealed=sealed, identity={"sessionId": "author"}, command=["cargo", "check", "--locked"],
                    environment_digest="e" * 64)
                prepared_before = preparation.verify_source(sealed["sourceManifest"], execution_root, approved)
                _root, after, observed = management.validation_observation(before)
                self.assertEqual(b"declared dependency\n", (execution_root.parent / "zr_vm/vm.rs").read_bytes())
                self.assertFalse(observed["inputsUnchanged"])
                self.assertFalse(observed["externalInputsUnchanged"])
                self.assertTrue(observed["originalEngineUnchanged"])
                self.assertTrue(observed["dependencyCacheUnchanged"])
                self.assertNotEqual(before["sourceDigest"], after["sourceDigest"])
                prepared_after = preparation.verify_source(sealed["sourceManifest"], execution_root, approved)
                proof = {"validationInputPolicy": "sealed-source", "capturedInputVerified": True,
                         "originalEngineUnchanged": observed["originalEngineUnchanged"],
                         "dependencyCacheUnchanged": observed["dependencyCacheUnchanged"],
                         "preparedInputsUnchanged": prepared_before == prepared_after,
                         "inputsUnchanged": before == after, "preparation": prepared,
                         "sourceDigest": before["sourceDigest"]}
                request = {"validationInputPolicy": "sealed-source", "capturedInputVerified": True,
                           "preparedSource": prepared_before, "preparation": prepared,
                           "source": before, "inputObservation": observed}
                self.assertTrue(runner.validation_inputs_verified(proof, request))
                with self.assertRaises(runner.SourceInputMismatch):
                    management.validation_with_cache()
                (repo / "owned.rs").write_bytes(b"changed owned source\n")
                _root, _after, changed_owned = management.validation_observation(before)
                self.assertFalse(changed_owned["originalEngineUnchanged"])
                proof["originalEngineUnchanged"] = changed_owned["originalEngineUnchanged"]
                self.assertFalse(runner.validation_inputs_verified(proof))

    def test_admission_submission_binds_policy_and_preserves_full_pins_and_original_markers(self):
        with tempfile.TemporaryDirectory(prefix="ordinary-admission-entry-") as temporary:
            repo = Path(temporary) / "repository"
            context = {"repo": repo, "manager": Mock()}
            payload = {"action": "validate", "repositoryId": "repository", "sessionId": "author",
                       "requestId": "request", "operationId": "operation", "driverDigest": "d" * 64,
                       "runtimeOperationId": "runtime", "ownedPaths": ["owned.rs"],
                       "sourceView": "worktree", "worktreeId": "candidate",
                       "externalRoots": [str(repo.parent / "zr_vm")],
                       "verb": "check", "packages": ["scoped-package"],
                       "externalCapturePolicy": "jenkins-admission", "expectedEngineSourceDigest": "e" * 64}
            with patch.object(entry, "record", return_value=None), \
                    patch.object(entry, "source_digest", side_effect=AssertionError("No prequeue external capture")):
                result = entry.submit_payload(context, payload, dry_run=True)
            self.assertEqual("prepared-not-submitted", result["status"])
            self.assertEqual(payload, result["payload"])
            self.assertNotIn("expectedSourceDigest", result["payload"])
            self.assertEqual(entry.sha(entry.encoded(payload)), result["inputDigest"])
            changed_intent = {**payload, "expectedEngineSourceDigest": "a" * 64}
            self.assertNotEqual(result["inputDigest"], entry.sha(entry.encoded(changed_intent)))
            pinned = {**payload, "expectedSourceDigest": "f" * 64}
            with patch.object(entry, "record", return_value=None), \
                    patch.object(entry, "source_digest", return_value=("f" * 64, repo, {})) as capture:
                fixed = entry.submit_payload(context, pinned, dry_run=True)
                self.assertEqual(pinned, fixed["payload"])
                capture.assert_called_once_with(context, pinned)
                with self.assertRaises(entry.EntryError):
                    entry.submit_payload(context, {**pinned, "expectedSourceDigest": "b" * 64}, dry_run=True)
            legacy = {name: value for name, value in pinned.items()
                      if name not in {"externalCapturePolicy", "expectedEngineSourceDigest"}}
            original = {"payload": {"payload": legacy, "inputDigest": entry.sha(entry.encoded(legacy)),
                                    "phase": "complete", "result": {"status": "failed"}}}
            with patch.object(entry, "record", return_value=original), \
                    patch.object(entry, "source_digest", side_effect=AssertionError("Original identity is immutable")):
                observed = entry.submit_payload(context, {name: value for name, value in legacy.items()
                                                         if name != "expectedSourceDigest"})
                self.assertEqual("observed", observed["status"])
                self.assertEqual(original["payload"]["inputDigest"], observed["inputDigest"])
                with self.assertRaises(entry.EntryError):
                    entry.submit_payload(context, payload, dry_run=True)
            context["manager"]._request.assert_not_called()
            context["manager"]._request_with_headers.assert_not_called()

    def test_admission_verifies_every_engine_byte_before_external_capture_or_preparation(self):
        validator = runner.Management.__new__(runner.Management)
        base = {"verb": "check", "packages": ["scoped-package"]}
        for fields in ({"externalCapturePolicy": "jenkins-admission"},
                       {"expectedEngineSourceDigest": "e" * 64},
                       {"externalCapturePolicy": "client", "expectedEngineSourceDigest": "e" * 64},
                       {"externalCapturePolicy": "jenkins-admission", "expectedEngineSourceDigest": "a" * 40}):
            with self.subTest(fields=fields):
                validator.payload = {**base, **fields}
                with self.assertRaises(ValueError):
                    validator.validation_recipe()
        with tempfile.TemporaryDirectory(prefix="ordinary-admission-engine-") as temporary:
            root, state = Path(temporary), MemoryState()
            repo, dependency = root / "repository", root / "zr_vm"
            repo.mkdir()
            dependency.mkdir()
            (repo / "owned.rs").write_bytes(b"owned\n")
            sibling = repo / "sibling.rs"
            sibling.write_bytes(b"original sibling\n")
            expected = snapshot(repo, "a" * 40)["sourceDigest"]
            # The entire engine is pinned, including files outside ownedPaths.
            sibling.write_bytes(b"changed sibling at the same HEAD\n")
            actual = snapshot(repo, "a" * 40)
            management = runner.Management.__new__(runner.Management)
            management.repo, management.state, management.key = repo, state, "7" * 64
            management.payload = {**base, "ownedPaths": ["owned.rs"], "sourceView": "worktree",
                                  "worktreeId": "candidate", "externalRoots": [str(dependency)],
                                  "externalCapturePolicy": "jenkins-admission",
                                  "expectedEngineSourceDigest": expected}
            management.wt = lambda name: ({}, repo)
            management.source = Mock(side_effect=lambda path, **kwargs: copy.deepcopy(actual) if path == repo
                                     else self.fail("External bytes must remain unread on engine mismatch"))
            management.dependency_cache = Mock(side_effect=AssertionError("No dependency-cache capture"))
            management.begin_validation_intent = Mock(side_effect=AssertionError("No preparation admission"))
            management.claim_validation_launch = Mock(side_effect=AssertionError("No native launch"))
            with patch.object(runner, "source_helper", side_effect=AssertionError("No source preparation")), \
                    patch("tools.jenkins.resources.capacity.ResourceManager",
                          side_effect=AssertionError("No resource reservation")):
                result = management.validate()
            self.assertEqual("conflict", result["status"])
            self.assertEqual("preparation-conflict", result["reasonCode"])
            self.assertEqual(expected, result["expectedEngineSourceDigest"])
            self.assertEqual(actual["sourceDigest"], result["actualEngineSourceDigest"])
            self.assertFalse(result["preparationStarted"])
            self.assertFalse(result["compileStarted"])
            self.assertEqual({}, state.rows)
            management.source.assert_called_once_with(repo, payload=management.payload)
            management.dependency_cache.assert_not_called()
            management.begin_validation_intent.assert_not_called()
            management.claim_validation_launch.assert_not_called()

    def test_admission_selects_and_seals_the_complete_external_closure_after_engine_capture(self):
        from tools.jenkins import source as formal_source
        from tools.jenkins.workflow import pools
        with tempfile.TemporaryDirectory(prefix="ordinary-admission-closure-") as temporary:
            root, state = Path(temporary), MemoryState()
            repo, dependency, namespace = root / "repository", root / "zr_vm", root / "namespace"
            repo.mkdir()
            (dependency / "docs").mkdir(parents=True)
            (dependency / "tests").mkdir()
            (repo / "owned.rs").write_bytes(b"owned\n")
            (repo / "sibling.rs").write_bytes(b"engine closure\n")
            dependency_file = dependency / "vm.rs"
            dependency_file.write_bytes(b"client dependency\n")
            (dependency / "docs/note.md").write_bytes(b"client documentation\n")
            (dependency / "tests/fixture.rs").write_bytes(b"dependency test fixture\n")
            engine = snapshot(repo, "a" * 40)
            client_external = snapshot(dependency, "b" * 40)
            cache = {"home": str(namespace / "cargo-home"), "configSha256": {}}
            management = runner.Management.__new__(runner.Management)
            management.repo, management.state = repo, state
            management.payload = {"sourceView": "worktree", "worktreeId": "candidate", "ownedPaths": ["owned.rs"],
                                  "externalRoots": [str(dependency)], "externalCapturePolicy": "jenkins-admission",
                                  "expectedEngineSourceDigest": engine["sourceDigest"]}
            management.wt = lambda name: ({}, repo)
            management.text = lambda path, *args: str(path)
            capture_order = []
            def capture(path, **kwargs):
                capture_order.append(Path(path))
                if path == repo:
                    dependency_file.write_bytes(b"Jenkins admission dependency\n")
                    (dependency / "docs/note.md").write_bytes(b"Jenkins admission documentation\n")
                    (dependency / "docs/new.md").write_bytes(b"new complete-closure input\n")
                    return copy.deepcopy(engine)
                return snapshot(path, "b" * 40)
            management.source = capture
            management.dependency_cache = lambda **kwargs: copy.deepcopy(cache)
            captured_root, before = management.validation_with_cache()
            self.assertEqual(repo, captured_root)
            self.assertEqual([repo, dependency], capture_order)
            external = before["externalRoots"][0]["source"]
            self.assertNotEqual(client_external["sourceDigest"], external["sourceDigest"])
            self.assertEqual({"vm.rs", "docs/note.md", "docs/new.md", "tests/fixture.rs"}, set(external["files"]))
            self.assertEqual(engine, before["source"])
            self.assertEqual(runner.sha(runner.encoded({key: value for key, value in before.items()
                                                       if key != "sourceDigest"})), before["sourceDigest"])
            approved = SimpleNamespace(path=root, namespace=lambda: namespace)
            owner, writer, reservation = "execution", "shared-preparation", "reservation"
            state.put("resource_reservation", reservation, {"owner": owner, "status": "active"})
            state.put("pool_writer_hold", writer,
                      {"owner": owner, "status": "active", "reservationKey": reservation})
            execution_root = namespace / "pool/inputs/source"
            real_seal, seal_order = formal_source.seal_candidate, []
            def seal(path, claims, **kwargs):
                seal_order.append(Path(path))
                if Path(path) == repo:
                    dependency_file.write_bytes(b"foreign edit after external sealing\n")
                return real_seal(path, claims, **kwargs)
            physical = lambda approved, path, **kwargs: Path(path)
            with patch("tools.jenkins.source._check_object_store_domain"), \
                    patch("tools.jenkins.source._git_base_tree", return_value={}), \
                    patch("tools.jenkins.source.seal_candidate", side_effect=seal), \
                    patch("tools.jenkins.resources.paths.physical_path_under", side_effect=physical), \
                    patch.object(pools, "physical_path_under", side_effect=physical):
                sealed = preparation.seal_source(state, source_root=repo, snapshot=before,
                    repository_root=repo, repository_id="repository", session_id="author", owner=owner,
                    approved=approved, coverage={"packages": ["scoped-package"], "profile": "dev"})
                self.assertEqual([dependency, repo], seal_order)
                self.assertTrue(sealed["capturedInputVerified"])
                self.assertEqual(before["sourceDigest"], sealed["capturedSourceDigest"])
                sealed_external = sealed["sourceManifest"]["external"][0]["manifest"]
                preparation.verify_sealed_capture(external, sealed_external)
                self.assertEqual(set(external["files"]), {value["path"] for value in sealed_external["entries"]})
                prepared = preparation.materialize_source(state, approved=approved, owner=owner,
                    preparation_key_value=writer, source_path=execution_root, reservation_key=reservation,
                    sealed=sealed, identity={"sessionId": "author"}, command=["cargo", "check", "--locked"],
                    environment_digest="e" * 64)
                verified = preparation.verify_source(sealed["sourceManifest"], execution_root, approved)
            self.assertEqual(b"Jenkins admission dependency\n", (execution_root.parent / "zr_vm/vm.rs").read_bytes())
            self.assertEqual(b"Jenkins admission documentation\n",
                             (execution_root.parent / "zr_vm/docs/note.md").read_bytes())
            self.assertEqual(b"new complete-closure input\n", (execution_root.parent / "zr_vm/docs/new.md").read_bytes())
            self.assertEqual(b"dependency test fixture\n", (execution_root.parent / "zr_vm/tests/fixture.rs").read_bytes())
            self.assertEqual(before["sourceDigest"], prepared["capturedSourceDigest"])
            self.assertEqual(sealed["sealedInputRef"], verified["sourceDigest"])
            self.assertEqual(sealed["sourceManifest"],
                             state.get("sealed_input", sealed["sealedInputRef"])["payload"]["sourceManifest"])

    def dirty_fixture(self, root):
        repo = root / "repository"
        controls = repo / ".jenkins/state/controls"
        controls.mkdir(parents=True)
        base = "a" * 40
        before = {"readme.md": b"base\n", "crate/lib.rs": b"pub const VALUE: u8 = 1;\n"}
        paths = {"a": repo / ".jenkins/worktrees/a", "b": repo / ".jenkins/worktrees/b"}
        records = {}
        for name, path in paths.items():
            for relative, content in before.items():
                target = path / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(content)
            records[name] = {"sessionId": "author-" + name, "status": "ready", "path": str(path),
                             "ownedPaths": list(before)}
        state = MemoryState()
        management = runner.Management.__new__(runner.Management)
        management.repo, management.state, management.repo_id = repo, state, "repository"
        management.session, management.owned = "integrator", list(before)
        management.config = {"driverDigest": "d" * 64, "runtimeOperationId": "runtime"}
        def capture(path):
            value = snapshot(path, base)
            # Git retains this spelling while the formal patch parser folds it.
            value["files"] = {("README.md" if name == "readme.md" else name): checksum
                              for name, checksum in value["files"].items()}
            value["sourceDigest"] = runner.sha(runner.encoded({key: item for key, item in value.items()
                                                             if key != "sourceDigest"}))
            return value
        management.source = capture
        management.wt = lambda name, **kwargs: (records[name], paths[name])
        calls = []
        blobs = {"readme-blob": before["readme.md"], "crate-blob": before["crate/lib.rs"]}
        def git(path, *args, **kwargs):
            calls.append((path, args))
            if args[:2] == ("cat-file", "blob"):
                output = blobs[args[2]]
            elif args and args[0] == "diff":
                output = b"".join(name.encode() + b"\0" for name, content in before.items()
                                 if (path / name).read_bytes() != content)
            elif args and args[0] == "ls-files":
                output = b""
            elif args and args[0] == "add":
                output = b""
            else:
                raise AssertionError("Unexpected Git action: " + repr(args))
            return SimpleNamespace(stdout=output, returncode=0)
        management.git = git
        management.text = lambda path, *args: "c" * 40 if args == ("write-tree",) else base
        def new_worktree(**kwargs):
            self.assertTrue(kwargs["merge"])
            name, path = "candidate", repo / ".jenkins/worktrees/candidate"
            for relative, content in before.items():
                target = path / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(content)
            record = {"sessionId": "integrator", "status": "ready", "path": str(path), "ownedPaths": list(before)}
            paths[name], records[name] = path, record
            state.put("management_worktree", name, record)
            return name, path, record
        management.new_worktree = Mock(side_effect=new_worktree)
        inputs = []
        for name, relative, patch_text, postimage in (
                ("a", "readme.md", "diff --git a/readme.md b/readme.md\n--- a/readme.md\n+++ b/readme.md\n@@ -1 +1,2 @@\n base\n+comment a\n",
                 b"base\ncomment a\n"),
                ("b", "crate/lib.rs", "diff --git a/crate/lib.rs b/crate/lib.rs\n--- a/crate/lib.rs\n+++ b/crate/lib.rs\n@@ -1 +1,2 @@\n pub const VALUE: u8 = 1;\n+// comment b\n",
                 b"pub const VALUE: u8 = 1;\n// comment b\n")):
            patch_path = controls / (name + ".patch")
            patch_path.write_text(patch_text, encoding="utf-8")
            (paths[name] / relative).write_bytes(postimage)
            key = ("1" if name == "a" else "2") * 64
            original = {"repositoryId": "repository", "worktreeId": name, "expectedHead": base,
                        "driverDigest": "d" * 64, "runtimeOperationId": "runtime",
                        "patchReference": {"path": str(patch_path), "sha256": runner.sha(patch_path.read_bytes())},
                        "beforeHashes": {relative: runner.sha(before[relative])}}
            captured = management.source(paths[name])
            result = {"status": "complete", "operationKey": key, "sessionId": "author-" + name,
                      "driverDigest": "d" * 64, "runtimeOperationId": "runtime",
                      "source": captured, "afterHashes": {relative: runner.sha(postimage)}}
            state.put("management_operation", key,
                      {"phase": "complete", "action": "apply-patch", "sessionId": "author-" + name,
                       "payload": original, "result": result})
            inputs.append({"worktreeId": name, "head": base, "sourceDigest": captured["sourceDigest"],
                           "patchOperationKeys": [key]})
        management.payload = {"baseHead": base, "inputs": inputs, "name": "candidate"}
        tree = {"readme.md": ("readme.md", "readme-blob"), "crate/lib.rs": ("crate/lib.rs", "crate-blob")}
        return management, tree, calls

    def test_dirty_two_worktree_merge_uses_completed_author_operations_without_commit(self):
        with tempfile.TemporaryDirectory(prefix="ordinary-merge-") as temporary:
            management, tree, calls = self.dirty_fixture(Path(temporary))
            with patch("tools.jenkins.source._git_base_tree", return_value=tree):
                result = management.prepare_merge()
            self.assertEqual("merge-prepared", result["status"])
            candidate = Path(result["path"])
            self.assertEqual(b"base\ncomment a\n", (candidate / "readme.md").read_bytes())
            self.assertIn(b"// comment b", (candidate / "crate/lib.rs").read_bytes())
            self.assertEqual(2, len(result["inputs"]))
            self.assertFalse(result["commitPerformed"])
            self.assertEqual(result["mainHeadBefore"], result["mainHeadAfter"])
            self.assertTrue(any(args[0] == "add" for _root, args in calls))
            self.assertFalse(any("commit" in args or "push" in args for _root, args in calls))
            authorized_authors = {session for _repo, session, action, _paths in management.state.authorizations
                                  if action == "implementation"}
            self.assertTrue({"author-a", "author-b", "integrator"} <= authorized_authors)

    def test_unfinished_original_patch_blocks_merge_before_candidate_creation(self):
        with tempfile.TemporaryDirectory(prefix="ordinary-merge-pending-") as temporary:
            management, tree, _calls = self.dirty_fixture(Path(temporary))
            row = management.state.get("management_operation", "2" * 64)
            management.state.put("management_operation", "2" * 64, {**row["payload"], "phase": "unknown"})
            with patch("tools.jenkins.source._git_base_tree", return_value=tree):
                with self.assertRaisesRegex(ValueError, "completed patch operation"):
                    management.prepare_merge()
            management.new_worktree.assert_not_called()

    def test_commit_matches_full_cached_snapshot_and_actual_prepared_native_cwd(self):
        self.commit_contract()

    def test_admission_commit_and_delivery_require_the_actual_full_dependency_closure(self):
        self.commit_contract(admission=True)

    def commit_contract(self, *, admission=False):
        with tempfile.TemporaryDirectory(prefix="ordinary-commit-contract-") as temporary:
            root = Path(temporary)
            repo, worktree, namespace = root / "repository", root / "worktree", root / "namespace"
            (repo / ".jenkins/state/controls").mkdir(parents=True)
            worktree.mkdir()
            (worktree / "owned.rs").write_bytes(b"owned change\n")
            dependency = root / "zr_vm"
            dependency.mkdir()
            (dependency / "vm.rs").write_bytes(b"declared dependency\n")
            state = MemoryState()
            key, base = "1" * 64, "a" * 40
            execution_root = namespace / "management/pools/shared/inputs/source"
            execution_root.mkdir(parents=True)
            original = {"repositoryId": "repository", "worktreeId": "owned", "sourceView": "worktree",
                        "buildRoot": str(root), "driverDigest": "d" * 64, "runtimeOperationId": "runtime"}
            engine = snapshot(worktree, base)
            value = {"sourceView": "worktree", "sourceRoot": str(worktree), "source": engine,
                     "externalRoots": [], "dependencyCache": {"home": str(namespace / "cargo-home")}}
            value["externalRoots"] = [{"path": str(dependency), "source": snapshot(dependency, "c" * 40)}]
            before = {**value, "sourceDigest": runner.sha(runner.encoded(value))}
            original.update(externalRoots=[str(dependency)])
            if admission:
                original.update(externalCapturePolicy="jenkins-admission",
                                expectedEngineSourceDigest=engine["sourceDigest"])
            else:
                original.update(expectedSourceDigest=before["sourceDigest"])
            native = {"command": [str(root / "cargo.exe"), "test", "--locked"], "cwd": str(execution_root),
                      "pid": 123, "creationTime": "134000000000000000", "rootExitCode": 0, "terminalErrors": [],
                      "processTree": {"scope": "retained-windows-job", "terminal": True,
                                      "nativeWaiterJoined": True, "pipeEOF": True, "readersFinished": True,
                                      "pipes": {"stdout": {"eof": True}, "stderr": {"eof": True}}},
                      "nativeJobEvidence": {"errors": [], "records": {
                          "afterTerminal": {"activeProcesses": 0}, "afterClose": {"activeProcesses": 0}}}}
            report = namespace / "management/reports" / key
            prepared = {"physicalPath": str(execution_root), "sealedInputRef": "e" * 64,
                        "sourceDigest": "e" * 64, "capturedSourceDigest": before["sourceDigest"]}
            request = {"source": before, "payload": original, "command": native["command"],
                       "executionRoot": str(execution_root), "preparation": prepared,
                       "validationInputPolicy": "sealed-source", "capturedInputVerified": True,
                       "preparedSource": {"sourceDigest": prepared["sealedInputRef"]},
                       "inputObservation": {"originalEngineUnchanged": True, "dependencyCacheUnchanged": True}}
            request_ref = runner.write_json(report / "request.json", request)
            native_ref = runner.write_json(report / "native-terminal.json", native)
            result = {"status": "passed", "sourceDigest": before["sourceDigest"], "inputsUnchanged": False,
                      "validationInputPolicy": "sealed-source", "capturedInputVerified": True,
                      "originalEngineUnchanged": True, "dependencyCacheUnchanged": True,
                      "preparedInputsUnchanged": True, "executionRoot": str(execution_root),
                      "preparation": prepared, "request": request_ref, "nativeTerminal": native_ref}
            state.put("management_operation", key, {"phase": "complete", "action": "validate",
                      "sessionId": "author", "payload": original, "result": result})
            state.put("management_source", key, before)
            record = {"status": "ready", "sessionId": "author"}
            state.put("management_worktree", "owned", record)
            management = runner.Management.__new__(runner.Management)
            management.repo, management.state, management.repo_id = repo, state, "repository"
            management.session, management.owned, management.key = "author", ["owned.rs"], "2" * 64
            management.config = {"driverDigest": "d" * 64, "runtimeOperationId": "runtime"}
            management.payload = {"worktreeId": "owned", "expectedHead": base, "validationKey": key,
                                  "message": "Synthetic contract fixture"}
            management.wt = lambda reference: (record, worktree)
            management.validation_source = Mock(return_value=(worktree, copy.deepcopy(before)))
            management.dependency_cache = Mock(return_value=copy.deepcopy(before["dependencyCache"]))
            management.validation_with_cache = Mock(wraps=lambda **kwargs:
                runner.Management.validation_with_cache(management, **kwargs))
            management.source = lambda path: copy.deepcopy(engine)
            management.git = Mock(side_effect=lambda path, *args, **kwargs:
                SimpleNamespace(stdout=b"owned.rs\0" if args[0] == "diff" else b"", returncode=0))
            management.text = lambda path, *args: ("b" * 40 if path == worktree else base)
            changed = copy.deepcopy(before)
            changed["externalRoots"][0]["source"]["files"]["vm.rs"] = runner.sha(b"foreign input")
            management.validation_source.return_value = (worktree, changed)
            mismatch = ValueError if admission else runner.SourceInputMismatch
            with self.assertRaises(mismatch):
                management.commit_worktree()
            with self.assertRaises(mismatch):
                management.validation_delivery(key, "owned")
            management.git.assert_not_called()
            management.validation_source.return_value = (worktree, copy.deepcopy(before))
            management.validation_with_cache.reset_mock()
            approved = SimpleNamespace(path=root, namespace=lambda: namespace)
            with patch("tools.jenkins.resources.paths.canonical_build_root", return_value=approved), \
                    patch("tools.jenkins.resources.paths.physical_path_under",
                          side_effect=lambda approved, path, **kwargs: Path(path)):
                committed = management.commit_worktree()
            self.assertEqual("complete", committed["status"])
            self.assertEqual(before["sourceDigest"], committed["validatedSourceDigest"])
            self.assertEqual(2, management.validation_with_cache.call_count)
            for call in management.validation_with_cache.call_args_list:
                self.assertEqual(original, call.kwargs["payload"])
            self.assertIn(("repository", "author", "commit", ("owned.rs",)), state.authorizations)


    def test_failed_preparation_releases_only_its_durable_unlaunched_reservation(self):
        from tools.jenkins.resources.capacity import Capacity, ResourceManager
        with tempfile.TemporaryDirectory(prefix="ordinary-unlaunched-") as temporary:
            root, state = Path(temporary), MemoryState()
            management = runner.Management.__new__(runner.Management)
            management.repo, management.state, management.repo_id = root, state, "repository"
            management.session, management.key = "author", "3" * 64
            management.request, management.operation = "request", "operation"
            management.config = {"driverDigest": "d" * 64, "runtimeOperationId": "runtime"}
            resources = ResourceManager(state, Capacity(8, 32 * 1024**3, 64 * 1024**3))
            owner, writer = "management-" + management.key, "shared-writer"
            management.begin_validation_intent(resources, owner, writer, root / "source", root / "report")
            key = owner + ":" + owner
            state.put("resource_reservation", key, {"owner": owner, "status": "active"})
            state.put("pool_writer_hold", writer, {"owner": owner, "status": "active", "reservationKey": key})
            state.put("resource_reservation", "foreign", {"owner": "foreign", "status": "active"})
            state.put("pool_writer_hold", "foreign", {"owner": "foreign", "status": "active",
                                                    "reservationKey": "foreign"})
            from tools.jenkins.contracts import JenkinsError
            failure = JenkinsError("before_hash_mismatch", "source changed after path claim",
                                   details={"path": "vm.rs", "expected": "a" * 64, "observed": "b" * 64})
            management.validate_prepared = Mock(side_effect=failure)
            result = management.validate()
            self.assertEqual("failed", result["status"])
            self.assertEqual("before_hash_mismatch", result["reasonCode"])
            self.assertEqual(failure.details, result["details"])
            self.assertTrue(result["reservationRelease"]["released"])
            self.assertEqual("durable-unlaunched", result["reservationRelease"]["kind"])
            self.assertTrue(state.get("execution", owner)["payload"]["failedBeforeLaunch"])
            self.assertIsNone(state.get("execution_launch", owner))
            self.assertEqual("released", state.get("resource_reservation", key)["payload"]["status"])
            self.assertEqual("released", state.get("pool_writer_hold", writer)["payload"]["status"])
            self.assertEqual("active", state.get("resource_reservation", "foreign")["payload"]["status"])
            self.assertEqual("active", state.get("pool_writer_hold", "foreign")["payload"]["status"])

    def test_validated_combined_delivery_releases_one_input_and_preserves_candidate_and_evidence(self):
        with tempfile.TemporaryDirectory(prefix="ordinary-release-") as temporary:
            root, state = Path(temporary), MemoryState()
            repo = root / "repository"
            controls = repo / ".jenkins/state/controls"
            controls.mkdir(parents=True)
            worktrees = repo / ".jenkins/worktrees"
            original, candidate = worktrees / "input", worktrees / "candidate"
            original.mkdir(parents=True)
            candidate.mkdir()
            (original / "owned.txt").write_bytes(b"base\nowned\n")
            (original / ".git").write_bytes(b"gitdir: retained-metadata\n")
            (candidate / "owned.txt").write_bytes(b"base\nowned\n")
            base = "a" * 40
            source = snapshot(original, base)
            source["files"].pop(".git")
            source["sourceDigest"] = runner.sha(runner.encoded(
                {key: value for key, value in source.items() if key != "sourceDigest"}))
            candidate_source = snapshot(candidate, base)
            patch_path = controls / "input.patch"
            patch_path.write_bytes(b"diff --git a/owned.txt b/owned.txt\n--- a/owned.txt\n+++ b/owned.txt\n"
                                   b"@@ -1 +1,2 @@\n base\n+owned\n")
            patch_key, merge_key, validation_key = "1" * 64, "2" * 64, "4" * 64
            patch_reference = {"path": str(patch_path), "sha256": runner.sha(patch_path.read_bytes())}
            patch_record = {"phase": "complete", "action": "apply-patch", "sessionId": "author",
                            "payload": {"repositoryId": "repository", "worktreeId": "input",
                                "expectedHead": base, "driverDigest": "d" * 64, "runtimeOperationId": "runtime",
                                "patchReference": patch_reference,
                                "beforeHashes": {"owned.txt": runner.sha(b"base\n")}},
                            "result": {"status": "complete", "operationKey": patch_key, "sessionId": "author",
                                "driverDigest": "d" * 64, "runtimeOperationId": "runtime", "source": source}}
            state.put("management_operation", patch_key, patch_record)
            inputs = [{"worktreeId": "input", "sessionId": "author", "head": base,
                       "sourceDigest": source["sourceDigest"], "patchOperationKeys": [patch_key]}]
            merged = {"status": "merge-prepared", "worktreeId": "candidate", "inputs": inputs,
                      "driverDigest": "d" * 64, "runtimeOperationId": "runtime",
                      "integrationKind": "completed-patch-replay", "commitPerformed": False,
                      "pushPerformed": False, "source": candidate_source}
            state.put("management_operation", merge_key, {"phase": "complete", "action": "prepare-merge",
                      "sessionId": "author", "payload": {"inputs": inputs}, "result": merged})
            validation_row = state.put("management_operation", validation_key, {"phase": "complete",
                      "action": "validate", "sessionId": "author", "payload": {"worktreeId": "candidate"},
                      "result": {"status": "passed"}})
            original_record = {"status": "ready", "path": str(original), "sessionId": "author",
                               "ownedPaths": ["owned.txt"], "baseHead": base, "branch": "codex/jenkins/author/input"}
            candidate_record = {"status": "merge-prepared", "path": str(candidate), "sessionId": "author",
                                "ownedPaths": ["owned.txt"], "inputs": inputs}
            state.put("management_worktree", "input", original_record)
            state.put("management_worktree", "candidate", candidate_record)
            state.put("native_job", "foreign", {"owner": "foreign", "status": "unknown"})
            management = runner.Management.__new__(runner.Management)
            management.repo, management.worktrees, management.state, management.repo_id = repo, worktrees, state, "repository"
            management.session, management.owned, management.key = "author", ["owned.txt"], "5" * 64
            management.request, management.operation = "release-request", "release-operation"
            management.config = {"driverDigest": "d" * 64, "runtimeOperationId": "runtime"}
            management.payload = {"worktreeId": "input", "expectedHead": base, "sourceDigest": source["sourceDigest"],
                                  "patchOperationKeys": [patch_key], "mergeOperationKey": merge_key,
                                  "validationKey": validation_key}
            management.wt = lambda name: ((original_record, original) if name == "input"
                                         else (candidate_record, candidate))
            management.source = lambda path: copy.deepcopy(source if path == original else candidate_source)
            management.text = lambda path, *args: base
            management.git = Mock(return_value=SimpleNamespace(stdout=b""))
            management.outputs = lambda: root / "management"
            delivery = {"nativeProofRef": "management-" + validation_key + "-native", "sourceDigest": "s" * 64}
            management.validation_delivery = Mock(return_value=(validation_row, candidate,
                                                               {"source": candidate_source}, delivery))
            git_path = root / "git.exe"
            git_path.write_bytes(b"synthetic pinned Git")
            def native_remove(command, cwd, environment, report, timeout):
                self.assertEqual("releasing", state.get("management_worktree", "input")["payload"]["status"])
                self.assertEqual(["worktree", "remove", "--force", "--", str(original)], command[-5:])
                self.assertEqual(repo, cwd)
                for path in original.iterdir():
                    path.unlink()
                original.rmdir()
                return {"command": command, "cwd": str(cwd), "pid": 123,
                        "creationTime": "134000000000000000", "rootExitCode": 0, "terminalErrors": [],
                        "processTree": {"scope": "retained-windows-job", "terminal": True,
                            "nativeWaiterJoined": True, "pipeEOF": True, "readersFinished": True,
                            "pipes": {"stdout": {"eof": True}, "stderr": {"eof": True}}},
                        "nativeJobEvidence": {"errors": [], "records": {
                            "afterTerminal": {"activeProcesses": 0}, "afterClose": {"activeProcesses": 0}}}}
            helper = SimpleNamespace(free_bytes=Mock(side_effect=[1000, 5096]), _run_owned_command=Mock(side_effect=native_remove))
            spec = SimpleNamespace(loader=Mock())
            with patch("tools.jenkins.workflow.planning._environment_policy",
                       return_value=({"PATH": str(root)}, {}, root / "environment")), \
                    patch("shutil.which", return_value=str(git_path)), \
                    patch.object(runner.importlib.util, "spec_from_file_location", return_value=spec), \
                    patch.object(runner.importlib.util, "module_from_spec", return_value=helper):
                result = management.release_worktree()
            self.assertEqual("released", result["status"])
            self.assertFalse(original.exists())
            self.assertEqual(b"base\nowned\n", (candidate / "owned.txt").read_bytes())
            self.assertTrue(patch_path.is_file())
            self.assertEqual(patch_record, state.get("management_operation", patch_key)["payload"])
            self.assertEqual("released", state.get("management_worktree", "input")["payload"]["status"])
            self.assertEqual("merge-prepared", state.get("management_worktree", "candidate")["payload"]["status"])
            self.assertEqual("unknown", state.get("native_job", "foreign")["payload"]["status"])
            self.assertEqual(2, result["deletedFileCount"])
            self.assertEqual(4096, result["actualVolumeFreeBytesDelta"])
            self.assertFalse(result["commitPerformed"])
            self.assertFalse(result["pushPerformed"])

    def test_declared_source_conflict_retains_recursive_diagnostics_before_preparation(self):
        def source(head, files):
            value = {"head": head, "files": files, "diffSha256": runner.sha(b"")}
            return {**value, "sourceDigest": runner.sha(runner.encoded(value))}
        child = source("b" * 40, {"runtime/lib.rs": "1" * 64})
        external = source("c" * 40, {"vendor/unity": {"gitlinkCommit": "b" * 40, "source": child}})
        view = {"sourceView": "worktree", "sourceRoot": "E:/registered/input",
                "source": source("a" * 40, {"owned.rs": "2" * 64}),
                "externalRoots": [{"path": "E:/Git/zr_vm", "source": external}]}
        cache = {"home": "F:/cargo-targets/zircon-local/cache/cargo-home",
                 "configSha256": {"config": None, "config.toml": None}}
        declared = {**view, "dependencyCache": cache}
        declared["sourceDigest"] = runner.sha(runner.encoded(declared))
        current = copy.deepcopy(view)
        changed_child = source("b" * 40, {"runtime/lib.rs": "3" * 64})
        current["externalRoots"][0]["source"] = source(
            "c" * 40, {"vendor/unity": {"gitlinkCommit": "b" * 40, "source": changed_child}})
        management = runner.Management.__new__(runner.Management)
        management.state, management.key = MemoryState(), "6" * 64
        management.payload = {"verb": "check", "packages": ["zr_math"],
                              "expectedSourceDigest": declared["sourceDigest"]}
        management.validation_source = Mock(return_value=(Path(view["sourceRoot"]), current))
        management.dependency_cache = Mock(return_value=cache)
        management.begin_validation_intent = Mock(side_effect=AssertionError("No admission before capture matches"))
        management.claim_validation_launch = Mock(side_effect=AssertionError("No native launch on input conflict"))
        for admission in (False, True):
            with self.subTest(admission=admission):
                if admission:
                    management.payload.update(externalCapturePolicy="jenkins-admission",
                        expectedEngineSourceDigest=declared["source"]["sourceDigest"])
                with patch.object(runner, "source_helper", side_effect=AssertionError("No source preparation")), \
                        patch("tools.jenkins.resources.capacity.ResourceManager",
                              side_effect=AssertionError("No resource reservation")):
                    result = management.validate()
                self.assertEqual("conflict", result["status"])
                self.assertEqual("preparation-conflict", result["reasonCode"])
                self.assertFalse(result["preparationStarted"])
                self.assertFalse(result["compileStarted"])
                self.assertEqual({}, management.state.rows)
                management.begin_validation_intent.assert_not_called()
                management.claim_validation_launch.assert_not_called()
                self.assertEqual(declared["sourceDigest"], result["expectedSourceDigest"])
                self.assertNotEqual(result["expectedSourceDigest"], result["actualSourceDigest"])
                expected_components = runner.source_components(declared)
                actual_components = result["sourceComponents"]
                self.assertEqual(expected_components["source"], actual_components["source"])
                self.assertEqual(expected_components["dependencyCache"], actual_components["dependencyCache"])
                expected_child = expected_components["externalRoots"][0]["source"]["gitlinks"]["vendor/unity"]["source"]
                actual_child = actual_components["externalRoots"][0]["source"]["gitlinks"]["vendor/unity"]["source"]
                self.assertEqual(expected_child["head"], actual_child["head"])
                self.assertEqual(expected_child["diffSha256"], actual_child["diffSha256"])
                self.assertNotEqual(expected_child["filesDigest"], actual_child["filesDigest"])

if __name__ == "__main__":
    unittest.main()

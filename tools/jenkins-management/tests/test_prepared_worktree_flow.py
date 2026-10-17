"""Prepared-WT Flow keeps narrow ownership and genuinely sealed driver inputs."""
from __future__ import annotations

import copy
from contextlib import ExitStack, contextmanager
import hashlib
import importlib.util
import json
import os
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from tools.jenkins import current_source, managed_worktree, managed_worktree_lifetime
from tools.jenkins.contracts import JenkinsError, digest, file_digest
from tools.jenkins.resources.paths import ApprovedBuildRoot, canonical_build_root
from tools.jenkins.state import State
from tools.jenkins.tests import test_managed_worktree_preparation as baseline
from tools.jenkins.workflow import handler

ROOT = Path(__file__).resolve().parents[3]


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, ROOT / "tools/jenkins-management" / file)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


helper = load("prepared_flow_helper_tests", "source_preparation.py")
runner = load("prepared_flow_runner_tests", "runner.py")


@unittest.skipUnless(os.name == "nt", "Real prepared source pins require Windows")
class PreparedWorktreeFlowTests(unittest.TestCase):
    @contextmanager
    def prepared(self, *, metadata=False):
        f = baseline.ManagedWorktreePreparationTests(methodName="runTest")
        f.setUp()
        try:
            f.state = State(f.repo / ".jenkins/state/coordination.sqlite3")
            f.management.state = f.state
            if metadata:
                locked = {"Cargo.lock": b"version = 3\n"}
                f.before.update(locked)
                f.desired.update(locked)
                f.write(f.repo, locked)
                f.write(f.target, locked)
            absent = ["never-created-one.rs", "never-created-two.rs"]
            present = {"scope/keep-" + str(index).zfill(2) + ".rs":
                       b"" if index == 0 else b"unchanged owned bytes\n"
                       for index in range(21)}
            f.before.update(present)
            f.desired.update(present)
            f.write(f.repo, present)
            f.write(f.target, present)
            # Frozen read inputs may retain absent paths outside the narrow writer.
            dependency_absent = ["dependency/never-created-one.rs", "dependency/never-created-two.rs"]
            f.desired.update({name: None for name in dependency_absent})
            f.tracked = set(f.before)
            owned = sorted(["owned.txt", "deleted.txt", *present, *absent])
            self.assertEqual(25, len(owned))
            writer = f.state.authorize_session(f.repo_id, "w", "user", owned,
                ["implementation", "validation"], {"source": "user", "task": "narrow fixture W25"})

            def payload(action, request, session="w"):
                return {**f.payload(action, request, session), "ownedPaths": owned}

            created = {**payload("create-worktree", "flow-create"), "name": "t", "baseHead": f.base}
            creation_key = digest({name: created[name] for name in
                                   ("repositoryId", "sessionId", "requestId", "operationId")})
            f.record = {**f.record, **{name: created[name] for name in
                ("repositoryId", "sessionId", "requestId", "operationId")},
                "branch": "codex/jenkins/w/t", "ownedPaths": owned,
                "creationOperationKey": creation_key, "writerPrincipal": writer["principal"],
                "writerAuthorizationSnapshot": writer["snapshotDigest"]}
            f.state.put("management_worktree", "w-t", f.record)
            self.assertEqual(creation_key, f.complete(created,
                {**f.record, "worktreeId": "w-t", "source": f.source(f.target)}))
            f.management.owned = owned
            approved = canonical_build_root(r"E:\cargo-targets")
            manager = SimpleNamespace(spec=SimpleNamespace(storage={"buildRoot": r"E:\cargo-targets"}))
            with ExitStack() as stack:
                stack.enter_context(patch.object(ApprovedBuildRoot, "namespace", return_value=f.cas))
                stack.enter_context(patch.object(managed_worktree, "canonical_build_root", return_value=approved))
                stack.enter_context(patch.object(helper, "local_request_principal", return_value="user"))
                inspection_input = {**payload("inspect-worktree", "flow-inspect"),
                    "targetSourceOperationKey": creation_key, "externalRoots": [str(f.external)]}
                f.use(inspection_input)
                inspected = helper.managed_inspect_worktree(f.management, preparation_helper=helper)
                inspected_key = f.complete(inspection_input, inspected)
                preparation_input = {**payload("prepare-worktree", "flow-prepare"),
                    "inspectionOperationKey": inspected_key, "preparationInputRef": inspected["preparationInputRef"]}
                routed, _ = helper.route_managed_context(
                    f.management.config, f.repo, manager, preparation_input, state=f.state)
                f.use(routed)
                with managed_worktree_lifetime.writer_hold(f.management):
                    prepared = helper.managed_prepare_worktree(f.management, preparation_helper=helper)
                preparation_key = f.complete(routed, prepared)
                self.assertEqual("complete", prepared["status"])
                frozen = f.state.get("managed_worktree_preparation", preparation_key)
                self.assertEqual("sealed-observed-canonical-input", frozen["payload"]["sourceKind"])
                for name in [*absent, *dependency_absent]:
                    self.assertIsNone(frozen["payload"]["beforeHashes"][name])
                    self.assertIsNone(frozen["payload"]["currentHashes"][name])
                    self.assertFalse((f.target / name).exists())
                    self.assertNotIn(name, frozen["payload"]["source"]["files"])
                self.assertIsNotNone(frozen["payload"]["beforeHashes"]["deleted.txt"])
                self.assertIsNone(frozen["payload"]["currentHashes"]["deleted.txt"])

                actor = runner.Management.__new__(runner.Management)
                actor.__dict__.update(vars(f.management))
                actor.session, actor.request, actor.operation = "w", "flow-control", "flow-control"
                actor.owned = owned
                actor.payload = {**payload("prepare-flow", "flow-control"),
                    "flowIdentity": {"sessionId": routed["sessionId"], "requestId": "product-flow", "generation": 1},
                    "attemptId": "product-flow-1", "worktreePreparationRef": preparation_key,
                    "patchOperationKeys": [],
                    "coverage": {"template": "editor_product_development",
                                 "requiredTests": ["editor::prepared_input_case"], "noDefaultFeatures": True}}
                actor.source = f.source
                actor.build_root = lambda: r"E:\cargo-targets"

                def git_identity(command, **kwargs):
                    if command[-1] == "--git-common-dir":
                        value = f.repo / ".git"
                    elif command[-1] == "--show-toplevel":
                        value = f.target
                    else:
                        raise AssertionError("Only Git identity observation is stubbed")
                    return SimpleNamespace(returncode=0, stdout=str(value))

                def selection(root, paths):
                    tracked = {name.casefold() for name in f.tracked}
                    actual = {name.casefold() for name, checksum in f.source(root)["files"].items()
                              if checksum is not None}
                    return tracked & set(paths), (actual - tracked) & set(paths)

                def git_inventory(root, *args, **kwargs):
                    if args and args[0] == "ls-files":
                        tracked = {name.casefold() for name in f.tracked}
                        actual = {name.casefold() for name, checksum in f.source(root)["files"].items()
                                  if checksum is not None}
                        names = tracked if "--cached" in args else actual - tracked
                        return SimpleNamespace(returncode=0,
                            stdout=b"".join(name.encode("utf-8") + b"\0" for name in sorted(names)))
                    return f.git(root, *args, **kwargs)
                actor.git = git_inventory

                # Git/deployment observations are fixture boundaries. State, all WT
                # bytes, authority, sealing, typed operations and driver reuse run real.
                stack.enter_context(patch.object(current_source.subprocess, "run", side_effect=git_identity))
                stack.enter_context(patch.object(current_source, "_head", return_value=f.base))
                stack.enter_context(patch.object(current_source, "_git_selection", side_effect=selection))
                stack.enter_context(patch.object(handler, "_trusted_driver", return_value="d" * 64))
                stack.enter_context(patch.object(runner, "source_helper", return_value=helper))
                yield SimpleNamespace(f=f, actor=actor, absent=absent, dependency_absent=dependency_absent,
                    owned=owned, writer=writer,
                    preparation_key=preparation_key, prepared=copy.deepcopy(frozen),
                    inventory=Path(inspected["preparationInputRef"]["path"]),
                    payload=payload)
        finally:
            f.doCleanups()

    def test_real_typed_sealing_and_original_prepared_consumer_preserve_full_w25_and_absences(self):
        from tools.jenkins import source as formal_source
        with self.prepared() as value:
            f, actor = value.f, value.actor
            before_wt, before_checkout = f.source(f.target), f.source(f.repo)
            before_inventory = value.inventory.read_bytes()
            before_writer = f.state.check_authorization(f.repo_id, "w", "implementation")
            stage_proofs = []
            original_exit = helper._ManagedInputStage.__exit__
            def finish_stage(stage, *failure):
                observed = (stage.phase, stage.observed, stage.hashed, stage.byte_count)
                result = original_exit(stage, *failure)
                stage_proofs.append(observed)
                return result
            real_prepare = current_source.prepare_current_source
            with patch.object(helper._ManagedInputStage, "__exit__", finish_stage), \
                    patch.object(formal_source, "seal_candidate", wraps=formal_source.seal_candidate) as producer, \
                    patch.object(current_source, "prepare_current_source", wraps=real_prepare) as consumer, \
                    patch.object(managed_worktree, "preparation", wraps=managed_worktree.preparation) as full_preparation, \
                    patch.object(current_source, "_verify_current",
                        side_effect=AssertionError("The original fresh path cannot accept unchanged absent-owned inputs")):
                result = actor.prepare_flow()
            self.assertEqual(1, producer.call_count)
            self.assertEqual(1, consumer.call_count)
            # Initial authority, final acceptance and the genuine consumer still
            # execute; repeated intermediate inventory/CAS reads must not return.
            self.assertEqual(4, full_preparation.call_count)
            reference = result["currentSourceRef"]
            intent = f.state.get("current_source_intent", reference)["payload"]
            prepared = f.state.get("current_source_preparation", reference)["payload"]["result"]
            typed = f.state.get_operation(prepared["sourcePreparationRef"])
            sealed = f.state.get("sealed_input", prepared["sealedInputRef"])["payload"]
            self.assertEqual("complete", typed["status"])
            self.assertEqual("source_preparation", typed["kind"])
            self.assertEqual(intent, typed["payload"])
            self.assertEqual(prepared["sealedInputRef"], typed["result"]["sourceDigest"])
            self.assertEqual(value.owned, intent["ownedPaths"])
            self.assertEqual(25, len(intent["ownedPaths"]))
            self.assertEqual(value.prepared["payload"]["currentHashes"], intent["currentHashes"])
            expected = intent["currentHashes"]
            present = {name: checksum for name, checksum in expected.items() if checksum is not None}
            byte_count = sum((f.target / name).stat().st_size for name in present)
            self.assertEqual(hashlib.sha256(b"").hexdigest(), present["scope/keep-00.rs"])
            self.assertEqual([("flow-intake-paths", len(expected), 0, 0)],
                             [item for item in stage_proofs if item[0] == "flow-intake-paths"])
            self.assertEqual([("flow-canonical-paths", len(expected), 0, 0)] * 3,
                             [item for item in stage_proofs if item[0] == "flow-canonical-paths"])
            self.assertEqual([("flow-source-bytes", len(expected), len(present), byte_count)] * 3,
                             [item for item in stage_proofs if item[0] == "flow-source-bytes"])
            self.assertEqual(value.absent, sealed["sourceManifest"]["preparedAbsentOwnedPaths"])
            self.assertNotIn("deleted.txt", sealed["sourceManifest"]["preparedAbsentOwnedPaths"])
            for name in value.dependency_absent:
                self.assertNotIn(name, intent["ownedPaths"])
                self.assertNotIn(name, sealed["sourceManifest"]["preparedAbsentOwnedPaths"])
                self.assertIsNone(value.prepared["payload"]["beforeHashes"][name])
                self.assertIsNone(value.prepared["payload"]["currentHashes"][name])
                self.assertNotIn(name, before_writer["paths"])
            for name in [*value.absent, *value.dependency_absent, "deleted.txt"]:
                self.assertIsNone(intent["currentHashes"][name])
                entry = next(item for item in sealed["sourceManifest"]["entries"] if item["path"] == name)
                self.assertEqual("deleted", entry["status"])
                self.assertIsNone(entry["sha256"])
                if name in [*value.absent, *value.dependency_absent]:
                    self.assertIsNone(entry["baseBlob"])
                    self.assertIsNone(entry["beforeHash"])
                    self.assertNotIn("objectDigest", entry)
                else:
                    self.assertIsNotNone(entry["baseBlob"])
            self.assertIn("cargo.toml", intent["currentHashes"])
            self.assertNotIn("Cargo.toml", intent["currentHashes"])
            self.assertEqual(intent["identity"], result["flowPayload"]["identity"])
            request = f.state.get_request(f.repo_id, intent["identity"]["sessionId"], "product-flow")
            self.assertEqual(prepared["sealedInputRef"], request["payload"]["sealedInputRef"])
            self.assertEqual(value.prepared, f.state.get("managed_worktree_preparation", value.preparation_key))
            self.assertEqual(before_inventory, value.inventory.read_bytes())
            self.assertEqual(before_writer, f.state.check_authorization(f.repo_id, "w", "implementation"))
            self.assertEqual(before_wt, f.source(f.target))
            self.assertEqual(before_checkout, f.source(f.repo))

            # The old immutable consumer must still reject both genuine object domains.
            engine_entry = next(item for item in sealed["sourceManifest"]["entries"] if item["status"] == "present")
            external_entry = next(item for item in sealed["sourceManifest"]["external"][0]["manifest"]["entries"]
                                  if item["status"] == "present")
            for entry in (engine_entry, external_entry):
                obj = Path(sealed["objectRoot"]) / "inputs/objects" / entry["objectDigest"]
                original = obj.read_bytes()
                try:
                    obj.write_bytes(b"corrupt real CAS object")
                    with self.assertRaises(JenkinsError):
                        real_prepare(f.state, f.repo, {
                            "identity": intent["identity"], "currentSourceRef": reference,
                            "buildRoot": intent["buildRoot"], "runtimeOperationId": intent["runtimeOperationId"]},
                            trusted_driver_digest="d" * 64)
                finally:
                    obj.write_bytes(original)
            self.assertEqual("complete", f.state.get_operation(prepared["sourcePreparationRef"])["status"])

    def test_real_deleted_changed_or_extra_paths_cannot_be_claimed_as_unchanged_absences(self):
        for boundary in ("tracked-deletion", "present-bytes", "owned-absence-created"):
            with self.subTest(boundary=boundary), self.prepared() as value:
                f, actor = value.f, value.actor
                if boundary == "tracked-deletion":
                    f.tracked.remove("deleted.txt")
                elif boundary == "present-bytes":
                    (f.target / "owned.txt").write_bytes(b"unapproved changed bytes\n")
                else:
                    (f.target / value.absent[0]).write_bytes(b"not absent\n")
                before = f.source(f.target)
                preparations, inputs = f.state.list("current_source_preparation"), f.state.list("sealed_input")
                with self.assertRaises(JenkinsError):
                    actor.prepare_flow()
                self.assertEqual(preparations, f.state.list("current_source_preparation"))
                self.assertEqual(inputs, f.state.list("sealed_input"))
                self.assertEqual(value.prepared, f.state.get("managed_worktree_preparation", value.preparation_key))
                self.assertEqual(before, f.source(f.target))
                self.assertEqual(value.writer["snapshotDigest"],
                    f.state.check_authorization(f.repo_id, "w", "implementation")["snapshotDigest"])
        # Real mutations after the real producer must be observed before a
        # completed typed preparation or request can be accepted.
        from tools.jenkins import source as formal_source
        for boundary in ("external-cas-after-seal", "preparation-version-after-seal",
                         "source-bytes-after-seal", "owned-absence-after-seal", "present-directory-after-seal"):
            with self.subTest(boundary=boundary), self.prepared() as value:
                f, actor = value.f, value.actor
                previous_inputs = f.state.list("sealed_input")
                previous_preparations = f.state.list("current_source_preparation")
                original_producer = formal_source.seal_candidate
                changed_object = []
                def produce_then_change(*args, **kwargs):
                    manifest = original_producer(*args, **kwargs)
                    if boundary == "external-cas-after-seal":
                        entry = next(item for item in value.prepared["payload"]["sourceManifest"]["external"][0]
                                     ["manifest"]["entries"] if item["status"] == "present")
                        obj = Path(value.prepared["payload"]["objectRoot"]) / "inputs/objects" / entry["objectDigest"]
                        changed_object.append((obj, obj.read_bytes()))
                        obj.write_bytes(b"real external CAS corruption after sealing")
                    elif boundary == "preparation-version-after-seal":
                        row = f.state.get("managed_worktree_preparation", value.preparation_key)
                        f.state.put("managed_worktree_preparation", value.preparation_key,
                                    row["payload"], expected_version=row["version"])
                    elif boundary == "source-bytes-after-seal":
                        (f.target / "owned.txt").write_bytes(b"real WT corruption after sealing")
                    elif boundary == "owned-absence-after-seal":
                        (f.target / value.absent[0]).write_bytes(b"real formerly absent WT input")
                    else:
                        (f.target / "owned.txt").unlink()
                        (f.target / "owned.txt").mkdir()
                    return manifest
                try:
                    with patch.object(formal_source, "seal_candidate", side_effect=produce_then_change) as producer:
                        with self.assertRaises(JenkinsError):
                            actor.prepare_flow()
                    self.assertEqual(1, producer.call_count)
                    self.assertEqual(previous_inputs, f.state.list("sealed_input"))
                    self.assertEqual(previous_preparations, f.state.list("current_source_preparation"))
                    self.assertIsNone(f.state.get_request(f.repo_id, actor.payload["flowIdentity"]["sessionId"],
                                                         actor.payload["flowIdentity"]["requestId"]))
                    row = f.state.get("managed_worktree_preparation", value.preparation_key)
                    self.assertEqual(value.prepared["payload"], row["payload"])
                    self.assertEqual(value.prepared["version"] +
                                     (boundary == "preparation-version-after-seal"), row["version"])
                    self.assertEqual(value.writer["snapshotDigest"],
                                     f.state.check_authorization(f.repo_id, "w", "implementation")["snapshotDigest"])
                finally:
                    for obj, original in changed_object:
                        obj.write_bytes(original)

        with self.prepared() as value:
            result = value.actor.prepare_flow()
            intent = value.f.state.get("current_source_intent", result["currentSourceRef"])["payload"]
            forged = copy.deepcopy(intent)
            forged["currentHashes"]["outside-w25-never-created.rs"] = None
            with self.assertRaises(JenkinsError):
                helper._verify_prepared_worktree_flow(value.actor, value.f.repo, forged)
            self.assertNotIn("outside-w25-never-created.rs",
                value.f.state.check_authorization(value.f.repo_id, "w", "implementation")["paths"])
            collision = copy.deepcopy(intent)
            collision["currentHashes"]["OWNED.TXT"] = intent["currentHashes"]["owned.txt"]
            inputs = value.f.state.list("sealed_input")
            preparations = value.f.state.list("current_source_preparation")
            with self.assertRaises(JenkinsError) as observed:
                helper.register_prepared_worktree_flow(value.actor, collision)
            self.assertEqual("current_source_scope_invalid", observed.exception.code)
            self.assertEqual(inputs, value.f.state.list("sealed_input"))
            self.assertEqual(preparations, value.f.state.list("current_source_preparation"))


    def test_frozen_external_metadata_runs_real_collector_and_preserves_its_sealed_binding(self):
        from tools.jenkins import contracts, patches, source as formal_source, state as formal_state
        from tools.jenkins.source import external_inputs
        from tools.jenkins.workflow import planning
        # These exact bytes are the live 0fc implementation, including its frozen
        # external-manifest bug. Canonical fixture imports cannot hide a repair.
        driver = ROOT / ".jenkins/runtime/drivers/0fc997ca92dd1a6501aa31f3bb0c1c5c9841f4a4d65cb69fb45931330af83565"
        for module, relative, expected in (
            (formal_source, "source/__init__.py", "79ffc8f47df8170d5692735eeabebe989f8ae3a8e90dcb7a93e1952f1fa59e00"),
            (external_inputs, "source/external_inputs.py", "b091b921b56a0eb6be069f0b6c7ab75a44138d662275f34077eda384f0bcec60"),
            (patches, "patches.py", "411950f863edd4d90e7a6054a28515a6e68733fca36999cf9b253bad4816eaee"),
            (current_source, "current_source.py", "abfb85864825769e03f4a6a55b86d9290e2c3141dd3431d7982996b6ec2b2382"),
            (planning, "workflow/planning.py", "f27011dac12f9ac1e521f9177e78aaaf7604e1eab2db9e7aed944d3506d2b044"),
            (formal_state, "state/__init__.py", "b409898fb90458ea18501cc11149fbbcffba0c5dea4cf37907230ed85fa6c201"),
            (contracts, "contracts/__init__.py", "9609dbd3f90b09fe91bc36a624994629d14613cd0576e88e81de39c0de0c05f9"),
        ):
            self.assertEqual(expected, file_digest(driver / "tools/jenkins" / relative))
            self.assertEqual(expected, file_digest(module.__file__))
        with self.prepared(metadata=True) as value:
            f, actor = value.f, value.actor
            environment = {"PATH": "fixture", "INCLUDE": "fixture", "LIB": "fixture", "LIBPATH": "fixture"}
            policy = {"schemaVersion": 1, "status": "verified", "environment": environment}
            f.state.put("environment_policy", "default", policy)
            git_run = formal_source.subprocess.run
            original_collect = patches.collect_cargo_evidence
            commands, reproduced = [], []
            def observe(command, **kwargs):
                if command[0] not in {"fixture-cargo.exe", "fixture-rustc.exe"}:
                    return git_run(command, **kwargs)
                commands.append(list(command))
                if command[1] == "metadata":
                    workspace = Path(kwargs["cwd"])
                    stdout = json.dumps({"packages": [], "workspace_members": [], "resolve": None,
                        "workspace_root": str(workspace), "target_directory": str(f.cas / "target"), "version": 1})
                else:
                    stdout = "cargo fixture\n" if command[0] == "fixture-cargo.exe" else "rustc fixture\n"
                return SimpleNamespace(returncode=0, stdout=stdout, stderr="")
            def collect(manifest, *args, **kwargs):
                self.assertIsInstance(manifest, dict)
                source_digest = formal_source.compute_source_digest(manifest)
                intent = f.state.list("current_source_intent")[0]["payload"]
                sealed = f.state.get("sealed_input", source_digest)["payload"]
                # Reproduce the original complete metadata path on the same real
                # sealed SQLite/CAS input before exercising the capsule adapter.
                with patch.object(patches, "collect_cargo_evidence", original_collect):
                    with self.assertRaisesRegex(TypeError, "mappingproxy"):
                        patches._ensure_patch_metadata(f.state, f.repo, intent, sealed, policy_root=f.repo)
                self.assertEqual([], commands)
                self.assertIsNone(f.state.get("sealed_cargo_evidence", source_digest))
                reproduced.append(source_digest)
                return original_collect(manifest, *args, **kwargs)
            with patch.object(patches, "_verified_metadata_inputs",
                    return_value=(environment, policy, "fixture-cargo.exe", "fixture-rustc.exe")), \
                    patch.object(formal_source.subprocess, "run", side_effect=observe), \
                    patch.object(patches, "collect_cargo_evidence", side_effect=collect) as collector:
                result = actor.prepare_flow()
                self.assertEqual(1, collector.call_count)
            prepared = f.state.get("current_source_preparation", result["currentSourceRef"])["payload"]["result"]
            source_digest = prepared["sealedInputRef"]
            self.assertEqual([source_digest], reproduced)
            self.assertEqual(["metadata", "--version", "--version"], [command[1] for command in commands])
            evidence = f.state.get("sealed_cargo_evidence", source_digest)["payload"]
            sealed = f.state.get("sealed_input", source_digest)["payload"]
            self.assertEqual(source_digest, evidence["sourceDigest"])
            self.assertEqual(source_digest, evidence["manifestDigest"])
            self.assertEqual(digest(policy), evidence["environmentPolicyDigest"])
            self.assertEqual({"zr_vm": sealed["sourceManifest"]["external"][0]["sourceDigest"]},
                             evidence["externalSourceDigests"])
            workspace = Path(evidence["workspaceRoot"])
            self.assertEqual("source", workspace.name)
            self.assertEqual(file_digest(f.target / "Cargo.lock"), file_digest(workspace / "Cargo.lock"))
            self.assertEqual(file_digest(f.external / "src/lib.rs"),
                             file_digest(workspace.parent / "zr_vm/src/lib.rs"))
            self.assertEqual("complete", f.state.get_operation(prepared["sourcePreparationRef"])["status"])
            request = f.state.get_request(f.repo_id, result["flowPayload"]["identity"]["sessionId"], "product-flow")
            self.assertEqual(source_digest, request["payload"]["sealedInputRef"])
            intent = f.state.get("current_source_intent", result["currentSourceRef"])["payload"]
            self.assertEqual(value.owned, intent["ownedPaths"])
            self.assertEqual(25, len(intent["ownedPaths"]))
            previous = f.state.get("sealed_cargo_evidence", source_digest)
            with patch.object(patches, "_verified_metadata_inputs",
                    return_value=(environment, policy, "fixture-cargo.exe", "fixture-rustc.exe")), \
                    patch.object(patches, "collect_cargo_evidence",
                        side_effect=AssertionError("Existing evidence must retain original validation without collection")):
                helper._ensure_prepared_flow_metadata(f.state, f.repo, intent, sealed)
            self.assertEqual(previous, f.state.get("sealed_cargo_evidence", source_digest))


if __name__ == "__main__":
    unittest.main()

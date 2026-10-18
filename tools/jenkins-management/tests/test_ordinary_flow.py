"""Focused ordinary-management contracts using only tiny synthetic fixtures."""
from __future__ import annotations

import ast
import copy
from contextlib import contextmanager
import hashlib
import importlib.util
import io
import json
import os
import re
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

    def check_authorization(self, repository, session, action, paths=None, *, connection=None):
        self.authorizations.append((repository, session, action, tuple(paths or [])))
        return {"status": "active", "principal": "synthetic-owner", "snapshotDigest": "e" * 64,
                "paths": list(paths or []), "actions": ["implementation", "validation"]}


def snapshot(root, head):
    files = {path.relative_to(root).as_posix(): runner.sha(path.read_bytes())
             for path in root.rglob("*") if path.is_file()}
    value = {"head": head, "files": files, "diffSha256": runner.sha(runner.encoded(files))}
    return {**value, "sourceDigest": runner.sha(runner.encoded(value))}


class PreSubmitCaptureDiagnosticsTests(unittest.TestCase):
    ROOT = Path("E:/ordinary-capture-fixture/repository")

    def _source(self, files, head="a" * 40):
        value = {"sourceEvidencePolicy": "full-file-index-v2", "head": head, "files": dict(files),
                 "gitIndex": {"entries": [{"path": name, "mode": "100644", "object": "c" * 40, "stage": 0}
                                          for name in sorted(files)], "metadataSha256": "d" * 64}}
        return {**value, "sourceDigest": runner.sha(runner.encoded(value))}

    def _snapshot(self, source, *, external=None, cache=None):
        value = {"sourceView": "checkout", "sourceRoot": str(self.ROOT), "source": source,
                 "externalRoots": external or [], "dependencyCache": cache or {"home": "E:/cargo-targets/cache"}}
        return {**value, "sourceDigest": runner.sha(runner.encoded(value))}

    def _rejected(self, before, after, *, input_changes=runner.input_changes, stderr=None, after_root=None):
        captures = Mock(side_effect=[(self.ROOT, before), (after_root or self.ROOT, after)])
        verify = Mock()
        actor = type("CapturedManagement", (), {"validation_with_cache": captures, "verify_declared_inputs": verify})
        module = SimpleNamespace(Management=actor, input_changes=input_changes, source_components=Mock())
        context = {"repo": self.ROOT, "manager": Mock(), "config": {}, "module": module}
        payload = {"action": "validate", "repositoryId": "repository", "sessionId": "author",
                   "requestId": "request", "operationId": "operation", "driverDigest": "d" * 64,
                   "runtimeOperationId": "runtime", "ownedPaths": ["owned.rs"], "sourceView": "checkout",
                   "sourceEvidencePolicy": "full-file-index-v2",
                   "verb": "test", "noRun": True, "packages": ["zircon_app"]}
        original = copy.deepcopy(payload)
        marker, stdout = Mock(), io.StringIO()
        marker.exists.return_value = False
        stderr = stderr if stderr is not None else io.StringIO()
        with patch.object(entry, "plain", return_value=marker), \
                patch.object(entry, "record", return_value=None), \
                patch.object(entry, "document", return_value=(payload, entry.encoded(payload))), \
                patch.object(entry, "dispatch", side_effect=lambda *a, **k: entry.submit_payload(context, payload)), \
                patch.object(entry.sys, "stdout", stdout), patch.object(entry.sys, "stderr", stderr):
            status = entry.main(["submit", "--repo-root", str(self.ROOT), "--input-file", "original.json",
                                 "--session-id", "author", "--request-id", "request", "--operation-id", "operation",
                                 "--action", "validate"])
        self.assertEqual(2, status)
        self.assertEqual({"status": "pending", "reasonCode": "ordinary_entry_unproven",
                          "message": "Source changed during pre-submit capture",
                          "formalAcceptance": False, "m8Accepted": False}, json.loads(stdout.getvalue()))
        self.assertEqual(2, captures.call_count)
        verify.assert_not_called()
        module.source_components.assert_not_called()
        marker.parent.mkdir.assert_not_called()
        marker.open.assert_not_called()
        context["manager"]._request.assert_not_called()
        context["manager"]._request_with_headers.assert_not_called()
        self.assertEqual(original, payload)
        return stderr.getvalue()

    def _details(self, output):
        prefix = "[ordinary-entry] pre-submit capture drift "
        self.assertEqual(1, len(output.splitlines()))
        self.assertTrue(output.startswith(prefix))
        self.assertLessEqual(len(output.encode("utf-8")), 32768)
        return json.loads(output[len(prefix):])

    def test_changed_file_bytes_are_visible_with_stable_head_index_and_membership(self):
        old_bytes, new_bytes = b"old\n", b"new\n"
        self.assertEqual(len(old_bytes), len(new_bytes))
        before = self._snapshot(self._source({"same-size.rs": runner.sha(old_bytes)}))
        after = self._snapshot(self._source({"same-size.rs": runner.sha(new_bytes)}))
        details = self._details(self._rejected(before, after))
        root = details["roots"][0]
        for field in ("head", "gitIndexDigest", "gitIndexMetadataSha256", "membershipDigest", "fileCount"):
            self.assertEqual(root["before"][field], root["after"][field])
        self.assertNotEqual(root["before"]["filesDigest"], root["after"]["filesDigest"])
        self.assertNotEqual(details["sourceDigestBefore"], details["sourceDigestAfter"])
        changed = root["fileChanges"]["values"]
        self.assertEqual(1, len(changed))
        self.assertEqual("same-size.rs", changed[0]["path"]["path"])
        self.assertEqual(runner.sha(old_bytes), changed[0]["before"])
        self.assertEqual(runner.sha(new_bytes), changed[0]["after"])

    def test_external_heads_membership_and_cache_changes_remain_bounded_and_redacted(self):
        source = self._source({"owned.rs": "1" * 64})
        names = ["long/" + "\u754c" * 260 + f"/{index:02}.rs" for index in range(40)]
        old_external = [{"path": f"E:/external/{index:02}", "source": self._source(dict.fromkeys(names, "2" * 64), "b" * 40)}
                        for index in range(12)]
        new_external = [{"path": value["path"], "source": self._source(
            {**dict.fromkeys(names, "3" * 64), "added.rs": None}, "f" * 40)} for value in old_external]
        before = self._snapshot(source, external=old_external, cache={"secret": "do-not-print-before"})
        after = self._snapshot(source, external=new_external, cache={"secret": "do-not-print-after"})
        output = self._rejected(before, after)
        details = self._details(output)
        self.assertNotIn("do-not-print", output)
        self.assertNotIn("secret", output)
        self.assertEqual(13, details["rootCount"])
        self.assertEqual(12, details["changedRootCount"])
        self.assertTrue(details["rootsTruncated"])
        self.assertTrue(details["outputTruncated"])
        self.assertLessEqual(len(details["roots"]), 8)
        root = details["roots"][0]
        self.assertEqual("E:/external/00", root["root"]["path"])
        self.assertEqual("b" * 40, root["before"]["head"])
        self.assertEqual("f" * 40, root["after"]["head"])
        self.assertEqual(1, root["addedPathCount"])
        self.assertEqual("added.rs", root["addedPaths"][0]["path"])
        self.assertNotEqual(root["before"]["membershipDigest"], root["after"]["membershipDigest"])
        self.assertNotEqual(details["dependencyCacheBeforeSha256"], details["dependencyCacheAfterSha256"])
        self.assertTrue(root["fileChanges"]["truncated"])
        self.assertLessEqual(len(root["fileChanges"]["values"]), 16)
        self.assertTrue(root["fileChanges"]["values"][0]["path"]["truncated"])

    def test_diagnostic_failures_and_malformed_capture_preserve_original_rejection(self):
        before = self._snapshot(self._source({"owned.rs": "1" * 64}))
        after = self._snapshot(self._source({"owned.rs": "2" * 64}))
        for observed, helper in ((None, runner.input_changes),
                                 (after, Mock(side_effect=ValueError("do-not-print-exception")))):
            with self.subTest(malformed=observed is None):
                output = self._rejected(before, observed, input_changes=helper)
                self.assertEqual("[ordinary-entry] pre-submit capture drift diagnostics unavailable\n", output)
        broken_stderr = Mock(write=Mock(side_effect=OSError("closed diagnostic stream")))
        broken_stderr.getvalue.return_value = ""
        self._rejected(before, after, stderr=broken_stderr)


class OrdinaryFlowTests(unittest.TestCase):
    def test_public_management_actions_match_runner_and_fixed_job(self):
        template = (MANAGEMENT / "zircon-management.groovy").read_text(encoding="utf-8")
        declaration = re.search(r"(?m)^def allowed = (\[[^\r\n]*\])$", template)
        self.assertIsNotNone(declaration, "The fixed job must declare its bounded action list")
        actions = ast.literal_eval(declaration.group(1))
        expected = {"create-branch", "create-worktree", "prepare-merge", "apply-patch",
                    "commit-worktree", "release-worktree", "recover-worktree",
                    "validate", "generate", "prepare-flow", "submit-flow",
                    "prepare-worktree", "inspect-worktree", "retire-worktree-reader", "preview-current-editor-r34"}
        managed_expected = expected - {entry.PREVIEW_ACTION}
        self.assertEqual(len(actions), len(set(actions)), "The job action list has duplicates")
        self.assertSetEqual(expected, set(actions))
        self.assertSetEqual(set(entry.ACTIONS), set(actions))
        self.assertSetEqual(set(runner.ACTIONS), set(actions))
        wrapper = (ROOT / "tools/dev/submit-jenkins-management.ps1").read_text(encoding="utf-8")
        ps_actions = re.search(r"\[ValidateSet\(([^)]*)\)\]\[string\]\$Action", wrapper)
        self.assertIsNotNone(ps_actions)
        ps_values = re.findall(r"'([^']+)'", ps_actions.group(1))
        public_expected = expected | {"commit-checkout"}
        self.assertEqual(len(public_expected), len(ps_values))
        self.assertSetEqual(public_expected, set(ps_values))
        self.assertRegex(wrapper, r"if \(\$Action -eq 'commit-checkout'\) \{\s*"
                                 r"\$taskEntry = Join-Path \$taskRepo 'tools\\dev\\jenkins_commit\.py'\s*\}")
        self.assertIn("if ($Action -ne 'commit-checkout') { $taskArguments += @('--action', $Action) }", wrapper)
        checkout_addon = (MANAGEMENT / "checkout-commit.groovy").read_text(encoding="utf-8")
        self.assertRegex(checkout_addon, r"(?m)^if \(params\.ACTION == 'commit-checkout'\) \{")
        self.assertIn("tools/jenkins-management/checkout_commit.py", checkout_addon)
        self.assertRegex(checkout_addon, r"(?m)^    return$")
        self.assertEqual(entry.MANAGED_WORKTREE_CAPABILITY, runner.MANAGED_WORKTREE_CAPABILITY)
        self.assertEqual(sorted(managed_expected), entry.MANAGED_WORKTREE_CAPABILITY["actions"])
        installer = (ROOT / "tools/dev/install-jenkins-management.ps1").read_text(encoding="utf-8")
        managed = re.search(r"function Get-ManagedWorktreeCapability \{(.*?)\n\}", installer, re.S)
        self.assertIsNotNone(managed)
        declared = re.search(r"actions=@\((.*?)\);", managed.group(1), re.S)
        self.assertIsNotNone(declared)
        self.assertEqual(sorted(managed_expected), re.findall(r"'([^']+)'", declared.group(1)))

        self.assertLess(template.index("if (!(params.ACTION in allowed))"),
                        template.index("def input = parseJson(params.INPUT_JSON)"))

        launch = re.findall(r"withEnv\((\[[^\r\n]*\])\)\s*\{\s*status = bat", template)
        self.assertEqual(len(launch), 1, "Runner launch requires one explicit environment binding")
        bindings = ast.literal_eval(launch[0])
        environment = dict(value.split("=", 1) for value in bindings)
        self.assertEqual(len(bindings), len(environment), "Duplicate runner environment bindings")
        self.assertEqual(environment["INPUT_JSON"], "")
        self.assertEqual(environment["XDG_CONFIG_HOME"], str(ROOT / ".jenkins").replace("\\", "/"))

    @contextmanager
    def _sealed_reuse_fixture(self):
        with tempfile.TemporaryDirectory(prefix="ordinary-sealed-reuse-") as temporary:
            root, state = Path(temporary), MemoryState()
            repo, dependency, namespace = root / "repository", root / "zr_vm", root / "namespace"
            repo.mkdir()
            dependency.mkdir()
            (repo / "owned.rs").write_bytes(b"owned source\n")
            (repo / "sibling.rs").write_bytes(b"complete engine closure\n")
            (dependency / "vm.rs").write_bytes(b"complete dependency closure\n")
            engine = snapshot(repo, "a" * 40)
            engine["files"]["deleted.rs"] = None
            engine_value = {name: value for name, value in engine.items() if name != "sourceDigest"}
            engine["sourceDigest"] = preparation.digest(engine_value)
            value = {"sourceView": "worktree", "sourceRoot": str(repo), "source": engine,
                     "externalRoots": [{"path": str(dependency), "source": snapshot(dependency, "b" * 40)}],
                     "dependencyCache": {"home": str(namespace / "cargo-home"), "configSha256": {}}}
            before = {**value, "sourceDigest": preparation.digest(value)}
            approved = SimpleNamespace(path=root, namespace=lambda: namespace)
            producer_key, producer_session = "a" * 64, "producer-session"
            producer = "management-" + producer_key
            arguments = {"source_root": repo, "snapshot": before, "repository_root": repo,
                         "repository_id": "repository", "session_id": producer_session, "owner": producer,
                         "approved": approved, "coverage": {"packages": ["first-package"], "profile": "dev"}}
            physical = lambda approved, path, **kwargs: Path(path)
            with patch("tools.jenkins.source._check_object_store_domain"), \
                    patch("tools.jenkins.source._git_base_tree", return_value={}), \
                    patch("tools.jenkins.resources.paths.canonical_build_root", side_effect=lambda value:
                          SimpleNamespace(path=Path(value), namespace=lambda: Path(value) / "namespace")), \
                    patch("tools.jenkins.resources.paths.physical_path_under", side_effect=physical), \
                    patch("tools.jenkins.workflow.pools.physical_path_under", side_effect=physical):
                first = preparation.seal_source(state, **arguments)
                state.put("management_source", producer_key, before, expected_version=0)
                state.put("execution", producer, {"kind": "management_owned_cargo", "status": "running",
                    "repositoryId": "repository", "sessionId": producer_session, "sourceRoot": str(repo),
                    "capturedSourceDigest": before["sourceDigest"], "sealedInputRef": first["sealedInputRef"],
                    "sourceDigest": first["sealedInputRef"]}, expected_version=0)
                yield {"state": state, "arguments": arguments, "first": first, "repo": repo,
                       "dependency": dependency, "namespace": namespace, "producer": producer,
                       "producerKey": producer_key}

    def test_same_capture_reuses_sealed_bytes_for_new_consumers_and_coverage_without_changing_history(self):
        from tools.jenkins import source as formal_source
        with self._sealed_reuse_fixture() as fixture:
            state, arguments, first = fixture["state"], fixture["arguments"], fixture["first"]
            history = copy.deepcopy(state.rows)
            store = fixture["namespace"] / "inputs/objects"
            objects = {path.name: path.read_bytes() for path in store.iterdir()}
            with patch.object(formal_source, "seal_candidate",
                              side_effect=AssertionError("Same capture must reuse its verified CAS")):
                same = preparation.seal_source(state, **{**arguments,
                    "session_id": "consumer-session", "owner": "management-" + "b" * 64})
                changed = preparation.seal_source(state, **{**arguments,
                    "session_id": "consumer-session", "owner": "management-" + "c" * 64,
                    "coverage": {"packages": ["second-package"], "profile": "test"}})
            self.assertFalse(first["metrics"]["reused"])
            self.assertIsNone(first["metrics"]["reusedSealedInputRef"])
            self.assertEqual(first["sealedInputRef"], same["sealedInputRef"])
            self.assertNotEqual(first["sealedInputRef"], changed["sealedInputRef"])
            for result in (same, changed):
                self.assertTrue(result["metrics"]["reused"])
                self.assertEqual(first["sealedInputRef"], result["metrics"]["reusedSealedInputRef"])
                self.assertTrue(result["capturedInputVerified"])
                self.assertEqual(arguments["snapshot"]["sourceDigest"], result["capturedSourceDigest"])
                preparation.verify_sealed_capture(arguments["snapshot"]["source"], result["sourceManifest"])
                preparation.verify_sealed_capture(arguments["snapshot"]["externalRoots"][0]["source"],
                                                  result["sourceManifest"]["external"][0]["manifest"])
            for key, record in history.items():
                self.assertEqual(record, state.rows[key])
            self.assertEqual("running", state.get("execution", fixture["producer"])["payload"]["status"])
            self.assertEqual(1, len(state.list("execution")))
            self.assertEqual(2, len(state.list("sealed_input")))
            self.assertEqual(objects, {path.name: path.read_bytes() for path in store.iterdir()})
            for owner, result in (("management-" + "b" * 64, same), ("management-" + "c" * 64, changed)):
                consumer = formal_source.require_sealed_consumer(state, repository_id_value="repository",
                    session_id="consumer-session", owner=owner, source_digest=result["sealedInputRef"],
                    manifest_digest=result["sealedInputRef"])
                self.assertEqual(owner, consumer["payload"]["owner"])
            changed_record = state.get("sealed_input", changed["sealedInputRef"])["payload"]
            self.assertEqual({"packages": ["second-package"], "profile": "test"}, changed_record["coverage"])
            self.assertEqual(preparation.digest(changed_record["coverage"]), changed_record["coverageDigest"])
            self.assertEqual(changed_record["coverageDigest"],
                             changed_record["sourceManifest"]["validationCoverageDigest"])
            self.assertEqual([], state.list("acceptance"))

    def test_changed_complete_capture_uses_original_external_first_sealing(self):
        from tools.jenkins import source as formal_source
        for case in ("engine-bytes", "external-bytes", "capture-metadata", "approved-root"):
            with self.subTest(case=case), self._sealed_reuse_fixture() as fixture:
                state, arguments = fixture["state"], fixture["arguments"]
                history = copy.deepcopy(state.rows)
                before = copy.deepcopy(arguments["snapshot"])
                if case == "engine-bytes":
                    (fixture["repo"] / "owned.rs").write_bytes(b"changed engine\n")
                    before["source"] = snapshot(fixture["repo"], "a" * 40)
                    before["source"]["files"]["deleted.rs"] = None
                elif case == "external-bytes":
                    (fixture["dependency"] / "vm.rs").write_bytes(b"changed dependency\n")
                    before["externalRoots"][0]["source"] = snapshot(fixture["dependency"], "b" * 40)
                elif case == "approved-root":
                    selected_root = arguments["approved"].path / "second-build-root"
                    selected_namespace = selected_root / "namespace"
                    arguments = {**arguments, "approved": SimpleNamespace(
                        path=selected_root, namespace=lambda: selected_namespace)}
                else:
                    # The byte inventory stays identical, but the full Git
                    # capture is a different input and cannot use old provenance.
                    before["source"]["diffSha256"] = "d" * 64
                engine_value = {name: value for name, value in before["source"].items() if name != "sourceDigest"}
                before["source"]["sourceDigest"] = preparation.digest(engine_value)
                value = {name: item for name, item in before.items() if name != "sourceDigest"}
                before["sourceDigest"] = preparation.digest(value)
                real_seal, order = formal_source.seal_candidate, []
                def seal(path, claims, **kwargs):
                    order.append(Path(path))
                    return real_seal(path, claims, **kwargs)
                with patch.object(formal_source, "seal_candidate", side_effect=seal):
                    result = preparation.seal_source(state, **{**arguments, "snapshot": before,
                        "session_id": "consumer-session", "owner": "management-" + "b" * 64})
                self.assertEqual([fixture["dependency"], fixture["repo"]], order)
                self.assertFalse(result["metrics"]["reused"])
                self.assertIsNone(result["metrics"]["reusedSealedInputRef"])
                self.assertEqual(before["sourceDigest"], result["capturedSourceDigest"])
                if case == "approved-root":
                    self.assertEqual(fixture["arguments"]["snapshot"], before)
                    self.assertEqual(fixture["arguments"]["coverage"], arguments["coverage"])
                    self.assertNotEqual(fixture["first"]["sealedInputRef"], result["sealedInputRef"])
                    record = state.get("sealed_input", result["sealedInputRef"])["payload"]
                    self.assertEqual(str(selected_root), record["buildRoot"])
                    self.assertEqual(str(selected_namespace), record["objectRoot"])
                    self.assertEqual(str(selected_namespace), record["sourceManifest"]["selectedObjectNamespace"])
                    state.put("management_source", "b" * 64, before, expected_version=0)
                    state.put("execution", "management-" + "b" * 64, {
                        "kind": "management_owned_cargo", "status": "preparing", "repositoryId": "repository",
                        "sessionId": "consumer-session", "sourceRoot": str(fixture["repo"]),
                        "capturedSourceDigest": before["sourceDigest"], "sealedInputRef": result["sealedInputRef"],
                        "sourceDigest": result["sealedInputRef"]}, expected_version=0)
                    with patch.object(formal_source, "seal_candidate",
                                      side_effect=AssertionError("Selected namespace must remain reusable")):
                        reused = preparation.seal_source(state, **{**arguments, "snapshot": before,
                            "session_id": "next-session", "owner": "management-" + "c" * 64})
                    self.assertTrue(reused["metrics"]["reused"])
                    self.assertEqual(result["sealedInputRef"], reused["sealedInputRef"])
                preparation.verify_sealed_capture(before["source"], result["sourceManifest"])
                preparation.verify_sealed_capture(before["externalRoots"][0]["source"],
                                                  result["sourceManifest"]["external"][0]["manifest"])
                for key, record in history.items():
                    self.assertEqual(record, state.rows[key])

    def test_corrupt_matching_sealed_input_or_producer_rejects_before_new_metadata(self):
        from tools.jenkins import source as formal_source
        from tools.jenkins.contracts import JenkinsError
        for case in ("engine-object", "external-object", "manifest", "producer-binding",
                     "producer-capture", "object-namespace", "manifest-namespace", "foreign-collision"):
            with self.subTest(case=case), self._sealed_reuse_fixture() as fixture:
                state, arguments, first = fixture["state"], fixture["arguments"], fixture["first"]
                sealed_record = state.rows[("sealed_input", first["sealedInputRef"])]["payload"]
                manifest = sealed_record["sourceManifest"]
                if case in {"engine-object", "external-object"}:
                    entries = (manifest["entries"] if case == "engine-object"
                               else manifest["external"][0]["manifest"]["entries"])
                    entry = next(item for item in entries if item["status"] == "present")
                    (fixture["namespace"] / "inputs/objects" / entry["objectDigest"]).write_bytes(b"corrupt\n")
                elif case == "manifest":
                    manifest["validationCoverageDigest"] = "e" * 64
                elif case == "manifest-namespace":
                    manifest["selectedObjectNamespace"] = str(fixture["namespace"] / "wrong")
                    reference = preparation.digest(manifest)
                    row = state.rows.pop(("sealed_input", first["sealedInputRef"]))
                    row["key"], row["payload"]["sourceDigest"] = reference, reference
                    state.rows[("sealed_input", reference)] = row
                    producer = state.rows[("execution", fixture["producer"])]["payload"]
                    producer["sourceDigest"] = producer["sealedInputRef"] = reference
                    consumer_key = next(key for key in state.rows if key[0] == "sealed_consumer")
                    state.rows.pop(consumer_key)
                    formal_source.register_sealed_consumer(state, repository_id_value="repository",
                        session_id=arguments["session_id"], owner=fixture["producer"],
                        source_digest=reference, manifest_digest=reference)
                elif case == "foreign-collision":
                    manifest["validationCoverageDigest"] = "e" * 64
                    selected_root = arguments["approved"].path / "second-build-root"
                    arguments = {**arguments, "approved": SimpleNamespace(
                        path=selected_root, namespace=lambda: selected_root / "namespace")}
                elif case == "producer-binding":
                    consumer_key = next(key for key in state.rows if key[0] == "sealed_consumer")
                    state.rows.pop(consumer_key)
                elif case == "producer-capture":
                    state.rows[("management_source", fixture["producerKey"])]["payload"]["dependencyCache"]["home"] += "-changed"
                else:
                    sealed_record["objectRoot"] = str(fixture["namespace"].parent / "foreign-namespace")
                damaged = copy.deepcopy(state.rows)
                seal_patch = (patch.object(formal_source, "seal_candidate", wraps=formal_source.seal_candidate)
                              if case == "foreign-collision" else patch.object(formal_source, "seal_candidate",
                                  side_effect=AssertionError("Corrupt matching input must not be resealed")))
                with seal_patch as seal, self.assertRaises((ValueError, JenkinsError)):
                    preparation.seal_source(state, **{**arguments, "session_id": "consumer-session",
                        "owner": "management-" + "b" * 64,
                        "coverage": (arguments["coverage"] if case == "foreign-collision"
                                     else {"packages": ["second-package"], "profile": "test"})})
                if case == "foreign-collision":
                    self.assertEqual([fixture["dependency"], fixture["repo"]],
                                     [Path(call.args[0]) for call in seal.call_args_list])
                self.assertEqual(damaged, state.rows)

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
                # A case-only external rename changes identity, never its physical bytes.
                case_only = copy.deepcopy(deleted)
                case_external = case_only["external"][0]
                case_external["manifest"]["entries"][0]["path"] = "binding/CARGO.toml"
                case_external["sourceDigest"] = preparation.digest(case_external["manifest"])
                sibling_identity = (sibling.stat().st_dev, sibling.stat().st_ino)
                with patch.object(pools, "_atomic_copy", side_effect=AssertionError("Unchanged bytes were copied")), \
                        patch.object(Path, "unlink", side_effect=AssertionError("Case-only input was deleted")):
                    fifth = invoke("fifth", case_only)
                self.assertEqual((fifth["changedFileCount"], fifth["preservedFileCount"]), (0, 2))
                self.assertEqual(original_times, (untouched.stat().st_mtime_ns, sibling.stat().st_mtime_ns))
                self.assertEqual(sibling_identity, (sibling.stat().st_dev, sibling.stat().st_ino))
                self.assertEqual(2, preparation.verify_source(case_only, source, approved)["fileCount"])
                old = state.get("preparation_source", "shared-preparation")["payload"]
                self.assertIn("zr_vm/binding/CARGO.toml", old["ownedFiles"])
                self.assertNotIn("zr_vm/binding/Cargo.toml", old["ownedFiles"])
                ambiguous = {**old["ownedFiles"],
                             "zr_vm/binding/Cargo.toml": old["ownedFiles"]["zr_vm/binding/CARGO.toml"]}
                for before, after in ((ambiguous, {}), ({}, ambiguous)):
                    with self.assertRaisesRegex(RuntimeError, "identities collide"):
                        preparation._pool_update_files(before, after)

                # Recreate an old partial journal: both case entries are false,
                # and a different copy finished before its false bit was checkpointed.
                from tools.jenkins.source.external_inputs import flattened
                legacy_manifest = copy.deepcopy(case_only)
                copied_bytes = b"[workspace]\n# copy completed before checkpoint\n"
                legacy_manifest["entries"][1] = entry("Cargo.toml", copied_bytes)
                external = legacy_manifest["external"][0]
                external["manifest"]["entries"][0]["path"] = "binding/Cargo.toml"
                external["sourceDigest"] = preparation.digest(external["manifest"])
                normalized, desired = pools._manifest(flattened(legacy_manifest), namespace, approved)
                legacy_files = [{"path": name, "oldHash": old["ownedFiles"].get(name),
                                 "newHash": desired.get(name),
                                 "applied": old["ownedFiles"].get(name) == desired.get(name)}
                                for name in sorted(set(old["ownedFiles"]) | set(desired))]
                legacy = {"executionId": "interrupted-case-update", "preparationKey": "shared-preparation",
                          "sourceDigest": preparation.digest(legacy_manifest),
                          "oldSourceDigest": old["sourceDigest"], "status": "updating",
                          "files": legacy_files, "oldOwnedFiles": old["ownedFiles"], "ownedFiles": desired,
                          "manifest": normalized, "manifestDigest": preparation.digest(normalized),
                          "objectRoot": str(namespace), "createdAt": 1234.0}
                prior = state.get("preparation_update", "shared-preparation")
                original = state.put("preparation_update", "shared-preparation", legacy,
                                     expected_version=prior["version"])
                untouched.write_bytes(b"neither old nor new\n")
                with self.assertRaisesRegex(RuntimeError, "unrecognised hash"):
                    preparation._recover_pool(state, "shared-preparation", source.parent, namespace, approved)
                self.assertEqual(original, state.get("preparation_update", "shared-preparation"))
                untouched.write_bytes(copied_bytes)
                with patch.object(pools, "_atomic_copy", side_effect=AssertionError("Checkpoint lag caused a second copy")), \
                        patch.object(Path, "unlink", side_effect=AssertionError("Recovery deleted a case alias")):
                    sixth = invoke("sixth", legacy_manifest)
                self.assertTrue(sixth["reused"])
                journal = state.get("preparation_update", "shared-preparation")["payload"]
                self.assertEqual(journal["status"], "complete")
                self.assertEqual(journal["ownedFiles"], desired)
                self.assertEqual(journal["manifestDigest"], legacy["manifestDigest"])
                evidence = journal["fileIdentityRecovery"]
                self.assertEqual(evidence["originalVersion"], original["version"])
                self.assertEqual(evidence["originalPayloadDigest"], preparation.digest(legacy))
                self.assertEqual(evidence["originalFilesDigest"], preparation.digest(legacy_files))
                self.assertEqual(evidence["caseFoldGroupCount"], 1)
                recovered_alias = next(item for item in journal["files"]
                                       if item["path"] == "zr_vm/binding/Cargo.toml")
                self.assertEqual(recovered_alias["legacyEntries"],
                    [{"index": index, "entry": item} for index, item in enumerate(legacy_files)
                     if item["path"].casefold() == recovered_alias["path"].casefold()])
                self.assertTrue(all(item["applied"] for item in journal["files"]))
                self.assertTrue(all(not item["entry"]["applied"] for item in recovered_alias["legacyEntries"]))
                self.assertEqual(sibling_identity, (sibling.stat().st_dev, sibling.stat().st_ino))
                self.assertEqual(original_times[1], sibling.stat().st_mtime_ns)
                self.assertEqual(copied_bytes, untouched.read_bytes())
                self.assertEqual(2, preparation.verify_source(legacy_manifest, source, approved)["fileCount"])
                # Unexpected files are retained and block acceptance.
                (source / "foreign.rs").write_bytes(b"foreign\n")
                with self.assertRaises(ValueError):
                    preparation.verify_source(legacy_manifest, source, approved)
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

            # Actual checkout keeps complete observed control bytes, while the
            # existing formal external gate sees only compiler-file membership.
            from tools.jenkins import managed_worktree as formal
            from tools.jenkins.contracts import JenkinsError
            plan = ".codex/plans/contract.md"
            ordinary = {"Cargo.toml": "1" * 64, "Cargo.lock": "2" * 64,
                        ".cargo/config.toml": "3" * 64, "build.rs": "4" * 64,
                        "path-dependency/src/lib.rs": "5" * 64, plan: "6" * 64}
            value = {"sourceEvidencePolicy": "full-file-index-v2", "head": "b" * 40,
                     "files": ordinary, "gitIndex": {
                         "entries": [{"path": name, "mode": "100644", "object": "c" * 40, "stage": 0}
                                     for name in sorted(ordinary)], "metadataSha256": "d" * 64}}
            children, stage = {}, []
            for ordinal in range(1, 8):
                name, head = "vendor/dep" + str(ordinal), str(ordinal) * 40
                (dependency / name).mkdir(parents=True)
                child = {"head": head, "files": {"lib.rs": "7" * 64}, "diffSha256": "8" * 64}
                child["sourceDigest"] = preparation.digest(child)
                children[name] = child
                value["files"][name] = {"gitlinkCommit": head, "source": child}
                value["gitIndex"]["entries"].append(
                    {"path": name, "mode": "160000", "object": head, "stage": 0})
                stage.append(("160000 " + head + " 0\t" + name).encode() + b"\0")
            value["gitIndex"]["entries"].sort(key=lambda item: (item["path"], item["stage"]))
            observed = {**value, "sourceDigest": preparation.digest(value)}
            management.payload = {"action": "validate", "sourceView": "checkout",
                                  "sourceEvidencePolicy": "full-file-index-v2",
                                  "externalRoots": [str(dependency)]}
            management.source = lambda path, **kwargs: copy.deepcopy(
                engine if path == repo else observed)
            def git(path, *arguments):
                if arguments == ("ls-files", "--stage", "-z"):
                    return SimpleNamespace(stdout=b"".join(stage))
                self.assertEqual(("ls-files", "--cached", "-z"), arguments)
                names = observed["files"] if path == dependency else children[path.relative_to(dependency).as_posix()]["files"]
                return SimpleNamespace(stdout=b"".join(name.encode() + b"\0" for name in names))
            management.git = git
            management.text = lambda path, *arguments: (
                str(path) if arguments == ("rev-parse", "--show-toplevel")
                else children[path.relative_to(dependency).as_posix()]["head"])
            raw = copy.deepcopy(observed)
            with self.assertRaises(JenkinsError) as rejected:
                formal._external_capture(management, preparation, inspection=True)
            self.assertEqual("external_control_forbidden", rejected.exception.code)
            inputs, captures, gitlinks = formal._external_capture(
                management, preparation.checkout_external_file_view(), inspection=True)
            self.assertEqual(7, len(gitlinks))
            self.assertEqual(
                {name.casefold(): checksum for name, checksum in preparation.source_files(raw).items()
                 if name != plan}, inputs[0]["currentHashes"])
            self.assertEqual(raw, captures[0]["source"])
            self.assertIn(plan, preparation.source_files(captures[0]["source"]))
            with patch.object(runner, "source_helper", return_value=preparation):
                _, captured = management.validation_source()
            self.assertEqual(raw, captured["externalRoots"][0]["source"])
            changed = copy.deepcopy(raw)
            changed["files"][plan] = "9" * 64
            changed["sourceDigest"] = preparation.digest({key: item for key, item in changed.items()
                                                         if key != "sourceDigest"})
            self.assertNotEqual(raw["sourceDigest"], changed["sourceDigest"])
            self.assertEqual(preparation.checkout_external_file_view().source_files(raw),
                             preparation.checkout_external_file_view().source_files(changed))
            for forbidden in (".codex/config.toml", ".codex/hooks/run.py",
                              ".jenkins/payload.json", ".opencode/config.json", "target/output"):
                with self.subTest(forbidden=forbidden):
                    observed = copy.deepcopy(raw)
                    observed["files"][forbidden] = "a" * 64
                    observed["sourceDigest"] = preparation.digest(
                        {key: item for key, item in observed.items() if key != "sourceDigest"})
                    with self.assertRaises(JenkinsError) as rejected:
                        formal._external_capture(
                            management, preparation.checkout_external_file_view(), inspection=True)
                    self.assertEqual("external_control_forbidden", rejected.exception.code)
            for admission in ("untracked", "executable", "unmerged"):
                with self.subTest(planAdmission=admission):
                    observed = copy.deepcopy(raw)
                    entries = observed["gitIndex"]["entries"]
                    if admission == "untracked":
                        entries[:] = [item for item in entries if item["path"] != plan]
                    else:
                        entry = next(item for item in entries if item["path"] == plan)
                        entry["mode" if admission == "executable" else "stage"] = (
                            "100755" if admission == "executable" else 1)
                    observed["sourceDigest"] = preparation.digest(
                        {key: item for key, item in observed.items() if key != "sourceDigest"})
                    with self.assertRaises(JenkinsError) as rejected:
                        formal._external_capture(
                            management, preparation.checkout_external_file_view(), inspection=True)
                    self.assertEqual("external_control_forbidden", rejected.exception.code)
            self.assertEqual(raw["files"][plan], "6" * 64)

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

    def test_sealed_commit_matches_original_capture_and_actual_prepared_native_cwd(self):
        self.commit_contract()

    def test_admission_delivery_keeps_original_sealed_closure_and_observes_foreign_drift(self):
        self.commit_contract(admission=True)

    def test_legacy_unsealed_delivery_keeps_strict_full_dependency_comparison(self):
        self.commit_contract(sealed=False)

    def commit_contract(self, *, admission=False, sealed=True):
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
            if not sealed:
                for field in ("validationInputPolicy", "capturedInputVerified"):
                    request.pop(field)
            request_ref = runner.write_json(report / "request.json", request)
            native_ref = runner.write_json(report / "native-terminal.json", native)
            result = {"status": "passed", "sourceDigest": before["sourceDigest"], "inputsUnchanged": False,
                      "validationInputPolicy": "sealed-source", "capturedInputVerified": True,
                      "originalEngineUnchanged": True, "dependencyCacheUnchanged": True,
                      "preparedInputsUnchanged": True, "executionRoot": str(execution_root),
                      "preparation": prepared, "request": request_ref, "nativeTerminal": native_ref}
            if not sealed:
                for field in ("validationInputPolicy", "capturedInputVerified"):
                    result.pop(field)
                result["inputsUnchanged"] = True
            terminal = runner.native_terminal_proof(native, native_ref)
            state.put("native_job", "management-" + key + "-native", {
                "owner": "management-" + key, "status": "terminal", "nativeEvidence": native,
                "completeProof": terminal, "terminalProof": terminal})
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
            management.validation_observation = Mock(wraps=lambda *args, **kwargs:
                runner.Management.validation_observation(management, *args, **kwargs))
            management.validation_sealed_observation = Mock(wraps=lambda *args, **kwargs:
                runner.Management.validation_sealed_observation(management, *args, **kwargs))
            management.source = Mock(return_value=copy.deepcopy(engine))
            management.git = Mock(side_effect=lambda path, *args, **kwargs:
                SimpleNamespace(stdout=b"owned.rs\0" if args[0] == "diff" else b"", returncode=0))
            management.text = lambda path, *args: ("b" * 40 if path == worktree else base)
            changed = copy.deepcopy(before)
            changed["externalRoots"][0]["source"]["files"]["vm.rs"] = runner.sha(b"foreign input")
            management.validation_source.return_value = (worktree, changed)
            approved = SimpleNamespace(path=root, namespace=lambda: namespace)
            with patch("tools.jenkins.resources.paths.canonical_build_root", return_value=approved), \
                    patch("tools.jenkins.resources.paths.physical_path_under",
                          side_effect=lambda approved, path, **kwargs: Path(path)):
                if sealed:
                    unchanged_state = copy.deepcopy(state.rows)
                    _row, actual_root, captured, delivery = management.validation_delivery(key, "owned")
                    self.assertEqual(worktree, actual_root)
                    self.assertEqual(before, captured)
                    self.assertFalse(delivery["deliveryObservation"]["inputsUnchanged"])
                    self.assertIsNone(delivery["deliveryObservation"]["externalInputsUnchanged"])
                    self.assertFalse(delivery["deliveryObservation"]["externalInputsObserved"])
                    self.assertEqual(unchanged_state, state.rows)
                    changed_engine = copy.deepcopy(changed)
                    changed_engine["source"]["files"]["owned.rs"] = runner.sha(b"later owned edit")
                    management.source.return_value = changed_engine["source"]
                    with self.assertRaises(ValueError):
                        management.validation_delivery(key, "owned")
                    with self.assertRaises(ValueError):
                        management.commit_worktree()
                    management.source.return_value = copy.deepcopy(engine)
                    management.dependency_cache.return_value = {**before["dependencyCache"], "configSha256": {"config": "f" * 64}}
                    with self.assertRaises(ValueError):
                        management.validation_delivery(key, "owned")
                    with self.assertRaises(ValueError):
                        management.commit_worktree()
                    management.dependency_cache.return_value = copy.deepcopy(before["dependencyCache"])
                else:
                    with self.assertRaises(runner.SourceInputMismatch):
                        management.commit_worktree()
                    with self.assertRaises(runner.SourceInputMismatch):
                        management.validation_delivery(key, "owned")
                    management.validation_source.return_value = (worktree, copy.deepcopy(before))
                management.git.assert_not_called()
                management.validation_with_cache.reset_mock()
                management.validation_observation.reset_mock()
                management.validation_sealed_observation.reset_mock()
                committed = management.commit_worktree()
            self.assertEqual("complete", committed["status"])
            self.assertEqual(before["sourceDigest"], committed["validatedSourceDigest"])
            self.assertEqual("management-" + key + "-native", committed["validationDelivery"]["nativeProofRef"])
            if sealed:
                management.validation_with_cache.assert_not_called()
                management.validation_observation.assert_not_called()
                self.assertEqual(2, management.validation_sealed_observation.call_count)
                self.assertFalse(committed["commitObservation"]["inputsUnchanged"])
                for call in management.validation_sealed_observation.call_args_list:
                    self.assertEqual(before, call.args[0])
                    self.assertEqual(original, call.kwargs["payload"])
            else:
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
            management.payload = {}
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

    def _batch_source_manifest(self, before, coverage, repository_id):
        return {"schemaVersion": 1, "repositoryId": repository_id, "preparationKind": "current-source",
                "baseHead": before["source"]["head"], "declaredDependencies": [], "external": [],
                "validationCoverageDigest": runner.sha(runner.encoded(coverage)),
                "entries": [{"path": path, "sha256": checksum, "objectDigest": checksum,
                             "status": "present" if checksum is not None else "deleted", "size": 0}
                            for path, checksum in sorted(before["source"]["files"].items())]}

    @contextmanager
    def _batch_fixture(self, *, large_capture=False):
        """Synthetic Windows evidence exercises the batch broker, never Cargo."""
        with tempfile.TemporaryDirectory(prefix="ordinary-shared-batch-") as temporary:
            root, state = Path(temporary), MemoryState()
            repo, namespace = root / "repository", root / "namespace"
            repo.mkdir()
            (repo / "owned.rs").write_bytes(b"synthetic owned input\n")
            helper_path = repo / "tools/dev/local_cargo.py"
            helper_path.parent.mkdir(parents=True)
            helper_path.write_bytes(b"synthetic pinned helper")
            tool = root / "cargo.exe"
            tool.write_bytes(b"synthetic trusted executable")
            recipes = [
                {"name": "check", "ordinal": 1, "cargoArguments": ["check", "-p", "zr_math", "--locked", "--offline"],
                 "ownedNativeTimeoutSeconds": 7200},
                {"name": "compile-tests", "ordinal": 2,
                 "cargoArguments": ["test", "-p", "zr_math", "--no-run", "--locked", "--offline"],
                 "ownedNativeTimeoutSeconds": 7200},
                {"name": "run-tests", "ordinal": 3,
                 "cargoArguments": ["test", "-p", "zr_math", "--lib", "--locked", "--offline"],
                 "ownedNativeTimeoutSeconds": 10800, "minimumExpectedMatchedTests": 2,
                 "requiredNamedFixtureTests": ["fixture::first", "fixture::second"],
                 "requiredTestBinaries": ["synthetic_binary"]},
            ]
            contract_ref = runner.write_json(root / "contract.json", {
                "schemaVersion": 1, "sharedSingleBatchRequired": True, "semanticCommands": recipes})
            actor = runner.Management.__new__(runner.Management)
            actor.repo, actor.state, actor.repo_id = repo, state, "repository"
            actor.session, actor.request, actor.operation = "author", "request", "operation"
            actor.action, actor.query, actor.owned = "validate", False, ["owned.rs"]
            actor.key = "b" * 64
            actor.config = {"driverDigest": "d" * 64, "runtimeOperationId": "runtime",
                            "clientSources": {"tools/dev/local_cargo.py": runner.sha(helper_path.read_bytes())}}
            actor.payload = {"action": "validate", "repositoryId": actor.repo_id, "sessionId": actor.session,
                "requestId": actor.request, "operationId": actor.operation, "ownedPaths": actor.owned,
                "driverDigest": actor.config["driverDigest"], "runtimeOperationId": "runtime",
                "sourceView": "worktree", "worktreeId": "owned", "buildRoot": str(root),
                "externalRoots": [], "batch": {"schemaVersion": 1, "sharedSingleBatchRequired": True,
                                              "contractRef": contract_ref, "recipes": recipes}}
            actor.input_digest = runner.sha(runner.encoded(actor.payload))
            actor._batch_parser = entry.parse_cargo
            actor.wt = Mock(return_value=({}, repo))
            actor.build_root = Mock(return_value=str(root))
            actor.outputs = Mock(return_value=namespace / "management")
            approved = SimpleNamespace(path=root, namespace=lambda: namespace, volume_identity="synthetic-volume")
            cache = {"home": str(namespace / "cargo-home"), "configSha256": {}}
            engine = snapshot(repo, "a" * 40)
            if large_capture:
                checksum = runner.sha(b"synthetic declared source bytes; no source tree")
                engine["files"].update({f"fixture/large-evidence/source-inventory/bounded-reader/{n:06d}.rs": checksum
                                        for n in range(65000)})
                engine["diffSha256"] = runner.sha(runner.encoded(engine["files"]))
                engine["sourceDigest"] = runner.sha(runner.encoded({k: v for k, v in engine.items() if k != "sourceDigest"}))
            capture_value = {"sourceView": "worktree", "sourceRoot": str(repo),
                             "source": engine, "externalRoots": [], "dependencyCache": cache}
            before = {**capture_value, "sourceDigest": runner.sha(runner.encoded(capture_value))}
            actor.validation_with_cache = Mock(return_value=(repo, before))
            actor.dependency_cache = Mock(return_value=cache)
            observation = {"originalEngineUnchanged": True, "dependencyCacheUnchanged": True,
                           "inputsUnchanged": True, "externalInputsUnchanged": True}
            actor.validation_observation = Mock(return_value=(repo, before, observation))
            actor.artifact_manifest = Mock(return_value={"path": str(root / "artifacts.json"), "sha256": "a" * 64})
            owner = "management-" + actor.key
            state.put("resource_policy", "default", {"cpuBudget": 8, "memoryBudget": 32 * 1024**3,
                      "diskBudget": 64 * 1024**3, "diskReserveBytes": 35 * 1024**3})
            state.put("management_operation", actor.key, {"inputDigest": actor.input_digest, "phase": "waiting",
                      "payload": actor.payload, "requestId": actor.request, "operationId": actor.operation})
            resources = SimpleNamespace(queue=Mock(), scan_inventory=Mock(return_value=({}, "inventory")),
                                        admit=Mock(), release=Mock(), release_unlaunched=Mock())
            def admit(*args, **kwargs):
                key = owner + ":" + owner
                reservation = state.put("resource_reservation", key, {
                    "owner": owner, "status": "active", "buildRoot": str(root)})
                state.put("pool_writer_hold", kwargs["writer_key"], {
                    "owner": owner, "status": "active", "reservationKey": key})
                return reservation
            resources.admit.side_effect = admit
            def release(key, **kwargs):
                self.assertEqual(3, len(actor.batch_claims()))
                self.assertTrue(all(claim["status"] == "complete" for claim in actor.batch_claims()))
                row = state.get("resource_reservation", key)
                if row["payload"]["status"] == "released":
                    return False
                state.put("resource_reservation", key, {**row["payload"], "status": "released"})
                hold = state.get("pool_writer_hold", kwargs["writer_key"])
                state.put("pool_writer_hold", kwargs["writer_key"], {**hold["payload"], "status": "released"})
                return True
            resources.release.side_effect = release
            def scratch(*, environment=None, **kwargs):
                report = actor.outputs() / "reports" / actor.key
                if environment is not None:
                    value = {"environmentDigest": runner.sha(runner.encoded(environment)), "status": "owned"}
                    state.put("management_task_scratch", actor.key, value)
                    return runner.write_json(report / "tmp-generation.json", value)
                self.assertFalse(resources.release.called)
                self.assertEqual("closed", actor.batch_checkpoint()["stage"])
                self.assertEqual(3, len(actor.batch_claims()))
                return {"status": "complete", "removedFileCount": 0, "reclaimedBytes": 0, "receipt": None}
            actor.task_tmp = Mock(side_effect=scratch)
            preparation_helper = SimpleNamespace(
                preparation_key=Mock(return_value="c" * 64), seal_source=Mock(),
                materialize_source=Mock(), verify_source=Mock(),
                verify_sealed_capture=preparation.verify_sealed_capture)
            def seal(_state, **kwargs):
                manifest = self._batch_source_manifest(before, kwargs["coverage"], actor.repo_id)
                reference = runner.sha(runner.encoded(manifest))
                record = {"status": "sealed", "repositoryRoot": str(repo), "repositoryId": actor.repo_id,
                    "sourceDigest": reference, "sourceManifest": manifest, "manifest": manifest,
                    "objectRoot": str(namespace), "buildRoot": str(root), "coverage": kwargs["coverage"],
                    "coverageDigest": manifest["validationCoverageDigest"]}
                old = state.get("sealed_input", reference)
                if old:
                    self.assertEqual(record, old["payload"])
                else:
                    state.put("sealed_input", reference, record, expected_version=0)
                sealed = {"sourceManifest": manifest, "sealedInputRef": reference,
                    "sourceRoot": str(repo), "capturedSourceDigest": before["sourceDigest"],
                    "capturedInputVerified": True, "coverageDigest": manifest["validationCoverageDigest"],
                    "metrics": {"reused": True, "fixture": True}}
                return sealed
            preparation_helper.seal_source.side_effect = seal
            def materialize(_state, **kwargs):
                state.put("execution", owner, {"kind": "management_owned_cargo", "status": "preparing",
                    "reservationKey": kwargs["reservation_key"], "preparationKey": kwargs["preparation_key_value"],
                    "command": kwargs["command"], "sessionId": actor.session})
                value = {"physicalPath": str(kwargs["source_path"]), "sourceDigest": kwargs["sealed"]["sealedInputRef"],
                    "sealedInputRef": kwargs["sealed"]["sealedInputRef"],
                    "capturedSourceDigest": before["sourceDigest"], "reused": True, "changedFileCount": 0}
                return value
            preparation_helper.materialize_source.side_effect = materialize
            preparation_helper.verify_source.side_effect = lambda manifest, path, approved: {
                "sourceDigest": runner.sha(runner.encoded(manifest)), "fileCount": 2, "filesDigest": "f" * 64}
            local = SimpleNamespace(free_bytes=Mock(return_value=100 * 1024**3),
                prepare_command=Mock(side_effect=lambda args, target, **kwargs: ["cargo", *args, "--target-dir", str(target)]),
                _run_owned_command=Mock())
            options = {"failedOrdinal": None, "zeroOrdinal": None, "incompleteOrdinal": None}
            launches = []
            def native(command, cwd, environment, report, timeout):
                ordinal = int(report.name)
                launches.append(ordinal)
                self.assertEqual("active", state.get("pool_writer_hold", actor.batch_checkpoint()["writer"])["payload"]["status"])
                self.assertEqual(recipes[ordinal - 1]["ownedNativeTimeoutSeconds"], timeout)
                executable = actor.outputs() / "pools" / actor.batch_checkpoint()["pool"] / "target/synthetic_binary.exe"
                stdout = [json.dumps({"reason": "compiler-artifact", "target": {"name": "synthetic_binary"},
                    "executable": str(executable), "filenames": [str(executable)], "fresh": ordinal != 1}),
                    json.dumps({"reason": "build-finished", "success": True})]
                running = command[1] == "test" and "--no-run" not in command
                if running:
                    cases = [] if options["zeroOrdinal"] == ordinal else ["fixture::first", "fixture::second"]
                    stdout += ["running %d tests" % len(cases)]
                    stdout += ["test " + name + " ... ok" for name in cases]
                    stdout += ["test result: ok. %d passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;" % len(cases)]
                (report / "stdout.bin").write_text("\n".join(stdout) + "\n", encoding="utf-8")
                (report / "stderr.bin").write_text("Running unittests owned.rs (" + str(executable) + ")\n"
                                                  if running else "", encoding="utf-8")
                value = {"command": command, "cwd": str(cwd), "pid": 100 + ordinal,
                    "creationTime": "13400000000000000" + str(ordinal),
                    "rootExitCode": 1 if options["failedOrdinal"] == ordinal else 0, "terminalErrors": [],
                    "processTree": {"scope": "retained-windows-job", "terminal": True,
                        "nativeWaiterJoined": True, "pipeEOF": True, "readersFinished": True,
                        "pipes": {"stdout": {"eof": True}, "stderr": {"eof": options["incompleteOrdinal"] != ordinal}}},
                    "nativeJobEvidence": {"errors": [], "records": {
                        "afterTerminal": {"activeProcesses": 0}, "afterClose": {"activeProcesses": 0}}}}
                runner.write_json(report / "started.json", {k: value[k] for k in ("command", "cwd", "pid", "creationTime")})
                return value
            local._run_owned_command.side_effect = native
            trusted = {"cargoPath": str(tool), "rustcPath": str(tool),
                       "cargoSha256": runner.sha(tool.read_bytes()), "rustcSha256": runner.sha(tool.read_bytes())}
            physical = lambda approved, value, **kwargs: Path(value)
            spec = SimpleNamespace(loader=Mock())
            @contextmanager
            def lock(*args, **kwargs):
                yield
            with patch("tools.jenkins.resources.capacity.ResourceManager", return_value=resources), \
                    patch("tools.jenkins.resources.paths.canonical_build_root", return_value=approved), \
                    patch("tools.jenkins.resources.paths.physical_path_under", side_effect=physical), \
                    patch("tools.jenkins.workflow.planning._environment_policy", return_value=({}, trusted, root / "policy")), \
                    patch("tools.jenkins.state.locks.process_lock", side_effect=lock), \
                    patch.object(runner, "source_helper", return_value=preparation_helper), \
                    patch.object(runner.importlib.util, "spec_from_file_location", return_value=spec), \
                    patch.object(runner.importlib.util, "module_from_spec", return_value=local):
                yield {"actor": actor, "state": state, "resources": resources, "preparation": preparation_helper,
                       "local": local, "launches": launches, "options": options, "before": before, "recipes": recipes}

    def test_batch_preserves_all_original_commands_producers_selectors_and_single_identity(self):
        path = ROOT / ".jenkins/state/records-queue/archive/8cb75329f06f6fe219e1641ac3a7b0fb53135fd40b7f7a463d5deff9eb4452a6/runtime-editor-optimize-batch-handoff-01a0df1a-20261012-v1.json"
        raw = path.read_bytes()
        self.assertEqual("ad1ce1a11b8aa9abfe985cf100d968db64c9f9a99c8a068398e8e7fa3e72bcc3", runner.sha(raw))
        original = json.loads(raw)["originalCommandContract"]
        actor = runner.Management.__new__(runner.Management)
        actor._batch_parser = entry.parse_cargo
        actor.build_root = Mock(return_value=r"E:\cargo-targets")
        actor.payload = {"sourceView": "worktree", "worktreeId": "input", "buildRoot": r"E:\cargo-targets",
            "batch": {"schemaVersion": 1, "sharedSingleBatchRequired": True,
                      "contractRef": {"path": str(path), "sha256": runner.sha(raw)},
                      "recipes": copy.deepcopy(original["semanticCommands"])}}
        batch = actor.validation_batch()
        self.assertEqual(original["semanticCommands"], batch["coverage"]["recipes"])
        self.assertEqual(15, len(batch["recipes"]))
        self.assertEqual(16, batch["coverage"]["producerContract"]["totalProducerIdentities"])
        self.assertEqual(111600, sum(value["ownedNativeTimeoutSeconds"] for value in batch["coverage"]["recipes"]))
        self.assertTrue(batch["recipes"][0]["lib"] and batch["recipes"][0]["bins"] and batch["recipes"][0]["allTests"])
        self.assertTrue(batch["recipes"][8]["ignored"] and batch["recipes"][8]["nocapture"])
        self.assertGreater(len(batch["recipes"][1]["harnessFilters"]), 1)
        self.assertTrue(batch["recipes"][9]["exact"])
        actor.payload = {"verb": "check", "packages": ["zr_math"]}
        single = actor.validation_recipe()
        expected = {"verb": "check", "profile": "dev", "packages": ["zr_math"], "features": [], "tests": [],
            "testFilter": None, "jobs": 1, "testThreads": 1, "offline": False, "noDefaultFeatures": False,
            "lib": False, "noRun": False, "allFeatures": False, "locked": True,
            "command": ["check", "--jobs", "1", "--message-format=json", "--locked", "--package", "zr_math"]}
        self.assertEqual(expected, single)
        self.assertEqual(runner.sha(runner.encoded(expected)), runner.sha(runner.encoded(single)))
        self.assertNotIn("harnessFilters", entry.parse_cargo(["check", "-p", "zr_math"]))

    def test_batch_shared_capture_seal_pool_and_native_sequence_retains_actual_each_step(self):
        with self._batch_fixture() as f:
            result = f["actor"].run()
            self.assertEqual("passed", result["status"])
            self.assertEqual([1, 2, 3], f["launches"])
            f["actor"].validation_with_cache.assert_called_once()
            f["preparation"].seal_source.assert_called_once()
            f["preparation"].materialize_source.assert_called_once()
            self.assertEqual(2, f["preparation"].verify_source.call_count)
            self.assertEqual(2, f["actor"].validation_observation.call_count)
            self.assertEqual("c" * 64, f["actor"].batch_checkpoint()["pool"])
            self.assertEqual(3, len(f["state"].list("native_job")))
            self.assertEqual(2, result["steps"][-1]["passedTestCount"])
            self.assertTrue(result["steps"][1]["compileOnly"])
            self.assertTrue(all(row["payload"]["completeProof"]["complete"] for row in f["state"].list("native_job")))
            self.assertEqual("complete", f["actor"].batch_checkpoint()["stage"])
            self.assertFalse(result["formalAcceptance"] or result["performanceAccepted"] or result["productsAccepted"])
            again = f["actor"].run()
            self.assertEqual(result, again)
            self.assertEqual([1, 2, 3], f["launches"])

    def _batch_delivery_consumer(self, fixture):
        actor, state, before = fixture["actor"], fixture["state"], fixture["before"]
        consumer = runner.Management.__new__(runner.Management)
        consumer.repo = actor.repo.parent / "delivery-repository"
        (consumer.repo / ".jenkins/state/controls").mkdir(parents=True)
        consumer.state, consumer.repo_id, consumer.session = state, actor.repo_id, actor.session
        consumer.config, consumer.owned = actor.config, actor.owned
        consumer.key, consumer.request, consumer.operation = "e" * 64, "delivery-request", "delivery-operation"
        consumer._batch_parser, consumer.build_root = entry.parse_cargo, actor.build_root
        record = {"status": "ready", "sessionId": actor.session}
        state.put("management_worktree", "owned", record)
        consumer.wt = Mock(return_value=(record, actor.repo))
        consumer.payload = {"worktreeId": "owned", "validationKey": actor.key, "expectedHead": before["source"]["head"],
                            "message": "Synthetic complete batch delivery"}
        consumer.validation_source = Mock(return_value=(actor.repo, copy.deepcopy(before)))
        consumer.dependency_cache = Mock(return_value=copy.deepcopy(before["dependencyCache"]))
        consumer.source = Mock(return_value=copy.deepcopy(before["source"]))
        consumer.git = Mock(side_effect=lambda path, *args, **kwargs:
            SimpleNamespace(stdout=b"owned.rs\0" if args[0] == "diff" else b"", returncode=0))
        consumer.text = lambda path, *args: "f" * 40 if path == actor.repo else before["source"]["head"]
        return consumer

    def test_batch_delivery_reads_all_steps_and_commit_uses_the_same_original_proofs(self):
        with self._batch_fixture() as f:
            result = f["actor"].run()
            consumer = self._batch_delivery_consumer(f)
            original_state = copy.deepcopy(f["state"].rows)
            _row, root, captured, delivery = consumer.validation_delivery(f["actor"].key, "owned")
            self.assertEqual(f["actor"].repo, root)
            self.assertEqual(f["before"], captured)
            self.assertEqual(result["nativeProofRefs"], delivery["nativeProofRefs"])
            self.assertEqual([step["nativeTerminal"] for step in result["steps"]], delivery["nativeTerminals"])
            self.assertEqual(3, len(delivery["stepReceipts"]))
            self.assertEqual(original_state, f["state"].rows)
            committed = consumer.commit_worktree()
            self.assertEqual("complete", committed["status"])
            self.assertEqual(result["nativeProofRefs"], committed["validationDelivery"]["nativeProofRefs"])
            self.assertEqual(result["sourceDigest"], committed["validatedSourceDigest"])
            self.assertEqual([1, 2, 3], f["launches"])
            self.assertEqual(3, f["local"]._run_owned_command.call_count)
            self.assertIn(("repository", "author", "commit", ("owned.rs",)), f["state"].authorizations)
            self.assertFalse(result["formalAcceptance"] or result["performanceAccepted"] or result["productsAccepted"])

    def test_batch_consumers_reject_missing_middle_native_or_eof_without_replay(self):
        for missing in ("native", "stderr-eof"):
            with self.subTest(missing=missing), self._batch_fixture() as f:
                f["actor"].run()
                consumer = self._batch_delivery_consumer(f)
                middle = f["actor"].batch_claims()[1]
                path = Path(middle["nativeTerminal"]["path"])
                if missing == "native":
                    path.unlink()
                else:
                    native = json.loads(path.read_bytes())
                    native["processTree"]["pipes"]["stderr"]["eof"] = False
                    path.write_bytes(runner.encoded(native) + b"\n")
                original_state = copy.deepcopy(f["state"].rows)
                with self.assertRaises((ValueError, OSError)):
                    consumer.validation_delivery(f["actor"].key, "owned")
                with self.assertRaises((ValueError, OSError)):
                    consumer.commit_worktree()
                consumer.git.assert_not_called()
                self.assertEqual(original_state, f["state"].rows)
                self.assertEqual([1, 2, 3], f["launches"])
                self.assertEqual(3, f["local"]._run_owned_command.call_count)

    def _batch_sealed_donor(self, fixture):
        """Retain an independent successful single-recipe donor and exact receipt."""
        actor, state, before = fixture["actor"], fixture["state"], fixture["before"]
        key = "1" * 64
        original = {**{k: v for k, v in actor.payload.items() if k != "batch"},
                    "requestId": "donor-request", "operationId": "donor-operation",
                    "verb": "test", "packages": ["zr_math"], "lib": True,
                    "expectedSourceDigest": before["sourceDigest"]}
        target = actor.outputs() / "pools" / ("c" * 64) / "target"
        execution_root = target.parent / "inputs/source"
        probe = runner.Management.__new__(runner.Management)
        probe.payload = original
        donor_coverage = {name: value for name, value in probe.validation_recipe().items() if name not in {"verb", "command"}}
        manifest = self._batch_source_manifest(before, donor_coverage, actor.repo_id)
        sealed_ref = runner.sha(runner.encoded(manifest))
        state.put("sealed_input", sealed_ref, {"status": "sealed", "repositoryRoot": str(actor.repo),
            "repositoryId": actor.repo_id, "sourceDigest": sealed_ref, "sourceManifest": manifest, "manifest": manifest,
            "objectRoot": str(actor.outputs().parent), "buildRoot": actor.build_root(),
            "coverage": donor_coverage, "coverageDigest": manifest["validationCoverageDigest"]}, expected_version=0)
        prepared = {"physicalPath": str(execution_root), "sealedInputRef": sealed_ref,
                    "sourceDigest": sealed_ref, "capturedSourceDigest": before["sourceDigest"]}
        native = {"command": [str(actor.repo.parent / "cargo.exe"), "test", "--lib", "--locked"],
            "cwd": str(execution_root), "pid": 901, "creationTime": "134000000000000901", "rootExitCode": 0,
            "terminalErrors": [], "processTree": {"scope": "retained-windows-job", "terminal": True,
                "nativeWaiterJoined": True, "pipeEOF": True, "readersFinished": True,
                "pipes": {"stdout": {"eof": True}, "stderr": {"eof": True}}},
            "nativeJobEvidence": {"errors": [], "records": {
                "afterTerminal": {"activeProcesses": 0}, "afterClose": {"activeProcesses": 0}}}}
        report = actor.outputs() / "reports" / key
        request = {"payload": original, "source": before, "executionRoot": str(execution_root),
            "command": native["command"], "preparation": prepared,
            "preparedSource": {"sourceDigest": sealed_ref}, "validationInputPolicy": "sealed-source",
            "capturedInputVerified": True, "inputObservation": {
                "originalEngineUnchanged": True, "dependencyCacheUnchanged": True}}
        request_ref = runner.write_json(report / "request.json", request)
        native_ref = runner.write_json(report / "native-terminal.json", native)
        result = {"status": "passed", "sourceDigest": before["sourceDigest"], "executionRoot": str(execution_root),
            "validationInputPolicy": "sealed-source", "capturedInputVerified": True,
            "originalEngineUnchanged": True, "dependencyCacheUnchanged": True,
            "inputsUnchanged": False, "preparedInputsUnchanged": True, "preparation": prepared,
            "request": request_ref, "nativeTerminal": native_ref, "formalAcceptance": False}
        receipt_ref = runner.write_json(report / "receipt.json", result)
        proof = runner.native_terminal_proof(native, native_ref)
        state.put("native_job", "management-" + key + "-native", {"owner": "management-" + key,
                  "status": "terminal", "nativeEvidence": native, "completeProof": proof, "terminalProof": proof})
        envelope = {"operationKey": key, "requestId": original["requestId"], "operationId": original["operationId"],
                    "sessionId": actor.session, "driverDigest": actor.config["driverDigest"],
                    "runtimeOperationId": actor.config["runtimeOperationId"]}
        state.put("management_operation", key, {"phase": "complete", "action": "validate",
            "sessionId": actor.session, "requestId": original["requestId"], "operationId": original["operationId"],
            "payload": original, "result": {**result, **envelope}})
        state.put("management_source", key, before)
        return {"operationKey": key, "capturedSourceDigest": before["sourceDigest"], "receiptRef": receipt_ref}

    def _batch_explicit_reuse(self, fixture, reuse):
        actor, state, before = fixture["actor"], fixture["state"], fixture["before"]
        actor.payload["batch"]["reuseValidation"] = reuse
        actor.payload["expectedSourceDigest"] = reuse["capturedSourceDigest"]
        actor.input_digest = runner.sha(runner.encoded(actor.payload))
        row = state.get("management_operation", actor.key)
        state.put("management_operation", actor.key, {
            **row["payload"], "payload": actor.payload, "inputDigest": actor.input_digest},
            expected_version=row["version"])
        actor.source = Mock(return_value=copy.deepcopy(before["source"]))
        actor.validation_with_cache = Mock(side_effect=AssertionError("Explicit donor must not recapture live external"))
        actor.validation_source = Mock(side_effect=AssertionError("Mutable external traversal is outside sealed reuse"))
        actor.validation_observation = Mock(side_effect=AssertionError("Explicit reuse needs only engine/cache observation"))
        actor.validation_sealed_observation = Mock(wraps=lambda *args, **kwargs:
            runner.Management.validation_sealed_observation(actor, *args, **kwargs))

    def test_explicit_sealed_batch_reuse_binds_old_receipt_and_runs_all_new_recipes(self):
        with self._batch_fixture(large_capture=True) as f:
            reuse = self._batch_sealed_donor(f)
            donor_rows = {pair: copy.deepcopy(row) for pair, row in f["state"].rows.items()
                          if pair[0] == "sealed_input" or pair[1] == reuse["operationKey"]
                          or pair[1] == "management-" + reuse["operationKey"] + "-native"}
            self._batch_explicit_reuse(f, reuse)
            probe = runner.Management.__new__(runner.Management)
            probe.payload, probe.repo, probe.config = f["actor"].payload, f["actor"].repo, f["actor"].config
            probe._batch_parser, probe.build_root = entry.parse_cargo, f["actor"].build_root
            self.assertEqual(reuse, probe.validation_batch()["reuseValidation"])
            self.assertFalse(hasattr(probe, "state"))
            single = runner.Management.__new__(runner.Management)
            single.payload = {"verb": "check", "packages": ["zr_math"], "reuseValidation": reuse}
            with self.assertRaises(ValueError):
                single.validation_recipe()
            result = f["actor"].run()
            self.assertEqual("passed", result["status"])
            self.assertEqual([1, 2, 3], f["launches"])
            self.assertEqual(f["before"], f["state"].get("management_source", f["actor"].key)["payload"])
            request = json.loads(Path(result["request"]["path"]).read_bytes())
            donor_request = f["state"].get("management_operation", reuse["operationKey"])["payload"]["result"]["request"]
            for reference in (donor_request, result["request"], f["actor"].batch_checkpoint()["sealed"]):
                self.assertGreater(Path(reference["path"]).stat().st_size, 8 * 1024**2)
                self.assertLess(Path(reference["path"]).stat().st_size, runner.SOURCE_EVIDENCE_MAX_BYTES)
                with self.assertRaises(ValueError):
                    runner.read_reference(reference)
            provenance = request["sourceProvenance"]
            self.assertEqual(provenance, f["actor"].batch_checkpoint()["sourceProvenance"])
            self.assertNotEqual(f["before"]["sourceDigest"], request["preparation"]["sealedInputRef"])
            self.assertNotEqual(provenance["sealedInputRef"], request["preparation"]["sealedInputRef"])
            donor = f["state"].get("sealed_input", provenance["sealedInputRef"])["payload"]
            current = f["state"].get("sealed_input", request["preparation"]["sealedInputRef"])["payload"]
            self.assertNotEqual(donor["coverageDigest"], current["coverageDigest"])
            self.assertEqual({k: v for k, v in donor["sourceManifest"].items() if k != "validationCoverageDigest"},
                             {k: v for k, v in current["sourceManifest"].items() if k != "validationCoverageDigest"})
            self.assertEqual(reuse["operationKey"], provenance["operationKey"])
            self.assertEqual(reuse["receiptRef"], provenance["receiptRef"])
            self.assertEqual(f["before"]["sourceDigest"], provenance["capturedSourceDigest"])
            self.assertNotIn("management-" + reuse["operationKey"] + "-native", result["nativeProofRefs"])
            for pair, original in donor_rows.items():
                self.assertEqual(original, f["state"].rows[pair])
            actor = f["actor"]
            actor.validation_with_cache.assert_not_called()
            actor.validation_source.assert_not_called()
            actor.validation_observation.assert_not_called()
            self.assertEqual(3, actor.validation_sealed_observation.call_count)
            f["preparation"].seal_source.assert_called_once()
            call = f["preparation"].seal_source.call_args.kwargs
            self.assertEqual(f["before"], call["snapshot"])
            self.assertEqual("management-" + actor.key, call["owner"])
            self.assertEqual(f["recipes"], call["coverage"]["recipes"])
            self.assertEqual(provenance, request["sourceProvenance"])
            for observation in result["inputObservations"].values():
                self.assertTrue(observation["originalEngineUnchanged"])
                self.assertTrue(observation["dependencyCacheUnchanged"])
                self.assertFalse(observation["externalInputsObserved"])
                self.assertIsNone(observation["externalInputsUnchanged"])
            self.assertFalse(result["inputsUnchanged"])
            consumer = self._batch_delivery_consumer(f)
            _row, _root, captured, delivery = consumer.validation_delivery(actor.key, "owned")
            self.assertEqual(f["before"], captured)
            self.assertEqual(result["nativeProofRefs"], delivery["nativeProofRefs"])
            self.assertFalse(delivery["deliveryObservation"]["externalInputsObserved"])
            self.assertFalse(result["formalAcceptance"] or result["performanceAccepted"] or result["productsAccepted"])

    def test_explicit_batch_reuse_rejects_bad_donor_receipt_capture_engine_or_cache_before_native(self):
        for mismatch in ("receipt", "capture", "engine", "cache"):
            with self.subTest(mismatch=mismatch), self._batch_fixture() as f:
                reuse = self._batch_sealed_donor(f)
                if mismatch == "receipt":
                    reuse["receiptRef"] = {**reuse["receiptRef"], "sha256": "0" * 64}
                elif mismatch == "capture":
                    reuse["capturedSourceDigest"] = "0" * 64
                self._batch_explicit_reuse(f, reuse)
                actor = f["actor"]
                if mismatch == "engine":
                    engine = copy.deepcopy(f["before"]["source"])
                    engine["files"]["owned.rs"] = runner.sha(b"later engine edit")
                    actor.source.return_value = engine
                elif mismatch == "cache":
                    actor.dependency_cache.return_value = {**f["before"]["dependencyCache"], "configSha256": {"config": "0" * 64}}
                with self.assertRaises(ValueError):
                    actor.validate_batch()
                self.assertIsNone(f["state"].get("management_source", actor.key))
                self.assertEqual([], f["launches"])
                f["preparation"].seal_source.assert_not_called()
                f["preparation"].materialize_source.assert_not_called()

    def test_batch_capacity_wait_recovers_same_parent_and_volume_without_source_walk(self):
        with self._batch_fixture() as f:
            actor = f["actor"]
            f["local"].free_bytes.return_value = 1
            result = actor.run()
            self.assertEqual("waiting", result["status"])
            self.assertEqual(actor.payload["buildRoot"], result["buildRoot"])
            self.assertEqual([], f["launches"])
            actor.validation_with_cache.assert_not_called()
            f["preparation"].seal_source.assert_not_called()
            f["resources"].admit.assert_not_called()
            digest, key, payload = actor.input_digest, actor.key, copy.deepcopy(actor.payload)
            f["local"].free_bytes.return_value = 100 * 1024**3
            self.assertEqual("passed", actor.run()["status"])
            self.assertEqual((digest, key, payload), (actor.input_digest, actor.key, actor.payload))
            self.assertEqual([1, 2, 3], f["launches"])

    def test_batch_resume_reconciles_real_step_files_without_native_replay_and_unknown_eof_stays_held(self):
        with self._batch_fixture() as f:
            actor, original = f["actor"], f["actor"].batch_step_proof
            def interrupted(claim, *, persist=False):
                if claim["ordinal"] == 1 and persist:
                    raise RuntimeError("Synthetic interruption after actual terminal bytes")
                return original(claim, persist=persist)
            with patch.object(actor, "batch_step_proof", side_effect=interrupted):
                self.assertEqual("unknown", actor.run()["status"])
            self.assertEqual([1], f["launches"])
            self.assertTrue(actor.batch_recovery_ready())
            self.assertEqual("prepared", actor.batch_checkpoint()["stage"])
            self.assertEqual("passed", actor.run()["status"])
            self.assertEqual([1, 2, 3], f["launches"])
            actor.validation_with_cache.assert_called_once()
            f["preparation"].seal_source.assert_called_once()
            f["preparation"].materialize_source.assert_called_once()
        with self._batch_fixture() as f:
            f["options"]["incompleteOrdinal"] = 1
            self.assertEqual("unknown", f["actor"].run()["status"])
            self.assertFalse(f["actor"].batch_recovery_ready())
            self.assertEqual("unknown", f["actor"].run()["status"])
            self.assertEqual([1], f["launches"])
            f["resources"].release.assert_not_called()
            self.assertEqual("active", f["state"].get("pool_writer_hold", f["actor"].batch_checkpoint()["writer"])["payload"]["status"])

    def test_batch_zero_matching_and_failed_early_steps_cannot_be_overwritten_by_later_success(self):
        for option, ordinal in (("zeroOrdinal", 3), ("failedOrdinal", 1)):
            with self.subTest(option=option), self._batch_fixture() as f:
                f["options"][option] = ordinal
                result = f["actor"].run()
                self.assertEqual("failed", result["status"])
                self.assertEqual("failed", result["steps"][ordinal - 1]["status"])
                self.assertEqual([1, 2, 3], f["launches"])
                if option == "zeroOrdinal":
                    self.assertEqual(0, result["steps"][-1]["passedTestCount"])
                    self.assertFalse(result["steps"][-1]["testEvidenceComplete"])
                else:
                    self.assertEqual("passed", result["steps"][-1]["status"])
                self.assertFalse(result["formalAcceptance"] or result["m8Accepted"]
                                 or result["performanceAccepted"] or result["productsAccepted"])
                f["resources"].release.assert_called_once()


class WorktreeLifecycleTests(unittest.TestCase):
    """Contracts for path budgeting, worktree adoption, and traversal refusal."""

    def _management(self, temporary, *, session="s", operation="o", owned=("owned.txt",)):
        repo = Path(temporary) / "repository"
        worktrees = repo / ".jenkins/worktrees"
        worktrees.mkdir(parents=True)
        management = runner.Management.__new__(runner.Management)
        management.repo, management.worktrees = repo, worktrees
        management.state, management.repo_id = MemoryState(), "repository"
        management.session, management.request, management.operation = session, "r", operation
        management.owned = list(owned)
        management.config = {"driverDigest": "d" * 64, "runtimeOperationId": "runtime"}
        management.payload = {"name": "task", "baseHead": "a" * 40}
        return management, worktrees, management.state

    def test_deep_checkout_is_refused_before_any_git_write(self):
        with tempfile.TemporaryDirectory(prefix="ordinary-budget-") as temporary:
            management, worktrees, _state = self._management(temporary)
            management.text = lambda path, *args: "a" * 40
            calls = []
            management.git = lambda path, *args, **kwargs: calls.append(args) or SimpleNamespace(stdout=b"", returncode=0)
            with patch.object(runner, "deepest_tracked_paths", return_value=(210, 200)):
                with self.assertRaises(runner.ManagedPathBudget) as caught:
                    management.new_worktree()
            self.assertEqual("managed_path_budget_exceeded", caught.exception.code)
            self.assertEqual([], calls, "budget must be decided before Git touches the disk")
            self.assertFalse((worktrees / ("s-task")).exists())

    def test_repeated_create_worktree_with_same_identity_reuses_its_record(self):
        with tempfile.TemporaryDirectory(prefix="ordinary-idempotent-") as temporary:
            management, worktrees, state = self._management(temporary)
            management.text = lambda path, *args: "a" * 40
            added = []

            def git(path, *args, **kwargs):
                added.append(args)
                (worktrees / "s-task").mkdir(parents=True, exist_ok=True)
                return SimpleNamespace(stdout=b"", returncode=0)

            management.git = git
            with patch.object(runner, "deepest_tracked_paths", return_value=(10, 10)):
                first = management.new_worktree()
                second = management.new_worktree()
            self.assertEqual(first[0], second[0])
            self.assertEqual(first[2]["baseHead"], second[2]["baseHead"])
            self.assertEqual(1, len(added), "the second call must not add a second worktree")
            self.assertEqual(("worktree", "add"), added[0][:2])
            self.assertEqual("repository", first[2]["repositoryId"])
            self.assertEqual("runtime", first[2]["runtimeOperationId"])
            self.assertEqual("e" * 64, first[2]["writerAuthorizationSnapshot"])
            self.assertEqual(runner.sha(runner.encoded({"repositoryId": "repository", "sessionId": "s",
                "requestId": "r", "operationId": "o"})), first[2]["creationOperationKey"])

    def test_reference_bound_to_another_operation_reports_a_reason_code(self):
        with tempfile.TemporaryDirectory(prefix="ordinary-conflict-") as temporary:
            management, worktrees, state = self._management(temporary)
            management.text = lambda path, *args: "a" * 40
            (worktrees / "s-task").mkdir(parents=True)
            state.put("management_worktree", "s-task", {
                "status": "ready", "path": str(worktrees / "s-task"), "branch": None,
                "baseHead": "b" * 40, "sessionId": "s", "operationId": "other",
                "ownedPaths": ["owned.txt"], "driverDigest": "d" * 64})
            with self.assertRaises(runner.WorktreeConflict) as caught:
                management.new_worktree()
            self.assertEqual("reference_in_use", caught.exception.reason)
            self.assertEqual("other", caught.exception.details["recordedOperation"])

    def test_unregistered_directory_is_adopted_only_when_git_identity_matches(self):
        with tempfile.TemporaryDirectory(prefix="ordinary-adopt-") as temporary:
            management, worktrees, _state = self._management(temporary)
            (worktrees / "s-task").mkdir(parents=True)

            def own_repository(path, *args):
                # The base commit resolves first, then adoption compares the
                # physical Git identity of the directory that already exists.
                if args[:1] == ("rev-parse",) and args[1].endswith("^{commit}"):
                    return "a" * 40
                if "--git-common-dir" in args:
                    return str(management.repo / ".git")
                if args[:2] == ("rev-parse", "HEAD"):
                    return "a" * 40
                raise AssertionError("Unexpected read while adopting: " + repr(args))

            management.text = own_repository
            adopted = management.new_worktree()
            self.assertTrue(adopted[2]["adopted"])

            foreign = runner.Management.__new__(runner.Management)
            foreign.repo, foreign.worktrees = management.repo, worktrees
            foreign.state, foreign.repo_id = MemoryState(), "repository"
            foreign.session, foreign.request, foreign.operation = "s", "r", "o"
            foreign.owned = ["owned.txt"]
            foreign.config = {"driverDigest": "d" * 64, "runtimeOperationId": "runtime"}
            foreign.payload = {"name": "second", "baseHead": "a" * 40}

            def other_repository(path, *args):
                if args[:1] == ("rev-parse",) and args[1].endswith("^{commit}"):
                    return "a" * 40
                if "--git-common-dir" in args:
                    return "C:\\somewhere\\else\\.git"
                raise AssertionError("Unexpected read for a foreign directory: " + repr(args))

            foreign.text = other_repository
            (worktrees / "s-second").mkdir(parents=True)
            with self.assertRaises(runner.WorktreeConflict) as caught:
                foreign.new_worktree()
            self.assertEqual("foreign_repository", caught.exception.reason)

    def test_current_source_reader_blocks_release_even_when_preparation_is_terminal(self):
        with tempfile.TemporaryDirectory(prefix="ordinary-reader-release-") as temporary:
            management, worktrees, state = self._management(temporary)
            root = worktrees / "s-task"
            root.mkdir()
            for intent in ({"worktreeId": "s-task", "sourceRoot": str(root)},
                           {"sourceRoot": str(root)}):
                state.put("current_source_intent", "reader", intent)
                state.put("current_source_preparation", "reader", {"status": "prepared"})
                with state.transaction() as connection:
                    with self.assertRaisesRegex(ValueError, "retained current-source reader"):
                        management.assert_no_current_source_readers("s-task", root, connection=connection)
            state.rows.pop(("current_source_intent", "reader"))
            state.put("current_source_intent", "foreign", {"worktreeId": "foreign", "sourceRoot": str(worktrees / "foreign")})
            with state.transaction() as connection:
                management.assert_no_current_source_readers("s-task", root, connection=connection)

    def test_physical_paths_refuse_parent_traversal(self):
        with tempfile.TemporaryDirectory(prefix="ordinary-traversal-") as temporary:
            with self.assertRaises(ValueError):
                runner.plain(Path(temporary) / ".." / "escape")
            self.assertTrue(runner.plain(Path(temporary)).is_absolute())


class WorktreeRecoveryTests(unittest.TestCase):
    """A dead owner must be recoverable without certifying the work it never finished."""

    KEY = "a" * 64

    def _fixture(self, temporary, *, worktree_status="releasing", operation_phase="running",
                 session="s", operation_session="s", present=False, common_dir=None):
        repo = Path(temporary) / "repository"
        worktrees = repo / ".jenkins/worktrees"
        worktrees.mkdir(parents=True)
        path = worktrees / "s-task"
        if present:
            path.mkdir(parents=True)
        state = MemoryState()
        state.put("management_worktree", "s-task", {
            "status": worktree_status, "path": str(path), "branch": None, "baseHead": "b" * 40,
            "sessionId": "s", "ownedPaths": ["owned.txt"], "driverDigest": "d" * 64,
            "releaseOperationKey": self.KEY, "releaseRequest": {"stale": True}})
        state.put("management_operation", self.KEY, {
            "phase": operation_phase, "action": "release-worktree", "sessionId": operation_session,
            "payload": {"worktreeId": "s-task"}, "requestId": "r", "operationId": "o"})
        management = runner.Management.__new__(runner.Management)
        management.repo, management.worktrees = repo, worktrees
        management.state, management.repo_id = state, "repository"
        management.session, management.request, management.operation = session, "r", "o"
        management.owned = ["owned.txt"]
        management.config = {"driverDigest": "d" * 64, "runtimeOperationId": "runtime"}
        management.key = "f" * 64
        management.payload = {"worktreeId": "s-task", "operationKey": self.KEY}
        management.text = lambda target, *args: (
            common_dir if "--git-common-dir" in args else "b" * 40)
        return management, state, path

    def test_releasing_without_a_directory_resolves_to_released(self):
        with tempfile.TemporaryDirectory(prefix="ordinary-recover-released-") as temporary:
            management, state, _path = self._fixture(temporary)
            result = management.recover_worktree()
            self.assertEqual("released", result["resolution"])
            self.assertEqual("releasing", result["previousWorktreeStatus"])
            self.assertFalse(result["directoryPresent"])
            self.assertFalse(result["formalAcceptance"] or result["commitPerformed"])
            record = state.get("management_worktree", "s-task")["payload"]
            self.assertEqual("released", record["status"])
            self.assertNotIn("releaseRequest", record)
            self.assertEqual("unknown", state.get("management_operation", self.KEY)["payload"]["phase"])

    def test_present_directory_owned_by_this_repository_resolves_to_ready(self):
        with tempfile.TemporaryDirectory(prefix="ordinary-recover-ready-") as temporary:
            repo = Path(temporary) / "repository"
            management, state, _path = self._fixture(
                temporary, worktree_status="ready", operation_phase="running",
                present=True, common_dir=str(repo / ".git"))
            result = management.recover_worktree()
            self.assertEqual("ready", result["resolution"])
            self.assertTrue(result["directoryPresent"])
            self.assertEqual("ready", state.get("management_worktree", "s-task")["payload"]["status"])

    def test_foreign_directory_and_foreign_operation_are_both_refused(self):
        with tempfile.TemporaryDirectory(prefix="ordinary-recover-refuse-") as temporary:
            management, state, _path = self._fixture(
                temporary, worktree_status="ready", present=True,
                common_dir=str(Path(temporary) / "elsewhere" / ".git"))
            with self.assertRaises(ValueError):
                management.recover_worktree()
            self.assertEqual("ready", state.get("management_worktree", "s-task")["payload"]["status"],
                             "a refused recovery must not move the record")

        with tempfile.TemporaryDirectory(prefix="ordinary-recover-session-") as temporary:
            management, _state, _path = self._fixture(temporary, operation_session="other")
            with self.assertRaises(ValueError):
                management.recover_worktree()

    def test_recovery_never_claims_the_interrupted_work_was_completed(self):
        with tempfile.TemporaryDirectory(prefix="ordinary-recover-honest-") as temporary:
            management, _state, _path = self._fixture(temporary)
            result = management.recover_worktree()
            for key in ("formalAcceptance", "commitPerformed", "pushPerformed"):
                self.assertFalse(result[key])
            self.assertNotIn("nativeTerminal", result)


class FlowBridgeTests(unittest.TestCase):
    """The management chain must hand Flow a product-only current source."""

    def _fixture(self, temporary, *, files, tracked, owned):
        repo = Path(temporary) / "repository"
        worktrees = repo / ".jenkins/worktrees"
        worktrees.mkdir(parents=True)
        root = worktrees / "s-task"
        root.mkdir()
        management = runner.Management.__new__(runner.Management)
        management.repo, management.worktrees = repo, worktrees
        management.state, management.repo_id = MemoryState(), "repository"
        management.session, management.request, management.operation = "s", "r", "o"
        management.owned = list(owned)
        management.config = {"driverDigest": "d" * 64, "runtimeOperationId": "runtime"}
        management.payload = {
            "worktreeId": "s-task",
            "flowIdentity": {"sessionId": "reader", "requestId": "r", "generation": 1},
            "coverage": {"template": "editor_product_development",
                         "requiredTests": ["some::case"], "noDefaultFeatures": True}}
        management.wt = lambda reference, **kwargs: ({"path": str(root), "sessionId": "s"}, root)
        management.source = lambda target, **kwargs: {
            "head": "a" * 40, "files": dict(files), "diffSha256": "b" * 64, "sourceDigest": "c" * 64}
        management.git = lambda target, *args, **kwargs: SimpleNamespace(
            stdout=b"".join(name.encode() + b"\0" for name in sorted(tracked)))
        management.build_root = lambda **kwargs: r"E:\cargo-targets"
        return management

    def test_flow_preparation_excludes_checkout_control_from_product_input(self):
        with tempfile.TemporaryDirectory(prefix="ordinary-flow-bridge-") as temporary:
            management = self._fixture(
                temporary,
                files={".jenkins/README.md": "1" * 64, ".codex/note.md": "2" * 64,
                       "src/main.rs": "3" * 64, "src/new.rs": "4" * 64},
                tracked={"src/main.rs"},
                owned=["src/main.rs", ".jenkins/README.md"])
            seen = {}

            def capture(action, payload, state, repo_root, **kwargs):
                seen.update(payload)
                self.assertEqual("register", action)
                return {"status": "registered", "currentSourceRef": "e" * 64}

            import tools.jenkins.current_source as current_source_module
            with patch.object(current_source_module, "handle", side_effect=capture):
                result = management.prepare_flow()

            self.assertEqual({"src/main.rs": "3" * 64, "src/new.rs": "4" * 64},
                             seen["currentHashes"], "checkout control must not become product input")
            self.assertEqual(["src/main.rs"], seen["ownedPaths"],
                             "a control owned path cannot be handed to Flow as owned product source")
            self.assertEqual(["src/new.rs"], seen["untrackedPaths"])
            self.assertEqual(str(management.worktrees / "s-task"), seen["sourceRoot"])
            self.assertEqual("s-task", seen["worktreeId"])
            self.assertEqual("reader", seen["identity"]["sessionId"])
            self.assertEqual("e" * 64, result["currentSourceRef"])
            self.assertEqual("prepared", result["status"])
            self.assertFalse(result["formalAcceptance"] or result["commitPerformed"])

    def test_prepared_flow_preserves_casefold_tracked_controls_and_frozen_external(self):
        from tools.jenkins import current_source, managed_worktree, product_inputs
        control = next(iter(product_inputs.COMPILER_CONTROL_PATHS))
        external = [{"name": "zr_vm", "currentHashes": {"vendor/v6/lib.rs": "7" * 64}}]
        prepared = {"currentHashes": {"cargo.toml": "1" * 64, "src/main.rs": "2" * 64,
                    "old-new.rs": None, control: "3" * 64}, "compilerControlInputs": {control: "3" * 64},
                    "externalInputs": external}
        with tempfile.TemporaryDirectory(prefix="ordinary-prepared-flow-") as temporary:
            management = self._fixture(temporary, files={"Cargo.toml": "1" * 64, "src/main.rs": "2" * 64,
                control: "3" * 64}, tracked={"Cargo.toml", "src/main.rs", control, "old-new.rs"},
                owned=["src/main.rs"])
            management.payload.update(worktreePreparationRef="4" * 64, patchOperationKeys=["5" * 64])
            seen = {}
            def register(action, payload, *args, **kwargs):
                seen.update(payload)
                return {"status": "registered", "currentSourceRef": "e" * 64}
            with patch.object(managed_worktree, "preparation", return_value={"payload": prepared}), \
                    patch.object(current_source, "handle", side_effect=register):
                management.prepare_flow()
            self.assertEqual([], seen["untrackedPaths"])
            self.assertEqual(prepared["currentHashes"], seen["currentHashes"])
            self.assertEqual(external, seen["externalInputs"])
            self.assertEqual(["5" * 64], seen["patchOperationKeys"])
            self.assertEqual({control: "3" * 64}, seen["compilerControlInputs"])

    def test_flow_preparation_refuses_when_only_control_paths_are_owned(self):
        with tempfile.TemporaryDirectory(prefix="ordinary-flow-control-only-") as temporary:
            management = self._fixture(
                temporary, files={".jenkins/README.md": "1" * 64},
                tracked={".jenkins/README.md"}, owned=[".jenkins/README.md"])
            with self.assertRaisesRegex(ValueError, "owned product source"):
                management.prepare_flow()

    def test_flow_preparation_requires_an_explicit_coverage_template(self):
        with tempfile.TemporaryDirectory(prefix="ordinary-flow-coverage-") as temporary:
            management = self._fixture(
                temporary, files={"src/main.rs": "3" * 64},
                tracked={"src/main.rs"}, owned=["src/main.rs"])
            management.payload.pop("coverage")
            with self.assertRaisesRegex(ValueError, "coverage template"):
                management.prepare_flow()


class FixedEditorPreviewTests(unittest.TestCase):
    def payload(self):
        return {"action": runner.PREVIEW_ACTION, "repositoryId": "r", "sessionId": "s",
                "requestId": "q", "operationId": "o", "driverDigest": "d" * 64,
                "runtimeOperationId": "runtime", "ownedPaths": ["zircon_editor"],
                "productDescriptor": str(runner.PREVIEW_REPORTS / "current-product.json"),
                "productDescriptorSha256": "a" * 64}

    def terminal(self):
        return {"schema": "current-unreal-ordinary-window-preview/v1", "status": "preview-pixel-verified",
                "productDescriptor": runner.fixed_editor_preview_recipe(self.payload())["descriptor"],
                "pixelFrameVerified": True, "failure": None, "formalAcceptance": False,
                "wholeUiAccepted": False, "sourceCopiesCreated": False,
                "existingDesktop": {"windowStation": "WinSta0", "desktop": "Default", "foregroundWindow": 1},
                "cleanup": [{"waitedForExit": True, "exitCode": 0}],
                "auxiliaryCleanup": [{"waitedForExit": True, "exitCode": 0}]}

    def native(self, command):
        return {"command": command, "cwd": str(ROOT), "pid": 1, "creationTime": "1", "rootExitCode": 0,
                "processTree": {"scope": "retained-windows-job", "terminal": True, "nativeWaiterJoined": True,
                                "pipes": {"stdout": {"eof": True}, "stderr": {"eof": True}},
                                "pipeEOF": True, "readersFinished": True},
                "nativeJobEvidence": {"records": {"afterTerminal": {"activeProcesses": 0},
                                                   "afterClose": {"activeProcesses": 0}}, "errors": []},
                "terminalErrors": [], "cleanup": {"attempted": False}}

    def test_recipe_exposes_only_the_two_descriptor_arguments(self):
        payload = self.payload()
        recipe = runner.fixed_editor_preview_recipe(payload)
        self.assertEqual(recipe["command"][:5],
                         [str(runner.PREVIEW_PYTHON), "-B", "-X", "utf8", str(runner.PREVIEW_SCRIPT)])
        self.assertEqual(recipe["command"][5:],
                         ["--product-descriptor", payload["productDescriptor"],
                          "--product-descriptor-sha256", payload["productDescriptorSha256"]])
        self.assertEqual(runner.PREVIEW_TIMEOUT_SECONDS, 900)
        self.assertEqual(entry.AUTH[runner.PREVIEW_ACTION], "validation")
        self.assertEqual(runner.AUTH[runner.PREVIEW_ACTION], "validation")
        self.assertEqual(entry.MANAGED_WORKTREE_CAPABILITY, runner.MANAGED_WORKTREE_CAPABILITY)
        self.assertNotIn(runner.PREVIEW_ACTION, runner.MANAGED_WORKTREE_CAPABILITY["actions"])

    def test_recipe_rejects_execution_overrides_and_invalid_descriptor_inputs(self):
        for field in ("command", "argv", "script", "interpreter", "environment", "output",
                      "timeoutSeconds", "sourceView", "externalRoots", "buildRoot", "batch"):
            with self.subTest(field=field), self.assertRaises(ValueError):
                runner.fixed_editor_preview_recipe({**self.payload(), field: "override"})
        for key, value in (("productDescriptor", "relative.json"),
                           ("productDescriptor", str(ROOT / "outside-task.json")),
                           ("productDescriptorSha256", "a" * 40), ("productDescriptorSha256", "A" * 64)):
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                runner.fixed_editor_preview_recipe({**self.payload(), key: value})

    def test_entry_runs_the_same_strict_schema_before_submission(self):
        from types import SimpleNamespace
        from unittest.mock import patch
        module = SimpleNamespace(route_request_context=lambda *a, **k: (dict(a[3]), None),
                                 fixed_editor_preview_recipe=runner.fixed_editor_preview_recipe)
        context = {"repo": ROOT, "module": module, "manager": object(),
                   "config": {"driverDigest": "d" * 64, "runtimeOperationId": "runtime"}}
        with patch.object(entry, "authorize"), patch.dict(entry.os.environ, {"ZIRCON_JENKINS_SESSION_ID": ""}):
            self.assertEqual(entry.input_payload(context, self.payload()), self.payload())
            with self.assertRaises(ValueError):
                entry.input_payload(context, {**self.payload(), "command": ["arbitrary"]})

    def test_native_gate_requires_this_command_join_both_eof_and_no_forced_cleanup(self):
        command = runner.fixed_editor_preview_recipe(self.payload())["command"]
        ref = {"path": "native-terminal.json", "sha256": "b" * 64}
        self.assertTrue(runner.fixed_editor_preview_native(self.native(command), ref, command, ROOT)["naturalSuccess"])
        for kind in ("stdout", "stderr", "job", "waiter", "command", "cleanup", "exit"):
            native = self.native(command)
            if kind in {"stdout", "stderr"}:
                native["processTree"]["pipes"][kind]["eof"] = False
            elif kind == "job":
                native["nativeJobEvidence"]["records"]["afterClose"]["activeProcesses"] = 1
            elif kind == "waiter":
                native["processTree"]["nativeWaiterJoined"] = False
            elif kind == "command":
                native["command"] = ["another.exe"]
            elif kind == "cleanup":
                native["cleanup"]["attempted"] = True
            else:
                native["rootExitCode"] = 1
            with self.subTest(kind=kind):
                self.assertFalse(runner.fixed_editor_preview_native(native, ref, command, ROOT)["naturalSuccess"])

    def test_receipt_requires_this_descriptor_actual_desktop_pixels_and_all_owned_exits(self):
        descriptor = runner.fixed_editor_preview_recipe(self.payload())["descriptor"]
        runner.fixed_editor_preview_receipt(self.terminal(), descriptor, {"pixelFrameVerified": True})
        for kind in ("descriptor", "desktop", "foreground", "pixel", "cleanup", "auxiliary", "failure", "formal"):
            terminal, pixel = self.terminal(), {"pixelFrameVerified": True}
            if kind == "descriptor":
                terminal["productDescriptor"] = {**descriptor, "sha256": "f" * 64}
            elif kind == "desktop":
                terminal["existingDesktop"]["desktop"] = "Other"
            elif kind == "foreground":
                terminal["existingDesktop"]["foregroundWindow"] = 0
            elif kind == "pixel":
                pixel["pixelFrameVerified"] = False
            elif kind == "cleanup":
                terminal["cleanup"][0]["waitedForExit"] = False
            elif kind == "auxiliary":
                terminal["auxiliaryCleanup"] = []
            elif kind == "failure":
                terminal["failure"] = {"message": "failed"}
            else:
                terminal["formalAcceptance"] = True
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                runner.fixed_editor_preview_receipt(terminal, descriptor, pixel)




class UnlaunchedFlowRecoveryTests(unittest.TestCase):
    """The fixed adapter uses real cancellation/resource policy on in-memory State."""

    class State(MemoryState):
        def __init__(self):
            super().__init__()
            self.requests = {}

        def put(self, domain, key, payload, expected_version=None, *, connection=None):
            value = super().put(domain, key, payload, expected_version, connection=connection)
            self.rows[(domain, key)]["domain"] = value["domain"] = domain
            return value

        def get_request(self, repository, session, request, *, connection=None):
            return copy.deepcopy(self.requests.get((repository, session, request)))

        def transition_request(self, repository, session, request, generation, before, after, evidence,
                               *, connection=None):
            row = self.requests[(repository, session, request)]
            if row["generation"] != generation or row["status"] != before:
                raise AssertionError("Original request transition changed")
            row["status"] = after
            return copy.deepcopy(row)

    def setUp(self):
        from tools.jenkins.contracts import JenkinsError
        from tools.jenkins.resources import Capacity, ResourceManager
        from tools.jenkins.workflow import execution
        self.error, self.execution_module = JenkinsError, execution
        self.state = state = self.State()
        source, driver, prep = "a" * 64, "b" * 64, "c" * 64
        recipe = {"preparationKey": prep, "buildRoot": "E:/cargo-targets"}
        recipe_ref = preparation.digest(recipe)
        key = execution.ExecutionKey(source, prep, recipe_ref, driver)
        eid = preparation.digest({"identity": key.digest, "ordinal": 1})[:32]
        identity = {"repositoryId": "repo", "sessionId": "reader", "requestId": "task",
                    "attemptId": "task-1", "generation": 1, "sourceInputDigest": source,
                    "coverageDigest": "d" * 64, "stageImplementationDigest": driver}
        self.consumer = consumer = "reader:task:task-1"
        intent = {"identity": identity, "worktreeId": "wt", "runtimeOperationId": "old",
                  "ownedPaths": ["owned.rs"], "buildRoot": "E:/cargo-targets",
                  "authorizationSnapshot": "e" * 64}
        self.reference = reference = preparation.digest(intent)
        state.put("current_source_intent", reference, intent)
        prepared = {"identity": identity, "sealedInputRef": source,
                    "currentSourceRef": reference, "sourcePreparationRef": "current-source-" + reference}
        state.put("current_source_preparation", reference, {"status": "prepared", "result": prepared})
        self.binding = binding = {"digest": driver, "generation": 1, "runtimeOperationId": "old"}
        state.put("recipe_plan", recipe_ref, {"recipe": recipe, "sourceRef": source, "driver": binding})
        state.put("management_worktree", "wt", {"status": "ready", "repositoryId": "repo", "sessionId": "writer",
            "runtimeOperationId": "old", "ownedPaths": ["owned.rs"], "writerPrincipal": "synthetic-owner",
            "writerAuthorizationSnapshot": "e" * 64, "path": "E:/fixture/wt"})
        state.put("workflow", consumer, {"identity": identity, "repositoryId": "repo", "executionId": eid,
            "recipeRef": recipe_ref, "sourceDigest": source, "coverageDigest": identity["coverageDigest"],
            "driverDigest": driver, "generation": 1, "ownedPaths": ["owned.rs"], "status": "executing"})
        state.put("execution", eid, {"executionKey": key.__dict__, "executionKeyDigest": key.digest,
            "ordinal": 1, "sourceDigest": source, "sealedInputRef": source, "preparationKey": prep,
            "recipeDigest": recipe_ref, "recipeRef": recipe_ref, "driverDigest": driver, "driver": binding,
            "operationId": "old", "generation": 1, "coverageDigest": identity["coverageDigest"],
            "status": "preparing", "consumers": [consumer], "refs": 1, "reservationKey": "reservation"})
        state.put("execution_consumer", eid + ":" + consumer,
                  {"executionId": eid, "consumerId": consumer, "status": "active"})
        state.put("execution_binding", preparation.digest({"consumer": consumer, "executionKey": key.digest}),
                  {"executionId": eid, "consumerId": consumer})
        state.put("resource_reservation", "reservation", {"owner": eid, "status": "active"})
        state.put("pool_writer_hold", prep, {"owner": eid, "reservationKey": "reservation", "status": "active"})
        from tools.jenkins.current_source import _request
        state.requests[("repo", "reader", "task")] = {"repositoryId": "repo", "sessionId": "reader",
            "requestId": "task", "generation": 1, "status": "running", "payload": _request(intent, prepared)}
        registered = state.requests[("repo", "reader", "task")]
        registered["payloadDigest"] = preparation.digest(registered["payload"])
        recovery = {"schemaVersion": 1, "kind": "cancelled-before-native-launch-v1", "executionId": eid,
            "recipeRef": recipe_ref, "originalIdentity": identity, "terminalRef": {"path": "pinned-proof"},
            "successor": {"originalLogicalTaskId": "task", "requestId": "task-r2", "attemptId": "task-2",
                          "attemptOrdinal": 2, "generation": 1, "runtimeOperationId": "current"}}
        self.actor = SimpleNamespace(state=state, repo=Path("E:/fixture"), repo_id="repo",
            session="writer", owned=["owned.rs"], key="f" * 64, input_digest="1" * 64,
            config={"driverDigest": driver, "runtimeOperationId": "current"}, manager=Mock(),
            payload={"unlaunchedRecovery": recovery, "worktreeId": "wt", "currentSourceRef": reference,
                     "buildRoot": "E:/cargo-targets"})
        self.eid, self.prep = eid, prep
        self.owner = {"operationId": "current", "driverDigest": driver}
        self.proof = {"kind": "observed-terminal-original"}
        self.locked = False

        @contextmanager
        def retained_execution_lock(_state, actual_eid):
            self.assertEqual(eid, actual_eid)
            self.assertFalse(self.locked, "Execution lock must never be acquired reentrantly")
            self.locked = True
            try:
                yield
            finally:
                self.locked = False

        # Use the existing Engine.cancel and ResourceManager.release_unlaunched;
        # no fake terminal or resource release is supplied by the fixture.
        engine = execution.ExecutionEngine.__new__(execution.ExecutionEngine)
        engine.state, engine.repo_root = state, self.actor.repo
        engine.approved_build_root = SimpleNamespace(path=Path("E:/cargo-targets"))
        engine.resources = ResourceManager(state, Capacity(4, 8 * 1024**3, 32 * 1024**3))
        self.real_cancel = engine.cancel
        self.cancel = Mock(wraps=engine.cancel)
        engine.cancel = self.cancel
        self.addCleanup(patch.stopall)
        patch.object(execution, "execution_lock", retained_execution_lock).start()
        patch.object(execution, "canonical_build_root", return_value=engine.approved_build_root).start()
        def cancellation_engine(authority, **_kwargs):
            engine.state = authority
            return engine
        patch.object(execution, "ExecutionEngine", side_effect=cancellation_engine).start()
        self.terminal = patch.object(preparation, "_unlaunched_terminal", return_value=(self.proof, self.owner)).start()
        patch("tools.jenkins.deployment.drain_observations.owner_observation", return_value=self.owner).start()

    def test_old_runtime_cancellation_retires_reader_with_explicit_successor(self):
        from tools.jenkins.managed_worktree_lifetime import reader_retired
        capacity_rows, capacity_requests = copy.deepcopy(self.state.rows), copy.deepcopy(self.state.requests)
        original = copy.deepcopy(self.state.get("execution", self.eid)["payload"])
        request = self.state.requests[("repo", "reader", "task")]
        self.assertNotIn("runtimeOperationId", request["payload"], "Use the real current-source request producer")
        for explicit in ("different-runtime", None):
            with self.subTest(explicitRuntime=explicit):
                request["payload"]["runtimeOperationId"] = explicit
                rows, requests = copy.deepcopy(self.state.rows), copy.deepcopy(self.state.requests)
                with self.assertRaises(self.error):
                    preparation.retire_unlaunched_flow_reader(self.actor)
                self.assertEqual(rows, self.state.rows)
                self.assertEqual(requests, self.state.requests)
                self.cancel.assert_not_called()
        del request["payload"]["runtimeOperationId"]
        registered_payload, registered_digest = copy.deepcopy(request["payload"]), request["payloadDigest"]
        result = preparation.retire_unlaunched_flow_reader(self.actor)
        self.assertEqual(registered_payload, request["payload"])
        self.assertEqual(registered_digest, request["payloadDigest"])
        self.assertNotIn("runtimeOperationId", request["payload"])
        self.assertEqual({"source": "validated-current-source-intent", "runtimeOperationId": "old",
                          "originalPayloadDigest": registered_digest, "registeredPayloadHasRuntime": False},
                         result["requestRuntimeBinding"])
        self.assertTrue(result["cancelledBeforeNativeLaunch"])
        self.assertFalse(result["successorSubmitted"])
        self.assertEqual("task-2", result["successor"]["attemptId"])
        self.cancel.assert_called_once_with(self.eid, self.consumer, cancel_request=True)
        current = self.state.get("execution", self.eid)["payload"]
        self.assertEqual("cancelled", current["status"])
        for name in ("driver", "operationId", "recipeRef", "executionKey", "sealedInputRef"):
            self.assertEqual(original[name], current[name])
        self.assertEqual("durable_unlaunched",
                         self.state.get("resource_reservation", "reservation")["payload"]["releaseEvidence"])
        self.assertTrue(reader_retired(self.state, self.reference))
        self.assertEqual([], self.state.list("native_job"))
        self.assertEqual([], self.state.list("execution_launch"))

        # Model recover_flow's existing cancel_request=False outcome, then retire
        # it without another cancellation, request transition or native proof.
        from tools.jenkins.current_source import _request
        self.state.rows = copy.deepcopy(capacity_rows)
        self.state.requests = copy.deepcopy(capacity_requests)
        self.cancel.reset_mock()
        intent = copy.deepcopy(self.state.get("current_source_intent", self.reference)["payload"])
        prepared = copy.deepcopy(self.state.get("current_source_preparation", self.reference)["payload"]["result"])
        intent["runtimeOperationId"] = "current"
        intent["identity"] = {name: value for name, value in intent["identity"].items()
                              if name not in {"sourceInputDigest", "coverageDigest"}}
        self.assertEqual(6, len(intent["identity"]))
        self.assertEqual(8, len(prepared["identity"]))
        reference = preparation.digest(intent)
        prepared.update(currentSourceRef=reference, sourcePreparationRef="current-source-" + reference)
        del self.state.rows[("current_source_intent", self.reference)]
        del self.state.rows[("current_source_preparation", self.reference)]
        self.state.put("current_source_intent", reference, intent)
        self.state.put("current_source_preparation", reference, {"status": "prepared", "result": prepared})
        recipe_ref = self.state.get("execution", self.eid)["payload"]["recipeRef"]
        for domain, key, changes in (
            ("management_worktree", "wt", {"runtimeOperationId": "current", "driverDigest": "b" * 64}),
            ("recipe_plan", recipe_ref, {"driver": {**self.binding, "runtimeOperationId": "current"}}),
            ("execution", self.eid, {"driver": {**self.binding, "runtimeOperationId": "current"},
                "operationId": "current", "buildRoot": "E:/cargo-targets", "status": "pending", "pendingReason": "capacity"})):
            row = self.state.get(domain, key)
            payload = {**row["payload"], **changes}
            if domain == "execution":
                del payload["reservationKey"]
            self.state.put(domain, key, payload, expected_version=row["version"])
        del self.state.rows[("resource_reservation", "reservation")]
        del self.state.rows[("pool_writer_hold", self.prep)]
        self.state.put("resource_reservation", "foreign-history", {"owner": "foreign", "status": "active"})
        self.state.put("resource_queue", self.eid + ":" + self.eid, {"owner": self.eid,
            "writerKey": self.prep, "buildRoot": "E:/cargo-targets", "status": "waiting"})
        request = self.state.requests[("repo", "reader", "task")]
        request.update(status="running", payload=_request(intent, prepared))
        request["payloadDigest"] = preparation.digest(request["payload"])
        registered = copy.deepcopy(request)
        self.real_cancel.__self__.state = self.state
        self.real_cancel(self.eid, self.consumer)
        self.assertEqual("running", request["status"], "Natural consumer cancellation does not rewrite its request")
        self.assertTrue(self.state.get("execution", self.eid)["payload"]["cancelledBeforeLaunch"])
        # These are the existing failed collector rows, not outputs of retirement.
        flow = self.state.get("workflow", self.consumer)
        evidence = {"status": "complete", "kind": "coordinator-preparation-preserved", "sourcePreserved": True,
                    "canonicalSourceModified": False}
        self.state.put("workflow", self.consumer, {**flow["payload"], "status": "failed", "recovery": evidence},
                       expected_version=flow["version"])
        self.state.transition_request("repo", "reader", "task", 1, "running", "failed", {"recovery": evidence})
        outcome_key = preparation.digest(["repo", "reader", "task"])
        self.state.put("request_result", outcome_key, {"repositoryId": "repo", "sessionId": "reader", "requestId": "task",
            "generation": 1, "status": "failed", "result": {"recovery": evidence}})
        self.actor.payload = {"action": "retire-worktree-reader", "repositoryId": "repo", "sessionId": "writer",
            "requestId": "retire-existing-capacity-reader", "operationId": "retire-existing-capacity-reader",
            "driverDigest": "b" * 64, "runtimeOperationId": "current", "ownedPaths": ["owned.rs"],
            "worktreeId": "wt", "currentSourceRef": reference, "buildRoot": "E:/cargo-targets",
            "unlaunchedRecovery": {"schemaVersion": 1, "kind": "cancelled-before-resource-admission-v1", "phase": "retire",
                "executionId": self.eid, "recipeRef": recipe_ref, "originalIdentity": prepared["identity"],
                "terminalRef": {"path": "actual-two-root-terminal-proof"}}}
        self.actor.key = preparation.digest({name: self.actor.payload[name] for name in
                                            ("repositoryId", "sessionId", "requestId", "operationId")})
        self.actor.input_digest = preparation.digest(self.actor.payload)
        rows, requests = copy.deepcopy(self.state.rows), copy.deepcopy(self.state.requests)
        # Supply only the actual HTTP/Home boundary; locked State is evaluated normally.
        with patch.object(preparation, "_capacity_terminal") as terminal, \
                patch.object(self.state, "transition_request", wraps=self.state.transition_request) as transition:
            terminal.side_effect = self.error("unlaunched_recovery_cps_unproven", "Original CPS is not terminal")
            with self.assertRaises(self.error):
                preparation.retire_unlaunched_flow_reader(self.actor)
            self.assertEqual(rows, self.state.rows)
            self.assertEqual(requests, self.state.requests)
            self.assertFalse(reader_retired(self.state, reference))
            terminal.side_effect, terminal.return_value = None, {"kind": "observed-two-root-terminal"}
            retired = preparation.retire_unlaunched_flow_reader(self.actor)
            transition.assert_not_called()
        self.cancel.assert_not_called()
        self.assertTrue(retired["readerRetired"])
        self.assertFalse(retired["cancellationPerformed"])
        self.assertFalse(retired["nativeTerminationProofCreated"])
        self.assertFalse(retired["resourceReservationReleased"])
        self.assertFalse(retired["successorSubmitted"])
        self.assertEqual(("failed", "failed"), (retired["requestStatus"], retired["workflowStatus"]))
        self.assertEqual(requests, self.state.requests)
        self.assertEqual(registered["payload"], request["payload"])
        self.assertEqual(registered["payloadDigest"], request["payloadDigest"])
        retirement = self.state.get("managed_worktree_reader_retirement", reference)["payload"]
        self.assertEqual(rows, {key: value for key, value in self.state.rows.items()
                               if key != ("managed_worktree_reader_retirement", reference)})
        self.assertIn("request_result", {pin["domain"] for pin in retirement["terminalPins"]})
        self.assertNotIn("management_worktree", {pin["domain"] for pin in retirement["terminalPins"]})
        self.assertTrue(reader_retired(self.state, reference))
        # A later legal worktree preparation/writer reservation cannot revive this old reader.
        worktree = self.state.get("management_worktree", "wt")
        self.state.put("management_worktree", "wt", {**worktree["payload"], "buildRoot": "D:/cargo-targets"},
                       expected_version=worktree["version"])
        self.state.put("managed_worktree_writer", "wt", {"status": "released", "operationKey": "later-writer"})
        self.assertTrue(reader_retired(self.state, reference))

    def test_native_host_never_releases_holds_or_retires_reader(self):
        self.state.put("execution_host", self.eid, {"executionId": self.eid, "status": "running"})
        with self.assertRaises(self.error):
            preparation.retire_unlaunched_flow_reader(self.actor)
        self.cancel.assert_not_called()
        self.assertEqual("active", self.state.get("pool_writer_hold", self.prep)["payload"]["status"])
        self.assertIsNone(self.state.get("managed_worktree_reader_retirement", self.reference))

    def test_unproven_worker_or_pipeline_is_pending_before_any_cancellation(self):
        self.terminal.side_effect = self.error("unlaunched_recovery_worker_unproven", "Original birth is unproven")
        with self.assertRaises(self.error):
            preparation.retire_unlaunched_flow_reader(self.actor)
        self.cancel.assert_not_called()
        self.assertIsNone(self.state.get("unlaunched_flow_recovery", self.eid))
        self.assertEqual("preparing", self.state.get("execution", self.eid)["payload"]["status"])

    def test_host_race_is_actual_cancel_requested_and_keeps_reader_and_holds(self):
        def racing_cancel(*args, **kwargs):
            self.state.put("execution_host", self.eid, {"executionId": self.eid, "status": "running"})
            return self.real_cancel(*args, **kwargs)
        self.cancel.side_effect = racing_cancel
        with self.assertRaises(self.error):
            preparation.retire_unlaunched_flow_reader(self.actor)
        self.assertEqual("cancel_requested", self.state.get("execution", self.eid)["payload"]["status"])
        self.assertEqual("active", self.state.get("pool_writer_hold", self.prep)["payload"]["status"])
        self.assertIsNone(self.state.get("managed_worktree_reader_retirement", self.reference))

    def test_exact_replay_does_not_cancel_or_change_lineage_again(self):
        first = preparation.retire_unlaunched_flow_reader(self.actor)
        rows = copy.deepcopy(self.state.rows)
        second = preparation.retire_unlaunched_flow_reader(self.actor)
        self.assertEqual(first, second)
        self.assertEqual(rows, self.state.rows)
        self.assertEqual(1, self.cancel.call_count)
        self.actor.payload["unlaunchedRecovery"]["successor"]["attemptId"] = "task-other-2"
        with self.assertRaises(self.error):
            preparation.retire_unlaunched_flow_reader(self.actor)
        self.assertEqual(rows, self.state.rows)
        self.actor.payload["unlaunchedRecovery"]["successor"]["attemptId"] = "task-2"
        actor = runner.Management.__new__(runner.Management)
        actor.__dict__.update(vars(self.actor))
        actor.action, actor.query = "retire-worktree-reader", False
        actor.request, actor.operation = "recover-original", "recover-original-operation"
        self.state.put("management_operation", actor.key, {"phase": "unknown",
            "inputDigest": actor.input_digest, "payload": copy.deepcopy(actor.payload)})

        @contextmanager
        def original_management_lock(path, **_kwargs):
            self.assertEqual("management-operation-" + actor.key + ".lock", path.name)
            yield

        with patch("tools.jenkins.state.locks.process_lock", original_management_lock), \
                patch.object(runner, "source_helper", return_value=preparation):
            reconciled = actor.run()
            completed = copy.deepcopy(self.state.rows)
            self.assertEqual(reconciled, actor.run())
        self.assertTrue(reconciled["cancelledBeforeNativeLaunch"])
        self.assertEqual("complete", self.state.get("management_operation", actor.key)["payload"]["phase"])
        self.assertEqual(completed, self.state.rows)
        self.assertEqual(1, self.cancel.call_count)

    def test_legal_worktree_rebind_and_shared_hold_reuse_do_not_revive_old_reader(self):
        from tools.jenkins.managed_worktree_lifetime import reader_retired
        preparation.retire_unlaunched_flow_reader(self.actor)
        worktree = self.state.get("management_worktree", "wt")
        self.state.put("management_worktree", "wt", {**worktree["payload"], "runtimeOperationId": "current"},
                       expected_version=worktree["version"])
        hold = self.state.get("pool_writer_hold", self.prep)
        self.state.put("pool_writer_hold", self.prep, {"owner": "successor-execution",
            "reservationKey": "next-reservation", "status": "active"}, expected_version=hold["version"])
        self.assertTrue(reader_retired(self.state, self.reference))
        self.assertFalse(self.state.get("unlaunched_flow_recovery", self.eid)["payload"]["nativeTerminationProofCreated"])

    def test_existing_retire_action_keeps_ordinary_native_dispatch(self):
        actor = runner.Management.__new__(runner.Management)
        actor.action, actor.payload = "retire-worktree-reader", {}
        with patch("tools.jenkins.managed_worktree_lifetime.retire_reader", return_value={"native": True}) as native:
            self.assertEqual({"native": True}, actor._dispatch())
        native.assert_called_once_with(actor)




class CommittedWorktreeSuccessorTests(unittest.TestCase):
    """A real State CAS retains the physical tree and durable creation anchors."""

    @contextmanager
    def fixture(self):
        from tools.jenkins.state import State
        with tempfile.TemporaryDirectory(prefix="ordinary-committed-successor-") as temporary:
            repo = Path(temporary) / "repository"
            wt = repo / ".jenkins/worktrees/writer-work"
            wt.mkdir(parents=True)
            (repo / ".git").mkdir()
            (wt / "owned.txt").write_bytes(b"old-owned\n")
            (wt / "dependency.txt").write_bytes(b"old-dependency\n")
            state = State(repo / ".jenkins/state/fixture.sqlite3")
            auth = state.authorize_session("repo", "writer", "windows:fixture-owner", ["owned.txt"],
                ["implementation", "validation"], {"source": "user", "task": "committed-worktree-fixture"})
            old_head, head, driver = "a" * 40, "b" * 40, "d" * 64
            old_source = snapshot(wt, old_head)
            prior_payload = {"action": "create-worktree", "repositoryId": "repo", "sessionId": "writer",
                "requestId": "original", "operationId": "create-original", "name": "work",
                "baseHead": old_head, "ownedPaths": ["owned.txt"], "driverDigest": driver,
                "runtimeOperationId": "old-runtime"}
            original_key = runner.sha(runner.encoded({n: prior_payload[n] for n in
                ("repositoryId", "sessionId", "requestId", "operationId")}))
            old_record = {"status": "ready", "repositoryId": "repo", "sessionId": "writer",
                "requestId": "original", "operationId": "create-original", "path": str(wt),
                "branch": "codex/jenkins/writer/work", "baseHead": old_head, "ownedPaths": ["owned.txt"],
                "driverDigest": driver, "runtimeOperationId": "old-runtime", "creationOperationKey": original_key,
                "writerPrincipal": auth["principal"], "writerAuthorizationSnapshot": auth["snapshotDigest"]}
            state.put("management_worktree", "writer-work", old_record, expected_version=0)
            original = {"action": "create-worktree", "phase": "complete", "payload": prior_payload,
                "inputDigest": runner.sha(runner.encoded(prior_payload)),
                "result": {**old_record, "status": "complete", "worktreeId": "writer-work",
                           "operationKey": original_key, "source": old_source}}
            state.put("management_operation", original_key, original, expected_version=0)
            # Uncommitted dependency bytes are preserved in the new full source anchor.
            (wt / "owned.txt").write_bytes(b"committed-owned\n")
            (wt / "dependency.txt").write_bytes(b"read-authorized-dirty-dependency\n")
            checksum = runner.sha((wt / "owned.txt").read_bytes())
            oid = hashlib.sha1(b"blob 16\0committed-owned\n").hexdigest()
            controls = repo / ".jenkins/state/controls"; controls.mkdir(parents=True)
            receipt_path = controls / "branch-owned-commit.115.response.json"
            archive_name = "branch-owned-commit.115.receipt.json"
            receipt = {"schemaVersion": 1, "status": "committed", "sourceRoot": str(wt),
                "branch": old_record["branch"], "headBefore": old_head, "sourceCommit": head,
                "request": {"sessionId": "writer", "requestId": "deliver-owned",
                    "operationId": "owned-commit", "sourceRoot": str(wt), "paths": ["owned.txt"]},
                "runtime": {"driverDigest": driver, "runtimeOperationId": "old-runtime"},
                "files": {"owned.txt": checksum}, "commitPaths": ["owned.txt"],
                "expectedGitObjects": {"owned.txt": oid}}
            receipt_raw = runner.encoded(receipt); receipt_path.write_bytes(receipt_raw)
            home = repo / ".jenkins/jenkins_home/jobs/zircon-management/builds/115"
            (home / "archive").mkdir(parents=True)
            (home / "archive" / archive_name).write_bytes(receipt_raw)
            (home / "build.xml").write_text("<flow-build><completed>true</completed><result>SUCCESS</result></flow-build>",
                                           encoding="utf-8")
            build = {"number": 115, "building": False, "result": "SUCCESS",
                "actions": [{"parameters": [{"name": k, "value": v} for k, v in {
                    "SESSION_ID": "writer", "REQUEST_ID": "deliver-owned", "OPERATION_ID": "owned-commit",
                    "ACTION": "commit-branch"}.items()]}],
                "artifacts": [{"relativePath": archive_name}]}
            retirement_key, reference = "e" * 64, "f" * 64
            state.put("current_source_intent", reference, {"worktreeId": "writer-work", "sourceRoot": str(wt)})
            state.put("managed_worktree_reader_retirement", reference,
                {"status": "retired", "currentSourceRef": reference, "retiredBy": retirement_key, "terminalPins": [
                    {"domain": "management_operation", "key": original_key, "version": 1,
                     "payloadDigest": preparation.digest(original)}]})
            state.put("management_operation", retirement_key, {"action": "retire-worktree-reader", "phase": "complete",
                "result": {"status": "complete", "readerRetired": True, "worktreeId": "writer-work", "currentSourceRef": reference}})
            state.put("current_source_intent", "9" * 64, {"worktreeId": "another-task"})
            actor = runner.Management.__new__(runner.Management)
            actor.repo, actor.worktrees, actor.state, actor.repo_id = repo, wt.parent, state, "repo"
            actor.session, actor.request, actor.operation, actor.action = "writer", "successor", "create-successor", "create-worktree"
            actor.owned = ["owned.txt"]
            actor.config = {"managedWorktree": runner.MANAGED_WORKTREE_CAPABILITY,
                            "driverDigest": driver, "runtimeOperationId": "current-runtime"}
            actor.payload = {"action": actor.action, "repositoryId": "repo", "sessionId": actor.session,
                "requestId": actor.request, "operationId": actor.operation, "worktreeId": "writer-work",
                "name": "work", "baseHead": head, "ownedPaths": actor.owned,
                "driverDigest": driver, "runtimeOperationId": "current-runtime",
                "resumeFromOperationKey": original_key,
                "committedSuccessor": {"schemaVersion": 1,
                    "commitReceiptRef": {"path": str(receipt_path), "sha256": runner.sha(receipt_raw), "bytes": len(receipt_raw)},
                    "readerRetirementOperationKey": retirement_key}}
            actor.key = runner.sha(runner.encoded({n: actor.payload[n] for n in
                ("repositoryId", "sessionId", "requestId", "operationId")}))
            actor.input_digest = runner.sha(runner.encoded(actor.payload))
            actor.manager = SimpleNamespace(_request=Mock(side_effect=lambda endpoint, **kw:
                (200, runner.encoded(build)) if "/api/json" in endpoint else (200, receipt_raw)))
            actor.worktree_resume_lineage = Mock(return_value={"kind": "verified-fixture-reboot"})
            reads = {"head": head, "parent": old_head}
            def text(root, *args):
                if args == ("rev-parse", head + "^{commit}"): return head
                if args == ("rev-parse", "--path-format=absolute", "--git-common-dir"): return str(repo / ".git")
                if args == ("symbolic-ref", "--short", "HEAD"): return old_record["branch"]
                if args == ("rev-parse", "HEAD"): return reads["head"]
                if args == ("rev-parse", head + "^"): return reads["parent"]
                raise AssertionError("Unexpected Git read: " + repr(args))
            actor.text = text
            def git(root, *args, **kw):
                self.assertIn(args[0], {"diff-tree", "ls-tree"}, "Successor cannot mutate Git")
                raw = b"owned.txt\0" if args[0] == "diff-tree" else ("100644 blob " + oid + "\towned.txt\0").encode()
                return SimpleNamespace(stdout=raw, returncode=0)
            actor.git = Mock(side_effect=git)
            actor.source = Mock(side_effect=lambda root: snapshot(root, reads["head"]))
            with patch.object(runner, "source_helper", return_value=preparation):
                yield SimpleNamespace(actor=actor, state=state, wt=wt, old_head=old_head, head=head,
                    original_key=original_key, old_record=old_record, original=original, reference=reference,
                    retirement_key=retirement_key, reads=reads)

    def test_committed_successor_preserves_dependencies_and_next_reboot_anchor(self):
        with self.fixture() as f:
            actor, state = f.actor, f.state
            # The current independent S4U owner is not the historical keeper runtime.
            from tools.jenkins.deployment import drain_observations
            retirement = state.get("management_operation", f.retirement_key)
            retired = state.get("managed_worktree_reader_retirement", f.reference)
            native_owner = {"operationId": "current-runtime", "driverDigest": actor.config["driverDigest"]}
            recovery_request = {"schemaVersion": 1, "kind": "cancelled-before-native-launch-v1",
                "executionId": "1" * 32, "recipeRef": "2" * 64, "originalIdentity": {"requestId": "product"},
                "terminalRef": {"path": "fixture-terminal", "sha256": "3" * 64},
                "successor": {"originalLogicalTaskId": "product", "requestId": "product-attempt2",
                              "runtimeOperationId": "current-runtime"}}
            recovery_digest = preparation.digest(recovery_request)
            retire_payload = {"unlaunchedRecovery": recovery_request}
            retire_digest = preparation.digest(retire_payload)
            proof = {"schemaVersion": 1, "kind": recovery_request["kind"], "status": "complete",
                "executionId": recovery_request["executionId"], "currentSourceRef": f.reference,
                "originalRuntimeOperationId": "old-runtime", "recoveredBy": f.retirement_key,
                "inputDigest": retire_digest, "recoveryDigest": recovery_digest,
                "successor": recovery_request["successor"], "serviceOwner": native_owner,
                "nativeLaunchIssued": False, "nativeTerminationProofCreated": False,
                "resourceReleaseEvidence": "durable_unlaunched", "terminalPins": retired["payload"]["terminalPins"],
                "worktreeWriter": {"sessionId": actor.session, "principal": f.old_record["writerPrincipal"],
                    "snapshotDigest": f.old_record["writerAuthorizationSnapshot"], "ownedPaths": actor.owned}}
            state.put("unlaunched_flow_recovery", recovery_request["executionId"], proof, expected_version=0)
            state.put("management_operation", f.retirement_key,
                {**retirement["payload"], "payload": retire_payload, "inputDigest": retire_digest,
                 "result": {**retirement["payload"]["result"], "cancelledBeforeNativeLaunch": True,
                    "executionId": recovery_request["executionId"], "recoveryDigest": recovery_digest,
                    "successor": recovery_request["successor"], "runtimeOperationId": native_owner["operationId"],
                    "driverDigest": native_owner["driverDigest"]}},
                expected_version=retirement["version"])
            state.put("managed_worktree_reader_retirement", f.reference,
                {**retired["payload"], "retirementKind": recovery_request["kind"], "requestStatus": "cancelled",
                 "acceptanceSynthesized": False, "recoveryExecutionId": recovery_request["executionId"],
                 "recoveryDigest": recovery_digest}, expected_version=retired["version"])
            preserved = {domain: state.list(domain) for domain in (
                "unlaunched_flow_recovery", "managed_worktree_reader_retirement", "management_operation")}
            actor.worktree_resume_lineage.side_effect = AssertionError("Historical keeper cannot represent current S4U")
            def observe(manager, *, expected):
                self.assertIs(manager, actor.manager)
                self.assertEqual(native_owner, expected)
                return expected
            with patch.object(drain_observations, "owner_observation", side_effect=observe) as owner:
                result = actor._dispatch()
            self.assertEqual(2, owner.call_count, "Verify the actual owner before and after source capture")
            actor.worktree_resume_lineage.assert_not_called()
            self.assertEqual(preserved, {domain: state.list(domain) for domain in preserved})
            actor.worktree_resume_lineage.side_effect = None
            self.assertEqual(1, actor.source.call_count, "Publish exactly the captured source, without rescanning it")
            record = state.get("management_worktree", "writer-work")["payload"]
            self.assertTrue(record["reusedPhysicalWorktree"])
            self.assertEqual(f.original_key, record["previousCreationOperationKey"])
            self.assertEqual(actor.key, record["creationOperationKey"])
            self.assertEqual(f.head, record["baseHead"])
            self.assertEqual((actor.request, actor.operation), (record["requestId"], record["operationId"]))
            self.assertEqual(runner.sha(b"read-authorized-dirty-dependency\n"), result["source"]["files"]["dependency.txt"])
            self.assertEqual(f.original, state.get("management_operation", f.original_key)["payload"])
            result = {**result, "operationKey": actor.key}
            state.put("management_operation", actor.key, {"phase": "complete", "action": "create-worktree",
                "payload": copy.deepcopy(actor.payload), "inputDigest": actor.input_digest, "result": copy.deepcopy(result)})
            previous_key = actor.key
            actor.request, actor.operation = "next-reboot", "create-next-reboot"
            actor.config = {**actor.config, "runtimeOperationId": "next-runtime"}
            actor.payload = {**actor.payload, "requestId": actor.request, "operationId": actor.operation,
                "runtimeOperationId": "next-runtime", "resumeFromOperationKey": previous_key}
            actor.payload.pop("committedSuccessor")
            actor.key = runner.sha(runner.encoded({n: actor.payload[n] for n in
                ("repositoryId", "sessionId", "requestId", "operationId")}))
            actor.input_digest = runner.sha(runner.encoded(actor.payload))
            reference, path, resumed = actor.resume_existing_worktree("writer-work", f.wt,
                state.get("management_worktree", "writer-work"), f.head, merge=False)
            self.assertEqual(("writer-work", f.wt), (reference, path))
            self.assertEqual(actor.key, resumed["creationOperationKey"])
            self.assertEqual("next-runtime", resumed["runtimeOperationId"])
            self.assertEqual(result, state.get("management_operation", previous_key)["payload"]["result"])
            self.assertEqual(f.original, state.get("management_operation", f.original_key)["payload"])
            self.assertEqual(1, actor.worktree_resume_lineage.call_count,
                             "Ordinary next-runtime metadata resume retains its original keeper path")

    def test_committed_successor_drift_is_rejected_before_capture_or_worktree_cas(self):
        from tools.jenkins.contracts import JenkinsError
        for drift in ("owned", "head", "parent", "receipt"):
            with self.subTest(drift=drift), self.fixture() as f:
                before = f.state.get("management_worktree", "writer-work")
                if drift == "owned": (f.wt / "owned.txt").write_bytes(b"foreign-edit\n")
                elif drift == "head": f.reads["head"] = "c" * 40
                elif drift == "parent": f.reads["parent"] = "c" * 40
                else: f.actor.payload["committedSuccessor"]["commitReceiptRef"]["sha256"] = "0" * 64
                with self.assertRaises(JenkinsError):
                    f.actor.resume_existing_worktree("writer-work", f.wt, before, f.head, merge=False)
                self.assertEqual(before, f.state.get("management_worktree", "writer-work"))
                f.actor.source.assert_not_called()

    def test_committed_successor_waits_for_its_related_reader(self):
        from tools.jenkins.contracts import JenkinsError
        with self.fixture() as f:
            before = f.state.get("management_worktree", "writer-work")
            row = f.state.get("managed_worktree_reader_retirement", f.reference)
            f.state.put("managed_worktree_reader_retirement", f.reference,
                        {**row["payload"], "status": "pending"}, expected_version=row["version"])
            with self.assertRaises(JenkinsError):
                f.actor.resume_existing_worktree("writer-work", f.wt, before, f.head, merge=False)
            self.assertEqual(before, f.state.get("management_worktree", "writer-work"))
            f.actor.source.assert_not_called()


class BranchCandidatePreparationTests(unittest.TestCase):
    @contextmanager
    def _candidate_fixture(self):
        """Isolated owner/State fixtures; CAS, admission, preparation and Cargo stay real."""
        from contextlib import ExitStack
        import shutil
        import subprocess
        import tomllib
        from tools.jenkins.resources import paths
        from tools.jenkins.workflow import planning

        actual = paths.canonical_build_root("E:/cargo-targets")
        parent = paths.physical_path_under(actual, os.environ.get("ZR_COMMIT_TEST_ROOT", tempfile.gettempdir()),
                                           allow_missing=False)
        with tempfile.TemporaryDirectory(prefix="branch-input-", dir=parent) as temporary:
            top = Path(temporary)
            repo, external = top / "repository", top / "zr_vm"
            repo.mkdir()
            (external / "binding/src").mkdir(parents=True)
            (repo / "src").mkdir()
            (repo / ".gitignore").write_text(".jenkins/\ntarget/\n", encoding="utf-8")
            (repo / "Cargo.toml").write_text(
                '[package]\nname="candidate_fixture"\nversion="0.0.0"\nedition="2021"\n'
                '[dependencies]\nvm_binding={path="../zr_vm/binding",optional=true}\n'
                '[features]\nvm=["dep:vm_binding"]\n', encoding="utf-8")
            (repo / "Cargo.lock").write_text(
                'version = 3\n[[package]]\nname = "candidate_fixture"\nversion = "0.0.0"\n'
                'dependencies = ["vm_binding"]\n[[package]]\nname = "vm_binding"\nversion = "0.0.0"\n',
                encoding="utf-8")
            (repo / "src/lib.rs").write_text(
                '#[cfg(feature="vm")] pub fn value()->u8 { vm_binding::value() }\n', encoding="utf-8")
            (external / "binding/Cargo.toml").write_text(
                '[package]\nname="vm_binding"\nversion="0.0.0"\nedition="2021"\n', encoding="utf-8")
            (external / "binding/src/lib.rs").write_text('pub fn value()->u8 { 7 }\n', encoding="utf-8")
            program = str(Path(shutil.which("git")).absolute())
            def git(root, *args):
                return subprocess.run([program, "-c", "core.longpaths=true", "-c", "core.autocrlf=false", "-c",
                    "core.hooksPath=" + str(top / "no-hooks"), "-C", str(root), *args],
                    check=True, capture_output=True).stdout.decode("utf-8").strip()
            for root in (repo, external):
                git(root, "init", "--initial-branch=main")
                git(root, "config", "user.name", "Synthetic branch fixture")
                git(root, "config", "user.email", "branch-fixture@example.invalid")
                git(root, "add", "--all")
                git(root, "commit", "-m", "Synthetic bounded fixture")
            head = git(repo, "rev-parse", "HEAD")
            candidate = repo / ".jenkins/worktrees/integration-queue"
            candidate.parent.mkdir(parents=True)
            git(repo, "worktree", "add", "--detach", str(candidate), head)
            request = {"operationId": "fixture-branch", "sessionId": "fixture-owner",
                "requestId": "fixture-request", "ownerThreadId": "fixture-thread",
                "paths": [".gitignore", "Cargo.toml", "Cargo.lock", "src/lib.rs"]}
            runtime = {"driverDigest": "d" * 64, "runtimeOperationId": "fixture-current-runtime"}
            config = {**runtime, "repositoryRoot": str(repo)}
            capsule = top / ("a" * 64)
            state = MemoryState()
            state.put("resource_policy", "default", {"cpuBudget": 1, "memoryBudget": 4 * 1024**3,
                "diskBudget": 8 * 1024**3, "diskReserveBytes": 0, "maxHeavyWriters": 1,
                "allowedBuildRoots": [str(actual.path)]}, expected_version=0)
            approved = SimpleNamespace(path=actual.path, volume_identity=actual.volume_identity,
                                       namespace=lambda: top / "namespace")
            helper = load("branch_fixture_local_cargo", ROOT / "tools/dev/local_cargo.py")
            merge_report = top / "merge-proof"
            merge_report.mkdir()
            merge_command = [program, "-c", "core.longpaths=true", "-c", "core.autocrlf=false", "-C", str(candidate),
                             "merge", "--no-edit", head]
            merged = helper._run_owned_command(merge_command, candidate, dict(os.environ), merge_report, 60)
            native_ref = runner.write_json(merge_report / "native-terminal.json", merged)
            self.assertEqual(0, merged["rootExitCode"])
            self.assertTrue(runner.native_terminal_proof(merged, native_ref)["complete"])
            command_ref = runner.write_json(merge_report / "command.json",
                {"runtime": runtime, "operationId": request["operationId"], "operationKind": "merge",
                 "cwd": str(candidate), "command": merge_command})
            row = {"request": request, "runtime": {**runtime, "runtimeOperationId": "fixture-original-runtime"},
                "sourceCommit": head, "sourceRoot": str(repo), "branch": "fixture-original-branch",
                "candidateHead": head, "mainHeadBefore": head, "candidateRoot": str(candidate),
                "commitPaths": request["paths"], "integrationStatus": "validating",
                "candidateGit": {"operationKind": "merge", "nativeComplete": True, "rootExitCode": 0,
                    "reportRoot": str(merge_report), "commandRef": command_ref, "nativeTerminalRef": native_ref}}
            record_path = repo / ".jenkins/state/management/branch-integration/requests/fixture-branch.json"
            marker_path = record_path.parent.parent / "candidate.json"
            runner.write_json(record_path, row)
            runner.write_json(marker_path, {"status": "active", "operationId": request["operationId"],
                                            "path": str(candidate), "mainHead": head})
            runner.write_json(repo / ".jenkins/state/deployment/driver.json", runtime)
            runner.write_json(repo / ".jenkins/state/deployment-operation.json",
                              {"operationId": runtime["runtimeOperationId"], "state": "running"})
            rustup = Path(os.environ.get("RUSTUP_HOME") or str(Path(os.environ["ZR_COMMIT_TEST_CARGO_PATH"]).parents[3]))
            settings = tomllib.loads((rustup / "settings.toml").read_text(encoding="utf-8"))
            selected = ROOT / "rust-toolchain.toml"
            channel = (tomllib.loads(selected.read_text(encoding="utf-8"))["toolchain"]["channel"]
                       if selected.is_file() else (os.environ.get("RUSTUP_TOOLCHAIN") or settings["default_toolchain"]))
            host = settings.get("default_host_triple", "x86_64-pc-windows-msvc")
            toolroot = next(path for path in (rustup / "toolchains" / channel,
                rustup / "toolchains" / (channel + "-" + host)) if (path / "bin/cargo.exe").is_file())
            trusted = {key + suffix: value for key in ("cargo", "rustc") for suffix, value in (
                ("Path", str(toolroot / "bin" / (key + ".exe"))),
                ("Sha256", runner.sha((toolroot / "bin" / (key + ".exe")).read_bytes())))}
            for tool in ("cargo", "rustc"):
                self.assertEqual(Path(os.environ["ZR_COMMIT_TEST_" + tool.upper() + "_PATH"]),
                                 Path(trusted[tool + "Path"]))
                self.assertEqual(os.environ["ZR_COMMIT_TEST_" + tool.upper() + "_SHA256"],
                                 trusted[tool + "Sha256"])
            environment = dict(os.environ)
            environment["PATH"] = str(toolroot / "bin") + os.pathsep + environment.get("PATH", "")
            original_spec = importlib.util.spec_from_file_location
            def helper_spec(name, path, *args, **kwargs):
                return original_spec(name, ROOT / "tools/dev/local_cargo.py" if
                    name == "jenkins_managed_local_cargo" else path, *args, **kwargs)
            def scratch(actor, *, environment=None, **unused):
                path = actor.outputs() / "reports" / actor.key / "tmp"
                if environment is not None:
                    path.mkdir(parents=True)
                    return {"path": str(path), "fixtureOnly": True}
                return {"status": "retained-fixture", "removedFileCount": 0, "reclaimedBytes": 0,
                        "receipt": None}
            env = {"JOB_NAME": "zircon-merge-queue", "NODE_NAME": "zircon-windows-agent",
                   "ZR_COMMIT_NATIVE": "1", "ZR_INTEGRATION_DRIVER": runtime["driverDigest"],
                   "ZR_INTEGRATION_RUNTIME": runtime["runtimeOperationId"]}
            with ExitStack() as stack:
                for context in (patch.dict(os.environ, env), patch.object(runner, "boot",
                    return_value=(config, repo, SimpleNamespace())), patch("tools.jenkins.state.State",
                    return_value=state), patch.object(preparation, "scoped_authorization_state",
                    side_effect=lambda supplied: supplied), patch.object(runner, "source_helper",
                    return_value=preparation), patch.object(runner, "ensure_request_context"),
                    patch.object(paths, "canonical_build_root", return_value=approved),
                    patch.object(planning, "_environment_policy",
                        side_effect=lambda *_: (dict(environment), dict(trusted), top / "fixture-environment.json")),
                    patch.object(runner.Management, "dependency_cache",
                        return_value={"home": str(top / "cargo-home"), "configSha256": {}}),
                    patch.object(runner.Management, "task_tmp", scratch),
                    patch.object(importlib.util, "spec_from_file_location", side_effect=helper_spec)):
                    stack.enter_context(context)
                yield {"repo": repo, "root": candidate, "state": state, "capsule": capsule,
                    "request": request, "runtime": runtime, "config": config, "component": "b" * 64,
                    "record": record_path, "marker": marker_path, "native": merge_report / "native-terminal.json",
                    "recipe": {"verb": "check", "packages": ["candidate_fixture"], "features": ["vm"],
                               "offline": True, "locked": True, "jobs": 1}}

    def test_missing_candidate_sibling_uses_managed_prepared_cargo(self):
        with self._candidate_fixture() as fixture:
            row = json.loads(fixture["record"].read_bytes())
            fixture["request"].pop("ownerThreadId")
            fixture["request"]["paths"] = [
                name.upper() if name == "src/lib.rs" else name
                for name in fixture["request"]["paths"]]
            row["request"] = fixture["request"]
            fixture["record"].write_bytes(runner.encoded(row) + b"\n")
            historical = {name: fixture[name].read_bytes() for name in ("record", "marker", "native")}
            self.assertFalse((fixture["root"].parent / "zr_vm").exists())
            from tools.jenkins.source import repository_id
            def ordinary_context(config, repo, manager, payload, **kwargs):
                if payload.get("observationOnly") is not True:
                    raise ValueError("This identity already belongs to another local context")
            with patch.object(runner, "ensure_request_context", side_effect=ordinary_context) as ordinary_admission, \
                    patch.object(fixture["state"], "check_authorization",
                                 wraps=fixture["state"].check_authorization) as authorization:
                result = runner.validate_branch_candidate(fixture["capsule"], component_digest=fixture["component"],
                    root=fixture["root"], branch_request=fixture["request"], cargo_recipe=fixture["recipe"],
                    ordinal=1, runtime=fixture["runtime"])
            ordinary_admission.assert_not_called()
            authorization.assert_any_call(repository_id(fixture["repo"]), fixture["request"]["sessionId"],
                                          runner.AUTH["validate"], fixture["request"]["paths"])
            self.assertGreaterEqual(authorization.call_count, 2)
            self.assertEqual("passed", result["status"], result)
            self.assertIsNone(result["branchCandidateBinding"]["ownerThreadId"])
            self.assertEqual(fixture["request"]["paths"],
                             result["branchCandidateBinding"]["ownedPaths"])
            execution = Path(result["executionRoot"])
            self.assertTrue((execution.parent / "zr_vm/binding/Cargo.toml").is_file())
            self.assertEqual("integration-candidate",
                fixture["state"].get("management_source", result["operationKey"])["payload"]["sourceView"])
            capture = fixture["state"].get("management_source", result["operationKey"])["payload"]
            self.assertEqual("full-file-index-v2", capture["source"]["sourceEvidencePolicy"])
            self.assertEqual("full-file-index-v2", capture["externalRoots"][0]["source"]["sourceEvidencePolicy"])
            self.assertTrue(result["capturedInputVerified"])
            self.assertTrue(result["preparedInputsUnchanged"])
            self.assertEqual(result["sealedInputRef"], result["preparation"]["sealedInputRef"])
            native = runner.read_reference(result["nativeTerminal"])
            self.assertEqual(0, native["rootExitCode"])
            self.assertIn("--locked", native["command"])
            self.assertIn("--offline", native["command"])
            self.assertTrue(runner.native_terminal_proof(native, result["nativeTerminal"])["complete"])
            self.assertTrue(fixture["state"].list("sealed_input"))
            self.assertTrue(fixture["state"].list("preparation_source"))
            self.assertTrue(all(row["payload"]["status"] == "released"
                                for row in fixture["state"].list("resource_reservation")))
            self.assertFalse(fixture["state"].list("management_worktree"))
            self.assertEqual(historical,
                {name: fixture[name].read_bytes() for name in ("record", "marker", "native")})

    def test_changed_candidate_binding_retains_evidence_and_does_not_launch(self):
        with self._candidate_fixture() as fixture:
            original = json.loads(fixture["record"].read_bytes())
            scope = list(fixture["request"]["paths"])
            invalid = [
                (["src/foreign.rs"], scope),
                ([*original["commitPaths"], "SRC/lib.rs"], scope),
                (original["commitPaths"], [*scope, "SRC/lib.rs"]),
            ]
            for committed, owned in invalid:
                with self.subTest(committed=committed, owned=owned):
                    fixture["request"]["paths"] = list(owned)
                    row = {**original, "request": fixture["request"],
                           "commitPaths": list(committed)}
                    fixture["record"].write_bytes(runner.encoded(row) + b"\n")
                    evidence = {name: fixture[name].read_bytes()
                                for name in ("record", "marker", "native")}
                    with patch.object(preparation, "seal_source", wraps=preparation.seal_source) as seal, \
                            patch.object(runner.Management, "run", wraps=runner.Management.run) as launch:
                        with self.assertRaisesRegex(ValueError, "Original branch queue candidate binding changed"):
                            runner.validate_branch_candidate(fixture["capsule"], component_digest=fixture["component"],
                                root=fixture["root"], branch_request=fixture["request"], cargo_recipe=fixture["recipe"],
                                ordinal=1, runtime=fixture["runtime"])
                    seal.assert_not_called()
                    launch.assert_not_called()
                    for domain in ("resource_reservation", "execution", "execution_launch", "sealed_input"):
                        self.assertFalse(fixture["state"].list(domain))
                    self.assertEqual(evidence,
                        {name: fixture[name].read_bytes() for name in ("record", "marker", "native")})
            fixture["request"]["paths"] = scope
            row = {**original, "request": fixture["request"]}
            row["candidateHead"] = "0" * 40
            fixture["record"].write_bytes(runner.encoded(row) + b"\n")
            evidence = {name: fixture[name].read_bytes() for name in ("record", "marker", "native")}
            with patch.object(preparation, "seal_source", wraps=preparation.seal_source) as seal, \
                    patch.object(runner.Management, "run", wraps=runner.Management.run) as launch:
                with self.assertRaisesRegex(ValueError, "Candidate HEAD"):
                    runner.validate_branch_candidate(fixture["capsule"], component_digest=fixture["component"],
                        root=fixture["root"], branch_request=fixture["request"], cargo_recipe=fixture["recipe"],
                        ordinal=1, runtime=fixture["runtime"])
            seal.assert_not_called()
            launch.assert_not_called()
            for domain in ("resource_reservation", "execution", "execution_launch", "sealed_input"):
                self.assertFalse(fixture["state"].list(domain))
            self.assertEqual(evidence,
                {name: fixture[name].read_bytes() for name in ("record", "marker", "native")})


if __name__ == "__main__":
    unittest.main()

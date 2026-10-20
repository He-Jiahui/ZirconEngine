"""Versioned full-file/index evidence and bounded legacy Git supervision."""
from __future__ import annotations

import copy
import hashlib
import importlib.util
import os
from pathlib import Path
from types import SimpleNamespace
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from tools.jenkins.contracts import JenkinsError

ROOT = Path(__file__).resolve().parents[3]
AREA = Path("E:/cargo-targets/zircon-local/tmp/local-validation-admission-tests")


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, ROOT / "tools/jenkins-management" / file)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


helper = load("inventory_v2_helper_tests", "source_preparation.py")
runner = load("inventory_v2_runner_tests", "runner.py")


def source(head, files, index=None):
    value = {"head": head, "files": files}
    if index is None:
        value["diffSha256"] = hashlib.sha256(b"actual legacy diff").hexdigest()
    else:
        value.update(sourceEvidencePolicy="full-file-index-v2", gitIndex={
            "entries": index, "metadataSha256": "d" * 64})
    return {**value, "sourceDigest": helper.digest(value)}


class SourceInventoryV2Tests(unittest.TestCase):
    def metadata(self):
        state = {"names": b"a.rs\0deleted.rs\0", "head": "b" * 40,
                 "index": b"100644 " + b"a" * 40 + b" 0\ta.rs\0"
                          b"100644 " + b"c" * 40 + b" 0\tdeleted.rs\0"}

        def git(root, *args):
            if args == ("ls-files", "--cached", "--others", "--exclude-standard", "-z"):
                raw = state["names"]
            elif args == ("ls-files", "--stage", "-z"):
                raw = state["index"]
            elif args == ("ls-files", "--stage", "--debug", "-z"):
                raw = state["index"] + b"flags: 0\n"
            else:
                raise AssertionError("Full inventory must not request a binary patch")
            return SimpleNamespace(stdout=raw)
        manager = SimpleNamespace(git=git, text=lambda root, *args: state["head"])
        return state, manager

    @unittest.skipUnless(os.name == "nt", "Real source pins require Windows")
    def test_native_v2_reads_all_bytes_absences_and_changed_same_stat_without_binary_diff(self):
        AREA.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="inventory-v2-", dir=AREA) as temporary:
            root = Path(temporary)
            self.assertTrue(root.resolve().is_relative_to(AREA.resolve()))
            file = root / "a.rs"
            file.write_bytes(b"abc")
            facts = file.stat()
            state, metadata = self.metadata()
            actor = runner.Management.__new__(runner.Management)
            actor.payload = {"action": "validate", "sourceEvidencePolicy": "full-file-index-v2"}
            # Public preflight also constructs an actor without an action attribute.
            actor.git, actor.text = metadata.git, metadata.text
            original = copy.deepcopy(actor.payload)
            with patch.object(runner, "source_helper", return_value=helper), patch.object(
                    runner, "plain", side_effect=AssertionError("Per-file plain fallback is forbidden")):
                before = actor.source(root)
                file.write_bytes(b"xyz")
                os.utime(file, ns=(facts.st_atime_ns, facts.st_mtime_ns))
                after = actor.source(root)
            self.assertEqual({"a.rs": hashlib.sha256(b"abc").hexdigest(), "deleted.rs": None}, before["files"])
            self.assertEqual(hashlib.sha256(b"xyz").hexdigest(), after["files"]["a.rs"])
            self.assertNotEqual(before["sourceDigest"], after["sourceDigest"])
            self.assertNotIn("diffSha256", before)
            self.assertEqual(helper._source_index_entries(state["index"]), before["gitIndex"]["entries"])
            self.assertEqual(original, actor.payload)

    def test_complete_index_mode_object_stage_and_flags_change_source_identity(self):
        state, manager = self.metadata()
        root = Path("E:/cargo-targets/metadata-only")
        hashes = {"a.rs": "f" * 64, "deleted.rs": None}
        before = helper.finish_full_file_index(manager, root, hashes, helper.full_file_index_metadata(manager, root))
        original = state["index"]
        for updated in (original.replace(b"100644", b"100755", 1),
                        original.replace(b"a" * 40, b"e" * 40, 1),
                        original.replace(b" 0\ta.rs", b" 2\ta.rs", 1)):
            with self.subTest(index=updated):
                state["index"] = updated
                captured = helper.finish_full_file_index(manager, root, hashes, helper.full_file_index_metadata(manager, root))
                self.assertNotEqual(before["sourceDigest"], captured["sourceDigest"])
        state["index"] = original
        with patch.object(manager, "git", side_effect=lambda root, *args: SimpleNamespace(
                stdout=(b"changed flags" if "--debug" in args else (
                    state["names"] if "--cached" in args else state["index"])))):
            captured = helper.finish_full_file_index(manager, root, hashes, helper.full_file_index_metadata(manager, root))
        self.assertNotEqual(before["sourceDigest"], captured["sourceDigest"])

    def test_capture_rejects_head_index_or_complete_membership_drift(self):
        state, manager = self.metadata()
        root, hashes = Path("E:/cargo-targets/metadata-only"), {"a.rs": "f" * 64, "deleted.rs": None}
        for field, value in (("head", "c" * 40), ("index", state["index"].replace(b" 0\t", b" 1\t", 1)),
                             ("names", state["names"] + b"foreign.rs\0")):
            with self.subTest(field=field):
                old = state[field]
                before = helper.full_file_index_metadata(manager, root)
                state[field] = value
                with self.assertRaisesRegex(ValueError, "changed during capture"):
                    helper.finish_full_file_index(manager, root, hashes, before)
                state[field] = old

    def test_nested_v2_components_and_legacy_external_keep_their_own_original_versions(self):
        child = source("b" * 40, {"a.rs": "f" * 64}, [
            {"path": "a.rs", "mode": "100644", "object": "a" * 40, "stage": 0}])
        outer = source("c" * 40, {"vendor": {"gitlinkCommit": "b" * 40, "source": child}}, [
            {"path": "vendor", "mode": "160000", "object": "b" * 40, "stage": 0}])
        self.assertIs(outer, helper.checked_source_evidence(outer))
        component = runner.source_component(outer)
        self.assertNotIn("diffSha256", component)
        self.assertNotIn("gitIndex", component)
        self.assertEqual(1, component["gitIndexEntryCount"])
        self.assertEqual(child["sourceDigest"], component["gitlinks"]["vendor"]["source"]["sourceDigest"])
        legacy = source("b" * 40, {"a.rs": "f" * 64})
        raw = helper.encoded(legacy)
        payload = {"action": "validate", "sourceEvidencePolicy": "full-file-index-v2"}
        self.assertEqual({"action": "validate"}, helper.source_evidence_payload(payload, legacy))
        self.assertEqual(raw, helper.encoded(helper.checked_source_evidence(legacy)))
        self.assertEqual(legacy["diffSha256"], runner.source_component(legacy)["diffSha256"])
        tampered = copy.deepcopy(outer)
        tampered["gitIndex"]["entries"][0]["object"] = "e" * 40
        tampered["sourceDigest"] = helper.digest({key: value for key, value in tampered.items() if key != "sourceDigest"})
        with self.assertRaisesRegex(JenkinsError, "Nested Gitlink"):
            helper.checked_source_evidence(tampered)

    def test_legacy_binary_diff_streams_real_hash_and_times_out_only_its_owned_tree(self):
        from tools.jenkins.pilot.native import process_supervisor
        actor = runner.Management.__new__(runner.Management)
        actor.key = "k" * 64
        actor.git_command = lambda root, *args: ["git", *args]
        chunks = [b"actual-patch\n" * 1024, b"binary\0" * 4096]
        for timeout in (False, True):
            managed = SimpleNamespace(pid=123, creation_time=456, terminal_scope="windows_job")
            events = []

            def wait(*, timeout, _raise=timeout):
                events.append("wait")
                if _raise and events.count("wait") == 1:
                    raise subprocess.TimeoutExpired(["git"], timeout)
                return 0
            managed.wait = wait
            managed.terminate = lambda **kwargs: events.append("terminate")

            class Supervisor:
                def __init__(self, **kwargs):
                    self.callback = kwargs["output_callback"]
                def start(self, command, **kwargs):
                    for chunk in chunks:
                        self.callback(kwargs["owner"], "stdout", chunk)
                    self.callback(kwargs["owner"], "stderr", b"diagnostic")
                    return managed
                def close(self):
                    events.append("close")

            arguments = ("diff", "--no-ext-diff", "--no-textconv", "--binary", "--full-index", "HEAD", "--")
            with patch.object(process_supervisor, "ProcessSupervisor", Supervisor):
                if timeout:
                    with self.assertRaises(subprocess.TimeoutExpired):
                        actor.git(ROOT, *arguments)
                    self.assertEqual(["wait", "terminate", "wait", "close"], events)
                else:
                    result = actor.git(ROOT, *arguments)
                    self.assertEqual(b"", result.stdout, "A large legacy patch must never be retained or copied")
                    self.assertEqual(hashlib.sha256(b"".join(chunks)).hexdigest(), result.stdoutSha256)
                    self.assertEqual("windows_job", result.nativeProof["terminalScope"])
                    self.assertTrue(result.nativeProof["stdoutAndStderrEof"])
                    self.assertEqual(["wait", "close"], events)


    def test_batch_recipe_preserves_preparation_ref_and_v2_policy_without_cargo_flags(self):
        entry = load("inventory_v2_batch_entry_tests", "entry.py")
        commands = [{"ordinal": 1, "name": "owned-regression",
            "cargoArguments": ["test", "--package", "zircon_ecs", "--lib", "--locked",
                               "--jobs", "1", "--", "--test-threads", "1", "owned_regression"],
            "ownedNativeTimeoutSeconds": 600, "minimumExpectedMatchedTests": 1}]
        AREA.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="inventory-v2-batch-", dir=AREA) as temporary:
            contract = Path(temporary) / "contract.json"
            raw = helper.encoded({"sharedSingleBatchRequired": True, "semanticCommands": commands})
            contract.write_bytes(raw)
            actor = runner.Management.__new__(runner.Management)
            actor._batch_parser = entry.parse_cargo
            actor.payload = {"action": "validate", "sourceView": "worktree", "worktreeId": "w-t",
                "worktreePreparationRef": "b" * 64, "sourceEvidencePolicy": "full-file-index-v2",
                "batch": {"schemaVersion": 1, "sharedSingleBatchRequired": True,
                    "contractRef": {"path": str(contract), "sha256": hashlib.sha256(raw).hexdigest()},
                    "recipes": commands}}
            original = copy.deepcopy(actor.payload)
            recipe_method = runner.Management.validation_recipe
            with patch.object(runner.Management, "validation_recipe", autospec=True,
                              side_effect=recipe_method) as checked:
                result = actor.validation_batch()
            checked.assert_called_once()
            forwarded = checked.call_args.args[0].payload
            for field in ("worktreePreparationRef", "sourceEvidencePolicy"):
                self.assertEqual(original[field], forwarded[field])
                self.assertNotIn(field, result["recipes"][0]["command"])
            self.assertEqual(commands, result["coverage"]["recipes"])
            self.assertEqual(["owned_regression"], result["recipes"][0]["harnessFilters"])
            self.assertEqual(original, actor.payload)
            for field, invalid in (("worktreePreparationRef", None), ("worktreePreparationRef", {}),
                    ("worktreePreparationRef", "g" * 64), ("worktreePreparationRef", "b" * 63),
                    ("sourceEvidencePolicy", None), ("sourceEvidencePolicy", {}),
                    ("sourceEvidencePolicy", "legacy-v1")):
                with self.subTest(field=field, invalid=invalid):
                    actor.payload = {**original, field: invalid}
                    with self.assertRaises(ValueError):
                        actor.validation_batch()
            actor.payload = {**original, "unrecognizedCargoField": True}
            with self.assertRaisesRegex(ValueError, "Unsupported Cargo fields"):
                actor.validation_batch()


    @unittest.skipUnless(os.name == "nt", "Real sealed and validation producers require native Windows pins")
    def test_sealed_external_producer_reconciles_validation_path_spelling_with_full_evidence(self):
        AREA.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="inventory-v2-external-", dir=AREA) as temporary:
            area = Path(temporary)
            external, checkout, worktree = area / "zr_vm", area / "engine", area / "worktree"
            for path in (external, checkout, worktree):
                path.mkdir()
            (external / "CMakeLists.txt").write_bytes(b"project(external_fixture)\n")
            (external / "README.md").write_bytes(b"external fixture\n")
            actor = runner.Management.__new__(runner.Management)
            actor.repo, actor.action, actor.payload = checkout, "validate", {"action": "validate"}
            names = b"CMakeLists.txt\0README.md\0"

            def git(root, *args, **kwargs):
                self.assertEqual(external, root)
                if args == ("ls-files", "--cached", "--others", "--exclude-standard", "-z"):
                    raw = names
                elif args == ("ls-files", "--stage", "-z"):
                    raw = b""
                elif args == ("diff", "--no-ext-diff", "--no-textconv", "--binary", "--full-index", "HEAD", "--"):
                    raw = b"actual external fixture diff\n"
                else:
                    raise AssertionError("The test must not execute Git or mutations")
                return SimpleNamespace(stdout=raw)

            actor.git = git
            actor.text = lambda root, *args: str(root) if args[-1] == "--show-toplevel" else "a" * 40
            with patch.object(runner, "source_helper", return_value=helper):
                original = helper._ManagedCapturedExternal(actor, area / "objects").source(
                    external, payload={"externalGitlinks": {}})
                captured = actor.source(external)
            self.assertNotEqual(original, captured)
            self.assertEqual(original["files"]["cmakelists.txt"], captured["files"]["CMakeLists.txt"])
            original_bytes, captured_bytes = helper.encoded(original), helper.encoded(captured)
            actor.config, actor.state, actor.session = {}, object(), "writer"
            actor.repo_id = "external-compat-fixture"
            actor.payload = {"action": "validate", "sourceView": "worktree", "worktreeId": "w-t",
                "worktreePreparationRef": "b" * 64, "sourceEvidencePolicy": "full-file-index-v2",
                "externalRoots": [str(external)]}
            actor.validation_root = lambda **kwargs: worktree
            engine = source("c" * 40, {"owned.rs": "f" * 64}, [
                {"path": "owned.rs", "mode": "100644", "object": "e" * 40, "stage": 0}])
            actor.source = lambda root, **kwargs: captured if root == external else engine
            prepared = {"path": str(worktree), "writerSessionId": "writer",
                "sourceKind": "sealed-observed-canonical-input",
                "externalRoots": [{"path": str(external), "source": original}]}
            with patch.object(runner, "source_helper", return_value=helper), patch.object(
                    helper, "_managed_route_preparation", return_value=({"payload": prepared}, {})):
                root, result = actor.validation_source()
            self.assertEqual(worktree, root)
            self.assertEqual(original, result["externalRoots"][0]["source"])
            self.assertIsNot(original, result["externalRoots"][0]["source"])
            proof = result["externalCompatibility"][0]
            self.assertEqual(captured["sourceDigest"], proof["actualCaptureDigest"])
            self.assertEqual(original["sourceDigest"], proof["reconciledCaptureDigest"])
            self.assertEqual(original["sourceDigest"], proof["preparedCaptureDigest"])
            self.assertTrue(proof["exactHeadDiffFilesAndNestedGitlinks"])
            self.assertEqual({"CMakeLists.txt", "README.md"}, {item["from"] for item in proof["pathRewrites"]})
            self.assertEqual(original_bytes, helper.encoded(original))
            self.assertEqual(captured_bytes, helper.encoded(captured))

    def test_legacy_external_reconciliation_keeps_nested_identity_and_rejects_drift_or_aliases(self):
        original_child = source("b" * 40, {"api.h": "a" * 64})
        captured_child = source("b" * 40, {"Api.h": "a" * 64})
        original = source("c" * 40, {"cmakelists.txt": "f" * 64,
            "vendor/LibFoo": {"gitlinkCommit": "b" * 40, "source": original_child}})
        captured = source("c" * 40, {"CMakeLists.txt": "f" * 64,
            "vendor/LibFoo": {"gitlinkCommit": "b" * 40, "source": captured_child}})
        raw_original, raw_captured = helper.encoded(original), helper.encoded(captured)
        with patch.object(runner, "source_helper", return_value=helper):
            reconciled, proof = runner.reconcile_prepared_external_capture(captured, original, legacy_sealed=True)
            self.assertEqual(original, reconciled)
            self.assertEqual({"", "vendor/LibFoo"}, {item["repository"] for item in proof["pathRewrites"]})
            with self.assertRaises(ValueError):
                runner.reconcile_prepared_external_capture(captured, original, legacy_sealed=False)
            for field in ("content", "head", "diff", "nestedHead", "gitlink", "membership", "collision"):
                changed = copy.deepcopy(captured)
                child = changed["files"]["vendor/LibFoo"]["source"]
                if field == "content":
                    changed["files"]["CMakeLists.txt"] = "e" * 64
                elif field == "head":
                    changed["head"] = "d" * 40
                elif field == "diff":
                    changed["diffSha256"] = "d" * 64
                elif field == "nestedHead":
                    child["head"] = "d" * 40
                elif field == "gitlink":
                    changed["files"]["vendor/LibFoo"]["gitlinkCommit"] = "d" * 40
                elif field == "membership":
                    del changed["files"]["CMakeLists.txt"]
                else:
                    changed["files"]["cmakelists.txt"] = changed["files"]["CMakeLists.txt"]
                child["sourceDigest"] = helper.digest({k: v for k, v in child.items() if k != "sourceDigest"})
                changed["sourceDigest"] = helper.digest({k: v for k, v in changed.items() if k != "sourceDigest"})
                with self.subTest(field=field), self.assertRaises((ValueError, JenkinsError)):
                    runner.reconcile_prepared_external_capture(changed, original, legacy_sealed=True)
        self.assertEqual(raw_original, helper.encoded(original))
        self.assertEqual(raw_captured, helper.encoded(captured))

    @unittest.skipUnless(os.name == "nt", "Prepared CAS integration requires native Windows pins")
    def test_completed_preparation_seals_and_materializes_original_external_cas_after_live_drift(self):
        from tools.jenkins import managed_worktree as formal, managed_worktree_lifetime as lifetime
        from tools.jenkins.resources.paths import ApprovedBuildRoot, canonical_build_root
        sealed_tests = load("inventory_v2_sealed_fixture", "tests/test_managed_worktree_sealed_inventory.py")
        fixture = sealed_tests.ManagedWorktreeSealedInventoryTests(methodName="runTest")
        self.addCleanup(fixture.doCleanups)
        fixture.setUp()
        f = fixture.fixture
        approved = canonical_build_root(r"E:\cargo-targets")
        # Retain the existing small fixture namespace; physical admission remains real.
        with patch.object(ApprovedBuildRoot, "namespace", return_value=f.cas), patch.object(
                formal, "canonical_build_root", return_value=approved), patch.object(
                runner, "source_helper", return_value=helper):
            f.write(f.target, {".codex/extra-control.py": b"actual WT control\n",
                               "old-untracked.txt": b"removed by preparation\n"})
            actor = runner.Management.__new__(runner.Management)
            actor.__dict__.update(f.management.__dict__)
            del actor.__dict__["source"]
            actor.state = helper.scoped_authorization_state(f.state)
            actor.action, actor.session = "prepare-worktree", "w"
            actor.payload = {"action": "prepare-worktree", "sourceEvidencePolicy": "full-file-index-v2"}
            mutable = {"externalHead": "c" * 40, "forbidExternal": False}

            def git(root, *args, **kwargs):
                if root != f.target:
                    if mutable["forbidExternal"] and (root == f.external or f.external in root.parents):
                        raise AssertionError("Validation reopened the mutable external checkout")
                    return f.git(root, *args, **kwargs)
                if args[:3] == ("add", "--intent-to-add", "--force"):
                    return f.git(root, *args, **kwargs)
                if args == ("ls-files", "--cached", "--others", "--exclude-standard", "-z"):
                    names = set(f.tracked) | set(f.source(root)["files"])
                    raw = b"".join(name.encode() + b"\0" for name in sorted(names))
                elif args in (("ls-files", "--stage", "-z"), ("ls-files", "--stage", "--debug", "-z")):
                    raw = b"".join(("100644 " + "e" * 40 + " 0\t" + name).encode() + b"\0"
                                   for name in sorted(f.tracked))
                    if "--debug" in args:
                        raw += b"flags: 0\n"
                elif args == ("diff", "--no-ext-diff", "--no-textconv", "--binary", "--full-index", "HEAD", "--"):
                    raw = b"actual fixture legacy diff\n"
                else:
                    return f.git(root, *args, **kwargs)
                return SimpleNamespace(stdout=raw)

            def text(root, *args):
                if root == f.external and args[-1] == "HEAD":
                    if mutable["forbidExternal"]:
                        raise AssertionError("Validation observed the mutable external HEAD")
                    return mutable["externalHead"]
                return f.text(root, *args)

            actor.git, actor.text = git, text
            f.management.git, f.management.source, f.management.text = git, actor.source, text
            creator = f.payload("create-worktree", "real-v1-create", "w")
            f.creator = f.complete(creator, {"worktreeId": "w-t",
                "source": actor.source(f.target, payload={"action": "prepare-worktree"})})
            inspection, reference = fixture.inspect(declared=False)
            original_capture = copy.deepcopy(fixture.inputs["sealedSource"]["externalCapture"])
            original_manifest = copy.deepcopy(fixture.inputs["sealedSource"]["sourceManifest"])
            prepare = {**f.payload("prepare-worktree", "real-v2-prepare", "r"),
                "sourceEvidencePolicy": "full-file-index-v2",
                "inspectionOperationKey": inspection, "preparationInputRef": reference}
            f.use(prepare)
            actor.payload = prepare
            with lifetime.writer_hold(f.management):
                prepared_result = helper.managed_prepare_worktree(f.management, preparation_helper=helper)
            prepared_key = f.complete(prepare, prepared_result)
            prepared_row = copy.deepcopy(f.state.get("managed_worktree_preparation", prepared_key))
            self.assertEqual("full-file-index-v2", prepared_result["source"]["sourceEvidencePolicy"])
            self.assertFalse((f.target / "old-untracked.txt").exists())
            self.assertTrue((f.target / ".codex/extra-control.py").is_file())
            original_external = (f.external / "src/lib.rs").read_bytes()
            f.write(f.external, {"src/lib.rs": b"later foreign content\n"})
            mutable.update(externalHead="f" * 40, forbidExternal=True)
            actor.action, actor.session = "validate", "w"
            actor.payload = {**f.payload("validate", "sealed-validation", "w"), "sourceView": "worktree",
                "worktreePreparationRef": prepared_key, "sourceEvidencePolicy": "full-file-index-v2",
                "buildRoot": str(approved.path), "externalRoots": [str(f.external)],
                "externalGitlinks": fixture.inputs["externalGitlinks"]}
            root, before = actor.validation_source()
            self.assertEqual(original_capture, before["externalRoots"])
            self.assertEqual("prepared-sealed-CAS", before["externalInputPolicy"])
            self.assertFalse(before["preparedSealedInput"]["liveExternalObserved"])
            self.assertEqual(prepared_result["source"], before["source"])
            unchanged = (f.target / "owned.txt").read_bytes()
            (f.target / "owned.txt").write_bytes(b"unapproved WT drift")
            with self.assertRaises((ValueError, JenkinsError)):
                actor.validation_source()
            (f.target / "owned.txt").write_bytes(unchanged)
            owner, writer, reservation = "management-fixture-cas", "fixture-pool", "fixture-reservation"
            f.state.put("resource_reservation", reservation, {"status": "active", "owner": owner})
            f.state.put("pool_writer_hold", writer, {"status": "active", "owner": owner,
                "reservationKey": reservation})
            sealed = helper.seal_source(actor.state, source_root=root, snapshot=before,
                repository_root=f.repo, repository_id=f.repo_id, session_id="w", owner=owner,
                approved=approved, coverage={"kind": "fixture"}, management=actor)
            helper.verify_sealed_capture(before["source"], sealed["sourceManifest"])
            self.assertNotEqual(helper.digest(original_manifest), sealed["sealedInputRef"])
            self.assertEqual(original_manifest["external"], sealed["sourceManifest"]["external"])
            self.assertTrue(any(value["path"] == ".codex/extra-control.py"
                                for value in sealed["sourceManifest"]["entries"]))
            self.assertFalse(any(value["path"] == "old-untracked.txt"
                                 for value in sealed["sourceManifest"]["entries"]))
            execution_root = f.area / "pool/inputs/source"
            materialized = helper.materialize_source(actor.state, approved=approved, owner=owner,
                preparation_key_value=writer, source_path=execution_root, reservation_key=reservation,
                sealed=sealed, identity={"repositoryId": f.repo_id, "sessionId": "w"},
                command=["cargo", "test"], environment_digest="a" * 64)
            first = helper.verify_source(sealed["sourceManifest"], execution_root, approved)
            self.assertEqual(original_external, (execution_root.parent / "zr_vm/src/lib.rs").read_bytes())
            self.assertNotEqual(original_external, (f.external / "src/lib.rs").read_bytes())
            self.assertEqual(b"actual WT control\n", (execution_root / ".codex/extra-control.py").read_bytes())
            self.assertEqual(first, helper.verify_source(sealed["sourceManifest"], execution_root, approved))
            self.assertEqual(str(execution_root), materialized["physicalPath"])
            self.assertEqual(prepared_row, f.state.get("managed_worktree_preparation", prepared_key))
            self.assertEqual(original_manifest, fixture.inputs["sealedSource"]["sourceManifest"])
            checksum = fixture.inputs["externalInputs"][0]["currentHashes"]["src/lib.rs"]
            obj = f.cas / "inputs/objects" / checksum
            original_object = obj.read_bytes()
            obj.write_bytes(b"corrupt sealed dependency")
            try:
                with self.assertRaises((ValueError, JenkinsError)):
                    actor.validation_source()
                with self.assertRaises((ValueError, JenkinsError)):
                    helper.seal_source(actor.state, source_root=root, snapshot=before,
                        repository_root=f.repo, repository_id=f.repo_id, session_id="w", owner=owner + "-bad",
                        approved=approved, coverage={"kind": "fixture"}, management=actor)
            finally:
                obj.write_bytes(original_object)


    @unittest.skipUnless(os.name == "nt", "Absent owned preparation requires native Windows pins")
    def test_absent_owned_null_inventory_routes_reader_and_prepares_without_expanding_scope(self):
        import json
        from tools.jenkins import managed_worktree as formal, managed_worktree_lifetime as lifetime
        from tools.jenkins.resources.paths import ApprovedBuildRoot, canonical_build_root
        from tools.jenkins.tests import test_managed_worktree_preparation as baseline
        from tools.jenkins.state import State

        f = baseline.ManagedWorktreePreparationTests(methodName="runTest")
        self.addCleanup(f.doCleanups)
        f.setUp()
        f.state = State(f.repo / ".jenkins/state/coordination.sqlite3")
        f.management.state = f.state
        absent, reference, writer_session = "never-created-owned.txt", "w-absent", "w-absent"
        owned = [absent, "owned.txt"]
        writer = f.state.authorize_session(f.repo_id, writer_session, "user", owned,
            ["implementation", "validation"], {"source": "user", "task": "fixture absent owned writer"})
        f.target = f.repo / ".jenkins/worktrees" / reference
        unknown = b"unowned WT control\n"
        f.write(f.target, {**f.before, ".codex/extra-control.py": unknown})
        f.record = {**f.record, "sessionId": writer_session, "ownedPaths": owned,
                    "path": str(f.target), "writerPrincipal": writer["principal"],
                    "writerAuthorizationSnapshot": writer["snapshotDigest"]}
        f.management.owned = owned

        def payload(action, request):
            return {**f.payload(action, request, writer_session),
                    "ownedPaths": owned, "worktreeId": reference}

        creator_payload = payload("create-worktree", "absent-create")
        prior = f.source(f.target)
        creator = f.complete(creator_payload, {"worktreeId": reference, "source": prior})
        f.record["creationOperationKey"] = creator
        f.state.put("management_worktree", reference, f.record)
        # This owned path has never existed in the base, prior source, index or either tree.
        for names in (f.before, f.desired, f.tracked, prior["files"]):
            self.assertNotIn(absent, names)
        self.assertFalse((f.repo / absent).exists())
        self.assertFalse((f.target / absent).exists())
        canonical, external = f.source(f.repo), f.source(f.external)
        declared = {absent: None, "owned.txt": hashlib.sha256(f.desired["owned.txt"]).hexdigest()}
        approved = canonical_build_root(r"E:\cargo-targets")
        manager = SimpleNamespace(spec=SimpleNamespace(storage={"buildRoot": r"E:\cargo-targets"}))
        with patch.object(ApprovedBuildRoot, "namespace", return_value=f.cas), \
                patch.object(formal, "canonical_build_root", return_value=approved), \
                patch.object(helper, "local_request_principal", return_value="user"):
            inspect_payload = {**payload("inspect-worktree", "absent-inspect"),
                "targetSourceOperationKey": creator, "externalRoots": [str(f.external)],
                "sourceHashes": declared}
            f.use(inspect_payload)
            inspected = helper.managed_inspect_worktree(f.management, preparation_helper=helper)
            inspection = f.complete(inspect_payload, inspected)
            inventory = json.loads(Path(inspected["preparationInputRef"]["path"]).read_bytes())["preparationInput"]
            self.assertIsNone(inventory["currentHashes"][absent])
            self.assertIsNone(inventory["beforeHashes"][absent])
            entry = next(value for value in inventory["sealedSource"]["sourceManifest"]["entries"]
                         if value["path"] == absent)
            self.assertEqual(("deleted", None, None), (entry["status"], entry["sha256"], entry["baseBlob"]))

            prepare_payload = {**payload("prepare-worktree", "absent-prepare"),
                "inspectionOperationKey": inspection, "preparationInputRef": inspected["preparationInputRef"]}
            routed, route = helper.route_managed_context(
                f.management.config, f.repo, manager, prepare_payload, state=f.state)
            reader = f.state.check_authorization(f.repo_id, routed["sessionId"], "validation")
            self.assertNotEqual(writer_session, routed["sessionId"])
            self.assertEqual(["validation"], reader["actions"])
            self.assertIn(absent, reader["paths"])
            self.assertEqual(reader["snapshotDigest"], route["readerAuthorizationSnapshot"])
            self.assertEqual(writer["snapshotDigest"], f.state.check_authorization(
                f.repo_id, writer_session, "implementation")["snapshotDigest"])
            with self.assertRaises(JenkinsError):
                f.state.check_authorization(f.repo_id, routed["sessionId"], "implementation", owned)
            f.use(routed)
            with lifetime.writer_hold(f.management):
                prepared = helper.managed_prepare_worktree(f.management, preparation_helper=helper)
            prepared_key = f.complete(routed, prepared)
            self.assertEqual("complete", prepared["status"])
            self.assertIsNone(prepared["afterHashes"][absent])
            self.assertFalse((f.target / absent).exists())
            self.assertEqual(unknown, (f.target / ".codex/extra-control.py").read_bytes())
            self.assertEqual(f.desired["foreign.bin"], (f.target / "foreign.bin").read_bytes())
            preparation = f.state.get("managed_worktree_preparation", prepared_key)["payload"]
            self.assertEqual((routed["sessionId"], writer_session),
                             (preparation["readerSessionId"], preparation["writerSessionId"]))
            self.assertEqual(writer["snapshotDigest"], preparation["writerAuthorizationSnapshot"])

            for request, hashes, code in (
                    ("unowned-null", {**declared, "never-authorized.txt": None}, "managed_owned_scope_invalid"),
                    ("present-owned-null", {**declared, "owned.txt": None}, "managed_owned_source_changed")):
                with self.subTest(request=request):
                    invalid = {**payload("inspect-worktree", request),
                        "targetSourceOperationKey": prepared_key, "externalRoots": [str(f.external)],
                        "sourceHashes": hashes}
                    f.use(invalid)
                    before = f.source(f.target)
                    with self.assertRaises(JenkinsError) as observed:
                        helper.managed_inspect_worktree(f.management, preparation_helper=helper)
                    self.assertEqual(code, observed.exception.code)
                    self.assertFalse((f.cas / "management/reports" / f.management.key / "source-inventory.json").exists())
                    self.assertEqual(before, f.source(f.target))
            self.assertEqual(canonical, f.source(f.repo))
            self.assertEqual(external, f.source(f.external))
            self.assertEqual(unknown, (f.target / ".codex/extra-control.py").read_bytes())
            self.assertEqual(1, len(f.state.list("managed_worktree_preparation")))


    @unittest.skipUnless(os.name == "nt", "Mixed-case pool recovery requires native Windows paths")
    def test_mixed_case_pool_materialization_verification_recovery_and_reuse(self):
        from tools.jenkins.workflow import pools
        from tools.jenkins.resources.paths import ApprovedBuildRoot, canonical_build_root
        from tools.jenkins.tests import test_managed_worktree_preparation as baseline

        f = baseline.ManagedWorktreePreparationTests(methodName="runTest")
        self.addCleanup(f.doCleanups)
        f.setUp()
        approved = canonical_build_root(r"E:\cargo-targets")
        objects = f.cas / "inputs/objects"
        objects.mkdir(parents=True)
        source, key = f.cas / "pool/inputs/source", "mixed-case-pool"

        def entry(name, data):
            checksum = hashlib.sha256(data).hexdigest()
            (objects / checksum).write_bytes(data)
            return {"path": name, "status": "present", "sha256": checksum,
                    "objectDigest": checksum, "size": len(data), "beforeHash": checksum}

        external = {"schemaVersion": 1, "baseHead": "c" * 40, "external": [], "entries": [
            entry("tests/compileTime/a.c", b"first parent spelling\n"),
            entry("tests/compiletime/test_compile_tool_phase_gate.c", b"phase gate\n")]}
        manifest = {"schemaVersion": 1, "baseHead": f.base, "repositoryId": f.repo_id,
            "preparationKind": "current-source", "entries": [entry("Cargo.toml", b"[workspace]\n")],
            "external": [{"kind": "sealed-external-source", "name": "zr_vm",
                "repositoryRoot": str(f.external), "manifest": external, "sourceDigest": helper.digest(external)}]}
        original_manifest, manifest_digest = copy.deepcopy(manifest), helper.digest(manifest)

        def invoke(label, value=manifest):
            owner, reservation = "mixed-case-" + label, "mixed-case-" + label + "-reservation"
            sealed_digest = helper.digest(value)
            sealed_input = {"sourceManifest": value, "objectRoot": str(f.cas)}
            existing = f.state.get("sealed_input", sealed_digest)
            if existing is None:
                f.state.put("sealed_input", sealed_digest, sealed_input, expected_version=0)
            else:
                self.assertEqual(sealed_input, existing["payload"])
            f.state.put("resource_reservation", reservation, {"owner": owner, "status": "active"})
            f.state.put("pool_writer_hold", key,
                        {"owner": owner, "status": "active", "reservationKey": reservation})
            return helper.materialize_source(f.state, approved=approved, owner=owner,
                preparation_key_value=key, source_path=source, reservation_key=reservation,
                sealed={"sealedInputRef": sealed_digest, "sourceManifest": value,
                        "sourceRoot": str(f.target), "capturedSourceDigest": "c" * 64},
                identity={"repositoryId": f.repo_id, "sessionId": "w"},
                command=["cargo", "test"], environment_digest="e" * 64)

        real_put = f.state.put
        interrupted = False

        def stop_before_complete(kind, row_key, value, *args, **kwargs):
            nonlocal interrupted
            if kind == "preparation_update" and row_key == key and value.get("status") == "complete" and not interrupted:
                interrupted = True
                raise InterruptedError("fixture final journal checkpoint not persisted")
            return real_put(kind, row_key, value, *args, **kwargs)

        with patch.object(ApprovedBuildRoot, "namespace", return_value=f.cas):
            # The real copy and inventory finish; only the final durable checkpoint is interrupted.
            with patch.object(f.state, "put", side_effect=stop_before_complete):
                with self.assertRaisesRegex(InterruptedError, "final journal checkpoint"):
                    invoke("interrupted")
            journal = f.state.get("preparation_update", key)["payload"]
            self.assertEqual("updating", journal["status"])
            self.assertFalse(journal["files"][-1]["applied"])
            self.assertIsNone(f.state.get("preparation_source", key))
            parent = source.parent / "zr_vm/tests"
            self.assertEqual(["compileTime"], [path.name for path in parent.iterdir()])
            phase_gate = parent / "compiletime/test_compile_tool_phase_gate.c"
            self.assertEqual(b"phase gate\n", phase_gate.read_bytes())
            verified = helper.verify_source(manifest, source, approved)
            self.assertEqual(3, verified["fileCount"])
            mtimes = {path.relative_to(source.parent).as_posix(): path.stat().st_mtime_ns
                      for path in source.parent.rglob("*") if path.is_file()}
            with patch.object(pools, "_atomic_copy", wraps=pools._atomic_copy) as copied:
                recovered = invoke("recovered")
                self.assertTrue(recovered["reused"])
                self.assertEqual(0, recovered["changedFileCount"])
                self.assertEqual("complete", f.state.get("preparation_update", key)["payload"]["status"])
                self.assertTrue(invoke("reused")["reused"])
                copied.assert_not_called()
            self.assertEqual(verified, helper.verify_source(manifest, source, approved))
            self.assertEqual(mtimes, {path.relative_to(source.parent).as_posix(): path.stat().st_mtime_ns
                                     for path in source.parent.rglob("*") if path.is_file()})
            self.assertEqual(original_manifest, manifest)
            self.assertEqual(manifest_digest, helper.digest(manifest))
            self.assertEqual(original_manifest, f.state.get("sealed_input", manifest_digest)["payload"]["sourceManifest"])

            alternate = copy.deepcopy(manifest)
            alternate["baseHead"] = "9" * 40
            previous, update = f.state.get("preparation_source", key), f.state.get("preparation_update", key)
            real_inventory = helper._pool_inventory
            with patch.object(helper, "_pool_inventory", wraps=real_inventory) as inventoried, \
                    patch.object(pools, "_atomic_copy", wraps=pools._atomic_copy) as copied, \
                    patch.object(f.state, "put", wraps=f.state.put) as published:
                adopted = invoke("same-bytes-new-identity", alternate)
            self.assertTrue(adopted["reused"])
            self.assertEqual(0, adopted["changedFileCount"])
            self.assertEqual(1, inventoried.call_count)
            copied.assert_not_called()
            self.assertEqual(update, f.state.get("preparation_update", key))
            current = f.state.get("preparation_source", key)
            self.assertEqual(previous["version"] + 1, current["version"])
            self.assertEqual(helper.digest(alternate), current["payload"]["sourceDigest"])
            source_puts = [call for call in published.call_args_list if call.args[0] == "preparation_source"]
            self.assertEqual(1, len(source_puts))
            self.assertEqual(previous["version"], source_puts[0].kwargs["expected_version"])
            self.assertIsNotNone(source_puts[0].kwargs["connection"])
            self.assertFalse(any(call.args[0] == "preparation_update" for call in published.call_args_list))
            self.assertEqual(verified["filesDigest"], helper.verify_source(alternate, source, approved)["filesDigest"])
            self.assertEqual(mtimes, {path.relative_to(source.parent).as_posix(): path.stat().st_mtime_ns
                                     for path in source.parent.rglob("*") if path.is_file()})

            for ordinal, domain in enumerate(("preparation_source", "preparation_update",
                                               "pool_writer_hold", "resource_reservation")):
                with self.subTest(identity_adoption_race=domain):
                    label = "same-bytes-race-" + str(ordinal)
                    changed_manifest = copy.deepcopy(manifest)
                    changed_manifest["baseHead"] = str(8 - ordinal) * 40
                    before_source = f.state.get("preparation_source", key)
                    before_update = f.state.get("preparation_update", key)
                    injected = []

                    def change_authority(*args, **kwargs):
                        actual = real_inventory(*args, **kwargs)
                        row_key = "mixed-case-" + label + "-reservation" if domain == "resource_reservation" else key
                        row = f.state.get(domain, row_key)
                        value = copy.deepcopy(row["payload"])
                        if domain in {"pool_writer_hold", "resource_reservation"}:
                            value["status"] = "released"
                        real_put(domain, row_key, value, expected_version=row["version"])
                        injected.append(f.state.get(domain, row_key))
                        return actual

                    with patch.object(helper, "_pool_inventory", side_effect=change_authority) as inventoried, \
                            patch.object(pools, "_atomic_copy", wraps=pools._atomic_copy) as copied:
                        with self.assertRaisesRegex(ValueError, "authority changed before identity adoption"):
                            invoke(label, changed_manifest)
                    self.assertEqual(1, inventoried.call_count)
                    copied.assert_not_called()
                    self.assertEqual(1, len(injected))
                    self.assertEqual(injected[0] if domain == "preparation_source" else before_source,
                                     f.state.get("preparation_source", key))
                    self.assertEqual(injected[0] if domain == "preparation_update" else before_update,
                                     f.state.get("preparation_update", key))
                    self.assertEqual(verified, helper.verify_source(manifest, source, approved))

            def refuse(label, value, code):
                previous = f.state.get("preparation_source", key)
                update = f.state.get("preparation_update", key)
                with self.assertRaises((ValueError, JenkinsError)):
                    helper.verify_source(value, source, approved)
                with self.assertRaises(JenkinsError) as observed:
                    invoke(label, value)
                self.assertEqual(code, observed.exception.code)
                self.assertEqual(previous, f.state.get("preparation_source", key))
                self.assertEqual(update, f.state.get("preparation_update", key))

            original = phase_gate.read_bytes()
            for case, code in (("foreign-file", "preparation_foreign_file"),
                               ("foreign-directory", "preparation_foreign_file"),
                               ("missing", "preparation_hash_mismatch"),
                               ("wrong-hash", "preparation_hash_mismatch")):
                with self.subTest(case=case):
                    foreign = parent / ("unregistered" if case == "foreign-directory" else "compileTime/foreign.c")
                    if case == "foreign-file":
                        foreign.write_bytes(b"foreign writer\n")
                    elif case == "foreign-directory":
                        foreign.mkdir()
                    elif case == "missing":
                        phase_gate.unlink()
                    else:
                        phase_gate.write_bytes(b"foreign changed hash\n")
                    try:
                        refuse(case, manifest, code)
                        if case.startswith("foreign"):
                            self.assertTrue(foreign.exists(), "Rejected input must remain available as evidence")
                        elif case == "missing":
                            self.assertFalse(phase_gate.exists(), "A rejected reused pool must not repair missing data")
                        else:
                            self.assertEqual(b"foreign changed hash\n", phase_gate.read_bytes())
                    finally:
                        if case == "foreign-file":
                            foreign.unlink()
                        elif case == "foreign-directory":
                            foreign.rmdir()
                        else:
                            phase_gate.write_bytes(original)

            collision = copy.deepcopy(manifest)
            dependency = collision["external"][0]
            alias = copy.deepcopy(dependency["manifest"]["entries"][0])
            alias["path"] = "tests/COMPILETIME/A.C"
            dependency["manifest"]["entries"].append(alias)
            dependency["sourceDigest"] = helper.digest(dependency["manifest"])
            with self.subTest(case="manifest-full-path-case-collision"):
                refuse("manifest-collision", collision, "manifest_path_collision")

            # Ordinary Windows directories cannot contain two distinct case-only file names.
            # Exercise actual collisions only when this fixture volume permits both entities.
            actual_alias = phase_gate.with_name("TEST_COMPILE_TOOL_PHASE_GATE.C")
            try:
                with actual_alias.open("xb") as output:
                    output.write(b"distinct case-only entity\n")
            except FileExistsError:
                self.assertTrue(actual_alias.samefile(phase_gate))
                print("[fixture] actual case-only entity collision unavailable; manifest collision was rejected", flush=True)
            else:
                try:
                    self.assertFalse(actual_alias.samefile(phase_gate))
                    with self.subTest(case="actual-full-path-case-collision"):
                        refuse("actual-collision", manifest, "preparation_path_collision")
                finally:
                    actual_alias.unlink()
            self.assertTrue(invoke("restored")["reused"])
            self.assertEqual(verified, helper.verify_source(manifest, source, approved))



class MigratedExternalInventoryTests(unittest.TestCase):
    """Compiler projections keep complete source and legacy-journal evidence."""

    def entry(self, path, mode="100644", stage=0):
        return {"path": path, "mode": mode, "stage": stage, "object": "b" * 40}

    def capture(self, files, entries):
        return source("a" * 40, files, sorted(entries, key=lambda item: (item["path"], item["stage"])))

    def fixture(self):
        omitted = ".codex/plans/tracked.md"
        retained = [".codex/plans/untracked.md", ".codex/plans/nested/note.md",
                    ".codex/settings.md", ".codex/plans/executable.md",
                    ".codex/plans/link.md", ".codex/plans/conflicted.md", "Cargo.toml"]
        files = {path: "c" * 64 for path in [omitted, *retained]}
        files[".codex/plans/deleted.md"] = None
        entries = [self.entry(omitted), self.entry(retained[1]), self.entry(retained[2]),
                   self.entry(retained[3], "100755"), self.entry(retained[4], "120000"),
                   self.entry(retained[5]), self.entry(retained[5], stage=2),
                   self.entry(retained[6]), self.entry(".codex/plans/deleted.md")]
        return self.capture(files, entries), omitted

    def test_only_tracked_regular_unconflicted_root_plan_is_excluded_from_compiler_inventory(self):
        captured, omitted = self.fixture()
        original = copy.deepcopy(captured)
        expected = {name: value for name, value in captured["files"].items() if name != omitted}
        self.assertEqual(expected, helper._managed_external_compiler_files(captured))
        self.assertEqual(expected, helper.checkout_external_file_view().source_files(captured))
        self.assertEqual(original, captured)
        legacy = source(captured["head"], captured["files"])
        self.assertEqual(captured["files"], helper._managed_external_compiler_files(legacy))

    def test_raw_plan_changes_remain_bound_and_stale_source_digest_is_rejected(self):
        captured, omitted = self.fixture()
        changed = copy.deepcopy(captured)
        changed["files"][omitted] = "d" * 64
        with self.assertRaises(JenkinsError):
            helper._managed_external_compiler_files(changed)
        changed["sourceDigest"] = helper.digest({key: value for key, value in changed.items()
                                                if key != "sourceDigest"})
        self.assertNotEqual(captured["sourceDigest"], changed["sourceDigest"])
        self.assertEqual(helper._managed_external_compiler_files(captured),
                         helper._managed_external_compiler_files(changed))
        self.assertEqual("d" * 64, helper.source_files(changed)[omitted])

    def test_nested_gitlink_projection_preserves_managed_root_only_policy(self):
        nested = self.capture({".codex/plans/nested.md": "c" * 64, "lib.rs": "d" * 64},
                              [self.entry(".codex/plans/nested.md"), self.entry("lib.rs")])
        outer = self.capture({"module": {"gitlinkCommit": "b" * 40, "source": nested}},
                             [self.entry("module", "160000")])
        original = copy.deepcopy(outer)
        self.assertEqual({"module/lib.rs": "d" * 64},
                         helper.checkout_external_file_view().source_files(outer))
        self.assertEqual({"module/.codex/plans/nested.md": "c" * 64, "module/lib.rs": "d" * 64},
                         helper._managed_external_compiler_files(outer))
        self.assertEqual(original, outer)

    def test_sealed_compiler_manifest_requires_explicit_managed_external_mode(self):
        captured, omitted = self.fixture()
        files = {name: value for name, value in captured["files"].items() if name != omitted}
        manifest = {"baseHead": captured["head"], "entries": [
            {"path": path, "status": "deleted" if checksum is None else "present",
             "sha256": checksum, **({"objectDigest": checksum} if checksum is not None else {})}
            for path, checksum in files.items()]}
        helper.verify_sealed_capture(captured, manifest, managed_external=True)
        with self.assertRaisesRegex(ValueError, "complete declared capture"):
            helper.verify_sealed_capture(captured, manifest)
        self.assertIn(omitted, captured["files"])
        self.assertNotIn(omitted, {entry["path"] for entry in manifest["entries"]})

    def test_legacy_cross_version_case_alias_is_joined_without_erasing_original_entries(self):
        old, new = "inputs/compileTime/a.c", "inputs/compiletime/a.c"
        intent = {"oldOwnedFiles": {old: "a" * 64}, "ownedFiles": {new: "b" * 64},
                  "files": [{"path": old, "oldHash": "a" * 64, "newHash": None, "applied": True},
                            {"path": new, "oldHash": None, "newHash": "b" * 64, "applied": False}]}
        original = copy.deepcopy(intent)
        files, lineage = helper._pool_recovery_files(intent, 7)
        self.assertEqual(1, len(files))
        self.assertEqual((new, "a" * 64, "b" * 64, False),
                         tuple(files[0][key] for key in ("path", "oldHash", "newHash", "applied")))
        self.assertEqual(intent["files"], [entry["entry"] for entry in files[0]["legacyEntries"]])
        self.assertEqual((7, helper.digest(original), helper.digest(original["files"]), 2, 1),
                         tuple(lineage[key] for key in ("originalVersion", "originalPayloadDigest",
                               "originalFilesDigest", "originalFileCount", "caseFoldGroupCount")))
        for drift in ("unknown-entry", "same-version-alias", "nonboolean-applied"):
            changed = copy.deepcopy(intent)
            if drift == "unknown-entry":
                changed["files"].append({"path": "foreign.c", "oldHash": None, "newHash": "e" * 64,
                                         "applied": False})
            elif drift == "same-version-alias":
                changed["oldOwnedFiles"][new] = "e" * 64
            else:
                changed["files"][0]["applied"] = 1
            with self.subTest(drift=drift), self.assertRaises(JenkinsError):
                helper._pool_recovery_files(changed, 7)
        self.assertEqual(original, intent)

if __name__ == "__main__":
    unittest.main()

"""Sealed management inventories retain immutable inputs without source-tree copies."""
from __future__ import annotations

import copy
import hashlib
import json
import os
import subprocess
from pathlib import Path
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.tests import test_managed_worktree_preparation as baseline

helper = baseline.helper


@unittest.skipUnless(os.name == "nt", "Native management capture requires Windows")
class ManagedWorktreeSealedInventoryTests(unittest.TestCase):
    def setUp(self):
        self.fixture = baseline.ManagedWorktreePreparationTests(methodName="runTest")
        self.addCleanup(self.fixture.doCleanups)
        self.fixture.setUp()
        # Freeze the external index independently of later worktree byte edits.
        external_indexes = {}
        for root in [self.fixture.external, *(self.fixture.external / name for name in self.fixture.links)]:
            rows = []
            for name, value in self.fixture.source(root)["files"].items():
                if isinstance(value, dict):
                    mode, object_id = "160000", value["gitlinkCommit"]
                else:
                    data = (root / name).read_bytes()
                    blob = b"blob " + str(len(data)).encode("ascii") + b"\0" + data
                    mode, object_id = "100644", hashlib.sha1(blob).hexdigest()
                rows.append((mode + " " + object_id + " 0\t" + name).encode("utf-8") + b"\0")
            external_indexes[root] = b"".join(rows)

        def git_with_stage(root, *args, **kwargs):
            result = self.fixture.git(root, *args, **kwargs)
            if args and args[0] == "ls-files" and "--stage" in args and root in external_indexes:
                result.stdout = external_indexes[root]
            return result
        self.fixture.management.git = git_with_stage

    def inspect(self, *, owned_hashes=None, declared=True):
        fixture = self.fixture
        payload = {**fixture.payload("inspect-worktree", "sealed-inspect", "w"),
                   "targetSourceOperationKey": fixture.creator, "externalRoots": [str(fixture.external)]}
        if declared:
            payload["sourceHashes"] = ({"owned.txt": hashlib.sha256(fixture.desired["owned.txt"]).hexdigest()}
                                       if owned_hashes is None else owned_hashes)
        fixture.use(payload)
        result = helper.managed_inspect_worktree(fixture.management, preparation_helper=helper)
        key = fixture.complete(payload, result)
        reference = result["preparationInputRef"]
        self.inputs = json.loads(Path(reference["path"]).read_bytes())["preparationInput"]
        fixture.state.authorize_session(fixture.repo_id, "r", "user",
            sorted([*self.inputs["currentHashes"],
                    *["external/zr_vm/" + name for value in self.inputs["externalInputs"]
                      for name in value["currentHashes"]]]),
            ["validation"], {"source": "user", "task": "complete sealed fixture input"})
        return key, reference

    def prepare(self, inspection, reference, *, request="sealed-prepare"):
        fixture = self.fixture
        fixture.use({**fixture.payload("prepare-worktree", request, "r"),
                     "inspectionOperationKey": inspection, "preparationInputRef": reference})
        with baseline.lifetime.writer_hold(fixture.management):
            result = helper.managed_prepare_worktree(fixture.management, preparation_helper=helper)
        return fixture.complete(fixture.management.payload, result), result

    def test_complete_capture_survives_later_source_and_external_edits(self):
        fixture = self.fixture
        inspection, reference = self.inspect()
        captured = copy.deepcopy(self.inputs["sealedSource"]["externalCapture"])
        self.assertEqual(7, len(self.inputs["externalGitlinks"]))
        fixture.write(fixture.repo, {"owned.txt": b"later owned version\n", "foreign.bin": b"later foreign version"})
        fixture.write(fixture.external, {"src/lib.rs": b"later sibling version"})
        fixture.write(fixture.external / "vendor/v6", {"src/lib.rs": b"later registered Gitlink version"})
        key, result = self.prepare(inspection, reference)
        self.assertEqual(fixture.desired["owned.txt"], (fixture.target / "owned.txt").read_bytes())
        self.assertEqual(fixture.desired["foreign.bin"], (fixture.target / "foreign.bin").read_bytes())
        prepared = baseline.support.preparation(fixture.state, key, "w-t")["payload"]
        self.assertEqual(captured, prepared["externalRoots"])
        self.assertEqual(self.inputs["sealedSource"]["sourceManifest"], prepared["sourceManifest"])
        self.assertEqual("sealed-observed-canonical-input", prepared["sourceKind"])
        self.assertFalse(result["canonicalSourceModified"])
        self.assertEqual(fixture.writer["snapshotDigest"], prepared["writerAuthorizationSnapshot"])

    def test_canonical_deletion_and_external_rename_do_not_reopen_sources(self):
        fixture = self.fixture
        inspection, reference = self.inspect(declared=False)
        (fixture.repo / "foreign.bin").unlink()
        (fixture.repo / "owned.txt").rename(fixture.repo / "later-owned.txt")
        fixture.external.rename(fixture.area / "moved-zr_vm")
        original_source = fixture.management.source

        def only_target(root, **kwargs):
            self.assertEqual(fixture.target, root, "Preparation reopened a mutable canonical or sibling source")
            return original_source(root, **kwargs)

        with patch.object(fixture.management, "source", side_effect=only_target):
            _, result = self.prepare(inspection, reference)
        self.assertEqual(fixture.desired["owned.txt"], (fixture.target / "owned.txt").read_bytes())
        self.assertEqual(fixture.desired["foreign.bin"], (fixture.target / "foreign.bin").read_bytes())
        self.assertEqual("complete", result["status"])

    def test_corrupt_object_is_rejected_before_any_target_write(self):
        fixture = self.fixture
        inspection, reference = self.inspect()
        before = fixture.source(fixture.target)
        checksum = self.inputs["currentHashes"]["foreign.bin"]
        (fixture.cas / "inputs/objects" / checksum).write_bytes(b"corrupt object")
        with self.assertRaises(JenkinsError) as observed:
            self.prepare(inspection, reference)
        self.assertEqual("managed_source_object_changed", observed.exception.code)
        self.assertEqual(before, fixture.source(fixture.target))
        self.assertFalse(getattr(fixture.management, "_worktree_mutation_started", False))
        self.assertEqual([], fixture.state.list("managed_worktree_preparation"))

    def test_declared_owned_hash_drift_refuses_inventory_publication(self):
        fixture = self.fixture
        (fixture.repo / "owned.txt").write_bytes(b"foreign edit before capture")
        with self.assertRaises(JenkinsError) as observed:
            self.inspect()
        self.assertEqual("managed_owned_source_changed", observed.exception.code)
        report = fixture.cas / "management/reports" / fixture.management.key
        self.assertFalse((report / "source-inventory.json").exists())
        self.assertEqual(fixture.before["owned.txt"], (fixture.target / "owned.txt").read_bytes())

    def test_declared_owned_scope_must_be_exact(self):
        with self.assertRaises(JenkinsError) as observed:
            self.inspect(owned_hashes={"foreign.bin": "a" * 64})
        self.assertEqual("managed_owned_scope_invalid", observed.exception.code)
        self.assertFalse((self.fixture.cas / "inputs").exists())

    def test_legacy_inventory_retains_the_live_source_guard(self):
        fixture = self.fixture
        inspection, reference = fixture.inventory("legacy-inspect", fixture.creator)
        self.assertNotIn("sealedSource", fixture.inputs)
        before = fixture.source(fixture.target)
        (fixture.repo / "foreign.bin").write_bytes(b"changed after legacy inventory")
        with self.assertRaises(JenkinsError):
            self.prepare(inspection, reference)
        self.assertEqual(before, fixture.source(fixture.target))
        self.assertFalse(getattr(fixture.management, "_worktree_mutation_started", False))

    def long_document(self, marker, *, canonical):
        fixture = self.fixture
        padding = 273 - len(str(fixture.target / "docs" / (marker + ".md")))
        self.assertGreater(padding, 0)
        name = "docs/" + marker + "x" * padding + ".md"
        self.assertEqual(273, len(str(fixture.target / name)))
        self.assertLessEqual(len(Path(name).name), 255)
        before = b"existing long document\n"
        fixture.before[name] = before
        fixture.tracked.add(name)
        fixture.write(fixture.target, {name: before})
        if canonical is not None:
            fixture.desired[name] = canonical
            fixture.write(fixture.repo, {name: canonical})
        fixture.creator = fixture.complete(
            fixture.payload("create-worktree", "create-long-document-" + marker, "w"),
            {"worktreeId": "w-t", "source": fixture.source(fixture.target)})
        return name, before

    def test_273_character_unchanged_document_and_deletion_keep_native_checks(self):
        fixture = self.fixture
        retained, data = self.long_document("retained", canonical=b"existing long document\n")
        removed, _ = self.long_document("removed", canonical=None)
        inspection, reference = self.inspect()
        with patch.object(helper, "_managed_checksums", wraps=helper._managed_checksums) as checks:
            _, result = self.prepare(inspection, reference)
        self.assertEqual(data, (fixture.target / retained).read_bytes())
        self.assertFalse((fixture.target / removed).exists())
        self.assertEqual(hashlib.sha256(data).hexdigest(), result["afterHashes"][retained])
        self.assertIsNone(result["afterHashes"][removed])
        for phase in ("target-preimages", "target-postimages"):
            self.assertTrue(any(call.args[0] == fixture.target and call.args[2] == phase
                                and {retained, removed} <= set(call.args[1])
                                for call in checks.call_args_list), phase)

    def test_273_character_changed_present_document_still_refuses_writes(self):
        fixture = self.fixture
        name, before_data = self.long_document("changed", canonical=b"new long document version\n")
        inspection, reference = self.inspect()
        before = fixture.source(fixture.target)
        with self.assertRaises(JenkinsError) as observed:
            self.prepare(inspection, reference)
        self.assertEqual("managed_worktree_path_invalid", observed.exception.code)
        self.assertEqual(before_data, (fixture.target / name).read_bytes())
        self.assertEqual(before, fixture.source(fixture.target))
        self.assertFalse(getattr(fixture.management, "_worktree_mutation_started", False))
        self.assertEqual([], fixture.state.list("managed_worktree_preparation"))

    def test_cas_creation_denies_reads_until_the_complete_handle_closes(self):
        fixture = self.fixture
        native = helper._managed_native()
        original_write = native._write_handle
        observed = []

        def write_and_probe(handle, data):
            original_write(handle, data)
            if observed:
                return
            checksum = hashlib.sha256(data).hexdigest()
            with helper.managed_input_stage(fixture.cas, "exclusive-object-publication-read") as stage:
                with self.assertRaises(JenkinsError) as blocked:
                    stage.checksum("inputs/objects/" + checksum)
            self.assertEqual("managed_source_unavailable", blocked.exception.code)
            self.assertTrue(blocked.exception.retryable)
            self.assertEqual(32, blocked.exception.details["nativeError"])
            observed.append(checksum)

        with patch.object(native, "_write_handle", side_effect=write_and_probe):
            self.inspect()
        self.assertEqual(1, len(observed))
        with helper.managed_input_stage(fixture.cas, "complete-object-publication-read") as stage:
            self.assertEqual(observed[0], stage.checksum("inputs/objects/" + observed[0]))

    def test_native_capture_hash_and_payload_use_the_same_read(self):
        fixture = self.fixture
        expected = fixture.desired["owned.txt"]
        with helper.managed_input_stage(fixture.repo, "captured-file-contract") as stage:
            checksum, payload, mode = stage.capture("owned.txt")
            (fixture.repo / "owned.txt").write_bytes(b"later file version")
            self.assertNotEqual(checksum, stage.checksum("owned.txt"))
        self.assertEqual(expected, payload)
        self.assertEqual(hashlib.sha256(payload).hexdigest(), checksum)
        self.assertIn(mode, (0o444, 0o666))

        # Real Windows directory spelling survives a tracked old leaf and a lower-case new request.
        from tools.jenkins import managed_worktree as formal
        from tools.jenkins.resources.paths import canonical_build_root
        from tools.jenkins.source.external_inputs import KIND, verify_prepared
        from tools.jenkins.workflow import pools

        case_root = fixture.area / "case-evidence/zr_vm"
        (case_root / "tests/compileTime").mkdir(parents=True)
        (case_root / "tests/compileTime/old.c").write_bytes(b"tracked old leaf\n")
        (case_root / "tests/compileTime/deleted.c").write_bytes(b"tracked deletion\n")

        def git(*args):
            return subprocess.run(["git", "--no-optional-locks", "-c", "core.fsmonitor=false",
                "-c", "core.hooksPath=" + os.devnull, "-C", str(case_root), *args],
                capture_output=True, check=True)

        git("init", "-q")
        git("add", "--", "tests/compileTime/old.c", "tests/compileTime/deleted.c")
        git("-c", "user.name=Fixture", "-c", "user.email=fixture@invalid",
            "commit", "-qm", "physical spelling fixture")
        (case_root / "tests/compileTime/deleted.c").unlink()
        (case_root / "tests/compiletime/new.c").write_bytes(b"untracked new leaf\n")
        tree = {}
        for row in git("ls-tree", "-r", "-z", "--full-tree", "HEAD").stdout.split(b"\0"):
            if row:
                fields, name = row.split(b"\t", 1)
                tree[name.decode().casefold()] = (name.decode(), fields.decode().split()[2])
        actor = copy.copy(fixture.management)
        actor.repo = case_root.parent / "engine"
        actor.payload = {"externalRoots": [str(case_root)], "externalGitlinks": {},
                         "ownedPaths": ["owned.txt"]}
        actor.git = lambda root, *args: git(*args)
        actor.text = lambda root, *args: git(*args).stdout.decode("utf-8").strip()
        payload_before, writer_before = copy.deepcopy(actor.payload), copy.deepcopy(fixture.writer)
        index_before = git("ls-files", "--stage", "--debug", "-z").stdout
        capture = helper._ManagedCapturedExternal(actor, fixture.cas)
        values, raw_captures, _ = formal._external_capture(
            capture, type("Files", (), {"source_files": staticmethod(helper._managed_external_compiler_files)})(),
            inspection=True)
        raw = raw_captures[0]["source"]
        raw_before = helper.encoded(raw)
        self.assertEqual("full-file-index-v2", raw["sourceEvidencePolicy"])
        self.assertIn("tests/compiletime/new.c", values[0]["currentHashes"])
        # This fixture stubs base lookup globally; supply its real Git tree for this repository.
        with patch.object(formal, "_git_base_tree", return_value=tree):
            sealed = helper._managed_captured_manifest(case_root, raw["head"], values[0]["currentHashes"],
                                                       capture.flattened_metadata(case_root, raw))
        helper.verify_sealed_capture(raw, sealed, managed_external=True)
        self.assertEqual({"tests/compileTime/old.c", "tests/compileTime/new.c",
                          "tests/compileTime/deleted.c"}, {entry["path"] for entry in sealed["entries"]})
        layout = {"preparationKind": "current-source", "external": [
            {"kind": KIND, "name": "zr_vm", "repositoryRoot": str(case_root),
             "manifest": sealed, "sourceDigest": helper.digest(sealed)}]}
        prepared = fixture.area / "case-evidence/prepared/inputs/source"
        prepared.mkdir(parents=True)
        for entry in sealed["entries"]:
            if entry["status"] == "present":
                pools._atomic_copy(fixture.cas / "inputs/objects" / entry["objectDigest"],
                                   prepared.parent / "zr_vm" / entry["path"])
        self.assertIsNone(verify_prepared(layout, prepared, canonical_build_root(r"E:\cargo-targets")))
        self.assertEqual({"tests/compileTime/old.c", "tests/compileTime/new.c"},
            {path.relative_to(prepared.parent / "zr_vm").as_posix()
             for path in (prepared.parent / "zr_vm").rglob("*") if path.is_file()})
        self.assertEqual(raw_before, helper.encoded(raw))
        self.assertEqual(index_before, git("ls-files", "--stage", "--debug", "-z").stdout)
        self.assertEqual(payload_before, actor.payload)
        self.assertEqual(writer_before, fixture.writer)

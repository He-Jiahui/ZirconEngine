"""Candidate-only maintenance contracts; fixtures never compile or clone source."""
from contextlib import contextmanager, ExitStack
from dataclasses import dataclass
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from tools.jenkins.cli import dispatch
from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.resources.paths import ApprovedBuildRoot
from tools.jenkins.source import repository_id
from tools.jenkins.state import State
from tools.jenkins.workflow import build_cache_gc as gc


@dataclass(frozen=True)
class FileIdentity:
    file_id: bytes
    device: int
    size: int
    mtime_ns: int
    link_count: int


class FixtureStorage:
    def __init__(self, root):
        self.root = root
        self.deleted = []
        self.fail = None
        self.after_delete = None

    def delete(self, relative, *, expected_identity):
        if relative == self.fail:
            raise OSError("fixture sharing violation")
        path = self.root / relative
        info = path.stat()
        if info.st_ino.to_bytes(16, "big") != expected_identity.file_id:
            raise OSError("fixture file identity changed")
        path.unlink()
        self.deleted.append(relative)
        if self.after_delete: self.after_delete(relative)

    def exists(self, relative):
        return (self.root / relative).exists()


class BuildCacheMaintenanceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.repo = self.root / "repository"; self.repo.mkdir()
        self.audit = self.root / "cargo-targets" / "zircon-local" / "audit"
        self.candidate = self.audit / "runtime-check-d-r1" / "compile" / "build"
        self.candidate.mkdir(parents=True)
        (self.candidate / "a.bin").write_bytes(b"compiled-a")
        (self.candidate / "b.bin").write_bytes(b"compiled-bbb")
        self.target = self.candidate.parent / "target"; self.target.mkdir()
        (self.target / "retained.exe").write_bytes(b"target-retained")
        self.storage = FixtureStorage(self.candidate.parent)
        self.state = State(self.root / "fixture.sqlite3")
        self.identity = {"repositoryId": repository_id(self.repo), "sessionId": "gc-session", "requestId": "gc-request",
                         "attemptId": "gc-attempt", "generation": 1, "sourceInputDigest": "a" * 64,
                         "coverageDigest": "b" * 64, "stageImplementationDigest": "c" * 64}
        self.scope = {"candidatePath": str(self.candidate), "maxFiles": 10, "maxBytes": 1024}
        self.intent = {**self.identity, "operationId": "cache-gc-fixture", "cacheKind": gc.CACHE_KIND,
                       "cleanupScope": self.scope, "ownedPaths": ["tools/jenkins"]}
        self.state.authorize_session(self.identity["repositoryId"], self.identity["sessionId"], "fixture-user",
                                     ["tools/jenkins"], ["validation", "gc"], {"source": "user", "task": "bounded cache GC fixture"})
        self.state.submit_request(self.intent)
        self.pins = []
        for index, (name, _, _, field) in enumerate(gc.EVIDENCE_PINS):
            evidence_dir = self.audit / name; evidence_dir.mkdir(parents=True, exist_ok=True)
            native = self._native(index)
            request = ({"targetDir": str(self.target), "command": native["command"]} if field == "native" else
                       {"physicalPaths": {"target": str(self.target), "build": str(self.candidate)}})
            request_path = evidence_dir / "request.json"
            request_hash = self._write(request_path, request)
            reference = str(request_path) if field == "native" else {"path": str(request_path), "sha256": request_hash}
            receipt = {"request": reference, field: native, "JenkinsAcceptance": False}
            if field == "native": receipt["requestSha256"] = request_hash
            receipt_hash = self._write(evidence_dir / "receipt.json", receipt)
            (evidence_dir / "stdout.bin").write_bytes(b"preserved log")
            self.pins.append((name, request_hash, receipt_hash, field))
        self.history = {p: p.read_bytes() for p in self.audit.rglob("*") if p.is_file() and not p.is_relative_to(self.candidate)}
        self.stack = ExitStack(); self.addCleanup(self.stack.close)
        self.stack.enter_context(patch.object(gc, "CANDIDATE", str(self.candidate)))
        self.stack.enter_context(patch.object(gc, "EVIDENCE_PINS", tuple(self.pins)))
        approved = ApprovedBuildRoot(self.root / "cargo-targets", "fixture-volume")
        self.stack.enter_context(patch.object(gc, "_physical", return_value=(approved, self.candidate)))
        self.stack.enter_context(patch.object(gc, "physical_path_under", side_effect=self._physical_path))
        self.stack.enter_context(patch.object(gc, "identity_matches", return_value=False))
        self.native_probe = self.stack.enter_context(patch.object(gc, "_native_writers", return_value={"complete": True, "writers": []}))
        self.stack.enter_context(patch.object(gc, "_storage", side_effect=self._storage))
        self.stack.enter_context(patch.object(gc, "_bounded_files", side_effect=self._files))
        self.stack.enter_context(patch.object(gc, "_file_fact", side_effect=self._fact))

    def _native(self, index=0):
        pid, birth = 50000 + index, str(123456789 + index)
        return {"schemaVersion": 1, "command": [r"C:\fixture\pwsh.exe", "-File", "fixture.ps1"],
                "cwd": str(self.repo), "pid": pid, "creationTime": birth, "rootExitCode": 0 if not index else 101,
                "processTree": {"scope": "retained-windows-job", "terminal": True, "nativeWaiterJoined": True,
                                "readersFinished": True, "pipes": {name: {"eof": True, "closed": True} for name in ("stdout", "stderr")},
                                "pipeEOF": True, "jobHandleRetainedUntilCliExit": False},
                "nativeJobEvidence": {"access": "JOB_OBJECT_QUERY", "errors": [], "records": {
                    "beforeResume": {"activeProcesses": 1, "members": [{"pid": pid, "creationTime": birth,
                                          "exeBasename": "pwsh.exe", "member": True}]},
                    "afterTerminal": {"activeProcesses": 0}, "afterClose": {"activeProcesses": 0}}},
                "terminalErrors": [], "formalAcceptance": False}

    def _write(self, path, value):
        raw = json.dumps(value, sort_keys=True).encode()
        path.write_bytes(raw)
        return hashlib.sha256(raw).hexdigest()

    def _physical_path(self, approved, path, **_):
        selected = Path(path)
        if not selected.is_relative_to(approved.path):
            raise JenkinsError("namespace_escape", "fixture path escapes build root")
        return selected

    @contextmanager
    def _storage(self, *_):
        yield self.storage

    def _files(self, storage, limits, deadline):
        selected = storage.root / limits.get("inventoryPrefix", "build")
        files = sorted(selected.rglob("*"))
        if len([p for p in files if p.is_file()]) > limits["maxFiles"]:
            raise JenkinsError("cache_inventory_limit", "fixture file bound")
        for path in files:
            if path.is_symlink(): raise JenkinsError("cache_reparse_rejected", "fixture alias")
            if path.is_file(): yield path.relative_to(storage.root).as_posix()

    def _fact(self, storage, relative, maximum):
        path = storage.root / relative; info = path.stat(); raw = path.read_bytes()
        if len(raw) > maximum: raise JenkinsError("cache_inventory_limit", "fixture byte bound")
        identity = FileIdentity(info.st_ino.to_bytes(16, "big"), info.st_dev, info.st_size, info.st_mtime_ns, info.st_nlink)
        return {"path": relative, "sha256": hashlib.sha256(raw).hexdigest(), "byteLength": len(raw),
                "fileId": identity.file_id.hex(), "device": info.st_dev, "mtimeNs": info.st_mtime_ns,
                "linkCount": info.st_nlink}, identity

    def _payload(self, phase):
        value = {"identity": dict(self.identity), "operationId": self.intent["operationId"], "cacheKind": gc.CACHE_KIND,
                 "cleanupScope": dict(self.scope), "ownedPaths": list(self.intent["ownedPaths"]), "phase": phase}
        if "reviewedProtection" in self.intent:
            value["reviewedProtection"] = self.intent["reviewedProtection"]
        inventory = self.state.get("maintenance_cache_inventory", digest(self._authority()))
        if inventory: value["inventoryDigest"] = inventory["payload"]["inventoryDigest"]
        return value

    def _authority(self):
        authority = {"identity": self.identity, "operationId": self.intent["operationId"],
                     "cleanupScope": {"maxSeconds": 300, **self.scope}, "requestPayloadDigest": digest(self.intent),
                     "authorizationSnapshot": self.state.check_authorization(self.identity["repositoryId"], self.identity["sessionId"], "gc")["snapshotDigest"]}
        if "reviewedProtection" in self.intent:
            authority["reviewedProtection"] = self.intent["reviewedProtection"]
        return authority

    def _call(self, phase, **updates):
        return dispatch("maintenance", "cache-maintenance", {**self._payload(phase), **updates}, self.state, self.repo)

    def _intake_inventory(self):
        self._call("intake")
        return self._call("inventory")

    def _select_fixed_subtree(self, reviewed_protection=None):
        selected = self.candidate.joinpath(*gc.RELATIVE_SUBTREE.split("/"))
        selected.mkdir(parents=True)
        (selected / "a.bin").write_bytes(b"selected-a")
        (selected / "b.bin").write_bytes(b"selected-bbb")
        self.identity["requestId"] = "scoped-subtree-request"
        self.intent = {**self.intent, **self.identity, "operationId": "scoped-cache-gc-fixture",
                       "cleanupScope": {**self.scope, "relativeSubtree": gc.RELATIVE_SUBTREE}}
        self.scope = self.intent["cleanupScope"]
        if reviewed_protection is not None:
            self.intent["reviewedProtection"] = reviewed_protection
        self.state.submit_request(self.intent)
        return selected

    def test_fixed_subtree_public_inventory_chunks_preserve_all_siblings(self):
        selected = self._select_fixed_subtree()
        prefix = "build/" + gc.RELATIVE_SUBTREE
        untouched = {path: path.read_bytes() for path in self.audit.rglob("*")
                     if path.is_file() and not path.is_relative_to(selected)}
        inventory = self._intake_inventory()
        self.assertEqual(inventory["result"]["candidatePath"], str(selected))
        self.assertEqual(inventory["result"]["baseCandidatePath"], str(self.candidate))
        self.assertEqual(inventory["result"]["relativeSubtree"], gc.RELATIVE_SUBTREE)
        stored = self.state.list("maintenance_cache_inventory")[0]["payload"]
        self.assertTrue(stored["complete"])
        self.assertEqual(stored["candidatePath"], str(selected))
        self.assertEqual([item["path"] for item in stored["files"]], [prefix + "/a.bin", prefix + "/b.bin"])
        yielded = self._yield_after_one_delete()
        self.assertEqual(yielded["status"], "waiting")
        self.assertEqual(self._call("reconcile")["status"], "waiting")
        completed = self._call("gc")
        self.assertEqual(completed["status"], "accepted")
        self.assertEqual(completed["result"]["candidatePath"], str(selected))
        self.assertEqual(completed["result"]["reclaimedBytes"], len(b"selected-a") + len(b"selected-bbb"))
        self.assertEqual(self.storage.deleted, [prefix + "/a.bin", prefix + "/b.bin"])
        self.assertEqual(list(selected.iterdir()), [])
        for path, raw in untouched.items(): self.assertEqual(path.read_bytes(), raw)

    def test_fixed_subtree_rejects_arbitrary_names_or_new_scope_fields(self):
        for name in ("debug/deps", "../target", "debug/incremental/foreign", None, []):
            with self.subTest(name=name), self.assertRaises(JenkinsError) as error:
                gc._scope({**self.scope, "relativeSubtree": name})
            self.assertEqual(error.exception.code, "cache_scope_invalid")
        with self.assertRaises(JenkinsError) as error:
            gc._scope({**self.scope, "relativeSubtree": gc.RELATIVE_SUBTREE, "protectedPaths": ["build/a.bin"]})
        self.assertEqual(error.exception.code, "cache_scope_invalid")

    def test_fixed_subtree_physical_alias_denies_before_intake(self):
        selected = self._select_fixed_subtree()
        path_gate = self._physical_path
        def reject_selected(approved, path, **options):
            if Path(path) == selected:
                raise JenkinsError("build_root_reparse_point", "fixture subtree junction")
            return path_gate(approved, path, **options)
        with patch.object(gc, "physical_path_under", side_effect=reject_selected):
            with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "build_root_reparse_point")
        self.assertEqual(self.state.list("maintenance_build_cache"), [])
        self.assertEqual(self.storage.deleted, [])

    def test_fixed_subtree_hardlink_blocks_inventory_without_affecting_siblings(self):
        selected = self._select_fixed_subtree()
        from tools.jenkins.pilot.native.windows_storage import WindowsPathError
        with patch.object(gc, "_file_fact", side_effect=WindowsPathError("worker file has a hard-link alias")):
            self._call("intake")
            with self.assertRaises(JenkinsError) as error: self._call("inventory")
        self.assertEqual(error.exception.code, "cache_alias_rejected")
        self.assertEqual(self.state.list("maintenance_cache_inventory"), [])
        self.assertEqual(self.state.list("maintenance_cache_gc"), [])
        self.assertTrue((selected / "a.bin").is_file())
        self.assertEqual(self.storage.deleted, [])

    def test_public_intake_inventory_gc_retains_all_history_and_target(self):
        inventory = self._intake_inventory()
        self.assertEqual(inventory["status"], "accepted")
        self.assertEqual(inventory["result"]["fileCount"], 2)
        self.assertNotIn("files", inventory["result"])
        self.assertEqual(len(inventory["inventoryDigest"]), 64)
        result = self._call("gc")["result"]
        self.assertEqual(result["status"], "complete")
        self.assertEqual(result["reclaimedBytes"], len(b"compiled-a") + len(b"compiled-bbb"))
        self.assertFalse(result["JenkinsAcceptance"])
        self.assertTrue(self.candidate.is_dir())
        self.assertEqual(list(self.candidate.iterdir()), [])
        for path, original in self.history.items(): self.assertEqual(path.read_bytes(), original)
        repeated = self._call("gc")["result"]
        self.assertEqual(repeated, result)
        self.assertEqual(len(self.storage.deleted), 2)
        top = self.state.get("maintenance_cache_gc", self.intent["operationId"])["payload"]
        self.assertNotIn("removed", top)
        self.assertLess(len(json.dumps(top)), 2048)
        self.assertEqual(len(self.state.list("maintenance_cache_gc_file")), 2)

    def test_intake_is_idempotent_and_writes_no_cache_bytes(self):
        first = self._call("intake")
        row = self.state.list("maintenance_build_cache")[0]
        self.assertEqual(self._call("intake"), first)
        self.assertEqual(self.state.list("maintenance_build_cache")[0]["version"], row["version"])
        self.assertEqual(self.storage.deleted, [])
        self.assertEqual((self.candidate / "a.bin").read_bytes(), b"compiled-a")

    def test_empty_caller_scope_does_not_bypass_registered_grant(self):
        with self.assertRaises(JenkinsError) as error: self._call("intake", ownedPaths=[])
        self.assertEqual(error.exception.code, "request_payload_mismatch")
        self.assertEqual(self.state.list("maintenance_build_cache"), [])

    def test_missing_gc_grant_denies_intake(self):
        self.state.revoke_session(self.identity["repositoryId"], self.identity["sessionId"])
        with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "action_not_authorized")
        self.assertEqual(self.state.list("maintenance_build_cache"), [])

    def test_generation_conflict_and_unregistered_identity_deny_before_intake(self):
        for changed, expected in (({"generation": 2}, "request_generation_mismatch"),
                                  ({"requestId": "not-registered"}, "request_not_found")):
            with self.subTest(changed=changed), self.assertRaises(JenkinsError) as error:
                self._call("intake", identity={**self.identity, **changed})
            self.assertEqual(error.exception.code, expected)
        self.assertEqual(self.state.list("maintenance_build_cache"), [])

    def test_caller_cannot_select_target_sibling_or_expand_bytes(self):
        for scope in ({**self.scope, "candidatePath": str(self.target)}, {**self.scope, "maxBytes": 2048}):
            with self.subTest(scope=scope), self.assertRaises(JenkinsError): self._call("intake", cleanupScope=scope)
        self.assertEqual(self.state.list("maintenance_build_cache"), [])

    def test_original_receipt_hash_tamper_denies_intake(self):
        receipt = self.audit / self.pins[0][0] / "receipt.json"
        receipt.write_bytes(receipt.read_bytes() + b" ")
        with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "cache_evidence_changed")
        self.assertEqual(self.state.list("maintenance_build_cache"), [])

    def test_native_terminal_boolean_without_full_tuple_is_rejected(self):
        for native in ({"processTree": {"terminal": True}}, {**self._native(), "creationTime": ""}):
            with self.subTest(native=native), self.assertRaises(JenkinsError) as error: gc._terminal(native, self.repo)
            self.assertEqual(error.exception.code, "native_termination_unproven")

    def test_malformed_native_nested_records_are_typed_unproven(self):
        native = self._native()
        values = [{**native, "processTree": None}, {**native, "nativeJobEvidence": []},
                  {**native, "processTree": {**native["processTree"], "pipes": []}},
                  {**native, "nativeJobEvidence": {**native["nativeJobEvidence"], "records": []}}]
        for stage in ("beforeResume", "afterTerminal", "afterClose"):
            values.append({**native, "nativeJobEvidence": {**native["nativeJobEvidence"],
                           "records": {**native["nativeJobEvidence"]["records"], stage: []}}})
        for value in values:
            with self.subTest(value=value), self.assertRaises(JenkinsError) as error: gc._terminal(value, self.repo)
            self.assertEqual(error.exception.code, "native_termination_unproven")

    def test_reparse_authority_failure_denies_intake(self):
        with patch.object(gc, "_physical", side_effect=JenkinsError("build_root_reparse_point", "fixture junction")):
            with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "build_root_reparse_point")
        self.assertEqual(self.state.list("maintenance_build_cache"), [])

    def test_live_native_or_incomplete_probe_denies_intake(self):
        for code in ("cache_writer_active", "cache_ownership_unknown"):
            self.native_probe.side_effect = JenkinsError(code, "fixture process observation")
            with self.subTest(code=code), self.assertRaises(JenkinsError) as error: self._call("intake")
            self.assertEqual(error.exception.code, code)
        self.assertEqual(self.state.list("maintenance_build_cache"), [])

    def test_candidate_hold_blocks_but_unrelated_explicit_namespace_is_retained(self):
        self.state.put("pool_writer_hold", "foreign", {"status": "active", "path": str(self.candidate)})
        with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "cache_ownership_unknown")
        self.state.put("pool_writer_hold", "foreign", {"status": "active", "path": r"D:\cargo-targets\zircon-local\zircon-jenkins\pools\foreign"})
        self._call("intake")
        self.assertEqual(self.state.get("pool_writer_hold", "foreign")["payload"]["status"], "active")

    def test_unknown_hold_blocks_even_with_empty_caller_paths(self):
        self.state.put("resource_reservation", "unknown", {"status": "active", "owner": "unknown", "buildRoot": r"D:\cargo-targets"})
        with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "cache_ownership_unknown")

    def test_object_store_root_alone_cannot_prove_a_compiler_or_hold_is_unrelated(self):
        for domain in ("execution", "pool_writer_hold", "resource_reservation"):
            with self.subTest(domain=domain):
                self.state.put(domain, "object-only", {"status": "active", "objectRoot": r"F:\cargo-targets\zircon-local\zircon-jenkins"})
                with self.assertRaises(JenkinsError) as error:
                    self._call("intake")
                self.assertEqual(error.exception.code, "cache_ownership_unknown")
                self.state.put(domain, "object-only", {"status": "released"})
        self.assertEqual(self.state.list("maintenance_build_cache"), [])

    def test_previous_boot_native_record_is_retained_without_releasing_holds(self):
        native = {"status": "running", "pid": 10001, "birthToken": "100", "executable": r"C:\fixture\cargo.exe"}
        self.state.put("native_job", "old-boot", native)
        with patch("tools.jenkins.processes.boot.windows_boot_filetime", return_value=200):
            self._call("intake")
        self.assertEqual(self.state.get("native_job", "old-boot")["payload"], native)
        self.state.put("pool_writer_hold", "unproven", {"status": "active", "owner": "old-boot"})
        with patch("tools.jenkins.processes.boot.windows_boot_filetime", return_value=200):
            with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "cache_ownership_unknown")
        self.assertEqual(self.state.get("pool_writer_hold", "unproven")["payload"]["status"], "active")

    def test_same_boot_or_unknown_boot_native_cannot_be_ignored(self):
        native = {"status": "running", "pid": 10001, "birthToken": "300", "executable": r"C:\fixture\cargo.exe"}
        self.state.put("native_job", "current-boot", native)
        with patch("tools.jenkins.processes.boot.windows_boot_filetime", return_value=200):
            with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "cache_ownership_unknown")
        with patch("tools.jenkins.processes.boot.windows_boot_filetime", side_effect=JenkinsError("boot_evidence_unavailable", "fixture")):
            with self.assertRaises(JenkinsError) as error: self._call("intake")
        self.assertEqual(error.exception.code, "boot_evidence_unavailable")

    def test_inventory_byte_bound_does_not_register_incomplete_inventory(self):
        self._call("intake")
        (self.candidate / "too-large.bin").write_bytes(b"x" * 2048)
        with self.assertRaises(JenkinsError) as error: self._call("inventory")
        self.assertEqual(error.exception.code, "cache_inventory_limit")
        self.assertEqual(self.state.list("maintenance_cache_inventory"), [])

    def test_inventory_bound_and_mutation_prevent_any_deletion(self):
        self._intake_inventory()
        (self.candidate / "a.bin").write_bytes(b"foreign-new-bytes")
        with self.assertRaises(JenkinsError) as error: self._call("gc")
        self.assertEqual(error.exception.code, "cache_inventory_changed")
        self.assertEqual(self.storage.deleted, [])
        self.assertEqual(self.state.list("maintenance_cache_gc"), [])

    def test_wrong_inventory_digest_prevents_any_deletion(self):
        self._intake_inventory()
        with self.assertRaises(JenkinsError) as error: self._call("gc", inventoryDigest="e" * 64)
        self.assertEqual(error.exception.code, "cache_inventory_mismatch")
        self.assertEqual(self.storage.deleted, [])

    def test_partial_gc_preserves_actual_bytes_and_reconcile_never_retries(self):
        self._intake_inventory(); self.storage.fail = "build/b.bin"
        response = self._call("gc")
        self.assertEqual(response["status"], "blocked")
        self.assertEqual(response["reasonCode"], "cache_gc_partial")
        result = response["result"]
        self.assertEqual(result["status"], "partial")
        self.assertEqual(result["reclaimedBytes"], len(b"compiled-a"))
        self.assertEqual(result["unreclaimedBytes"], len(b"compiled-bbb"))
        self.assertTrue((self.candidate / "b.bin").exists())
        self.storage.fail = None
        reconciled = self._call("reconcile")["result"]
        self.assertEqual(reconciled["status"], "unknown")
        self.assertTrue(reconciled["pendingDeletionUnacknowledged"])
        self.assertEqual(reconciled["reclaimedBytes"], len(b"compiled-a"))
        self.assertEqual(self.storage.deleted, ["build/a.bin"])

    def test_cancellation_after_inventory_denies_gc_and_keeps_history(self):
        self._intake_inventory()
        self.state.put("cancellation", ":".join(self.identity[key] for key in gc._IDS[:3]), {"generation": 1})
        with self.assertRaises(JenkinsError) as error: self._call("gc")
        self.assertEqual(error.exception.code, "maintenance_cancelled")
        self.assertEqual(self.storage.deleted, [])

    def test_unindexed_cache_cannot_skip_intake(self):
        with self.assertRaises(JenkinsError) as error: self._call("gc")
        self.assertEqual(error.exception.code, "cache_intake_missing")
        self.assertEqual(self.storage.deleted, [])

    def _yield_after_one_delete(self):
        ticks = {"expired": False}
        def after_delete(_): ticks["expired"] = True
        self.storage.after_delete = after_delete
        with patch.object(gc.time, "monotonic", side_effect=lambda: 1000 if ticks["expired"] else 0):
            return self._call("gc")

    def test_time_budget_yields_without_unknown_and_explicit_same_operation_resumes(self):
        self._intake_inventory()
        result = self._yield_after_one_delete()
        self.assertEqual(result["status"], "waiting")
        self.assertEqual(result["result"]["status"], "yielded")
        self.assertIsNone(result["result"]["pendingFile"])
        self.assertEqual(result["result"]["operationId"], self.intent["operationId"])
        self.assertEqual(result["result"]["unknownFileCount"], 0)
        self.assertEqual(result["result"]["reclaimedBytes"], len(b"compiled-a"))
        top = self.state.get("maintenance_cache_gc", self.intent["operationId"])["payload"]
        self.assertIsNone(top["pendingFile"])
        self.assertEqual(top["unknown"], [])
        self.assertEqual(self._call("reconcile")["status"], "waiting")
        completed = self._call("gc")
        self.assertEqual(completed["status"], "accepted")
        self.assertEqual(completed["result"]["status"], "complete")
        self.assertEqual(completed["result"]["removedFileCount"], 2)
        self.assertEqual(self.storage.deleted, ["build/a.bin", "build/b.bin"])

    def test_new_or_changed_file_between_chunks_denies_resume(self):
        self._intake_inventory(); self._yield_after_one_delete()
        (self.candidate / "foreign.bin").write_bytes(b"foreign")
        with self.assertRaises(JenkinsError) as error: self._call("gc")
        self.assertEqual(error.exception.code, "cache_inventory_changed")
        self.assertEqual(self.storage.deleted, ["build/a.bin"])

    def test_reappeared_acknowledged_file_denies_resume(self):
        self._intake_inventory(); self._yield_after_one_delete()
        (self.candidate / "a.bin").write_bytes(b"compiled-a")
        with self.assertRaises(JenkinsError) as error: self._call("gc")
        self.assertEqual(error.exception.code, "cache_deletion_unknown")
        self.assertEqual(self.storage.deleted, ["build/a.bin"])

    def test_complete_without_all_positive_acknowledgements_is_not_accepted(self):
        self._intake_inventory(); self._yield_after_one_delete()
        row = self.state.get("maintenance_cache_gc", self.intent["operationId"])
        self.state.put("maintenance_cache_gc", row["key"], {**row["payload"], "status": "complete"})
        with self.assertRaises(JenkinsError) as error: self._call("gc")
        self.assertEqual(error.exception.code, "cache_deletion_unknown")
        self.assertEqual(self.storage.deleted, ["build/a.bin"])

    def test_malformed_ack_payload_file_and_status_are_typed_unknown(self):
        self._intake_inventory(); self._yield_after_one_delete()
        inventory = self.state.list("maintenance_cache_inventory")[0]["payload"]
        item = inventory["files"][1]
        valid = {"operationId": self.intent["operationId"], "inventoryDigest": inventory["inventoryDigest"],
                 "file": item, "status": "pending"}
        malformed = [None, [], {}, {**valid, "file": None}, {**valid, "file": []},
                     {**valid, "file": {"path": []}}, {**valid, "status": []}, {**valid, "operationId": []}]
        for value in malformed:
            with self.subTest(value=value):
                self.state.put("maintenance_cache_gc_file", "malformed-ack", value)
                with self.assertRaises(JenkinsError) as error: self._call("gc")
                self.assertEqual(error.exception.code, "cache_deletion_unknown")
                self.assertEqual(self.storage.deleted, ["build/a.bin"])

    def test_tampered_complete_digest_shape_and_semantics_deny_replay(self):
        self._intake_inventory(); self._call("gc")
        operation = self.intent["operationId"]
        original = self.state.get("maintenance_cache_gc", operation)["payload"]
        changes = [{"resultDigest": "e" * 64}, {"unknown": [{"path": "build/a.bin", "reasonCode": "changed"}]},
                   {"unreclaimedBytes": 1}, {"historyPreserved": False}, {"pendingDeletionUnacknowledged": True},
                   {"pendingFile": self.state.list("maintenance_cache_inventory")[0]["payload"]["files"][0]},
                   {"status": []}, {"reclaimedBytes": True}]
        for changed in changes:
            with self.subTest(changed=changed):
                value = {**original, **changed}
                if "resultDigest" not in changed:
                    value["resultDigest"] = digest({k: v for k, v in value.items() if k != "resultDigest"})
                self.state.put("maintenance_cache_gc", operation, value)
                with self.assertRaises(JenkinsError) as error: self._call("reconcile")
                self.assertEqual(error.exception.code, "cache_deletion_unknown")
        self.assertEqual(self.storage.deleted, ["build/a.bin", "build/b.bin"])

    def test_tampered_yielded_digest_and_semantics_cannot_continue(self):
        self._intake_inventory(); self._yield_after_one_delete()
        operation = self.intent["operationId"]
        original = self.state.get("maintenance_cache_gc", operation)["payload"]
        remaining = self.state.list("maintenance_cache_inventory")[0]["payload"]["files"][1]
        for changed in ({"resultDigest": "f" * 64}, {"pendingFile": remaining}, {"unknown": [None]},
                        {"unreclaimedBytes": 0}, {"historyPreserved": False}, {"pendingDeletionUnacknowledged": True}):
            with self.subTest(changed=changed):
                value = {**original, **changed}
                if "resultDigest" not in changed:
                    value["resultDigest"] = digest({k: v for k, v in value.items() if k != "resultDigest"})
                self.state.put("maintenance_cache_gc", operation, value)
                with self.assertRaises(JenkinsError) as error: self._call("gc")
                self.assertEqual(error.exception.code, "cache_deletion_unknown")
                self.assertEqual(self.storage.deleted, ["build/a.bin"])

    def test_unacknowledged_post_unlink_file_is_never_credited_or_redeleted(self):
        self._intake_inventory()
        inventory = self.state.list("maintenance_cache_inventory")[0]["payload"]
        item = inventory["files"][0]; operation = self.intent["operationId"]
        (self.candidate.parent / item["path"]).unlink()
        self.state.put("maintenance_cache_gc_file", digest([operation, item["path"]]),
                       {"operationId": operation, "inventoryDigest": inventory["inventoryDigest"], "file": item, "status": "pending"})
        self.state.put("maintenance_cache_gc", operation,
                       {"operationId": operation, "authorityDigest": digest(self._authority()),
                        "candidateRef": digest({"candidatePath": str(self.candidate)}),
                        "inventoryDigest": inventory["inventoryDigest"], "status": "running", "pendingFile": item,
                        "reclaimedBytes": 0, "removedFileCount": 0, "unknown": [], "historyPreserved": True,
                        "JenkinsAcceptance": False, "milestoneAcceptance": False})
        for phase in ("reconcile", "gc"):
            response = self._call(phase)
            self.assertEqual(response["status"], "blocked")
            self.assertEqual(response["result"]["status"], "unknown")
            self.assertEqual(response["result"]["reclaimedBytes"], 0)
        self.assertEqual(self.storage.deleted, [])

    def test_revoked_request_and_unknown_hold_between_chunks_deny_resume(self):
        self._intake_inventory(); self._yield_after_one_delete()
        self.state.put("build_cache_ref", "new-unknown", {"status": "active", "owner": "foreign"})
        with self.assertRaises(JenkinsError) as error: self._call("gc")
        self.assertEqual(error.exception.code, "cache_ownership_unknown")
        self.state.put("build_cache_ref", "new-unknown", {"status": "released"})
        self.state.put("cancellation", ":".join(self.identity[key] for key in gc._IDS[:3]), {"generation": 1})
        with self.assertRaises(JenkinsError) as error: self._call("gc")
        self.assertEqual(error.exception.code, "maintenance_cancelled")
        self.assertEqual(self.storage.deleted, ["build/a.bin"])


if __name__ == "__main__":
    unittest.main()

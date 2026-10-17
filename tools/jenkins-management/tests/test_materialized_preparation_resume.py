"""Recover a physically materialized interrupted preparation without rewriting it."""
from __future__ import annotations

import copy
import ctypes
from ctypes import wintypes
import hashlib
import importlib.util
import json
import os
from pathlib import Path
from types import SimpleNamespace
import sys
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins import managed_worktree as formal

ROOT = Path(__file__).resolve().parents[3]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


sealed = load("materialized_resume_existing_fixture",
              ROOT / "tools/jenkins-management/tests/test_managed_worktree_sealed_inventory.py")
helper = sealed.helper
runner = load("materialized_resume_runner", ROOT / "tools/jenkins-management/runner.py")


@unittest.skipUnless(os.name == "nt", "Real materialization and native pins require Windows")
class MaterializedPreparationResumeTests(unittest.TestCase):
    def setUp(self):
        self.sealed = sealed.ManagedWorktreeSealedInventoryTests(methodName="runTest")
        self.addCleanup(self.sealed.doCleanups)
        self.sealed.setUp()
        self.fixture = self.sealed.fixture
        f = self.fixture
        inspection, reference = self.sealed.inspect()
        self.inputs = self.sealed.inputs
        self.original = {**f.payload("prepare-worktree", "interrupted", "r"),
                         "inspectionOperationKey": inspection, "preparationInputRef": reference}
        f.use(self.original)
        self.actor = runner.Management.__new__(runner.Management)
        self.actor.__dict__.update(f.management.__dict__)
        self.actor.config = {**f.management.config, "managedWorktree": runner.MANAGED_WORKTREE_CAPABILITY}
        self.actor.state = helper.scoped_authorization_state(f.state)
        self.actor.action, self.actor.query = "prepare-worktree", False
        self.actor.request = self.actor.operation = "interrupted"
        self.addCleanup(patch.stopall)
        patch.object(runner, "source_helper", return_value=helper).start()
        original_source = f.source
        calls = [0]

        def crash_after_postimages(root, **kwargs):
            calls[0] += 1
            if calls[0] == 3:
                raise TimeoutError("simulated crash after complete target postimages")
            return original_source(root, **kwargs)
        self.actor.source = crash_after_postimages
        failed = self.actor.run()
        self.assertEqual("unknown", failed["status"])
        self.assertIn("simulated crash", failed["message"])
        self.old_key = self.actor.key
        self.old_operation = f.state.get("management_operation", self.old_key)
        self.old_preparation = f.state.get("managed_worktree_preparation", self.old_key)
        self.old_writer = f.state.get("managed_worktree_writer", "w-t")
        self.assertEqual("materializing", self.old_preparation["payload"]["status"])
        self.assertEqual("interrupted", self.old_writer["payload"]["status"])
        self.assertEqual(f.desired["owned.txt"], (f.target / "owned.txt").read_bytes())
        self.owner = {"driverDigest": "d" * 64, "operationId": "start-test", "fixtureOwner": True}
        self.build = {"job": "zircon-management", "number": 55, "queueId": 2899,
                      "building": False, "result": "FAILURE"}
        self.console = (self.old_key + " interrupted\n[Pipeline] End of Pipeline\nFinished: FAILURE\n").encode()
        self.receipt = helper.encoded(failed)
        self.report = f.repo / ".jenkins/state/management/reports/terminal"
        self.native = [{"role": role, "pid": number, "creationTime": str(10000 + number),
            "executable": str(f.area / ("python.exe" if role == "runner" else role + ".exe")),
            "observation": "original-process-absent", "retainedHandleSignaled": None, "exitCode": None}
            for role, number in (("runner", 991001), ("git-wrapper", 991002), ("git-child", 991003))]
        observation = {"kind": "recorded-earlier-live-process-observation",
            "operationKey": self.old_key, "buildNumber": 55, "queueId": 2899,
            "native": [{name: value[name] for name in ("role", "pid", "creationTime", "executable")}
                       for value in self.native]}
        home = f.repo / ".jenkins/jenkins_home/jobs/zircon-management/builds/55/archive/receipt.json"
        controls = f.repo / ".jenkins/state/controls/receipt.json"
        proof = {"schemaVersion": 1, "kind": "jenkins-prepare-recovery-terminal-v1",
            "status": "original-attempt-terminal", "operationKey": self.old_key, "worktreeId": "w-t",
            "driverDigest": "d" * 64, "runtimeOperationId": "start-test", "build": self.build,
            "fullPipelineFinished": True, "serviceOwnerUnchanged": True, "keeperUnchanged": True,
            "serviceOwner": self.owner, "native": self.native,
            "consoleRef": self.document(self.report / "console.txt", self.console),
            "receiptRef": self.document(controls, self.receipt),
            "homeReceiptRef": self.document(home, self.receipt),
            "originalNativeObservationRef": self.document(self.report / "observed.json", helper.encoded(observation)),
            "recordCurrentOperationPin": {"key": self.old_key, "version": self.old_operation["version"],
                "payloadDigest": helper.digest(self.old_operation["payload"]),
                "inputDigest": self.old_operation["payload"]["inputDigest"]},
            "originalPreparationPin": {"key": self.old_key, "version": self.old_preparation["version"],
                "payloadDigest": helper.digest(self.old_preparation["payload"])}}
        self.proof_ref = self.document(self.report / "verification.json", helper.encoded(proof))
        self.new_payload = {**self.original, "requestId": "recovery", "operationId": "recovery",
            "sourceEvidencePolicy": "full-file-index-v2", "resumePreparationFromOperationKey": self.old_key,
            "prepareRecoveryTerminalRef": self.proof_ref}
        f.use(self.new_payload)
        self.actor.payload, self.actor.key, self.actor.input_digest = (
            f.management.payload, f.management.key, f.management.input_digest)
        self.actor.request = self.actor.operation = "recovery"
        del self.actor.__dict__["source"]
        self.actor.git = self.git
        self.actor.manager = SimpleNamespace(_request=self.http)
        self.real_absent = helper._native_original_absent
        patch.object(helper, "_native_original_absent").start()
        patch.object(helper, "_recovery_no_descendants").start()
        patch("tools.jenkins.deployment.drain_observations.owner_observation", return_value=self.owner).start()
        self.before_bytes = {path.relative_to(f.target).as_posix(): path.read_bytes()
                             for path in f.target.rglob("*") if path.is_file()}

    def document(self, path, raw):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(raw)
        return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "size": len(raw)}

    def http(self, path):
        if path.endswith("api/json?tree=number,queueId,building,result"):
            raw = helper.encoded(self.build)
        elif path.endswith("consoleText"):
            raw = self.console
        elif path.endswith("artifact/receipt.json"):
            raw = self.receipt
        else:
            raise AssertionError("Recovery issued an unexpected HTTP request")
        return 200, raw

    def git(self, root, *args, **kwargs):
        f = self.fixture
        self.assertEqual(f.target, root, "Recovery must not reopen canonical or external sources")
        base = {name: ("100644", "e" * 40) for name in f.before}
        current = dict(base)
        for name in f.tracked:
            if name.casefold() not in {value.casefold() for value in base}:
                current[name] = ("100644", hashlib.sha1(b"blob 0\0").hexdigest())
        if args == ("ls-files", "--cached", "--others", "--exclude-standard", "-z"):
            raw = b"".join(name.encode() + b"\0" for name in sorted(f.source(f.target)["files"]))
        elif args in (("ls-files", "--stage", "-z"), ("ls-files", "--stage", "--debug", "-z")):
            raw = b"".join((mode + " " + obj + " 0\t" + name).encode() + b"\0"
                           for name, (mode, obj) in sorted(current.items()))
        elif args == ("ls-tree", "-r", "-z", "HEAD"):
            raw = b"".join((mode + " blob " + obj + "\t" + name).encode() + b"\0"
                           for name, (mode, obj) in sorted(base.items()))
        else:
            raise AssertionError("Recovery must not issue source mutations or a binary diff")
        return SimpleNamespace(stdout=raw)

    def unchanged_originals(self):
        f = self.fixture
        self.assertEqual(self.old_operation, f.state.get("management_operation", self.old_key))
        self.assertEqual(self.old_preparation, f.state.get("managed_worktree_preparation", self.old_key))
        self.assertEqual(self.before_bytes, {path.relative_to(f.target).as_posix(): path.read_bytes()
                                            for path in f.target.rglob("*") if path.is_file()})

    def test_real_materialized_failure_recovers_new_ref_without_any_source_rewrite(self):
        f = self.fixture
        with patch.object(helper, "_managed_prewrite_check",
                          side_effect=AssertionError("Recovery must not reenter materialization")):
            result = self.actor.run()
        self.assertEqual("complete", result["status"], result)
        self.assertEqual(self.actor.key, result["worktreePreparationRef"])
        self.assertNotEqual(self.old_key, result["worktreePreparationRef"])
        self.assertFalse(result["sourceFilesRewritten"])
        self.unchanged_originals()
        prepared = formal.preparation(f.state, self.actor.key, "w-t")["payload"]
        self.assertEqual(self.inputs["sealedSource"]["externalCapture"], prepared["externalRoots"])
        self.assertEqual(self.old_key, prepared["resumedFrom"]["operationKey"])
        writer = f.state.get("managed_worktree_writer", "w-t")["payload"]
        self.assertEqual("released", writer["status"])
        self.assertEqual(self.old_writer["payload"], writer["resumedFrom"]["writerPayload"])

    def test_partial_or_changed_postimage_refuses_recovery_and_keeps_interrupted_reservation(self):
        f = self.fixture
        (f.target / "owned.txt").write_bytes(b"incomplete materialization\n")
        changed = (f.target / "owned.txt").read_bytes()
        result = self.actor.run()
        self.assertEqual("unknown", result["status"])
        self.assertIn("incomplete or changed postimage", result["message"])
        self.assertEqual(changed, (f.target / "owned.txt").read_bytes())
        self.assertEqual(self.old_writer, f.state.get("managed_worktree_writer", "w-t"))
        self.assertIsNone(f.state.get("managed_worktree_preparation", self.actor.key))
        self.assertEqual(self.old_preparation, f.state.get("managed_worktree_preparation", self.old_key))

    def test_live_original_birth_is_rejected_even_with_claimed_pipeline_eof(self):
        kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        kernel.GetCurrentProcess.restype = wintypes.HANDLE
        kernel.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
        times = [wintypes.FILETIME() for _ in range(4)]
        self.assertTrue(kernel.GetProcessTimes(kernel.GetCurrentProcess(), *[ctypes.byref(value) for value in times]))
        identity = {"pid": os.getpid(), "creationTime": str((times[0].dwHighDateTime << 32) | times[0].dwLowDateTime),
                    "executable": sys.executable}
        with self.assertRaisesRegex(JenkinsError, "still active"):
            self.real_absent(identity)
        with patch.object(helper, "_native_original_absent", side_effect=JenkinsError(
                "managed_recovery_original_active", "Original process still active")):
            result = self.actor.run()
        self.assertEqual("unknown", result["status"])
        self.assertIn("still active", result["message"])
        self.assertEqual(self.old_writer, self.fixture.state.get("managed_worktree_writer", "w-t"))
        self.assertIsNone(self.fixture.state.get("managed_worktree_preparation", self.actor.key))

    def test_foreign_interrupted_reservation_cannot_be_adopted(self):
        f = self.fixture
        row = f.state.get("managed_worktree_writer", "w-t")
        f.state.put("managed_worktree_writer", "w-t", {**row["payload"], "operationKey": "f" * 64},
                    expected_version=row["version"])
        foreign = f.state.get("managed_worktree_writer", "w-t")
        result = self.actor.run()
        self.assertEqual("unknown", result["status"])
        self.assertIn("original interrupted reservation", result["message"])
        self.assertEqual(foreign, f.state.get("managed_worktree_writer", "w-t"))
        self.unchanged_originals()


if __name__ == "__main__":
    unittest.main()

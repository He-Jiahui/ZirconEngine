"""Synthetic support controls; no Jenkins, native launch, Cargo, or live DB."""
from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import types
import unittest
from unittest.mock import patch
from uuid import uuid4

REPO = Path(__file__).absolute().parents[3]
from tools.jenkins.contracts import JenkinsError, digest, file_digest
from tools.jenkins.state import State
from tools.jenkins.source import _consumer_key, repository_id
from tools.jenkins.source import closeout as support
from tools.jenkins.validation.receipts import closure_manifest, issue_acceptance_receipt
from tools.jenkins.workflow.orchestrator import TEMPLATES
from tools.jenkins.workflow import handler


def patched_handler():
    return handler


class SourceClaimCloseoutTests(unittest.TestCase):
    def setUp(self):
        area = REPO / ".jenkins/tmp"
        tmp = tempfile.TemporaryDirectory(prefix="source-claim-closeout-control-", dir=area)
        self.root = Path(tmp.name)
        assert self.root.resolve().parent == area.resolve()
        self.addCleanup(tmp.cleanup)
        self.state = State(self.root / "support.sqlite3")
        self.build = Path(r"E:\cargo-targets")
        self.objects = self.build / "zircon-local/zircon-jenkins"
        assert self.objects.resolve().is_relative_to(self.build.resolve())
        (self.objects / "inputs/objects").mkdir(parents=True, exist_ok=True)
        artifact = tempfile.TemporaryDirectory(prefix="source-closeout-artifacts-", dir=self.objects)
        self.artifacts = Path(artifact.name)
        assert self.artifacts.resolve().parent == self.objects.resolve()
        self.addCleanup(artifact.cleanup)
        self.repo = repository_id(self.root)
        self.old_driver, self.new_driver = "a" * 64, "b" * 64
        self.coverage = "c" * 64
        entries = []
        for relative in ("src/lib.rs", "cargo.toml"):
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(("synthetic-control-" + uuid4().hex + "\n").encode())
            checksum = file_digest(path)
            obj = self.objects / "inputs/objects" / checksum
            # Synthetic unique data only; never replace any existing CAS object.
            with obj.open("xb") as stream:
                stream.write(path.read_bytes())
            self.addCleanup(self._remove_owned_object, obj, checksum)
            entries.append(dict(path=relative, status="present", sha256=checksum, objectDigest=checksum, size=path.stat().st_size))
        self.manifest = dict(schemaVersion=1, repositoryId=self.repo, baseHead=None, entries=entries, external=[])
        self.source = digest(self.manifest)
        self.identity = dict(repositoryId=self.repo, sessionId="owner-session", requestId="original-request", attemptId="original-attempt",
                             generation=1, sourceInputDigest=self.source, coverageDigest=self.coverage, stageImplementationDigest=self.old_driver)
        self.key = "owner-session:original-request:original-attempt"
        self.eid = "control-execution"
        self.claim = dict(repositoryId=self.repo, repositoryRoot=str(self.root), path="src/lib.rs", owner="original-attempt",
                          epoch=1, status="applied", beforeHash=entries[0]["sha256"], afterHash=entries[0]["sha256"], originalBeforeHash="d" * 64)
        self.claim_key = self.repo + ":src/lib.rs"
        self.state.put("path_claims", self.claim_key, self.claim)
        self.authorize()
        self.configure("module_unit")

    def _remove_owned_object(self, path, checksum):
        assert path.absolute().parent == self.objects / "inputs/objects"
        if path.exists():
            assert file_digest(path) == checksum
            path.unlink()

    def authorize(self, actions=("implementation", "validation")):
        self.grant = self.state.authorize_session(self.repo, "owner-session", "legitimate-scoped-user-principal", ["src", "cargo.toml"], list(actions),
                                                  {"source": "user", "task": "isolated accepted-source release controls"})

    def configure(self, template):
        heavy = template != "comments_only"
        required = ["patch", "syntax", "format"] + (["build", "unit_test"] if heavy else [])
        if template == "cross_module":
            required.append("integration_test")
        if template == "failure_repair":
            required.append("regression_test")
        driver = dict(digest=self.old_driver, generation="1", runtimeOperationId="original-runtime")
        commands = {phase: [r"E:\trusted\cargo.exe", "test", "--" + phase] for phase in ("compile", "list", "test")}
        recipe = dict(kind="cargo", driverDigest=self.old_driver, sourceDigest=self.source, coverageDigest=self.coverage,
                      buildRoot=str(self.build), commands=commands, requiredTests=["owned_test"], requiredPhases=["compile", "list", "test"],
                      requiredTestStages=[s for s in required if s.endswith("test")])
        recipe_ref = digest(recipe)
        phases, jobs = [], {}
        for index, phase in enumerate(("compile", "list", "test")):
            job_id = "native-" + phase
            identity = dict(pid=3100 + index, creationTime=str(134360700000000000 + index), executable=r"E:\trusted\cargo.exe",
                            commandDigest=hashlib.sha256("\0".join(commands[phase]).encode()).hexdigest())
            proof = dict(complete=True, childrenGone=True, stdoutEof=True, stderrEof=True, requested=True,
                         activeProcesses=0, processExitCode=0, identity=identity)
            job = dict(nativeJobId=job_id, owner=self.eid, executionId=self.eid, role="phase", guardNativeJobId="native-guard",
                       status="terminal", pid=identity["pid"], birthToken=identity["creationTime"], executable=identity["executable"], completeProof=proof, terminalProof=proof)
            self.state.put("native_job", job_id, job)
            jobs[phase] = job
            phases.append(dict(phase=phase, nativeJobId=job_id, terminalProof=proof, commandDigest=digest(commands[phase])))
            if phase == "list":
                phases[-1]["discovery"] = dict(source="cargo-test-list", testCount=1, tests=["tests::owned_test"])
        guard_identity = dict(pid=3090, creationTime="134360699000000000", executable=r"E:\trusted\python.exe", commandDigest=digest(["guard"]))
        guard_proof = dict(complete=True, childrenGone=True, stdoutEof=True, stderrEof=True, requested=True, activeProcesses=0, processExitCode=0, identity=guard_identity)
        self.state.put("native_job", "native-guard", dict(nativeJobId="native-guard", owner=self.eid, executionId=self.eid, role="execution_host_guard", status="terminal",
                       pid=3090, birthToken=guard_identity["creationTime"], executable=guard_identity["executable"], completeProof=guard_proof, terminalProof=guard_proof))
        self.state.put("recipe_plan", recipe_ref, dict(recipe=recipe, recipeDigest=recipe_ref, sourceRef=self.source, coverageRef=self.coverage, driver=driver))
        self.state.put("execution", self.eid, dict(status="passed", sourceDigest=self.source, coverageDigest=self.coverage, driverDigest=self.old_driver,
                       recipeRef=recipe_ref, recipeDigest=recipe_ref, driver=driver, operationId="original-runtime", phaseRecords=phases,
                       buildRoot=str(self.build), phaseOutputRoot=str(self.artifacts), terminalProof=phases[-1]["terminalProof"], consumers=[self.key], refs=1))
        self.state.put("execution_host", self.eid, dict(status="terminal", executionId=self.eid, operationId="original-runtime", phaseRecords=phases,
                       terminalProof=phases[-1]["terminalProof"], hostPid=3090, hostBirthToken=guard_identity["creationTime"]))
        self.state.put("execution_launch", self.eid, dict(status="exited", executionId=self.eid, sourceDigest=self.source, recipeRef=recipe_ref,
                       driverDigest=self.old_driver, operationId="original-runtime", exitCode=0, guardNativeJobId="native-guard",
                       launcherIdentity=guard_identity, launcherPid=3090, launcherBirthToken=guard_identity["creationTime"], launcherExecutable=guard_identity["executable"]))
        stage_refs, stage_recipes, last = {}, {}, None
        for stage in required:
            phase = "compile" if stage == "build" else "test"
            native = stage in {"build", "unit_test", "integration_test", "regression_test"}
            ref = self.eid + ":compile" if stage == "build" else self.eid + ":test:" + stage if native else self.eid + ":" + stage
            stage_recipe = recipe_ref if native else digest(["original-light-recipe", stage])
            output = self.artifacts / phase if native else self.artifacts / stage
            output.mkdir(exist_ok=True)
            (output / "stdout.log").write_bytes(((phase if native else stage) + " passed\n").encode())
            receipt = dict(schemaVersion=1, kind="jenkins.managed.producer-receipt", status="passed", executionId=ref, stage=stage,
                           sourceDigest=self.source, coverageDigest=self.coverage, driverDigest=self.old_driver, recipeDigest=stage_recipe,
                           producer=dict(managed=True, name="zircon-jenkins"), outputClosure=closure_manifest(output))
            attempt = dict(executionId=ref, stage=stage, status="passed", sourceDigest=self.source, coverageDigest=self.coverage,
                           driverDigest=self.old_driver, recipeDigest=stage_recipe)
            if native:
                job = jobs[phase]
                receipt["terminalProof"] = dict(job["completeProof"], nativeJobId=job["nativeJobId"], birthToken=job["birthToken"], pid=job["pid"], state="exited", exitCode=0, eofObserved=True, nativeJobActive=False)
                attempt.update(phase=phase, nativeJobId=job["nativeJobId"], birthToken=job["birthToken"], outputRoot=str(output))
            else:
                receipt["lightEvidence"] = dict(status="passed", exitCode=0, tool="source-parser")
            attempt["receipt"] = receipt
            self.state.put("validation_attempt", ref, attempt)
            self.state.put("validation_receipt", ref, receipt)
            stage_refs[stage], stage_recipes[stage], last = ref, stage_recipe, receipt
        original = dict(sourceDigest=self.source, coverageDigest=self.coverage, driverDigest=self.old_driver, generation=1, beforePatch=None, dagDigest=digest(list(TEMPLATES[template])))
        acceptance_id = digest(dict(flow=self.key, identity=original, stages=required))
        acceptance = issue_acceptance_receipt(last, acceptance_id=acceptance_id)
        acceptance.update(stages=required, flowIdentity=original)
        acceptance["receiptDigest"] = digest({k: v for k, v in acceptance.items() if k != "receiptDigest"})
        self.state.put("acceptance", acceptance_id, acceptance)
        op_id = "source-owned-patch"
        op = self.state.get_operation(op_id)
        if op is None:
            self.state.put_operation("source_patch", op_id, dict(owner="original-attempt", repositoryRoot=str(self.root), paths=["src/lib.rs"],
                                     beforeHashes={"src/lib.rs": self.claim["originalBeforeHash"]}))
            self.state.transition_operation(op_id, "prepared", "complete", dict(afterHashes={"src/lib.rs": self.claim["afterHash"]}))
        sealed = dict(status="sealed", sourceDigest=self.source, coverageDigest=self.coverage, owner="original-attempt", sessionId="owner-session", repositoryId=self.repo,
                      repositoryRoot=str(self.root), buildRoot=str(self.build), objectRoot=str(self.objects), sourceManifest=self.manifest, ownedPaths=["src/lib.rs"],
                      coverage=dict(requiredTests=["owned_test"]),
                      patchOperationRefs=[op_id], claims=[dict(path="src/lib.rs", owner="original-attempt", epoch=1, version=1, before_hash=self.claim["beforeHash"], repository_id=self.repo)])
        self.state.put("sealed_input", self.source, sealed)
        self.consumer_key = _consumer_key(self.repo, "owner-session", "original-attempt", self.source)
        self.state.put("sealed_consumer", self.consumer_key, dict(status="active", manifestDigest=self.source, sourceDigest=self.source, repositoryId=self.repo, sessionId="owner-session", owner="original-attempt"))
        self.state.put("workflow", self.key, dict(identity=self.identity, repositoryId=self.repo, repoRoot=str(self.root), status="accepted", template=template,
                       sourceDigest=self.source, sourceManifest=self.manifest, coverageDigest=self.coverage, driverDigest=self.old_driver, generation=1, dagDigest=original["dagDigest"],
                       buildRoot=str(self.build), coverage=dict(requiredTests=["owned_test"]),
                       ownedPaths=["src/lib.rs"], requiredStages=required, stageReceipts=stage_refs, stageRecipeDigests=stage_recipes, acceptance=acceptance,
                       executionId=self.eid if heavy else None, recipeRef=recipe_ref if heavy else None, patchOperationRefs=[op_id]))
        if self.state.get_request(self.repo, "owner-session", "original-request") is None:
            self.state.submit_request(dict(**self.identity, authorizationSnapshot=self.grant["snapshotDigest"]))
            self.state.transition_request(self.repo, "owner-session", "original-request", 1, "pending", "running")
        self.payload = dict(identity=self.identity, releaseRequestId="independent-release-request", sourceDigest=self.source,
                            acceptanceReceiptDigest=acceptance["receiptDigest"], buildRoot=str(self.build),
                            expectedClaims=[dict(key=self.claim_key, version=1, payloadDigest=digest(self.claim))])

    def release(self):
        return support.release_accepted_claims(self.state, self.payload, identity=self.identity, issuer_driver_digest=self.new_driver)

    def update(self, domain, key, **changes):
        row = self.state.get(domain, key)
        self.state.put(domain, key, dict(row["payload"], **changes), expected_version=row["version"])

    def rejected(self):
        before = self.state.get("path_claims", self.claim_key)
        with self.assertRaises(JenkinsError):
            self.release()
        self.assertEqual(before, self.state.get("path_claims", self.claim_key))
        self.assertEqual([], self.state.list("source_claim_closeout"))

    def test_public_branch_closes_without_git_or_consumer_cancellation(self):
        result = patched_handler().handle("source-release", self.payload, self.state, self.root)
        self.assertEqual("complete", result["status"])
        self.assertIsNone(self.state.get("path_claims", self.claim_key))

    def selected_driver_context(self):
        # Brand-new synthetic inventory only. Production verify_driver and
        # active_driver_binding code executes in memory; no source is copied,
        # no global production function or lifecycle provider is replaced.
        contents = {"launcher.py": b"# synthetic control launcher; never executed\n",
                    "tools/jenkins/deployment/driver.py": b"# synthetic inventory entry\n",
                    "pipeline/steps.groovy": b"// synthetic control steps; never executed\n"}
        files = {p: dict(bytes=len(v), sha256=hashlib.sha256(v).hexdigest()) for p, v in contents.items()}
        current = digest(dict(schemaVersion=1, files=files))
        root = self.root / ".jenkins/runtime/drivers" / current
        for relative, data in contents.items():
            path = root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        (root / "driver-manifest.json").write_text(json.dumps(dict(schemaVersion=1, driverDigest=current, files=files)), encoding="utf-8")
        marker = self.root / ".jenkins/state/deployment/driver.json"
        marker.parent.mkdir(parents=True, exist_ok=True)
        marker.write_text(json.dumps(dict(driverDigest=current, root=str(root), runtimeOperationId="new-runtime")), encoding="utf-8")
        provider = types.ModuleType("tools.jenkins.deployment.driver")
        provider.__package__ = "tools.jenkins.deployment"
        provider.__file__ = str(root / "tools/jenkins/deployment/driver.py")
        code = (REPO / "tools/jenkins/deployment/driver.py").read_bytes()
        exec(compile(code, provider.__file__, "exec"), provider.__dict__)
        return provider, current, dict(ZIRCON_DRIVER_DIGEST=current, ZIRCON_DRIVER_LAUNCHER=str(root / "launcher.py"),
                                      JENKINS_PYTHON=str(self.root / ".jenkins/runtime/python/python.exe"))

    def test_public_dispatch_preserves_old_identity_and_uses_verified_new_issuer(self):
        from tools.jenkins.cli import dispatch
        provider, current, env = self.selected_driver_context()
        handler = patched_handler()
        with patch.dict(sys.modules, {"tools.jenkins.deployment.driver": provider, "tools.jenkins.workflow.handler": handler}), patch.dict(os.environ, env):
            self.assertEqual(current, provider.active_driver_binding(self.root)["digest"])
            result = dispatch("flow", "source-release", self.payload, self.state, self.root)
        self.assertNotEqual(self.old_driver, current)
        self.assertEqual(self.old_driver, result["receipt"]["acceptedDriverDigest"])
        self.assertEqual(current, result["receipt"]["closeoutIssuerDriverDigest"])
        self.assertEqual(self.identity, self.state.get("workflow", self.key)["payload"]["identity"])

    def test_public_dispatch_changed_selected_inventory_fails_closed(self):
        from tools.jenkins.cli import dispatch
        provider, current, env = self.selected_driver_context()
        (self.root / ".jenkins/runtime/drivers" / current / "launcher.py").write_bytes(b"changed selected launcher\n")
        with patch.dict(sys.modules, {"tools.jenkins.deployment.driver": provider, "tools.jenkins.workflow.handler": patched_handler()}), patch.dict(os.environ, env):
            with self.assertRaises(JenkinsError):
                dispatch("flow", "source-release", self.payload, self.state, self.root)
        self.assertIsNotNone(self.state.get("path_claims", self.claim_key))
        self.assertEqual([], self.state.list("source_claim_closeout"))

    def test_historical_driver_and_all_references_are_preserved(self):
        domains = ("workflow", "sealed_input", "sealed_consumer", "execution", "acceptance", "validation_receipt", "native_job", "recipe_plan")
        before = {d: self.state.list(d) for d in domains}
        request = self.state.get_request(self.repo, "owner-session", "original-request")
        hashes = {e["path"]: file_digest(self.root / e["path"]) for e in self.manifest["entries"]}
        result = self.release()["receipt"]
        self.assertEqual(self.old_driver, result["acceptedDriverDigest"])
        self.assertEqual(self.new_driver, result["closeoutIssuerDriverDigest"])
        self.assertEqual(before, {d: self.state.list(d) for d in domains})
        self.assertEqual(request, self.state.get_request(self.repo, "owner-session", "original-request"))
        self.assertEqual(hashes, {e["path"]: file_digest(self.root / e["path"]) for e in self.manifest["entries"]})
        self.assertTrue(result["sourceClaimsReleased"])
        self.assertFalse(result["sourceBytesWritten"])

    def test_comments_only_needs_no_compiler_records(self):
        self.configure("comments_only")
        for domain in ("execution", "execution_host", "execution_launch", "native_job", "recipe_plan"):
            for row in self.state.list(domain):
                self.state.delete(domain, row["key"])
        self.assertEqual("complete", self.release()["status"])

    def test_cross_module_uses_integration_receipt(self):
        self.configure("cross_module")
        self.assertEqual("complete", self.release()["status"])

    def test_failure_repair_uses_regression_receipt(self):
        self.configure("failure_repair")
        self.assertEqual("complete", self.release()["status"])

    def test_completed_closeout_replay_does_not_delete_later_foreign_claim(self):
        first = self.release()
        foreign = dict(self.claim, owner="later-owner", epoch=2)
        self.state.put("path_claims", self.claim_key, foreign)
        replay = self.release()
        self.assertTrue(replay["reused"])
        self.assertEqual(first["receipt"], replay["receipt"])
        self.assertEqual(foreign, self.state.get("path_claims", self.claim_key)["payload"])

    def test_duplicate_closeout_id_with_new_claim_pin_is_rejected(self):
        self.release()
        self.payload["expectedClaims"][0]["version"] = 2
        with self.assertRaises(JenkinsError):
            self.release()

    def test_pending_failed_and_cancelled_flow_cannot_release(self):
        for status in ("pending", "failed", "cancelled"):
            with self.subTest(status=status):
                self.update("workflow", self.key, status=status)
                self.rejected()

    def test_foreign_physical_edit_is_preserved(self):
        path = self.root / "src/lib.rs"
        path.write_bytes(b"foreign later bytes\n")
        self.rejected()
        self.assertEqual(b"foreign later bytes\n", path.read_bytes())

    def test_dependency_change_blocks_even_when_claimed_source_unchanged(self):
        (self.root / "cargo.toml").write_bytes(b"foreign dependency\n")
        self.rejected()

    def test_foreign_owner_claim_is_preserved(self):
        self.update("path_claims", self.claim_key, owner="foreign-owner")
        self.rejected()

    def test_claim_epoch_and_version_cas_rejects(self):
        self.update("path_claims", self.claim_key, epoch=2)
        self.rejected()

    def test_changed_claim_payload_digest_rejects(self):
        self.payload["expectedClaims"][0]["payloadDigest"] = "f" * 64
        self.rejected()

    def test_cancelled_sealed_consumer_cannot_substitute_for_claim_closeout(self):
        self.update("sealed_consumer", self.consumer_key, status="released")
        self.rejected()

    def test_revoked_original_authorization_rejects(self):
        self.state.revoke_session(self.repo, "owner-session")
        self.rejected()

    def test_original_grant_snapshot_changed_rejects(self):
        with self.state.transaction() as c:
            c.execute("UPDATE authorizations SET snapshot_digest=?", ("f" * 64,))
        self.rejected()

    def test_caller_cannot_replace_old_driver_identity(self):
        self.identity["stageImplementationDigest"] = self.new_driver
        self.rejected()

    def test_caller_cannot_replace_acceptance_digest(self):
        self.payload["acceptanceReceiptDigest"] = "f" * 64
        self.rejected()

    def test_missing_original_stage_authority_rejects(self):
        self.state.delete("validation_attempt", self.eid + ":test:unit_test")
        self.rejected()

    def test_fabricated_acceptance_cannot_replace_state_authority(self):
        flow = self.state.get("workflow", self.key)["payload"]
        fake = dict(flow["acceptance"], authority="caller")
        self.update("workflow", self.key, acceptance=fake)
        self.rejected()

    def test_missing_list_phase_native_rejects(self):
        self.state.delete("native_job", "native-list")
        self.rejected()

    def test_phase_command_digest_must_match_immutable_recipe(self):
        value = self.state.get("execution", self.eid)["payload"]
        phases = copy.deepcopy(value["phaseRecords"])
        phases[1]["commandDigest"] = "f" * 64
        self.update("execution", self.eid, phaseRecords=phases)
        self.update("execution_host", self.eid, phaseRecords=phases)
        self.rejected()

    def test_native_argv_digest_must_match_immutable_recipe(self):
        value = self.state.get("native_job", "native-list")["payload"]
        proof = copy.deepcopy(value["completeProof"])
        proof["identity"]["commandDigest"] = "f" * 64
        self.update("native_job", "native-list", completeProof=proof, terminalProof=proof)
        phases = copy.deepcopy(self.state.get("execution", self.eid)["payload"]["phaseRecords"])
        phases[1]["terminalProof"] = proof
        self.update("execution", self.eid, phaseRecords=phases)
        self.update("execution_host", self.eid, phaseRecords=phases)
        self.rejected()

    def test_empty_native_discovery_cannot_release(self):
        phases = copy.deepcopy(self.state.get("execution", self.eid)["payload"]["phaseRecords"])
        phases[1]["discovery"] = dict(source="cargo-test-list", testCount=0, tests=[])
        self.update("execution", self.eid, phaseRecords=phases)
        self.update("execution_host", self.eid, phaseRecords=phases)
        self.rejected()

    def test_missing_required_test_in_real_discovery_rejects(self):
        phases = copy.deepcopy(self.state.get("execution", self.eid)["payload"]["phaseRecords"])
        phases[1]["discovery"]["tests"] = ["unrelated_test"]
        self.update("execution", self.eid, phaseRecords=phases)
        self.update("execution_host", self.eid, phaseRecords=phases)
        self.rejected()

    def test_host_launcher_identity_is_not_substitutable(self):
        self.update("execution_launch", self.eid, launcherIdentity=dict(pid=3090))
        self.rejected()

    def test_boolean_or_float_native_counts_are_not_integer_zero(self):
        row = self.state.get("native_job", "native-list")
        for value in (False, 0.0, None):
            with self.subTest(value=value):
                proof = dict(row["payload"]["completeProof"], activeProcesses=value)
                self.update("native_job", "native-list", completeProof=proof, terminalProof=proof)
                self.rejected()

    def test_missing_birth_pid_executable_identity_rejects(self):
        for field, value in (("pid", False), ("birthToken", ""), ("executable", "")):
            with self.subTest(field=field):
                self.update("native_job", "native-list", **{field: value})
                self.rejected()

    def test_unfinished_host_guard_rejects(self):
        self.update("native_job", "native-guard", status="running")
        self.rejected()

    def test_extra_unknown_native_job_rejects(self):
        self.state.put("native_job", "unclassified-native", dict(owner=self.eid, executionId=self.eid, status="running"))
        self.rejected()

    def test_pending_owned_source_operation_rejects(self):
        self.state.put_operation("source_patch", "new-pending-source", dict(owner="original-attempt"))
        self.rejected()

    def test_atomic_failure_after_first_delete_rolls_back_claims_and_receipt(self):
        original = self.state.put
        def fail_closeout(domain, *args, **kwargs):
            if domain == "source_claim_closeout":
                raise RuntimeError("simulated durable closeout write failure")
            return original(domain, *args, **kwargs)
        self.state.put = fail_closeout
        with self.assertRaises(RuntimeError):
            self.release()
        self.assertEqual(self.claim, self.state.get("path_claims", self.claim_key)["payload"])
        self.assertEqual([], self.state.list("source_claim_closeout"))


if __name__ == "__main__":
    unittest.main()

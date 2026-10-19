"""Regressions for the observed Flow 53/54 and Execution 37/38 failures."""
from copy import deepcopy
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, digest, file_digest
from tools.jenkins.source import external_inputs as external
from tools.jenkins.state import State
from tools.jenkins.tests import test_execution_engine as engine_fixture
from tools.jenkins.tests import test_post_task_cache as cache_fixture
from tools.jenkins.workflow import post_task_cache as cache


class FlowExecutionRepairTests(unittest.TestCase):
    def cache_fixture(self, transform=None):
        fixture = cache_fixture.PostTaskCacheTests("test_light_task_assessment_has_no_native_job_or_gc")
        submit = State.submit_request
        with patch.object(State, "submit_request", lambda state, value: submit(state, transform(value) if transform else value)):
            fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        return fixture

    def legacy_cache_fixture(self):
        original = {}
        def transform(value):
            intent = {"identity": {key: value[key] for key in
                ("repositoryId", "sessionId", "requestId", "attemptId", "generation", "stageImplementationDigest")},
                "buildRoot": value["buildRoot"], "runtimeOperationId": value["runtimeOperationId"],
                "authorizationSnapshot": value.get("authorizationSnapshot")}
            reference = digest(intent)
            original.update(reference=reference, intent=intent)
            value = {**value, "currentSourceRef": reference}
            value.pop("runtimeOperationId")
            return value
        fixture = self.cache_fixture(transform)
        fixture.state.put("current_source_intent", original["reference"], original["intent"])
        return fixture, original

    def test_v1_request_uses_its_hashed_original_runtime(self):
        fixture, original = self.legacy_cache_fixture()
        authority, _, _, execution = cache._authority(fixture.payload, fixture.state)
        self.assertEqual(fixture.eid, authority["executionId"])
        self.assertEqual("runtime", execution["payload"]["operationId"])
        self.assertEqual(original["reference"], digest(original["intent"]))

    def test_changed_or_foreign_v1_intent_cannot_authorize_maintenance(self):
        fixture, original = self.legacy_cache_fixture()
        for update in ({"runtimeOperationId": "different-runtime"},
                       {"identity": {**original["intent"]["identity"], "sessionId": "foreign"}}):
            with self.subTest(update=update):
                fixture.state.put("current_source_intent", original["reference"], {**original["intent"], **update})
                with self.assertRaises(JenkinsError):
                    cache._authority(fixture.payload, fixture.state)

    def test_missing_runtime_cannot_fall_back_to_current_deployment(self):
        fixture = self.cache_fixture(lambda value: {key: v for key, v in value.items() if key != "runtimeOperationId"})
        with self.assertRaises(JenkinsError):
            cache._authority(fixture.payload, fixture.state)

    def aborted_engine(self):
        fixture = engine_fixture.ExecutionEngineTests("test_one_cancelled_consumer_does_not_discard_an_active_consumer_validation")
        fixture.setUp()
        self.addCleanup(fixture.tearDown)
        eid = fixture.claim()["executionId"]
        row = fixture.state.get("execution", eid)
        fixture.state.put("execution", eid, {**row["payload"], "status": "running"}, expected_version=row["version"])
        proof = {"complete": True, "identity": {"pid": 123, "creationTime": "1234567", "executable": "python.exe"},
                 "processExitCode": 1, "activeProcesses": 0, "stdoutEof": True, "stderrEof": True, "childrenGone": True}
        fixture.state.put("native_job", "guard", {"role": "execution_host_guard", "owner": eid,
            "executionId": eid, "status": "terminal", "completeProof": proof, "terminalProof": proof})
        fixture.state.put("execution_launch", eid, {"guardNativeJobId": "guard"})
        fixture.state.put("execution_abort", eid, {"guardNativeJobId": "guard", "proofDigest": digest(proof)})
        return fixture, eid, proof

    def test_prephase_failure_retains_guard_proof_and_failed_status(self):
        fixture, eid, proof = self.aborted_engine()
        with patch("tools.jenkins.artifacts.publish_execution_outputs") as publish:
            result = fixture.engine.observe(eid)
        self.assertEqual("failed", result["status"])
        self.assertEqual(proof, result["terminalProof"])
        self.assertFalse(result["abortProof"]["artifactPublicationAllowed"])
        publish.assert_not_called()

    def test_original_failed_execution_can_reconcile_missing_guard_proof(self):
        fixture, eid, proof = self.aborted_engine()
        row = fixture.state.get("execution", eid)
        from tools.jenkins.processes.guards import abort_evidence
        fixture.state.put("execution", eid, {**row["payload"], "status": "failed",
            "abortProof": abort_evidence(fixture.state, eid)}, expected_version=row["version"])
        result = fixture.engine.observe(eid)
        self.assertEqual("failed", result["status"])
        self.assertEqual(proof, result["terminalProof"])

    def test_unproven_guard_cannot_supply_terminal_proof(self):
        fixture, eid, _ = self.aborted_engine()
        row = fixture.state.get("native_job", "guard")
        fixture.state.put("native_job", "guard", {**row["payload"], "status": "running"}, expected_version=row["version"])
        with self.assertRaises(JenkinsError):
            fixture.engine.observe(eid)
        self.assertNotIn("terminalProof", fixture.state.get("execution", eid)["payload"])

    def test_boot_boundary_failure_does_not_invent_guard_or_pipe_eof_proof(self):
        fixture, eid, _ = self.aborted_engine()
        evidence = {"kind": "windows-boot-boundary", "executionId": eid,
                    "outcome": "failed", "artifactPublicationAllowed": False}
        with patch("tools.jenkins.processes.guards.abort_evidence", return_value=evidence):
            result = fixture.engine.observe(eid)
            again = fixture.engine.observe(eid)
        self.assertEqual("failed", result["status"])
        self.assertEqual(evidence, again["abortProof"])
        self.assertNotIn("terminalProof", result)
        self.assertNotIn("terminalProof", again)

    def external_fixture(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        source = Path(temp.name) / "source"
        source.mkdir()
        root = source.parent / "zr_vm"
        target = root / "tests/compileTime/test.c"
        target.parent.mkdir(parents=True)
        target.write_bytes(b"int compile_time_test;\n")
        entry = {"path": "tests/compiletime/test.c", "status": "present", "sha256": file_digest(target)}
        sibling = {"entries": [entry]}
        manifest = {"preparationKind": "current-source", "external": [{"kind": external.KIND, "name": "zr_vm",
            "manifest": sibling, "sourceDigest": digest(sibling)}]}
        return source, target, manifest

    def test_case_preserving_materialization_still_matches_canonical_closure(self):
        source, _, manifest = self.external_fixture()
        with patch("tools.jenkins.resources.physical_path_under", side_effect=lambda _, path, **kw: path):
            external.verify_prepared(manifest, source, object())

    def test_changed_content_or_extra_file_still_rejects_prepared_sibling(self):
        source, target, manifest = self.external_fixture()
        with patch("tools.jenkins.resources.physical_path_under", side_effect=lambda _, path, **kw: path):
            target.write_bytes(b"foreign change")
            with self.assertRaises(JenkinsError):
                external.verify_prepared(manifest, source, object())
            target.write_bytes(b"int compile_time_test;\n")
            (target.parent / "foreign.c").write_bytes(b"unknown")
            with self.assertRaises(JenkinsError):
                external.verify_prepared(manifest, source, object())

    def test_duplicate_case_insensitive_manifest_paths_are_rejected(self):
        source, _, manifest = self.external_fixture()
        sibling = manifest["external"][0]["manifest"]
        sibling["entries"].append({**sibling["entries"][0], "path": "tests/compileTime/test.c"})
        manifest["external"][0]["sourceDigest"] = digest(sibling)
        with patch("tools.jenkins.resources.physical_path_under", side_effect=lambda _, path, **kw: path):
            with self.assertRaises(JenkinsError):
                external.verify_prepared(manifest, source, object())


if __name__ == "__main__":
    unittest.main()

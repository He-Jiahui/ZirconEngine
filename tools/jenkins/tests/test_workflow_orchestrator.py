from __future__ import annotations

import unittest
from pathlib import Path
import tempfile
import json

from tools.jenkins.state import State
from tools.jenkins.workflow.handler import handle

from tools.jenkins.workflow.orchestrator import (
    EXECUTION_JOB, FLOW_JOB, MAINTENANCE_JOB, build_dag, run_control,
)

REPO_ROOT = Path(__file__).resolve().parents[3]
JENKINS_TMP = REPO_ROOT / ".jenkins" / "tmp"
BUILD_ROOT = REPO_ROOT / ".jenkins" / "builds"
JENKINS_TMP.mkdir(parents=True, exist_ok=True)
BUILD_ROOT.mkdir(parents=True, exist_ok=True)


IDENTITY = {
    "sessionId": "session-1", "requestId": "request-1", "attemptId": "attempt-1",
    "generation": 1, "sourceInputDigest": "a" * 64, "coverageDigest": "b" * 64,
    "stageImplementationDigest": "c" * 64,
}


class WorkflowOrchestratorTests(unittest.TestCase):
    def test_real_owned_comment_flow_acceptance(self):
        """Bounded sealed comment flow fixture; no Cargo or product checkout writes."""
        import hashlib, subprocess
        from tools.jenkins.source import handle as source_handle
        with tempfile.TemporaryDirectory(dir=str(JENKINS_TMP)) as temp:
            root = Path(temp); (root / "src").mkdir(); (root / ".jenkins/tmp").mkdir(parents=True)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True)
            original = "fn main() {}\n"
            (root / "src/lib.rs").write_text(original)
            subprocess.run(["git", "add", "."], cwd=root, check=True); subprocess.run(["git", "-c", "user.email=a@b", "-c", "user.name=t", "commit", "-qm", "base"], cwd=root, check=True)
            state = State(root / "state.sqlite3")
            state.authorize_session("repo", "session-1", "tester", ["src/lib.rs"], ["implementation", "validation"], {"source":"user", "task":"m3"})
            before = hashlib.sha256(original.encode()).hexdigest()
            claim = source_handle("claim", {"repositoryId":"repo", "sessionId":"session-1", "owner":"attempt-1", "ownedPaths":["src/lib.rs"]}, state, root)
            changed = "// note\nfn main() {}\n"
            source_handle("apply", {"repositoryId":"repo", "sessionId":"session-1", "owner":"attempt-1", "patch":{"src/lib.rs":changed}, "beforeHashes":{"src/lib.rs":claim["claims"][0]["before_hash"]}, "operationId":"patch-1"}, state, root)
            from tools.jenkins.source import repository_id
            current_claim = state.get("path_claims", repository_id(root) + ":src/lib.rs")
            claim = {"claims": [{"path":"src/lib.rs", "owner":"attempt-1", "epoch":1,
                                  "before_hash":current_claim["payload"]["beforeHash"], "version":current_claim["version"]}]}
            objroot = root / "objects"; objroot.mkdir()
            build_root = BUILD_ROOT
            object_root = build_root / "zircon-jenkins"
            sealed = source_handle("seal", {"repositoryId":"repo", "sessionId":"session-1", "owner":"attempt-1", "buildRoot":str(build_root), "claims":claim["claims"], "objectRoot":str(object_root), "coverage":{"selectedTests":1}, "patchOperationRefs":["patch-1"], "baseHead":subprocess.run(["git","rev-parse","HEAD"],cwd=root,capture_output=True,text=True,check=True).stdout.strip()}, state, root)
            identity = dict(IDENTITY); identity["sourceInputDigest"] = sealed["sourceDigest"]
            identity["coverageDigest"] = state.get("sealed_input", sealed["sourceDigest"])["payload"]["coverageDigest"]
            # source_handle(seal) is the authoritative sealed input record.
            registered = handle("register-flow", {"identity":identity, "sealedInputRef":sealed["sourceDigest"]}, state, root)
            handle("compose-flow", {"identity":identity}, state, root)
            handle("run-stage", {"identity":identity, "stage":"patch", "patchJournal":"patch-1"}, state, root)
            handle("run-stage", {"identity":identity, "stage":"syntax"}, state, root)
            handle("run-stage", {"identity":identity, "stage":"format"}, state, root)
            accepted = handle("accept-flow", {"identity":identity}, state, root)
            self.assertEqual(accepted["status"], "accepted")
            self.assertFalse((root / "target").exists())
    def test_fixed_job_names(self):
        self.assertEqual((FLOW_JOB, EXECUTION_JOB, MAINTENANCE_JOB),
                         ("zircon-flow", "zircon-execution", "zircon-maintenance"))

    def test_dag_requires_full_identity_and_preserves_template_order(self):
        plan = build_dag("comments_only", IDENTITY, execution_id="exec-1")
        self.assertEqual(plan.stages, ("patch", "syntax", "format", "acceptance", "commit"))
        self.assertEqual(plan.execution_id, "exec-1")
        with self.assertRaises(ValueError):
            build_dag("comments_only", {"requestId": "request-1"})

    def test_control_delegates_to_injected_authority(self):
        class Authority:
            def observe(self, payload):
                return {"status": "waiting", "observedGeneration": payload["generation"]}
        result = run_control("observe", {"generation": 3}, cli=Authority())
        self.assertEqual(result["observedGeneration"], 3)

    def test_handler_requires_sealed_inputs_and_composes_dag(self):
        with tempfile.TemporaryDirectory(dir=str(JENKINS_TMP)) as temp:
            state = State(Path(temp) / "state.sqlite3")
            with self.assertRaises(Exception):
                handle("register-flow", {"identity": IDENTITY}, state, Path(temp))
            state = State(Path(temp) / "state.sqlite3")
            state.put("sealed_input", IDENTITY["sourceInputDigest"], {"status": "sealed", "manifest": {}, "coverage": {}})
            payload = {"identity": IDENTITY, "sealedInputRef": IDENTITY["sourceInputDigest"], "buildRoot": str(BUILD_ROOT)}
            registered = handle("register-flow", payload, state, Path(temp))
            self.assertEqual(registered["status"], "pending")
            with self.assertRaises(Exception):
                handle("compose-flow", {"identity": IDENTITY,
                    "changeSet": {"src/lib.rs": "// note\n"}, "requestedTemplate": "auto"}, state, Path(temp))
            observed = handle("observe-flow", {"identity": IDENTITY,
                "executionId": registered["executionId"]}, state, Path(temp))
            self.assertEqual(observed["status"], "pending")

    def test_register_rejects_coverage_bound_to_another_sealed_input(self):
        with tempfile.TemporaryDirectory(dir=str(JENKINS_TMP)) as temp:
            state = State(Path(temp) / "state.sqlite3")
            sealed = {"status": "sealed", "manifest": {}, "coverage": {},
                      "coverageDigest": "d" * 64, "sourceDigest": IDENTITY["sourceInputDigest"]}
            state.put("sealed_input", IDENTITY["sourceInputDigest"], sealed)
            with self.assertRaises(Exception) as error:
                handle("register-flow", {"identity": IDENTITY,
                    "sealedInputRef": IDENTITY["sourceInputDigest"]}, state, Path(temp))
            self.assertIn("coverage", str(error.exception))

    def test_pipeline_requires_sealed_driver_and_per_call_node(self):
        root = Path(__file__).resolve().parents[3]
        flow = (root / ".jenkins/pipeline/zircon-flow.groovy").read_text(encoding="utf8")
        self.assertIn("ZIRCON_SEALED_DRIVER", flow)
        self.assertIn("sleep(time:", flow)
        self.assertGreaterEqual(flow.count("node('zircon-windows')"), 1)
        self.assertIn("def waitControl", flow)

    def test_execution_rejects_unregistered_recipe_and_zero_tests(self):
        with tempfile.TemporaryDirectory(dir=str(JENKINS_TMP)) as temp:
            state = State(Path(temp) / "state.sqlite3")
            with self.assertRaises(Exception):
                handle("claim-execution", {"identity": IDENTITY, "executionId": "exec-1",
                    "recipe": {"command": ["powershell", "Remove-Item -Recurse C:\\"]}}, state, Path(temp), domain="execution")

    def test_run_stage_records_receipt_and_blocks_descendants_on_syntax_failure(self):
        with tempfile.TemporaryDirectory(dir=str(JENKINS_TMP)) as temp:
            state = State(Path(temp) / "state.sqlite3")
            state.put("sealed_input", IDENTITY["sourceInputDigest"], {"status": "sealed", "manifest": {}, "coverage": {}})
            handle("register-flow", {"identity": IDENTITY, "sealedInputRef": IDENTITY["sourceInputDigest"]}, state, Path(temp))
            with self.assertRaises(Exception):
                handle("compose-flow", {"identity": IDENTITY, "changeSet": {"a.rs": "fn main() {}\n"}, "requestedTemplate": "module_unit"}, state, Path(temp))
            with self.assertRaises(Exception):
                handle("run-stage", {"identity": IDENTITY, "stage": "patch"}, state, Path(temp))


if __name__ == "__main__":
    unittest.main()

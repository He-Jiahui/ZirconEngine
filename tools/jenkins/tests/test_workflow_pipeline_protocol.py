from pathlib import Path
import unittest


class PipelineProtocolTests(unittest.TestCase):
    def test_steps_avoid_unsandboxed_system_calls_and_use_launcher(self):
        root = Path(__file__).resolve().parents[3]
        steps = (root / '.jenkins/pipeline/steps.groovy').read_text(encoding='utf8')
        self.assertIn('UUID.randomUUID()', steps)
        self.assertNotIn('System.nanoTime', steps)
        self.assertIn('ZIRCON_DRIVER_LAUNCHER', steps)
        self.assertIn('groovy.json.JsonSlurper()', steps)
        self.assertNotIn('JsonSlurperClassic', steps)

    def test_flow_reconciles_without_masking_original_error(self):
        root = Path(__file__).resolve().parents[3]
        flow = (root / '.jenkins/pipeline/zircon-flow.groovy').read_text(encoding='utf8')
        self.assertIn('def origin = err.toString()', flow)
        self.assertIn('reconcile failed after original error', flow)
        self.assertIn('throw err', flow)

    def test_comments_flow_has_no_external_duplicate_accept_call(self):
        root = Path(__file__).resolve().parents[3]
        flow = (root / '.jenkins/pipeline/zircon-flow.groovy').read_text(encoding='utf8')
        self.assertEqual(flow.count("'accept-flow'"), 0)
        self.assertIn("stage: 'acceptance'", flow)

    def test_heavy_flow_binds_authoritative_execution_and_dispatches_job(self):
        root = Path(__file__).resolve().parents[3]
        flow = (root / '.jenkins/pipeline/zircon-flow.groovy').read_text(encoding='utf8')
        execution = (root / '.jenkins/pipeline/zircon-execution.groovy').read_text(encoding='utf8')
        self.assertIn("submitted.executionId", flow)
        self.assertIn("submitted.recipeRef", flow)
        self.assertIn("recipeRef: recipeRef", flow)
        self.assertIn("'ensure-execution-job'", flow)
        self.assertNotIn("'claim-execution'", execution)
        self.assertIn("'observe-execution'", execution)
        self.assertIn("'reconcile-execution'", execution)
        self.assertIn("repositoryId", execution)

    def test_incremental_patch_stage_seals_identity_before_register(self):
        root = Path(__file__).resolve().parents[3]
        flow = (root / '.jenkins/pipeline/zircon-flow.groovy').read_text(encoding='utf8')
        self.assertIn("stage('Apply incremental patch')", flow)
        self.assertIn("invokeControl('flow', 'prepare-patch'", flow)
        self.assertIn("prepared.status != 'prepared'", flow)
        self.assertIn("patchRequestRef: patchRequestRef", flow)
        self.assertIn("registeredFlow = true", flow)
        self.assertIn("invokeControl('flow', 'reconcile-patch'", flow)

    def test_job_provisioning_requires_an_explicit_compiler_root(self):
        from tools.jenkins.deployment.job_provisioning import _config
        environment = {"ZIRCON_REPO_ROOT": str(Path(__file__).resolve().parents[3]),
                       "JENKINS_PYTHON": "python.exe", "ZIRCON_SEALED_DRIVER": "driver",
                       "ZIRCON_DRIVER_LAUNCHER": "launcher", "ZIRCON_DRIVER_DIGEST": "a" * 64,
                       "ZIRCON_AGENT_LABEL": "agent-runtime", "ZIRCON_RUNTIME_OPERATION_ID": "start-test"}
        with self.assertRaises(Exception):
            _config("echo compiler root fixture", environment)
        xml = _config("echo compiler root fixture", {**environment, "ZIRCON_BUILD_ROOT": r"D:\cargo-targets"})
        self.assertNotIn(".jenkins/builds", xml)
        self.assertIn("cargo-targets", xml)

    def test_job_parameter_declares_patch_request_reference(self):
        from tools.jenkins.deployment.job_provisioning import _config
        xml = _config("echo 'fixture'", {
            "ZIRCON_REPO_ROOT": str(Path(__file__).resolve().parents[3]),
            "JENKINS_PYTHON": "python.exe", "ZIRCON_SEALED_DRIVER": "driver",
            "ZIRCON_DRIVER_LAUNCHER": "launcher", "ZIRCON_DRIVER_DIGEST": "a" * 64,
            "ZIRCON_BUILD_ROOT": r"D:\cargo-targets", "ZIRCON_AGENT_LABEL": "agent-runtime",
            "ZIRCON_RUNTIME_OPERATION_ID": "start-test",
        })
        self.assertIn('<name>PATCH_REQUEST_REF</name>', xml)


if __name__ == '__main__':
    unittest.main()

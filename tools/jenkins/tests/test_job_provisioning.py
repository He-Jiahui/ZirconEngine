from pathlib import Path
import unittest
import xml.etree.ElementTree as ET

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment.job_provisioning import _config
from tools.jenkins.submission import _submission_fields


class JobRuntimeDefaultsTests(unittest.TestCase):
    def setUp(self):
        self.root = Path(__file__).resolve().parents[3]
        self.environment = {
            "ZIRCON_REPO_ROOT": str(self.root),
            "JENKINS_PYTHON": "python.exe",
            "ZIRCON_SEALED_DRIVER": "sealed-driver",
            "ZIRCON_DRIVER_LAUNCHER": "sealed-launcher",
            "ZIRCON_DRIVER_DIGEST": "a" * 64,
            "ZIRCON_BUILD_ROOT": r"D:\cargo-targets",
            "ZIRCON_AGENT_LABEL": "agent-runtime",
            "ZIRCON_RUNTIME_OPERATION_ID": "start-verified-runtime",
        }
        self.source = (self.root / ".jenkins/pipeline/zircon-execution.groovy").read_text(encoding="utf-8")

    def configured_defaults(self):
        config = ET.fromstring(_config(self.source, self.environment))
        definitions = config.findall("./properties/hudson.model.ParametersDefinitionProperty/parameterDefinitions/*")
        defaults = {definition.findtext("name"): definition.findtext("defaultValue") or ""
                    for definition in definitions}
        return config, defaults

    def test_omitted_dispatch_runtime_fields_inherit_selected_deployment(self):
        _, defaults = self.configured_defaults()
        fields = _submission_fields({"repositoryId": "repository", "sessionId": "session",
                                     "requestId": "request", "parameters": {"executionId": "execution"}})
        self.assertNotIn("RUNTIME_OPERATION_ID", fields)
        self.assertNotIn("STAGE_IMPLEMENTATION_DIGEST", fields)
        effective = {**defaults, **fields}
        self.assertEqual(self.environment["ZIRCON_RUNTIME_OPERATION_ID"], effective["RUNTIME_OPERATION_ID"])
        self.assertEqual(self.environment["ZIRCON_DRIVER_DIGEST"], effective["STAGE_IMPLEMENTATION_DIGEST"])
        nonempty = {name: value for name, value in defaults.items() if value}
        self.assertEqual({"RUNTIME_OPERATION_ID": self.environment["ZIRCON_RUNTIME_OPERATION_ID"],
                          "STAGE_IMPLEMENTATION_DIGEST": self.environment["ZIRCON_DRIVER_DIGEST"]}, nonempty)

    def test_explicit_mismatch_is_preserved_for_unchanged_sandboxed_guard(self):
        config, defaults = self.configured_defaults()
        fields = _submission_fields({"repositoryId": "repository", "sessionId": "session",
                                     "requestId": "request", "parameters": {
                                         "runtimeOperationId": "other-runtime",
                                         "stageImplementationDigest": "b" * 64}})
        effective = {**defaults, **fields}
        self.assertEqual("other-runtime", effective["RUNTIME_OPERATION_ID"])
        self.assertEqual("b" * 64, effective["STAGE_IMPLEMENTATION_DIGEST"])
        self.assertNotEqual(self.environment["ZIRCON_RUNTIME_OPERATION_ID"], effective["RUNTIME_OPERATION_ID"])
        self.assertNotEqual(self.environment["ZIRCON_DRIVER_DIGEST"], effective["STAGE_IMPLEMENTATION_DIGEST"])
        script = config.findtext("./definition/script")
        self.assertTrue(script.endswith(self.source))
        self.assertIn("params.RUNTIME_OPERATION_ID != env.ZIRCON_RUNTIME_OPERATION_ID", script)
        self.assertIn("params.STAGE_IMPLEMENTATION_DIGEST != env.ZIRCON_DRIVER_DIGEST", script)
        self.assertEqual("true", config.findtext("./definition/sandbox"))

    def test_defaults_require_a_complete_trusted_runtime_binding(self):
        for name in ("ZIRCON_AGENT_LABEL", "ZIRCON_RUNTIME_OPERATION_ID", "ZIRCON_DRIVER_DIGEST"):
            with self.subTest(name=name):
                environment = {**self.environment, name: ""}
                with self.assertRaises(JenkinsError) as raised:
                    _config(self.source, environment)
                self.assertEqual("runtime_binding_missing", raised.exception.code)


if __name__ == "__main__":
    unittest.main()

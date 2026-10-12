from pathlib import Path
import unittest


class PipelineStepResponseTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        root = Path(__file__).resolve().parents[3]
        cls.steps = (root / ".jenkins/pipeline/steps.groovy").read_text(encoding="utf-8")

    def test_control_result_keeps_bounded_patch_identity_and_pipeline_fields(self):
        for field in (
            "status", "reasonCode", "executionId", "recipeRef", "template", "receiptRef",
            "receipt", "stages", "generation", "observedGeneration", "retryable",
            "identity", "runtimeOperationId", "driverDigest",
            "sealedInputRef", "patchOperationRef", "patchRequestRef",
            "sourceInputDigest", "coverageDigest", "stageImplementationDigest",
        ):
            with self.subTest(field=field):
                self.assertIn("'" + field + "'", self.steps)

    def test_nested_json_is_copied_to_plain_cps_safe_collections(self):
        self.assertIn("@NonCPS\ndef plainJsonValue(Object value)", self.steps)
        self.assertIn("value instanceof Map", self.steps)
        self.assertIn("value instanceof List", self.steps)
        self.assertIn("new LinkedHashMap()", self.steps)
        self.assertIn("new ArrayList(value.size())", self.steps)
        self.assertIn("plainJsonValue(parsed.get(field))", self.steps)
        self.assertIn("JSON object keys must be strings", self.steps)
        self.assertIn("Unsupported JSON value type", self.steps)

    def test_control_injects_authoritative_runtime_binding_without_mutating_identity(self):
        for required in (
            "def runtimeOperationId = env.ZIRCON_RUNTIME_OPERATION_ID",
            "def driverDigest = env.ZIRCON_DRIVER_DIGEST",
            "if (!runtimeOperationId || !driverDigest)",
            "payload.get('runtimeOperationId')",
            "payload.get('driverDigest')",
            "new LinkedHashMap(payload)",
            "boundPayload.put('runtimeOperationId', runtimeOperationId)",
            "boundPayload.put('driverDigest', driverDigest)",
            "JsonOutput.toJson(boundPayload)",
        ):
            with self.subTest(required=required):
                self.assertIn(required, self.steps)
        self.assertIn("identity", self.steps)
        self.assertNotIn("identity.runtimeOperationId =", self.steps)
        self.assertNotIn("identity.driverDigest =", self.steps)

    def test_parser_rejects_dynamic_evaluation_and_unbounded_result_passthrough(self):
        for forbidden in ("Eval.", "GroovyShell", "JsonSlurperClassic"):
            self.assertNotIn(forbidden, self.steps)
        self.assertNotIn("return parsed", self.steps)
        self.assertNotIn("return [status: parsed.status", self.steps)


if __name__ == "__main__":
    unittest.main()

"""Protect diagnostic identity, coverage and evidence boundaries without fixtures."""
import copy
import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location("resolver_entry_subject", ROOT / "tools/jenkins-management/resolver_entry_observation.py")
SUBJECT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SUBJECT)


def plan():
    return {"schemaVersion": 1, "requestId": SUBJECT.RUN, "attemptId": SUBJECT.RUN,
            "output": str(SUBJECT.OUTPUT), "budgetSeconds": 120,
            "test": {"path": str(SUBJECT.TEST), "sha256": SUBJECT.TEST_SHA},
            "resolver": {"path": str(SUBJECT.ROOT / "tools/maintenance/WindowsPathResolver.psm1"), "sha256": SUBJECT.RESOLVER_SHA},
            "amendment": {"sha256": "bfb1036013cab88b3a509982830bba7ecb65fa1a2fe9c423feef4d6494b41d51"}}


class ResolverObservationContractTests(unittest.TestCase):
    def test_rejects_relabelled_request_and_unregistered_output(self):
        for field, value in [("requestId", "historical-s77"), ("attemptId", "retry"),
                             ("output", "C:/Temp/observer"), ("budgetSeconds", 121)]:
            value_plan = plan()
            value_plan[field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                SUBJECT.validate_plan(value_plan)

    def test_rejects_changed_test_module_or_missing_owner_amendment(self):
        for field in ("test", "resolver", "amendment"):
            value_plan = copy.deepcopy(plan())
            value_plan[field]["sha256"] = "0" * 64
            with self.subTest(field=field), self.assertRaises(ValueError):
                SUBJECT.render_runner(value_plan)

    def test_entry_failure_xml_is_not_six_case_coverage(self):
        proof = SUBJECT.xml_cases(b'<test-results total="1" failures="1"><test-suite><results><test-case name="Error in test script" executed="True" success="False" result="Failure"><failure><message>null Path</message><stack-trace>line 2</stack-trace></failure></test-case></results></test-suite></test-results>')
        self.assertEqual(proof["originalNamedCases"], 0)
        self.assertEqual(proof["cases"][0]["message"], "null Path")
        self.assertEqual(proof["cases"][0]["stackTrace"], "line 2")
        self.assertFalse(SUBJECT.xml_cases(None)["present"])

    def test_raw_encoding_never_discards_original_bytes(self):
        proof = SUBJECT.output_encoding(b"\xff\xfe\x00\xff", 65001)
        self.assertEqual(proof["bytes"], 4)
        self.assertIsNone(proof["selected"])
        self.assertEqual(proof["strictRoundtripEncodings"], [])


if __name__ == "__main__":
    unittest.main()

"""The shared test job keeps ordinary and previously installed routes usable."""
from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location("test_job_subject", ROOT / "tools/jenkins-management/test_job.py")
job = importlib.util.module_from_spec(spec)
spec.loader.exec_module(job)

RUN = "retained-diagnostic-r4"
ADDON = "// BEGIN ZIRCON SCOPED COMMIT TESTS\nif (params.RUN_ID.startsWith('checkout-commit-')) { return }\n// END ZIRCON SCOPED COMMIT TESTS\n"
ORDINARY = "def ordinaryRun = params.RUN_ID\nnode('zircon-windows-agent') { echo ordinaryRun }\n"
DIAGNOSTIC = ("def runId = params.RUN_ID.toString().trim()\n"
              "if (runId != '" + RUN + "') error('Unexpected RUN_ID')\n"
              "node('zircon-windows-agent') {\n"
              "if (env.JOB_NAME != 'zircon-management-tests') error('wrong job')\n"
              "echo 'original pinned program'\n}\n")


def config(script):
    root = ET.Element("flow-definition")
    ET.SubElement(root, "disabled").text = "false"
    ET.SubElement(root, "description").text = "foreign description remains"
    definition = ET.SubElement(root, "definition", {"class": "org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition"})
    ET.SubElement(definition, "script").text = script
    ET.SubElement(definition, "sandbox").text = "true"
    return ET.tostring(root)


class Manager:
    def __init__(self):
        self.raw = config(ADDON.rstrip() + "\n\n" + DIAGNOSTIC)
        self.posts, self.busy, self.drift, self.lost_ack, self.bad_readback = [], False, False, False, False
        self.reads = 0

    def _request(self, endpoint, method="GET", data=None, headers=None):
        if endpoint == job.ENDPOINT:
            if method == "POST":
                self.posts.append(data)
                self.raw = config("wrong") if self.bad_readback else data
                if self.lost_ack:
                    raise TimeoutError("acknowledgement lost after Jenkins saved the exact config")
                return 200, b""
            self.reads += 1
            if self.drift and self.reads == 2:
                self.raw = config(DIAGNOSTIC)
            return 200, self.raw
        if endpoint.startswith("job/zircon-management-tests/api/"):
            return 200, json.dumps({"inQueue": False, "builds": [{"number": 54, "building": self.busy}]}).encode()
        if endpoint.startswith("queue/api/"):
            return 200, b'{"items": []}'
        if endpoint == "crumbIssuer/api/json":
            return 200, b'{"crumbRequestField":"test-crumb", "crumb":"fixture"}'
        raise AssertionError(endpoint)


class SharedTestJobTests(unittest.TestCase):
    def restore(self, manager):
        return job.restore(manager, job.sha(manager.raw), ORDINARY, ADDON, RUN, lambda: None)

    def test_restoration_retains_exact_specialized_program_and_commit_route_and_is_idempotent(self):
        manager = Manager()
        result = self.restore(manager)
        self.assertTrue(result["ordinaryRoute"] and result["scopedCommitRoutePreserved"])
        script = ET.fromstring(manager.raw).findtext("definition/script")
        self.assertTrue(script.startswith(ADDON.rstrip() + "\n\n"))
        self.assertIn(DIAGNOSTIC, script)
        self.assertIn("params.RUN_ID.toString().trim() == '" + RUN + "'", script)
        self.assertTrue(script.endswith(ORDINARY))
        self.assertEqual("foreign description remains", ET.fromstring(manager.raw).findtext("description"))
        self.assertTrue(self.restore(manager)["unchanged"])
        self.assertEqual(1, len(manager.posts))

    def test_active_tests_config_drift_unknown_route_and_wrong_owner_are_not_overwritten(self):
        for mode in ("busy", "drift", "unknown", "wrong-job", "wrong-extension"):
            with self.subTest(mode=mode):
                manager = Manager()
                if mode in ("busy", "drift"):
                    setattr(manager, mode, True)
                elif mode == "unknown":
                    manager.raw = config("arbitrary foreign program")
                elif mode == "wrong-job":
                    manager.raw = config(DIAGNOSTIC.replace("env.JOB_NAME", "env.OTHER"))
                else:
                    manager.raw = config(ADDON.replace("return", "error('foreign')") + DIAGNOSTIC)
                with self.assertRaises(ValueError):
                    self.restore(manager)
                self.assertEqual([], manager.posts)

    def test_lost_post_acknowledgement_is_reconciled_and_mismatched_readback_is_never_replayed(self):
        manager = Manager()
        manager.lost_ack = True
        self.assertEqual("TimeoutError", self.restore(manager)["postTransportError"])
        self.assertEqual(1, len(manager.posts))
        manager = Manager()
        manager.bad_readback = True
        with self.assertRaisesRegex(ValueError, "read-back differs"):
            self.restore(manager)
        self.assertEqual(1, len(manager.posts))


class RegressionCapsuleTests(unittest.TestCase):
    def test_current_product_schema_and_legacy_schema_keep_complete_capsule_and_client_pins(self):
        for version in (1, 2):
            with self.subTest(version=version):
                self.verify(version)

    def test_new_schema_does_not_admit_unknown_versions_changed_file_bytes_or_changed_clients(self):
        for failure, expected in (("schema", "management_product_capsule_files_invalid"),
                                  ("file", "management_capsule_file_mismatch"),
                                  ("client", "native_helper_capsule_pin_mismatch")):
            with self.subTest(failure=failure), self.assertRaisesRegex(ValueError, expected):
                self.verify(3 if failure == "schema" else 2, failure)

    def verify(self, version, failure=None):
        checks_spec = importlib.util.spec_from_file_location("test_job_regression_checks", ROOT / "tools/jenkins-management/run_checks.py")
        checks = importlib.util.module_from_spec(checks_spec)
        checks_spec.loader.exec_module(checks)
        from tools.jenkins.deployment import driver
        # Jenkins sets TEMP to this run's registered output namespace. The only
        # fixture bytes are a capsule metadata document, never source copies.
        with tempfile.TemporaryDirectory(prefix="routing-capsule-") as temporary:
            repo = Path(temporary)
            files = {name: "1" * 64 for name in ("runner.py", "source_preparation.py",
                     "zircon-management.groovy", "checkout_guard.py", "checkout_product.py")}
            clients = {name: "2" * 64 for name in ("tools/dev/local_cargo.py", "tools/dev/local-cargo.ps1",
                       "tools/dev/dev-fast-build.ps1", "tools/dev/submit-jenkins-management.ps1",
                       "tools/jenkins/frontend.py", "tools/jenkins-management/entry.py")}
            metadata = {"schemaVersion": 1, "repositoryRoot": str(repo), "agentLabel": "zircon-windows-agent",
                        "driverDigest": "d" * 64, "runtimeOperationId": "runtime", "files": files,
                        "clientSources": clients, "checkoutProduct": {"enabled": True,
                        "kind": "zircon-development-editor-preview", "schemaVersion": version,
                        "helper": "checkout_product.py"}}
            raw = json.dumps(metadata).encode()
            checksum = job.sha(raw)
            capsule = repo / ".jenkins/runtime/management" / checksum
            capsule.mkdir(parents=True)
            (capsule / "capsule.json").write_bytes(raw)
            selected = {"driverDigest": "d" * 64, "runtimeOperationId": "runtime",
                        "root": str(repo / ".jenkins/runtime/drivers" / ("d" * 64))}
            installed = {"capsuleSha256": checksum, "capsule": str(capsule),
                         "driverDigest": "d" * 64, "runtimeOperationId": "runtime"}
            observed = dict(clients)
            if failure == "client":
                observed["tools/jenkins-management/entry.py"] = "3" * 64
            with patch.object(checks, "read_json", side_effect=[selected,
                       {"operationId": "runtime", "state": "running"}, installed]), \
                    patch.object(checks, "stable_hash", return_value="3" * 64 if failure == "file" else "1" * 64), \
                    patch.object(driver, "verify_driver") as verify:
                self.assertEqual(checksum, checks.verify_pins(repo, "d" * 64, "runtime", observed))
                verify.assert_called_once_with(repo / ".jenkins/runtime/drivers" / ("d" * 64), expected_digest="d" * 64)


if __name__ == "__main__":
    unittest.main()

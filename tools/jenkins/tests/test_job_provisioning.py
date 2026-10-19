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
        from tools.jenkins.deployment.job_provisioning import _service_xml_shape
        with_actions = ET.tostring(config)
        actions = config.find("actions")
        self.assertIsNotNone(actions)
        config.remove(actions)
        without_actions = ET.tostring(config)
        self.assertEqual(_service_xml_shape(with_actions), _service_xml_shape(without_actions))
        for nonempty in ('<actions foreign="true"/>', '<actions>foreign</actions>',
                         '<actions> </actions>', '<actions><foreign/></actions>'):
            with self.subTest(actions=nonempty):
                changed = ET.fromstring(without_actions)
                changed.insert(0, ET.fromstring(nonempty))
                self.assertNotEqual(_service_xml_shape(without_actions),
                                    _service_xml_shape(ET.tostring(changed)))

        from copy import deepcopy
        import hashlib
        import json
        from html import escape
        from types import SimpleNamespace
        from unittest.mock import patch
        from tools.jenkins.deployment import independent_service as service, job_provisioning as jobs
        control_root = self.root / ".jenkins/state/management/reports/zircon-git-sync-20261018-r1"
        def ref(name, sha, size):
            return {"path": str(control_root / name), "sha256": sha, "bytes": size}
        bridge = {"controlRoot": str(control_root), "ownerThreadId": "01a12fdb-4d74-75d1-92fa-ca578416c6ab",
                  "controller": ref("controller-r4.py", "9686b02535f8919372f5e279a557b3f1f5ceb278897aee800b34b39bb2b71e61", 7253),
                  "worker": ref("worker-r4.py", "8b2bd6b9e483c114c60b469921ccf750a59b94c3557732e8f2236a9384f1d59a", 25125),
                  "ownershipRef": ref("artifacts.json", "a" * 64, 1255)}
        controller = ref("controller-r5-mq19.py", "1ec2f68288e56c954879b2987b3e945246093b1a2405fae6283b74a02a3eae17", 7393)
        worker = ref("worker-r5-mq19.py", "6e76b4a07f7949adbde3f14534c3ccdb2cbad8a602c71f545ab7226b632ce908", 30283)
        recovery = {"path": str(Path("E:/cargo-targets/zircon-local/reports/jenkins-merge-queue-repair-20261019/merge_queue_recovery.py")),
                    "sha256": "bdb01051ca2b09b03c1436e1985d71ac469fd4629d74e02b5af953ea06913bf2", "bytes": 6863}
        publisher = ref("publisher.py", "b" * 64, 20)
        plan_ref = ref("runtime-plans/mq19-before-r5.json", "c" * 64, 100)
        unsized = {k: v for k, v in recovery.items() if k != "bytes"}
        runtime = {"driverDigest": "d" * 64, "runtimeOperationId": "old-runtime"}
        plan = {"schemaVersion": 1, "job": "zircon-merge-queue", "repository": str(self.root),
                "runtime": runtime, "controller": controller, "worker": worker, "manifest": bridge["ownershipRef"],
                "sourcePins": {p["path"]: p for p in (controller, worker, unsized)}, "queueRecoverySource": unsized,
                "publisherSource": publisher, "authorization": {"forcePush": False}, "revision": 5}
        argv = ('"' + Path(controller["path"]).as_posix() + '" --self-sha ' + controller["sha256"]
                + " --plan-sha " + plan_ref["sha256"] + ' --plan-file "' + Path(plan_ref["path"]).as_posix() + '"')
        script = "foreign prefix\n" + argv + "\nforeign suffix"
        xml = ("<flow-definition><definition><script>" + escape(script, quote=True)
               + "</script></definition><triggers><hudson.triggers.TimerTrigger><spec>H/15 * * * *</spec>"
               + "</hudson.triggers.TimerTrigger></triggers><properties>"
               + "<org.jenkinsci.plugins.workflow.job.properties.DisableConcurrentBuildsJobProperty>"
               + "<abortPrevious>false</abortPrevious>"
               + "</org.jenkinsci.plugins.workflow.job.properties.DisableConcurrentBuildsJobProperty>"
               + "</properties></flow-definition>")
        before = {**runtime, "jobs": {"zircon-merge-queue": {"xml": xml}}}
        manager = SimpleNamespace(paths=SimpleNamespace(repo=self.root, state=self.root / ".jenkins/state"))
        refs = {p["path"]: p for p in (bridge["controller"], bridge["worker"], bridge["ownershipRef"],
                                       controller, worker, recovery, publisher, plan_ref)}
        ownership = {"ownerThreadId": bridge["ownerThreadId"], "taskId": "zircon-git-sync-20261018-r1",
                     "paths": [str(control_root)]}
        def read_ref(value):
            service._require(value == refs[value["path"]], "Runtime rebind input changed", "service_rebind_pending")
            data = plan if value["path"] == plan_ref["path"] else ownership
            return json.dumps(data).encode()
        published, observed = {}, {}
        def write_plan(path, value):
            published["path"], published["plan"] = path, deepcopy(value)
            raw = json.dumps(value).encode()
            refs[str(path)] = {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}
        def jobs_cas(manager, before, current, output, key, plans, *, driver):
            observed["xml"] = plans[0]["after"]
        with patch.object(jobs, "_service_ref", side_effect=lambda path: refs[str(path)]), \
                patch.object(jobs, "_service_ref_read", side_effect=read_ref), \
                patch.object(service, "_write_control_once", side_effect=write_plan), \
                patch("tools.jenkins.state.atomic.atomic_json"), \
                patch.object(jobs, "_service_jobs_cas", side_effect=jobs_cas), \
                patch.object(jobs, "_service_job", side_effect=lambda manager, name: observed):
            binding = jobs._service_merge_queue_binding(manager, before, {"mergeQueue": bridge})
            self.assertEqual(controller, binding["bridge"]["controller"])
            self.assertEqual(worker, binding["bridge"]["worker"])
            self.assertEqual("controller-r4.py", Path(bridge["controller"]["path"]).name)
            before["mergeQueue"] = binding
            result = jobs._service_merge_queue_rebind(manager, before, {"operationId": "new-runtime"},
                                                      self.root / "unused-report", driver="e" * 64)
            successor = published["plan"]
            self.assertEqual(controller, result["controller"])
            self.assertEqual(worker, result["worker"])
            self.assertEqual({"driverDigest": "e" * 64, "runtimeOperationId": "new-runtime"}, successor["runtime"])
            self.assertEqual(6, successor["revision"])
            self.assertEqual(unsized, successor["queueRecoverySource"])
            self.assertEqual(plan["sourcePins"], successor["sourcePins"])
            self.assertEqual(plan["authorization"], successor["authorization"])
            self.assertEqual(publisher, successor["publisherSource"])
            self.assertEqual(plan_ref["sha256"], successor["previousPlan"]["sha256"])
            self.assertEqual(control_root / "runtime-plans/new-runtime.json", published["path"])
            self.assertIn("controller-r5-mq19.py", ET.fromstring(observed["xml"]).findtext("definition/script"))
            self.assertEqual(jobs._service_xml_shape(xml, bindings=True),
                             jobs._service_xml_shape(observed["xml"], bindings=True))
            plan["queueRecoverySource"] = {**unsized, "bytes": None}
            with self.assertRaisesRegex(JenkinsError, "Runtime rebind input changed"):
                jobs._service_merge_queue_binding(manager, before, {"mergeQueue": bridge})
            plan["queueRecoverySource"] = {**unsized, "sha256": "0" * 64}
            with self.assertRaisesRegex(JenkinsError, "Runtime rebind input changed"):
                jobs._service_merge_queue_binding(manager, before, {"mergeQueue": bridge})

    def test_defaults_require_a_complete_trusted_runtime_binding(self):
        for name in ("ZIRCON_AGENT_LABEL", "ZIRCON_RUNTIME_OPERATION_ID", "ZIRCON_DRIVER_DIGEST"):
            with self.subTest(name=name):
                environment = {**self.environment, name: ""}
                with self.assertRaises(JenkinsError) as raised:
                    _config(self.source, environment)
                self.assertEqual("runtime_binding_missing", raised.exception.code)


if __name__ == "__main__":
    unittest.main()

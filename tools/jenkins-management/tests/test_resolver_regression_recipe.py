"""Run-loop regressions plus bounded real Windows children emitting synthetic Pester success data.

Synthetic objects/XML never establish actual PowerShell/Pester or six-business-case acceptance.
"""
from pathlib import Path
from types import SimpleNamespace
from unittest import mock
import contextlib
import hashlib
import importlib.util
import io
import json
import os
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET

RECIPE = Path(__file__).resolve().parents[1] / "resolver_regression_recipe.py"
spec = importlib.util.spec_from_file_location("resolver_regression_recipe_under_test", RECIPE)
recipe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(recipe)
REAL_LOAD_HELPER = recipe.load_helper
REPOSITORY = RECIPE.parents[2]
UNIT_RUN = "engine-s96-resolver-entry-unit-20261018-r1"
UNIT_ROOT = recipe.OUTPUT.parent / "s96-loader-entry-unit"
UNIT_EVIDENCE = UNIT_ROOT / "controlled-native"

class RecipeRunLoopTests(unittest.TestCase):
    def setUp(self):
        # Validate real registered paths before any fixture or fixture mocks exist.
        self.shared = REAL_LOAD_HELPER()
        temp_parent = self.shared.physical(recipe.ordinary(Path(os.environ["TEMP"])))
        self.assertEqual(temp_parent, UNIT_ROOT / "temp")
        manifest = json.loads(self.shared.pin(UNIT_ROOT / "artifacts.json")[1])
        registration = json.loads(self.shared.checked(manifest["registration"]))
        self.assertEqual(manifest["owner"], UNIT_RUN)
        self.assertEqual(registration["ownerRunId"], UNIT_RUN)
        self.assertEqual(self.shared.physical(Path(registration["root"])), UNIT_ROOT)
        parent_identity = self.shared.native_identity(temp_parent)
        self.directory = tempfile.TemporaryDirectory(prefix="s96-unit-", dir=temp_parent)
        self.retain_unknown_native = False
        def cleanup_fixture():
            if self.retain_unknown_native:
                self.directory._finalizer.detach()
            else:
                self.directory.cleanup()
        self.addCleanup(cleanup_fixture)
        self.root = self.shared.physical(recipe.ordinary(Path(self.directory.name)))
        self.assertEqual(self.shared.native_identity(temp_parent)["fileIdentity"], parent_identity["fileIdentity"])
        self.output = self.root / "result"
        (self.root / "artifacts.json").write_text(json.dumps({
            "owner": UNIT_RUN, "root": str(self.root),
            "paths": ["owned-input.txt", "plan.json", "result"],
            "fixtureOnly": True, "sourceCopies": 0,
            "retention": "Remove owned fixtures after confirmed native closure; retain unknown native evidence"}), encoding="utf-8")
        self.fixture = self.root / "owned-input.txt"
        self.fixture.write_text("original unit input", encoding="utf-8")
        self.now = 0.0
        self.starts = []
        self.native_exit = 0
        self.complete = True
        self.fail_pin = None
        self.fail_post = False
        self.malformed_xml = False
        self.source_drift = False
        self.mutate_object = lambda value: None
        self.mutate_xml = lambda value: None
        self.native = {"pid": 4242, "creationTime": "mocked-birth", "rootExitCode": 0,
                       "processTree": {"jobHandleRetainedUntilCliExit": False}}
        component = self.root / "component/component.json"
        pester = self.root / "Pester/Pester.psd1"
        self.runtime = {"driverDigest": "a" * 64, "runtimeOperationId": "unit-current"}
        dependencies = [RECIPE, recipe.REGRESSIONS, recipe.HELPER, recipe.TEST,
                        self.fixture, component, self.root / "resolver.psm1",
                        self.root / "powershell.exe", Path(sys.executable), pester]
        facts = {name: {"sha256": "0" * 64} for name in
                 ("local_cargo.py", "run_checks.py", "delivery.py", "branch_integration.py", "checkout_commit.py")}
        dependencies += [component.parent / name for name in facts]
        self.plan = {"executionAdmitted": True, "dispatchAdmitted": True,
            "requestId": recipe.REQUEST, "attemptId": recipe.REQUEST, "runId": recipe.RUN,
            "output": str(self.output), "overallBudgetSeconds": 180, "terminalDrainBudgetSeconds": 30,
            "maximumNativeStarts": 1, "test": {"path": str(recipe.TEST), "sha256": recipe.TEST_SHA},
            "resolver": {"path": str(self.root / "resolver.psm1")}, "expectedOriginalCases": list(recipe.ORIGINAL_CASES),
            "pester": {"path": str(pester)}, "pesterMembers": [str(pester)],
            "pinnedInputs": [{"path": str(p)} for p in dependencies], "controlPins": [],
            "component": {"path": str(component)}, "runtime": self.runtime,
            "powershell": str(self.root / "powershell.exe"), "artifactRegistration": {"unitOnly": True},
            "runnerSha256": hashlib.sha256(recipe.SCRIPT.lstrip().replace("\n", "\r\n").encode("utf-8-sig")).hexdigest()}
        fixture_sha = hashlib.sha256(self.fixture.read_bytes()).hexdigest()
        def observe(expected, before=None):
            if before is not None and self.fail_post:
                raise OSError("unit post-input read failed")
            matches = hashlib.sha256(self.fixture.read_bytes()).hexdigest() == fixture_sha
            return [{"actual": {"path": item["path"]}, "matchesPlan": matches, "unchanged": matches}
                    for item in expected]
        def pin(path):
            path = Path(path)
            if path.name == self.fail_pin:
                raise OSError("unit raw/result read failed")
            raw = path.read_bytes()
            return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}, raw
        def xml_result(raw):
            result = {"present": raw is not None, "attributes": None, "cases": [], "parseError": None}
            try:
                root = ET.fromstring(raw)
                result["attributes"] = dict(root.attrib)
                result["cases"] = [{**node.attrib, "message": node.findtext("failure/message"),
                                    "stackTrace": node.findtext("failure/stack-trace")}
                                   for node in root.iter("test-case")]
            except Exception as error:
                result["parseError"] = str(error)
            return result
        cargo = SimpleNamespace(_run_owned_command=lambda *args: self.native_call(*args))
        environment = SimpleNamespace(runtime_files=lambda root: (None, self.runtime, None),
            environment=lambda output, runtime: {key: str(output / "temp") for key in ("TEMP", "TMP", "TMPDIR")})
        self.helper = SimpleNamespace(observe_inputs=observe,
            checked=lambda item: json.dumps({"files": facts}).encode(), pester_members=lambda plan: [str(pester)],
            physical=lambda p: Path(p), same_path=lambda a, b: str(Path(a)).casefold() == str(Path(b)).casefold(),
            load=lambda name, path, sha: cargo if Path(path).name == "local_cargo.py" else
                environment if Path(path).name == "delivery.py" else SimpleNamespace(),
            devices=lambda: {"unit-mapping": "preserved"},
            native_identity=lambda p: {"fileIdentity": "mocked-unit-temp-identity"},
            pin=pin, xml_result=xml_result,
            native_stream_observation=lambda *args: {"nativeComplete": self.complete,
                "bothStreamsComplete": self.complete, "errors": []})
        stack = contextlib.ExitStack()
        self.addCleanup(stack.close)
        stack.enter_context(mock.patch.object(recipe, "ROOT", self.root))
        stack.enter_context(mock.patch.object(recipe, "OUTPUT", self.output))
        stack.enter_context(mock.patch.object(recipe, "load_helper", return_value=self.helper))
        stack.enter_context(mock.patch.object(recipe, "clock", side_effect=lambda: self.now))
        stack.enter_context(mock.patch.object(recipe, "system_code_page", return_value=65001))
        stack.enter_context(mock.patch.object(sys, "path", list(sys.path)))
        stack.enter_context(mock.patch.dict(os.environ, {"JOB_NAME": "zircon-management-tests",
            "NODE_NAME": "zircon-windows-agent", "RUN_ID": recipe.RUN, "BUILD_NUMBER": "123"}, clear=False))
        self.self_sha = hashlib.sha256(RECIPE.read_bytes()).hexdigest()

    def native_call(self, command, cwd, environment, leaf, timeout):
        self.starts.append({"command": command, "cwd": cwd, "environment": environment, "timeout": timeout})
        observation = {"metadata": {"processId": 4242, "birthFileTimeUtc": "mocked-birth",
            "edition": "Desktop", "powershell": "5.1.0", "pesterVersion": "3.4.0",
            "buildNumber": environment.get("BUILD_NUMBER"), "jobName": environment.get("JOB_NAME"),
            "nodeName": environment.get("NODE_NAME"), "runId": environment.get("RUN_ID"),
            "pesterPath": self.plan["pester"]["path"],
            "pesterModulePath": str(Path(self.plan["pester"]["path"]).with_suffix(".psm1")),
            "executable": self.plan["powershell"], "cwd": str(self.root),
            **{k: str(self.output / "temp") for k in ("temp", "tmp", "tmpdir", "physicalTemp")}},
            "originalCounts": {"total": 6, "passed": 6, "failed": 0, "skipped": 0, "pending": 0, "inconclusive": None},
            "cases": [{"name": name, "describe": recipe.DESCRIBE, "context": "", "result": "Passed",
                       "passed": True, "failureMessage": "", "stackTrace": ""} for name in recipe.ORIGINAL_CASES],
            "normalEntry": True, "sourceRewritten": False, "breakpointUsed": False, "traceUsed": False,
            "repoRootInjected": False, "bootstrapError": None, "collectorErrors": [], "errors": [],
            "testDrivePresentAfterInvoke": False}
        self.mutate_object(observation)
        (self.output / "observation.json").write_text(json.dumps(observation), encoding="utf-8")
        xml = ET.Element("test-results", {"name": "Pester", "total": "6", "errors": "0", "failures": "0",
            "not-run": "0", "inconclusive": "0", "ignored": "0", "skipped": "0", "invalid": "0"})
        top = ET.SubElement(xml, "test-suite", {"name": "Pester", "type": "Powershell",
            "executed": "True", "result": "Success", "success": "True"})
        fixture = ET.SubElement(ET.SubElement(top, "results"), "test-suite",
            {"name": recipe.DESCRIBE, "type": "TestFixture", "executed": "True", "result": "Success", "success": "True"})
        results = ET.SubElement(fixture, "results")
        for name in recipe.ORIGINAL_CASES:
            ET.SubElement(results, "test-case", {"name": recipe.DESCRIBE + "." + name, "description": name,
                "executed": "True", "success": "True", "result": "Success"})
        self.mutate_xml(xml)
        ET.ElementTree(xml).write(self.output / "pester-results.xml", encoding="utf-8")
        if self.malformed_xml:
            (self.output / "pester-results.xml").write_bytes(b"<broken")
        if self.source_drift:
            self.fixture.write_text("changed unit input", encoding="utf-8")
        self.native["rootExitCode"] = self.native_exit
        return self.native

    def invoke(self):
        plan_file = self.root / "plan.json"
        plan_file.write_text(json.dumps(self.plan), encoding="utf-8")
        plan_sha = hashlib.sha256(plan_file.read_bytes()).hexdigest()
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            try:
                return recipe.run(plan_file, plan_sha, self.self_sha)
            finally:
                self.last_stdout, self.last_stderr = out.getvalue(), err.getvalue()

    def summary(self):
        return json.loads((self.output / "summary.json").read_text(encoding="utf-8"))

    def assert_rejected(self):
        self.assertNotEqual(self.invoke(), 0)
        self.assertFalse(self.summary()["normalSixCaseExecutionPassed"])

    def test_success_requires_run_loop_agreement_and_preserves_native_false(self):
        self.assertEqual(self.invoke(), 0)
        self.assertEqual(len(self.starts), 1)
        result = self.summary()
        self.assertTrue(result["normalSixCaseExecutionPassed"])
        self.assertIsNone(result["originalCounts"]["inconclusive"])
        self.assertIs(result["jobHandleRetainedUntilCliExit"], False)
        self.assertIs(result["formalAcceptance"], False)
        self.assertEqual(self.starts[0]["environment"]["BUILD_NUMBER"], "123")

    def test_exit_zero_loader_is_failure(self):
        def loader(value):
            value["originalCounts"].update(total=1, passed=0, failed=1)
            value["cases"] = [{"name": "Error in test script", "result": "Failed"}]
        self.mutate_object = loader
        self.assert_rejected()

    def test_exit_zero_duplicate_xml_identity_is_failure(self):
        def duplicate(xml):
            cases = list(xml.iter("test-case"))
            cases[1].attrib.update(cases[0].attrib)
        self.mutate_xml = duplicate
        self.assert_rejected()

    def test_exit_zero_missing_xml_case_is_failure(self):
        self.mutate_xml = lambda xml: xml.find(".//test-suite/results/test-suite/results").remove(list(xml.iter("test-case"))[-1])
        self.assert_rejected()

    def test_exit_zero_pending_object_is_failure(self):
        self.mutate_object = lambda value: value["cases"][0].update(result="Pending", passed=False)
        self.assert_rejected()

    def test_exit_zero_skipped_xml_is_failure(self):
        self.mutate_xml = lambda xml: next(xml.iter("test-case")).set("executed", "False")
        self.assert_rejected()

    def test_exit_zero_inconclusive_xml_is_failure(self):
        self.mutate_xml = lambda xml: xml.set("inconclusive", "1")
        self.assert_rejected()

    def test_object_xml_disagreement_is_failure(self):
        self.mutate_object = lambda value: value["cases"][0].update(describe="Another Describe")
        self.assert_rejected()

    def test_suffix_identity_is_failure(self):
        self.mutate_xml = lambda xml: next(xml.iter("test-case")).set("name", "Other." + recipe.ORIGINAL_CASES[0])
        self.assert_rejected()

    def test_missing_count_stays_unknown_and_cannot_pass(self):
        self.mutate_object = lambda value: value["originalCounts"].pop("pending")
        self.assert_rejected()
        self.assertIsNone(self.summary()["exactObjectAndXmlAgreement"])

    def test_xml_success_native_nonzero_is_failure(self):
        self.native_exit = 9
        self.assert_rejected()

    def test_xml_success_native_incomplete_stays_pending(self):
        self.complete = False
        self.assert_rejected()
        self.assertEqual(self.summary()["status"], "pending-native-proof")

    def test_wrong_process_identity_is_failure(self):
        self.mutate_object = lambda value: value["metadata"].update(processId=4343)
        self.assert_rejected()

    def test_budget_is_recomputed_after_fsync(self):
        original_fsync = os.fsync
        count = 0
        def delayed(handle):
            nonlocal count
            original_fsync(handle)
            count += 1
            if count == 5:  # launch-intent persistence
                self.now = 124
        with mock.patch.object(recipe.os, "fsync", side_effect=delayed):
            self.assertEqual(self.invoke(), 0)
        self.assertEqual(self.starts[0]["timeout"], 56)

    def test_expired_persistence_budget_starts_no_child(self):
        original_fsync = os.fsync
        count = 0
        def expired(handle):
            nonlocal count
            original_fsync(handle)
            count += 1
            if count == 5:
                self.now = 181
        with mock.patch.object(recipe.os, "fsync", side_effect=expired):
            self.assert_rejected()
        self.assertEqual(self.starts, [])

    def test_post_input_read_failure_cannot_pass(self):
        self.fail_post = True
        self.assert_rejected()
        self.assertIsNone(self.summary()["inputPinsUnchanged"])
        self.assertIsNone(json.loads((self.output / "input-pins-after.json").read_text())["inputs"])

    def test_xml_read_failure_cannot_pass(self):
        self.fail_pin = "pester-results.xml"
        self.assert_rejected()
        self.assertIsNone(self.summary()["exactObjectAndXmlAgreement"])

    def test_post_xml_parse_failure_cannot_pass(self):
        self.mutate_xml = lambda xml: xml.set("total", "unknown")
        self.assert_rejected()

    def test_evidence_write_failure_cannot_pass(self):
        original_open = Path.open
        def failed(path, mode="r", *args, **kwargs):
            if path.name == "xml-observation.json" and mode == "xb":
                raise OSError("unit evidence write failed")
            return original_open(path, mode, *args, **kwargs)
        with mock.patch.object(Path, "open", failed):
            self.assert_rejected()

    def test_summary_write_failure_is_nonzero(self):
        original_open = Path.open
        def failed(path, mode="r", *args, **kwargs):
            if path.name == "summary.json" and mode == "xb":
                raise OSError("unit summary write failed")
            return original_open(path, mode, *args, **kwargs)
        with mock.patch.object(Path, "open", failed):
            self.assertNotEqual(self.invoke(), 0)

    def test_actual_xml_parse_error_cannot_pass(self):
        self.malformed_xml = True
        self.assert_rejected()
        self.assertIsNone(self.summary()["exactObjectAndXmlAgreement"])

    def test_raw_stream_observation_error_cannot_pass(self):
        self.helper.native_stream_observation = mock.Mock(side_effect=OSError("unit raw stream read failed"))
        self.assert_rejected()
        self.assertIsNone(self.summary()["nativeComplete"])

    def test_post_source_drift_cannot_pass(self):
        self.source_drift = True
        self.assert_rejected()
        self.assertIs(self.summary()["inputPinsUnchanged"], False)

    def test_duplicate_object_identity_cannot_pass(self):
        self.mutate_object = lambda value: value["cases"][1].update(name=recipe.ORIGINAL_CASES[0])
        self.assert_rejected()

    def test_extra_xml_loader_case_cannot_pass(self):
        self.mutate_xml = lambda xml: ET.SubElement(xml, "test-case", {"name": "loader"})
        self.assert_rejected()

    def test_runner_error_cannot_be_overridden_by_successful_xml(self):
        self.mutate_object = lambda value: value.update(bootstrapError={"message": "unit runner failed"})
        self.assert_rejected()

    def test_existing_output_is_retained_without_dispatch(self):
        self.output.mkdir()
        marker = self.output / "original.txt"
        marker.write_text("retain", encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "Existing output"):
            self.invoke()
        self.assertEqual(self.starts, [])
        self.assertEqual(marker.read_text(encoding="utf-8"), "retain")

    def test_missing_parent_build_rejected_before_child(self):
        with mock.patch.dict(os.environ, {"BUILD_NUMBER": ""}):
            with self.assertRaisesRegex(ValueError, "BUILD_NUMBER"):
                self.invoke()
        self.assertEqual(self.starts, [])

    def test_invalid_parent_build_rejected_before_child(self):
        with mock.patch.dict(os.environ, {"BUILD_NUMBER": "not-a-build"}):
            with self.assertRaisesRegex(ValueError, "BUILD_NUMBER"):
                self.invoke()
        self.assertEqual(self.starts, [])

    def test_preparation_contract_never_dispatches(self):
        self.plan["executionAdmitted"] = self.plan["dispatchAdmitted"] = False
        with self.assertRaisesRegex(ValueError, "Preparation only"):
            self.invoke()
        self.assertEqual(self.starts, [])

    def test_child_build_number_mismatch_is_failure(self):
        self.mutate_object = lambda value: value["metadata"].update(buildNumber="999")
        self.assert_rejected()
        self.assertFalse(self.summary()["sameProcessMetadataMatches"])

    def test_missing_child_build_number_stays_unknown(self):
        self.mutate_object = lambda value: value["metadata"].pop("buildNumber")
        self.assert_rejected()
        self.assertIsNone(self.summary()["sameProcessMetadataMatches"])

    def test_collector_error_is_not_hidden_by_successful_xml(self):
        self.mutate_object = lambda value: value.update(collectorErrors=[{"phase": "raw-error-encoding"}])
        self.assert_rejected()
        self.assertIs(self.summary()["exactObjectAndXmlAgreement"], False)

    def test_missing_collector_status_is_unknown(self):
        self.mutate_object = lambda value: value.pop("collectorErrors")
        self.assert_rejected()
        self.assertIsNone(self.summary()["exactObjectAndXmlAgreement"])

    def test_expected_negative_test_errors_are_preserved_without_blanket_rejection(self):
        self.mutate_object = lambda value: value.update(errors=[{"message": "expected ShouldThrow"}])
        self.assertEqual(self.invoke(), 0)
        self.assertEqual(json.loads((self.output / "observation.json").read_text())["errors"],
                         [{"message": "expected ShouldThrow"}])

    def test_initial_owned_write_failure_still_emits_failure_summary(self):
        original_open = Path.open
        def failed(path, mode="r", *args, **kwargs):
            if path.name == "artifacts.json" and path.parent == self.output and mode == "xb":
                raise OSError("unit initial owned write failed")
            return original_open(path, mode, *args, **kwargs)
        with mock.patch.object(Path, "open", failed):
            self.assert_rejected()
        self.assertEqual(self.starts, [])
        self.assertIn("unit initial owned write", self.summary()["error"]["message"])
        self.assertTrue(self.summary()["outputOwned"])
        for field in ("sameProcessMetadataMatches", "exactObjectAndXmlAgreement", "dosDevicesUnchanged",
                      "ownedTempAndTestDriveSettled", "nativeComplete"):
            self.assertIsNone(self.summary()[field], field)

    def test_environment_preparation_failure_retains_primary_and_zero_starts(self):
        self.helper.devices = mock.Mock(side_effect=OSError("unit environment observation failed"))
        self.assert_rejected()
        self.assertEqual(self.starts, [])
        self.assertIn("unit environment observation", self.summary()["error"]["message"])

    def test_stream_xml_post_pin_and_persistence_errors_are_all_retained(self):
        self.fail_post = True
        self.fail_pin = "pester-results.xml"
        self.helper.native_stream_observation = mock.Mock(side_effect=OSError("unit primary raw stream failure"))
        original_open = Path.open
        def failed(path, mode="r", *args, **kwargs):
            if path.name == "xml-observation.json" and mode == "xb":
                raise OSError("unit secondary evidence failure")
            return original_open(path, mode, *args, **kwargs)
        with mock.patch.object(Path, "open", failed):
            self.assert_rejected()
        result = self.summary()
        self.assertEqual(result["error"]["phase"], "native-stream-observation")
        self.assertIn("unit primary", result["error"]["message"])
        phases = {item["phase"] for item in result["secondaryErrors"]}
        self.assertTrue({"xml-read", "post-input-pins", "evidence-persistence:xml-observation.json"}.issubset(phases))

    def test_fsync_failure_preserves_primary_and_prevents_success(self):
        original_fsync = os.fsync
        count = 0
        def failed(handle):
            nonlocal count
            count += 1
            if count == 1:
                raise OSError("unit first fsync failure")
            return original_fsync(handle)
        with mock.patch.object(recipe.os, "fsync", side_effect=failed):
            self.assert_rejected()
        self.assertEqual(self.starts, [])
        self.assertIn("unit first fsync", self.summary()["error"]["message"])
        self.assertIsNone(self.summary()["sameProcessMetadataMatches"])
        self.assertIsNone(self.summary()["ownedTempAndTestDriveSettled"])

    def test_summary_failure_stderr_retains_original_primary_and_secondary(self):
        self.fail_post = True
        self.helper.native_stream_observation = mock.Mock(side_effect=OSError("unit primary stream failure"))
        original_open = Path.open
        def failed(path, mode="r", *args, **kwargs):
            if path.name == "summary.json" and mode == "xb":
                raise OSError("unit final summary failure")
            return original_open(path, mode, *args, **kwargs)
        with mock.patch.object(Path, "open", failed):
            self.assertNotEqual(self.invoke(), 0)
        result = json.loads(self.last_stderr)
        self.assertEqual(result["error"]["phase"], "native-stream-observation")
        self.assertTrue({"post-input-pins", "summary-persistence"}.issubset(
            {item["phase"] for item in result["secondaryErrors"]}))
        self.assertFalse(result["normalSixCaseExecutionPassed"])

    def test_existing_foreign_evidence_is_never_overwritten(self):
        original_call = self.native_call
        def collision(*args):
            result = original_call(*args)
            (self.output / "xml-observation.json").write_bytes(b"foreign evidence")
            return result
        self.native_call = collision
        self.assert_rejected()
        self.assertEqual((self.output / "xml-observation.json").read_bytes(), b"foreign evidence")

    def test_failed_post_membership_preserves_observed_pins(self):
        members = self.plan["pesterMembers"]
        self.helper.pester_members = mock.Mock(side_effect=[members, OSError("unit post membership failed")])
        self.assert_rejected()
        result = self.summary()
        self.assertIsNone(result["pesterMembershipUnchanged"])
        self.assertIs(result["inputPinsUnchanged"], True)
        after = json.loads((self.output / "input-pins-after.json").read_text())
        self.assertTrue(after["inputs"])
        self.assertIs(after["unchanged"], True)
        self.assertIsNone(after["pesterMembersAfter"])

    def test_failed_post_devices_preserves_other_successful_facts(self):
        self.helper.devices = mock.Mock(side_effect=[{"unit-mapping": "preserved"}, OSError("unit devices failed")])
        self.assert_rejected()
        result = self.summary()
        self.assertIsNone(result["dosDevicesUnchanged"])
        self.assertIs(result["inputPinsUnchanged"], True)
        self.assertIs(result["ownedTempAndTestDriveSettled"], True)

    def test_observed_device_mismatch_is_false(self):
        self.helper.devices = mock.Mock(side_effect=[{"unit-mapping": "before"}, {"unit-mapping": "after"}])
        self.assert_rejected()
        self.assertIs(self.summary()["dosDevicesUnchanged"], False)

    def test_failed_post_temp_identity_is_unknown_without_erasing_pins(self):
        original = self.helper.native_identity
        def identity(path):
            if self.starts and Path(path) == self.output / "temp":
                raise OSError("unit post temp identity failed")
            return original(path)
        self.helper.native_identity = identity
        self.assert_rejected()
        self.assertIsNone(self.summary()["ownedTempAndTestDriveSettled"])
        self.assertIs(self.summary()["inputPinsUnchanged"], True)
        after = json.loads((self.output / "fixture-after.json").read_text())
        self.assertIsNone(after["ownedTempAfter"])
        self.assertIsNotNone(after["ownedTempBefore"])

    def test_missing_test_drive_observation_is_unknown(self):
        self.mutate_object = lambda value: value.pop("testDrivePresentAfterInvoke")
        self.assert_rejected()
        self.assertIsNone(self.summary()["ownedTempAndTestDriveSettled"])

    def test_observed_remaining_test_drive_is_false(self):
        self.mutate_object = lambda value: value.update(testDrivePresentAfterInvoke=True)
        self.assert_rejected()
        self.assertIs(self.summary()["ownedTempAndTestDriveSettled"], False)

    def install_real_child(self, mode, exit_code):
        """Replace only the native boundary with the pinned retained-Job helper.

        Child objects/XML are explicitly synthetic success-style fixtures. The
        actual PID/birth, exit, Job membership, waiter and raw EOF are genuine.
        """
        self.assertEqual(os.name, "nt")
        shared = self.shared
        component_path = Path(os.environ["S96_NATIVE_COMPONENT"])
        component_raw = recipe.read_hash(component_path, os.environ["S96_NATIVE_COMPONENT_SHA256"])
        facts = json.loads(component_raw)["files"]
        driver_root = shared.physical(Path(os.environ["S96_DRIVER_ROOT"]))
        self.assertEqual(driver_root.parent, REPOSITORY / ".jenkins/runtime/drivers")
        sys.path.insert(0, str(driver_root))
        for name, pin in facts.items():
            recipe.read_hash(component_path.parent / name, pin["sha256"])
        cargo = shared.load("s96_unit_real_cargo", component_path.parent / "local_cargo.py",
                            facts["local_cargo.py"]["sha256"])
        self.actual_cargo = cargo
        checks = shared.load("s96_unit_real_checks", component_path.parent / "run_checks.py",
                             facts["run_checks.py"]["sha256"])
        original_load = self.helper.load
        original_fixture_call = self.native_call
        child_code = r'''
import ctypes, json, os, subprocess, sys, time
from ctypes import wintypes
from pathlib import Path
path = Path(sys.argv[1])
value = json.loads(path.read_text(encoding="utf-8"))
api = ctypes.WinDLL("kernel32", use_last_error=True)
api.GetCurrentProcess.restype = wintypes.HANDLE
api.GetProcessTimes.argtypes = [wintypes.HANDLE, *([ctypes.POINTER(wintypes.FILETIME)] * 4)]
api.GetProcessTimes.restype = wintypes.BOOL
times = [wintypes.FILETIME() for _ in range(4)]
if not api.GetProcessTimes(api.GetCurrentProcess(), *(ctypes.byref(item) for item in times)):
    raise ctypes.WinError(ctypes.get_last_error())
birth = str((times[0].dwHighDateTime << 32) | times[0].dwLowDateTime)
value["metadata"].update(processId=os.getpid(), birthFileTimeUtc=birth,
    buildNumber=os.environ.get("BUILD_NUMBER"), jobName=os.environ.get("JOB_NAME"),
    nodeName=os.environ.get("NODE_NAME"), runId=os.environ.get("RUN_ID"))
if sys.argv[2] == "wrong-metadata":
    value["metadata"]["processId"] += 1
path.write_text(json.dumps(value), encoding="utf-8")
print(json.dumps({"syntheticSixSuccessStyle": True}), flush=True)
if sys.argv[2] == "pipe-holder":
    subprocess.Popen([sys.executable, "-B", "-c", "import time; time.sleep(10)"],
                     stdout=sys.stdout, stderr=sys.stderr)
sys.exit(int(sys.argv[3]))
'''
        def real_call(command, cwd, environment, leaf, timeout):
            # Admission and exclusive evidence registration precede the real start.
            evidence_root = shared.physical(recipe.ordinary(Path(os.environ["S96_UNIT_EVIDENCE_ROOT"])))
            self.assertEqual(evidence_root, UNIT_EVIDENCE)
            evidence = evidence_root / self._testMethodName
            evidence.mkdir()
            with (evidence / "artifacts.json").open("x", encoding="utf-8") as stream:
                json.dump({"owner": UNIT_RUN, "fixture": str(self.root),
                    "paths": ["native-returned.json", "started.json", "stdout.bin", "stderr.bin"],
                    "syntheticPesterPayload": True, "actualNativeEvidence": True,
                    "retain": "Independent native helper regression review; retain unknown original fixtures"}, stream)
                stream.flush(); os.fsync(stream.fileno())
            self.native_evidence_path = evidence
            original_fixture_call(command, cwd, environment, leaf, timeout)
            actual = [sys.executable, "-B", "-X", "utf8", "-c", child_code,
                      str(self.output / "observation.json"), mode, str(exit_code)]
            self.actual_child_command, self.actual_child_cwd = actual, cwd
            child_env = dict(os.environ)
            child_env.update(environment)
            self.retain_unknown_native = True
            self.real_native = cargo._run_owned_command(actual, cwd, child_env, leaf,
                min(timeout, 2.0 if mode == "pipe-holder" else 4.0))
            closed = shared.native_closed(self.real_native) is True
            with (evidence / "native-returned.json").open("x", encoding="utf-8") as stream:
                json.dump(self.real_native, stream)
                stream.flush(); os.fsync(stream.fileno())
            if closed:
                for name in ("started.json", "stdout.bin", "stderr.bin"):
                    _, raw = shared.pin(leaf / name)
                    with (evidence / name).open("xb") as stream:
                        stream.write(raw); stream.flush(); os.fsync(stream.fileno())
                self.retain_unknown_native = False
            return self.real_native
        self.helper.load = lambda name, path, sha: SimpleNamespace(_run_owned_command=real_call) \
            if Path(path).name == "local_cargo.py" else original_load(name, path, sha)
        self.helper.native_stream_observation = lambda directory, command, cwd, ignored_checks, acp, native: \
            shared.native_stream_observation(directory, self.actual_child_command, self.actual_child_cwd,
                                             checks, acp, native)

    def test_missing_native_evidence_environment_starts_no_owned_child(self):
        self.install_real_child("exit", 0)
        with mock.patch.object(self.actual_cargo, "_run_owned_command") as call:
            saved = os.environ.pop("S96_UNIT_EVIDENCE_ROOT")
            try:
                self.assert_rejected()
            finally:
                os.environ["S96_UNIT_EVIDENCE_ROOT"] = saved
            call.assert_not_called()
        self.assertEqual(self.starts, [])

    def test_wrong_native_evidence_root_starts_no_owned_child(self):
        self.install_real_child("exit", 0)
        foreign = self.root / "foreign-evidence"
        foreign.mkdir()
        marker = foreign / "original.txt"
        marker.write_bytes(b"retain")
        with mock.patch.dict(os.environ, {"S96_UNIT_EVIDENCE_ROOT": str(foreign)}), \
                mock.patch.object(self.actual_cargo, "_run_owned_command") as call:
            self.assert_rejected()
            call.assert_not_called()
        self.assertEqual(self.starts, [])
        self.assertEqual(marker.read_bytes(), b"retain")

    def test_existing_native_evidence_namespace_starts_no_owned_child(self):
        self.install_real_child("exit", 0)
        # The collision is an owned test fixture; its marker survives admission.
        with tempfile.TemporaryDirectory(prefix="s96-admission-", dir=UNIT_EVIDENCE) as name:
            evidence = Path(name)
            marker = evidence / "original.txt"
            marker.write_bytes(b"retain")
            with mock.patch.object(self, "_testMethodName", evidence.name), \
                    mock.patch.object(self.actual_cargo, "_run_owned_command") as call:
                self.assert_rejected()
                call.assert_not_called()
            self.assertEqual(marker.read_bytes(), b"retain")
        self.assertEqual(self.starts, [])

    def test_owned_call_exception_retains_original_fixture(self):
        self.install_real_child("exit", 0)
        with mock.patch.object(self.actual_cargo, "_run_owned_command", side_effect=OSError("unit owned call failed")) as call:
            self.assert_rejected()
            self.assertEqual(call.call_count, 1)
        self.assertTrue(self.retain_unknown_native)
        self.assertTrue(self.root.exists())
        self.assertTrue((self.native_evidence_path / "artifacts.json").exists())

    def test_native_close_check_exception_retains_original_fixture(self):
        self.install_real_child("exit", 0)
        with mock.patch.object(self.actual_cargo, "_run_owned_command", return_value={}) as call, \
                mock.patch.object(self.shared, "native_closed", side_effect=OSError("unit close check failed")):
            self.assert_rejected()
            self.assertEqual(call.call_count, 1)
        self.assertTrue(self.retain_unknown_native)
        self.assertTrue(self.root.exists())

    def test_real_owned_child_success_style_output_with_nonzero_exit_is_failure(self):
        self.install_real_child("exit", 7)
        self.assert_rejected()
        self.assertEqual(self.real_native["rootExitCode"], 7)
        self.assertTrue(self.summary()["nativeComplete"])
        self.assertIn(b'"syntheticSixSuccessStyle": true',
                      (self.output / "resolver-tests/stdout.bin").read_bytes())
        self.assertFalse(self.retain_unknown_native)

    def test_real_owned_descendant_holding_pipes_times_out_without_success(self):
        self.install_real_child("pipe-holder", 0)
        self.assert_rejected()
        self.assertTrue(self.real_native["cleanup"]["attempted"])
        self.assertTrue(any(item["type"] == "TimeoutError" for item in self.real_native["terminalErrors"]))
        self.assertIn(b'"syntheticSixSuccessStyle": true',
                      (self.output / "resolver-tests/stdout.bin").read_bytes())
        self.assertEqual(self.summary()["status"], "pending-native-proof")
        self.assertFalse(self.retain_unknown_native)

    def test_real_owned_child_wrong_metadata_identity_is_failure(self):
        self.install_real_child("wrong-metadata", 0)
        self.assert_rejected()
        self.assertTrue(self.summary()["nativeComplete"])
        self.assertFalse(self.summary()["sameProcessMetadataMatches"])
        self.assertFalse(self.retain_unknown_native)

    def test_real_owned_child_reads_forwarded_build_environment(self):
        self.install_real_child("exit", 0)
        self.assertEqual(self.invoke(), 0)
        self.assertEqual(self.summary()["childEnvironment"]["buildNumber"], "123")
        self.assertEqual(self.summary()["childEnvironment"]["runId"], recipe.RUN)
        self.assertFalse(self.summary()["formalAcceptance"])
        self.assertFalse(self.retain_unknown_native)

if __name__ == "__main__":
    unittest.main()

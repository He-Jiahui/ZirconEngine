"""Pure S78/S86 contract regressions; no process or fixture tree is created."""
import copy
from contextlib import ExitStack
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location("fixture_owner_subject", ROOT / "tools/jenkins-management/fixture_owner_recipe.py")
SUBJECT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SUBJECT)


def fact(path, digest="0" * 64, size=0):
    return {"path": str(path), "sha256": digest, "bytes": size}


def plan():
    driver = "0fc997ca92dd1a6501aa31f3bb0c1c5c9841f4a4d65cb69fb45931330af83565"
    paths = [SUBJECT.RECIPE, SUBJECT.TESTS, SUBJECT.PYTHON, SUBJECT.POWERSHELL,
             SUBJECT.ROOT / "tools/maintenance/WindowsPathResolver.psm1",
             SUBJECT.ROOT / "tools/mvp/MvpArtifactStoragePolicy.psm1",
             SUBJECT.ROOT / "tools/mvp/mvp-artifact-storage-policy.json",
             SUBJECT.ROOT / "tools/tests/MvpTestCleanup.psm1",
             SUBJECT.ROOT / "tools/build/build-editor.ps1",
             SUBJECT.ROOT / ".jenkins/runtime/drivers" / driver / "driver-manifest.json",
             Path("C:/Program Files/WindowsPowerShell/Modules/Pester/3.4.0/Pester.psd1"),
             *(SUBJECT.ROOT / name for name in SUBJECT.CONSUMER_INPUTS),
             *(SUBJECT.ROOT / "tools/tests" / name for name in SUBJECT.SUITES.values())]
    pins = {SUBJECT.windows_path(path): fact(path) for path in paths}
    for name, (digest, size, lines, mtime) in SUBJECT.FROZEN.items():
        path = SUBJECT.ROOT / name
        pins[SUBJECT.windows_path(path)] = {**fact(path, digest, size), "lines": lines, "mtime_ns": mtime}
    return {"schemaVersion": 1, "taskId": SUBJECT.RUN, "requestId": SUBJECT.RUN, "attemptId": SUBJECT.RUN,
            "output": str(SUBJECT.OUTPUT), "ownerThreadId": "source-owner",
            "message": fact(SUBJECT.MESSAGE, SUBJECT.MESSAGE_SHA),
            "sourceClaim": fact("E:/cargo-targets/zircon-local/reports/claim.json"),
            "artifactRegistration": fact(SUBJECT.REGISTRATION, SUBJECT.REGISTRATION_SHA),
            "component": fact(SUBJECT.COMPONENT / "component.json", SUBJECT.COMPONENT_ID),
            "runtime": {"driverDigest": driver, "runtimeOperationId": "start-1792251299183933800"},
            "powershell": str(SUBJECT.POWERSHELL),
            "pester": {"path": "C:/Program Files/WindowsPowerShell/Modules/Pester/3.4.0/Pester.psd1", "version": "3.4.0"},
            "controlPins": [fact(SUBJECT.ROOT / name) for name in
                (".jenkins/state/deployment/driver.json", ".jenkins/state/deployment-operation.json",
                 ".jenkins/state/management/installed.json", ".jenkins/runtime/management/selected/capsule.json")],
            "pinnedInputs": list(pins.values()), "contractTestsBudgetSeconds": 90,
            "invocations": [{"invocationId": f"s86-f{i + 1:02d}-{suite}", "commandId": f"suite-{suite}",
                             "suite": suite, "keep": False, "budgetSeconds": 180, "argv": []}
                            for i, suite in enumerate(SUBJECT.SUITES)] +
                           [{"invocationId": "s86-f09-fixture-contract-keep", "commandId": "suite-fixture-contract-keep",
                             "suite": "fixture-contract", "keep": True, "budgetSeconds": 180, "argv": []}]}


def invocation(suite="fixture-contract", keep=False):
    return {"suite": suite, "invocationId": "normal-contract", "commandId": "contract", "keep": keep,
            "driverId": "0fc997ca92dd1a6501aa31f3bb0c1c5c9841f4a4d65cb69fb45931330af83565"}


def first_suite_plan():
    candidate = plan()
    candidate.update(mode=SUBJECT.FIRST_SUITE_MODE, output=str(SUBJECT.FIRST_SUITE_OUTPUT),
                     maximumSuiteStarts=1, overallBudgetSeconds=180, expectedOriginalCases=21,
                     message=fact(SUBJECT.FIRST_SUITE_MESSAGE, SUBJECT.FIRST_SUITE_MESSAGE_SHA),
                     artifactRegistration=fact(SUBJECT.FIRST_SUITE_REGISTRATION),
                     producerRegressionEvidence=fact("E:/cargo-targets/zircon-local/reports/producer-regression.json"),
                     capabilityEvidence=fact("E:/cargo-targets/zircon-local/reports/capability-244.json"))
    for name, (digest, size, lines, mtime) in SUBJECT.FIRST_SUITE_FROZEN.items():
        existing = next((p for p in candidate["pinnedInputs"] if SUBJECT.same_path(p["path"], SUBJECT.ROOT / name)), None)
        value = {**fact(SUBJECT.ROOT / name, digest, size), "lines": lines, "mtime_ns": mtime}
        if existing is None:
            candidate["pinnedInputs"].append(value)
        else:
            existing.update(value)
    for item, invocation_id in zip(candidate["invocations"], SUBJECT.FIRST_SUITE_IDS):
        item["invocationId"] = invocation_id
    return candidate


class FirstSuiteModeTests(unittest.TestCase):
    def test_current_full_suite_plan_preserves_original_identity_and_eight_not_run_slots(self):
        candidate = SUBJECT.validate_plan(first_suite_plan())
        self.assertEqual(len(candidate["invocations"]), 9)
        item = next(SUBJECT.selected_invocations(candidate))
        self.assertEqual((item["suite"], item["argv"], item["keep"]), ("build-editor", [], False))
        command = SUBJECT.wrapper_command(candidate, item)
        self.assertEqual(command[command.index("-TaskId") + 1], SUBJECT.RUN)
        self.assertNotIn("-TestName", command)

    def test_a_passing_first_dispatch_still_never_launches_a_second_suite(self):
        for result in ("passed", "failed", "pending"):
            launch = mock.Mock(return_value={"status": result})
            for item in SUBJECT.selected_invocations(SUBJECT.validate_plan(first_suite_plan())):
                launch(item)
            with self.subTest(first_result=result):
                launch.assert_called_once()
                self.assertEqual(launch.call_args.args[0]["invocationId"], "s90-f01-build-editor")

    def test_old_or_wrong_frozen_source_is_rejected_before_launch(self):
        for digest in ("0" * 64, SUBJECT.FROZEN["tools/tests/build-editor.Tests.ps1"][0],
                       "50fb4763bdcd0b6d9d1445c2a4d22aadd21640dcc819c38599224e4468bd3057"):
            candidate = first_suite_plan()
            item = next(p for p in candidate["pinnedInputs"] if SUBJECT.same_path(p["path"], SUBJECT.ROOT / "tools/tests/build-editor.Tests.ps1"))
            item["sha256"] = digest
            with self.subTest(digest=digest), self.assertRaisesRegex(ValueError, "Frozen source pin differs"):
                SUBJECT.validate_plan(candidate)

    def test_real_input_drift_is_rejected_by_checked(self):
        expected = fact(SUBJECT.RECIPE, "1" * 64, 3)
        with mock.patch.object(SUBJECT, "pin", return_value=(fact(SUBJECT.RECIPE, "2" * 64, 3), b"bad")):
            with self.assertRaisesRegex(ValueError, "Pinned input drift"):
                SUBJECT.checked(expected)

    def test_existing_summary_or_invocation_directory_remains_a_no_replay_error(self):
        candidate = first_suite_plan()
        for occupied in (SUBJECT.FIRST_SUITE_OUTPUT, SUBJECT.invocation_directory(candidate["invocations"][0])):
            def observation(value):
                result = mock.Mock()
                result.exists.return_value = SUBJECT.same_path(value, occupied)
                result.is_dir.return_value = SUBJECT.same_path(value, SUBJECT.WRAPPER_OUTPUT)
                return result
            with self.subTest(occupied=occupied), mock.patch.object(SUBJECT, "physical", side_effect=observation):
                with self.assertRaisesRegex(ValueError, "already exists"):
                    SUBJECT.require_fresh_outputs(candidate)

    def test_preflight_time_reduces_the_first_suite_budget_and_exhaustion_prevents_launch(self):
        self.assertEqual(SUBJECT.remaining_budget(10, 180, clock=lambda: 80), 110)
        self.assertEqual(SUBJECT.remaining_budget(10, 30, clock=lambda: 80), 30)
        with self.assertRaisesRegex(TimeoutError, "including preflight"):
            SUBJECT.remaining_budget(10, 180, clock=lambda: 190)

    def test_cannot_change_one_start_cap_total_deadline_or_reuse_old_registration(self):
        for key, value in (("maximumSuiteStarts", 2), ("maximumSuiteStarts", True),
                           ("overallBudgetSeconds", 181), ("overallBudgetSeconds", 0),
                           ("expectedOriginalCases", 1), ("output", str(SUBJECT.OUTPUT)),
                           ("artifactRegistration", fact(SUBJECT.REGISTRATION, SUBJECT.REGISTRATION_SHA))):
            candidate = first_suite_plan()
            candidate[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                SUBJECT.validate_plan(candidate)

    def test_fresh_registration_includes_exact_new_leaf_without_extra_probe_or_keep_launch(self):
        candidate = first_suite_plan()
        manifest = SUBJECT.registration(candidate)
        self.assertEqual(len(manifest["steps"]), 9)
        self.assertIsNone(manifest["fixtureNamespaceRegistration"]["keepInvocationId"])
        self.assertTrue(SUBJECT.same_path(manifest["steps"][0]["report"], SUBJECT.WRAPPER_OUTPUT / "s90-f01-build-editor"))
        self.assertTrue(SUBJECT.registered_output(SUBJECT.FIRST_SUITE_OUTPUT / "summary.json"))
        self.assertFalse(SUBJECT.registered_output(SUBJECT.ORIGINAL_OUTPUT / "s90-unregistered/summary.json"))


def original_events(item=None):
    item = item or invocation()
    owner = "a" * 32
    metadata = {"taskId": SUBJECT.RUN, "requestId": SUBJECT.RUN, "attemptId": SUBJECT.RUN,
                "driverId": item["driverId"], "component": SUBJECT.COMPONENT_ID,
                "commandId": item["commandId"], "invocationId": item["invocationId"], "suite": item["suite"],
                "suiteArgv": [], "suiteSha256": "b" * 64,
                "suitePath": str(SUBJECT.ROOT / "tools/tests" / SUBJECT.SUITES[item["suite"]])}
    base = {"schemaVersion": 1, "ownerId": owner}
    return [{**base, "kind": "owner", "state": "Active", "testOwner": False, "metadata": metadata,
             "processId": 123, "processStartFileTimeUtc": "134300000000000000", "keep": item["keep"],
             "parentPath": "E:/cargo-targets/zircon-local/mvp-test-fixtures-123"},
            {**base, "kind": "suite", "state": "Started", "suite": item["suite"], "suiteArgv": []},
            {**base, "kind": "suite", "state": "Passed", "suite": item["suite"]},
            {**base, "kind": "owner", "state": "Retained" if item["keep"] else "Released",
             "parentState": "Active" if item["keep"] else "Released", "pending": []}]


def proof(events, item=None, raw_xml=None):
    journal = SUBJECT.read_journal(b"\n".join(json.dumps(x).encode() for x in events))
    return SUBJECT.interpret_journal(journal, item or invocation(),
                                     {"pid": 123, "creationTime": "134300000000000000"},
                                     "b" * 64, SUBJECT.xml_result(raw_xml))


def settled_native(command, exit_code=0):
    return {"pid": 123, "creationTime": "134300000000000000", "command": command,
            "cwd": str(SUBJECT.ROOT), "rootExitCode": exit_code, "terminalErrors": [],
            "processTree": {"terminal": True, "scope": "retained-windows-job", "nativeWaiterJoined": True,
                "readersFinished": True, "pipeEOF": True, "jobHandleRetainedUntilCliExit": False,
                "pipes": {name: {"closed": True, "eof": True} for name in ("stdout", "stderr")}},
            "nativeJobEvidence": {"records": {"beforeResume": {"activeProcesses": 1},
                "afterTerminal": {"activeProcesses": 0}, "afterClose": {"activeProcesses": 0}}, "errors": []}}


class ProducerAndSupervisorTests(unittest.TestCase):
    def test_post_pass_evidence_failure_exercises_run_except_finally_without_false_success(self):
        candidate = first_suite_plan()
        item = {**candidate["invocations"][0], "driverId": candidate["runtime"]["driverDigest"]}
        command = SUBJECT.wrapper_command(candidate, item)
        native = settled_native(command)
        raw_xml = ('<test-results total="21" failures="0"><test-suite><results>' +
                   ''.join(f'<test-case name="original-case-{i}" executed="True" success="True" result="Success" />'
                           for i in range(21)) + '</results></test-suite></test-results>').encode()
        events = original_events(item)
        events[0]["metadata"]["suiteSha256"] = SUBJECT.FIRST_SUITE_FROZEN["tools/tests/build-editor.Tests.ps1"][0]
        events.insert(2, {"schemaVersion": 1, "ownerId": "a" * 32, "kind": "pester", "state": "Completed",
                         "version": "3.4.0", "total": 21, "passed": 21, "failed": 0, "skipped": 0,
                         "pending": 0, "xmlCases": 21, "xmlFailures": 0, "resultSha256": hashlib.sha256(raw_xml).hexdigest()})
        journal_raw = b'\n'.join(json.dumps(event).encode() for event in events)
        regression = {"status": "passed", "nativeComplete": True,
                      "sourcePins": {str(SUBJECT.RECIPE): "0" * 64, str(SUBJECT.TESTS): "0" * 64}}
        plan_path = SUBJECT.FIRST_SUITE_OUTPUT.parent / "unit-plan.json"
        written = {}
        launched = False

        def checked(expected):
            if SUBJECT.same_path(expected["path"], plan_path):
                return json.dumps(candidate).encode()
            if SUBJECT.same_path(expected["path"], candidate["producerRegressionEvidence"]["path"]):
                return json.dumps(regression).encode()
            return b"same canonical source bytes"

        def write(path, value, **kwargs):
            if path.name == "journal-observation.json":
                raise OSError("injected post-pass journal evidence persistence failure")
            written[str(path)] = copy.deepcopy(value)

        def read_bytes(path):
            if path.suffix == ".jsonl":
                return journal_raw
            if path.name.endswith(".pester.xml"):
                return raw_xml
            if path.name == "environment-before.json":
                return b'{"temporaryPaths":{"TEMP":{"fileIdentity":"original-temp"}}}'
            raise AssertionError("unexpected file read: " + str(path))

        def run_native(*args):
            nonlocal launched
            launched = True
            return native, True, {}

        delivery = mock.Mock()
        delivery.runtime_files.return_value = (None, candidate["runtime"], None)
        with ExitStack() as stack:
            replacements = {"checked": checked, "expand_inputs": (candidate["pinnedInputs"], {
                name: {"sha256": "0" * 64} for name in ("local_cargo.py", "run_checks.py", "delivery.py")}),
                "source_members": [], "pester_members": [], "observe_inputs": [{"matchesPlan": True, "unchanged": True}],
                "physical": lambda path: Path(path), "load": lambda *args: delivery, "require_fresh_outputs": None,
                "prepare_step": {}, "run_native": run_native, "write": write, "devices": {},
                "membership_observation": {"unchanged": True}, "observe_fixtures": {"satisfied": True, "instances": [], "parents": []},
                "native_identity": {"fileIdentity": "original-temp"}}
            mocks = {}
            for name, value in replacements.items():
                mocks[name] = stack.enter_context(mock.patch.object(SUBJECT, name, **(
                    {"side_effect": value} if callable(value) else {"return_value": value})))
            for name in ("mkdir", "rmdir"):
                stack.enter_context(mock.patch.object(Path, name))
            stack.enter_context(mock.patch.object(Path, "iterdir", side_effect=lambda: iter(())))
            stack.enter_context(mock.patch.object(Path, "exists", autospec=True,
                side_effect=lambda path: launched and (path.suffix == ".jsonl" or path.name.endswith(".pester.xml"))))
            stack.enter_context(mock.patch.object(Path, "read_bytes", autospec=True, side_effect=read_bytes))
            stack.enter_context(mock.patch.dict(SUBJECT.os.environ, JOB_NAME="zircon-management-tests",
                NODE_NAME="zircon-windows-agent", RUN_ID=SUBJECT.RUN, BUILD_NUMBER="1"))
            stack.enter_context(mock.patch.object(SUBJECT.sys, "path", list(SUBJECT.sys.path)))
            stack.enter_context(mock.patch("builtins.print"))
            exit_code = SUBJECT.run(plan_path, "0" * 64, "0" * 64)
        summary = written[str(SUBJECT.FIRST_SUITE_OUTPUT / "summary.json")]
        self.assertNotEqual(exit_code, 0)
        self.assertEqual(summary["producerError"]["type"], "OSError")
        self.assertIn("post-pass", summary["firstActionableFailure"]["message"])
        self.assertFalse(summary["firstSuiteMeasurementPassed"])
        self.assertEqual(summary["status"], "stopped-original-evidence-retained")
        self.assertEqual(summary["invocations"][0]["status"], "passed")
        self.assertEqual(summary["invocations"][0]["pester"]["passed"], 21)
        self.assertEqual(summary["suiteStarts"], 1)
        self.assertTrue(all(row["status"] == "not-run" for row in summary["invocations"][1:]))
        self.assertEqual(summary["cleanup"]["unknown"], [])
        mocks["run_native"].assert_called_once()

    def test_launch_persistence_consumes_deadline_and_exhaustion_starts_no_process(self):
        for persisted_at, expected in ((100, 80), (180, None), (181, None)):
            now = [40]
            run_owned = mock.Mock(return_value="native receipt")
            record = []

            def persist(value):
                record.append(value)
                now[0] = persisted_at

            args = (run_owned, ["original"], SUBJECT.ROOT, {}, Path("E:/cargo-targets/test"), 0, persist, {})
            with self.subTest(persisted_at=persisted_at):
                if expected is None:
                    with self.assertRaises(TimeoutError):
                        SUBJECT.dispatch_after_persistence(*args, clock=lambda: now[0])
                    run_owned.assert_not_called()
                else:
                    self.assertEqual(SUBJECT.dispatch_after_persistence(*args, clock=lambda: now[0]), "native receipt")
                    self.assertEqual(run_owned.call_args.args[-1], expected)
                self.assertEqual(record[0]["budgetAtRecordSeconds"], 140)

    def stream_observation(self, directory, native, *, missing=(), start_override=None):
        start = {key: native[key] for key in ("pid", "creationTime", "command", "cwd")}
        start.update(start_override or {})
        contents = {"native-terminal.json": json.dumps(native).encode(), "started.json": json.dumps(start).encode(),
                    "stdout.bin": b"original stdout\n", "stderr.bin": b"original failure\n"}
        for name in missing:
            contents.pop(name)
        checks = mock.Mock()
        checks.native_complete.return_value = True
        with mock.patch.object(SUBJECT, "physical", side_effect=lambda path: path), \
             mock.patch.object(Path, "exists", autospec=True, side_effect=lambda path: path.name in contents), \
             mock.patch.object(SUBJECT, "pin", side_effect=lambda path: (fact(path), contents[path.name])):
            result = SUBJECT.native_stream_observation(directory, ["original"], SUBJECT.ROOT, checks, 65001)
        return result

    def test_outer_complete_does_not_certify_inner_unjoined_streams(self):
        outer = settled_native(["original"])
        inner = settled_native(["original"])
        inner["processTree"]["pipes"]["stderr"]["closed"] = False
        result_outer = self.stream_observation(Path("E:/cargo-targets/outer"), outer)
        result_inner = self.stream_observation(Path("E:/cargo-targets/inner"), inner)
        self.assertTrue(result_outer["bothStreamsComplete"])
        self.assertFalse(result_inner["bothStreamsComplete"])
        self.assertIsNone(result_inner["streams"]["stdout"]["truncated"])

    def test_unjoined_outer_preserves_existing_partial_bytes_without_complete_label(self):
        native = settled_native(["original"])
        native["processTree"]["nativeWaiterJoined"] = False
        result = self.stream_observation(Path("E:/cargo-targets/outer"), native)
        self.assertFalse(result["nativeComplete"])
        self.assertFalse(result["streams"]["stdout"]["completeBytesRead"])
        self.assertEqual(result["streams"]["stdout"]["completeness"], "partial-or-unknown")
        self.assertEqual(result["streams"]["stdout"]["candidateDecodedFullText"]["utf-8"], "original stdout\n")
        self.assertIsNone(result["streams"]["stdout"]["truncated"])

    def test_failed_execution_can_have_complete_streams_but_missing_stream_cannot(self):
        native = settled_native(["original"], exit_code=2)
        result = self.stream_observation(Path("E:/cargo-targets/inner"), native)
        self.assertTrue(result["bothStreamsComplete"])
        self.assertEqual(result["nativeExitCode"], 2)
        missing = self.stream_observation(Path("E:/cargo-targets/inner"), native, missing=("stderr.bin",))
        self.assertFalse(missing["bothStreamsComplete"])
        self.assertEqual(missing["streams"]["stderr"]["completeness"], "missing")

    def test_own_start_identity_and_exact_command_are_required_for_stream_completeness(self):
        for override in ({"creationTime": "134300000000000001"}, {"pid": 999}, {"command": ["replacement"]}):
            with self.subTest(override=override):
                result = self.stream_observation(Path("E:/cargo-targets/inner"), settled_native(["original"]), start_override=override)
                self.assertFalse(result["identityMatchesStart"])
                self.assertFalse(result["bothStreamsComplete"])


class FixturePlanTests(unittest.TestCase):
    def test_accepts_original_request_with_exact_s86_continuation(self):
        self.assertEqual(len(SUBJECT.validate_plan(plan())["invocations"]), 9)

    def test_rejects_identity_relabel_and_output_aliases(self):
        changes = [("taskId", "retry"), ("requestId", "other"), ("attemptId", "attempt2"),
                   ("output", "C:/Temp/s78"), ("output", str(SUBJECT.OUTPUT) + "/../redirect")]
        for key, value in changes:
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                candidate = plan()
                candidate[key] = value
                SUBJECT.validate_plan(candidate)
        for value in ("x" * 129, "-bad", "bad/name", "a\n"):
            with self.subTest(identity=value), self.assertRaises(ValueError):
                SUBJECT.identity(value, "invocation")

    def test_rejects_unbounded_budget_nonempty_argv_and_reused_identity(self):
        for field, value in (("budgetSeconds", 0), ("budgetSeconds", 120), ("budgetSeconds", 179),
                             ("budgetSeconds", 181), ("budgetSeconds", 601), ("budgetSeconds", True),
                             ("argv", ["-TestName", "only-one"]), ("argv", "[]"), ("argv", [None]),
                             ("keep", True), ("suite", "replacement-suite")):
            with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                candidate = plan()
                candidate["invocations"][0][field] = value
                SUBJECT.validate_plan(candidate)
        candidate = plan()
        candidate["invocations"][1]["invocationId"] = candidate["invocations"][0]["invocationId"].upper()
        with self.assertRaises(ValueError):
            SUBJECT.validate_plan(candidate)

    def test_requires_frozen_four_fields_all_nine_and_original_component(self):
        for name in SUBJECT.FROZEN:
            for field, value in (("sha256", "0" * 64), ("bytes", 1), ("lines", 1), ("mtime_ns", 0)):
                candidate = plan()
                frozen = next(p for p in candidate["pinnedInputs"] if SUBJECT.same_path(p["path"], SUBJECT.ROOT / name))
                frozen[field] = value
                with self.subTest(path=name, field=field), self.assertRaises(ValueError):
                    SUBJECT.validate_plan(candidate)
        candidate = plan()
        candidate["invocations"].pop()
        with self.assertRaises(ValueError):
            SUBJECT.validate_plan(candidate)
        candidate = plan()
        candidate["component"]["sha256"] = "0" * 64
        with self.assertRaises(ValueError):
            SUBJECT.validate_plan(candidate)

    def test_literal_paths_reject_traversal_subst_spellings_and_short_aliases(self):
        for value in ("E:relative", "E:/cargo-targets/a/../b", "E:/cargo-targets/./b",
                      "E:/cargo-targets/a. ", "E:/CARGOT~1/x", "\\\\?\\E:\\cargo-targets\\x", "\\\\server\\share\\x"):
            with self.subTest(path=value), self.assertRaises(ValueError):
                SUBJECT.windows_path(value)
        self.assertTrue(SUBJECT.is_under("E:/cargo-targets/actual", "E:/cargo-targets"))
        self.assertFalse(SUBJECT.is_under("E:/nested/cargo-targets/actual", "E:/cargo-targets"))

    def test_command_is_genuine_original_wrapper_file_with_empty_json_argv(self):
        candidate = plan()
        item = candidate["invocations"][0]
        command = SUBJECT.wrapper_command(candidate, item)
        self.assertEqual(command[command.index("-File") + 1], str(SUBJECT.WRAPPER))
        self.assertNotIn("-Command", command)
        self.assertNotIn("-TestName", command)
        self.assertEqual(command[command.index("-TaskId") + 1], SUBJECT.RUN)
        self.assertEqual(command[command.index("-InvocationId") + 1], item["invocationId"])
        self.assertIn("-PesterResultPath", command)
        self.assertNotIn("-PesterResultPath", SUBJECT.wrapper_command(candidate, candidate["invocations"][-1]))

    def test_continuation_keeps_wrapper_paths_and_excludes_old_siblings(self):
        candidate = plan()
        command = SUBJECT.wrapper_command(candidate, candidate["invocations"][0])
        directory = SUBJECT.ORIGINAL_OUTPUT / "s78-fixture-owner/s86-f01-build-editor"
        for option, name in (("-JournalPath", "fixture-owner-s86-f01-build-editor.jsonl"),
                             ("-SuiteArgumentsJsonPath", "argv.json"),
                             ("-PesterResultPath", "fixture-owner-s86-f01-build-editor.pester.xml")):
            self.assertTrue(SUBJECT.same_path(command[command.index(option) + 1], directory / name))
        self.assertTrue(SUBJECT.same_path(candidate["output"], SUBJECT.ORIGINAL_OUTPUT / "s86-continuation"))
        for path in (SUBJECT.OUTPUT / "summary.json", directory / "argv.json"):
            self.assertTrue(SUBJECT.registered_output(path))
        for path in (SUBJECT.ORIGINAL_OUTPUT / "summary.json",
                     SUBJECT.WRAPPER_OUTPUT / "f01-build-editor/argv.json",
                     SUBJECT.WRAPPER_OUTPUT / "s83-f10-replacement/argv.json"):
            self.assertFalse(SUBJECT.registered_output(path))

    def test_rejects_old_handoff_registration_and_reused_invocation_names(self):
        substitutions = [
            ("message", fact(SUBJECT.ROOT / ".jenkins/records/engine-audit-20261009-01a12097-s78-fixture-owner-source-ready-handoff.md",
                             "9ad60f6319a6670bfb1cd72d6468192788eba1131fbf6716e8ae54e0573e4788")),
            ("artifactRegistration", fact(SUBJECT.REGISTRATION.parent / "s83-fixture-artifact-registration.json",
                                          "6f2af18ea6b001fe0093c2346a64bad61c771b5b6118b0a604ae244b05cfb09c")),
            ("artifactRegistration", fact(SUBJECT.REGISTRATION, "0" * 64)),
            ("output", str(SUBJECT.ORIGINAL_OUTPUT)),
        ]
        for key, value in substitutions:
            candidate = plan()
            candidate[key] = value
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                SUBJECT.validate_plan(candidate)
        for name in ("f01-build-editor", "s83-f01-build-editor-other", "s83-f02-terminal-receipt"):
            candidate = plan()
            candidate["invocations"][0]["invocationId"] = name
            with self.subTest(invocation=name), self.assertRaises(ValueError):
                SUBJECT.validate_plan(candidate)

    def test_registration_lists_all_exact_reports_and_temporary_roots(self):
        manifest = SUBJECT.registration(plan())
        self.assertEqual(len(manifest["steps"]), 11)
        self.assertTrue(SUBJECT.same_path(manifest["steps"][0]["report"], SUBJECT.OUTPUT / "producer-contract-tests"))
        for row, item in zip(manifest["steps"][2:], plan()["invocations"]):
            report = SUBJECT.WRAPPER_OUTPUT / item["invocationId"]
            self.assertTrue(SUBJECT.same_path(row["report"], report))
            self.assertTrue(SUBJECT.same_path(row["TEMP"], report / "temp"))
            self.assertTrue(SUBJECT.same_path(row["TMP"], report / "temp"))
        self.assertFalse(manifest["fixtureNamespaceRegistration"]["metadataGrantsDeletionAuthority"])

    def test_plan_cannot_omit_direct_acceptance_consumer(self):
        candidate = plan()
        target = SUBJECT.ROOT / "tools/tests/Invoke-MvpAcceptanceTestDriver.ps1"
        candidate["pinnedInputs"] = [p for p in candidate["pinnedInputs"] if not SUBJECT.same_path(p["path"], target)]
        with self.assertRaises(ValueError):
            SUBJECT.validate_plan(candidate)


class FixtureEvidenceTests(unittest.TestCase):

    def test_observes_original_s86_released_instance_under_registered_physical_parent(self):
        path = "E:/cargo-targets/zircon-local/mvp-test-fixtures-36612/editor-build-" + "a" * 32
        event = {"ownerId": "b" * 32, "instanceId": "c" * 32, "path": path, "state": "Released", "physicalAbsent": True}
        absent = mock.Mock()
        absent.exists.return_value = False
        absent.is_dir.return_value = False
        with mock.patch.object(SUBJECT, "physical", return_value=absent) as resolve:
            rows = SUBJECT.observe_fixtures({"instanceFinals": [event], "owners": [], "ownerTerminals": []}, False)["instances"]
        resolve.assert_called_once_with(path)
        self.assertTrue(rows[0]["satisfied"])
        self.assertFalse(rows[0]["existsAfterNativeTermination"])
        self.assertFalse(rows[0]["deletionAuthorized"])
        self.assertNotIn("error", rows[0])

    def test_rejects_unregistered_fixture_namespaces_before_physical_observation(self):
        suffix = "/mvp-test-fixtures-36612/editor-build-" + "a" * 32
        for parent in ("E:/cargo-targets", "E:/cargo-targets/other",
                       "E:/other/cargo-targets/zircon-local", "D:/cargo-targets/zircon-local",
                       "C:/cargo-targets/zircon-local"):
            event = {"ownerId": "b" * 32, "instanceId": "c" * 32,
                     "path": parent + suffix, "state": "Released"}
            with self.subTest(parent=parent), mock.patch.object(SUBJECT, "physical") as resolve:
                rows = SUBJECT.observe_fixtures({"instanceFinals": [event], "owners": [], "ownerTerminals": []}, False)["instances"]
            resolve.assert_not_called()
            self.assertFalse(rows[0]["satisfied"])
            self.assertFalse(rows[0]["deletionAuthorized"])
            self.assertEqual(rows[0]["error"], "Unregistered fixture namespace")

    def test_empty_or_corrupt_original_journal_never_invents_compilation(self):
        for raw in (None, b"", b"{bad-json}\n"):
            journal = SUBJECT.read_journal(raw)
            result = SUBJECT.interpret_journal(journal, invocation(), {"pid": 123, "creationTime": "134300000000000000"},
                                               "b" * 64, SUBJECT.xml_result(None))
            self.assertEqual(result["compileStatus"], "unknown")
            self.assertFalse(result["suitePassed"])
            self.assertFalse(result["lifecycleTerminal"])
            self.assertFalse(result["fixtureDeletionAuthority"])
            self.assertEqual(SUBJECT.invocation_status({"rootExitCode": 1}, True, result, False), "failed")

    def test_same_pid_without_original_birth_or_metadata_is_rejected(self):
        events = original_events()
        events[0]["processStartFileTimeUtc"] = "134300000000000001"
        self.assertTrue(proof(events)["errors"])
        events = original_events()
        events[0]["metadata"]["invocationId"] = "other-original-process"
        self.assertTrue(proof(events)["errors"])

    def test_malformed_original_record_is_retained_as_error_without_parser_crash(self):
        events = original_events()
        events.insert(2, {"schemaVersion": 1, "ownerId": "a" * 32, "kind": "fixture", "state": "Reserved", "instanceId": []})
        result = proof(events)
        self.assertTrue(result["errors"])
        self.assertEqual(SUBJECT.invocation_status({"rootExitCode": 1}, True, result, False), "failed")

    def test_suite_passed_before_finally_never_overrides_nonzero_exit(self):
        result = proof(original_events())
        self.assertTrue(result["suitePassed"])
        self.assertFalse(result["liveHandleDisposalObservedByProducer"])
        self.assertEqual(SUBJECT.invocation_status({"rootExitCode": 1}, True, result, True), "failed")
        self.assertEqual(SUBJECT.invocation_status({"rootExitCode": 0}, False, result, True), "pending")
        self.assertEqual(SUBJECT.invocation_status({"rootExitCode": 0}, True, result, True), "passed")
        events = original_events()[:-1]
        self.assertEqual(SUBJECT.invocation_status({"rootExitCode": 0}, True, proof(events), True), "pending")

    def test_real_pester_counts_require_matching_raw_xml_and_no_failure(self):
        item = invocation("build-editor")
        events = original_events(item)
        raw = b'<test-results total="1" failures="0"><test-suite><results><test-case name="actual complete-file case" executed="True" success="True" result="Success" /></results></test-suite></test-results>'
        counts = {"schemaVersion": 1, "ownerId": "a" * 32, "kind": "pester", "state": "Completed", "version": "3.4.0",
                  "total": 1, "passed": 1, "failed": 0, "skipped": 0, "pending": 0, "xmlCases": 1,
                  "xmlFailures": 0, "resultSha256": hashlib.sha256(raw).hexdigest()}
        events.insert(2, counts)
        self.assertTrue(proof(events, item, raw)["pesterCountsAgree"])
        counts["passed"] = 2
        self.assertFalse(proof(events, item, raw)["pesterCountsAgree"])
        counts["passed"] = 1
        counts["total"] = 0
        self.assertFalse(proof(events, item, raw)["pesterCountsAgree"])
        self.assertFalse(proof(events, item)["pesterCountsAgree"])

    def test_preserves_loader_failure_counts_messages_and_stack(self):
        raw = b'<test-results total="1" failures="1"><test-suite><results><test-case name="Error in test script" executed="True" success="False" result="Failure"><failure><message>original null Path</message><stack-trace>line 2</stack-trace></failure></test-case></results></test-suite></test-results>'
        xml = SUBJECT.xml_result(raw)
        self.assertEqual(xml["cases"][0]["message"], "original null Path")
        self.assertEqual(xml["cases"][0]["stackTrace"], "line 2")
        self.assertEqual(xml["attributes"]["failures"], "1")

    def test_cleanup_failure_history_is_retained_after_real_retry(self):
        events = original_events()
        base = {"schemaVersion": 1, "ownerId": "a" * 32, "kind": "fixture", "instanceId": "c" * 32,
                "path": "E:/cargo-targets/zircon-local/mvp-test-fixtures-123/paths-contract-" + "d" * 32}
        events[2:2] = [{**base, "state": "Reserved"}, {**base, "state": "Active", "fileIdentity": "12345678:0000000000000001"},
                       {**base, "state": "CleanupFailed", "error": "genuine locked-file error"},
                       {**base, "state": "Released", "fileIdentity": "12345678:0000000000000001", "physicalAbsent": True}]
        result = proof(events)
        self.assertEqual(result["instanceFinals"][0]["state"], "Released")
        self.assertEqual(result["historicalFailures"][0]["error"], "genuine locked-file error")

    def test_keep_needs_actual_retained_record_then_postprocess_identity(self):
        item = invocation(keep=True)
        result = proof(original_events(item), item)
        self.assertEqual(result["ownerTerminals"][0]["state"], "Retained")
        self.assertFalse(result["liveHandleDisposalObservedByProducer"])
        self.assertEqual(SUBJECT.invocation_status({"rootExitCode": 0}, True, result, False), "pending")
        event = {"state": "Retained", "reason": "KEEP"}
        self.assertTrue(SUBJECT.fixture_satisfied(event, True, True, True, True))
        self.assertFalse(SUBJECT.fixture_satisfied(event, True, False, True, True))
        self.assertFalse(SUBJECT.fixture_satisfied(event, True, True, False, True))
        self.assertFalse(SUBJECT.fixture_satisfied({"state": "Released"}, True, True, True, True))
        self.assertFalse(SUBJECT.fixture_satisfied(event, False, True, True, True))

    def test_job_zero_or_parent_pid_never_substitutes_dual_eof_close_and_wait(self):
        native = {"pid": 123, "creationTime": "134300000000000000", "processTree": {
            "nativeWaiterJoined": True, "readersFinished": True, "pipeEOF": True, "jobHandleRetainedUntilCliExit": False,
            "pipes": {"stdout": {"eof": True, "closed": True}, "stderr": {"eof": True, "closed": True}}},
            "nativeJobEvidence": {"records": {"afterClose": {"activeProcesses": 0}}}}
        self.assertTrue(SUBJECT.native_closed(native))
        for stream in ("stdout", "stderr"):
            candidate = copy.deepcopy(native)
            candidate["processTree"]["pipes"][stream]["closed"] = False
            self.assertFalse(SUBJECT.native_closed(candidate))
        self.assertFalse(SUBJECT.native_closed({"pid": 123, "nativeJobEvidence": native["nativeJobEvidence"]}))


if __name__ == "__main__":
    unittest.main()

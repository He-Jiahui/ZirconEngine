"""Failed startup review uses immutable input and mocked OS observations only."""
from __future__ import annotations

from contextlib import ExitStack
from copy import deepcopy
import hashlib
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, canonical_json, digest
from tools.jenkins.deployment import lost_owner


ROOT = Path(__file__).absolute().parents[3]


class FailedStart:
    def __init__(self):
        self.state, self.home = ROOT / ".jenkins/state", ROOT / ".jenkins/jenkins_home"
        self.manager = SimpleNamespace(paths=SimpleNamespace(repo=ROOT, state=self.state, home=self.home),
            spec=SimpleNamespace(path=ROOT / ".jenkins/deployment-spec.json"),
            java=ROOT / ".jenkins/runtime/jdk/bin/java.exe", war=ROOT / ".jenkins/runtime/jenkins.war")
        self.request_id, self.old_request_id = "2" * 32, "1" * 32
        self.runtime, self.boot, self.files = "start-200", "134358454705000000", {}
        self.paths = {"originalCommand": self.state / "recovery-lifecycle/command.json",
            "startTransaction": self.state / "recovery-lifecycle/start-transactions/start-200.json",
            "attemptedHost": self.state / "deployment/host.tmp", "committedHost": self.state / "deployment/host.json",
            "operation": self.state / "deployment-operation.json", "selected": self.state / "deployment/driver.json"}
        self.identities = {name: {"pid": pid, "creationTime": str(int(self.boot) + 100 + index),
            "executable": str(ROOT / ".jenkins/runtime/python/python.exe") if index < 2 else str(self.manager.java),
            "commandDigest": str(index + 1) * 64} for index, (name, pid) in
            enumerate(zip(("keeper", "supervisor", "controller", "agent"), (43956, 33908, 31012, 24984)))}
        self.holds = [{"domain": domain, "key": str(index), "version": 1,
                       "payload": {"status": "active", "owner": "legacy-" + str(index // 2)}}
            for index, domain in enumerate(("pool_writer_hold", "resource_reservation") * 2)]
        self.execution = {"recordsDigest": "8" * 64, "protectedHolds": self.holds,
            "protectedHoldsDigest": digest(self.holds), "resourceInventoryDigest": "9" * 64,
            "classification": {"activeExecutions": [], "historicalExecutions": [], "protectedHolds": self.holds},
            "nativeProofIssued": False, "executionHoldsReleaseAllowed": False}
        binding = {"repository": str(ROOT), "homePath": str(self.home), "driverDigest": "a" * 64,
                   "driverRoot": str(ROOT / ".jenkins/runtime/drivers" / ("a" * 64)),
                   "endpoint": {"address": "127.0.0.1", "port": 18080}}
        previous_operation = {"operationId": "start-100", "state": "running"}
        previous_host, previous_selector = {"status": "running"}, {"driverDigest": "a" * 64}
        previous_owner = {"operation": {"operationId": "start-100"}}
        previous_runtime = self.put(self.state / "deployment/history/previous-runtime.json", {"kind": "runtime"})
        self.previous_containment = {"driverDigest": "a" * 64, "nativeTerminationClaimed": False}
        self.containment = {"driverDigest": "b" * 64, "nativeTerminationClaimed": False}
        self.previous = {"owner": previous_owner, "descriptorSha256": "c" * 64, "startupBodyDigest": "d" * 64,
            "anchor": SimpleNamespace(digest="e" * 64, value={"protectedHolds": self.holds}),
            "descriptor": {"runtimeReceipt": previous_runtime}, "selected": previous_selector}
        self.fence = {}
        for name in ("deployment/execution-activation.json", "deployment/execution-upgrade-selection.json",
                     "deployment/execution-upgrade-start.json", "formal-lifecycle/keeper.json",
                     "deployment/formal-lifecycle-command.json"):
            self.fence[name] = self.put(self.state / name, {"unchanged": name})["sha256"]
        old_fence = {**self.fence, "deployment/driver.json": "f" * 64}
        initial = {"schemaVersion": 1, "kind": lost_owner.KIND, "requestId": self.old_request_id,
            "scope": "execution-runtime", "outcome": "interrupted", "owner": previous_owner,
            "binding": binding, "descriptorSha256": self.previous["descriptorSha256"],
            "anchorDigest": self.previous["anchor"].digest, "previousRuntimeReceipt": previous_runtime,
            "containmentReview": self.previous_containment, "operationDigest": digest(previous_operation),
            "hostRecordDigest": digest(previous_host), "selectedDriverDigest": digest(previous_selector),
            "lifecycleFence": old_fence, "bootFileTime": self.boot, "executionQuarantine": deepcopy(self.execution),
            **{key: False for key in ("nativeProofIssued", "historicalEofClaimed", "bootBoundaryClaimed",
                                     "runtimeAcceptance", "historicalHoldsReleased", "executionHoldsReleaseAllowed")}}
        self.initial_path = self.state / "recovery-lifecycle/history/recovery-initial-plan-original.json"
        self.put(self.initial_path, initial)
        self.release = {"schemaVersion": 1, "kind": "formal-lost-owner-recovery-release",
            "scope": "execution-runtime", "requestId": self.old_request_id,
            "expectedPreviousOwner": previous_owner, "expectedLifecycleFence": old_fence,
            "protectedHolds": self.holds, "runtimeAcceptance": False, "cargoAcceptance": False}
        for key, path in (("config", ROOT / ".codex/config.toml"), ("deploymentSpec", self.manager.spec.path),
                          ("workflowSpec", ROOT / ".jenkins/workflow-spec.json")):
            self.release[key] = self.put(path, {"fixed": key})
        self.release_ref = self.put(self.state / "deployment/recovery-release.json", self.release)
        original_request = {"schemaVersion": 1, "kind": lost_owner._REQUEST_KIND, "action": "recover",
            "requestId": self.old_request_id, "release": self.release_ref, "keeper": self.identities["keeper"],
            "recoveryDigest": digest(initial), "runtimeAcceptance": False}
        self.request_ref = self.put(self.state / "recovery-lifecycle/history/original-request.json", original_request)
        root = ROOT / ".jenkins/runtime/drivers" / ("b" * 64)
        selected = {"driverDigest": "b" * 64, "root": str(root), "launcher": str(root / "launcher.py"),
            "runtimeOperationId": self.runtime, "state": "selected-for-runtime-start",
            "runtimeAcceptance": False, "cargoAcceptance": False}
        selected_ref = self.put(self.paths["selected"], selected)
        args = [str(ROOT / ".jenkins/runtime/python/python.exe"), selected["launcher"], "__lifecycle-host", "--spec",
            str(self.manager.spec.path), "--java", str(self.manager.java), "--war", str(self.manager.war),
            "--operation-id", self.runtime]
        self.identities["supervisor"]["commandDigest"] = hashlib.sha256("\0".join(args).encode()).hexdigest()
        operation = {"schemaVersion": 2, "state": "failed", "operationId": self.runtime, "generation": self.runtime,
            "component": "lifecycle-host", "controlPlaneOnly": False, "homePath": str(self.home),
            "hostPid": self.identities["supervisor"]["pid"], **self.identities["supervisor"],
            "lostOwnerRecoveryDigest": digest(initial)}
        self.put(self.paths["operation"], operation)
        host = {"status": "starting", "operationId": self.runtime, "generation": self.runtime,
            "hostPid": self.identities["supervisor"]["pid"], "identity": self.identities["controller"],
            "agentLaunchAttempted": False, "startedAt": 100.0}
        self.put(self.paths["committedHost"], host)
        attempted = {**host, "status": "running", "agentLaunchAttempted": True, "controlPlaneOnly": False,
            "controllerIdentity": self.identities["controller"], "agentIdentity": self.identities["agent"],
            "healthProof": {"controller": True, "agent": True, "ready": True}}
        self.put(self.paths["attemptedHost"], attempted)
        command = {"schemaVersion": 1, "kind": lost_owner._COMMAND_KIND, "status": "unknown",
            "request": self.request_ref, "release": self.release_ref, "keeper": self.identities["keeper"],
            "recoveryDigest": digest(initial), "previousRuntimeReceipt": None,
            "runtimeAcceptance": False, "historicalHoldsReleased": False}
        self.put(self.paths["originalCommand"], command)
        transaction = {"schemaVersion": 1, "kind": "formal-recovery-start-transaction", "state": "unknown",
            "operationId": self.runtime, "homePath": str(self.home), "spawnAttempted": True,
            "keeper": self.identities["keeper"], "release": self.release_ref, "request": self.request_ref,
            "operation": operation, "selected": selected, "selectedSha256": selected_ref["sha256"],
            "createdIdentity": self.identities["supervisor"], "createdPid": self.identities["supervisor"]["pid"],
            "requestedCommand": args, "requestedCommandDigest": self.identities["supervisor"]["commandDigest"],
            "previousOperation": previous_operation, "previousHost": previous_host, "previousSelector": previous_selector,
            "previousSelectorSha256": old_fence["deployment/driver.json"],
            "interruption": {**initial, "observedAt": 100.0, "recoveryDigest": digest(initial)},
            **{key: False for key in ("automaticRetry", "runtimeAcceptance", "cargoAcceptance",
                                     "nativeTerminationProven", "historicalHoldsReleased")}}
        self.put(self.paths["startTransaction"], transaction)
        for name, key in (("deployment-operation.json", "operation"), ("deployment/host.json", "committedHost"),
                          ("deployment/driver.json", "selected")):
            self.fence[name] = self.ref(self.paths[key])["sha256"]

    def put(self, path, document):
        self.files[str(path)] = canonical_json(document) + b"\n"
        return self.ref(path)

    def ref(self, path):
        return {"path": str(path), "sha256": hashlib.sha256(self.files[str(path)]).hexdigest()}

    def change(self, key, **changes):
        path = self.paths[key]
        document = lost_owner._document(self.files[str(path)])
        self.put(path, {**document, **changes})

    def observations(self):
        stack = ExitStack()
        stack.enter_context(patch.object(lost_owner, "_raw", side_effect=lambda path: self.files[str(Path(path).absolute())]))
        # Method mocks need their receiver to keep this entire fixture in memory.
        stack.enter_context(patch.object(Path, "exists", autospec=True, side_effect=lambda path: str(path) in self.files))
        stack.enter_context(patch.object(Path, "glob", autospec=True,
            side_effect=lambda path, pattern: iter([self.initial_path]) if pattern == "recovery-initial-plan-*.json" else iter([])))
        stack.enter_context(patch.object(lost_owner, "_assert_request_unused"))
        stack.enter_context(patch.object(lost_owner, "_failed_recovery_release", return_value=self.release))
        stack.enter_context(patch.object(lost_owner, "_verified_issuance", return_value=self.previous))
        stack.enter_context(patch.object(lost_owner, "_verified_containment",
            side_effect=lambda value: self.containment if value["selected"].get("driverDigest") == "b" * 64 else self.previous_containment))
        stack.enter_context(patch.object(lost_owner, "_lifecycle_fence", return_value=self.fence))
        self.boot_probe = stack.enter_context(patch.object(lost_owner, "windows_boot_filetime", return_value=int(self.boot)))
        self.departure_probe = stack.enter_context(patch.object(lost_owner, "observe_departed_identity",
            side_effect=lambda expected: {"pid": expected["pid"], "expectedCreationTime": expected["creationTime"], "status": "absent"}))
        self.inventory_probe = stack.enter_context(patch.object(lost_owner, "runtime_owner_inventory", return_value=[]))
        self.execution_probe = stack.enter_context(patch.object(lost_owner, "_execution_gate", return_value=self.execution))
        stack.enter_context(patch.object(lost_owner, "_home_policy", return_value={"controllerExecutors": 0}))
        self.home_probe = stack.enter_context(patch.object(lost_owner, "observe_home_available", return_value={"staleMarkersDeleted": False}))
        self.endpoint_probe = stack.enter_context(patch.object(lost_owner, "check_port_available"))
        return stack

    def prepare(self):
        return lost_owner.prepare_failed_recovery_start(self.manager, request_id=self.request_id)

    def replacement(self, plan, *, archive_bytes=None):
        artifacts, archives = plan.original_artifacts, {}
        for key in ("originalCommand", "startTransaction", "attemptedHost"):
            path = self.state / "recovery-lifecycle/history" / (key + ".json")
            self.files[str(path)] = artifacts[key]["raw"] if archive_bytes is None else archive_bytes
            archives[key] = self.ref(path)
        plan_ref = self.put(self.state / "migration-baseline/reviewed-failed-plan.json", plan.document)
        release = {"schemaVersion": 1, "kind": "formal-lost-owner-recovery-release", "requestId": self.request_id,
            "scope": "execution-runtime", "expectedPreviousOwner": plan.document["owner"],
            "expectedLifecycleFence": plan.document["lifecycleFence"], "runtimeAcceptance": False, "cargoAcceptance": False,
            "failedOwnStart": {"plan": plan_ref, **{key: plan.document[key] for key in archives}}}
        release_ref = self.put(self.state / "deployment/new-reviewed-release.json", release)
        keeper = {**self.identities["keeper"], "pid": 12345, "creationTime": str(int(self.boot) + 200)}
        request_ref = self.put(self.state / "recovery-lifecycle/history/replacement-request.json",
            {"schemaVersion": 1, "kind": lost_owner._REQUEST_KIND, "requestId": self.request_id,
             "action": "recover", "release": release_ref, "keeper": keeper, "runtimeAcceptance": False,
             "recoveryDigest": plan.recovery_digest})
        interruption_ref = self.put(self.state / "recovery-lifecycle/history/failed-attempt-interruption.json",
            {"schemaVersion": 1, "kind": "failed-recovery-start-interruption", **archives,
             "originalRequest": plan.document["originalRequest"], "operationId": self.runtime,
             "recoveryDigest": plan.recovery_digest, "outcome": "interrupted",
             **{key: False for key in ("nativeProofIssued", "nativeTerminationProven", "historicalEofClaimed",
                                      "runtimeAcceptance", "historicalHoldsReleased")}})
        arguments = {"keeper": keeper, "release_ref": release_ref, "request": request_ref,
            "original_archive": archives["originalCommand"], "transaction_archive": archives["startTransaction"],
            "attempted_host_archive": archives["attemptedHost"], "interruption_ref": interruption_ref}
        command = {"schemaVersion": 1, "kind": lost_owner._COMMAND_KIND, "status": "pending", "keeper": keeper,
            "release": release_ref, "request": request_ref, "recoveryDigest": plan.recovery_digest,
            "replacesOriginalCommand": archives["originalCommand"], "failedOwnStartInterruption": interruption_ref,
            "previousRuntimeReceipt": None, "runtimeAcceptance": False, "historicalHoldsReleased": False}
        return arguments, command


class FailedRecoveryStartTests(unittest.TestCase):
    def test_full_attempt_is_interrupted_without_promoting_candidate_or_releasing_holds(self):
        scenario = FailedStart()
        before = deepcopy(scenario.files)
        with scenario.observations():
            plan = scenario.prepare()
            value = plan.revalidate(scenario.manager)
            self.assertEqual(16, scenario.departure_probe.call_count)
            self.assertEqual(24984, value["owner"]["agent"]["pid"])
            self.assertEqual(scenario.ref(scenario.paths["attemptedHost"]), value["attemptedHost"])
            self.assertEqual(scenario.files[str(scenario.paths["originalCommand"])], plan.original_artifacts["originalCommand"]["raw"])
        self.assertEqual(before, scenario.files)
        self.assertEqual(scenario.holds, value["executionQuarantine"]["protectedHolds"])
        self.assertEqual("failed-own-start", value["recoveryVariant"])
        for key in ("nativeProofIssued", "nativeTerminationProven", "historicalEofClaimed", "runtimeAcceptance",
                    "historicalHoldsReleased", "executionHoldsReleaseAllowed", "attemptedHostCommitted", "automaticRetry"):
            self.assertIs(value[key], False)

    def test_stale_preagent_record_cannot_ignore_missing_candidate_agent(self):
        scenario = FailedStart()
        scenario.change("attemptedHost", agentIdentity=None)
        with scenario.observations(), self.assertRaises(JenkinsError):
            scenario.prepare()

    def test_every_live_or_inconclusive_exact_component_blocks_replacement(self):
        for role in ("keeper", "supervisor", "controller", "agent"):
            with self.subTest(role=role):
                scenario = FailedStart()
                with scenario.observations():
                    def departure(expected):
                        if expected == scenario.identities[role]:
                            raise JenkinsError("deployment_owner_live", "Exact attempted process is still live")
                        return {"pid": expected["pid"], "expectedCreationTime": expected["creationTime"], "status": "absent"}
                    scenario.departure_probe.side_effect = departure
                    with self.assertRaises(JenkinsError):
                        scenario.prepare()
                    scenario.endpoint_probe.assert_not_called()

    def test_revalidation_rejects_changed_command_transaction_candidate_and_selector(self):
        for key, changes in (("originalCommand", {"reasonCode": "changed"}),
                             ("startTransaction", {"updatedAt": 201}),
                             ("attemptedHost", {"startedAt": 101.0}),
                             ("selected", {"runtimeOperationId": "start-201"})):
            with self.subTest(key=key):
                scenario = FailedStart()
                with scenario.observations():
                    plan = scenario.prepare()
                    scenario.change(key, **changes)
                    with self.assertRaises(JenkinsError):
                        plan.revalidate(scenario.manager)

    def test_scope_original_request_replay_and_foreign_transaction_fail_closed(self):
        scenario = FailedStart()
        with scenario.observations():
            for request_id, scope in ((scenario.old_request_id, "execution-runtime"), (scenario.request_id, "quiet-management")):
                with self.subTest(request_id=request_id, scope=scope), self.assertRaises(JenkinsError):
                    lost_owner.prepare_failed_recovery_start(scenario.manager, request_id=request_id, scope=scope)
            scenario.change("startTransaction", request={"path": "foreign", "sha256": "f" * 64})
            with self.assertRaises(JenkinsError):
                scenario.prepare()

    def test_boot_inventory_home_and_endpoint_races_block(self):
        for probe in ("boot_probe", "inventory_probe", "home_probe", "endpoint_probe"):
            with self.subTest(probe=probe):
                scenario = FailedStart()
                with scenario.observations():
                    observed = getattr(scenario, probe)
                    if probe == "boot_probe":
                        observed.side_effect = [int(scenario.boot), int(scenario.boot) + 1]
                    elif probe == "inventory_probe":
                        observed.return_value = [scenario.identities["agent"]]
                    else:
                        observed.side_effect = JenkinsError("observation_unavailable", "Injected ownership conflict")
                    with self.assertRaises(JenkinsError):
                        scenario.prepare()

    def test_native_budget_and_protected_hold_changes_block_without_mutation(self):
        for field in ("recordsDigest", "resourceInventoryDigest", "protectedHoldsDigest"):
            with self.subTest(field=field):
                scenario = FailedStart()
                with scenario.observations():
                    plan = scenario.prepare()
                    scenario.execution[field] = "0" * 64
                    with self.assertRaises(JenkinsError):
                        plan.revalidate(scenario.manager)

    def test_only_private_transition_accepts_bound_replacement_pending_and_start_attempted(self):
        scenario = FailedStart()
        with scenario.observations():
            plan = scenario.prepare()
            arguments, command = scenario.replacement(plan)
            transitioned = plan.authorize_keeper_command(scenario.manager, **arguments)
            self.assertEqual(plan.snapshot, transitioned.snapshot)
            scenario.put(scenario.paths["originalCommand"], command)
            with self.assertRaises(JenkinsError):
                plan.revalidate(scenario.manager)
            transitioned.revalidate(scenario.manager)
            scenario.change("originalCommand", status="start-attempted")
            transitioned.revalidate(scenario.manager)
            with self.assertRaises(JenkinsError):
                transitioned.authorize_keeper_command(scenario.manager, **arguments)

    def test_bound_transition_rejects_foreign_actor_or_archive_tampering_and_native_claims(self):
        for change in ("foreign-keeper", "spawned-child", "archive-bytes", "unknown-status"):
            with self.subTest(change=change):
                scenario = FailedStart()
                with scenario.observations():
                    plan = scenario.prepare()
                    arguments, command = scenario.replacement(plan)
                    transitioned = plan.authorize_keeper_command(scenario.manager, **arguments)
                    if change == "foreign-keeper":
                        command["keeper"] = scenario.identities["keeper"]
                    elif change == "spawned-child":
                        command["createdSupervisor"] = {"child": scenario.identities["supervisor"]}
                    elif change == "archive-bytes":
                        scenario.files[arguments["original_archive"]["path"]] += b" "
                    else:
                        command["status"] = "unknown"
                    scenario.put(scenario.paths["originalCommand"], command)
                    with self.assertRaises(JenkinsError):
                        transitioned.revalidate(scenario.manager)

    def test_semantically_equal_reencoding_is_not_exact_original_archive(self):
        scenario = FailedStart()
        with scenario.observations():
            plan = scenario.prepare()
            raw = plan.original_artifacts["originalCommand"]["raw"] + b" "
            arguments, _ = scenario.replacement(plan, archive_bytes=raw)
            with self.assertRaises(JenkinsError):
                plan.authorize_keeper_command(scenario.manager, **arguments)

    def test_serialized_or_mutated_document_is_not_failed_start_authority(self):
        scenario = FailedStart()
        with scenario.observations():
            plan = scenario.prepare()
            edited = plan.document
            edited["owner"]["agent"]["pid"] = 1
            self.assertEqual(24984, plan.document["owner"]["agent"]["pid"])
            with self.assertRaises(JenkinsError):
                lost_owner.LostFormalOwnerRecovery(plan.snapshot, variant="failed-own-start")


class OriginalSealedRegressionTests(unittest.TestCase):
    def test_historical_complete_inventory_does_not_require_later_added_source(self):
        scenario = FailedStart()
        sources = {name: "a" * 64 for name in {*lost_owner._REVIEWED_CONTAINMENT,
            "tools/jenkins/deployment/manager.py", "tools/jenkins/deployment/recovery_keeper.py",
            "tools/jenkins/deployment/lost_owner.py", "tools/jenkins/deployment/start_transaction.py",
            "tools/jenkins/tests/test_original.py"}}
        sources.update({".jenkins/deployment-spec.json": scenario.release["deploymentSpec"]["sha256"],
                        ".jenkins/workflow-spec.json": scenario.release["workflowSpec"]["sha256"]})
        log = scenario.state / "migration-baseline/original-regression.log"
        scenario.files[str(log)] = b"original passing run\n"
        receipt = {"scope": "complete-current-Jenkins-support-pytest-discovery", "status": "passed",
            "sourceIdentityStable": True, "testsRun": 42, "exitCode": 0, "failures": 0, "errors": 0,
            "collectionErrors": [], "sourceFiles": sources, "log": str(log),
            "logSha256": scenario.ref(log)["sha256"]}
        scenario.release["regression"] = scenario.put(scenario.state / "migration-baseline/original-regression.json", receipt)
        files = {name: {"sha256": checksum} for name, checksum in sources.items()
                 if not name.startswith(".jenkins/") and "/tests/" not in name}
        files["launcher.py"] = {"sha256": hashlib.sha256(lost_owner._LAUNCHER.encode()).hexdigest()}
        files["tools/__init__.py"] = {"sha256": hashlib.sha256(
            b'"""This package belongs to one verified Jenkins snapshot."""\n').hexdigest()}
        manifest = scenario.state / "migration-baseline/original-driver-manifest.json"
        scenario.put(manifest, {"files": files})
        target = {"manifest": str(manifest)}
        with scenario.observations():
            actual = lost_owner._failed_historical_regression(ROOT, scenario.state, scenario.release, target)
            self.assertEqual(42, actual["testsRun"])
            self.assertNotIn("tools/jenkins/state/atomic.py", actual["sourceFiles"])
            changed = deepcopy(files)
            changed["tools/jenkins/deployment/host.py"]["sha256"] = "b" * 64
            scenario.put(manifest, {"files": changed})
            with self.assertRaises(JenkinsError):
                lost_owner._failed_historical_regression(ROOT, scenario.state, scenario.release, target)
            scenario.put(manifest, {"files": files})
            scenario.files[str(log)] += b"changed\n"
            with self.assertRaises(JenkinsError):
                lost_owner._failed_historical_regression(ROOT, scenario.state, scenario.release, target)


if __name__ == "__main__":
    unittest.main()

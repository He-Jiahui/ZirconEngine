"""A spawned pre-agent failure restores only authentic predecessor byte archives."""
from __future__ import annotations

from contextlib import contextmanager, nullcontext
from copy import deepcopy
import hashlib
import json
from pathlib import Path
from unittest import TestCase
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, canonical_json, digest
from tools.jenkins.deployment import failed_upgrade_recovery as recovery, owned_upgrade as upgrade
from tools.jenkins.deployment import recovery_keeper, prelaunch_upgrade_recovery
from tools.jenkins.tests.test_prelaunch_upgrade_recovery import PrelaunchFixture


class SpawnedFailure(PrelaunchFixture):
    def setup_spawned_failure(self):
        self.setup_failure()
        self.retry_target = {key: value.replace("b" * 64, "c" * 64) if isinstance(value, str) else value
                             for key, value in self.new_target.items()}
        self.targets["c" * 64] = self.retry_target
        self.runtime = "start-500"
        args = [str(self.repo / ".jenkins/runtime/python/python.exe"), "-B",
                str(self.repo / "tools/dev/jenkins_recovery_keeper.py"), "--config-sha256",
                self.successor["config"]["sha256"], "--release-file", self.successor_ref["path"],
                "--release-sha256", self.successor_ref["sha256"]]
        self.keeper["commandDigest"] = hashlib.sha256("\0".join(args).encode()).hexdigest()
        self.before_arguments = args
        request = json.loads(self.files[self.command["request"]["path"]])
        request["keeper"] = self.keeper
        self.command["request"] = self.put(Path(self.command["request"]["path"]), request)
        handoff = json.loads(self.files[self.command["ownedUpgradeHandoff"]["path"]])
        handoff.update(keeper=self.keeper, request=self.command["request"])
        self.command["ownedUpgradeHandoff"] = self.put(Path(self.command["ownedUpgradeHandoff"]["path"]), handoff)
        self.command.update(keeper=self.keeper, reasonCode="recovery_keeper_unproven", recoveryDigest=None)
        self.supervisor_args = [str(self.repo / ".jenkins/runtime/python/python.exe"), self.new_target["launcher"],
            "__lifecycle-host", "--spec", str(self.manager.spec.path), "--java", str(self.manager.java),
            "--war", str(self.manager.war), "--operation-id", self.runtime]
        self.supervisor = {**self.keeper, "pid": 30000, "creationTime": str(int(self.boot) + 400),
            "commandDigest": hashlib.sha256("\0".join(self.supervisor_args).encode()).hexdigest()}
        self.controller = {**self.identities["controller"], "pid": 30001, "creationTime": str(int(self.boot) + 410)}
        probe = {"associatedJob": {"memberPids": [self.keeper["pid"], self.supervisor["pid"]]}}
        self.command["createdSupervisor"] = {"creator": self.keeper, "child": self.supervisor,
            "creationFlags": 0x08000200, "requestedProfile": "zircon-strict", "creatorInJob": True, "childInJob": True,
            "actualRestrictionPreflight": probe, "associatedJobAtSpawn": probe}
        self.command_ref = self.put(self.paths["terminalCommand"], self.command)
        self.failed_operation = {**self.supervisor, "schemaVersion": 2, "component": "lifecycle-host",
            "hostPid": self.supervisor["pid"], "operationId": self.runtime, "generation": self.runtime,
            "homePath": str(self.home), "state": "failed", "reasonCode": "deployment_not_ready", "controlPlaneOnly": False,
            "ownedUpgradeDigest": self.command["ownedUpgradeDigest"], "previousOwnedStop": self.binding_ref}
        self.failed_host = {"operationId": self.runtime, "generation": self.runtime, "status": "failed",
            "reasonCode": "agent_node_migration_unproven", "hostPid": self.supervisor["pid"],
            "identity": self.controller, "controllerIdentity": self.controller, "agentLaunchAttempted": False,
            "agentTerminationProof": None, "nativeTerminationProof": self.proof(self.controller, requested=True)}
        self.failed_selected = {**self.new_target, "state": "selected-for-runtime-start", "runtimeOperationId": self.runtime,
            "recoveryRelease": self.successor_ref, "preparation": self.successor["preparation"]["path"],
            "supportRegression": self.successor["regression"], "runtimeAcceptance": False, "cargoAcceptance": False}
        self.put(self.paths["stoppedOperation"], self.failed_operation)
        self.put(self.paths["stoppedHost"], self.failed_host)
        self.put(self.paths["previousSelector"], self.failed_selected)
        originals = handoff["originalArtifacts"]
        self.transaction = {"schemaVersion": 1, "kind": "formal-recovery-start-transaction",
            "operationId": self.runtime, "state": "unknown", "spawnAttempted": True, "homePath": str(self.home),
            "keeper": self.keeper, "release": self.successor_ref, "request": self.command["request"],
            "createdPid": self.supervisor["pid"], "createdIdentity": self.supervisor,
            "operation": self.failed_operation, "selected": self.failed_selected,
            "selectedSha256": self.ref(self.paths["previousSelector"])["sha256"],
            "requestedCommand": self.supervisor_args, "requestedCommandDigest": self.supervisor["commandDigest"],
            "previousSelectorSha256": originals["previousSelector"]["archive"]["sha256"],
            **{field: json.loads(self.files[originals[name]["archive"]["path"]])
               for field, name in (("previousOperation", "stoppedOperation"), ("previousHost", "stoppedHost"),
                                   ("previousSelector", "previousSelector"))},
            **{key: False for key in ("automaticRetry", "runtimeAcceptance", "cargoAcceptance",
                                     "nativeTerminationProven", "historicalHoldsReleased")}}
        self.transaction_path = self.state / "recovery-lifecycle/start-transactions" / (self.runtime + ".json")
        self.put(self.transaction_path, self.transaction)
        self.retired_path = self.state / "recovery-lifecycle/history/failed-retired.json"
        self.retired = {"schemaVersion": 1, "kind": recovery.RETIREMENT_KIND, "keeper": self.keeper,
            "runtimeOperationId": self.runtime, "beforeIdentity": self.keeper, "beforeArguments": args,
            "handleIdentity": {key: self.keeper[key] for key in ("pid", "creationTime", "executable")},
            "nativeHandleRetained": True, "handleIdentityVerified": True, "method": "forced-terminate-held-handle",
            "terminateProcess": {"api": "TerminateProcess", "succeeded": True, "requestedExitCode": 198},
            "waitResult": 0, "processExitCode": 198, "endFileTime": str(int(self.boot) + 900),
            "afterDeparture": self.absent(self.keeper), "supervisorDeparture": self.absent(self.supervisor),
            "controllerDeparture": self.absent(self.controller), "protectedHolds": self.holds,
            "bootFileTime": self.boot, **{key: False for key in ("historicalHoldsReleased", "runtimeAcceptance",
                                                              "cargoAcceptance", "nativeTerminationProofSynthesized")}}
        self.refresh_retirement()

    def refresh_retirement(self):
        self.command_ref = self.put(self.paths["terminalCommand"], self.command)
        self.put(self.paths["stoppedOperation"], self.failed_operation)
        self.put(self.paths["stoppedHost"], self.failed_host)
        self.put(self.paths["previousSelector"], self.failed_selected)
        self.put(self.transaction_path, self.transaction)
        self.retired["originalRefs"] = {name: self.ref(path) for name, path in recovery._paths(self.state, self.runtime).items()}
        self.retired["serviceNativeTerminationProof"] = self.retired["originalRefs"]["host"]
        self.retired_ref = self.put(self.retired_path, self.retired)

    @contextmanager
    def observations(self):
        with super().observations() as stack:
            stack.enter_context(patch.object(recovery, "_ledger", side_effect=lambda state: self.ledger))
            stack.enter_context(patch.object(recovery, "windows_boot_filetime", side_effect=lambda: self.boot_probe.return_value))
            self.failed_inventory = stack.enter_context(patch.object(recovery, "runtime_owner_inventory", return_value=[]))
            stack.enter_context(patch.object(recovery, "process_lock", side_effect=lambda *a, **k: nullcontext()))
            def write_raw(path, raw, *, expected_sha=None):
                if self.ref(path)["sha256"] != expected_sha:
                    raise JenkinsError("cas_changed", "Original pointer changed")
                self.files[str(path)] = raw
            stack.enter_context(patch.object(recovery.keeper_control, "_write_raw", side_effect=write_raw))
            yield stack

    def prepare_recovery(self):
        return recovery.prepare_failed_upgrade_recovery(self.manager,
            expected_command_sha256=self.command_ref["sha256"], keeper_retirement_ref=self.retired_ref,
            replacement_request_id=self.replacement_id, replacement_target=self.retry_target)


class FailedUpgradeRecoveryTests(TestCase):
    def scenario(self):
        scenario = SpawnedFailure()
        with scenario.observations():
            scenario.setup_spawned_failure()
        return scenario

    def test_preparation_is_read_only_and_uses_real_native_proof_and_five_byte_archives(self):
        scenario = self.scenario()
        before = deepcopy(scenario.files)
        with scenario.observations():
            plan = scenario.prepare_recovery()
            self.assertIs(type(plan), recovery.FailedUpgradeRecovery)
            self.assertEqual(scenario.retired["originalRefs"], plan.document["eligibility"]["originalRefs"])
            self.assertEqual(scenario.request_id, plan.document["eligibility"]["predecessorRetirementRequestId"])
            self.assertEqual(scenario.holds, plan.document["eligibility"]["protectedHolds"])
            plan.revalidate(scenario.manager)
        self.assertEqual(before, scenario.files)

    def test_restore_archives_unknown_and_preserves_original_transaction_and_all_holds(self):
        scenario = self.scenario()
        before = deepcopy(scenario.files)
        with scenario.observations():
            result = scenario.prepare_recovery().restore(scenario.manager)
            receipt = recovery._receipt(scenario.state, result["reconciliation"])
            for name, binding in receipt["archivedEvidence"].items():
                self.assertEqual(before[binding["original"]["path"]], scenario.files[binding["archive"]["path"]])
            for name, relative in recovery._RESTORE.items():
                original = receipt["eligibility"]["predecessorArchives"][name]
                self.assertEqual(scenario.files[original["path"]], scenario.files[str(scenario.state / relative)])
            self.assertEqual("unknown", json.loads(scenario.files[receipt["archivedEvidence"]["command"]["archive"]["path"]])["status"])
            self.assertEqual(before[str(scenario.transaction_path)], scenario.files[str(scenario.transaction_path)])
            self.assertEqual(before[str(scenario.paths["stoppedDescriptor"])], scenario.files[str(scenario.paths["stoppedDescriptor"])])
            self.assertEqual(scenario.holds, receipt["eligibility"]["protectedHolds"])
            self.assertFalse(receipt["nativeProofSynthesized"])
            self.assertFalse(receipt["historicalHoldsReleased"])

    def test_absence_never_substitutes_for_eof_or_an_agent_that_was_started(self):
        for key, change in (("stdoutEof", False), ("stderrEof", False), ("activeProcesses", 1),
                            ("childrenGone", False), ("complete", False), ("identity", {}), ("processExitCode", 259)):
            with self.subTest(key=key):
                scenario = self.scenario()
                scenario.failed_host["nativeTerminationProof"][key] = change
                scenario.refresh_retirement()
                with scenario.observations(), self.assertRaises(JenkinsError):
                    scenario.prepare_recovery()
        for key, value in (("agentLaunchAttempted", True), ("agentIdentity", {"pid": 8}),
                           ("agentTerminationProof", {}), ("status", "starting")):
            with self.subTest(key=key):
                scenario = self.scenario()
                scenario.failed_host[key] = value
                scenario.refresh_retirement()
                with scenario.observations(), self.assertRaises(JenkinsError):
                    scenario.prepare_recovery()

    def test_positive_creation_and_exact_transaction_are_required(self):
        for change in ("no-created-child", "wrong-child-birth", "not-strict", "not-attempted", "wrong-command",
                       "wrong-previous", "transaction-native-claim"):
            with self.subTest(change=change):
                scenario = self.scenario()
                if change == "no-created-child":
                    scenario.command.pop("createdSupervisor")
                elif change == "wrong-child-birth":
                    scenario.command["createdSupervisor"]["child"] = {**scenario.supervisor, "creationTime": "1"}
                elif change == "not-strict":
                    scenario.command["createdSupervisor"]["childInJob"] = False
                elif change == "not-attempted":
                    scenario.transaction["spawnAttempted"] = False
                elif change == "wrong-command":
                    scenario.transaction["requestedCommand"] = ["another launcher"]
                elif change == "wrong-previous":
                    scenario.transaction["previousOperation"] = {"state": "stopped"}
                else:
                    scenario.transaction["nativeTerminationProven"] = True
                scenario.refresh_retirement()
                with scenario.observations(), self.assertRaises(JenkinsError):
                    scenario.prepare_recovery()

    def test_retirement_must_be_actual_retained_handle_exit_and_never_service_eof(self):
        for key, value in (("nativeHandleRetained", False), ("handleIdentityVerified", False),
                           ("waitResult", 258), ("processExitCode", 259), ("endFileTime", "1"),
                           ("stdoutEof", True), ("nativeTerminationProofSynthesized", True),
                           ("serviceNativeTerminationProof", {"path": "other", "sha256": "a" * 64})):
            with self.subTest(key=key):
                scenario = self.scenario()
                scenario.retired[key] = value
                scenario.retired_ref = scenario.put(scenario.retired_path, scenario.retired)
                with scenario.observations(), self.assertRaises(JenkinsError):
                    scenario.prepare_recovery()

    def test_live_original_birth_foreign_runtime_and_invariant_drift_block_restoration(self):
        for change in ("live-birth", "unknown-departure", "foreign-runtime", "hold", "ledger", "home", "boot", "cas"):
            with self.subTest(change=change):
                scenario = self.scenario()
                with scenario.observations():
                    plan = scenario.prepare_recovery()
                    if change in ("live-birth", "unknown-departure"):
                        scenario.departure.side_effect = JenkinsError("owner_unproven", "Not proven departed")
                    elif change == "foreign-runtime":
                        scenario.failed_inventory.return_value = [{"pid": 77}]
                    elif change == "hold":
                        scenario.holds[0]["version"] += 1
                    elif change == "ledger":
                        scenario.ledger = "changed ledger"
                    elif change == "home":
                        scenario.home_probe.return_value["exclusiveReadProbes"][0]["modifiedNs"] += 1
                    elif change == "boot":
                        scenario.boot_probe.return_value += 1
                    else:
                        scenario.files[str(scenario.paths["terminalCommand"])] += b" "
                    before = deepcopy(scenario.files)
                    with self.assertRaises(JenkinsError):
                        plan.restore(scenario.manager)
                    self.assertEqual(before, scenario.files)

    def test_original_handoff_archive_byte_drift_is_rejected(self):
        scenario = self.scenario()
        handoff = json.loads(scenario.files[scenario.command["ownedUpgradeHandoff"]["path"]])
        scenario.files[handoff["originalArtifacts"]["stoppedOperation"]["archive"]["path"]] += b" "
        with scenario.observations(), self.assertRaises(JenkinsError):
            scenario.prepare_recovery()

    def test_same_target_or_request_and_untyped_capability_cannot_retry(self):
        scenario = self.scenario()
        with scenario.observations():
            for request_id, target in ((scenario.request_id, scenario.retry_target),
                                      (scenario.replacement_id, scenario.new_target)):
                with self.subTest(request=request_id, target=target["driverDigest"]), self.assertRaises(JenkinsError):
                    recovery.prepare_failed_upgrade_recovery(scenario.manager,
                        expected_command_sha256=scenario.command_ref["sha256"], keeper_retirement_ref=scenario.retired_ref,
                        replacement_request_id=request_id, replacement_target=target)
            with self.assertRaises(JenkinsError):
                recovery.FailedUpgradeRecovery(canonical_json({}))
            plan = scenario.prepare_recovery()
            plan.restore(scenario.manager)
            with self.assertRaises(JenkinsError):
                plan.restore(scenario.manager)

    def test_gate_reserves_one_exact_request_target_and_release(self):
        scenario = self.scenario()
        with scenario.observations():
            result = scenario.prepare_recovery().restore(scenario.manager)
            candidate = {**scenario.successor, "requestId": scenario.replacement_id, "targetDriver": scenario.retry_target,
                         "failedStartReconciliation": result["reconciliation"], "runtimeRoleReview": None}
            reference = scenario.put(scenario.state / "deployment/retry-release.json", candidate)
            kwargs = {"request_id": scenario.replacement_id,
                "descriptor_sha256": scenario.ref(scenario.paths["stoppedDescriptor"])["sha256"],
                "command_sha256": result["restoredCommand"]["sha256"]}
            recovery.validate_failed_upgrade_restoration_gate(scenario.manager, reference, candidate, **kwargs)
            for field, value in (("requestId", "f" * 32), ("targetDriver", scenario.new_target),
                                 ("failedStartReconciliation", None)):
                with self.subTest(field=field), self.assertRaises(JenkinsError):
                    recovery.validate_failed_upgrade_restoration_gate(scenario.manager, reference,
                        {**candidate, field: value}, **{**kwargs, "request_id": value if field == "requestId" else kwargs["request_id"]})
            gate_path = scenario.state / recovery._GATE
            gate = json.loads(scenario.files[str(gate_path)])
            gate.update(status="claimed", claimedRelease={"path": "other", "sha256": "a" * 64})
            scenario.put(gate_path, gate)
            with self.assertRaises(JenkinsError):
                recovery.validate_failed_upgrade_restoration_gate(scenario.manager, reference, candidate, **kwargs)

    def test_interrupted_multi_pointer_cas_stays_blocked_with_original_archives(self):
        scenario = self.scenario()
        with scenario.observations():
            plan = scenario.prepare_recovery()
            write = recovery.keeper_control._write_raw.side_effect
            calls = []
            def fail_second(path, raw, **kwargs):
                calls.append(path)
                if len(calls) == 2:
                    raise JenkinsError("cas_unknown", "The second pointer was not changed")
                return write(path, raw, **kwargs)
            with patch.object(recovery.keeper_control, "_write_raw", side_effect=fail_second), self.assertRaises(JenkinsError):
                plan.restore(scenario.manager)
            gate = json.loads(scenario.files[str(scenario.state / recovery._GATE)])
            self.assertEqual("restoring", gate["status"])
            self.assertEqual("unknown", json.loads(scenario.files[str(scenario.paths["terminalCommand"])])["status"])
            intent = json.loads(scenario.files[gate["intent"]["path"]])
            self.assertEqual(set(recovery._paths(scenario.state, scenario.runtime)),
                             set(intent["archivedEvidence"]) & set(recovery._paths(scenario.state, scenario.runtime)))
            with self.assertRaises(JenkinsError):
                recovery.validate_failed_upgrade_restoration_gate(scenario.manager, {}, {},
                    request_id=scenario.replacement_id, descriptor_sha256="a" * 64, command_sha256="b" * 64)

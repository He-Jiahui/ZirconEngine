"""A blocked keeper never receives credit for the manager's actual NativeStop."""
from __future__ import annotations

from copy import deepcopy
from contextlib import nullcontext
from types import SimpleNamespace
import fnmatch
import hashlib
import json
from pathlib import Path
import unittest
from unittest.mock import Mock, patch

from tools.jenkins.contracts import JenkinsError, canonical_json, digest
from tools.jenkins.deployment import owned_upgrade as upgrade, recovery_keeper, lost_recovery_runtime, unknown_stop_recovery
from tools.jenkins.tests.test_owned_recovery_upgrade import StoppedOwner


class ExternalStoppedOwner(StoppedOwner):
    def __init__(self, **changes):
        super().__init__(**changes)
        old_stop_id = self.stop_id
        for area in ("requests", "results"):
            self.files.pop(str(self.state / "recovery-lifecycle" / area / (old_stop_id + ".json")))
        origin_path = Path(self.origin_ref["path"])
        origin = json.loads(self.files[str(origin_path)])
        request = self.put(self.state / "recovery-lifecycle/history/initial-recovery-request.json", {
            "schemaVersion": 1, "kind": upgrade._REQUEST_KIND, "requestId": self.old_release["requestId"],
            "action": "recover", "keeper": self.identities["keeper"], "release": self.old_release_ref,
            "runtimeAcceptance": False})
        origin.update(request=request, previousRuntimeReceipt=None, protectedHolds=self.holds,
            initialRecovery=None, recoveryDigest=None, failedOwnStartInterruption=None,
            **{name: False for name in upgrade._FLAGS})
        self.origin_ref = self.put(origin_path, origin)
        self.descriptor.update(admission="blocked", runtime=origin["runtime"], runtimeReceipt=self.origin_ref)
        self.put(self.paths["stoppedDescriptor"], self.descriptor)
        self.stopped_descriptor_ref = self.copy(self.paths["stoppedDescriptor"],
            self.state / "recovery-lifecycle/history/original-blocked-descriptor.json")
        binding = json.loads(self.files[self.binding_ref["path"]])
        binding["capturedOwner"]["liveIdentities"] = [self.identities[name] for name in ("supervisor", "controller", "agent")]
        self.binding_ref = self.put(Path(self.binding_ref["path"]), binding)
        self.operation["stopBinding"] = self.binding_ref
        self.put(self.paths["stoppedOperation"], self.operation)
        result = self.put(self.state / "recovery-lifecycle/history/initial-running-result.json", {
            "schemaVersion": 1, "kind": "recovery-keeper-command-result", "status": "running-acceptance-pending",
            "keeper": self.identities["keeper"], "release": self.old_release_ref, "request": request,
            "runtimeReceipt": self.origin_ref, "operationId": self.runtime,
            "runtimeAcceptance": False, "cargoAcceptance": False, "historicalHoldsReleased": False})
        self.terminal_command.update(status="committed", request=request, previousRuntimeReceipt=None, result=result)
        self.put(self.paths["terminalCommand"], self.terminal_command)
        self.committed_ref = self.copy(self.paths["terminalCommand"],
            self.state / "recovery-lifecycle/history/original-committed-command.json")
        self.put(self.state / "recovery-lifecycle/results" / (self.old_release["requestId"] + ".json"), result)
        self.retirement.pop("stoppedDescriptor")
        self.retirement.pop("ownedStop")
        self.retirement.update(variant=upgrade.EXTERNAL_VARIANT, originalBlockedDescriptor=self.stopped_descriptor_ref,
            externalNativeStop=self.binding_ref)
        self.retirement_ref = self.put(self.retirement_path, self.retirement)
        for name in ("deployment-operation.json", "deployment/host.json", "deployment/driver.json"):
            self.fence[name] = self.ref(self.state / name)["sha256"]
        self.successor.update(ownedUpgrade=self.marker, expectedLifecycleFence=deepcopy(self.fence))
        self.successor_ref = self.put(self.successor_path, self.successor)

    @property
    def marker(self):
        if not hasattr(self, "committed_ref"):
            return super().marker
        return {"variant": upgrade.EXTERNAL_VARIANT, "originalBlockedDescriptor": self.stopped_descriptor_ref,
            "committedCommand": self.committed_ref, "externalNativeStop": self.binding_ref,
            "keeperRetirement": self.retirement_ref}

    def prepare(self):
        return upgrade.prepare_external_native_stop_upgrade_start(self.manager, request_id=self.request_id,
            successor_release_ref=self.successor_ref, original_blocked_descriptor_ref=self.stopped_descriptor_ref,
            committed_command_ref=self.committed_ref, external_native_stop_ref=self.binding_ref,
            keeper_retirement_ref=self.retirement_ref)

    def observations(self):
        stack = super().observations()
        stack.enter_context(patch.object(lost_recovery_runtime, "_raw", side_effect=lambda path: self.files[str(Path(path).absolute())]))
        stack.enter_context(patch.object(lost_recovery_runtime, "_plain"))
        stack.enter_context(patch.object(Path, "glob", autospec=True, side_effect=lambda directory, pattern:
            iter(Path(name) for name in self.files if Path(name).parent == directory
                 and fnmatch.fnmatch(Path(name).name, pattern))))
        return stack

    def replacement(self, plan):
        archives, originals = {}, {}
        variant = plan.document["variant"]
        _, command_key, _ = upgrade._transfer_keys(plan.document)
        replacement_field = "replacesUnknownStopCommand" if variant == upgrade.UNKNOWN_EXTERNAL_VARIANT else "replacesCommittedCommand"
        for name, evidence in plan.original_artifacts.items():
            target = self.state / "recovery-lifecycle/history" / ("external-handoff-" + name + ".json")
            self.files[str(target)] = evidence["raw"]
            archives[name] = self.ref(target)
            originals[name] = {"original": evidence["reference"], "archive": archives[name]}
        keeper = {**self.identities["keeper"], "pid": 12345, "creationTime": str(int(self.boot) + 300)}
        self.self_identity.return_value = keeper
        request = self.put(self.state / "recovery-lifecycle/history/external-upgrade-request.json", {
            "schemaVersion": 1, "kind": upgrade._REQUEST_KIND, "requestId": self.request_id, "action": "upgrade",
            "keeper": keeper, "release": self.successor_ref, "ownedUpgradeDigest": plan.upgrade_digest,
            "protectedHoldsDigest": digest(self.holds), "runtimeAcceptance": False})
        handoff_body = {"schemaVersion": 1, "kind": upgrade.HANDOFF_KIND, "outcome": "owned-transfer",
            "variant": variant, "keeper": keeper, "release": self.successor_ref,
            "request": request, "ownedUpgradeDigest": plan.upgrade_digest,
            "previousKeeper": self.identities["keeper"], "previousRelease": self.old_release_ref,
            "originalBlockedDescriptor": archives["originalBlockedDescriptor"],
            command_key: archives[command_key], "externalNativeStop": self.binding_ref,
            "keeperRetirement": self.retirement_ref, "originalArtifacts": originals,
            **{name: False for name in (*upgrade._FLAGS, "queueReplayRequested", "automaticRetry")}}
        if plan.document.get("unknownStopRecovery") is not None:
            handoff_body["unknownStopRecovery"] = plan.document["unknownStopRecovery"]
        handoff = self.put(self.state / "recovery-lifecycle/history/external-upgrade-handoff.json", handoff_body)
        arguments = {"keeper": keeper, "release_ref": self.successor_ref, "request": request,
            "descriptor_archive": archives["originalBlockedDescriptor"], "command_archive": archives[command_key],
            "handoff_ref": handoff}
        command = {"schemaVersion": 1, "kind": upgrade._COMMAND_KIND, "status": "pending",
            "variant": variant, "keeper": keeper, "release": self.successor_ref,
            "request": request, "ownedUpgradeDigest": plan.upgrade_digest, "externalNativeStop": self.binding_ref,
            "replacesOriginalBlockedDescriptor": archives["originalBlockedDescriptor"],
            replacement_field: archives[command_key], "ownedUpgradeHandoff": handoff,
            "previousRuntimeReceipt": None, "runtimeAcceptance": False, "historicalHoldsReleased": False}
        if plan.document.get("unknownStopRecovery") is not None:
            command["unknownStopRecovery"] = plan.document["unknownStopRecovery"]
        return arguments, command


class ExternalNativeStopUpgradeTests(unittest.TestCase):
    def test_real_external_stop_issues_read_only_capability_and_keeps_blocked_committed_bytes(self):
        scenario = ExternalStoppedOwner()
        before = deepcopy(scenario.files)
        with scenario.observations(), patch.object(recovery_keeper, "observe_live_identity", side_effect=AssertionError("departed is not live")):
            plan = scenario.prepare()
            self.assertEqual(upgrade.EXTERNAL_VARIANT, plan.document["variant"])
            self.assertEqual(scenario.binding_ref, plan.document["externalNativeStop"])
            self.assertEqual("owned-stop-metadata-binding", json.loads(scenario.files[plan.document["externalNativeStop"]["path"]])["kind"])
            self.assertEqual("blocked", json.loads(plan.original_artifacts["originalBlockedDescriptor"]["raw"])["admission"])
            self.assertEqual("running", json.loads(plan.original_artifacts["originalBlockedDescriptor"]["raw"])["runtime"]["state"])
            self.assertEqual("committed", json.loads(plan.original_artifacts["committedCommand"]["raw"])["status"])
            plan.revalidate(scenario.manager)
        self.assertEqual(before, scenario.files)

    def test_ordinary_factory_does_not_accept_external_blocked_descriptor(self):
        scenario = ExternalStoppedOwner()
        with scenario.observations(), self.assertRaises(JenkinsError):
            upgrade.prepare_owned_upgrade_start(scenario.manager, request_id=scenario.request_id,
                successor_release_ref=scenario.successor_ref, stopped_descriptor_ref=scenario.stopped_descriptor_ref,
                owned_stop_ref=scenario.binding_ref, keeper_retirement_ref=scenario.retirement_ref)

    def test_complete_native_eof_and_actual_strict_spawn_are_required_even_when_pids_are_absent(self):
        for parameter, change in (("controller_proof", {"stdoutEof": False}), ("agent_proof", {"stderrEof": False}),
                ("controller_proof", {"childrenGone": False}), ("agent_proof", {"identity": {"pid": 1}}),
                ("strict_spawn", {"childInJob": False}), ("strict_spawn", {"creationFlags": 0x01000000})):
            with self.subTest(parameter=parameter, change=change):
                scenario = ExternalStoppedOwner(**{parameter: change})
                with scenario.observations(), self.assertRaises(JenkinsError):
                    scenario.prepare()

    def test_external_retirement_requires_real_retained_handle_wait_and_exact_scope(self):
        for change in ({"nativeHandleRetained": False}, {"waitResult": 258}, {"processExitCode": 259},
                {"serviceStoppedBeforeRetirement": False}, {"handleIdentity": {}}, {"stdoutEof": True}):
            with self.subTest(change=change):
                scenario = ExternalStoppedOwner(retirement=change)
                with scenario.observations(), self.assertRaises(JenkinsError):
                    scenario.prepare()

    def test_descriptor_command_birth_and_live_capture_drift_fail_before_capability(self):
        for change in ("descriptor", "command", "captured-birth", "capture-observation", "unknown-command"):
            with self.subTest(change=change):
                scenario = ExternalStoppedOwner()
                if change in {"descriptor", "command"}:
                    scenario.files[str(scenario.paths["stoppedDescriptor" if change == "descriptor" else "terminalCommand"])] += b" "
                elif change == "unknown-command":
                    scenario.put(scenario.paths["terminalCommand"], {**scenario.terminal_command, "status": "unknown"})
                else:
                    value = json.loads(scenario.files[scenario.binding_ref["path"]])
                    if change == "captured-birth":
                        value["capturedOwner"]["operation"]["creationTime"] = "1"
                    else:
                        value["capturedOwner"]["liveIdentities"][0]["commandDigest"] = "f" * 64
                    scenario.binding_ref = scenario.put(Path(scenario.binding_ref["path"]), value)
                with scenario.observations(), self.assertRaises(JenkinsError):
                    scenario.prepare()

    def test_retained_old_keeper_or_new_owner_historical_hold_and_cas_changes_are_rejected(self):
        for change in ("keeper-live", "foreign-runtime", "hold", "command-cas"):
            with self.subTest(change=change):
                scenario = ExternalStoppedOwner()
                with scenario.observations():
                    plan = scenario.prepare()
                    if change == "keeper-live":
                        original = scenario.departure.side_effect
                        def departed(value):
                            if value == scenario.identities["keeper"]:
                                raise JenkinsError("deployment_owner_live", "Keeper remains live")
                            return original(value)
                        scenario.departure.side_effect = departed
                    elif change == "foreign-runtime":
                        scenario.inventory.return_value = [{"pid": 77}]
                    elif change == "hold":
                        scenario.holds[0]["version"] += 1
                    else:
                        scenario.files[str(scenario.paths["terminalCommand"])] += b" "
                    with self.assertRaises(JenkinsError):
                        plan.revalidate(scenario.manager)

    def test_one_explicit_handoff_revalidates_after_command_cas_and_recursively_checks_original_runtime(self):
        scenario = ExternalStoppedOwner()
        with scenario.observations():
            plan = scenario.prepare()
            arguments, command = scenario.replacement(plan)
            bound = plan.authorize_keeper_command(scenario.manager, **arguments)
            with self.assertRaises(JenkinsError):
                bound.revalidate(scenario.manager)
            scenario.put(scenario.paths["terminalCommand"], command)
            self.assertEqual(plan.document, bound.document)
            bound.revalidate(scenario.manager)
            plan_ref = scenario.put(scenario.state / "recovery-lifecycle/history/external-initial-plan.json", plan.document)
            descriptor = {"keeper": arguments["keeper"], "release": scenario.successor_ref,
                "initialOwnedUpgrade": plan_ref, "ownedUpgradeDigest": plan.upgrade_digest,
                "ownedUpgradeHandoff": arguments["handoff_ref"], "previousStoppedRuntimeReceipt": scenario.binding_ref}
            result = recovery_keeper._owned_handoff_receipt(scenario.manager, descriptor, scenario.successor)
            self.assertEqual(upgrade.EXTERNAL_VARIANT, result["variant"])
            before = deepcopy(scenario.files)
            altered = {**command, "externalNativeStop": scenario.origin_ref}
            scenario.put(scenario.paths["terminalCommand"], altered)
            with self.assertRaises(JenkinsError):
                bound.revalidate(scenario.manager)
            self.assertEqual(before[scenario.stopped_descriptor_ref["path"]], scenario.files[scenario.stopped_descriptor_ref["path"]])

    def test_unresolved_foreign_request_is_preserved_and_blocks_external_transfer(self):
        scenario = ExternalStoppedOwner()
        request_id = "3" * 32
        reference = scenario.put(scenario.state / "recovery-lifecycle/history/foreign-pending-request.json", {
            "schemaVersion": 1, "kind": recovery_keeper.REQUEST_KIND, "requestId": request_id, "action": "stop",
            "keeper": scenario.identities["keeper"], "release": scenario.old_release_ref,
            "runtimeReceipt": scenario.origin_ref, "runtimeAcceptance": False})
        scenario.put(scenario.state / "recovery-lifecycle/requests" / (request_id + ".json"), reference)
        before = deepcopy(scenario.files)
        with scenario.observations(), self.assertRaises(JenkinsError):
            scenario.prepare()
        self.assertEqual(before, scenario.files)

    def test_retained_actor_archives_all_external_anchors_before_a_single_command_cas(self):
        scenario = ExternalStoppedOwner()
        with scenario.observations():
            plan = scenario.prepare()
            arguments, _ = scenario.replacement(plan)
            actor = object.__new__(recovery_keeper.RecoveryKeeper)
            actor.manager, actor.repo, actor.state = scenario.manager, scenario.manager.paths.repo, scenario.state
            actor.release, actor.release_ref, actor.target = scenario.successor, scenario.successor_ref, scenario.targets["b" * 64]
            actor.keeper = arguments["keeper"]
            actor.command_path = scenario.paths["terminalCommand"]
            actor.descriptor = scenario.paths["stoppedDescriptor"]
            actor.runtime, actor.runtime_receipt, actor.blocked = None, None, False
            actor.original_interruption, actor.runtime_interruption, actor.owned_handoff = None, None, None
            actor.initial_digest, actor.upgrade_digest, actor.upgrade_plan = None, plan.upgrade_digest, plan
            actor._fresh = Mock()
            before_descriptor = scenario.files[str(actor.descriptor)]
            writes = []
            def archive(state, stem, raw):
                target = state / "recovery-lifecycle/history" / (stem + "-" + hashlib.sha256(raw).hexdigest() + ".json")
                if str(target) in scenario.files:
                    self.assertEqual(raw, scenario.files[str(target)])
                scenario.files[str(target)] = raw
                return scenario.ref(target)
            def write(path, value, *, expected_sha=None):
                self.assertEqual(expected_sha, scenario.ref(path)["sha256"])
                self.assertIsNotNone(actor.owned_handoff)
                handoff = recovery_keeper._reference(scenario.state, actor.owned_handoff["receipt"])
                self.assertEqual(set(plan.original_artifacts), set(handoff["originalArtifacts"]))
                writes.append(path)
                scenario.put(path, value)
            with patch.object(recovery_keeper, "_archive_bytes", side_effect=archive), patch.object(recovery_keeper, "_write", side_effect=write):
                bound = actor._begin(arguments["request"], plan=plan)
            command = json.loads(scenario.files[str(actor.command_path)])
            self.assertEqual([actor.command_path], writes)
            self.assertEqual("pending", command["status"])
            self.assertEqual(upgrade.EXTERNAL_VARIANT, command["variant"])
            self.assertNotIn("replacesStoppedDescriptor", command)
            self.assertNotIn("replacesTerminalCommand", command)
            self.assertEqual(before_descriptor, scenario.files[str(actor.descriptor)])
            bound.revalidate(scenario.manager)


if __name__ == "__main__":
    unittest.main()


class UnknownStoppedOwner(ExternalStoppedOwner):
    def __init__(self, **changes):
        super().__init__(**changes)
        self.unknown_id = "d" * 32
        idle = {**self.descriptor, "admission": "idle"}
        request = self.put(self.state / "recovery-lifecycle/history/unknown-request.json", {
            "schemaVersion": 1, "kind": upgrade._REQUEST_KIND, "requestId": self.unknown_id, "action": "stop",
            "keeper": self.identities["keeper"], "release": self.old_release_ref,
            "runtimeReceipt": self.origin_ref, "protectedHoldsDigest": digest(self.holds),
            "descriptorSha256": hashlib.sha256(canonical_json(idle) + b"\n").hexdigest(), "runtimeAcceptance": False})
        self.put(self.state / "recovery-lifecycle/requests" / (self.unknown_id + ".json"), request)
        self.terminal_command.update(status="unknown", request=request, previousRuntimeReceipt=self.origin_ref,
                                     reasonCode="controller_unreachable")
        self.terminal_command.pop("result")
        self.put(self.paths["terminalCommand"], self.terminal_command)
        self.unknown_ref = self.copy(self.paths["terminalCommand"],
                                    self.state / "recovery-lifecycle/history/original-unknown-command.json")
        self.unknown_rows = [{"key": str(n), "version": 1, "rawPayloadSha256": str(n) * 64} for n in (1, 2, 3)]
        pref = {"schemaVersion": 1, "kind": "unknown-stop-native-preflight",
                "pins": {"requestId": "e" * 32, "commandSha256": self.unknown_ref["sha256"],
                         "descriptorSha256": self.stopped_descriptor_ref["sha256"]},
                "originalRequest": request, "bootFileTime": self.boot,
                "anchors": {"unknownStopCommand": self.unknown_ref,
                            "originalBlockedDescriptor": self.stopped_descriptor_ref},
                "unknownNativeRows": self.unknown_rows}
        attempt = self.put(self.state / "deployment/history" / ("unknown-stop-native-attempt-" + request["sha256"] + ".json"),
                           {"kind": "unknown-stop-native-attempt", "automaticRetry": False,
                            "preflight": pref, "preflightDigest": digest(pref)})
        self.recovery_ref = self.put(self.state / "recovery-lifecycle/history/unknown-native-recovery.json", {
            "kind": unknown_stop_recovery.KIND, "variant": unknown_stop_recovery.VARIANT, "status": "stopped",
            "requestId": "e" * 32, "attempt": attempt, "originalRequest": request,
            "originalBlockedDescriptor": self.stopped_descriptor_ref, "unknownStopCommand": self.unknown_ref,
            "externalNativeStop": self.binding_ref, "originalStopStatus": "unknown",
            "originalResultCreated": False, "automaticRetry": False, **{name: False for name in upgrade._FLAGS}})
        self.retirement.update(variant=unknown_stop_recovery.VARIANT, unknownStopRecovery=self.recovery_ref)
        self.retirement_ref = self.put(self.retirement_path, self.retirement)
        self.successor["ownedUpgrade"] = self.marker
        self.successor_ref = self.put(self.successor_path, self.successor)

    @property
    def marker(self):
        if not hasattr(self, "recovery_ref"):
            return super().marker
        return {"variant": unknown_stop_recovery.VARIANT, "originalBlockedDescriptor": self.stopped_descriptor_ref,
                "unknownStopCommand": self.unknown_ref, "externalNativeStop": self.binding_ref,
                "unknownStopRecovery": self.recovery_ref, "keeperRetirement": self.retirement_ref}

    def observations(self):
        stack = super().observations()
        stack.enter_context(patch.object(unknown_stop_recovery, "_unknown_native_rows", return_value=self.unknown_rows))
        stack.enter_context(patch.object(unknown_stop_recovery, "protected_holds", return_value=self.holds))
        return stack

    def prepare(self):
        return upgrade.prepare_owned_upgrade_from_release(self.manager, self.successor_ref, self.successor)


class UnknownStopNativeUpgradeTests(unittest.TestCase):
    def test_public_dispatch_transfers_real_native_proofs_but_keeps_old_stop_unknown(self):
        scenario = UnknownStoppedOwner()
        before = deepcopy(scenario.files)
        with scenario.observations():
            plan = scenario.prepare()
            plan.revalidate(scenario.manager)
            self.assertEqual(unknown_stop_recovery.VARIANT, plan.document["variant"])
            self.assertEqual("unknown", json.loads(plan.original_artifacts["unknownStopCommand"]["raw"])["status"])
            self.assertEqual(scenario.recovery_ref, plan.document["unknownStopRecovery"])
            recovery_keeper._owned_upgrade_marker(scenario.state, scenario.successor)
            self.assertFalse((scenario.state / "recovery-lifecycle/results" / (scenario.unknown_id + ".json")).exists())
        self.assertEqual(before, scenario.files)
        for role in ("controller_proof", "agent_proof"):
            bad = UnknownStoppedOwner(**{role: {"stdoutEof": False}})
            with bad.observations(), self.assertRaises(JenkinsError):
                bad.prepare()

    def test_wrong_attempt_namespace_or_original_pins_cannot_authorize_transfer(self):
        for corruption in ("attempt-namespace", "command-pin"):
            scenario = UnknownStoppedOwner()
            outcome = json.loads(scenario.files[scenario.recovery_ref["path"]])
            attempt = json.loads(scenario.files[outcome["attempt"]["path"]])
            if corruption == "attempt-namespace":
                outcome["attempt"] = scenario.put(scenario.state / "recovery-lifecycle/history" /
                    Path(outcome["attempt"]["path"]).name, attempt)
            else:
                attempt["preflight"]["pins"]["commandSha256"] = "0" * 64
                attempt["preflightDigest"] = digest(attempt["preflight"])
                outcome["attempt"] = scenario.put(Path(outcome["attempt"]["path"]), attempt)
            scenario.recovery_ref = scenario.put(Path(scenario.recovery_ref["path"]), outcome)
            scenario.retirement["unknownStopRecovery"] = scenario.recovery_ref
            scenario.retirement_ref = scenario.put(scenario.retirement_path, scenario.retirement)
            scenario.successor["ownedUpgrade"] = scenario.marker
            scenario.successor_ref = scenario.put(scenario.successor_path, scenario.successor)
            with scenario.observations(), self.assertRaises(JenkinsError):
                scenario.prepare()


class UnknownStopRecoveryDeliveryAndAttemptTests(unittest.TestCase):
    def test_completed_external_stops_cross_a_later_boot_without_rewriting_the_predecessor(self):
        for factory in (ExternalStoppedOwner, UnknownStoppedOwner):
            with self.subTest(factory=factory.__name__):
                scenario = factory()
                previous_boot = scenario.boot
                scenario.boot = str(int(previous_boot) + 1000)
                before = deepcopy(scenario.files)
                with scenario.observations():
                    plan = scenario.prepare()
                    plan.revalidate(scenario.manager)
                    self.assertEqual(previous_boot, plan.document["previousBootFileTime"])
                    self.assertEqual(scenario.boot, plan.document["bootFileTime"])
                    self.assertFalse(plan.document["nativeProofIssued"])
                    self.assertFalse(plan.document["runtimeAcceptance"])
                self.assertEqual(before, scenario.files)

    def test_later_boot_keeps_exact_retirement_and_complete_native_stop_requirements(self):
        for change in ({"controller_proof": {"stdoutEof": False}},
                       {"agent_proof": {"complete": False}},
                       {"retirement": {"waitResult": 258}},
                       {"retirement": {"bootFileTime": "134358454705001000"}}):
            with self.subTest(change=change):
                scenario = UnknownStoppedOwner(**change)
                scenario.boot = str(int(scenario.boot) + 1000)
                with scenario.observations(), self.assertRaises(JenkinsError):
                    scenario.prepare()

    def test_later_boot_does_not_adopt_future_births_or_ordinary_stopped_owners(self):
        for factory in (UnknownStoppedOwner, StoppedOwner):
            scenario = factory()
            latest_birth = max(int(value["creationTime"]) for value in scenario.identities.values())
            scenario.boot = str(latest_birth)
            with scenario.observations(), self.assertRaises(JenkinsError):
                scenario.prepare()
        for boot in ("0", "invalid", "134358454704999999"):
            scenario = UnknownStoppedOwner()
            with scenario.observations(), self.assertRaises((JenkinsError, ValueError)):
                scenario.boot_probe.return_value = boot
                scenario.prepare()

    def test_later_boot_handoff_preserves_unknown_stop_and_rechecks_the_boot_boundary(self):
        scenario = UnknownStoppedOwner()
        previous_boot = scenario.boot
        scenario.boot = str(int(previous_boot) + 1000)
        with scenario.observations():
            plan = scenario.prepare()
            arguments, command = scenario.replacement(plan)
            bound = plan.authorize_keeper_command(scenario.manager, **arguments)
            scenario.put(scenario.paths["terminalCommand"], command)
            bound.revalidate(scenario.manager)
            planned = scenario.put(scenario.state / "recovery-lifecycle/history/later-boot-plan.json", plan.document)
            descriptor = {"keeper": arguments["keeper"], "release": scenario.successor_ref,
                "initialOwnedUpgrade": planned, "ownedUpgradeDigest": plan.upgrade_digest,
                "ownedUpgradeHandoff": arguments["handoff_ref"], "previousStoppedRuntimeReceipt": scenario.binding_ref}
            recovery_keeper._owned_handoff_receipt(scenario.manager, descriptor, scenario.successor)
            self.assertEqual("unknown", json.loads(plan.original_artifacts["unknownStopCommand"]["raw"])["status"])
            self.assertEqual(previous_boot, json.loads(scenario.files[scenario.retirement_ref["path"]])["bootFileTime"])
            self.assertFalse((scenario.state / "recovery-lifecycle/results" / (scenario.unknown_id + ".json")).exists())
            scenario.boot_probe.return_value = int(scenario.boot) + 1
            with self.assertRaises(JenkinsError):
                bound.revalidate(scenario.manager)

    def test_successor_public_pending_enumeration_recognizes_only_original_unknown_without_result(self):
        scenario = UnknownStoppedOwner()
        with scenario.observations():
            plan = scenario.prepare()
            arguments, command = scenario.replacement(plan)
            bound = plan.authorize_keeper_command(scenario.manager, **arguments)
            scenario.put(scenario.paths["terminalCommand"], command)
            bound.revalidate(scenario.manager)
            planned = scenario.put(scenario.state / "recovery-lifecycle/history/unknown-initial-plan.json", plan.document)
            descriptor = {"keeper": arguments["keeper"], "release": scenario.successor_ref,
                "initialOwnedUpgrade": planned, "ownedUpgradeDigest": plan.upgrade_digest,
                "ownedUpgradeHandoff": arguments["handoff_ref"], "previousStoppedRuntimeReceipt": scenario.binding_ref}
            self.assertEqual(unknown_stop_recovery.VARIANT,
                recovery_keeper._owned_handoff_receipt(scenario.manager, descriptor, scenario.successor)["variant"])
            scenario.put(scenario.paths["stoppedDescriptor"], descriptor)
            result = scenario.put(scenario.state / "recovery-lifecycle/history/successor-running-result.json", {
                "kind": "recovery-keeper-command-result", "request": arguments["request"],
                "keeper": arguments["keeper"], "release": scenario.successor_ref,
                "status": "running-acceptance-pending", "runtimeAcceptance": False})
            scenario.put(scenario.paths["terminalCommand"], {**command, "status": "committed", "result": result})
            directory = scenario.state / "recovery-lifecycle"
            unknown = scenario.terminal_command["request"]
            with patch.object(lost_recovery_runtime, "_lineage_context", return_value=scenario.manager), patch.object(
                    Path, "exists", new=lambda path:
                    str(path) in scenario.files or path == directory / "requests"):
                self.assertTrue(lost_recovery_runtime.interrupted_request(scenario.state, unknown))
                recovery_keeper._submission_idle(scenario.state, directory)
                self.assertFalse((directory / "results" / (scenario.unknown_id + ".json")).exists())
                foreign_id = "f" * 32
                foreign = scenario.put(directory / "history/foreign-unknown-request.json", {
                    "kind": recovery_keeper.REQUEST_KIND, "requestId": foreign_id, "action": "stop"})
                scenario.put(directory / "requests" / (foreign_id + ".json"), foreign)
                self.assertFalse(lost_recovery_runtime.interrupted_request(scenario.state, foreign))
                with self.assertRaises(JenkinsError) as rejected:
                    recovery_keeper._submission_idle(scenario.state, directory)
                self.assertEqual("recovery_keeper_command_pending", rejected.exception.code)

    def test_persisted_attempt_precedes_native_drain_and_unknown_cannot_be_replayed(self):
        state = Path(__file__).absolute().parents[3] / ".jenkins/state"
        manager = SimpleNamespace(paths=SimpleNamespace(state=state))
        value = {"pins": {"requestId": "a" * 32}, "originalRequest": {"path": "original", "sha256": "b" * 64}}
        capability = unknown_stop_recovery.UnknownStopNativeRecovery(canonical_json(value), unknown_stop_recovery._ISSUED)
        reserved, events = {}, []
        def reserve(path, mode):
            self.assertEqual("xb", mode)
            if str(path) in reserved:
                raise FileExistsError(str(path))
            stream = Mock()
            stream.__enter__ = Mock(return_value=stream)
            stream.__exit__ = Mock(return_value=False)
            stream.write.side_effect = lambda raw: reserved.setdefault(str(path), raw)
            stream.fileno.return_value = 27
            return stream
        def native_stop():
            self.assertEqual(["fsync"], events)
            self.assertEqual(1, len(reserved))
            raise JenkinsError("controller_unreachable", "Preserve this fresh attempt as unknown")
        native = SimpleNamespace(_stop_locked=Mock(side_effect=native_stop))
        with patch.object(unknown_stop_recovery, "process_lock", return_value=nullcontext()), patch.object(
                unknown_stop_recovery, "_capture", return_value=(value, {}, native)), patch.object(
                upgrade, "_plain"), patch.object(Path, "open", autospec=True, side_effect=reserve), patch.object(
                unknown_stop_recovery.os, "fsync", side_effect=lambda fd: events.append("fsync")), patch.object(
                recovery_keeper, "_archive") as archive:
            with self.assertRaises(JenkinsError):
                unknown_stop_recovery.execute_unknown_stop_native_recovery(manager, capability)
            self.assertEqual("unknown", archive.call_args.args[2]["status"])
            with self.assertRaises(FileExistsError):
                unknown_stop_recovery.execute_unknown_stop_native_recovery(manager, capability)
            self.assertEqual(1, native._stop_locked.call_count)
            self.assertEqual(1, archive.call_count)
            self.assertEqual("unknown-stop-native-attempt-" + "b" * 64 + ".json", Path(next(iter(reserved))).name)

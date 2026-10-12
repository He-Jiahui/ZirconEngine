"""Interrupted requests survive authentic owned transfer archives, in memory."""
from copy import deepcopy
from pathlib import Path
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.deployment import lost_recovery_runtime as recovery, owned_upgrade, recovery_keeper
from tools.jenkins.tests.test_lost_recovery_runtime import DepartedRuntime


class OwnedInterruptionLineage:
    def __init__(self, scenario, stack):
        self.scenario, self.generation, self.nodes = scenario, 0, []
        plan = scenario.prepare()
        actor = scenario.actor(stack, plan)
        request = scenario.put(scenario.state / "recovery-lifecycle/history/lineage-recover-request.json", {
            "schemaVersion": 1, "kind": recovery_keeper.REQUEST_KIND, "action": "recover",
            "requestId": scenario.request_id, "keeper": actor.keeper, "release": actor.release_ref,
            "recoveryDigest": plan.recovery_digest, "runtimeAcceptance": False})
        actor._begin(request, plan=plan)
        self.descriptor = {"schemaVersion": 1, "kind": recovery_keeper.DESCRIPTOR_KIND,
            "admission": "idle", "keeper": actor.keeper, "release": actor.release_ref,
            "bootFileTime": str(scenario.boot), "protectedHolds": scenario.holds,
            "recoveryDigest": plan.recovery_digest,
            "lostRecoveryRuntimeInterruption": actor.runtime_interruption["receipt"],
            "runtime": {"state": "running", "operation": scenario.operation,
                        "host": scenario.host, "selected": scenario.selected},
            "runtimeAcceptance": False, "cargoAcceptance": False, "historicalHoldsReleased": False}
        scenario.put(actor.descriptor, self.descriptor)
        # Keep the real plan/handoff/reference/interruption validators.  The
        # native runtime and sealed historical implementation are lower test
        # boundaries; their dedicated suites check native EOF and source pins.
        scenario.historical.side_effect = lambda config, reference: (
            recovery_keeper._reference(scenario.state, reference, release=True), scenario.target)
        self.native_error = None
        self.native = stack.enter_context(patch.object(recovery_keeper, "_runtime_receipt",
                                                     side_effect=self.runtime_receipt))

    def runtime_receipt(self, config, descriptor, release, target):
        if self.native_error is not None:
            raise self.native_error
        if release["kind"] == recovery_keeper.OWNED_UPGRADE_RELEASE_KIND:
            recovery_keeper._owned_handoff_receipt(config, descriptor, release)
        else:
            recovery.validate_interruption_receipt(self.scenario.state, descriptor, release)
        return recovery_keeper._reference(self.scenario.state, descriptor["runtimeReceipt"])

    def archive(self, name, raw):
        path = self.scenario.state / "recovery-lifecycle/history" / ("lineage-%d-%s.json" % (self.generation, name))
        self.scenario.files[str(path)] = raw
        return self.scenario.ref(path)

    def upgrade(self):
        scenario = self.scenario
        self.generation += 1
        predecessor = deepcopy(self.descriptor)
        runtime = predecessor["runtime"]
        runtime["state"] = "stopped"
        stop = {"schemaVersion": 1, "kind": "recovery-keeper-owned-stop",
            "keeper": predecessor["keeper"], "release": predecessor["release"],
            "operation": runtime["operation"], "host": runtime["host"], "selected": runtime["selected"],
            "runtimeAcceptance": False, "cargoAcceptance": False, "historicalHoldsReleased": False}
        stop_ref = self.archive("owned-stop", recovery.canonical_json(stop) + b"\n")
        predecessor["runtimeReceipt"] = stop_ref
        command = {"status": "stopped", "keeper": predecessor["keeper"], "release": predecessor["release"]}
        raw_anchors, artifacts = {}, {}
        for name, relative, body in (
            ("stoppedDescriptor", "formal-lifecycle/recovery-keeper.json", predecessor),
            ("terminalCommand", "recovery-lifecycle/command.json", command),
            ("stoppedOperation", "deployment-operation.json", runtime["operation"]),
            ("stoppedHost", "deployment/host.json", runtime["host"]),
            ("previousSelector", "deployment/driver.json", runtime["selected"])):
            pointer = scenario.state / relative
            raw_anchors[name] = scenario.put(pointer, body)
            artifacts[name] = {"original": raw_anchors[name],
                               "archive": self.archive(name, scenario.files[str(pointer)])}
        retirement = {"kind": "owned-recovery-keeper-retirement", "keeper": predecessor["keeper"],
            "ownedStop": stop_ref, "runtimeAcceptance": False, "cargoAcceptance": False,
            "historicalHoldsReleased": False}
        retirement_ref = self.archive("retirement", recovery.canonical_json(retirement) + b"\n")
        marker = {"stoppedDescriptor": artifacts["stoppedDescriptor"]["archive"],
                  "ownedStop": stop_ref, "keeperRetirement": retirement_ref}
        spec_ref = scenario.put(scenario.manager.spec.path, {"controller": {
            **scenario.manager.spec.controller, "java": {"executable": str(scenario.manager.java)},
            "warPath": str(scenario.manager.war)}})
        request_id = "%032x" % (self.generation + 100)
        release = {"schemaVersion": 1, "kind": recovery_keeper.OWNED_UPGRADE_RELEASE_KIND,
            "requestId": request_id, "scope": "execution-runtime", "ownedUpgrade": marker,
            "deploymentSpec": spec_ref, "expectedPreviousOwner": scenario.owner,
            "expectedLifecycleFence": scenario.fence, "targetDriver": scenario.target,
            "protectedHolds": scenario.holds, "runtimeAcceptance": False, "cargoAcceptance": False,
            "automaticRestart": False, "queueReplayRequested": False}
        release_ref = scenario.put(scenario.state / "deployment" / ("lineage-release-%d.json" % self.generation), release)
        planned = {"kind": owned_upgrade.KIND, "outcome": "owned-transfer", "requestId": request_id,
            "successorRelease": release_ref, "previousKeeper": predecessor["keeper"],
            "previousRelease": predecessor["release"], "owner": scenario.owner,
            "lifecycleFence": scenario.fence, "targetDriver": scenario.target,
            "rawAnchors": raw_anchors, **marker}
        plan_ref = self.archive("plan", recovery.canonical_json(planned) + b"\n")
        keeper = {**predecessor["keeper"], "pid": 100 + self.generation,
                  "creationTime": str(scenario.boot + self.generation + 1)}
        request = {"kind": recovery_keeper.REQUEST_KIND, "action": "upgrade", "requestId": request_id,
            "keeper": keeper, "release": release_ref, "ownedUpgradeDigest": digest(planned),
            "protectedHoldsDigest": digest(scenario.holds), "runtimeAcceptance": False}
        request_ref = self.archive("request", recovery.canonical_json(request) + b"\n")
        handoff = {"kind": owned_upgrade.HANDOFF_KIND, "outcome": "owned-transfer", "keeper": keeper,
            "release": release_ref, "ownedUpgradeDigest": digest(planned), "request": request_ref,
            "previousKeeper": predecessor["keeper"], "previousRelease": predecessor["release"],
            "ownedStop": stop_ref, "keeperRetirement": retirement_ref, "originalArtifacts": artifacts,
            "stoppedDescriptor": marker["stoppedDescriptor"],
            "terminalCommand": artifacts["terminalCommand"]["archive"],
            **{name: False for name in ("nativeProofSynthesized", "historicalHoldsReleased", "runtimeAcceptance",
                                      "cargoAcceptance", "queueReplayRequested", "automaticRetry")}}
        handoff_ref = self.archive("handoff", recovery.canonical_json(handoff) + b"\n")
        self.descriptor = {"schemaVersion": 1, "kind": recovery_keeper.DESCRIPTOR_KIND,
            "admission": "idle", "keeper": keeper, "release": release_ref,
            "initialOwnedUpgrade": plan_ref, "ownedUpgradeDigest": digest(planned),
            "ownedUpgradeHandoff": handoff_ref, "previousStoppedRuntimeReceipt": stop_ref,
            "bootFileTime": str(scenario.boot), "protectedHolds": scenario.holds,
            "runtime": {**deepcopy(runtime), "state": "running"},
            "runtimeAcceptance": False, "cargoAcceptance": False, "historicalHoldsReleased": False}
        scenario.put(scenario.state / recovery._POINTERS["originalDescriptor"], self.descriptor)
        self.nodes.append({"descriptor": self.descriptor, "predecessor": predecessor,
                           "release": release, "handoff": handoff, "planned": planned,
                           "request": request, "stop": stop, "artifacts": artifacts})
        return self.nodes[-1]

    def repin_handoff(self, node):
        reference = self.scenario.put(Path(node["descriptor"]["ownedUpgradeHandoff"]["path"]), node["handoff"])
        node["descriptor"]["ownedUpgradeHandoff"] = reference
        self.scenario.put(self.scenario.state / recovery._POINTERS["originalDescriptor"], self.descriptor)


class InterruptedRequestLineageTests(unittest.TestCase):
    def test_two_owned_upgrades_preserve_exact_unknown_request_without_writes_or_result(self):
        scenario = DepartedRuntime()
        with scenario.observations() as stack:
            lineage = OwnedInterruptionLineage(scenario, stack)
            lineage.upgrade()
            lineage.upgrade()
            before = deepcopy(scenario.files)
            self.assertTrue(recovery.interrupted_request(scenario.state, scenario.request_ref))
            self.assertEqual(2, lineage.native.call_count)
            self.assertEqual(before, scenario.files)
            self.assertFalse((scenario.state / "recovery-lifecycle/results" / (scenario.stop_id + ".json")).exists())
            self.assertFalse(recovery.interrupted_request(scenario.state, {
                **scenario.request_ref, "sha256": "0" * 64}))
            lineage.native.assert_called()

    def test_sha_rebound_handoff_cannot_change_owner_or_claim_acceptance(self):
        for field, value in (("keeper", {"pid": 1}), ("previousRelease", {"path": "foreign"}),
                             ("nativeProofSynthesized", True), ("historicalHoldsReleased", True),
                             ("automaticRetry", True), ("runtimeAcceptance", True),
                             ("stoppedDescriptor", None), ("originalArtifacts", {})):
            with self.subTest(field=field):
                scenario = DepartedRuntime()
                with scenario.observations() as stack:
                    lineage = OwnedInterruptionLineage(scenario, stack)
                    node = lineage.upgrade()
                    node["handoff"][field] = value
                    lineage.repin_handoff(node)
                    with self.assertRaises(JenkinsError):
                        recovery.interrupted_request(scenario.state, scenario.request_ref)

    def test_missing_archive_marker_or_fake_old_result_each_blocks(self):
        for mutation in ("archive", "marker", "result", "plan"):
            with self.subTest(mutation=mutation):
                scenario = DepartedRuntime()
                with scenario.observations() as stack:
                    lineage = OwnedInterruptionLineage(scenario, stack)
                    node = lineage.upgrade()
                    if mutation == "archive":
                        scenario.files[node["handoff"]["terminalCommand"]["path"]] = b'{"status":"committed"}'
                    elif mutation == "marker":
                        scenario.put(scenario.state / "recovery-lifecycle/requests" / (scenario.stop_id + ".json"), {"changed": True})
                    elif mutation == "result":
                        scenario.put(scenario.state / "recovery-lifecycle/results" / (scenario.stop_id + ".json"), {"fake": True})
                    else:
                        scenario.files[node["descriptor"]["initialOwnedUpgrade"]["path"]] = b'{"outcome":"owned-transfer"}'
                    with self.assertRaises(JenkinsError):
                        recovery.interrupted_request(scenario.state, scenario.request_ref)

    def test_unproven_native_stop_is_not_replaced_by_descriptor_ancestry(self):
        scenario = DepartedRuntime()
        with scenario.observations() as stack:
            lineage = OwnedInterruptionLineage(scenario, stack)
            lineage.upgrade()
            lineage.native_error = JenkinsError("native_stop_unproven", "Native EOF missing")
            with self.assertRaises(JenkinsError) as raised:
                recovery.interrupted_request(scenario.state, scenario.request_ref)
            self.assertEqual("native_stop_unproven", raised.exception.code)
            lineage.native.assert_called_once()

    def test_pinned_spec_and_current_descriptor_changes_block(self):
        for mutation in ("spec", "pointer"):
            with self.subTest(mutation=mutation):
                scenario = DepartedRuntime()
                with scenario.observations() as stack:
                    lineage = OwnedInterruptionLineage(scenario, stack)
                    lineage.upgrade()
                    if mutation == "spec":
                        scenario.put(scenario.manager.spec.path, {"controller": {"warPath": "foreign"}})
                    else:
                        validate = recovery_keeper._owned_handoff_receipt
                        def changed(config, descriptor, release):
                            result = validate(config, descriptor, release)
                            scenario.put(scenario.state / recovery._POINTERS["originalDescriptor"], {**descriptor, "admission": "blocked"})
                            return result
                        stack.enter_context(patch.object(recovery_keeper, "_owned_handoff_receipt", side_effect=changed))
                    with self.assertRaises(JenkinsError):
                        recovery.interrupted_request(scenario.state, scenario.request_ref)

    def test_unknown_transfer_variant_does_not_select_an_unvalidated_ancestor(self):
        scenario = DepartedRuntime()
        with scenario.observations() as stack:
            lineage = OwnedInterruptionLineage(scenario, stack)
            node = lineage.upgrade()
            node["handoff"]["variant"] = "external-native-stop"
            node["handoff"]["originalBlockedDescriptor"] = node["handoff"]["stoppedDescriptor"]
            lineage.repin_handoff(node)
            with self.assertRaises(JenkinsError):
                recovery.interrupted_request(scenario.state, scenario.request_ref)

    def test_lineage_bound_rejects_before_recursive_native_verification(self):
        scenario = DepartedRuntime()
        with scenario.observations() as stack:
            lineage = OwnedInterruptionLineage(scenario, stack)
            for _ in range(65):
                lineage.upgrade()
            before = deepcopy(scenario.files)
            with self.assertRaises(JenkinsError) as raised:
                recovery.interrupted_request(scenario.state, scenario.request_ref)
            self.assertEqual("lost_recovery_runtime_unproven", raised.exception.code)
            self.assertIn("exceeds its bound", str(raised.exception))
            lineage.native.assert_not_called()
            self.assertEqual(before, scenario.files)


if __name__ == "__main__":
    unittest.main()

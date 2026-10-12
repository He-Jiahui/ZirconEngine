"""Synthetic proof/control tests only; no live process, State, or filesystem I/O."""
from __future__ import annotations

import base64
from copy import deepcopy
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import unittest


ENTRY = Path(__file__).absolute().parents[3] / ".jenkins/state/migration-baseline/formal_runtime_lineage_protocol_v2.py"
if hashlib.sha256(ENTRY.read_bytes()).hexdigest() != "795e1a8bf9b714b83ed76f455596c8a14e1e0a544c1e8f05b1bdf68dacbd2005":
    raise ValueError("Frozen pure lifecycle protocol changed")
spec = importlib.util.spec_from_file_location("formal_runtime_lineage_protocol", ENTRY)
protocol = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = protocol
spec.loader.exec_module(protocol)

ROOT = r"E:\Git\ZirconEngine"
HISTORY = ROOT + r"\.jenkins\state\deployment\history"


def artifact(name, document):
    raw = (json.dumps(document, sort_keys=True, indent=2) + "\n").encode("utf-8")
    return protocol.Artifact(HISTORY + "\\" + name + ".json", hashlib.sha256(raw).hexdigest(), raw)


def native(pid, birth, component):
    executable = ROOT + (r"\.jenkins\runtime\python\python.exe" if component == "supervisor"
                         else r"\.jenkins\runtime\java\bin\java.exe")
    return {"pid": pid, "creationTime": str(birth), "executable": executable,
            "commandDigest": hashlib.sha256((component + str(pid)).encode()).hexdigest()}


def generation(name, first_pid, first_birth):
    supervisor = native(first_pid, first_birth, "supervisor")
    controller = native(first_pid + 1, first_birth + 1, "controller")
    agent = native(first_pid + 2, first_birth + 2, "agent")
    operation = {**supervisor, "component": "lifecycle-host", "operationId": name, "generation": name,
                 "hostPid": first_pid, "homePath": ROOT + r"\.jenkins\home", "controlPlaneOnly": False,
                 "state": "running"}
    host = {"operationId": name, "generation": name, "hostPid": first_pid,
            "controllerIdentity": controller, "identity": controller, "agentIdentity": agent,
            "agentLaunchAttempted": True, "controlPlaneOnly": False, "status": "running"}
    return operation, host


def complete(who):
    return {"identity": who, "complete": True, "childrenGone": True, "activeProcesses": 0,
            "stdoutEof": True, "stderrEof": True, "processExitCode": 0}


class Fixture:
    def __init__(self, action="restart"):
        self.a_operation, self.a_host = generation("start-A", 100, 134360000000000000)
        self.a = protocol.owner(self.a_operation, self.a_host, self.a_operation["homePath"])
        self.keeper = native(90, 134359999999999000, "supervisor")
        self.binding = {"repository": ROOT, "homePath": self.a_operation["homePath"],
                        "driverDigest": "d" * 64, "driverRoot": ROOT + "\\.jenkins\\drivers\\" + "d" * 64,
                        "endpoint": {"address": "127.0.0.1", "port": 18080}}
        for key in protocol.REFERENCE_BINDINGS:
            self.binding[key] = artifact(key, {"fixture": key}).ref
        self.selected = {"driverDigest": "d" * 64, "root": self.binding["driverRoot"],
                         "launcher": self.binding["driverRoot"] + r"\launcher.py",
                         "runtimeOperationId": "start-A", "state": "selected-for-runtime-start", "runtimeAcceptance": False}
        self.holds = [{"domain": "resource_hold", "key": "historic-" + str(i),
                       "version": 7, "payload": {"status": "active", "unresolved": True}} for i in range(4)]
        self.journals = {key: artifact("old-" + key, {"fixture": key}).ref for key in ("selection", "start")}
        self.original = artifact("original-A-start", {"kind": "desktop-execution-upgrade-start", "status": "running-acceptance-pending"})
        self.anchor = protocol.RuntimeAnchor.from_verified_issuance(binding=self.binding, keeper=self.keeper,
            runtime_owner=self.a, selected_driver=self.selected, protected_holds=self.holds, journals=self.journals,
            runtime_receipt=self.original, restriction_preflight_digest="e" * 64)
        self.request = artifact("command-request", protocol.request_document(self.anchor, "1" * 32, action))
        self.host_command = artifact("actual-host-command", {"action": "stop", "operationId": "start-A", "generation": "start-A",
            "hostPid": 100, "controllerIdentity": self.a["controller"]})
        self.raw_response = {"status": "stopped", "operationId": "start-A", "generation": "start-A",
                             "nativeTerminationProof": complete(self.a["controller"]),
                             "agentTerminationProof": complete(self.a["agent"])}
        self.rebuild_stop()
        self.b_operation, self.b_host = generation("start-B", 200, 134360000000001000)
        self.b = protocol.owner(self.b_operation, self.b_host, self.binding["homePath"])

    def rebuild_stop(self):
        raw = (json.dumps(self.raw_response, sort_keys=True, indent=2) + "\n").encode()
        transport_hash = hashlib.sha256(raw).hexdigest()
        metadata = {"hostPid": 100, "controllerIdentity": self.a["controller"], "agentIdentity": self.a["agent"],
                    "agentLaunchAttempted": True, "controlPlaneOnly": False}
        self.stop_document = {"schemaVersion": 1, "kind": "owned-stop-metadata-binding",
            "capturedOwner": {"operation": self.a_operation, "host": self.a_host, "homePath": self.binding["homePath"],
                              "liveIdentities": [self.a[k] for k in ("supervisor", "controller", "agent")]},
            "command": self.host_command.document, "commandSha256": self.host_command.sha256,
            "rawStopResult": {"path": HISTORY + r"\actual-result.json", "archivePath": HISTORY + r"\raw-result.json",
                              "sha256": transport_hash, "base64": base64.b64encode(raw).decode()},
            "terminalHostRawSha256": transport_hash, "terminalHost": {**self.raw_response, **metadata},
            "hostDeparture": {"pid": 100, "expectedCreationTime": self.a["supervisor"]["creationTime"], "status": "absent"},
            "nativeProofSynthesized": False, "historicalHoldsReleased": False, "runtimeAcceptance": False}
        self.stop_binding = artifact("owned-stop-binding", self.stop_document)
        self.stopped_operation = {**self.a_operation, "state": "stopped", "serviceReady": False, "health": False,
            "nativeTerminationProof": self.raw_response["nativeTerminationProof"],
            "agentTerminationProof": self.raw_response.get("agentTerminationProof"), "stopBinding": self.stop_binding.ref,
            "stopResult": {"path": HISTORY + r"\raw-result.json", "sha256": transport_hash}}

    def verify_stop(self, **overrides):
        args = {"stop_binding": self.stop_binding, "command": self.host_command,
                "stopped_operation": self.stopped_operation, "holds_observation": self.holds}
        args.update(overrides)
        return protocol.verify_owned_stop(self.anchor, self.request, **args)

    def start_data(self, anchor=None, request=None):
        anchor, request = anchor or self.anchor, request or self.request
        verified = self.verify_stop()
        self.verified_stop = verified
        receipt = artifact("durable-terminal-A", verified.document)
        self.terminal_receipt = receipt
        started = artifact("actual-B-start", {"schemaVersion": 1, "kind": "formal-tray-strict-keeper-start-observation",
            "request": request.ref, "stopReceipt": receipt.ref, "binding": self.binding,
            "driverDigest": self.binding["driverDigest"], "driverRoot": self.binding["driverRoot"],
            "status": "running-acceptance-pending", "operation": self.b_operation, "host": self.b_host,
            **{key: False for key in protocol.ACCEPTANCE_FIELDS}})
        observations = {"owner": self.b, "nativeIdentities": [self.b[k] for k in ("supervisor", "controller", "agent")],
            "protectedHolds": self.holds, "selectedDriver": {**self.selected, "runtimeOperationId": "start-B"},
            "strictSpawn": {"creator": self.keeper, "child": self.b["supervisor"], "requestedProfile": "zircon-strict",
                            "creatorInJob": True, "childInJob": True, "configuration": self.binding["configuration"],
                            "restrictionPreflightDigest": "e" * 64, "creationFlags": 0x08000200},
            "endpoint": {**self.binding["endpoint"], "pid": self.b["controller"]["pid"], "ownerCount": 1},
            "health": {key: True for key in ("controller", "agent", "plugins", "ready", "acceptingRequests")},
            "quietingDown": False}
        return receipt, started, observations


class ProtocolTests(unittest.TestCase):
    def test_request_is_exact_and_observations_are_adapter_only(self):
        f = Fixture()
        expected = deepcopy(f.anchor.value)
        f.selected["driverDigest"] = "f" * 64  # Mutating callers does not alter anchor authority.
        result = protocol.validate_request(f.anchor, f.request, keeper_observation=f.keeper, owner_observation=f.a,
            selected_observation=expected["selected"], journal_observation=f.journals, holds_observation=f.holds,
            command_state="idle", used_request_ids=set())
        self.assertEqual(result, f.request.document)
        for key in ("observations", "allowAdoptPid", "alreadyStopped"):
            with self.subTest(key=key), self.assertRaises(protocol.LineageError):
                spoof = artifact("spoof", {**f.request.document, key: True})
                protocol.validate_request(f.anchor, spoof, keeper_observation=f.keeper, owner_observation=f.a,
                    selected_observation=expected["selected"], journal_observation=f.journals, holds_observation=f.holds,
                    command_state="idle", used_request_ids=set())

    def test_replay_pending_unknown_and_partial_outcomes_block(self):
        f = Fixture()
        for state, ids in (("unknown", set()), ("pending", set()), ("journals-exchange-pending", set()),
                           ("idle", {"1" * 32})):
            with self.subTest(state=state), self.assertRaises(protocol.LineageError):
                protocol.validate_request(f.anchor, f.request, keeper_observation=f.keeper, owner_observation=f.a,
                    selected_observation=f.selected, journal_observation=f.journals, holds_observation=f.holds,
                    command_state=state, used_request_ids=ids)

    def test_keeper_runtime_selector_journal_and_holds_fences(self):
        f = Fixture()
        base = {"keeper_observation": f.keeper, "owner_observation": f.a, "selected_observation": f.selected,
                "journal_observation": f.journals, "holds_observation": f.holds, "command_state": "idle", "used_request_ids": set()}
        for key in ("keeper_observation", "owner_observation", "selected_observation", "journal_observation", "holds_observation"):
            with self.subTest(key=key), self.assertRaises(protocol.LineageError):
                protocol.validate_request(f.anchor, f.request, **{**base, key: {} if key != "holds_observation" else []})

    def test_exact_native_stop_keeps_original_receipt_and_four_holds(self):
        f = Fixture()
        before = deepcopy(f.holds)
        result = f.verify_stop().document
        self.assertEqual(result["previousStartupReceipt"], f.original.ref)
        self.assertEqual(result["stopBinding"], f.stop_binding.ref)
        self.assertEqual(f.holds, before)
        self.assertTrue(all(result[k] is False for k in protocol.ACCEPTANCE_FIELDS))

    def test_incomplete_native_stop_never_becomes_a_stopped_receipt(self):
        for component in ("nativeTerminationProof", "agentTerminationProof"):
            for key, bad in (("complete", False), ("childrenGone", False), ("activeProcesses", True),
                             ("activeProcesses", 1), ("stdoutEof", False), ("stderrEof", False), ("processExitCode", True)):
                with self.subTest(component=component, key=key, bad=bad), self.assertRaises(protocol.LineageError):
                    f = Fixture()
                    f.raw_response[component][key] = bad
                    f.rebuild_stop()
                    f.verify_stop()

    def test_wrong_native_identity_or_command_rejected(self):
        for component in ("nativeTerminationProof", "agentTerminationProof"):
            for key, bad in (("creationTime", "134350000000000001"), ("pid", 999), ("commandDigest", "a" * 64)):
                with self.subTest(component=component, key=key), self.assertRaises(protocol.LineageError):
                    f = Fixture()
                    f.raw_response[component]["identity"] = {**f.raw_response[component]["identity"], key: bad}
                    f.rebuild_stop()
                    f.verify_stop()
        f = Fixture()
        with self.assertRaises(protocol.LineageError):
            f.verify_stop(command=artifact("foreign-command", {**f.host_command.document, "hostPid": 999}))

    def test_absence_alone_or_access_denied_does_not_prove_stop(self):
        f = Fixture()
        f.raw_response.pop("agentTerminationProof")
        f.rebuild_stop()
        with self.assertRaises(protocol.LineageError):
            f.verify_stop()
        for status in ("access-denied", "live", "unknown"):
            with self.subTest(status=status), self.assertRaises(protocol.LineageError):
                f = Fixture()
                changed = deepcopy(f.stop_document)
                changed["hostDeparture"]["status"] = status
                altered = artifact("bad-departure", changed)
                f.verify_stop(stop_binding=altered, stopped_operation={**f.stopped_operation, "stopBinding": altered.ref})

    def test_pid_reuse_requires_complete_later_birth(self):
        f = Fixture()
        changed = deepcopy(f.stop_document)
        changed["hostDeparture"] = {"pid": 100, "expectedCreationTime": f.a["supervisor"]["creationTime"],
                                    "status": "pid-reused", "observedIdentity": native(100, 134360000000010000, "supervisor")}
        receipt = artifact("pid-reuse-stop", changed)
        f.verify_stop(stop_binding=receipt, stopped_operation={**f.stopped_operation, "stopBinding": receipt.ref})
        changed["hostDeparture"]["observedIdentity"]["creationTime"] = f.a["supervisor"]["creationTime"]
        receipt = artifact("same-birth-stop", changed)
        with self.assertRaises(protocol.LineageError):
            f.verify_stop(stop_binding=receipt, stopped_operation={**f.stopped_operation, "stopBinding": receipt.ref})

    def test_stop_raw_bytes_and_stopped_journal_must_match(self):
        f = Fixture()
        for mutate in (lambda d: d["rawStopResult"].update(sha256="0" * 64),
                       lambda d: d["terminalHost"].update(agentLaunchAttempted=False),
                       lambda d: d.update(nativeProofSynthesized=True)):
            with self.subTest(mutate=mutate), self.assertRaises(protocol.LineageError):
                changed = deepcopy(f.stop_document); mutate(changed)
                altered = artifact("bad-raw", changed)
                f.verify_stop(stop_binding=altered, stopped_operation={**f.stopped_operation, "stopBinding": altered.ref})
        with self.assertRaises(protocol.LineageError):
            f.verify_stop(stopped_operation={**f.stopped_operation, "homePath": ROOT + r"\.jenkins\other-home"})
        with self.assertRaises(protocol.LineageError):
            f.verify_stop(holds_observation=[])

    def test_restart_issues_B_pending_without_reusing_A_acceptance(self):
        f = Fixture()
        terminal, started, observations = f.start_data()
        result = protocol.verify_owned_successor(f.anchor, f.request, terminal, started,
            verified_stop=f.verified_stop, fresh_observations=observations).document
        self.assertEqual(result["owner"], f.b)
        self.assertEqual(result["previousRuntimeReceipt"], f.original.ref)
        self.assertEqual(result["terminalReceipt"], terminal.ref)
        self.assertEqual(result["status"], "running-acceptance-pending")
        self.assertTrue(all(result[k] is False for k in protocol.ACCEPTANCE_FIELDS))

    def test_independent_stop_then_start_has_its_own_request_and_stop_link(self):
        f = Fixture(action="stop")
        verified = f.verify_stop()
        terminal = artifact("durable-terminal-A", verified.document)
        stopped = protocol.stopped_anchor(f.anchor, terminal, verified)
        request = artifact("independent-start", protocol.request_document(stopped, "2" * 32, "start"))
        _, started, observations = f.start_data(stopped, request)
        result = protocol.verify_owned_successor(stopped, request, terminal, started,
            verified_stop=verified, fresh_observations=observations).document
        self.assertEqual(result["previousRuntimeReceipt"], terminal.ref)
        self.assertEqual(result["request"], request.ref)

    def test_wrong_or_unrestricted_creator_cannot_be_adopted(self):
        for key, bad in (("creator", native(999, 134359999999999001, "supervisor")),
                         ("creatorInJob", False), ("childInJob", False), ("requestedProfile", "full-access"),
                         ("creationFlags", 0x09000200), ("creationFlags", True),
                         ("configuration", {"path": ROOT + r"\.codex\config.toml", "sha256": "0" * 64})):
            with self.subTest(key=key), self.assertRaises(protocol.LineageError):
                f = Fixture(); terminal, started, observations = f.start_data()
                observations["strictSpawn"][key] = bad
                protocol.verify_owned_successor(f.anchor, f.request, terminal, started,
                    verified_stop=f.verified_stop, fresh_observations=observations)

    def test_missing_endpoint_quietdown_or_native_observation_blocks_ready(self):
        for mutate in (lambda x: x["endpoint"].update(ownerCount=2), lambda x: x["endpoint"].update(port=18081),
                       lambda x: x["endpoint"].update(pid=999), lambda x: x.update(quietingDown=True),
                       lambda x: x["health"].pop("acceptingRequests"), lambda x: x.update(nativeIdentities=[]),
                       lambda x: x.update(protectedHolds=[]), lambda x: x["selectedDriver"].update(driverDigest="0" * 64)):
            with self.subTest(mutate=mutate), self.assertRaises(protocol.LineageError):
                f = Fixture(); terminal, started, observations = f.start_data(); mutate(observations)
                protocol.verify_owned_successor(f.anchor, f.request, terminal, started,
                    verified_stop=f.verified_stop, fresh_observations=observations)

    def test_changed_start_binding_or_false_acceptance_rejected(self):
        for key, bad in (("driverDigest", "a" * 64), ("driverRoot", ROOT + r"\.jenkins\drivers\foreign"),
                         ("runtimeAcceptance", True), ("trayAcceptance", True), ("cargoAcceptance", True)):
            with self.subTest(key=key), self.assertRaises(protocol.LineageError):
                f = Fixture(); terminal, started, observations = f.start_data()
                forged = artifact("forged-start", {**started.document, key: bad})
                protocol.verify_owned_successor(f.anchor, f.request, terminal, forged,
                    verified_stop=f.verified_stop, fresh_observations=observations)

    def test_guard_remains_pending_and_does_not_claim_activation_success(self):
        f = Fixture()
        result = protocol.pending_guard(f.anchor, f.request)
        self.assertEqual(result["status"], "pending")
        self.assertEqual(result["originalActivation"], f.binding["originalActivation"])
        self.assertIs(result["runtimeAcceptance"], False)
        self.assertIs(result["nativeExecutionHoldsReleaseAllowed"], False)

    def test_keeper_and_next_upgrade_anchor_advance_only_by_exact_verified_receipt(self):
        f = Fixture()
        terminal, started, observations = f.start_data()
        result = protocol.verify_owned_successor(f.anchor, f.request, terminal, started,
            verified_stop=f.verified_stop, fresh_observations=observations)
        successor = artifact("durable-B-lineage", result.document)
        next_journals = {key: artifact("B-" + key, {"kind": key, "normalLifecycleReceipt": successor.ref}).ref
                         for key in ("selection", "start")}
        b_anchor = protocol.successor_anchor(f.anchor, successor, result, journals=next_journals)
        self.assertEqual(b_anchor.value["owner"], f.b)
        self.assertEqual(b_anchor.value["keeper"], f.keeper)
        self.assertEqual(b_anchor.value["binding"], f.binding)
        self.assertEqual(b_anchor.runtime_receipt.ref, successor.ref)
        self.assertEqual(b_anchor.value["journals"], next_journals)
        self.assertEqual(b_anchor.value["protectedHolds"], f.holds)
        self.assertEqual(f.anchor.value["owner"], f.a)
        self.assertEqual(f.anchor.runtime_receipt.ref, f.original.ref)
        for key, bad in (("previousRuntimeReceipt", artifact("foreign-root", {}).ref),
                         ("runtimeAcceptance", True), ("owner", f.a)):
            with self.subTest(key=key), self.assertRaises(protocol.LineageError):
                tampered = artifact("wrong-lineage", {**result.document, key: bad})
                protocol.successor_anchor(f.anchor, tampered, result, journals=next_journals)

    def test_archived_chain_requires_original_stop_and_durable_exact_successor(self):
        f = Fixture(); terminal, started, observations = f.start_data()
        verified = protocol.verify_owned_successor(f.anchor, f.request, terminal, started,
            verified_stop=f.verified_stop, fresh_observations=observations)
        successor = artifact("durable-B-lineage", verified.document)
        packet = {"stopRequest": f.request, "stopBinding": f.stop_binding, "hostCommand": f.host_command,
                  "stoppedOperation": f.stopped_operation, "terminalReceipt": terminal, "startRequest": f.request,
                  "startupObservation": started, "successorReceipt": successor, "historicalObservations": observations}
        result = protocol.verify_archived_transition(f.anchor, packet)
        self.assertEqual(result.document, successor.document)
        forged = deepcopy(f.stop_document)
        forged["terminalHost"]["agentTerminationProof"]["childrenGone"] = False
        with self.assertRaises(protocol.LineageError):
            protocol.verify_archived_transition(f.anchor, {**packet, "stopBinding": artifact("bad-stop", forged)})
        with self.assertRaises(protocol.LineageError):
            protocol.verify_archived_transition(f.anchor, {k: v for k, v in packet.items() if k != "hostCommand"})

    def test_immutable_bytes_type_duplicate_keys_and_digest_are_strict(self):
        good = artifact("valid", {"x": 1})
        with self.assertRaises(protocol.LineageError):
            protocol.Artifact(good.path, "0" * 64, good.raw)
        duplicate = b'{"status":"pending","status":"succeeded"}'
        with self.assertRaises(protocol.LineageError):
            protocol.Artifact(good.path, hashlib.sha256(duplicate).hexdigest(), duplicate)
        who = native(1, 134360000000000000, "supervisor")
        for key, bad in (("pid", True), ("creationTime", 134360000000000000), ("commandDigest", "abc")):
            with self.subTest(key=key), self.assertRaises(protocol.LineageError):
                protocol.identity({**who, key: bad})


if __name__ == "__main__":
    unittest.main(verbosity=2)

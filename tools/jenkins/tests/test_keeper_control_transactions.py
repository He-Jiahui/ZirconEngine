"""Draft A→B transactions in a byte-addressed fake store; no live I/O."""
from __future__ import annotations

from contextlib import ExitStack, nullcontext
from copy import deepcopy
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
from types import SimpleNamespace
import unittest
from unittest.mock import patch


HERE = Path(__file__).parent
def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    value = importlib.util.module_from_spec(spec); sys.modules[name] = value
    spec.loader.exec_module(value); return value

fixtures = load("formal_lineage_fixtures", HERE / "test_formal_runtime_lineage.py")
controls = load("formal_control_fixtures", HERE / "test_keeper_control.py")
draft, protocol = controls.draft, fixtures.protocol
ROOT = Path(fixtures.ROOT)


class Memory:
    def __init__(self):
        self.files = {}; self.events = []; self.fail_pointer = None
    def raw(self, path):
        try: return self.files[str(path)]
        except KeyError: raise FileNotFoundError(str(path)) from None
    def sha(self, path): return hashlib.sha256(self.raw(path)).hexdigest()
    def read(self, path): return json.loads(self.raw(path))
    def write_raw(self, path, raw, *, expected_sha=None):
        path = str(path)
        if expected_sha is not None: self.require(self.sha(path) == expected_sha)
        if path == self.fail_pointer: raise OSError("injected unknown pointer write")
        self.events.append(("write", path)); self.files[path] = raw
    def write(self, path, doc, **kwargs): self.write_raw(path, draft.canonical_json(doc) + b"\n", **kwargs)
    def archive(self, state, prefix, raw):
        sha = hashlib.sha256(raw).hexdigest()
        path = state / "deployment/history" / (prefix + "-" + sha + ".json")
        if str(path) in self.files: self.require(self.raw(path) == raw)
        else: self.files[str(path)] = raw; self.events.append(("archive", str(path)))
        return {"path": str(path), "sha256": sha}
    def doc(self, state, prefix, doc): return self.archive(state, prefix, draft.canonical_json(doc) + b"\n")
    def put(self, path, doc):
        raw = (json.dumps(doc, sort_keys=True, indent=2) + "\n").encode()
        self.files[str(path)] = raw
        return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest()}
    def artifact(self, value): self.files[value.path] = value.raw; return value.ref
    def reference(self, ref):
        self.require(set(ref) == {"path", "sha256"} and self.sha(ref["path"]) == ref["sha256"])
        return self.raw(ref["path"])
    @staticmethod
    def require(value):
        if not value: raise draft.JenkinsError("memory_proof_mismatch", "Synthetic immutable or CAS fence changed")


class Scenario:
    def __init__(self):
        self.f = f = fixtures.Fixture()
        self.mem = mem = Memory(); self.state = ROOT / ".jenkins/state"
        self.spec_path = ROOT / ".jenkins/deployment-spec.json"
        self.workflow_path = ROOT / ".jenkins/workflow-spec.json"
        mem.put(self.spec_path, {"controller": f.binding["endpoint"]}); mem.put(self.workflow_path, {"schemaVersion": 1})
        self.config_sha = "a" * 64
        mem.files[str(ROOT / ".codex/config.toml")] = b"test-only canonical configuration"
        self.spawned = False; self.starts = 0; self.stops = 0; self.reject_drain = False
        self.operation, self.host = deepcopy(f.a_operation), deepcopy(f.a_host)
        self.origin_activation = b'{\n  "status": "succeeded", "transitionDigest": "' + b"a" * 64 + b'",\n  "operationId": "old-activation", "candidate": {"runtimeAcceptance": false}\n}\n'
        activation = self.state / "deployment/execution-activation.json"; mem.files[str(activation)] = self.origin_activation
        activation_ref = mem.archive(self.state, "activation-original", self.origin_activation)
        self.review = mem.doc(self.state, "original-upgrade-review", {"kind": "test-reviewed-authority"})
        self.release = mem.put(self.state / "deployment" / ("usable-upgrade-" + f.binding["driverDigest"][:16] + "-1791587008309259500.json"),
            {"schemaVersion": 1, "kind": "desktop-execution-upgrade-release", "repository": str(ROOT),
             "runtimeAcceptance": False, "nativeExecutionHoldsReleaseAllowed": False, "candidateDriverDigest": f.binding["driverDigest"],
            "candidateDriverRoot": f.binding["driverRoot"], "reviewedUpgradePath": self.review["path"], "reviewedUpgradeSha256": self.review["sha256"]})
        self.intent = {"kind": "execution-upgrade-selection-intent", "originalActivation": activation_ref, "review": self.review,
            "plan": {"candidateDriver": {"root": f.binding["driverRoot"], "driverDigest": f.binding["driverDigest"]},
                     "protectedHolds": f.holds, "sourceFiles": {".jenkins/deployment-spec.json": mem.sha(self.spec_path),
                                                               ".jenkins/workflow-spec.json": mem.sha(self.workflow_path)}}}
        intent_ref = mem.doc(self.state, "original-intent", self.intent)
        selection_receipt = mem.doc(self.state, "original-selection", {"kind": "test-reviewed-selection"})
        owned = self.owned(self.operation)
        self.body = {"kind": "desktop-execution-upgrade-start", "status": "running-acceptance-pending",
                     "operation": self.operation, "host": self.host, "ownedGeneration": owned,
                     "candidateDriverDigest": f.binding["driverDigest"], "runtimeAcceptance": False}
        startup_ref = mem.doc(self.state, "original-A-start", self.body)
        journal = {"status": "runtime-started-acceptance-pending", "intent": intent_ref, "selected": {**f.selected, "runtimeOperationId": None},
            "selectionReceipt": selection_receipt, "originalActivation": activation_ref, "upgradeDigest": draft.digest(self.intent),
            "runtimeOperationId": self.operation["operationId"], "ownedGeneration": owned, "startupReceipt": startup_ref,
            "startGuardRestoreAuthorized": True, "runtimeAcceptance": False}
        start = {"kind": "desktop-execution-upgrade-start", "status": "running-acceptance-pending", "runtimeOperationId": self.operation["operationId"],
            "startupReceipt": startup_ref, "ownedGeneration": owned, "upgradeDigest": journal["upgradeDigest"], "selectionReceipt": selection_receipt,
            "runtimeAcceptance": False, "cargoAcceptance": False, "trayAcceptance": False}
        selected_path = self.state / "deployment/driver.json"
        mem.put(selected_path, f.selected)
        self.journal_path = self.state / "deployment/execution-upgrade-selection.json"
        self.start_path = self.state / "deployment/execution-upgrade-start.json"
        mem.put(self.journal_path, journal); mem.put(self.start_path, start)
        self.manager = SimpleNamespace(paths=SimpleNamespace(state=self.state),
            spec=SimpleNamespace(path=self.spec_path, controller={"listenAddress": "127.0.0.1", "httpPort": 18080}),
            _read=lambda: deepcopy(self.operation), _stop_locked=self.stop, _start_locked=self.start,
            register_jobs=lambda: {"status": "registered"}, health=lambda: {k: True for k in ("controller", "agent", "plugins", "ready")},
            _request=lambda _: (200, b'{"quietingDown":false}'))
        self.selection = SimpleNamespace(repo=ROOT, home=ROOT / ".jenkins/home", state=self.state, manager=self.manager,
            journal=self.journal_path, start_journal=self.start_path, selector=selected_path, activation=activation,
            _reference=mem.reference, _snapshot=lambda: (deepcopy(self.operation), deepcopy(self.host)),
            _owner=lambda operation, host: protocol.owner(operation, host, f.binding["homePath"]), _started_chain=self.chain)
        self.keeper_state_path = self.state / "desktop-upgrade-holder/desktop-upgrade-test.json"
        self.keeper_state = {"kind": "desktop-execution-upgrade-keeper", "status": "holding-new-generation-acceptance-pending", "automaticRestart": False,
            "reviewedRelease": self.release, "ownedGeneration": owned, "startupReceipt": startup_ref,
            "restrictionPreflight": self.preflight(ROOT, self.config_sha)}
        mem.put(self.keeper_state_path, self.keeper_state)
        actual_result_path = self.state / "deployment/commands/stop-1.result.json"
        stop_doc = deepcopy(f.stop_document); stop_doc["rawStopResult"]["path"] = str(actual_result_path)
        stop_binding = fixtures.artifact("actual-stop-binding", stop_doc)
        mem.artifact(stop_binding)
        mem.files[str(actual_result_path.with_name("stop-1.json"))] = f.host_command.raw
        self.stopped_operation = {**f.stopped_operation, "stopBinding": stop_binding.ref}
        self.terminal_host = stop_doc["terminalHost"]
        self.published = []
    @staticmethod
    def owned(operation): return {k: operation[k] for k in ("operationId", "generation", "pid", "creationTime", "executable", "commandDigest", "homePath")}
    def chain(self, journal, start, *, check_activation=True):
        if "normalLifecycle" in journal or "normalLifecycle" in start:
            return draft.require_normal_started_chain(self.selection, journal, start, check_activation=check_activation)
        Memory.require(journal["status"] == "runtime-started-acceptance-pending" and start["status"] == "running-acceptance-pending")
        body = self.mem.read(journal["startupReceipt"]["path"])
        return self.intent, body, protocol.owner(body["operation"], body["host"], self.f.binding["homePath"])
    def preflight(self, repo, config_sha):
        return {"configuration": {"updateOpenDenied": True, "sha256Before": self.config_sha, "sha256After": self.config_sha},
                "holderIdentity": self.f.keeper, "associatedJob": {"holderMember": True,
                "memberPids": [self.f.keeper["pid"], *([self.f.b["supervisor"]["pid"]] if self.spawned else [])]}}
    def publish(self, status, evidence):
        self.published.append((status, evidence))
        self.keeper_state.update(evidence); self.keeper_state["status"] = status
        self.mem.write(self.keeper_state_path, self.keeper_state)
    def stop(self):
        self.stops += 1
        if self.reject_drain: raise draft.JenkinsError("drain_timeout", "Synthetic unknown work blocks drain")
        self.operation, self.host = deepcopy(self.stopped_operation), deepcopy(self.terminal_host)
        return deepcopy(self.operation)
    def start(self, lock, *, owned_keeper_start):
        self.starts += 1
        draft.consume_owned_keeper_start(self.manager, owned_keeper_start)
        Memory.require(owned_keeper_start.creation_flags() == 0x08000200)
        self.spawned = True; owned_keeper_start.record_spawn(self.f.b["supervisor"]["pid"])
        self.operation, self.host = deepcopy(self.f.b_operation), deepcopy(self.f.b_host)
        self.mem.write(self.selection.selector, {**self.f.selected, "runtimeOperationId": self.operation["operationId"]})
        return deepcopy(self.operation)
    def environment(self):
        stack = ExitStack(); mem = self.mem
        for name, replacement in (("_raw", mem.raw), ("_sha", mem.sha), ("_read", mem.read), ("_write", mem.write),
                                  ("_write_raw", mem.write_raw), ("_archive", mem.archive), ("_document_archive", mem.doc)):
            stack.enter_context(patch.object(draft, name, side_effect=replacement))
        stack.enter_context(patch.object(Path, "exists", new=lambda p: str(p) in mem.files))
        stack.enter_context(patch.object(draft, "_protocol", return_value=protocol))
        stack.enter_context(patch.object(draft, "_reviewed_code")); stack.enter_context(patch.object(draft, "_reviewed_keeper_argv"))
        stack.enter_context(patch.object(draft, "process_lock", side_effect=lambda *a, **k: nullcontext()))
        stack.enter_context(patch.object(draft, "protected_holds", side_effect=lambda _: deepcopy(self.f.holds)))
        stack.enter_context(patch.object(draft.os, "getpid", return_value=self.f.keeper["pid"]))
        stack.enter_context(patch.object(draft, "_full_identity", side_effect=lambda pid: self.f.keeper if pid == self.f.keeper["pid"] else self.f.b["supervisor"]))
        original_sha = mem.sha
        stack.enter_context(patch.object(draft, "_sha", side_effect=lambda path: self.config_sha
            if Path(path) == ROOT / ".codex/config.toml" else original_sha(path)))
        stack.enter_context(patch.object(draft, "observe_live_identity", side_effect=lambda value: value))
        stack.enter_context(patch.object(draft, "observe_endpoint_owner", side_effect=lambda address, port, pid:
                                      {"address": address, "port": port, "pid": pid, "ownerCount": 1}))
        stack.enter_context(patch("tools.jenkins.deployment.startup_observations.observe_departed_identity", side_effect=lambda expected:
            {"pid": expected["pid"], "expectedCreationTime": expected["creationTime"], "status": "absent"}))
        def guard_only(manager, *, guard_only=False):
            Memory.require(guard_only is True)
            Memory.require(mem.read(self.selection.activation).get("status") == "succeeded")
        stack.enter_context(patch("tools.jenkins.deployment.execution_activation.assert_activation_start_allowed", side_effect=guard_only))
        return stack
    def actor(self): return draft.StrictKeeperLifecycle(self.manager, self.selection, self.keeper_state, self.keeper_state_path,
                                                       preflight=self.preflight, publish=self.publish)
    def request(self, actor, action, request_id="1" * 32):
        ref = self.mem.doc(self.state, "request-" + request_id, protocol.request_document(actor.anchor, request_id, action))
        return draft._artifact(self.selection, ref)


class TransactionTests(unittest.TestCase):
    def test_verified_same_owner_closeout_refreshes_only_journal_fences(self):
        s = Scenario()
        with s.environment():
            actor = s.actor(); before = actor.anchor.value
            j, st = s.mem.read(s.journal_path), s.mem.read(s.start_path)
            closed_j = {**j, "status": "runtime-started-functional-acceptance-succeeded-closed", "successCloseout": {"path": "test", "sha256": "a" * 64}}
            closed_s = {**st, "status": "running-functional-acceptance-succeeded-closed"}
            s.selection._success_closed_documents = lambda ref: ({"plan": {"selectionJournal": j, "startJournal": st}}, closed_j, closed_s)
            s.mem.put(s.journal_path, closed_j); s.mem.put(s.start_path, closed_s)
            actor._refresh_issuance_locked()
            self.assertNotEqual(actor.anchor.value["journals"], before["journals"])
            self.assertEqual({k: v for k, v in actor.anchor.value.items() if k != "journals"}, {k: v for k, v in before.items() if k != "journals"})
            self.assertEqual((s.starts, s.stops), (0, 0))
            actor._execute(s.request(actor, "restart"))
            self.assertEqual(actor.anchor.value["owner"], s.f.b)

    def test_journal_refresh_cannot_adopt_foreign_runtime_or_incomplete_closeout(self):
        for foreign in (False, True):
            with self.subTest(foreign=foreign):
                s = Scenario()
                with s.environment():
                    actor = s.actor(); before = actor.anchor.digest
                    j, st = s.mem.read(s.journal_path), s.mem.read(s.start_path)
                    changed = {**j, "status": "runtime-started-functional-acceptance-succeeded-closed", "successCloseout": {"path": "test", "sha256": "a" * 64}}
                    s.mem.put(s.journal_path, changed)
                    if foreign:
                        s.selection._success_closed_documents = lambda ref: ({"plan": {"selectionJournal": j, "startJournal": st}}, changed, st)
                        s.operation, s.host = deepcopy(s.f.b_operation), deepcopy(s.f.b_host)
                    else:
                        s.selection._success_closed_documents = lambda ref: ({"plan": {"selectionJournal": j, "startJournal": st}}, changed, {**st, "status": "closed"})
                    with self.assertRaises(draft.JenkinsError): actor._refresh_issuance_locked()
                    self.assertEqual(actor.anchor.digest, before)
                    self.assertEqual((s.starts, s.stops), (0, 0))

    def test_restart_full_immutable_commit_and_next_reader_keep_B_pending(self):
        scenario = s = Scenario()
        with s.environment():
            actor = s.actor(); old_files = dict(s.mem.files); request = s.request(actor, "restart")
            actor._execute(request)
            self.assertEqual((s.starts, s.stops), (1, 1))
            self.assertEqual(actor.anchor.value["owner"], s.f.b)
            self.assertEqual(s.keeper_state["ownedGeneration"], s.owned(s.f.b_operation))
            self.assertEqual(s.mem.raw(s.selection.activation), s.origin_activation)
            self.assertEqual(s.mem.read(actor.command_path)["status"], "committed")
            self.assertFalse(actor.blocked)
            j, st = s.mem.read(s.journal_path), s.mem.read(s.start_path)
            intent, body, owner = draft.require_normal_started_chain(s.selection, j, st)
            self.assertEqual(owner, s.f.b)
            self.assertEqual(body["status"], "running-acceptance-pending")
            self.assertTrue(all(body[k] is False for k in protocol.ACCEPTANCE_FIELDS))
            for path, raw in old_files.items():
                if "history" in Path(path).parts: self.assertEqual(s.mem.raw(path), raw)
            self.assertEqual(actor.anchor.value["protectedHolds"], s.f.holds)
            self.assertEqual(s.mem.read(actor.descriptor)["admission"], "idle")

    def test_stop_then_new_explicit_start_links_original_A_without_reapplying(self):
        s = Scenario()
        with s.environment():
            actor = s.actor(); actor._execute(s.request(actor, "stop"))
            self.assertEqual((s.starts, s.stops), (0, 1))
            self.assertEqual(actor.anchor.value["state"], "stopped")
            actor._execute(s.request(actor, "start", "2" * 32))
            self.assertEqual((s.starts, s.stops), (1, 1))
            self.assertEqual(actor.anchor.value["owner"], s.f.b)
            draft.require_normal_started_chain(s.selection, s.mem.read(s.journal_path), s.mem.read(s.start_path))

    def test_partial_journal_exchange_retains_A_keeper_and_blocks_unknown_B(self):
        s = Scenario()
        with s.environment():
            actor = s.actor(); s.mem.fail_pointer = str(s.start_path)
            with self.assertRaises(OSError): actor._execute(s.request(actor, "restart"))
            self.assertEqual((s.starts, s.stops), (1, 1))
            self.assertEqual(s.keeper_state["ownedGeneration"], s.owned(s.f.a_operation))
            self.assertTrue(actor.blocked)
            self.assertEqual(s.mem.read(actor.command_path)["status"], "unknown")
            self.assertEqual(s.mem.read(s.selection.activation)["status"], "pending")
            self.assertIn("createdSupervisor", s.mem.read(actor.command_path))
            with self.assertRaises(draft.JenkinsError): draft.assert_no_unresolved_keeper_command(s.manager)
            self.assertEqual(actor.anchor.value["protectedHolds"], s.f.holds)

    def test_unknown_current_work_prevents_new_child_and_preserves_guard(self):
        s = Scenario(); s.reject_drain = True
        with s.environment():
            actor = s.actor()
            with self.assertRaises(draft.JenkinsError): actor._execute(s.request(actor, "restart"))
            self.assertEqual(s.starts, 0)
            self.assertEqual(s.keeper_state["ownedGeneration"], s.owned(s.f.a_operation))
            self.assertEqual(s.mem.read(actor.command_path)["status"], "unknown")
            self.assertEqual(s.mem.read(s.selection.activation)["status"], "pending")
            self.assertEqual(actor.anchor.value["protectedHolds"], s.f.holds)


if __name__ == "__main__": unittest.main(verbosity=2)


class TypedHoldObservationTests(unittest.TestCase):
    def test_bool_or_float_change_cannot_pass_exact_historical_hold_fence(self):
        for replacement in (True, 1.0):
            s = Scenario()
            s.f.holds[0]["typeProbe"] = 1
            with s.environment():
                actor = s.actor()
                request = s.request(actor, "restart")
                changed = deepcopy(s.f.holds)
                changed[0]["typeProbe"] = replacement
                with patch.object(draft, "protected_holds", return_value=changed):
                    with self.assertRaises(protocol.LineageError):
                        actor._execute(request)
                self.assertEqual((0, 0), (s.starts, s.stops))
                self.assertNotIn(str(actor.command_path), s.mem.files)

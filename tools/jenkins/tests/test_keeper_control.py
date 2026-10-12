"""Run draft executable control code in memory; all side effects are mocked."""
from __future__ import annotations

from contextlib import ExitStack
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
from types import ModuleType, SimpleNamespace
import unittest
from unittest.mock import Mock, patch

ROOT = Path(__file__).absolute().parents[3]
sys.path.insert(0, str(ROOT))
sys.dont_write_bytecode = True
from tools.jenkins.deployment import keeper_control as draft



class FakeProtocol:
    @staticmethod
    def pending_guard(anchor, request):
        return {"status": "pending", "request": request.ref, "anchorDigest": anchor.digest}


def cap_fixture():
    keeper = {"pid": 123, "creationTime": "134360000000000000", "executable": "python.exe", "commandDigest": "a" * 64}
    manager = object()
    config_sha = "b" * 64
    config_ref = {"path": str(ROOT / ".codex/config.toml"), "sha256": config_sha}
    preflight = {"configuration": {"updateOpenDenied": True, "sha256Before": config_sha, "sha256After": config_sha},
                 "associatedJob": {"holderMember": True, "memberPids": [123, 456]}, "holderIdentity": keeper}
    anchor = SimpleNamespace(value={"binding": {"configuration": config_ref, "originalRelease": {}},
                                   "restrictionPreflightDigest": "c" * 64}, digest="d" * 64)
    request = SimpleNamespace(ref={"path": "request.json", "sha256": "e" * 64})
    selection = SimpleNamespace(repo=ROOT, activation=Path("guard.json"))
    actor = SimpleNamespace(manager=manager, keeper=keeper, preflight=Mock(return_value=preflight), selection=selection,
                            config_sha=config_sha, anchor=anchor, protocol=FakeProtocol(), command_path=Path("command.json"), release={})
    guard = FakeProtocol.pending_guard(anchor, request)
    command = {"status": "start-attempted", "request": request.ref, "anchorDigest": anchor.digest}
    docs = {"guard.json": guard, "command.json": command}
    hashes = {"guard.json": "f" * 64, "command.json": "9" * 64}
    cap = draft._OwnedKeeperStartCapability(actor, request, hashes["guard.json"], hashes["command.json"])
    return actor, request, docs, hashes, cap


def finish_fixture():
    keeper = object.__new__(draft.StrictKeeperLifecycle)
    keeper.selection = SimpleNamespace(state=ROOT / ".jenkins/state", activation=Path("guard.json"))
    keeper.command_path = Path("command.json"); keeper.directory = Path("control")
    keeper.keeper_state_path = Path("keeper.json")
    owner = {key: key for key in ("operationId", "generation", "pid", "creationTime", "executable", "commandDigest", "homePath")}
    receipt = {"path": "startup.json", "sha256": "1" * 64}
    keeper.anchor = SimpleNamespace(value={"protectedHolds": [], "owner": {"operation": owner,
        "supervisor": {k: owner[k] for k in ("pid", "creationTime", "executable", "commandDigest")}}}, runtime_receipt=SimpleNamespace(ref=receipt))
    publication = {"status": "holding-new-generation-acceptance-pending", "automaticRestart": False,
                   "ownedGeneration": owner, "startupReceipt": receipt, "normalLifecycleState": "running"}
    keeper.blocked = True
    return keeper, publication


class ControlPatchTests(unittest.TestCase):
    def cap_environment(self, actor, docs, hashes):
        stack = ExitStack()
        stack.enter_context(patch.object(draft.os, "getpid", return_value=123))
        stack.enter_context(patch.object(draft, "_full_identity", side_effect=lambda pid: actor.keeper if pid == 123 else
            {"pid": pid, "creationTime": "134360000000000001", "executable": "python.exe", "commandDigest": "1" * 64}))
        stack.enter_context(patch.object(draft, "_sha", side_effect=lambda path: hashes[str(path)]))
        stack.enter_context(patch.object(draft, "_read", side_effect=lambda path: docs[str(path)]))
        stack.enter_context(patch.object(draft, "_reviewed_keeper_argv"))
        stack.enter_context(patch.object(draft, "_reviewed_code"))
        return stack

    def test_one_shot_actual_keeper_capability_does_not_restore_guard(self):
        actor, request, docs, hashes, cap = cap_fixture()
        before = json.loads(json.dumps(docs))
        with self.cap_environment(actor, docs, hashes), patch.object(draft, "_write") as write:
            self.assertIsNone(draft.consume_owned_keeper_start(actor.manager, cap))
            self.assertTrue(cap.used)
            write.assert_not_called()
            self.assertEqual(docs, before)
            with self.assertRaises(draft.JenkinsError):
                draft.consume_owned_keeper_start(actor.manager, cap)

    def test_serializable_or_wrong_actor_capability_never_grants_start(self):
        actor, request, docs, hashes, cap = cap_fixture()
        with self.cap_environment(actor, docs, hashes):
            for value in ({"request": request.ref}, None, SimpleNamespace(consume=lambda _: None)):
                with self.subTest(value=value), self.assertRaises(draft.JenkinsError):
                    draft.consume_owned_keeper_start(actor.manager, value)
            with self.assertRaises(draft.JenkinsError):
                draft.consume_owned_keeper_start(object(), cap)

    def test_wrong_live_keeper_birth_or_pid_blocks_even_valid_guard(self):
        actor, request, docs, hashes, cap = cap_fixture()
        with self.cap_environment(actor, docs, hashes):
            with patch.object(draft.os, "getpid", return_value=999), self.assertRaises(draft.JenkinsError):
                draft.consume_owned_keeper_start(actor.manager, cap)
            with patch.object(draft, "_full_identity", return_value={**actor.keeper, "creationTime": "134360000000000001"}), self.assertRaises(draft.JenkinsError):
                draft.consume_owned_keeper_start(actor.manager, cap)
            self.assertFalse(cap.used)

    def test_actual_restriction_job_and_configuration_observations_are_required(self):
        for section, key, bad in (("configuration", "updateOpenDenied", False), ("configuration", "sha256Before", "0" * 64),
                                  ("configuration", "sha256After", "0" * 64), ("associatedJob", "holderMember", False)):
            actor, request, docs, hashes, cap = cap_fixture()
            actor.preflight.return_value[section][key] = bad
            with self.subTest(key=key), self.cap_environment(actor, docs, hashes), self.assertRaises(draft.JenkinsError):
                draft.consume_owned_keeper_start(actor.manager, cap)
            self.assertFalse(cap.used)

    def test_pending_intent_guard_and_exact_delivery_fences_are_required(self):
        for target, key, bad in (("guard.json", "status", "succeeded"), ("command.json", "status", "unknown"),
                                 ("command.json", "status", "committed"), ("command.json", "anchorDigest", "0" * 64),
                                 ("command.json", "request", {"path": "foreign.json", "sha256": "0" * 64})):
            actor, request, docs, hashes, cap = cap_fixture(); docs[target][key] = bad
            with self.subTest(key=key, bad=bad), self.cap_environment(actor, docs, hashes), self.assertRaises(draft.JenkinsError):
                draft.consume_owned_keeper_start(actor.manager, cap)
        actor, request, docs, hashes, cap = cap_fixture(); hashes["command.json"] = "0" * 64
        with self.cap_environment(actor, docs, hashes), self.assertRaises(draft.JenkinsError):
            draft.consume_owned_keeper_start(actor.manager, cap)

    def test_spawn_is_nonbreakaway_and_exact_once_in_retained_job(self):
        actor, request, docs, hashes, cap = cap_fixture()
        with self.cap_environment(actor, docs, hashes), patch.object(draft, "_write") as write:
            draft.consume_owned_keeper_start(actor.manager, cap)
            self.assertEqual(cap.creation_flags(), 0x08000200)
            cap.record_spawn(456)
            self.assertEqual(cap.spawn["child"]["pid"], 456)
            self.assertEqual(cap.spawn["creator"], actor.keeper)
            self.assertEqual(cap.spawn["creationFlags"] & 0x01000000, 0)
            self.assertEqual(write.call_count, 1)
            with self.assertRaises(draft.JenkinsError):
                cap.record_spawn(456)
            with self.assertRaises(draft.JenkinsError):
                cap.creation_flags()

    def test_foreign_child_or_missing_creation_is_not_adopted(self):
        actor, request, docs, hashes, cap = cap_fixture()
        with self.cap_environment(actor, docs, hashes), patch.object(draft, "_write") as write:
            with self.assertRaises(draft.JenkinsError):
                cap.record_spawn(456)
            draft.consume_owned_keeper_start(actor.manager, cap)
            with self.assertRaises(draft.JenkinsError):
                cap.record_spawn(999)
            self.assertIsNone(cap.spawn)
            write.assert_not_called()

    def test_every_ordinary_start_blocks_unresolved_command(self):
        manager = SimpleNamespace(paths=SimpleNamespace(state=ROOT / ".jenkins/state"))
        command_path = manager.paths.state / "deployment/formal-lifecycle-command.json"
        with patch.object(Path, "exists", autospec=True, side_effect=lambda path: path == command_path):
            for status in ("pending", "stop-attempted", "start-attempted", "unknown", "pointer-exchange-pending"):
                with self.subTest(status=status), patch.object(draft, "_read", return_value={"kind": draft.COMMAND_KIND, "status": status}), self.assertRaises(draft.JenkinsError):
                    draft.assert_no_unresolved_keeper_command(manager)
            for status in draft.TERMINAL_COMMANDS:
                with patch.object(draft, "_read", return_value={"kind": draft.COMMAND_KIND, "status": status}):
                    draft.assert_no_unresolved_keeper_command(manager)
            with patch.object(draft, "_read", return_value={"kind": "foreign", "status": "committed"}), self.assertRaises(draft.JenkinsError):
                draft.assert_no_unresolved_keeper_command(manager)

    def test_guard_restoration_is_last_and_uses_original_exact_bytes(self):
        actor, request, docs, hashes, cap = cap_fixture()
        keeper, publication = finish_fixture()
        keeper.origin_activation = b'{\n  "status": "succeeded", "history": "exact original bytes"\n}\n'
        keeper.blocked = True
        events = []
        command = {"status": "start-attempted", "request": request.ref}
        actor_request = SimpleNamespace(ref=request.ref, document={"requestId": "1" * 32})
        with patch.object(draft, "protected_holds", return_value=[]), patch.object(draft, "_document_archive", return_value={"path": "result.json", "sha256": "a" * 64}), \
             patch.object(draft, "_read", side_effect=lambda path: publication if path == keeper.keeper_state_path else command), patch.object(draft, "_sha", return_value="b" * 64), \
             patch.object(draft, "_raw", return_value=keeper.origin_activation), \
             patch.object(draft, "_write", side_effect=lambda path, value, **kw: events.append(("write", str(path), value))), \
             patch.object(draft, "_write_raw", side_effect=lambda path, raw, **kw: events.append(("raw", str(path), raw))), \
             patch.object(keeper, "_advertise", side_effect=lambda: events.append(("advertise",))):
            keeper._finish(actor_request, {"status": "running-acceptance-pending", "runtimeAcceptance": False}, "committed")
        self.assertEqual(events[0][2]["status"], "committed")
        self.assertEqual(events[1], ("raw", "guard.json", keeper.origin_activation))
        self.assertEqual(events[-1], ("advertise",))
        self.assertFalse(keeper.blocked)

    def test_unknown_guard_restore_never_publishes_idle_or_success_result(self):
        actor, request, docs, hashes, cap = cap_fixture()
        keeper, publication = finish_fixture()
        keeper.origin_activation = b'{"status":"succeeded"}\n'; keeper.blocked = True
        request.document = {"requestId": "1" * 32}
        with patch.object(draft, "protected_holds", return_value=[]), patch.object(draft, "_document_archive", return_value={"path": "result.json", "sha256": "a" * 64}), \
             patch.object(draft, "_read", side_effect=lambda path: publication if path == keeper.keeper_state_path else {"status": "start-attempted", "request": request.ref}), \
             patch.object(draft, "_sha", return_value="b" * 64), patch.object(draft, "_write") as write, \
             patch.object(draft, "_write_raw", side_effect=OSError("unknown outcome")), patch.object(keeper, "_advertise") as advertise:
            with self.assertRaises(OSError):
                keeper._finish(request, {"status": "running-acceptance-pending", "runtimeAcceptance": False}, "committed")
            self.assertTrue(keeper.blocked)
            self.assertEqual(write.call_count, 1)
            advertise.assert_not_called()

    def test_missing_durable_keeper_owner_cannot_restore_start_guard(self):
        actor, request, docs, hashes, cap = cap_fixture()
        keeper, publication = finish_fixture()
        publication["ownedGeneration"] = {"pid": 999}
        with patch.object(draft, "protected_holds", return_value=[]), patch.object(draft, "_read", return_value=publication), \
             patch.object(draft, "_write_raw") as restore, self.assertRaises(draft.JenkinsError):
            keeper._finish(request, {}, "committed")
        restore.assert_not_called()

    def test_public_start_requires_keeper_for_same_selected_driver(self):
        manager = SimpleNamespace(paths=SimpleNamespace(state=ROOT / ".jenkins/state"))
        binding = {"driverDigest": "a" * 64, "driverRoot": "candidate-A"}
        descriptor = {"kind": "formal-tray-strict-keeper-descriptor", "anchor": {"binding": binding}}
        descriptor_path = manager.paths.state / "formal-lifecycle/keeper.json"
        with patch.object(Path, "exists", autospec=True, side_effect=lambda path: path == descriptor_path), \
             patch.object(draft, "_read", side_effect=[descriptor, {"driverDigest": "a" * 64, "root": "candidate-A"}]), self.assertRaises(draft.JenkinsError) as raised:
            draft.assert_keeper_managed_start(manager)
        self.assertEqual(raised.exception.code, "formal_keeper_request_required")
        # A subsequent reviewed candidate selection is a different generation,
        # so its original upgrade start follows its existing activation gate.
        with patch.object(Path, "exists", autospec=True, side_effect=lambda path: path == descriptor_path), \
             patch.object(draft, "_read", side_effect=[descriptor, {"driverDigest": "b" * 64, "root": "candidate-B"}]):
            draft.assert_keeper_managed_start(manager)

    def test_public_start_requires_recovery_keeper_before_legacy_driver_checks(self):
        manager = SimpleNamespace(paths=SimpleNamespace(state=ROOT / ".jenkins/state"))
        descriptor_path = manager.paths.state / "formal-lifecycle/recovery-keeper.json"
        with patch.object(Path, "exists", autospec=True, side_effect=lambda path: path == descriptor_path), \
             patch.object(draft, "_read") as read, self.assertRaises(draft.JenkinsError) as raised:
            draft.assert_keeper_managed_start(manager)
        self.assertEqual(raised.exception.code, "formal_keeper_request_required")
        read.assert_not_called()


class ReviewedKeeperArgvTests(unittest.TestCase):
    def setUp(self):
        release_name = "usable-upgrade-" + "a" * 16 + "-1791595837563356400.json"
        self.release = {"path": str(ROOT / ".jenkins/state/deployment" / release_name), "sha256": "b" * 64}
        self.config_sha = "c" * 64
        self.expected = [str(ROOT / ".jenkins/runtime/python/python.exe"), "-B",
            str(ROOT / ".jenkins/state/migration-baseline/jenkins_upgrade_desktop_holder.py"),
            "--config-sha256", self.config_sha, "--release-file", self.release["path"],
            "--release-sha256", self.release["sha256"]]
        self.native = {"pid": 6140, "creationTime": "134360698661655756",
                       "executable": str(ROOT / ".jenkins/runtime/python/python.exe")}

    def call_guard(self, observed, *, command_digest=None):
        keeper = {**self.native, "commandDigest": draft._digest_args(observed) if command_digest is None else command_digest}
        with patch.object(draft, "_windows_command_line", return_value=observed) as query, \
             patch.object(draft, "_write") as write, patch.object(draft, "_archive") as archive:
            draft._reviewed_keeper_argv(ROOT, keeper, self.release, self.config_sha)
            query.assert_called_once_with(self.native["pid"])
            write.assert_not_called()
            archive.assert_not_called()
        self.assertEqual(self.native, {key: keeper[key] for key in self.native})

    def test_actual_native_tuple_and_control_list_accept_exact_default_or_no_tray(self):
        # Real Windows observation exports tuple; test controls also use list.
        for factory in (tuple, list):
            for suffix in ([], ["--no-tray"]):
                with self.subTest(shape=factory.__name__, suffix=suffix):
                    self.call_guard(factory([*self.expected, *suffix]))

    def test_unknown_extra_argv_rejected_even_when_command_hash_matches(self):
        for suffix in (["--no-tray", "extra"], ["--check-only"], ["--no-tray", "--no-tray"]):
            with self.subTest(suffix=suffix), self.assertRaises(draft.JenkinsError):
                self.call_guard(tuple([*self.expected, *suffix]))

    def test_exact_argv_with_foreign_command_hash_is_rejected(self):
        for factory in (tuple, list):
            with self.subTest(shape=factory.__name__), self.assertRaises(draft.JenkinsError):
                self.call_guard(factory(self.expected), command_digest="d" * 64)

    def test_foreign_issuer_path_config_or_release_rejected_even_with_matching_hash(self):
        for index, foreign in ((0, str(ROOT / ".jenkins/runtime/foreign-python.exe")),
                               (2, str(ROOT / ".jenkins/state/migration-baseline/resume_selected_execution_upgrade.py")),
                               (4, "e" * 64), (6, self.release["path"] + "-foreign.json"), (8, "f" * 64)):
            observed = list(self.expected)
            observed[index] = foreign
            with self.subTest(index=index), self.assertRaises(draft.JenkinsError):
                self.call_guard(tuple(observed))


class PublicKeeperRequestArgvTests(unittest.TestCase):
    def environment(self, suffix=(), factory=tuple, mutation=None):
        stack = ExitStack()
        self.addCleanup(stack.close)
        state = ROOT / ".jenkins/state"
        config = SimpleNamespace(paths=SimpleNamespace(repo=ROOT, state=state, home=ROOT / ".jenkins/jenkins_home",
                                   runtime=ROOT / ".jenkins/runtime"), spec=SimpleNamespace(path=ROOT / ".jenkins/deployment-spec.json"))
        entry = ROOT / ".jenkins/state/migration-baseline/jenkins_upgrade_desktop_holder.py"
        release_ref = {"path": str(state / "deployment/usable-upgrade-aaaaaaaaaaaaaaaa-1791595837563356400.json"), "sha256": "b" * 64}
        origin_ref = {"path": str(state / "deployment/history/formal-lifecycle-keeper-origin.json"), "sha256": "c" * 64}
        runtime_ref = {"path": str(state / "deployment/history/formal-lifecycle-runtime.json"), "sha256": "d" * 64}
        journals = {"selection": {"path": "selection.json", "sha256": "e" * 64},
                    "start": {"path": "start.json", "sha256": "e" * 64}}
        observed = [str(config.paths.runtime / "python/python.exe"), "-B", str(entry), "--config-sha256", "f" * 64,
                    "--release-file", release_ref["path"], "--release-sha256", release_ref["sha256"], *suffix]
        if mutation == "extra":
            observed.append("--check-only")
        elif mutation == "issuer":
            observed[2] = str(ROOT / ".jenkins/state/migration-baseline/resume_selected_execution_upgrade.py")
        observed = factory(observed)
        native = {"pid": 6140, "creationTime": "134360698661655756", "executable": str(config.paths.runtime / "python/python.exe"),
                  "commandDigest": "1" * 64 if mutation == "command" else draft._digest_args(observed)}
        holder_identity = {**native, "commandDigest": ""}
        if mutation == "birth":
            holder_identity["creationTime"] = "134360698661655757"
        elif mutation == "exe":
            holder_identity["executable"] = str(ROOT / ".jenkins/runtime/foreign-python.exe")
        binding = {"repository": str(ROOT), "homePath": str(config.paths.home), "deploymentSpec": {"sha256": "e" * 64},
                   "configuration": {"sha256": "f" * 64}, "originalRelease": release_ref}
        value = {"binding": binding, "keeper": native, "owner": {"verified": "fixture-owner"}, "selected": {"verified": "fixture-driver"},
                 "protectedHolds": [], "journals": journals, "restrictionPreflightDigest": "2" * 64, "state": "running"}
        descriptor = {"kind": "formal-tray-strict-keeper-descriptor", "admission": "idle", "anchor": value,
                      "anchorDigest": "3" * 64, "keeperRecord": origin_ref, "runtimeReceipt": runtime_ref}
        origin = {"reviewedRelease": release_ref, "automaticRestart": False,
                  "restrictionPreflight": {"holderIdentity": holder_identity}}
        release = {"operatorFiles": {str(entry): "4" * 64}}
        request_ref = {"path": str(state / "deployment/history/request.json"), "sha256": "5" * 64}
        result_ref = {"path": str(state / "deployment/history/result.json"), "sha256": "6" * 64}
        result = {"request": request_ref, "status": "running-acceptance-pending", "runtimeAcceptance": False}
        documents = {origin_ref["path"]: origin, release_ref["path"]: release, runtime_ref["path"]: {"verified": "runtime"},
                     result_ref["path"]: result}
        class Artifact:
            def __init__(self, path, sha256, raw):
                self.document = json.loads(raw)
        anchor_api = Mock()
        anchor_api.from_verified_issuance.return_value = SimpleNamespace(digest="3" * 64)
        protocol = SimpleNamespace(Artifact=Artifact, RuntimeAnchor=anchor_api, request_document=Mock(return_value={"action": "restart"}))
        stack.enter_context(patch.object(draft, "_protocol", return_value=protocol))
        stack.enter_context(patch.object(draft, "process_lock", return_value=ExitStack()))
        stack.enter_context(patch.object(draft, "_read", side_effect=lambda p: result_ref if Path(p).parent.name == "results" else descriptor))
        stack.enter_context(patch.object(draft, "_raw", side_effect=lambda p: json.dumps(documents[str(p)]).encode()))
        stack.enter_context(patch.object(draft, "_sha", side_effect=lambda p: "0" * 64 if mutation == "operator" and Path(p) == entry else "4" * 64 if Path(p) == entry else "e" * 64))
        stack.enter_context(patch.object(draft, "_windows_command_line", return_value=observed))
        native_check = stack.enter_context(patch.object(draft, "observe_live_identity"))
        if mutation == "native":
            native_check.side_effect = draft.JenkinsError("native_changed", "Native owner birth changed")
        archive = stack.enter_context(patch.object(draft, "_document_archive", return_value=request_ref))
        write = stack.enter_context(patch.object(draft, "_write"))
        # This fixture models the legacy keeper and its exact result. A broad
        # existence mock would activate a separate recovery keeper protocol.
        existing_paths = {state / "formal-lifecycle/keeper.json",
                          state / "formal-lifecycle/results" / ("7" * 32 + ".json")}
        stack.enter_context(patch.object(Path, "exists", autospec=True,
                                        side_effect=lambda path: path in existing_paths))
        stack.enter_context(patch.object(draft.uuid, "uuid4", return_value=SimpleNamespace(hex="7" * 32)))
        return config, native, native_check, archive, write, result, protocol

    def test_public_request_accepts_actual_tuple_and_fixture_list_exact_default_or_no_tray(self):
        for factory in (tuple, list):
            for suffix in ((), ("--no-tray",)):
                with self.subTest(shape=factory.__name__, suffix=suffix):
                    config, native, check, archive, write, result, protocol = self.environment(suffix, factory)
                    self.assertEqual(result, draft.request_keeper_action(config, "restart", timeout=1))
                    check.assert_called_once_with(native)
                    archive.assert_called_once()
                    write.assert_called_once()
                    protocol.RuntimeAnchor.from_verified_issuance.assert_called_once()
                    self.doCleanups()

    def test_public_request_rejects_foreign_argv_hash_birth_executable_or_operator_before_issuance(self):
        for mutation in ("extra", "issuer", "command", "birth", "exe", "operator", "native"):
            with self.subTest(mutation=mutation):
                config, native, check, archive, write, result, protocol = self.environment(mutation=mutation)
                with self.assertRaises(draft.JenkinsError):
                    draft.request_keeper_action(config, "restart", timeout=1)
                archive.assert_not_called()
                write.assert_not_called()
                protocol.RuntimeAnchor.from_verified_issuance.assert_not_called()
                self.doCleanups()


class RecoveryKeeperDispatchTests(unittest.TestCase):
    def test_public_request_dispatches_exact_action_to_recovery_without_legacy_issuance(self):
        from tools.jenkins.deployment import recovery_keeper

        config = SimpleNamespace(paths=SimpleNamespace(state=ROOT / ".jenkins/state"))
        descriptor_path = config.paths.state / "formal-lifecycle/recovery-keeper.json"
        result = {"status": "running-acceptance-pending", "runtimeAcceptance": False}
        with patch.object(Path, "exists", autospec=True, side_effect=lambda path: path == descriptor_path), \
             patch.object(recovery_keeper, "request_recovery_keeper_action", return_value=result) as request, \
             patch.object(draft, "_protocol") as protocol, patch.object(draft, "process_lock") as lock, \
             patch.object(draft, "_document_archive") as archive, patch.object(draft, "_write") as write:
            self.assertIs(result, draft.request_keeper_action(config, "restart", timeout=17))
        request.assert_called_once_with(config, "restart", timeout=17)
        protocol.assert_not_called()
        lock.assert_not_called()
        archive.assert_not_called()
        write.assert_not_called()


if __name__ == "__main__":
    unittest.main(verbosity=2)

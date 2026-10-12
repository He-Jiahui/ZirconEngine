"""Positive pre-create authority, byte compensation, and retained exit guards."""
from contextlib import ExitStack, nullcontext
from copy import deepcopy
import fnmatch
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch

from tools.jenkins.contracts import JenkinsError, canonical_json, digest
from tools.jenkins.deployment import pre_spawn_recovery_cancel as cancel, recovery_keeper, lost_recovery_runtime

ROOT = Path(__file__).absolute().parents[3]


class FailedAttempt:
    def __init__(self):
        self.files, self.state = {}, ROOT / ".jenkins/state"
        self.manager = SimpleNamespace(paths=SimpleNamespace(repo=ROOT, state=self.state, home=ROOT / ".jenkins/jenkins_home"))
        self.boot, self.request_id, self.stop_id = "134361000000000000", "1" * 32, "2" * 32
        self.keeper = {"pid": 41, "creationTime": "134361000000000100", "executable": str(ROOT / ".jenkins/runtime/python/python.exe"),
                       "commandDigest": "3" * 64}
        self.dispatcher = {"pid": 40, "creationTime": "134361000000000050", "executable": "pwsh.exe", "commandDigest": "4" * 64}
        self.holds = [{"domain": "pool_writer_hold" if i < 2 else "resource_reservation", "key": str(i), "version": 1,
                       "payload": {"status": "active", "owner": "legacy"}} for i in range(4)]
        original_runtime = self.put(self.state / "recovery-lifecycle/history/original-runtime.json", {"old": "runtime"})
        original_request = self.put(self.state / "recovery-lifecycle/history/original-stop-request.json",
                                    {"requestId": self.stop_id, "action": "stop"})
        self.old_command = {"kind": recovery_keeper.COMMAND_KIND, "status": "unknown", "reasonCode": "drain_unproven",
                            "request": original_request, "previousRuntimeReceipt": original_runtime}
        self.old_raw = (json.dumps(self.old_command, indent=3) + "\r\n").encode("utf-8")
        self.put(self.state / cancel._POINTERS["originalCommand"], raw=self.old_raw)
        self.put(self.state / cancel._POINTERS["originalDescriptor"], {"runtimeReceipt": original_runtime})
        for name in ("previousOperation", "previousHost", "previousSelector"):
            self.put(self.state / cancel._POINTERS[name], {"oldOwner": name})
        originals = {name: self.ref(self.state / path) for name, path in cancel._POINTERS.items()}
        originals.update(originalRequest=original_request, previousRuntimeReceipt=original_runtime)
        self.fence = {name: self.ref(self.state / name)["sha256"] for name in
                      ("deployment-operation.json", "deployment/host.json", "deployment/driver.json")}
        for name in recovery_keeper._LEGACY_FILES:
            self.fence[name] = self.put(self.state / name, {"preserved": name})["sha256"]
        self.put(self.state / "recovery-lifecycle/requests" / (self.stop_id + ".json"), original_request)
        self.plan = {"schemaVersion": 1, "kind": "formal-lost-recovery-runtime-interruption", "recoveryVariant": "lost-recovery-runtime", "originalCommandStatus": "unknown",
                     "requestId": self.request_id, "scope": "execution-runtime", "operationId": "start-old", "originalRequestId": self.stop_id,
                     "bootFileTime": self.boot, "owner": {"operation": {"operationId": "start-old"}}, "lifecycleFence": self.fence,
                     "executionQuarantine": {"protectedHolds": self.holds}, **originals, **{name: False for name in cancel._FALSE}}
        plan_ref = self.put(self.state / "migration-baseline/plan.json", self.plan)
        archives = {name: {"original": reference, "archive": self.put(
                    self.state / "recovery-lifecycle/history" / ("original-" + name + ".json"), raw=self.files[reference["path"]])}
                    for name, reference in originals.items()}
        root = ROOT / ".jenkins/runtime/drivers" / ("a" * 64)
        self.target = {"driverDigest": "a" * 64, "root": str(root)}
        self.source_catalog, operators = {}, {}
        for name in next(iter(cancel._REVIEWED_FAILURE_CODE.values())):
            raw = ("audited fixture: " + name).encode("utf-8")
            self.files[str((root if name.startswith("tools/jenkins/") else ROOT) / name)] = raw
            self.source_catalog[name] = operators[str(ROOT / name)] = hashlib.sha256(raw).hexdigest()
        dispatcher_path = ROOT / "tools/dev/start-jenkins-coordinator.ps1"
        self.files[str(dispatcher_path)] = b"audited dispatcher"
        operators[str(dispatcher_path)] = self.ref(dispatcher_path)["sha256"]
        config_path = ROOT / ".codex/config.toml"
        self.files[str(config_path)] = b"strict fixture"
        self.config = self.ref(config_path)
        host_path = ROOT / ".jenkins/runtime/fixture-official/codex.exe"
        self.files[str(host_path)] = b"official fixture"
        self.release = {"schemaVersion": 1, "kind": recovery_keeper.RELEASE_KIND, "requestId": self.request_id,
                        "scope": "execution-runtime", "actorCodeRoot": str(root), "targetDriver": self.target,
                        "lostRecoveryRuntime": {"plan": plan_ref, "originalCommand": originals["originalCommand"],
                                                "originalDescriptor": originals["originalDescriptor"]},
                        "expectedPreviousOwner": self.plan["owner"], "expectedLifecycleFence": self.fence,
                        "protectedHolds": self.holds, "config": self.config, "operatorFiles": operators,
                        "sandboxHost": {**self.ref(host_path), "profile": "zircon-strict", "includeManagedConfig": True}}
        release_ref = self.put(self.state / "deployment/failed-release.json", self.release)
        initial = self.put(self.state / "recovery-lifecycle/history/initial-request.json", {
            "schemaVersion": 1, "kind": recovery_keeper.REQUEST_KIND, "action": "recover", "requestId": self.request_id,
            "release": release_ref, "keeper": self.keeper, "recoveryDigest": digest(self.plan), "runtimeAcceptance": False})
        interruption = self.put(self.state / "recovery-lifecycle/history/interruption.json", {
            "schemaVersion": 1, "kind": self.plan["kind"], "outcome": "interrupted", "originalCommandStatus": "unknown",
            "keeper": self.keeper, "release": release_ref, "request": initial, "recoveryDigest": digest(self.plan),
            "reviewedPlan": plan_ref, "operationId": self.plan["operationId"], "originalRequestId": self.stop_id,
            "bootFileTime": self.boot, "originalArtifacts": archives, **{name: False for name in cancel._FALSE}})
        self.command = {"schemaVersion": 1, "kind": recovery_keeper.COMMAND_KIND, "status": "unknown",
                        "reasonCode": "lost_recovery_runtime_changed", "request": initial, "release": release_ref,
                        "keeper": self.keeper, "recoveryDigest": digest(self.plan), "previousRuntimeReceipt": None,
                        "replacesOriginalCommand": archives["originalCommand"]["archive"],
                        "replacesLostRecoveryDescriptor": archives["originalDescriptor"]["archive"],
                        "lostRecoveryRuntimeInterruption": interruption, "runtimeAcceptance": False, "historicalHoldsReleased": False}
        self.command_path = self.state / cancel._POINTERS["originalCommand"]
        self.put(self.command_path, self.command)
        self.command_sha = self.ref(self.command_path)["sha256"]
        host = {"pid": 43, "creationTime": "134361000000000080", "executable": str(host_path), "commandDigest": "5" * 64}
        host_record = self.put(self.state / "recovery-lifecycle/sandbox-hosts/attempt.json", {
            "kind": "official-strict-recovery-sandbox-host", "release": release_ref, "host": host, "profile": "zircon-strict",
            "includeManagedConfig": True, "automaticRestart": False, "runtimeAcceptance": False,
            "dispatcher": self.ref(dispatcher_path)})
        self.holder = {"schemaVersion": 1, "kind": "formal-recovery-desktop-holder", "status": "recovery-blocked",
                       "reasonCode": self.command["reasonCode"], "message": "Original interruption evidence changed",
                       "reviewedRelease": release_ref, "actorCodeRoot": str(root), "recoveryDigest": digest(self.plan),
                       "ownedUpgradeDigest": None,
                       **{name: False for name in ("runtimeAcceptance", "cargoAcceptance", "automaticRestart", "requestsResubmitted", "serviceLifecycleExecuted")},
                       "restrictionPreflight": {"requestedProfile": "zircon-strict", "holderIdentity": self.keeper,
                           "configuration": {"path": str(config_path), "sha256Before": self.config["sha256"],
                                             "sha256After": self.config["sha256"], "updateOpenDenied": True},
                           "allowedProbe": {"path": str(self.state / "desktop-holder" / ("probe-" + "b" * 32 + ".tmp")),
                                            "exclusiveCreateWrite": True, "removed": True},
                           "associatedJob": {"holderMember": True, "query": "current-process-associated-job", "memberPids": [41, 42]}},
                       "officialSandboxHost": {"host": host, "record": host_record, "officialHostSha256": self.ref(host_path)["sha256"]}}
        self.holder_ref = self.put(self.state / "recovery-lifecycle/holders/failed.json", self.holder)
        self.ledger = "c" * 64
        self.native = SimpleNamespace(verify_scope=Mock(return_value={"terminationScope": ["keeper"]}),
            terminate_keeper=Mock(), collect=Mock(return_value={"exits": [{"processHandleSignaled": True}],
                                                               "logs": [{"exclusiveExistingRead": True}]}), close=Mock())
        self.before_replace = None

    def put(self, path, value=None, *, raw=None):
        self.files[str(path)] = raw if raw is not None else canonical_json(value) + b"\n"
        return self.ref(path)

    def ref(self, path):
        return {"path": str(path), "sha256": hashlib.sha256(self.files[str(path)]).hexdigest()}

    def archive(self, state, prefix, raw):
        return self.put(state / "recovery-lifecycle/history" / (prefix + "-" + hashlib.sha256(raw).hexdigest() + ".json"), raw=raw)

    def replace(self, source, destination, *, before_replace):
        if self.before_replace:
            self.before_replace()
        before_replace()
        self.files[str(destination)] = self.files.pop(str(source))

    def open_pending(self, path, mode):
        assert mode == "xb" and ".pending-" in path.name
        files = self.files
        class Pending(io.BytesIO):
            def fileno(self):
                return 741
            def close(self):
                if not self.closed:
                    files[str(path)] = self.getvalue()
                super().close()
        return Pending()

    def environment(self):
        stack = ExitStack()
        raw = lambda path: self.files[str(Path(path).absolute())]
        stack.enter_context(patch.object(cancel, "_raw", side_effect=raw))
        stack.enter_context(patch.object(recovery_keeper, "_raw", side_effect=raw))
        stack.enter_context(patch.object(lost_recovery_runtime, "_raw", side_effect=raw))
        stack.enter_context(patch.object(cancel, "_plain"))
        stack.enter_context(patch.object(recovery_keeper, "_plain"))
        stack.enter_context(patch.object(Path, "exists", autospec=True, side_effect=lambda path: str(path) in self.files))
        stack.enter_context(patch.object(Path, "glob", autospec=True, side_effect=lambda path, pattern: [Path(name) for name in sorted(self.files)
                                                                                              if Path(name).parent == path and fnmatch.fnmatch(Path(name).name, pattern)]))
        stack.enter_context(patch.object(cancel, "_historical_release", return_value=(self.release, self.target)))
        stack.enter_context(patch.object(cancel, "_REVIEWED_FAILURE_CODE", {"a" * 64: self.source_catalog}))
        stack.enter_context(patch.object(cancel, "windows_boot_filetime", return_value=int(self.boot)))
        stack.enter_context(patch.object(cancel, "protected_holds", side_effect=lambda state: deepcopy(self.holds)))
        stack.enter_context(patch.object(cancel, "_ledger", side_effect=lambda state: self.ledger))
        self.native_factory = stack.enter_context(patch.object(cancel, "_RetainedAttempt", return_value=self.native))
        stack.enter_context(patch.object(cancel, "process_lock", side_effect=lambda *args, **kw: nullcontext()))
        stack.enter_context(patch.object(cancel, "_archive_bytes", side_effect=self.archive))
        stack.enter_context(patch.object(cancel, "_archive", side_effect=lambda state, prefix, value: self.archive(state, prefix, canonical_json(value) + b"\n")))
        stack.enter_context(patch.object(Path, "open", autospec=True, side_effect=self.open_pending))
        stack.enter_context(patch.object(cancel.os, "fsync"))
        stack.enter_context(patch.object(cancel, "replace_file", side_effect=self.replace))
        return stack

    def prepare(self):
        return cancel.prepare_pre_spawn_recovery_cancel(self.manager, expected_command_sha256=self.command_sha,
                                                        holder_reference=self.holder_ref, dispatcher_identity=self.dispatcher)


class PreSpawnRecoveryCancelTests(unittest.TestCase):
    def test_factory_is_read_only_and_restoration_preserves_original_stop_bytes(self):
        fixture = FailedAttempt()
        before = deepcopy(fixture.files)
        with fixture.environment():
            capability = fixture.prepare()
            self.assertEqual(fixture.files, before)
            fixture.native.terminate_keeper.assert_not_called()
            result = capability.cancel()
            self.assertEqual(fixture.files[str(fixture.command_path)], fixture.old_raw)
            self.assertEqual(result["originalCommandStatus"], "unknown")
            self.assertTrue(all(result[name] is False for name in cancel._FALSE))
            for path, raw in before.items():
                if path != str(fixture.command_path):
                    self.assertEqual(fixture.files[path], raw)
            failed_archive = result["failedCommand"]
            self.assertEqual(fixture.files[failed_archive["path"]], before[str(fixture.command_path)])
            fixture.native.terminate_keeper.assert_called_once_with()
            fixture.native.close.assert_called_once_with()
            with self.assertRaises(JenkinsError):
                capability.cancel()

    def test_missing_transaction_does_not_authorize_wrong_failure_reason(self):
        fixture = FailedAttempt()
        fixture.holder["message"] = "Some other failure after spawn"
        fixture.holder_ref = fixture.put(Path(fixture.holder_ref["path"]), fixture.holder)
        with fixture.environment(), self.assertRaises(JenkinsError):
            fixture.prepare()
        fixture.native_factory.assert_not_called()
        fixture.native.terminate_keeper.assert_not_called()

    def test_unreviewed_implementation_and_changed_archive_fail_before_native_retention(self):
        for change in ("source", "archive"):
            fixture = FailedAttempt()
            path = (Path(fixture.target["root"]) / "tools/jenkins/deployment/manager.py" if change == "source" else
                    Path(fixture.command["replacesLostRecoveryDescriptor"]["path"]))
            fixture.files[str(path)] += b" "
            with fixture.environment(), self.assertRaises(JenkinsError):
                fixture.prepare()
            fixture.native_factory.assert_not_called()

    def test_durable_start_transaction_contradicts_pre_create_authority(self):
        fixture = FailedAttempt()
        fixture.put(fixture.state / "recovery-lifecycle/start-transactions/start-new.json",
                    {"keeper": fixture.keeper, "request": fixture.command["request"], "spawnAttempted": True})
        with fixture.environment(), self.assertRaises(JenkinsError):
            fixture.prepare()
        fixture.native_factory.assert_not_called()

    def test_pointer_hold_or_ledger_race_never_terminates_actor(self):
        for change in ("pointer", "hold", "ledger"):
            fixture = FailedAttempt()
            with fixture.environment():
                capability = fixture.prepare()
                if change == "pointer":
                    fixture.files[str(fixture.state / "deployment/driver.json")] += b" "
                elif change == "hold":
                    fixture.holds[0]["version"] += 1
                else:
                    fixture.ledger = "d" * 64
                with self.assertRaises(JenkinsError):
                    capability.cancel()
                fixture.native.terminate_keeper.assert_not_called()
                self.assertEqual(fixture.ref(fixture.command_path)["sha256"], fixture.command_sha)

    def test_changed_command_before_replace_is_preserved_with_cleanup_archived(self):
        fixture = FailedAttempt()
        competing = b'{"status":"competing-request"}\n'
        with fixture.environment():
            capability = fixture.prepare()
            fixture.before_replace = lambda: fixture.files.update({str(fixture.command_path): competing})
            with self.assertRaises(JenkinsError):
                capability.cancel()
            self.assertEqual(fixture.files[str(fixture.command_path)], competing)
            self.assertTrue(any("pre-spawn-recovery-cancel-cleanup-" in path for path in fixture.files))
            fixture.native.terminate_keeper.assert_called_once_with()
            fixture.native.close.assert_not_called()

    def test_native_wait_or_log_failure_keeps_unknown_and_retained_handles(self):
        for code in ("pre_spawn_cancel_exit_unproven", "pre_spawn_cancel_logs_held"):
            fixture = FailedAttempt()
            with fixture.environment():
                capability = fixture.prepare()
                cleanup = fixture.native.collect.return_value
                fixture.native.collect.side_effect = [JenkinsError(code, "not yet proven"), cleanup]
                with self.assertRaises(JenkinsError):
                    capability.cancel()
                self.assertEqual(fixture.ref(fixture.command_path)["sha256"], fixture.command_sha)
                fixture.native.close.assert_not_called()
                capability.cancel()
                self.assertEqual(fixture.files[str(fixture.command_path)], fixture.old_raw)
                fixture.native.terminate_keeper.assert_called_once_with()

    def test_factory_never_accepts_actor_absence_as_exit_proof(self):
        fixture = FailedAttempt()
        before = deepcopy(fixture.files)
        with fixture.environment():
            fixture.native_factory.side_effect = JenkinsError("pre_spawn_cancel_unproven", "Original handle is absent")
            with self.assertRaises(JenkinsError):
                fixture.prepare()
            self.assertEqual(fixture.files, before)
            fixture.native.terminate_keeper.assert_not_called()

    def test_extra_descendant_and_lost_job_membership_fail_closed(self):
        fixture = FailedAttempt()
        probe = cancel._RetainedAttempt.__new__(cancel._RetainedAttempt)
        probe.manager, probe.handles, probe.image_pins = fixture.manager, {"keeper": 1, "console": 2, "officialHost": 3}, {}
        probe.identities = {"keeper": fixture.keeper, "console": {**fixture.keeper, "pid": 42},
                            "officialHost": {**fixture.keeper, "pid": 43}}
        probe.holder, probe._verify = fixture.holder, Mock()
        def contained(handle, job, output):
            output._obj.value = True
            return True
        probe.api = SimpleNamespace(IsProcessInJob=contained)
        with patch.object(cancel, "_descendants", return_value=[41, 42, 43, 999]), self.assertRaises(JenkinsError):
            probe.verify_scope()
        probe.api.IsProcessInJob = Mock(return_value=False)
        with self.assertRaises(JenkinsError):
            probe.verify_scope()

    def test_caller_cannot_construct_cancellation_capability(self):
        with self.assertRaises(JenkinsError):
            cancel.PreSpawnRecoveryCancellation(None, "a" * 64, {}, b"{}", b"{}", b"{}", None, {})

    @unittest.skipUnless(os.name == "nt", "Windows retained native handles")
    def test_real_retained_handle_rejects_wrong_birth_then_proves_exact_owned_exit(self):
        args = [sys.executable, "-B", "-c", "import time; time.sleep(60)"]
        process = subprocess.Popen(args, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                                   creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        api, handle = cancel._kernel(), None
        try:
            identity = cancel._full_identity(process.pid)
            handle = api.OpenProcess(0x1000 | 0x00100000 | 1, False, process.pid)
            self.assertTrue(handle)
            native = cancel._RetainedAttempt.__new__(cancel._RetainedAttempt)
            native.api, native.handles = api, {"keeper": handle}
            native.identities = {"keeper": {**identity, "creationTime": "1"}}
            with self.assertRaises(JenkinsError):
                native.terminate_keeper()
            self.assertIsNone(process.poll())
            native.identities["keeper"] = identity
            native.terminate_keeper()
            self.assertEqual(api.WaitForSingleObject(handle, 10000), 0)
            born, exited, _, _ = cancel._times(api, handle)
            self.assertEqual(born, identity["creationTime"])
            self.assertGreater(int(exited), int(born))
            self.assertEqual(process.wait(timeout=10), 2)
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=10)
            if handle:
                api.CloseHandle(handle)


if __name__ == "__main__":
    unittest.main()

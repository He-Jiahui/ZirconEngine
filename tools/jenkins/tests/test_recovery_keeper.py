"""Recovery authority tests with memory files and mocked native observations."""
from __future__ import annotations

from contextlib import ExitStack, contextmanager
from copy import deepcopy
import fnmatch
import hashlib
import importlib.util
import json
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch

from tools.jenkins.contracts import JenkinsError, canonical_json, digest
from tools.jenkins.deployment import lost_owner, lost_recovery_runtime, owned_upgrade
from tools.jenkins.deployment import recovery_keeper as draft

ROOT = Path(__file__).absolute().parents[3]


class MemoryRuntime:
    """Native creation and filesystem boundaries are replaced, not authority."""
    def __init__(self):
        self.state = ROOT / ".jenkins/state"
        self.files, self.live, self.argv, self.job, self.locks = {}, {}, {}, set(), set()
        self.start_calls, self.stop_calls, self.capabilities = [], [], []
        self.terminal_allowed, self.child_member = True, True
        self.holds = [{"domain": "pool_writer_hold", "key": str(i), "version": 1,
                       "payload": {"status": "active", "token": "historical-" + str(i)}} for i in range(4)]
        self.manager = SimpleNamespace(paths=SimpleNamespace(repo=ROOT, state=self.state,
            home=ROOT / ".jenkins/home"), spec=SimpleNamespace(path=ROOT / ".jenkins/deployment-spec.json",
            controller={"listenAddress": "127.0.0.1", "httpPort": 18743}),
            java=ROOT / ".jenkins/runtime/jdk/bin/java.exe", war=ROOT / ".jenkins/runtime/jenkins.war")
        self.manager._read = lambda: self.read(self.state / "deployment-operation.json")
        self.manager._start_locked, self.manager._stop_locked = self.start, self.stop
        self.manager.register_jobs, self.manager.drain = Mock(return_value={"registered": True}), Mock()
        self.manager.health = Mock(return_value={"ready": True, "controller": True, "agent": True, "plugins": True})
        self.old_target, self.target = self.driver("a" * 64), self.driver("b" * 64)
        self.drivers = {value["driverDigest"]: value for value in (self.old_target, self.target)}
        old = {"schemaVersion": 2, "operationId": "start-10", "generation": "start-10", "state": "running",
               "hostPid": 90, **self.identity(90, ("old-supervisor",)),
               "homePath": str(self.manager.paths.home), "controlPlaneOnly": False}
        old_host = {"status": "running", "operationId": "start-10", "generation": "start-10", "hostPid": 90,
                    "controllerIdentity": self.identity(91, ("old-controller",)), "agentIdentity": self.identity(92, ("old-agent",)),
                    "agentLaunchAttempted": True, "controlPlaneOnly": False}
        previous = {**self.old_target, "runtimeOperationId": "start-10", "state": "selected-for-runtime-start"}
        self.write(self.state / "deployment-operation.json", old)
        self.write(self.state / "deployment/host.json", old_host)
        self.write(self.state / "deployment/driver.json", previous)
        for name in draft._LEGACY_FILES[:-1]:
            self.write(self.state / name, {"original": name, "runtimeAcceptance": False})
        fence = {name: self.sha(self.state / name) if self.exists(self.state / name) else None for name in
                 ("deployment-operation.json", "deployment/host.json", "deployment/driver.json", *draft._LEGACY_FILES)}
        owner = {"operation": old, "supervisor": {key: old[key] for key in
                 ("pid", "creationTime", "executable", "commandDigest")},
                 "controller": old_host["controllerIdentity"], "agent": old_host["agentIdentity"]}
        self.plan_document = {"schemaVersion": 1, "kind": lost_owner.KIND, "requestId": "f" * 32,
            "scope": "execution-runtime", "owner": owner, "lifecycleFence": fence, "controlPlaneOnly": False,
            "nativeProofIssued": False, "historicalEofClaimed": False, "runtimeAcceptance": False}
        self.plan = lost_owner.LostFormalOwnerRecovery(canonical_json(self.plan_document), token=lost_owner._ISSUED)
        config = ROOT / ".codex/config.toml"
        for path in (config, self.manager.spec.path, ROOT / ".jenkins/workflow-spec.json"):
            self.files[path] = ("pinned-input-" + path.name).encode()
        operators = [ROOT / name for name in draft._OPERATOR_FILES]
        operators.append(self.state / "migration-baseline/jenkins_desktop_holder.py")
        for path in operators:
            self.files[path] = ("reviewed-operator-" + path.name).encode()
        self.files[ROOT / "tools/jenkins/deployment/driver.py"] = ("_LAUNCHER = " + repr(draft._LAUNCHER) + "\n").encode()
        self.source_paths = {path for path in operators if path.is_relative_to(ROOT / "tools/jenkins")}
        for path in (ROOT / "tools/jenkins/workflow/handler.py", ROOT / "tools/jenkins/tests/test_recovery_keeper.py",
                     ROOT / ".jenkins/pipeline/steps.groovy"):
            self.files[path] = ("passed-source-" + path.name).encode()
            self.source_paths.add(path)
        self.source_paths.update((self.manager.spec.path, ROOT / ".jenkins/workflow-spec.json"))
        tested_source = self.current_sources()
        files = {}
        for name, checksum in tested_source.items():
            if "/tests/" in name or name in {".jenkins/deployment-spec.json", ".jenkins/workflow-spec.json"}:
                continue
            relative = name[len(".jenkins/"):] if name.startswith(".jenkins/") else name
            files[relative] = {"sha256": checksum, "bytes": len(self.raw(ROOT / name))}
        files["launcher.py"] = {"sha256": hashlib.sha256(draft._LAUNCHER.encode("utf-8")).hexdigest(),
                                "bytes": len(draft._LAUNCHER.encode("utf-8"))}
        package = b'"""This package belongs to one verified Jenkins snapshot."""\n'
        files["tools/__init__.py"] = {"sha256": hashlib.sha256(package).hexdigest(), "bytes": len(package)}
        for name in files:
            if name == "launcher.py": raw = draft._LAUNCHER.encode("utf-8")
            elif name == "tools/__init__.py": raw = package
            else: raw = self.raw(ROOT / (".jenkins/" + name if name.startswith("pipeline/") else name))
            self.files[Path(self.target["root"]) / name] = raw
        self.write(Path(self.target["manifest"]), {"schemaVersion": 1, "driverDigest": self.target["driverDigest"], "files": files})
        self.log = self.state / "migration-baseline/regression.log"
        self.files[self.log] = b"Complete passing support regression output\n"
        regression = self.reference(self.state / "migration-baseline/regression.json", {
            "scope": "complete-current-Jenkins-support-pytest-discovery", "status": "passed", "sourceIdentityStable": True,
            "testsRun": 13, "exitCode": 0, "failures": 0, "errors": 0, "collectionErrors": [],
            "log": str(self.log), "logSha256": self.sha(self.log), "sourceFiles": tested_source})
        preparation = self.reference(self.state / "migration-baseline/preparation.json", {
            "schemaVersion": 1, "state": "prepared-not-selected", "driver": self.target,
            "supportRegression": regression, "previousSelectedDriver": previous,
            "previousSelectorSha256": fence["deployment/driver.json"], "protectedHolds": self.holds})
        self.release = {"schemaVersion": 1, "kind": draft.RELEASE_KIND, "requestId": "f" * 32,
            "config": {"path": str(config), "sha256": self.sha(config)},
            "deploymentSpec": {"path": str(self.manager.spec.path), "sha256": self.sha(self.manager.spec.path)},
            "workflowSpec": {"path": str(ROOT / ".jenkins/workflow-spec.json"), "sha256": self.sha(ROOT / ".jenkins/workflow-spec.json")},
            "operatorFiles": {str(path): self.sha(path) for path in operators}, "targetDriver": self.target,
            "regression": regression, "preparation": preparation, "expectedPreviousOwner": owner,
            "expectedLifecycleFence": fence, "protectedHolds": deepcopy(self.holds), "scope": "execution-runtime",
            "runtimeAcceptance": False, "cargoAcceptance": False}
        self.release_ref = self.reference(self.state / "deployment/recovery-release.json", self.release)
        args = (str(ROOT / ".jenkins/runtime/python/python.exe"), "-B", str(ROOT / "tools/dev/jenkins_recovery_keeper.py"),
                "--config-sha256", self.release["config"]["sha256"], "--release-file", self.release_ref["path"],
                "--release-sha256", self.release_ref["sha256"])
        self.keeper = self.identity(123, args)
        self.live[123], self.argv[123] = self.keeper, args
        self.job.add(123)
        self.probe_mutation, self.stop_mutation = lambda value: None, lambda value: None
        self.publish = Mock()

    @staticmethod
    def driver(checksum):
        root = ROOT / ".jenkins/runtime/drivers" / checksum
        return {"root": str(root), "driverDigest": checksum, "manifest": str(root / "driver-manifest.json"),
                "launcher": str(root / "launcher.py"), "steps": str(root / "pipeline/steps.groovy")}

    @staticmethod
    def identity(pid, args):
        return {"pid": pid, "creationTime": str(134360000000000000 + pid),
                "executable": str(ROOT / ".jenkins/runtime/python/python.exe"), "commandDigest": draft._digest_args(args)}

    def raw(self, path): return self.files[Path(path)]
    def sha(self, path): return hashlib.sha256(self.raw(path)).hexdigest()
    def read(self, path): return json.loads(self.raw(path))
    def current_sources(self): return {path.relative_to(ROOT).as_posix(): self.sha(path) for path in sorted(self.source_paths)}

    def current_regression(self, _repo, reference, _candidate):
        receipt = self.read(reference["path"])
        if receipt["sourceFiles"] != self.current_sources():
            raise JenkinsError("execution_transition_source_changed", "Current source differs from the passed regression")
        return receipt
    def exists(self, path): return Path(path) in self.files or any(Path(path) in name.parents for name in self.files)
    def glob(self, path, pattern):
        return [name for name in self.files if name.parent == Path(path) and fnmatch.fnmatch(name.name, pattern)]

    def write(self, path, value, *, expected_sha=None):
        path = Path(path)
        if expected_sha is not None and self.sha(path) != expected_sha:
            raise JenkinsError("memory_compare_exchange_failed", "changed")
        self.files[path] = canonical_json(value) + b"\n"

    def reference(self, path, value):
        self.write(path, value)
        return {"path": str(path), "sha256": self.sha(path)}

    def archive(self, state, prefix, value):
        checksum = hashlib.sha256(canonical_json(value) + b"\n").hexdigest()
        path = state / "recovery-lifecycle/history" / (prefix + "-" + checksum + ".json")
        if path in self.files:
            assert self.read(path) == value
        return self.reference(path, value)

    def archive_bytes(self, state, prefix, raw):
        checksum = hashlib.sha256(raw).hexdigest()
        path = state / "recovery-lifecycle/history" / (prefix + "-" + checksum + ".json")
        if path in self.files:
            assert self.raw(path) == raw
        self.files[path] = raw
        return {"path": str(path), "sha256": checksum}

    @contextmanager
    def lock(self, path, **_kwargs):
        assert path not in self.locks, "Nested non-reentrant lifecycle lock"
        self.locks.add(path)
        try: yield
        finally: self.locks.remove(path)

    def preflight(self, _repo, _checksum):
        config = self.release["config"]
        value = {"requestedProfile": "zircon-strict", "holderIdentity": deepcopy(self.keeper),
            "configuration": {"path": config["path"], "sha256Before": config["sha256"],
                              "sha256After": config["sha256"], "updateOpenDenied": True},
            "allowedProbe": {"path": str(self.state / "desktop-holder" / ("probe-" + "1" * 32 + ".tmp")),
                             "exclusiveCreateWrite": True, "removed": True},
            "associatedJob": {"query": "current-process-associated-job", "holderMember": True,
                              "memberPids": sorted(self.job)}}
        self.probe_mutation(value)
        return value

    def observe(self, expected):
        actual = self.live.get(expected["pid"])
        if actual != {key: expected[key] for key in ("pid", "creationTime", "executable", "commandDigest")}:
            raise JenkinsError("memory_foreign_identity", "birth or command changed")
        return deepcopy(actual)

    def start(self, lock, *, lost_owner_recovery=None, recovery_keeper_start, owned_upgrade_start=None):
        assert lock in self.locks and not (lost_owner_recovery is not None and owned_upgrade_start is not None)
        cap = recovery_keeper_start
        plan = owned_upgrade_start if owned_upgrade_start is not None else lost_owner_recovery
        cap.consume(self.manager, plan)
        if plan is not None:
            plan.revalidate(self.manager)
        if owned_upgrade_start is not None:
            assert self.manager._read()["state"] == "stopped" and self.terminal_allowed
        target = cap.target_driver(self.manager)
        if target is not None:
            self.write(self.state / "deployment/driver.json", target)
        assert cap.creation_flags() == draft.CREATION_FLAGS
        number = len(self.start_calls) + 1
        pid, operation_id = 200 + number * 10, "start-" + str(10 + number)
        args = (str(ROOT / ".jenkins/runtime/python/python.exe"), self.target["launcher"], "__lifecycle-host",
                "--spec", str(self.manager.spec.path), "--java", str(self.manager.java), "--war", str(self.manager.war),
                "--operation-id", operation_id)
        child = self.identity(pid, args)
        self.live[pid], self.argv[pid] = child, args
        if self.child_member: self.job.add(pid)
        self.capabilities.append(cap)
        cap.record_spawn(pid)
        operation = {"schemaVersion": 2, "state": "running", "operationId": operation_id, "generation": operation_id,
                     "hostPid": pid, **child, "homePath": str(self.manager.paths.home), "controlPlaneOnly": False}
        for native_pid, native_args in ((pid + 1, ("controller", operation_id)), (pid + 2, ("agent", operation_id))):
            self.live[native_pid], self.argv[native_pid] = self.identity(native_pid, native_args), native_args
            self.job.add(native_pid)
        host = {"status": "running", "operationId": operation_id, "generation": operation_id, "hostPid": pid,
                "controllerIdentity": self.live[pid + 1], "agentIdentity": self.live[pid + 2],
                "agentLaunchAttempted": True, "controlPlaneOnly": False}
        selected = self.read(self.state / "deployment/driver.json")
        selected.update(state="selected-for-runtime-start", runtimeOperationId=operation_id)
        self.write(self.state / "deployment/driver.json", selected)
        self.write(self.state / "deployment-operation.json", operation)
        self.write(self.state / "deployment/host.json", host)
        self.start_calls.append(deepcopy(operation))
        return operation

    def stop(self):
        assert self.state / "deployment-start.lock" in self.locks
        operation, host = self.manager._read(), self.read(self.state / "deployment/host.json")
        self.manager.drain()
        terminal = {**host, "status": "stopped", "actualNativeComplete": True}
        binding = {"kind": "owned-stop-metadata-binding", "capturedOwner": {"operation": operation, "host": host},
                   "terminalHost": terminal, "nativeProofSynthesized": False}
        self.stop_mutation(binding)
        checksum = hashlib.sha256(canonical_json(binding)).hexdigest()
        reference = self.reference(self.state / "deployment/history" / ("binding-" + checksum + ".json"), binding)
        stopped = {**operation, "state": "stopped", "stopBinding": reference}
        for pid in (operation["pid"], operation["pid"] + 1, operation["pid"] + 2):
            self.live.pop(pid)
            self.job.discard(pid)
        self.write(self.state / "deployment-operation.json", stopped)
        self.write(self.state / "deployment/host.json", terminal)
        self.stop_calls.append(deepcopy(stopped))
        return stopped

    def environment(self):
        stack = ExitStack()
        for name, value in (("_raw", self.raw), ("_write", self.write), ("_archive", self.archive), ("_archive_bytes", self.archive_bytes),
                            ("_plain", Mock()), ("process_lock", self.lock), ("protected_holds", lambda _state: deepcopy(self.holds)),
                            ("verify_regression", self.current_regression),
                            ("verify_driver", lambda root, **_kwargs: deepcopy(self.drivers[root.name])),
                            ("windows_boot_filetime", lambda: 134360000000000000),
                            ("_windows_command_line", lambda pid: self.argv[pid]),
                            ("current_identity", lambda pid: SimpleNamespace(to_dict=lambda: deepcopy(self.live[pid]))),
                            ("observe_live_identity", self.observe),
                            ("observe_endpoint_owner", lambda _address, _port, pid: {"pid": pid, "uniqueListener": True}),
                            ("observe_departed_identity", lambda value: {"status": "absent", "pid": value["pid"]}),
                            ("has_terminal_lifecycle_proofs", lambda _op, host, *_args: self.terminal_allowed and host.get("actualNativeComplete") is True),
                            ("os", SimpleNamespace(name="nt", getpid=lambda: self.keeper["pid"]))):
            stack.enter_context(patch.object(draft, name, value))
        stack.enter_context(patch.object(Path, "exists", autospec=True, side_effect=self.exists))
        stack.enter_context(patch.object(Path, "glob", autospec=True, side_effect=self.glob))
        stack.enter_context(patch.object(Path, "read_bytes", autospec=True, side_effect=self.raw))
        stack.enter_context(patch.object(Path, "read_text", autospec=True,
            side_effect=lambda path, *args, **kwargs: self.raw(path).decode(kwargs.get("encoding") or "utf-8")))
        stack.enter_context(patch.object(lost_recovery_runtime, "_raw", side_effect=self.raw))
        stack.enter_context(patch.object(lost_recovery_runtime, "_plain"))
        stack.enter_context(patch.object(lost_owner, "_capture", side_effect=lambda *_args: deepcopy(self.plan_document)))
        return stack

    def actor(self, *, recovery_plan=None, upgrade_plan=None):
        return draft.RecoveryKeeper(self.manager, release_path=self.release_ref["path"],
            release_sha256=self.release_ref["sha256"], preflight=self.preflight, publish=self.publish,
            recovery_plan=recovery_plan, upgrade_plan=upgrade_plan)

    def failed_start(self):
        """Boundary fixture: failed typed capture is verified in lost-owner tests."""
        previous_release = self.reference(self.state / "deployment/previous-recovery-release.json", deepcopy(self.release))
        old_keeper = self.identity(55, ("previous-recovery-keeper",))
        original_request = self.archive(self.state, "original-request", {
            "schemaVersion": 1, "kind": draft.REQUEST_KIND, "requestId": "e" * 32,
            "action": "recover", "release": previous_release, "keeper": old_keeper, "runtimeAcceptance": False})
        command = self.reference(self.state / "recovery-lifecycle/command.json", {
            "schemaVersion": 1, "kind": draft.COMMAND_KIND, "status": "unknown", "keeper": old_keeper,
            "release": previous_release, "request": original_request, "runtimeAcceptance": False})
        operation = self.manager._read()
        operation.update(state="failed", reasonCode="host_record_publish_failed")
        self.write(self.state / "deployment-operation.json", operation)
        attempted = self.read(self.state / "deployment/host.json")
        attempted.update(status="running", ready=True)
        host = self.reference(self.state / "deployment/host.tmp", attempted)
        transaction = self.reference(self.state / "recovery-lifecycle/start-transactions" / (operation["operationId"] + ".json"), {
            "schemaVersion": 1, "kind": "formal-recovery-start-transaction", "operationId": operation["operationId"],
            "state": "unknown", "keeper": old_keeper, "release": previous_release, "request": original_request,
            "spawnAttempted": True, "runtimeAcceptance": False, "nativeTerminationProven": False})
        owner = deepcopy(self.plan_document["owner"])
        owner["operation"] = operation
        fence = deepcopy(self.plan_document["lifecycleFence"])
        fence["deployment-operation.json"] = self.sha(self.state / "deployment-operation.json")
        self.plan_document.update(recoveryVariant="failed-own-start", operationId=operation["operationId"],
            owner=owner, lifecycleFence=fence, originalCommand=command, startTransaction=transaction,
            attemptedHost=host, originalRequest=original_request,
            committedHost={"path": str(self.state / "deployment/host.json"), "sha256": self.sha(self.state / "deployment/host.json")})
        artifacts = tuple((name, canonical_json(self.plan_document[name]), self.raw(self.plan_document[name]["path"]))
                          for name in ("originalCommand", "startTransaction", "attemptedHost", "committedHost"))
        self.plan = lost_owner.LostFormalOwnerRecovery(canonical_json(self.plan_document), token=lost_owner._ISSUED,
            variant="failed-own-start", artifacts=artifacts)
        reviewed = self.reference(self.state / "migration-baseline/reviewed-failed-plan.json", self.plan_document)
        self.release.update(expectedPreviousOwner=owner, expectedLifecycleFence=fence,
            failedOwnStart={"plan": reviewed, "originalCommand": command, "startTransaction": transaction, "attemptedHost": host})
        self.release_ref = self.reference(Path(self.release_ref["path"]), self.release)
        args = list(self.argv[123])
        args[-1] = self.release_ref["sha256"]
        self.argv[123] = tuple(args)
        self.keeper = self.identity(123, tuple(args))
        self.live[123] = self.keeper
        self.authorized = None

    def failed_environment(self):
        """Exercise journal integration against a typed-plan API boundary."""
        stack = self.environment()

        def revalidate(plan, manager):
            assert manager is self.manager
            marker = self.release["failedOwnStart"]
            for name in ("startTransaction", "attemptedHost"):
                reference = marker[name]
                if self.sha(reference["path"]) != reference["sha256"]:
                    raise JenkinsError("formal_lost_owner_changed", "Original failed artifact changed")
            command = self.read(marker["originalCommand"]["path"])
            if self.authorized is None:
                if self.sha(marker["originalCommand"]["path"]) != marker["originalCommand"]["sha256"]:
                    raise JenkinsError("formal_lost_owner_changed", "Original unknown command changed")
            else:
                expected = self.authorized
                if (command.get("status") not in {"pending", "start-attempted"}
                        or command.get("keeper") != expected["keeper"] or command.get("release") != expected["release_ref"]
                        or command.get("request") != expected["request"]
                        or command.get("replacesOriginalCommand") != expected["original_archive"]
                        or command.get("failedOwnStartInterruption") != expected["interruption_ref"]):
                    raise JenkinsError("formal_lost_owner_changed", "Unauthorized replacement command")
            return {**plan.document, "recoveryDigest": plan.recovery_digest}

        def authorize(plan, manager, **value):
            revalidate(plan, manager)
            assert self.authorized is None
            for name, field in (("originalCommand", "original_archive"), ("startTransaction", "transaction_archive"),
                                ("attemptedHost", "attempted_host_archive")):
                original, archived = plan.document[name], value[field]
                assert original["sha256"] == archived["sha256"] and self.raw(original["path"]) == self.raw(archived["path"])
            receipt = self.read(value["interruption_ref"]["path"])
            assert receipt["kind"] == "failed-recovery-start-interruption" and receipt["outcome"] == "interrupted"
            assert receipt["nativeProofIssued"] is False and receipt["historicalEofClaimed"] is False
            self.authorized = value
            return lost_owner.LostFormalOwnerRecovery(plan.snapshot, token=lost_owner._ISSUED,
                variant="failed-own-start", artifacts=plan._artifacts, transition=canonical_json(value))

        stack.enter_context(patch.object(lost_owner.LostFormalOwnerRecovery, "revalidate", autospec=True, side_effect=revalidate))
        stack.enter_context(patch.object(lost_owner.LostFormalOwnerRecovery, "authorize_keeper_command", autospec=True,
                                       side_effect=authorize))
        return stack

    def owned_successor(self):
        """Typed-plan integration boundary; native preparation has separate tests."""
        with self.environment():
            previous_actor = self.actor()
            previous_actor.start(self.plan)
            previous_actor.stop()
        previous_descriptor = self.read(previous_actor.descriptor)
        self.previous_release, self.previous_release_ref = deepcopy(self.release), deepcopy(self.release_ref)
        self.previous_target = deepcopy(self.target)
        paths = {"stoppedDescriptor": previous_actor.descriptor, "terminalCommand": previous_actor.command_path,
                 "stoppedOperation": self.state / "deployment-operation.json", "stoppedHost": self.state / "deployment/host.json",
                 "previousSelector": self.state / "deployment/driver.json"}
        artifacts = {name: {"reference": {"path": str(path), "sha256": self.sha(path)}, "raw": self.raw(path)}
                     for name, path in paths.items()}
        stopped_ref = self.archive_bytes(self.state, "stopped-descriptor", artifacts["stoppedDescriptor"]["raw"])
        stop_ref = previous_descriptor["runtimeReceipt"]
        retirement = self.archive(self.state, "keeper-retirement", {
            "schemaVersion": 1, "kind": "owned-recovery-keeper-retirement", "requestId": "d" * 32,
            "keeper": previous_descriptor["keeper"], "release": self.release_ref, "stoppedDescriptor": stopped_ref,
            "ownedStop": stop_ref, "runtimeAcceptance": False, "cargoAcceptance": False, "historicalHoldsReleased": False})
        self.old_target, self.target = self.previous_target, self.driver("c" * 64)
        self.drivers[self.target["driverDigest"]] = self.target
        manifest = self.read(self.previous_target["manifest"])
        for name in manifest["files"]:
            self.files[Path(self.target["root"]) / name] = self.raw(Path(self.previous_target["root"]) / name)
        manifest["driverDigest"] = self.target["driverDigest"]
        self.write(Path(self.target["manifest"]), manifest)
        owner = deepcopy(previous_descriptor["runtime"]["owner"])
        owner["operation"] = self.manager._read()
        fence = {name: self.sha(self.state / name) if self.exists(self.state / name) else None for name in
                 ("deployment-operation.json", "deployment/host.json", "deployment/driver.json", *draft._LEGACY_FILES)}
        preparation = self.reference(self.state / "migration-baseline/owned-preparation.json", {
            "schemaVersion": 1, "state": "prepared-not-selected", "driver": self.target,
            "supportRegression": self.release["regression"], "previousSelectedDriver": self.read(paths["previousSelector"]),
            "previousSelectorSha256": fence["deployment/driver.json"], "protectedHolds": self.holds})
        gui = ROOT / ".jenkins/runtime/python/pythonw.exe"
        self.files[gui] = b"reviewed-gui-python"
        self.release.update(kind=draft.OWNED_UPGRADE_RELEASE_KIND, requestId="d" * 32, targetDriver=self.target,
            actorCodeRoot=self.target["root"], expectedPreviousOwner=owner, expectedLifecycleFence=fence,
            preparation=preparation, ownedUpgrade={"stoppedDescriptor": stopped_ref, "ownedStop": stop_ref,
                                                  "keeperRetirement": retirement},
            runtimeFiles={str(gui): self.sha(gui)}, automaticRestart=False, queueReplayRequested=False)
        self.release_ref = self.reference(self.state / "deployment/owned-upgrade-release.json", self.release)
        value = {"schemaVersion": 1, "kind": owned_upgrade.KIND, "requestId": self.release["requestId"],
            "scope": "execution-runtime", "outcome": "owned-transfer", "owner": owner, "lifecycleFence": fence,
            "previousKeeper": previous_descriptor["keeper"], "previousRelease": self.previous_release_ref,
            "successorRelease": self.release_ref, "targetDriver": self.target,
            **self.release["ownedUpgrade"], "descriptorSha256": artifacts["stoppedDescriptor"]["reference"]["sha256"],
            "terminalCommand": artifacts["terminalCommand"]["reference"],
            "rawAnchors": {name: value["reference"] for name, value in artifacts.items()},
            "operationDigest": digest(self.manager._read()), "hostRecordDigest": digest(self.read(paths["stoppedHost"])),
            "selectedDriverDigest": digest(self.read(paths["previousSelector"])),
            "executionQuarantine": {"protectedHoldsDigest": digest(self.holds)}}
        self.upgrade_plan = owned_upgrade.OwnedRecoveryUpgradePlan(canonical_json(value), owned_upgrade._ISSUED,
            artifacts=tuple((name, canonical_json(item["reference"]), item["raw"]) for name, item in sorted(artifacts.items())))
        args = (*self.argv[123][:-4], "--release-file", self.release_ref["path"], "--release-sha256", self.release_ref["sha256"])
        self.live.pop(123)
        self.job.discard(123)
        self.keeper = self.identity(124, args)
        self.live[124], self.argv[124] = self.keeper, args
        self.job.add(124)
        self.authorized = None

    def owned_environment(self):
        stack = self.environment()
        stack.enter_context(patch.object(draft, "__file__", str(Path(self.target["root"]) / "tools/jenkins/deployment/recovery_keeper.py")))
        stack.enter_context(patch.object(owned_upgrade, "_historical_release",
            side_effect=lambda *_args: (deepcopy(self.previous_release), deepcopy(self.previous_target))))

        def revalidate(plan, manager):
            assert manager is self.manager
            for name, item in plan.original_artifacts.items():
                path = Path(item["reference"]["path"])
                if name == "terminalCommand" and self.authorized is not None:
                    command = self.read(path)
                    expected = self.authorized
                    if (command.get("status") not in {"pending", "start-attempted"} or command.get("keeper") != expected["keeper"]
                            or command.get("release") != expected["release_ref"] or command.get("request") != expected["request"]
                            or command.get("ownedUpgradeDigest") != plan.upgrade_digest or command.get("previousRuntimeReceipt") is not None
                            or command.get("replacesStoppedDescriptor") != expected["descriptor_archive"]
                            or command.get("replacesTerminalCommand") != expected["command_archive"]
                            or command.get("ownedUpgradeHandoff") != expected["handoff_ref"]):
                        raise JenkinsError("owned_recovery_upgrade_changed", "Unauthorized successor command")
                elif self.raw(path) != item["raw"]:
                    raise JenkinsError("owned_recovery_upgrade_changed", "Stopped predecessor changed")
            return {**plan.document, "ownedUpgradeDigest": plan.upgrade_digest}

        def authorize(plan, manager, **value):
            revalidate(plan, manager)
            assert self.authorized is None
            for name, field in (("stoppedDescriptor", "descriptor_archive"), ("terminalCommand", "command_archive")):
                assert self.raw(value[field]["path"]) == plan.original_artifacts[name]["raw"]
            handoff = self.read(value["handoff_ref"]["path"])
            assert handoff["kind"] == "owned-recovery-keeper-handoff" and handoff["outcome"] == "owned-transfer"
            assert handoff["ownedStop"] == plan.document["ownedStop"] and handoff["nativeProofSynthesized"] is False
            self.authorized = value
            return owned_upgrade.OwnedRecoveryUpgradePlan(plan.snapshot, owned_upgrade._ISSUED,
                artifacts=plan._artifacts, transition=canonical_json(value))

        stack.enter_context(patch.object(owned_upgrade.OwnedRecoveryUpgradePlan, "revalidate", autospec=True, side_effect=revalidate))
        stack.enter_context(patch.object(owned_upgrade.OwnedRecoveryUpgradePlan, "authorize_keeper_command", autospec=True,
                                       side_effect=authorize))
        return stack

    def legacy(self):
        return {name: self.files.get(self.state / name) for name in draft._LEGACY_FILES}


class RecoveryKeeperTests(unittest.TestCase):
    def test_same_boot_committed_replacement_archives_real_result_and_consumes_one_capability(self):
        from tools.jenkins.tests.test_lost_recovery_runtime import DepartedRuntime
        scenario = DepartedRuntime().committed()
        management, holds = deepcopy(scenario.management_rows), deepcopy(scenario.holds)
        with scenario.observations() as stack:
            plan = scenario.prepare()
            originals = plan.original_artifacts
            old_result = scenario.files[scenario.result_ref["path"]]
            actor = scenario.actor(stack, plan)
            request = scenario.put(scenario.state / "recovery-lifecycle/history/new-request.json", {
                "schemaVersion": 1, "kind": draft.REQUEST_KIND, "action": "recover", "requestId": scenario.request_id,
                "keeper": actor.keeper, "release": actor.release_ref, "recoveryDigest": plan.recovery_digest,
                "runtimeAcceptance": False})
            replacement = actor._begin(request, plan=plan)
            command = lost_recovery_runtime._document(scenario.files[str(actor.command_path)])
            receipt = lost_recovery_runtime._document(scenario.files[actor.runtime_interruption["receipt"]["path"]])
            self.assertEqual("committed", receipt["originalCommandStatus"])
            for name, value in originals.items():
                archived = receipt["originalArtifacts"][name]["archive"]
                self.assertEqual(value["raw"], scenario.files[archived["path"]])
            archived_command = lost_recovery_runtime._document(
                scenario.files[receipt["originalArtifacts"]["originalCommand"]["archive"]["path"]])
            self.assertEqual("committed", archived_command["status"])
            self.assertEqual(scenario.result_ref, archived_command["result"])
            self.assertEqual(old_result, scenario.files[scenario.result_ref["path"]])
            replacement.revalidate(scenario.manager)
            command["status"] = "start-attempted"
            scenario.put(actor.command_path, command)
            capability = draft.RecoveryKeeperStart(actor, replacement, request,
                scenario.ref(actor.command_path)["sha256"], token=draft._CAPABILITY)
            capability.consume(scenario.manager, replacement)
            with self.assertRaises(JenkinsError):
                capability.consume(scenario.manager, replacement)
            descriptor = {"keeper": actor.keeper, "release": actor.release_ref, "bootFileTime": str(scenario.boot),
                "recoveryDigest": plan.recovery_digest, "lostRecoveryRuntimeInterruption": actor.runtime_interruption["receipt"]}
            scenario.put(actor.descriptor, descriptor)
            lost_recovery_runtime.validate_interruption_receipt(scenario.state, descriptor, actor.release)
            self.assertFalse(lost_recovery_runtime.interrupted_request(scenario.state, scenario.request_ref))
            self.assertEqual(scenario.result_ref, lost_recovery_runtime._document(scenario.files[
                str(scenario.state / "recovery-lifecycle/results" / (scenario.stop_id + ".json"))]))
        self.assertEqual(management, scenario.management_rows)
        self.assertEqual(holds, scenario.holds)

    def test_same_boot_original_committed_result_drift_never_overwrites_command_or_spawns(self):
        from tools.jenkins.tests.test_lost_recovery_runtime import DepartedRuntime
        scenario = DepartedRuntime().committed()
        with scenario.observations() as stack:
            plan = scenario.prepare()
            actor = scenario.actor(stack, plan)
            original_command = scenario.files[str(actor.command_path)]
            scenario.files[scenario.result_ref["path"]] += b" "
            request = scenario.put(scenario.state / "recovery-lifecycle/history/new-request.json", {
                "schemaVersion": 1, "kind": draft.REQUEST_KIND, "action": "recover", "requestId": scenario.request_id,
                "keeper": actor.keeper, "release": actor.release_ref, "recoveryDigest": plan.recovery_digest,
                "runtimeAcceptance": False})
            with self.assertRaises(JenkinsError):
                actor._begin(request, plan=plan)
            self.assertEqual(original_command, scenario.files[str(actor.command_path)])
            self.assertIsNone(actor.runtime)
            self.assertEqual(0, scenario.write_pointer.call_count)

    def test_memory_fixture_isolates_pending_submission_and_pump_from_live_state(self):
        memory = MemoryRuntime()
        directory = memory.state / "recovery-lifecycle"
        unseen = directory / "requests" / ("9" * 32 + ".json")
        def physical_access(*_args, **_kwargs):
            raise AssertionError("MemoryRuntime must never query the physical Jenkins State")
        with patch.object(Path, "read_bytes", physical_access), \
             patch.object(Path, "read_text", physical_access), \
             patch.object(Path, "exists", physical_access), \
             patch.object(Path, "glob", physical_access):
            with memory.environment():
                actor = memory.actor()
                actor.start(memory.plan)
                self.assertFalse(unseen.exists())
                self.assertNotIn(unseen, list((directory / "requests").glob("*.json")))
                descriptor = memory.read(actor.descriptor)
                self.assertEqual(memory.raw(actor.descriptor), actor.descriptor.read_bytes())
                self.assertEqual(memory.raw(actor.descriptor).decode(), actor.descriptor.read_text(encoding="utf-8"))
                self.assertNotIn("lostRecoveryRuntimeInterruption", descriptor)
                reference, request_id = draft._new_request(memory.state, descriptor, memory.sha(actor.descriptor), "stop")
                marker = directory / "requests" / (request_id + ".json")
                self.assertTrue(marker.exists())
                self.assertFalse((directory / "results" / marker.name).exists())
                self.assertFalse(lost_recovery_runtime.interrupted_request(memory.state, reference))
                with self.assertRaises(JenkinsError) as pending:
                    draft._submission_idle(memory.state, directory)
                self.assertEqual("recovery_keeper_command_pending", pending.exception.code)
                self.assertEqual(reference, pending.exception.details["request"])
                actor.pump()
                self.assertTrue((directory / "results" / marker.name).exists())
                draft._submission_idle(memory.state, directory)
                self.assertEqual(1, len(memory.stop_calls))
                self.assertFalse(actor.blocked)

    def test_initial_recovery_stop_and_public_start_use_own_native_authority(self):
        memory = MemoryRuntime()
        before, holds = memory.legacy(), deepcopy(memory.holds)
        with memory.environment():
            actor = memory.actor()
            first = actor.start(memory.plan)
            self.assertEqual(first["status"], "running-acceptance-pending")
            stopped = actor.stop()
            self.assertEqual(stopped["status"], "stopped")
            memory.manager.drain.assert_called_once()
            with patch.object(draft.time, "sleep", side_effect=lambda _seconds: actor.pump()):
                successor = draft.request_recovery_keeper_action(memory.manager, "start", timeout=2)
            self.assertEqual(successor["status"], "running-acceptance-pending")
            self.assertEqual(len(memory.start_calls), 2)
            self.assertIs(memory.capabilities[0].plan, memory.plan)
            self.assertIsNone(memory.capabilities[1].plan)
            self.assertIsNone(actor.stop_packet)
            self.assertEqual(memory.legacy(), before)
            self.assertEqual(memory.holds, holds)
            for reference in (first["runtimeReceipt"], stopped["runtimeReceipt"], successor["runtimeReceipt"]):
                receipt = draft._reference(memory.state, reference)
                self.assertFalse(receipt["runtimeAcceptance"])
                self.assertFalse(receipt["nativeProofSynthesized"])
                self.assertFalse(receipt["historicalHoldsReleased"])
            self.assertFalse(actor.blocked)

    def test_explicit_restart_preserves_original_lineage_and_requires_real_stop(self):
        memory = MemoryRuntime()
        with memory.environment():
            actor = memory.actor()
            actor.start(memory.plan)
            before = memory.legacy()
            with patch.object(draft.time, "sleep", side_effect=lambda _seconds: actor.pump()):
                result = draft.request_recovery_keeper_action(memory.manager, "restart", timeout=2)
            self.assertEqual(result["status"], "running-acceptance-pending")
            self.assertEqual((len(memory.stop_calls), len(memory.start_calls)), (1, 2))
            self.assertEqual(memory.legacy(), before)
            actor.pump()
            self.assertEqual(len(memory.start_calls), 2, "Completed request must not replay")

    def test_json_and_unissued_capabilities_never_authorize_start(self):
        memory = MemoryRuntime()
        with memory.environment():
            actor = memory.actor()
            for plan in (memory.plan.document, None, SimpleNamespace(document=memory.plan.document)):
                with self.subTest(plan=type(plan)), self.assertRaises(JenkinsError): actor.start(plan)
            with self.assertRaises(JenkinsError):
                draft.RecoveryKeeperStart(actor, memory.plan, {}, "a" * 64)
            self.assertEqual(memory.start_calls, [])
            self.assertFalse(actor.command_path.exists())

    def test_consumed_start_capability_is_one_shot_and_cannot_transfer(self):
        memory = MemoryRuntime()
        with memory.environment():
            actor = memory.actor()
            actor.start(memory.plan)
            cap = memory.capabilities[0]
            for manager in (memory.manager, object()):
                with self.subTest(manager=manager), self.assertRaises(JenkinsError): cap.consume(manager, memory.plan)
            with self.assertRaises(JenkinsError): cap.record_spawn(999)

    def test_missing_same_outer_job_child_blocks_original_command_without_retry(self):
        memory = MemoryRuntime()
        memory.child_member = False
        with memory.environment():
            actor = memory.actor()
            with self.assertRaises(JenkinsError): actor.start(memory.plan)
            command = memory.read(actor.command_path)
            self.assertEqual(command["status"], "unknown")
            self.assertTrue(actor.blocked)
            actor.pump()
            with self.assertRaises(JenkinsError): actor.stop()
            self.assertEqual(memory.read(actor.command_path)["request"], command["request"])
            self.assertEqual(len(memory.capabilities), 1)

    def test_false_strict_file_probe_never_constructs_actor_or_creates_supervisor(self):
        memory = MemoryRuntime()
        memory.probe_mutation = lambda value: value["configuration"].update(updateOpenDenied=False)
        with memory.environment(), self.assertRaises(JenkinsError): memory.actor()
        self.assertEqual(memory.start_calls, [])

    def test_noncanonical_check_only_command_cannot_become_live_keeper(self):
        memory = MemoryRuntime()
        memory.argv[123] = (*memory.argv[123], "--check-only")
        memory.live[123] = memory.identity(123, memory.argv[123])
        with memory.environment(), self.assertRaises(JenkinsError): memory.actor()
        self.assertEqual(memory.start_calls, [])

    def test_foreign_stop_binding_blocks_start_and_preserves_unknown_request(self):
        memory = MemoryRuntime()
        memory.stop_mutation = lambda value: value["capturedOwner"]["operation"].update(operationId="foreign")
        with memory.environment():
            actor = memory.actor()
            actor.start(memory.plan)
            with self.assertRaises(JenkinsError): actor.stop()
            self.assertTrue(actor.blocked)
            with self.assertRaises(JenkinsError): draft.request_recovery_keeper_action(memory.manager, "start", timeout=1)
            self.assertEqual(len(memory.start_calls), 1)
            self.assertEqual(memory.read(actor.command_path)["status"], "unknown")

    def test_missing_native_terminal_proof_rejects_public_start_before_new_request(self):
        memory = MemoryRuntime()
        with memory.environment():
            actor = memory.actor()
            actor.start(memory.plan)
            actor.stop()
            memory.terminal_allowed = False
            before = dict(memory.files)
            with self.assertRaises(JenkinsError): draft.request_recovery_keeper_action(memory.manager, "start", timeout=1)
            self.assertEqual(memory.files, before)
            self.assertEqual(len(memory.start_calls), 1)

    def test_timeout_reuses_original_pending_reference_without_new_nonce(self):
        memory = MemoryRuntime()
        with memory.environment():
            actor = memory.actor()
            actor.start(memory.plan)
            with patch.object(draft.uuid, "uuid4", return_value=SimpleNamespace(hex="1" * 32)) as nonce:
                with patch.object(draft.time, "monotonic", side_effect=(0, 1)), self.assertRaises(JenkinsError) as first:
                    draft.request_recovery_keeper_action(memory.manager, "stop", timeout=.1)
                with self.assertRaises(JenkinsError) as second:
                    draft.request_recovery_keeper_action(memory.manager, "stop", timeout=.1)
                self.assertEqual(first.exception.code, "recovery_keeper_command_pending")
                self.assertEqual(first.exception.details["request"], second.exception.details["request"])
                nonce.assert_called_once()
            self.assertEqual(memory.stop_calls, [])

    def test_release_operator_and_preparation_pins_are_fail_closed(self):
        for mutation in ("operator", "old-selector", "target"):
            with self.subTest(mutation=mutation):
                memory = MemoryRuntime()
                release = deepcopy(memory.release)
                if mutation == "operator": release["operatorFiles"].pop(str(ROOT / "tools/jenkins/deployment/keeper_control.py"))
                elif mutation == "old-selector":
                    preparation = memory.read(release["preparation"]["path"])
                    preparation.pop("previousSelectorSha256")
                    release["preparation"] = memory.reference(Path(release["preparation"]["path"]), preparation)
                else: release["targetDriver"]["root"] = str(ROOT / "unreviewed-driver")
                reference = memory.reference(Path(memory.release_ref["path"]), release)
                with memory.environment(), self.assertRaises(JenkinsError): draft._release(memory.manager, reference)

    def test_changed_physical_runtime_receipt_cannot_authorize_public_command(self):
        memory = MemoryRuntime()
        with memory.environment():
            actor = memory.actor()
            actor.start(memory.plan)
            memory.files[Path(actor.runtime_receipt["path"])] += b" "
            before = len(memory.files)
            with self.assertRaises(JenkinsError): draft.request_recovery_keeper_action(memory.manager, "stop", timeout=1)
            self.assertEqual(len(memory.files), before)
            self.assertEqual(memory.stop_calls, [])

    def test_changed_historical_holds_block_stop_without_release_or_native_action(self):
        memory = MemoryRuntime()
        with memory.environment():
            actor = memory.actor()
            actor.start(memory.plan)
            memory.holds[0]["payload"]["status"] = "released"
            before = memory.legacy()
            with self.assertRaises(JenkinsError): actor.stop()
            self.assertEqual(memory.stop_calls, [])
            self.assertEqual(memory.legacy(), before)

    def test_unrelated_source_and_inventory_updates_allow_own_stop_and_start(self):
        memory = MemoryRuntime()
        with memory.environment():
            actor = memory.actor()
            actor.start(memory.plan)
            ordinary_source = ROOT / "tools/jenkins/workflow/handler.py"
            memory.files[ordinary_source] += b"\nM6 maintenance change"
            new_test = ROOT / "tools/jenkins/tests/test_future_gc.py"
            memory.files[new_test] = b"new support test source"
            memory.source_paths.add(new_test)
            stopped = actor.stop()
            self.assertEqual(stopped["status"], "stopped")
            with patch.object(draft.time, "sleep", side_effect=lambda _seconds: actor.pump()):
                restarted = draft.request_recovery_keeper_action(memory.manager, "start", timeout=2)
            self.assertEqual(restarted["status"], "running-acceptance-pending")
            self.assertEqual(len(memory.start_calls), 2)
            with self.assertRaises(JenkinsError) as error: draft._release(memory.manager, memory.release_ref)
            self.assertEqual(error.exception.code, "execution_transition_source_changed")

    def test_canonical_admission_requires_full_current_source_inventory(self):
        memory = MemoryRuntime()
        memory.files[ROOT / "tools/jenkins/workflow/handler.py"] += b"changed-before-first-start"
        with memory.environment(), self.assertRaises(JenkinsError) as error:
            draft._release(memory.manager, memory.release_ref)
        self.assertEqual(error.exception.code, "execution_transition_source_changed")
        self.assertEqual(memory.start_calls, [])

    def test_exact_sealed_actor_constructs_and_selects_target_after_unrelated_checkout_edit(self):
        memory = MemoryRuntime()
        memory.release["actorCodeRoot"] = memory.target["root"]
        memory.release_ref = memory.reference(Path(memory.release_ref["path"]), memory.release)
        args = (*memory.argv[123][:-1], memory.release_ref["sha256"])
        memory.keeper = memory.identity(123, args)
        memory.live[123], memory.argv[123] = memory.keeper, args
        module = Path(memory.target["root"]) / "tools/jenkins/deployment/recovery_keeper.py"
        with memory.environment(), patch.object(draft, "__file__", str(module)):
            memory.files[ROOT / "tools/jenkins/workflow/handler.py"] += b"ordinary-checkout-maintenance"
            with self.assertRaises(JenkinsError) as error:
                draft._release(memory.manager, memory.release_ref)
            self.assertEqual("execution_transition_source_changed", error.exception.code)
            actor = memory.actor()
            result = actor.start(memory.plan)
            self.assertEqual("running-acceptance-pending", result["status"])
            self.assertEqual(memory.target["driverDigest"],
                             memory.read(memory.state / "deployment/driver.json")["driverDigest"])
            self.assertFalse(result["runtimeAcceptance"])
            self.assertEqual(1, len(memory.start_calls))

    def test_historical_log_candidate_and_operator_changes_block_existing_lifecycle(self):
        for mutation in ("log", "candidate", "operator"):
            with self.subTest(mutation=mutation):
                memory = MemoryRuntime()
                with memory.environment():
                    actor = memory.actor()
                    actor.start(memory.plan)
                    if mutation == "log": memory.files[memory.log] += b"changed"
                    elif mutation == "operator": memory.files[ROOT / "tools/jenkins/deployment/start_transaction.py"] += b"changed"
                    else:
                        manifest = memory.read(memory.target["manifest"])
                        manifest["files"]["tools/jenkins/workflow/handler.py"]["sha256"] = "c" * 64
                        memory.write(Path(memory.target["manifest"]), manifest)
                    before = len(memory.files)
                    with self.assertRaises(JenkinsError): actor.stop()
                    with self.assertRaises(JenkinsError): draft.request_recovery_keeper_action(memory.manager, "stop", timeout=1)
                    self.assertEqual(memory.stop_calls, [])
                    self.assertEqual(len(memory.files), before)

    def test_failed_own_start_archives_unknown_command_transaction_and_attempted_host_before_replacement(self):
        memory = MemoryRuntime()
        memory.failed_start()
        original = {name: memory.raw(memory.plan.document[name]["path"])
                    for name in ("originalCommand", "startTransaction", "attemptedHost", "committedHost")}
        before, holds = memory.legacy(), deepcopy(memory.holds)
        with memory.failed_environment():
            actor = memory.actor(recovery_plan=memory.plan)
            result = actor.start(memory.plan)
            self.assertEqual(result["status"], "running-acceptance-pending")
            command = memory.read(actor.command_path)
            interruption = draft._reference(memory.state, command["failedOwnStartInterruption"])
            self.assertEqual(memory.read(command["replacesOriginalCommand"]["path"])["status"], "unknown")
            for name, raw in original.items():
                archive = interruption["originalArtifacts"][name]["archive"]
                self.assertEqual(memory.raw(archive["path"]), raw)
                self.assertEqual(archive["sha256"], hashlib.sha256(raw).hexdigest())
            self.assertEqual(interruption["outcome"], "interrupted")
            self.assertFalse(interruption["nativeProofIssued"])
            self.assertFalse(interruption["nativeTerminationProven"])
            self.assertFalse(interruption["historicalEofClaimed"])
            self.assertFalse(interruption["runtimeAcceptance"])
            self.assertEqual(memory.legacy(), before)
            self.assertEqual(memory.holds, holds)
            self.assertIsNot(memory.capabilities[0].plan, memory.plan, "Only the private transitioned plan reaches the manager")
            self.assertIs(memory.capabilities[0].plan, actor.recovery_plan)
            with self.assertRaises(JenkinsError): actor.start(memory.plan)
            self.assertEqual(len(memory.start_calls), 1)

    def test_failed_start_json_unreviewed_or_different_plan_cannot_override_original_unknown(self):
        for boundary in ("json", "missing-plan", "unreviewed", "different-plan"):
            with self.subTest(boundary=boundary):
                memory = MemoryRuntime()
                memory.failed_start()
                plan = memory.plan
                if boundary == "json": plan = plan.document
                elif boundary == "missing-plan": plan = None
                elif boundary == "unreviewed":
                    memory.release.pop("failedOwnStart")
                    memory.release_ref = memory.reference(Path(memory.release_ref["path"]), memory.release)
                else:
                    changed = deepcopy(plan.document)
                    changed["originalCommand"]["sha256"] = "d" * 64
                    plan = lost_owner.LostFormalOwnerRecovery(canonical_json(changed), token=lost_owner._ISSUED)
                before = dict(memory.files)
                with memory.failed_environment(), self.assertRaises(JenkinsError): memory.actor(recovery_plan=plan)
                self.assertEqual(memory.files, before)
                self.assertEqual(memory.start_calls, [])

    def test_changed_original_unknown_after_constructor_remains_unmodified_and_no_spawn_occurs(self):
        memory = MemoryRuntime()
        memory.failed_start()
        with memory.failed_environment():
            actor = memory.actor(recovery_plan=memory.plan)
            command = memory.read(actor.command_path)
            command["reasonCode"] = "changed-after-review"
            memory.write(actor.command_path, command)
            raw = memory.raw(actor.command_path)
            with self.assertRaises(JenkinsError): actor.start(memory.plan)
            self.assertEqual(memory.raw(actor.command_path), raw)
            self.assertIsNone(memory.authorized)
            self.assertEqual(memory.start_calls, [])

    def test_incomplete_failed_artifact_archive_never_overwrites_unknown_command(self):
        memory = MemoryRuntime()
        memory.failed_start()
        with memory.failed_environment():
            actor = memory.actor(recovery_plan=memory.plan)
            original = memory.raw(actor.command_path)
            with patch.object(draft, "_archive_bytes", side_effect=JenkinsError("archive_incomplete", "not durable")):
                with self.assertRaises(JenkinsError): actor.start(memory.plan)
            self.assertEqual(memory.raw(actor.command_path), original)
            self.assertTrue(actor.blocked)
            self.assertIsNone(memory.authorized)
            self.assertEqual(memory.start_calls, [])

    def test_replacement_failure_preserves_original_unknown_archive_and_blocks_same_actor_retry(self):
        memory = MemoryRuntime()
        memory.failed_start()
        memory.child_member = False
        original = memory.raw(memory.plan.document["originalCommand"]["path"])
        with memory.failed_environment():
            actor = memory.actor(recovery_plan=memory.plan)
            with self.assertRaises(JenkinsError): actor.start(memory.plan)
            command = memory.read(actor.command_path)
            self.assertEqual(command["status"], "unknown")
            self.assertEqual(memory.raw(command["replacesOriginalCommand"]["path"]), original)
            self.assertTrue(actor.blocked)
            with self.assertRaises(JenkinsError): actor.start(memory.plan)
            actor.pump()
            self.assertEqual(len(memory.capabilities), 1)
            self.assertEqual(memory.read(actor.command_path)["request"], command["request"])

    def test_owned_upgrade_archives_genuine_terminal_owner_and_public_restart_keeps_transfer_lineage(self):
        memory = MemoryRuntime()
        memory.owned_successor()
        before, holds = memory.legacy(), deepcopy(memory.holds)
        originals = memory.upgrade_plan.original_artifacts
        with memory.owned_environment():
            actor = memory.actor(upgrade_plan=memory.upgrade_plan)
            result = actor.start_upgrade(memory.upgrade_plan)
            command = memory.read(actor.command_path)
            self.assertEqual(result["status"], "running-acceptance-pending")
            self.assertIsNone(command["previousRuntimeReceipt"])
            self.assertNotIn("replacesOriginalCommand", command)
            self.assertNotIn("failedOwnStartInterruption", command)
            handoff = draft._reference(memory.state, command["ownedUpgradeHandoff"])
            self.assertEqual(handoff["outcome"], "owned-transfer")
            self.assertEqual(handoff["ownedStop"], memory.release["ownedUpgrade"]["ownedStop"])
            for name, original in originals.items():
                archive = handoff["originalArtifacts"][name]["archive"]
                self.assertEqual(memory.raw(archive["path"]), original["raw"])
                self.assertEqual(archive["sha256"], original["reference"]["sha256"])
            descriptor = memory.read(actor.descriptor)
            self.assertEqual(draft._runtime_receipt(memory.manager, descriptor, memory.release, memory.target)["kind"],
                             "recovery-keeper-runtime")
            actor.stop()
            with patch.object(draft.time, "sleep", side_effect=lambda _seconds: actor.pump()):
                restarted = draft.request_recovery_keeper_action(memory.manager, "start", timeout=2)
            self.assertEqual(restarted["status"], "running-acceptance-pending")
            self.assertEqual(memory.legacy(), before)
            self.assertEqual(memory.holds, holds)
            self.assertIsNone(memory.capabilities[-1].plan)
            self.assertIsNot(memory.capabilities[-2].plan, memory.upgrade_plan)
            receipt = memory.read(restarted["runtimeReceipt"]["path"])
            self.assertEqual(receipt["ownedUpgradeHandoff"], command["ownedUpgradeHandoff"])
            self.assertFalse(receipt["runtimeAcceptance"])
            self.assertFalse(receipt["nativeProofSynthesized"])

    def test_owned_json_missing_or_wrong_release_cannot_adopt_stopped_descriptor(self):
        for boundary in ("json", "missing-plan", "lost-release", "changed-terminal-command"):
            with self.subTest(boundary=boundary):
                memory = MemoryRuntime()
                memory.owned_successor()
                plan = memory.upgrade_plan
                if boundary == "json": plan = plan.document
                elif boundary == "missing-plan": plan = None
                elif boundary == "lost-release":
                    memory.release["kind"] = draft.RELEASE_KIND
                    memory.release_ref = memory.reference(Path(memory.release_ref["path"]), memory.release)
                else: memory.files[memory.state / "recovery-lifecycle/command.json"] += b" "
                before = dict(memory.files)
                with memory.owned_environment(), self.assertRaises(JenkinsError): memory.actor(upgrade_plan=plan)
                self.assertEqual(memory.files, before)

    def test_owned_archive_failure_preserves_stopped_descriptor_and_command_before_any_spawn(self):
        memory = MemoryRuntime()
        memory.owned_successor()
        with memory.owned_environment():
            actor = memory.actor(upgrade_plan=memory.upgrade_plan)
            descriptor, command = memory.raw(actor.descriptor), memory.raw(actor.command_path)
            with patch.object(draft, "_archive_bytes", side_effect=JenkinsError("archive_incomplete", "not durable")):
                with self.assertRaises(JenkinsError): actor.start_upgrade(memory.upgrade_plan)
            self.assertEqual(memory.raw(actor.descriptor), descriptor)
            self.assertEqual(memory.raw(actor.command_path), command)
            self.assertIsNone(memory.authorized)
            self.assertTrue(actor.blocked)
            self.assertEqual(len(memory.start_calls), 1)

    def test_owned_failed_spawn_blocks_retry_and_preserves_old_native_stop_issuance(self):
        memory = MemoryRuntime()
        memory.owned_successor()
        memory.child_member = False
        with memory.owned_environment():
            actor = memory.actor(upgrade_plan=memory.upgrade_plan)
            previous = memory.raw(actor.descriptor)
            with self.assertRaises(JenkinsError): actor.start_upgrade(memory.upgrade_plan)
            command = memory.read(actor.command_path)
            self.assertEqual(command["status"], "unknown")
            self.assertEqual(memory.raw(actor.descriptor), previous)
            self.assertEqual(memory.read(command["replacesTerminalCommand"]["path"])["status"], "stopped")
            self.assertTrue(actor.blocked)
            with self.assertRaises(JenkinsError): actor.start_upgrade(memory.upgrade_plan)
            actor.pump()
            self.assertEqual(memory.read(actor.command_path)["request"], command["request"])

    def test_sealed_actor_operator_updates_allow_stop_but_snapshot_log_and_stable_entry_changes_block(self):
        for boundary in ("canonical-operator", "snapshot-operator", "historical-log", "stable-desktop-entry"):
            with self.subTest(boundary=boundary):
                memory = MemoryRuntime()
                memory.owned_successor()
                with memory.owned_environment():
                    actor = memory.actor(upgrade_plan=memory.upgrade_plan)
                    actor.start_upgrade(memory.upgrade_plan)
                    if boundary == "historical-log": changed = memory.log
                    elif boundary == "stable-desktop-entry": changed = ROOT / "tools/dev/jenkins_desktop_tray.py"
                    else:
                        relative = Path("tools/jenkins/deployment/start_transaction.py")
                        changed = (ROOT if boundary == "canonical-operator" else Path(memory.target["root"])) / relative
                    memory.files[changed] += b"changed"
                    if boundary == "canonical-operator":
                        self.assertEqual(actor.stop()["status"], "stopped")
                        with patch.object(draft.time, "sleep", side_effect=lambda _seconds: actor.pump()):
                            self.assertEqual(draft.request_recovery_keeper_action(memory.manager, "start", timeout=2)["status"],
                                             "running-acceptance-pending")
                        with self.assertRaises(JenkinsError): draft._release(memory.manager, memory.release_ref)
                    else:
                        stop_count = len(memory.stop_calls)
                        with self.assertRaises(JenkinsError): actor.stop()
                        self.assertEqual(len(memory.stop_calls), stop_count)

    def test_owned_descriptor_commit_requires_exact_original_compare_exchange(self):
        memory = MemoryRuntime()
        memory.owned_successor()
        with memory.owned_environment():
            actor = memory.actor(upgrade_plan=memory.upgrade_plan)
            original = actor._start
            def changed_after_start(plan, request):
                result = original(plan, request)
                memory.files[actor.descriptor] += b" "
                return result
            with patch.object(actor, "_start", side_effect=changed_after_start), self.assertRaises(JenkinsError):
                actor.start_upgrade(memory.upgrade_plan)
            self.assertTrue(actor.blocked)
            self.assertEqual(memory.read(actor.descriptor)["keeper"], memory.upgrade_plan.document["previousKeeper"])

    def test_manager_rejects_untyped_owned_authority_before_reading_or_writing_state(self):
        from tools.jenkins.deployment.manager import DeploymentManager
        memory = MemoryRuntime()
        memory.owned_successor()
        manager = DeploymentManager.__new__(DeploymentManager)
        manager._read = Mock(side_effect=AssertionError("No state read before exact-type rejection"))
        for value in (memory.upgrade_plan.document, SimpleNamespace(document=memory.upgrade_plan.document)):
            with self.subTest(value=type(value)), self.assertRaises(JenkinsError):
                manager._start_locked(memory.state / "deployment-start.lock", owned_upgrade_start=value,
                                      recovery_keeper_start=SimpleNamespace(consume=Mock()))
        manager._read.assert_not_called()


class RecoveryEntryBootstrapTests(unittest.TestCase):
    def entry(self):
        spec = importlib.util.spec_from_file_location("reviewed_recovery_entry_tests",
            ROOT / "tools/dev/jenkins_recovery_keeper.py")
        module = importlib.util.module_from_spec(spec)
        with patch("sys.dont_write_bytecode", True):
            spec.loader.exec_module(module)
        return module

    def snapshot(self, entry):
        names = {"tools/__init__.py", "tools/jenkins/contracts/__init__.py", "tools/jenkins/deployment/recovery_keeper.py",
                 "tools/jenkins/deployment/manager.py", "tools/jenkins/deployment/owned_upgrade.py", "tools/jenkins/tray/__main__.py",
                 "launcher.py", "pipeline/steps.groovy"}
        contents = {name: ("physical-passed-" + name).encode() for name in names}
        files = {name: {"bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()} for name, raw in contents.items()}
        checksum = digest({"schemaVersion": 1, "files": files})
        target = MemoryRuntime.driver(checksum)
        root = Path(target["root"])
        physical = {root / name: raw for name, raw in contents.items()}
        physical[Path(target["manifest"])] = canonical_json({"schemaVersion": 1, "driverDigest": checksum, "files": files})
        release = {"kind": draft.OWNED_UPGRADE_RELEASE_KIND, "targetDriver": target, "actorCodeRoot": str(root)}
        return release, physical

    def environment(self, entry, physical):
        stack = ExitStack()
        stack.enter_context(patch.object(entry, "_bootstrap_raw", side_effect=lambda path: physical[Path(path)]))
        stack.enter_context(patch.object(entry, "_bootstrap_plain", side_effect=lambda path: Path(path)))
        stack.enter_context(patch.object(Path, "rglob", autospec=True,
            side_effect=lambda root, _pattern: [path for path in physical if Path(root) in path.parents]))
        stack.enter_context(patch.object(Path, "is_file", autospec=True, side_effect=lambda path: Path(path) in physical))
        return stack

    def test_bootstrap_verifies_all_physical_bytes_and_rejects_undeclared_files_before_import(self):
        entry = self.entry()
        for boundary in ("complete", "changed-source", "undeclared-file"):
            with self.subTest(boundary=boundary):
                release, physical = self.snapshot(entry)
                root = Path(release["targetDriver"]["root"])
                if boundary == "changed-source": physical[root / "tools/jenkins/deployment/recovery_keeper.py"] += b"changed"
                elif boundary == "undeclared-file": physical[root / "tools/jenkins/unreviewed.py"] = b"extra"
                with self.environment(entry, physical):
                    if boundary == "complete": self.assertEqual(entry._bootstrap_snapshot(release), root)
                    else:
                        with self.assertRaises(ValueError): entry._bootstrap_snapshot(release)

    def test_bootstrap_rejects_an_existing_canonical_tools_namespace_without_importing_target(self):
        entry = self.entry()
        release, physical = self.snapshot(entry)
        before = list(entry.sys.path)
        with self.environment(entry, physical), patch.dict(entry.sys.modules, {"tools": SimpleNamespace(__file__=str(ROOT / "tools/__init__.py"))}):
            with patch("builtins.__import__", side_effect=AssertionError("A mixed actor must never import")):
                with self.assertRaises(ValueError): entry._load_runtime(release, {"path": "reviewed", "sha256": "a" * 64})
        self.assertEqual(entry.sys.path, before)

    def test_owned_keeper_entry_starts_typed_actor_and_never_creates_private_ui(self):
        entry = self.entry()
        memory = MemoryRuntime()
        memory.owned_successor()
        baseline = SimpleNamespace(restriction_preflight=memory.preflight, save=Mock())
        actor = SimpleNamespace(keeper=memory.keeper, release_ref=memory.release_ref,
            start_upgrade=Mock(return_value={"status": "running-acceptance-pending"}), start=Mock(), stop=Mock(),
            pump=Mock(side_effect=RuntimeError("bounded pure entry fixture")))
        globals_to_bind = {"JenkinsError": JenkinsError, "_plain": Mock(), "DeploymentManager": Mock(return_value=memory.manager),
                           "resolve_paths": lambda _spec: memory.manager.paths, "load_spec": lambda _path: memory.manager.spec,
                           "process_lock": memory.lock}
        with memory.environment(), ExitStack() as stack:
            for name, value in globals_to_bind.items():
                stack.enter_context(patch.object(entry, name, value, create=True))
            stack.enter_context(patch.object(entry, "_bootstrap_raw", side_effect=memory.raw))
            stack.enter_context(patch.object(entry, "_load_runtime", return_value=Path(memory.target["root"])))
            stack.enter_context(patch.object(entry, "reviewed_preflight", return_value=baseline))
            stack.enter_context(patch.object(entry, "verify_sandbox_host", return_value={"fixture": True}))
            stack.enter_context(patch.object(entry.signal, "signal"))
            stack.enter_context(patch.object(entry, "print", create=True))
            stack.enter_context(patch.object(Path, "mkdir"))
            stack.enter_context(patch.object(draft, "_release", return_value=(memory.release, memory.target)))
            stack.enter_context(patch.object(draft, "RecoveryKeeper", return_value=actor))
            stack.enter_context(patch.object(owned_upgrade, "prepare_owned_upgrade_start", return_value=memory.upgrade_plan))
            popen = stack.enter_context(patch.object(entry.subprocess, "Popen",
                side_effect=AssertionError("Owned keeper must never create private tray UI")))
            result = entry.main(["--config-sha256", memory.release["config"]["sha256"], "--release-file", memory.release_ref["path"],
                                 "--release-sha256", memory.release_ref["sha256"]])
            self.assertEqual(result, 2, "The pure fixture ends on its first actor pump")
            actor.start_upgrade.assert_called_once_with(memory.upgrade_plan)
            actor.start.assert_not_called()
            popen.assert_not_called()


if __name__ == "__main__":
    unittest.main()

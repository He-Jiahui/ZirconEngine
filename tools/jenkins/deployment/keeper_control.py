"""Reviewed tray requests executed by a retained official strict keeper.

Only explicit requests run; unknown outcomes require proof recovery.
"""
from __future__ import annotations

import base64
from contextlib import nullcontext
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import uuid

from tools.jenkins.contracts import JenkinsError, canonical_json, digest
from tools.jenkins.state.locks import process_lock
from tools.jenkins.state.atomic import replace_file
from .driver import _plain, verify_driver
from .execution_activation import protected_holds
from .live_observations import observe_live_identity, observe_endpoint_owner, _windows_command_line, _digest_args

PROTOCOL_FILE = ".jenkins/state/migration-baseline/formal_runtime_lineage_protocol_v2.py"
PROTOCOL_SHA256 = "795e1a8bf9b714b83ed76f455596c8a14e1e0a544c1e8f05b1bdf68dacbd2005"
COMMAND_KIND = "formal-tray-strict-keeper-command"
TERMINAL_COMMANDS = {"committed", "stopped"}


def _require(value, message):
    if not value:
        raise JenkinsError("formal_keeper_evidence_unproven", message)


def _raw(path):
    path = Path(path).absolute(); _plain(path)
    return path.read_bytes()


def _sha(path):
    return hashlib.sha256(_raw(path)).hexdigest()


def _read(path):
    value = json.loads(_raw(path).decode("utf-8"))
    _require(isinstance(value, dict), "Typed control document required")
    return value


def _write(path, value, *, expected_sha=None):
    return _write_raw(path, canonical_json(value) + b"\n", expected_sha=expected_sha)


def _write_raw(path, raw, *, expected_sha=None):
    path = Path(path).absolute(); _plain(path)
    if expected_sha is not None:
        _require(_sha(path) == expected_sha, "Control pointer changed before commit")
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + ".pending-" + uuid.uuid4().hex); _plain(temporary)
    with temporary.open("xb") as stream:
        stream.write(raw); stream.flush(); os.fsync(stream.fileno())
    def unchanged():
        if expected_sha is not None:
            _require(_sha(path) == expected_sha, "Control pointer changed during commit")
    replace_file(temporary, path, before_replace=unchanged)
    _require(_raw(path) == raw, "Control pointer commit is unknown")


def _archive(state, prefix, raw):
    checksum = hashlib.sha256(raw).hexdigest()
    path = state / "deployment/history" / (prefix + "-" + checksum + ".json"); _plain(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        _require(_raw(path) == raw, "Immutable lifecycle evidence conflicts")
    else:
        with path.open("xb") as stream:
            stream.write(raw); stream.flush(); os.fsync(stream.fileno())
    _require(_sha(path) == checksum, "Immutable lifecycle evidence write is unknown")
    return {"path": str(path), "sha256": checksum}


def _document_archive(state, prefix, value):
    return _archive(state, prefix, canonical_json(value) + b"\n")


_PROTOCOL_CACHE = {}


def _protocol(repo):
    path = repo / PROTOCOL_FILE
    raw = _raw(path)
    _require(hashlib.sha256(raw).hexdigest() == PROTOCOL_SHA256, "Reviewed pure lifecycle protocol changed")
    # Reuse only our own same-byte compilation, never sys.modules or pycache.
    name = "jenkins_formal_runtime_lineage_" + PROTOCOL_SHA256
    cached = _PROTOCOL_CACHE.get(PROTOCOL_SHA256)
    if cached is not None:
        _require(cached[0] == raw, "Cached protocol did not originate from these exact bytes")
        sys.modules[name] = cached[1]
        return cached[1]
    module = importlib.util.module_from_spec(importlib.util.spec_from_loader(name, loader=None))
    module.__file__ = str(path); sys.modules[name] = module
    exec(compile(raw, str(path), "exec"), module.__dict__)
    _PROTOCOL_CACHE[PROTOCOL_SHA256] = (raw, module)
    return module


def _artifact(selection, reference):
    import re
    protocol = _protocol(selection.repo)
    _require(isinstance(reference, dict) and set(reference) == {"path", "sha256"}, "Exact immutable reference required")
    path = Path(reference["path"]).absolute()
    _require(str(path) == reference["path"] and ".." not in path.parts, "Exact canonical evidence path required")
    regular = path.parent in {selection.state / "deployment/history", selection.state / "migration-baseline"}
    deployed = (path.parent == selection.state / "deployment"
                and re.fullmatch(r"usable-upgrade-([0-9a-f]{16})-([1-9][0-9]*)\.json", path.name))
    _require(regular or deployed, "Lifecycle evidence must retain a reviewed immutable namespace")
    artifact = protocol.Artifact(reference["path"], reference["sha256"], _raw(path))
    if deployed:
        doc = artifact.document
        _require(type(doc.get("schemaVersion")) is int and doc["schemaVersion"] == 1
                 and doc.get("kind") == "desktop-execution-upgrade-release" and doc.get("repository") == str(selection.repo)
                 and isinstance(doc.get("candidateDriverDigest"), str) and len(doc["candidateDriverDigest"]) == 64
                 and all(c in "0123456789abcdef" for c in doc["candidateDriverDigest"])
                 and deployed.group(1) == doc["candidateDriverDigest"][:16]
                 and doc.get("runtimeAcceptance") is False and doc.get("nativeExecutionHoldsReleaseAllowed") is False,
                 "Canonical release path must bind its real candidate and limited authority")
    return artifact


def _full_identity(pid):
    from .manager import identity
    value = identity(pid)
    _require(isinstance(value, dict), "Native process identity is unavailable")
    command = _windows_command_line(pid)
    return {"pid": value["pid"], "creationTime": str(value["creationTime"]),
            "executable": value["executable"], "commandDigest": _digest_args(command)}


def _reviewed_code(selection, release):
    candidate = _read(selection.selector)
    verified = verify_driver(Path(candidate["root"]), expected_digest=candidate["driverDigest"])
    manifest = _read(verified["manifest"])
    for name, entry in manifest["files"].items():
        if name.startswith("tools/jenkins/"):
            actual = selection.repo / name
        elif name.startswith("pipeline/"):
            actual = selection.repo / ".jenkins" / name
        else:
            continue
        _require(_sha(actual) == entry["sha256"], "Current support source differs from reviewed sealed driver")
    _require(_sha(selection.repo / PROTOCOL_FILE) == PROTOCOL_SHA256, "Lifecycle protocol changed")
    for name in (".jenkins/state/migration-baseline/jenkins_upgrade_desktop_holder.py", ".jenkins/state/migration-baseline/select_execution_upgrade.py"):
        _require(_sha(selection.repo / name) == release["operatorFiles"][str(selection.repo / name)], "Keeper/operator bytes differ from original reviewed release")


def _reviewed_keeper_argv(repo, keeper, release_ref, config_sha):
    args = [str(repo / ".jenkins/runtime/python/python.exe"), "-B",
            str(repo / ".jenkins/state/migration-baseline/jenkins_upgrade_desktop_holder.py"),
            "--config-sha256", config_sha, "--release-file", release_ref["path"], "--release-sha256", release_ref["sha256"]]
    observed = _windows_command_line(keeper["pid"])
    _require(list(observed) in (args, [*args, "--no-tray"]) and _digest_args(observed) == keeper["commandDigest"],
             "Only the exact reviewed keeper command may create service children")


def assert_no_unresolved_keeper_command(manager):
    """Recognized by every public/CLI start, even after activation restoration."""
    path = manager.paths.state / "deployment/formal-lifecycle-command.json"
    if path.exists():
        doc = _read(path)
        _require(doc.get("kind") == COMMAND_KIND and doc.get("status") in TERMINAL_COMMANDS,
                 "Preserve and explicitly reconcile the unresolved keeper command")


def assert_keeper_managed_start(manager):
    """A public start cannot create a same-driver successor outside its keeper."""
    recovery = manager.paths.state / "formal-lifecycle/recovery-keeper.json"
    if recovery.exists():
        raise JenkinsError("formal_keeper_request_required", "Start this runtime through its retained strict recovery keeper")
    descriptor = manager.paths.state / "formal-lifecycle/keeper.json"
    if not descriptor.exists():
        return
    doc = _read(descriptor)
    _require(doc.get("kind") == "formal-tray-strict-keeper-descriptor", "Keeper descriptor is unknown")
    selected = _read(manager.paths.state / "deployment/driver.json")
    binding = doc.get("anchor", {}).get("binding", {})
    if (binding.get("driverDigest") == selected.get("driverDigest")
            and binding.get("driverRoot") == selected.get("root")):
        raise JenkinsError("formal_keeper_request_required", "Start this selected runtime through the reviewed strict keeper")


class _OwnedKeeperStartCapability:
    """One in-memory start by the exact actor; not a serializable permission."""
    def __init__(self, actor, request, guard_sha, command_sha):
        self.actor, self.request = actor, request
        self.guard_sha, self.command_sha = guard_sha, command_sha
        self.used = False; self.spawn = None

    def consume(self, manager):
        _require(type(self) is _OwnedKeeperStartCapability and not self.used and manager is self.actor.manager,
                 "Internal keeper start capability cannot be replayed or transferred")
        actor = self.actor
        _require(os.getpid() == actor.keeper["pid"] and _full_identity(os.getpid()) == actor.keeper,
                 "Only the retained reviewed keeper may start the supervisor")
        _reviewed_keeper_argv(actor.selection.repo, actor.keeper, actor.anchor.value["binding"]["originalRelease"], actor.config_sha)
        _reviewed_code(actor.selection, actor.release)
        fresh = actor.preflight(actor.selection.repo, actor.config_sha)
        _require(fresh["configuration"]["updateOpenDenied"] is True
                 and fresh["configuration"]["sha256Before"] == actor.config_sha
                 and fresh["configuration"]["sha256After"] == actor.config_sha
                 and fresh["associatedJob"]["holderMember"] is True,
                 "Actual strict restriction/Job proof is required for each start")
        _require(_sha(actor.selection.activation) == self.guard_sha
                 and _sha(actor.command_path) == self.command_sha, "Internal start guard changed")
        guard, command = _read(actor.selection.activation), _read(actor.command_path)
        _require(guard == actor.protocol.pending_guard(actor.anchor, self.request)
                 and command.get("status") == "start-attempted" and command.get("request") == self.request.ref
                 and command.get("anchorDigest") == actor.anchor.digest,
                 "Only the exact durable explicit pending command authorizes start")
        self.used = True
        self.actual_preflight = fresh
        return None

    def creation_flags(self):
        _require(self.used and self.spawn is None, "Only a consumed one-shot capability may create a child")
        _require(os.name == "nt", "Strict keeper lifecycle requires Windows")
        return subprocess.CREATE_NEW_PROCESS_GROUP | subprocess.CREATE_NO_WINDOW

    def record_spawn(self, pid):
        _require(self.used and self.spawn is None, "One capability must create exactly one native supervisor")
        observed = self.actor.preflight(self.actor.selection.repo, self.actor.config_sha)
        _require(pid in observed["associatedJob"]["memberPids"], "Created supervisor is outside the retained outer Job")
        self.spawn = {"creator": self.actor.keeper, "child": _full_identity(pid), "requestedProfile": "zircon-strict",
                      "creatorInJob": True, "childInJob": True, "configuration": self.actor.anchor.value["binding"]["configuration"],
                      "restrictionPreflightDigest": self.actor.anchor.value["restrictionPreflightDigest"],
                      "creationFlags": self.creation_flags_after_spawn(), "actualRestrictionPreflight": self.actual_preflight,
                      "associatedJobAtSpawn": observed}
        command = _read(self.actor.command_path)
        _write(self.actor.command_path, {**command, "createdSupervisor": self.spawn}, expected_sha=_sha(self.actor.command_path))

    def creation_flags_after_spawn(self):
        return subprocess.CREATE_NEW_PROCESS_GROUP | subprocess.CREATE_NO_WINDOW


def consume_owned_keeper_start(manager, capability):
    _require(type(capability) is _OwnedKeeperStartCapability, "Verified keeper capability required")
    return capability.consume(manager)


def _pending_chain(selection, journal, start):
    """Verify closed evidence and retain its real original pending issuance."""
    variants = ((selection.SUCCESS_SELECTION if hasattr(selection, "SUCCESS_SELECTION") else "runtime-started-functional-acceptance-succeeded-closed",
                 "successCloseout", "_success_closed_documents"),
                ("runtime-started-functional-acceptance-failed-closed", "failedCloseout", "_closed_documents"),
                ("runtime-started-functional-acceptance-failed-before-native-launch-closed", "prelaunchFailureCloseout", "_prelaunch_closed_documents"))
    for status, field, verifier in variants:
        if journal.get("status") == status:
            body, expected_journal, expected_start = getattr(selection, verifier)(journal[field])
            _require(journal == expected_journal and start == expected_start, "Current closed journals differ from immutable closeout")
            return body["plan"]["selectionJournal"], body["plan"]["startJournal"]
    _require(journal.get("status") == "runtime-started-acceptance-pending" and start.get("status") == "running-acceptance-pending",
             "Unresolved acceptance/selection/start outcome blocks normal lifecycle")
    return journal, start


def _binding(selection, intent, release_ref, config_ref, spec_ref, workflow_ref):
    candidate = intent["plan"]["candidateDriver"]
    for ref, expected in ((spec_ref, intent["plan"]["sourceFiles"][".jenkins/deployment-spec.json"]),
                          (workflow_ref, intent["plan"]["sourceFiles"][".jenkins/workflow-spec.json"])):
        _require(_artifact(selection, ref).sha256 == expected, "Historical deployment/workflow specification differs")
    release = _artifact(selection, release_ref).document
    _require(release.get("candidateDriverDigest") == candidate["driverDigest"] and release.get("candidateDriverRoot") == candidate["root"]
             and release.get("reviewedUpgradePath") == intent["review"]["path"]
             and release.get("reviewedUpgradeSha256") == intent["review"]["sha256"], "Original actual keeper release differs from issuance")
    return {"repository": str(selection.repo), "homePath": str(selection.home), "driverDigest": candidate["driverDigest"],
            "driverRoot": candidate["root"], "endpoint": {"address": selection.manager.spec.controller["listenAddress"],
                                                         "port": int(selection.manager.spec.controller["httpPort"])},
            "deploymentSpec": spec_ref, "workflowSpec": workflow_ref, "configuration": config_ref, "originalRelease": release_ref,
            "upgradeIntent": intent.get("_reference") or {}, "selectionReceipt": {}, "originalActivation": intent["originalActivation"],
            "protocol": {"path": str(selection.repo / PROTOCOL_FILE), "sha256": PROTOCOL_SHA256}}


class StrictKeeperLifecycle:
    """Explicit request actor; retains A on an unknown B until manual recovery."""
    def __init__(self, manager, selection, keeper_state, keeper_state_path, *, preflight, publish):
        self.manager, self.selection, self.preflight, self.publish = manager, selection, preflight, publish
        self.keeper_state_path = keeper_state_path
        self.protocol = _protocol(selection.repo); self.keeper = _full_identity(os.getpid())
        self.config_sha = keeper_state["restrictionPreflight"]["configuration"]["sha256Before"]
        self.command_path = selection.state / "deployment/formal-lifecycle-command.json"
        self.directory = selection.state / "formal-lifecycle"; self.descriptor = self.directory / "keeper.json"
        self.seen = set(); self.blocked = False; self.stop_proof = None; self.stop_packet = None; self.stop_running_anchor = None
        assert_no_unresolved_keeper_command(manager)
        j, s = _read(selection.journal), _read(selection.start_journal)
        pending_j, pending_s = _pending_chain(selection, j, s)
        intent, body, current = selection._started_chain(pending_j, pending_s)
        self.release = _artifact(selection, keeper_state["reviewedRelease"]).document
        _reviewed_code(selection, self.release)
        _require(keeper_state.get("kind") == "desktop-execution-upgrade-keeper",
                 "A typed continuation actor requires its pinned issuance reader; it cannot impersonate the original upgrade keeper")
        _reviewed_keeper_argv(selection.repo, self.keeper, keeper_state["reviewedRelease"], self.config_sha)
        _require(current == selection._owner(*selection._snapshot())
                 and keeper_state.get("ownedGeneration") == body["ownedGeneration"]
                 and keeper_state.get("startupReceipt") == j["startupReceipt"], "Keeper does not own the issued current generation")
        keeper_record = _archive(selection.state, "formal-lifecycle-keeper-origin", _raw(keeper_state_path))
        self.origin_keeper = keeper_record
        spec_ref = _archive(selection.state, "formal-lifecycle-spec", _raw(manager.spec.path))
        workflow_ref = _archive(selection.state, "formal-lifecycle-workflow", _raw(selection.repo / ".jenkins/workflow-spec.json"))
        config_ref = {"path": str(selection.repo / ".codex/config.toml"), "sha256": self.config_sha}
        binding = _binding(selection, intent, keeper_state["reviewedRelease"], config_ref, spec_ref, workflow_ref)
        binding.update(upgradeIntent=j["intent"], selectionReceipt=j["selectionReceipt"])
        self.anchor = self.protocol.RuntimeAnchor.from_verified_issuance(binding=binding, keeper=self.keeper, runtime_owner=current,
            selected_driver=_read(selection.selector), protected_holds=protected_holds(selection.state),
            journals=self._journal_refs(), runtime_receipt=_artifact(selection, j["startupReceipt"]),
            restriction_preflight_digest=digest(keeper_state["restrictionPreflight"]))
        self.origin_activation = _raw(selection.activation)
        _require(hashlib.sha256(self.origin_activation).hexdigest() == binding["originalActivation"]["sha256"], "Activation history changed")
        self._advertise()

    def _journal_refs(self):
        return {key: {"path": str(path), "sha256": _sha(path)} for key, path in
                (("selection", self.selection.journal), ("start", self.selection.start_journal))}

    def _advertise(self):
        _write(self.descriptor, {"schemaVersion": 1, "kind": "formal-tray-strict-keeper-descriptor",
            "admission": "blocked" if self.blocked else "idle", "anchor": self.anchor.value,
            "anchorDigest": self.anchor.digest, "runtimeReceipt": self.anchor.runtime_receipt.ref,
            "keeperRecord": self.origin_keeper})

    def _fresh(self):
        _reviewed_code(self.selection, self.release)
        _reviewed_keeper_argv(self.selection.repo, self.keeper, self.anchor.value["binding"]["originalRelease"], self.config_sha)
        _require(_sha(self.manager.spec.path) == self.anchor.value["binding"]["deploymentSpec"]["sha256"]
                 and _sha(self.selection.repo / ".jenkins/workflow-spec.json") == self.anchor.value["binding"]["workflowSpec"]["sha256"]
                 and _sha(self.selection.repo / ".codex/config.toml") == self.config_sha,
                 "Current specification/configuration differs from original reviewed issuance")
        current = self.selection._owner(*self.selection._snapshot())
        _require((self.manager._read() or {}).get("state") == self.anchor.value["state"], "Native lifecycle state changed")
        if self.anchor.value["state"] == "running":
            for key in ("supervisor", "controller", "agent"):
                observe_live_identity(current[key])
            observe_endpoint_owner(self.manager.spec.controller["listenAddress"], int(self.manager.spec.controller["httpPort"]), current["controller"]["pid"])
        return current

    def _refresh_issuance_locked(self):
        refs = self._journal_refs()
        if refs == self.anchor.value["journals"]:
            return
        journal, start = _read(self.selection.journal), _read(self.selection.start_journal)
        j, s = _pending_chain(self.selection, journal, start)
        _, _, owner = self.selection._started_chain(j, s)
        actual = self._fresh()
        _require(owner == actual == self.anchor.value["owner"],
                 "Journal refresh cannot adopt a different native runtime")
        # The current receipt is the stopped proof after an explicit Stop;
        # normal issuance still references the unchanged original startup.
        expected_startup = (_artifact(self.selection, self.stop_packet["terminalReceipt"]).document["previousStartupReceipt"]
                            if self.anchor.value["state"] == "stopped" else self.anchor.runtime_receipt.ref)
        _require(journal.get("startupReceipt") == expected_startup
                 and _read(self.selection.selector) == self.anchor.value["selected"]
                 and digest(protected_holds(self.selection.state)) == digest(self.anchor.value["protectedHolds"]),
                 "Only verified same-owner acceptance journal changes may refresh admission")
        self.anchor = self.protocol.RuntimeAnchor.from_verified_issuance(binding=self.anchor.value["binding"], keeper=self.keeper,
            runtime_owner=owner, selected_driver=self.anchor.value["selected"], protected_holds=self.anchor.value["protectedHolds"],
            journals=refs, runtime_receipt=self.anchor.runtime_receipt,
            restriction_preflight_digest=self.anchor.value["restrictionPreflightDigest"], state=self.anchor.value["state"])
        self._advertise()

    def pump(self):
        if self.blocked:
            return
        if self._journal_refs() != self.anchor.value["journals"]:
            # Closeout may commit its two pointers in sequence. No native work
            # occurs until the complete same-owner chain verifies again.
            try:
                with process_lock(self.selection.state / "deployment-start.lock", reason="deployment_operation_busy"):
                    self._refresh_issuance_locked()
            except JenkinsError:
                return
        requests = self.directory / "requests"; _plain(requests)
        if not requests.exists():
            return
        for marker in sorted(requests.glob("*.json")):
            request_id = marker.stem
            if request_id in self.seen:
                continue
            processed = self.directory / "results" / (request_id + ".json")
            if processed.exists():
                body = _artifact(self.selection, _read(processed)).document
                _require(body.get("request") == _read(marker), "Previous request result reference changed")
                self.seen.add(request_id)
                continue
            self.seen.add(request_id)
            request = None
            try:
                request = _artifact(self.selection, _read(marker))
                _require(request.document.get("requestId") == request_id, "Request marker and immutable command differ")
                self._execute(request)
            except Exception as error:
                if not self.blocked and request is not None:
                    reference = _document_archive(self.selection.state, "formal-lifecycle-rejected", {
                        "status": "rejected", "request": request.ref, "reasonCode": getattr(error, "code", "formal_keeper_command_rejected"),
                        "serviceLifecycleExecuted": False, "runtimeAcceptance": False})
                    _write(processed, reference)
                    continue
                self.blocked = True
                self.publish("holding-formal-lifecycle-unknown", {"reasonCode": getattr(error, "code", "formal_keeper_command_unproven")})
                self._advertise(); return

    def _execute(self, request):
        selection, protocol = self.selection, self.protocol
        with process_lock(selection.state / "deployment-start.lock", reason="deployment_operation_busy"):
            assert_no_unresolved_keeper_command(self.manager)
            self._refresh_issuance_locked()
            before_j, before_s = _raw(selection.journal), _raw(selection.start_journal)
            old_j, old_s = json.loads(before_j), json.loads(before_s)
            pending_j, pending_s = _pending_chain(selection, old_j, old_s)
            selection._started_chain(pending_j, pending_s)
            current = self._fresh()
            doc = protocol.validate_request(self.anchor, request, keeper_observation=_full_identity(os.getpid()), owner_observation=current,
                selected_observation=_read(selection.selector), journal_observation=self._journal_refs(),
                holds_observation=protected_holds(selection.state), command_state="idle", used_request_ids=set())
            command = {"schemaVersion": 1, "kind": COMMAND_KIND, "status": "pending", "request": request.ref,
                       "anchorDigest": self.anchor.digest, "runtimeAcceptance": False, "historicalHoldsReleased": False}
            archived_j = _archive(selection.state, "formal-lifecycle-previous-selection", before_j)
            archived_s = _archive(selection.state, "formal-lifecycle-previous-start", before_s)
            # Unknown writes after this point are unresolved even if no native
            # call was reached. Never classify them as harmless rejection.
            self.blocked = True
            _write(self.command_path, command)
            guard = protocol.pending_guard(self.anchor, request)
            _require(_raw(selection.activation) == self.origin_activation, "Activation changed before explicit command")
            _write(selection.activation, guard, expected_sha=_sha(selection.activation))
            self._advertise()
            try:
                previous_anchor = self.anchor
                if doc["action"] in {"stop", "restart"}:
                    self.stop_running_anchor = previous_anchor
                    _write(self.command_path, {**command, "status": "stop-attempted"})
                    stopped = self.manager._stop_locked()
                    binding = _artifact(selection, stopped["stopBinding"])
                    actual_command_path = Path(binding.document["rawStopResult"]["path"])
                    actual_command_path = actual_command_path.with_name(actual_command_path.name.removesuffix(".result.json") + ".json")
                    command_ref = _archive(selection.state, "formal-lifecycle-host-command", _raw(actual_command_path))
                    host_command = _artifact(selection, command_ref)
                    self.stop_proof = protocol.verify_owned_stop(self.anchor, request, binding, host_command, stopped,
                                                               holds_observation=protected_holds(selection.state))
                    terminal_ref = _document_archive(selection.state, "formal-lifecycle-terminal", self.stop_proof.document)
                    terminal = _artifact(selection, terminal_ref)
                    self.stop_packet = {"stopRequest": request.ref, "stopBinding": binding.ref, "hostCommand": host_command.ref,
                                        "stoppedOperation": stopped, "terminalReceipt": terminal.ref}
                else:
                    _require(self.stop_proof is not None and self.stop_packet is not None, "No verified in-memory own stop; explicit recovery required")
                    terminal = _artifact(selection, self.stop_packet["terminalReceipt"])
                    stopped = self.stop_packet["stoppedOperation"]
                    _require(self.manager._read() == stopped, "Proven own stopped operation changed")
                    from .startup_observations import observe_departed_identity
                    observe_departed_identity(current["supervisor"])
                if doc["action"] == "stop":
                    self.anchor = protocol.stopped_anchor(self.anchor, terminal, self.stop_proof)
                    result = {"status": "stopped", "terminalReceipt": terminal.ref, "owner": current, "runtimeAcceptance": False}
                    self.publish("holding-new-generation-acceptance-pending", {"normalLifecycleState": "stopped", "normalStopReceipt": terminal.ref})
                    self._finish(request, result, "stopped")
                    return
                command = {**command, "status": "start-attempted", "terminalReceipt": terminal.ref}
                _write(self.command_path, command)
                cap = _OwnedKeeperStartCapability(self, request, _sha(selection.activation), _sha(self.command_path))
                started = self.manager._start_locked(selection.state / "deployment-start.lock", owned_keeper_start=cap)
                _require(cap.spawn is not None, "Native start has no strict creation proof")
                jobs = self.manager.register_jobs(); health = self.manager.health()
                operation, host = selection._snapshot(); new_owner = selection._owner(operation, host)
                observations = [observe_live_identity(new_owner[key]) for key in ("supervisor", "controller", "agent")]
                endpoint = observe_endpoint_owner(self.manager.spec.controller["listenAddress"], int(self.manager.spec.controller["httpPort"]), new_owner["controller"]["pid"])
                accepting_status, accepting_raw = self.manager._request("/api/json?tree=quietingDown")
                _require(accepting_status == 200, "Current Jenkins admission status is unavailable")
                accepting = json.loads(accepting_raw.decode("utf-8"))
                health = {**health, "acceptingRequests": accepting.get("quietingDown") is False}
                fresh = {"owner": new_owner, "nativeIdentities": observations, "protectedHolds": protected_holds(selection.state),
                         "selectedDriver": _read(selection.selector), "strictSpawn": cap.spawn, "endpoint": endpoint,
                         "health": health, "quietingDown": accepting.get("quietingDown")}
                start_doc = {"schemaVersion": 1, "kind": "formal-tray-strict-keeper-start-observation", "request": request.ref,
                    "stopReceipt": terminal.ref, "binding": self.anchor.value["binding"], "driverDigest": self.anchor.value["binding"]["driverDigest"],
                    "driverRoot": self.anchor.value["binding"]["driverRoot"], "status": "running-acceptance-pending", "operation": operation, "host": host,
                    **{key: False for key in protocol.ACCEPTANCE_FIELDS}}
                start_ref = _document_archive(selection.state, "formal-lifecycle-native-start", start_doc)
                verified = protocol.verify_owned_successor(self.anchor, request, terminal, _artifact(selection, start_ref),
                                                          verified_stop=self.stop_proof, fresh_observations=fresh)
                issued_ref = _document_archive(selection.state, "formal-lifecycle-successor", verified.document)
                root_anchor = previous_anchor
                if doc["action"] == "start":
                    # Replay from the original A running anchor, not a guessed
                    # current owner. Its stop request has the earlier exact refs.
                    root_anchor = self.stop_running_anchor
                packet = {"schemaVersion": 1, "kind": "formal-tray-owned-runtime-lineage", "previousSelection": archived_j,
                    "previousStart": archived_s, "rootAnchor": {"value": root_anchor.value, "runtimeReceipt": root_anchor.runtime_receipt.ref},
                    "keeperOrigin": self.origin_keeper, **self.stop_packet, "startRequest": request.ref,
                    "startupObservation": start_ref, "successorReceipt": issued_ref, "historicalObservations": fresh,
                    "runtimeAcceptance": False, "historicalHoldsReleased": False}
                packet_ref = _document_archive(selection.state, "formal-lifecycle-lineage", packet)
                remove = {"successCloseout", "failedCloseout", "prelaunchFailureCloseout"}
                new_j = {k: v for k, v in old_j.items() if k not in remove}
                new_s = {k: v for k, v in old_s.items() if k not in remove}
                owned = {key: operation[key] for key in ("operationId", "generation", "pid", "creationTime", "executable", "commandDigest", "homePath")}
                new_j.update(status="runtime-started-acceptance-pending", runtimeOperationId=operation["operationId"], ownedGeneration=owned,
                             startupReceipt=issued_ref, normalLifecycle=packet_ref, runtimeAcceptance=False)
                new_s.update(kind="formal-tray-owned-runtime-start", status="running-acceptance-pending", runtimeOperationId=operation["operationId"],
                             ownedGeneration=owned, startupReceipt=issued_ref, normalLifecycle=packet_ref, runtimeAcceptance=False, cargoAcceptance=False, trayAcceptance=False)
                _write(selection.journal, new_j, expected_sha=hashlib.sha256(before_j).hexdigest())
                _write(selection.start_journal, new_s, expected_sha=hashlib.sha256(before_s).hexdigest())
                require_normal_started_chain(selection, new_j, new_s, check_activation=False)
                self.anchor = protocol.successor_anchor(self.anchor, _artifact(selection, issued_ref), verified, journals=self._journal_refs())
                self.publish("holding-new-generation-acceptance-pending", {"ownedGeneration": owned, "startupReceipt": issued_ref,
                    "normalLifecycle": packet_ref, "normalLifecycleState": "running", "runtimeAcceptance": False})
                self._finish(request, {"status": "running-acceptance-pending", "startupReceipt": issued_ref, "normalLifecycle": packet_ref,
                    "ownedGeneration": owned, "jobProvisioning": jobs, "runtimeAcceptance": False}, "committed")
            except Exception as error:
                self.blocked = True
                _write(self.command_path, {**_read(self.command_path), "status": "unknown", "reasonCode": getattr(error, "code", "formal_keeper_command_unproven")})
                if _raw(selection.activation) == self.origin_activation:
                    _write(selection.activation, guard, expected_sha=_sha(selection.activation))
                raise

    def _finish(self, request, result, status):
        _require(digest(protected_holds(self.selection.state)) == digest(self.anchor.value["protectedHolds"]), "Protected holds changed during lifecycle")
        published = _read(self.keeper_state_path)
        expected = {**self.anchor.value["owner"]["operation"], **self.anchor.value["owner"]["supervisor"]}
        _require(published.get("status") == "holding-new-generation-acceptance-pending"
                 and published.get("automaticRestart") is False
                 and all(published.get("ownedGeneration", {}).get(k) == expected.get(k) for k in
                         ("operationId", "generation", "pid", "creationTime", "executable", "commandDigest", "homePath"))
                 and ((status == "committed" and published.get("startupReceipt") == self.anchor.runtime_receipt.ref
                       and published.get("normalLifecycleState") == "running")
                      or (status == "stopped" and published.get("normalStopReceipt") == self.anchor.runtime_receipt.ref
                          and published.get("normalLifecycleState") == "stopped")),
                 "Exact keeper-owned lifecycle evidence must be durable before guard restoration")
        result_ref = _document_archive(self.selection.state, "formal-lifecycle-command-result", {**result, "request": request.ref})
        active = _read(self.command_path)
        _require(active.get("request") == request.ref and active.get("status") in {"stop-attempted", "start-attempted"}, "Command commit ownership changed")
        # Both pointer commits and the keeper-owned receipt are already durable.
        # Commit command evidence first; restoring the old recognized guard is
        # the final start permission. A crash before restoration stays blocked.
        _write(self.command_path, {**active, "status": status, "result": result_ref})
        _write_raw(self.selection.activation, self.origin_activation, expected_sha=_sha(self.selection.activation))
        _require(_raw(self.selection.activation) == self.origin_activation, "Original activation bytes must be restored exactly")
        _write(self.directory / "results" / (request.document["requestId"] + ".json"), result_ref)
        self.blocked = False; self._advertise()


def require_normal_started_chain(selection, journal, start, *, check_activation=True):
    """Verify every same-driver predecessor; original A receipts stay immutable."""
    depth = getattr(selection, "_normal_lineage_depth", 0)
    _require(depth < 64, "Normal lifecycle chain exceeds bounded verification depth")
    selection._normal_lineage_depth = depth + 1
    try:
        ref = journal.get("normalLifecycle")
        _require(ref == start.get("normalLifecycle") and start.get("kind") == "formal-tray-owned-runtime-start",
                 "Typed normal lifecycle pointer mismatch")
        packet = _artifact(selection, ref).document
        _require(packet.get("kind") == "formal-tray-owned-runtime-lineage" and packet.get("schemaVersion") == 1
                 and packet.get("runtimeAcceptance") is False and packet.get("historicalHoldsReleased") is False, "Lineage authority mismatch")
        prev_j = _artifact(selection, packet["previousSelection"]).document
        prev_s = _artifact(selection, packet["previousStart"]).document
        j, s = _pending_chain(selection, prev_j, prev_s)
        intent, previous_body, previous_owner = selection._started_chain(j, s, check_activation=False)
        root = packet["rootAnchor"]; value = root["value"]
        origin = _artifact(selection, packet["keeperOrigin"]).document
        binding = _binding(selection, intent, origin["reviewedRelease"], value["binding"]["configuration"],
                           value["binding"]["deploymentSpec"], value["binding"]["workflowSpec"])
        binding.update(upgradeIntent=prev_j["intent"], selectionReceipt=prev_j["selectionReceipt"])
        _require(value["binding"] == binding and value["owner"] == previous_owner and value["state"] == "running"
                 and root["runtimeReceipt"] == prev_j["startupReceipt"]
                 and value["journals"]["selection"]["sha256"] == packet["previousSelection"]["sha256"]
                 and value["journals"]["start"]["sha256"] == packet["previousStart"]["sha256"]
                 and value["selected"] == {**prev_j["selected"], "runtimeOperationId": previous_owner["operation"]["operationId"]}
                 and digest(value["protectedHolds"]) == digest(intent["plan"]["protectedHolds"])
                 and origin.get("automaticRestart") is False
                 and all(origin["restrictionPreflight"]["holderIdentity"].get(k) == value["keeper"].get(k)
                         for k in ("pid", "creationTime", "executable"))
                 and value["restrictionPreflightDigest"] == digest(origin["restrictionPreflight"]), "Historical rooted keeper/binding changed")
        protocol = _protocol(selection.repo)
        anchor = protocol.RuntimeAnchor.from_verified_issuance(binding=binding, keeper=value["keeper"], runtime_owner=previous_owner,
            selected_driver=value["selected"], protected_holds=value["protectedHolds"], journals=value["journals"],
            runtime_receipt=_artifact(selection, root["runtimeReceipt"]), restriction_preflight_digest=value["restrictionPreflightDigest"])
        loaded = {key: _artifact(selection, packet[key]) for key in
                  ("stopRequest", "stopBinding", "hostCommand", "terminalReceipt", "startRequest", "startupObservation", "successorReceipt")}
        spawn = packet["historicalObservations"]["strictSpawn"]
        proof, at_spawn = spawn.get("actualRestrictionPreflight") or {}, spawn.get("associatedJobAtSpawn") or {}
        for observed in (proof, at_spawn):
            configuration, job = observed.get("configuration") or {}, observed.get("associatedJob") or {}
            _require(configuration.get("updateOpenDenied") is True and configuration.get("sha256Before") == binding["configuration"]["sha256"]
                     and configuration.get("sha256After") == binding["configuration"]["sha256"] and job.get("holderMember") is True
                     and all((observed.get("holderIdentity") or {}).get(k) == value["keeper"].get(k)
                             for k in ("pid", "creationTime", "executable")), "Historical actual strict restriction/Job observation changed")
        _require(value["keeper"]["pid"] in at_spawn["associatedJob"].get("memberPids", [])
                 and spawn["child"]["pid"] in at_spawn["associatedJob"].get("memberPids", []), "Strict Job creation membership is missing")
        verified = protocol.verify_archived_transition(anchor, {**loaded, "stoppedOperation": packet["stoppedOperation"],
                                                               "historicalObservations": packet["historicalObservations"]})
        body = verified.document; native = body["owner"]; runtime = native["operation"]["operationId"]
        _require(journal.get("status") == "runtime-started-acceptance-pending" and start.get("status") == "running-acceptance-pending"
                 and journal.get("startupReceipt") == packet["successorReceipt"] == start.get("startupReceipt")
                 and all(value.get("runtimeAcceptance") is False for value in (journal, start))
                 and start.get("cargoAcceptance") is False and start.get("trayAcceptance") is False
                 and journal.get("runtimeOperationId") == start.get("runtimeOperationId") == runtime
                 and journal.get("ownedGeneration") == start.get("ownedGeneration")
                 and all(journal["ownedGeneration"].get(k) == body["operation"].get(k) for k in
                         ("operationId", "generation", "pid", "creationTime", "executable", "commandDigest", "homePath"))
                 and all(journal.get(k) == prev_j.get(k) for k in ("intent", "selected", "originalActivation", "selectionReceipt", "upgradeDigest", "startGuardRestoreAuthorized")),
                 "Normal-start journal must bind exact immutable B and preserve original selection")
        if check_activation:
            from .execution_activation import assert_activation_start_allowed
            _require(_sha(selection.activation) == binding["originalActivation"]["sha256"]
                     and assert_activation_start_allowed(selection.manager, guard_only=True) is None, "Unresolved normal start guard blocks lineage")
        view = {**body, "candidateDriverDigest": binding["driverDigest"], "ownedGeneration": journal["ownedGeneration"],
                "previousOwner": previous_owner, "selectionReceipt": journal["selectionReceipt"], "upgradeDigest": journal["upgradeDigest"]}
        return intent, view, native
    finally:
        selection._normal_lineage_depth = depth


def request_keeper_action(config, action, *, timeout=240):
    """Visible GUI publishes one request; it never starts a service child."""
    recovery = config.paths.state / "formal-lifecycle/recovery-keeper.json"
    if recovery.exists():
        from .recovery_keeper import request_recovery_keeper_action
        return request_recovery_keeper_action(config, action, timeout=timeout)
    protocol = _protocol(config.paths.repo)
    directory = config.paths.state / "formal-lifecycle"
    with process_lock(config.paths.state / "formal-lifecycle-submit.lock", reason="formal_keeper_request_busy"):
        # The keeper derives new fences after an authorized same-owner M4
        # closeout. The GUI only waits; it cannot publish replacement authority.
        deadline = time.monotonic() + timeout
        while True:
            descriptor = _read(directory / "keeper.json")
            journals = descriptor.get("anchor", {}).get("journals", {})
            if (set(journals) == {"selection", "start"}
                    and all(_sha(ref["path"]) == ref["sha256"] for ref in journals.values())):
                break
            _require(time.monotonic() < deadline, "Keeper issuance refresh remains unproven; no lifecycle request was issued")
            time.sleep(.1)
        _require(descriptor.get("kind") == "formal-tray-strict-keeper-descriptor" and descriptor.get("admission") == "idle",
                 "A reviewed live strict keeper must accept this explicit command")
        value = descriptor["anchor"]
        _require(value["binding"]["repository"] == str(config.paths.repo)
                 and value["binding"]["homePath"] == str(config.paths.home)
                 and value["binding"]["deploymentSpec"]["sha256"] == _sha(config.spec.path), "Tray configuration differs from keeper")
        observe_live_identity(value["keeper"])
        origin_ref = descriptor["keeperRecord"]
        origin = protocol.Artifact(origin_ref["path"], origin_ref["sha256"], _raw(origin_ref["path"])).document
        release_ref = value["binding"]["originalRelease"]
        release = protocol.Artifact(release_ref["path"], release_ref["sha256"], _raw(release_ref["path"])).document
        entry = config.paths.repo / ".jenkins/state/migration-baseline/jenkins_upgrade_desktop_holder.py"
        expected_args = [str(config.paths.runtime / "python/python.exe"), "-B", str(entry), "--config-sha256",
                         value["binding"]["configuration"]["sha256"], "--release-file", release_ref["path"],
                         "--release-sha256", release_ref["sha256"]]
        actual_args = _windows_command_line(value["keeper"]["pid"])
        _require(list(actual_args) in (expected_args, [*expected_args, "--no-tray"])
                 and _digest_args(actual_args) == value["keeper"]["commandDigest"]
                 and _sha(entry) == release["operatorFiles"][str(entry)]
                 and origin.get("reviewedRelease") == release_ref
                 and origin.get("automaticRestart") is False
                 and all(origin["restrictionPreflight"]["holderIdentity"].get(k) == value["keeper"].get(k)
                         for k in ("pid", "creationTime", "executable")), "Only the original reviewed retained keeper may receive tray commands")
        runtime_ref = descriptor["runtimeReceipt"]
        runtime = protocol.Artifact(runtime_ref["path"], runtime_ref["sha256"], _raw(runtime_ref["path"]))
        anchor = protocol.RuntimeAnchor.from_verified_issuance(binding=value["binding"], keeper=value["keeper"], runtime_owner=value["owner"],
            selected_driver=value["selected"], protected_holds=value["protectedHolds"], journals=value["journals"], runtime_receipt=runtime,
            restriction_preflight_digest=value["restrictionPreflightDigest"], state=value["state"])
        _require(anchor.digest == descriptor["anchorDigest"], "Keeper descriptor changed")
        request_id = uuid.uuid4().hex
        request = protocol.request_document(anchor, request_id, action)
        reference = _document_archive(config.paths.state, "formal-lifecycle-request", request)
        _write(directory / "requests" / (request_id + ".json"), reference)
        result = directory / "results" / (request_id + ".json")
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if result.exists():
                ref = _read(result)
                body = protocol.Artifact(ref["path"], ref["sha256"], _raw(ref["path"])).document
                if body.get("request") == reference and body.get("status") == "rejected":
                    raise JenkinsError(body.get("reasonCode", "formal_keeper_command_rejected"), "The explicit request was rejected before lifecycle work")
                _require(body.get("request") == reference and body.get("status") in {"stopped", "running-acceptance-pending"}
                         and body.get("runtimeAcceptance") is False, "Exact request result is unknown")
                return body
            command = config.paths.state / "deployment/formal-lifecycle-command.json"
            if (_read(directory / "keeper.json").get("admission") == "blocked" and command.exists()
                    and _read(command).get("status") == "unknown"):
                raise JenkinsError("formal_keeper_command_unknown", "Preserve the original explicit request; manual proof recovery is required")
            time.sleep(.1)
        raise JenkinsError("formal_keeper_command_pending", "Reconcile this original request without resubmission", details={"request": reference})

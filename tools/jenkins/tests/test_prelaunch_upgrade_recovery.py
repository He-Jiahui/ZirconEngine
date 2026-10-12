"""Unknown owned-upgrade failures keep positive native and sealed-control evidence."""
from __future__ import annotations

from contextlib import nullcontext, contextmanager
from copy import deepcopy
import hashlib
import io
import json
from pathlib import Path
from unittest import TestCase
from unittest.mock import Mock, patch

from tools.jenkins.contracts import JenkinsError, canonical_json, digest
from tools.jenkins.deployment import prelaunch_upgrade_recovery as recovery
from tools.jenkins.deployment import owned_upgrade as upgrade, recovery_keeper
from tools.jenkins.tests.test_external_native_stop_upgrade import ExternalStoppedOwner


class PrelaunchFixture(ExternalStoppedOwner):
    def __init__(self):
        super().__init__()
        self.new_target = self.targets["b" * 64]
        self.repo = self.manager.paths.repo
        self.sources = {}
        self.control_names = tuple(next(iter(recovery._REVIEWED_FAILURE_CODE.values())))

    def setup_failure(self):
        plan = self.prepare()
        self.plan_ref = self.put(self.state / "recovery-lifecycle/history/owned-upgrade-initial-plan-fixture.json", plan.document)
        arguments, command = self.replacement(plan)
        self.keeper = arguments["keeper"]
        command.update(status="unknown", reasonCode="owned_recovery_upgrade_unproven")
        self.command_ref = self.put(self.paths["terminalCommand"], command)
        self.command = command
        self.sources.clear()
        for index, name in enumerate(self.control_names):
            path = (Path(self.new_target["root"]) if name.startswith("tools/jenkins/") else self.repo) / name
            raw = ("reviewed control " + str(index)).encode()
            self.files[str(path)] = raw
            checksum = hashlib.sha256(raw).hexdigest()
            self.sources[name] = checksum
            self.successor["operatorFiles"][str(self.repo / name)] = checksum
        # Rebind one internally consistent fixture release before publishing
        # the failed command, immutable plan/request and handoff.
        self.successor["actorCodeRoot"] = self.new_target["root"]
        self.successor_ref = self.put(self.successor_path, self.successor)
        for reference in (arguments["request"], arguments["handoff_ref"], self.plan_ref):
            value = json.loads(self.files[reference["path"]])
            if reference == self.plan_ref:
                value["successorRelease"] = self.successor_ref
            else:
                value["release"] = self.successor_ref
            self.put(Path(reference["path"]), value)
        plan_doc = json.loads(self.files[self.plan_ref["path"]])
        self.plan_ref = self.ref(Path(self.plan_ref["path"]))
        upgrade_digest = digest(plan_doc)
        for reference in (arguments["request"], arguments["handoff_ref"]):
            value = json.loads(self.files[reference["path"]])
            value["ownedUpgradeDigest"] = upgrade_digest
            if reference == arguments["handoff_ref"]:
                value["request"] = self.ref(Path(arguments["request"]["path"]))
            self.put(Path(reference["path"]), value)
        command.update(release=self.successor_ref, ownedUpgradeDigest=upgrade_digest,
                       request=self.ref(Path(arguments["request"]["path"])),
                       ownedUpgradeHandoff=self.ref(Path(arguments["handoff_ref"]["path"])))
        self.command_ref = self.put(self.paths["terminalCommand"], command)
        self.command = command
        host = {**self.keeper, "pid": 900, "executable": self.successor["sandboxHost"]["path"],
                "creationTime": str(int(self.boot) + 290)}
        self.host = host
        self.host_ref = self.put(self.state / "recovery-lifecycle/sandbox-hosts/host.json", {
            "kind": "official-strict-recovery-sandbox-host", "release": self.successor_ref, "host": host,
            "profile": "zircon-strict", "includeManagedConfig": True, "automaticRestart": False,
            "runtimeAcceptance": False})
        restriction = {"profile": "zircon-strict"}
        self.holder = {"schemaVersion": 1, "kind": "formal-recovery-desktop-holder", "status": "recovery-blocked",
            "reasonCode": command["reasonCode"], "message": "Another repository runtime still owns the deployment",
            "reviewedRelease": self.successor_ref, "actorCodeRoot": self.new_target["root"],
            "ownedUpgradeDigest": upgrade_digest, "recoveryDigest": None, "restrictionPreflight": restriction,
            "officialSandboxHost": {"record": self.host_ref, "host": host,
                                    "officialHostSha256": self.successor["sandboxHost"]["sha256"]},
            **{name: False for name in ("automaticRestart", "runtimeAcceptance", "cargoAcceptance",
                                        "requestsResubmitted", "serviceLifecycleExecuted")}}
        self.holder["restrictionPreflight"]["associatedJob"] = {"memberPids": [self.keeper["pid"], 901]}
        self.holder_ref = self.put(self.state / "recovery-lifecycle/holders/holder.json", self.holder)
        self.replacement_id = "d" * 32
        self.ledger = "ledger unchanged"
        self.targets[self.new_target["driverDigest"]] = self.new_target

    @contextmanager
    def observations(self):
        stack = super().observations()
        try:
            stack.enter_context(patch.object(recovery, "_ledger", side_effect=lambda state: getattr(self, "ledger", "ledger unchanged")))
            stack.enter_context(patch.object(recovery, "_local_reference", side_effect=lambda ref, parent:
                json.loads(self.files[ref["path"]]) if Path(ref["path"]).parent == parent and self.ref(Path(ref["path"])) == ref
                else (_ for _ in ()).throw(JenkinsError("bad_ref", "reference changed"))))
            stack.enter_context(patch.object(recovery, "verify_driver", side_effect=lambda root, expected_digest: self.targets[expected_digest]))
            stack.enter_context(patch.object(recovery, "runtime_owner_inventory", return_value=[]))
            stack.enter_context(patch.object(recovery, "observe_departed_identity", side_effect=self.absent))
            stack.enter_context(patch.object(recovery, "observe_live_identity"))
            stack.enter_context(patch.object(recovery, "process_lock", side_effect=lambda *args, **kw: nullcontext()))
            stack.enter_context(patch.object(recovery_keeper, "_restriction", side_effect=lambda probe, *args: probe))
            stack.enter_context(patch.object(recovery, "_REVIEWED_FAILURE_CODE",
                {self.new_target["driverDigest"]: getattr(self, "sources", {})}))
            original = upgrade._historical_release
            def release(manager, reference, **kwargs):
                if reference == self.successor_ref:
                    return deepcopy(self.successor), deepcopy(self.new_target)
                if reference == self.old_release_ref:
                    return deepcopy(self.old_release), deepcopy(self.targets["a" * 64])
                return original(manager, reference, **kwargs)
            stack.enter_context(patch.object(upgrade, "_historical_release", side_effect=release))
            stack.enter_context(patch.object(recovery_keeper, "_archive_bytes", side_effect=self.archive))
            stack.enter_context(patch.object(recovery_keeper, "_archive", side_effect=lambda state, prefix, value:
                self.archive(state, prefix, canonical_json(value) + b"\n")))
            stack.enter_context(patch.object(recovery_keeper, "_write", side_effect=self.write))
            stack.enter_context(patch.object(recovery, "replace_file", side_effect=lambda source, destination:
                self.files.__setitem__(str(destination), self.files.pop(str(source)))))
            stack.enter_context(patch.object(recovery.os, "fsync"))
            stack.enter_context(patch.object(Path, "unlink", autospec=True,
                side_effect=lambda path: self.files.pop(str(path), None)))
            owner = self
            class Stream(io.BytesIO):
                def __init__(self, path):
                    super().__init__()
                    self.path = str(path)
                def fileno(self):
                    return 987
                def close(self):
                    owner.files[self.path] = self.getvalue()
                    super().close()
            stack.enter_context(patch.object(Path, "open", autospec=True,
                side_effect=lambda path, *args, **kwargs: Stream(path)))

            yield stack
        finally:
            stack.close()

    def archive(self, state, prefix, raw):
        path = state / "recovery-lifecycle/history" / (prefix + "-" + hashlib.sha256(raw).hexdigest() + ".json")
        self.files[str(path)] = raw
        return self.ref(path)

    def write(self, path, value, *, expected_sha):
        current = self.ref(path)["sha256"] if str(path) in self.files else None
        if current != expected_sha:
            raise JenkinsError("cas_changed", "changed")
        self.put(path, value)

    def capture_retirement(self):
        pins, *_ = recovery._capture(self.manager, self.command_ref["sha256"], self.holder_ref)
        roles = {"keeper": self.keeper, "officialHost": self.host}
        for index, name in enumerate(("console", "runner", "hostConsole", "dispatcher"), 901):
            roles[name] = {**self.keeper, "pid": index, "creationTime": str(int(self.boot) + index)}
        scope = {"processes": roles, "publishedAssociatedJobMemberPids": sorted([self.keeper["pid"], 901]),
            "currentContainedActorPids": sorted([self.keeper["pid"], 901]),
            "currentOfficialHostDescendants": sorted(value["pid"] for name, value in roles.items() if name != "dispatcher"),
            "completeLiveJobInventoryClaimed": False, "terminationScope": ["keeper"], "otherActorsExitNaturally": True}
        args = [self.keeper["executable"], "-B", str(self.repo / "tools/dev/jenkins_recovery_keeper.py"),
            "--config-sha256", self.successor["config"]["sha256"], "--release-file", self.successor_ref["path"],
            "--release-sha256", self.successor_ref["sha256"]]
        self.digest_patch = patch.object(recovery, "_digest_args", return_value=self.keeper["commandDigest"])
        self.retired = {"schemaVersion": 1, "kind": recovery.RETIREMENT_KIND,
            **{name: pins[name] for name in ("originalCommand", "holder", "ownedUpgradeHandoff", "release",
                                            "keeper", "bootFileTime", "rawAnchors", "requestFiles", "controlFlow",
                                            "protectedHolds", "ledgerDigest")},
            "beforeIdentity": self.keeper, "beforeArguments": args,
            "handleIdentity": {name: self.keeper[name] for name in ("pid", "creationTime", "executable")},
            "handleIdentityVerified": True, "nativeHandleRetained": True, "waitResult": 0, "processExitCode": 196,
            "method": "forced-terminate-held-handle", "forced": True, "graceful": False,
            "terminateProcess": {"api": "TerminateProcess", "succeeded": True, "requestedExitCode": 196},
            "scope": scope, "runtimeOwnerInventoryBefore": [self.keeper], "afterDeparture": self.absent(self.keeper),
            "exits": [{"role": name, "identity": identity, "handleIdentityVerified": True,
                       "nativeHandleRetained": True, "waitResult": 0, "processExitCode": 196 if name == "keeper" else 0}
                      for name, identity in roles.items()], **{name: False for name in recovery._FALSE}}
        self.retired_ref = self.put(self.state / "recovery-lifecycle/history/retirement.json", self.retired)
        return pins

    def prepare_restore(self):
        return recovery.prepare_prelaunch_upgrade_recovery(self.manager, expected_command_sha256=self.command_ref["sha256"],
            holder_reference=self.holder_ref, replacement_request_id=self.replacement_id,
            replacement_target=self.new_target, retirement_reference=self.retired_ref)


class PrelaunchUpgradeRecoveryTests(TestCase):
    def fixture(self):
        fixture = PrelaunchFixture()
        with fixture.observations():
            fixture.setup_failure()
        return fixture

    def test_restore_preserves_unknown_handoff_request_hold_bytes_and_reserves_exact_successor(self):
        fixture = self.fixture()
        with fixture.observations():
            fixture.capture_retirement()
            before = deepcopy(fixture.files)
            original = fixture.files[str(fixture.paths["terminalCommand"])]
            with fixture.digest_patch:
                capability = fixture.prepare_restore()
                self.assertEqual(before, fixture.files)
                result = capability.restore(fixture.manager)
            restored = json.loads(fixture.files[str(fixture.paths["terminalCommand"])])
            self.assertEqual("committed", restored["status"])
            receipt = recovery._receipt(fixture.state, result["reconciliation"])
            archive = receipt["archivedEvidence"]["originalCommand"]["archive"]
            self.assertEqual(original, fixture.files[archive["path"]])
            self.assertEqual("unknown", json.loads(fixture.files[archive["path"]])["status"])
            for name, raw in before.items():
                if name != str(fixture.paths["terminalCommand"]):
                    self.assertEqual(raw, fixture.files[name])
            self.assertEqual(fixture.holds, receipt["eligibility"]["protectedHolds"])
            self.assertTrue(all(result[name] is False for name in recovery._FALSE))
            with self.assertRaises(JenkinsError):
                capability.restore(fixture.manager)

    def test_false_wait_pid_absence_wrong_birth_or_scope_cannot_authorize_restore(self):
        for change in ({"nativeHandleRetained": False}, {"waitResult": 258}, {"waitResult": False},
                       {"processExitCode": 259}, {"keeper": {}}, {"scope": {}}, {"exits": []},
                       {"nativeTerminationProof": {}}, {"runtimeOwnerInventoryBefore": []}):
            with self.subTest(change=change):
                fixture = self.fixture()
                with fixture.observations():
                    fixture.capture_retirement()
                    fixture.retired.update(change)
                    fixture.retired_ref = fixture.put(Path(fixture.retired_ref["path"]), fixture.retired)
                    before = deepcopy(fixture.files)
                    with fixture.digest_patch, self.assertRaises((JenkinsError, KeyError)):
                        fixture.prepare_restore()
                    self.assertEqual(before, fixture.files)

    def test_source_holder_transaction_native_stop_hold_and_pointer_changes_refuse_eligibility(self):
        for change in ("source", "holder", "transaction", "native-eof", "hold", "pointer", "reason"):
            with self.subTest(change=change):
                fixture = self.fixture()
                with fixture.observations():
                    if change == "source":
                        name = next(iter(fixture.sources))
                        root = Path(fixture.new_target["root"]) if name.startswith("tools/jenkins/") else fixture.repo
                        fixture.files[str(root / name)] += b" "
                    elif change == "holder":
                        fixture.holder.update(message="missing process")
                        fixture.holder_ref = fixture.put(Path(fixture.holder_ref["path"]), fixture.holder)
                    elif change == "transaction":
                        fixture.put(fixture.state / "recovery-lifecycle/start-transactions/new.json",
                                    {"keeper": fixture.keeper, "request": fixture.command["request"]})
                    elif change == "native-eof":
                        value = json.loads(fixture.files[fixture.binding_ref["path"]])
                        value["nativeProofSynthesized"] = True
                        fixture.put(Path(fixture.binding_ref["path"]), value)
                    elif change == "hold":
                        fixture.holds[0]["version"] += 1
                    elif change == "pointer":
                        fixture.files[str(fixture.paths["stoppedOperation"])] += b" "
                    else:
                        fixture.command.update(reasonCode="supervisor_launch_failed")
                        fixture.command_ref = fixture.put(fixture.paths["terminalCommand"], fixture.command)
                    before = deepcopy(fixture.files)
                    with self.assertRaises((JenkinsError, KeyError)):
                        recovery._capture(fixture.manager, fixture.command_ref["sha256"], fixture.holder_ref)
                    self.assertEqual(before, fixture.files)

    def test_issued_capability_rechecks_cas_and_rejects_dictionary_or_subclass(self):
        fixture = self.fixture()
        with fixture.observations():
            fixture.capture_retirement()
            with fixture.digest_patch:
                capability = fixture.prepare_restore()
                fixture.files[str(fixture.paths["terminalCommand"])] += b" "
                with self.assertRaises(JenkinsError):
                    capability.restore(fixture.manager)
            with self.assertRaises(JenkinsError):
                recovery.PrelaunchUpgradeRecovery(canonical_json({}))
            class Forged(recovery.PrelaunchUpgradeRecovery):
                pass
            forged = object.__new__(Forged)
            with self.assertRaises(JenkinsError):
                forged.restore(fixture.manager)

    def test_gate_rejects_missing_receipt_wrong_request_target_and_release_replay(self):
        fixture = self.fixture()
        with fixture.observations():
            pins = fixture.capture_retirement()
            with fixture.digest_patch:
                result = fixture.prepare_restore().restore(fixture.manager)
            args = {"request_id": fixture.replacement_id,
                    "descriptor_sha256": pins["rawAnchors"]["originalBlockedDescriptor"]["sha256"],
                    "command_sha256": pins["restoredCommand"]["sha256"]}
            release = {**fixture.successor, "requestId": fixture.replacement_id,
                       "prelaunchReconciliation": result["reconciliation"]}
            release_ref = {"path": str(fixture.state / "deployment/next.json"), "sha256": "9" * 64}
            recovery.validate_prelaunch_restoration_gate(fixture.manager, release_ref, release, **args)
            for field, value in (("prelaunchReconciliation", None), ("targetDriver", {}),
                                 ("requestId", "e" * 32)):
                with self.subTest(field=field):
                    changed = {**release, field: value}
                    changed_args = {**args, "request_id": changed["requestId"]}
                    with self.assertRaises(JenkinsError):
                        recovery.validate_prelaunch_restoration_gate(fixture.manager, release_ref, changed, **changed_args)
            gate_path = fixture.state / recovery._GATE
            gate = json.loads(fixture.files[str(gate_path)])
            fixture.put(gate_path, {**gate, "status": "claimed", "claimedRelease": release_ref})
            with self.assertRaises(JenkinsError):
                recovery.validate_prelaunch_restoration_gate(fixture.manager,
                    {**release_ref, "sha256": "8" * 64}, release, **args)

    def test_actual_capture_uses_original_retirement_only_for_the_exact_reserved_new_request(self):
        fixture = self.fixture()
        with fixture.observations():
            fixture.capture_retirement()
            with fixture.digest_patch:
                result = fixture.prepare_restore().restore(fixture.manager)
            fixture.request_id = fixture.replacement_id
            fixture.successor.update(requestId=fixture.replacement_id, prelaunchReconciliation=result["reconciliation"])
            for name, checksum in fixture.successor["operatorFiles"].items():
                path = Path(name)
                if path.is_relative_to(fixture.repo / "tools/jenkins"):
                    fixture.regression_doc["sourceFiles"][path.relative_to(fixture.repo).as_posix()] = checksum
                if name in fixture.files and hashlib.sha256(fixture.files[name]).hexdigest() == checksum:
                    continue
                sealed = Path(fixture.new_target["root"]) / path.relative_to(fixture.repo)
                if str(sealed) in fixture.files:
                    fixture.files[name] = fixture.files[str(sealed)]
            fixture.successor_ref = fixture.put(fixture.successor_path, fixture.successor)
            plan = fixture.prepare()
            self.assertEqual(fixture.replacement_id, plan.document["requestId"])
            self.assertEqual(fixture.retirement_ref, plan.document["keeperRetirement"])
            plan.revalidate(fixture.manager)
            with self.assertRaises(JenkinsError):
                upgrade.prepare_external_native_stop_upgrade_start(fixture.manager,
                    request_id="e" * 32, successor_release_ref=fixture.successor_ref,
                    original_blocked_descriptor_ref=fixture.stopped_descriptor_ref,
                    committed_command_ref=fixture.committed_ref, external_native_stop_ref=fixture.binding_ref,
                    keeper_retirement_ref=fixture.retirement_ref)

            keeper = {**fixture.keeper, "pid": 12456, "creationTime": str(int(fixture.boot) + 400)}
            fixture.self_pid.return_value = keeper["pid"]
            fixture.self_identity.return_value = keeper
            request = fixture.put(fixture.state / "recovery-lifecycle/history/new-upgrade-request.json", {
                "schemaVersion": 1, "kind": upgrade._REQUEST_KIND, "requestId": fixture.replacement_id,
                "action": "upgrade", "keeper": keeper, "release": fixture.successor_ref,
                "ownedUpgradeDigest": plan.upgrade_digest, "protectedHoldsDigest": digest(fixture.holds),
                "runtimeAcceptance": False})
            actor = object.__new__(recovery_keeper.RecoveryKeeper)
            actor.manager, actor.repo, actor.state = fixture.manager, fixture.repo, fixture.state
            actor.release, actor.release_ref, actor.target = fixture.successor, fixture.successor_ref, fixture.new_target
            actor.keeper = keeper
            actor.command_path = fixture.paths["terminalCommand"]
            actor.descriptor = fixture.paths["stoppedDescriptor"]
            actor.runtime, actor.runtime_receipt, actor.blocked = None, None, False
            actor.original_interruption, actor.runtime_interruption, actor.owned_handoff = None, None, None
            actor.initial_digest, actor.upgrade_digest, actor.upgrade_plan = None, plan.upgrade_digest, plan
            actor._fresh = Mock()
            bound = actor._begin(request, plan=plan)
            bound.revalidate(fixture.manager)
            plan_ref = fixture.archive(fixture.state, "new-owned-upgrade-initial-plan", canonical_json(plan.document) + b"\n")
            descriptor = {"keeper": keeper, "release": fixture.successor_ref,
                "initialOwnedUpgrade": plan_ref, "ownedUpgradeDigest": plan.upgrade_digest,
                "ownedUpgradeHandoff": actor.owned_handoff["receipt"],
                "previousStoppedRuntimeReceipt": fixture.binding_ref}
            handoff = recovery_keeper._owned_handoff_receipt(fixture.manager, descriptor, fixture.successor)
            self.assertEqual(fixture.retirement_ref, handoff["keeperRetirement"])
            gate = json.loads(fixture.files[str(fixture.state / recovery._GATE)])
            self.assertEqual("claimed", gate["status"])
            self.assertEqual(fixture.successor_ref, gate["claimedRelease"])
            with self.assertRaises(JenkinsError):
                recovery.claim_prelaunch_restoration_gate(fixture.manager, plan, request, keeper)

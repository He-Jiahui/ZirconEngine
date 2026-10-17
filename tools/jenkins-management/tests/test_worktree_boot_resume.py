"""Resume the same managed input after a reviewed Windows boot boundary."""
from __future__ import annotations
import copy
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock, patch

import test_ordinary_flow as ordinary
runner = ordinary.runner


class WorktreeBootResumeTests(unittest.TestCase):
    def fixture(self, directory):
        actor, roots, state = ordinary.WorktreeLifecycleTests()._management(directory, operation="resume")
        actor.config["managedWorktree"] = runner.MANAGED_WORKTREE_CAPABILITY
        actor.config["driverDigest"] = "f" * 64
        actor.key = "n" * 64
        actor.input_digest = "i" * 64
        actor.action = "create-worktree"
        actor.payload.update(worktreeId="s-task")
        root = roots / "s-task"
        root.mkdir()
        old_input = {"repositoryId": actor.repo_id, "sessionId": "s", "requestId": "r-old", "operationId": "o-old",
                     "action": "create-worktree", "driverDigest": "d" * 64, "runtimeOperationId": "old-runtime"}
        key = runner.sha(runner.encoded({n: old_input[n] for n in ("repositoryId", "sessionId", "requestId", "operationId")}))
        actor.payload["resumeFromOperationKey"] = key
        old = {**old_input, "status": "ready", "path": str(root), "baseHead": "a" * 40,
               "branch": "codex/jenkins/s/task", "ownedPaths": ["owned.txt"], "creationOperationKey": key,
               "writerPrincipal": "synthetic-owner", "writerAuthorizationSnapshot": "e" * 64}
        source = {"head": "a" * 40, "files": {"owned.txt": "b" * 64}, "diffSha256": "c" * 64}
        source["sourceDigest"] = runner.sha(runner.encoded(source))
        result = {**old, "status": "complete", "worktreeId": "s-task", "operationKey": key, "source": source}
        operation = {"action": "create-worktree", "phase": "complete", "payload": old_input, "result": result,
                     "inputDigest": runner.sha(runner.encoded(old_input))}
        state.put("management_worktree", "s-task", old)
        state.put("management_operation", key, operation)
        actor.worktree_resume_lineage = Mock(return_value={
            "kind": "reviewed-later-boot", "previousDriverDigest": old["driverDigest"],
            "targetDriverDigest": actor.config["driverDigest"]})
        def text(path, *args):
            if "--git-common-dir" in args: return str(actor.repo / ".git")
            if args[:2] == ("symbolic-ref", "--short"): return "codex/jenkins/s/task"
            return "a" * 40
        actor.text = Mock(side_effect=text)
        actor.source = Mock(return_value=source)
        actor.git = Mock(side_effect=AssertionError("Resume cannot run Git mutations"))
        return actor, state, root, key

    def test_resume_keeps_the_same_worktree_and_completed_original_receipt(self):
        with tempfile.TemporaryDirectory(prefix="worktree-resume-") as directory:
            actor, state, root, key = self.fixture(directory)
            prior = copy.deepcopy(state.get("management_operation", key))
            old_record = copy.deepcopy(state.get("management_worktree", "s-task"))
            result = actor.new_worktree()
            self.assertEqual(result[:2], ("s-task", root))
            self.assertEqual(result[2]["runtimeOperationId"], "runtime")
            self.assertEqual(result[2]["driverDigest"], actor.config["driverDigest"])
            self.assertNotEqual(result[2]["driverDigest"], old_record["payload"]["driverDigest"])
            self.assertEqual(result[2]["resumedFrom"]["driverDigest"], old_record["payload"]["driverDigest"])
            self.assertEqual(result[2]["resumedFrom"]["recordDigest"], runner.sha(runner.encoded(old_record["payload"])))
            self.assertEqual(result[2]["resumedFrom"]["recovery"]["previousDriverDigest"], old_record["payload"]["driverDigest"])
            self.assertEqual(result[2]["resumedFrom"]["recovery"]["targetDriverDigest"], actor.config["driverDigest"])
            self.assertEqual(result[2]["writerPrincipal"], old_record["payload"]["writerPrincipal"])
            self.assertEqual(result[2]["writerAuthorizationSnapshot"], old_record["payload"]["writerAuthorizationSnapshot"])
            self.assertEqual(state.get("management_worktree", "s-task")["payload"], result[2])
            actor.worktree_resume_lineage.assert_called_once_with("old-runtime", prior)
            self.assertEqual(result[2]["creationOperationKey"], actor.worktree_writer_binding()["creationOperationKey"])
            self.assertEqual(result[2]["resumedFrom"]["creationOperationKey"], key)
            self.assertEqual(prior, state.get("management_operation", key))
            self.assertEqual(state.get("managed_worktree_writer", "s-task")["payload"]["status"], "released")
            actor.git.assert_not_called()

    def test_changed_source_retains_the_original_record(self):
        with tempfile.TemporaryDirectory(prefix="worktree-resume-changed-") as directory:
            actor, state, root, key = self.fixture(directory)
            prior = state.get("management_worktree", "s-task")
            actor.source.return_value = {"sourceDigest": "changed"}
            with self.assertRaisesRegex(ValueError, "source or physical Git identity changed"):
                actor.new_worktree()
            self.assertEqual(prior, state.get("management_worktree", "s-task"))
            actor.git.assert_not_called()

    def test_existing_reader_blocks_rebinding(self):
        with tempfile.TemporaryDirectory(prefix="worktree-resume-reader-") as directory:
            actor, state, root, key = self.fixture(directory)
            prior = state.get("management_worktree", "s-task")
            state.put("current_source_intent", "reader", {"worktreeId": "s-task"})
            with self.assertRaisesRegex(Exception, "reader freezes"):
                actor.new_worktree()
            self.assertEqual(prior, state.get("management_worktree", "s-task"))
            actor.source.assert_not_called()


    def test_explicit_resume_rejects_missing_registry_with_or_without_directory(self):
        for physical_directory in (True, False):
            with self.subTest(physical_directory=physical_directory):
                with tempfile.TemporaryDirectory(prefix="worktree-resume-no-record-") as directory:
                    actor, state, root, key = self.fixture(directory)
                    prior = copy.deepcopy(state.get("management_operation", key))
                    state.rows.pop(("management_worktree", "s-task"))
                    if not physical_directory:
                        root.rmdir()
                    with self.assertRaisesRegex(runner.WorktreeConflict, "resume requires an existing managed worktree record"):
                        actor.new_worktree()
                    self.assertIsNone(state.get("management_worktree", "s-task"))
                    self.assertEqual(prior, state.get("management_operation", key))
                    actor.source.assert_not_called()
                    actor.git.assert_not_called()



    def same_boot_fixture(self, directory):
        from contextlib import ExitStack
        from tools.jenkins.deployment import lost_recovery_runtime as lost, owned_upgrade as upgrade, recovery_keeper as keeper
        from tools.jenkins.processes import boot as boot_module
        actor, state, root, key = self.fixture(directory)
        del actor.__dict__["worktree_resume_lineage"]
        prior = state.get("management_operation", key)
        unknown_input = {**prior["payload"]["payload"], "action": "inspect-worktree", "worktreeId": "s-task",
            "requestId": "old-read", "operationId": "old-read", "ownedPaths": list(actor.owned)}
        unknown_key = runner.sha(runner.encoded({name: unknown_input[name] for name in
            ("repositoryId", "sessionId", "requestId", "operationId")}))
        state.put("management_operation", unknown_key, {"action": "inspect-worktree", "phase": "unknown",
            "payload": unknown_input, "inputDigest": runner.sha(runner.encoded(unknown_input)),
            "result": {"status": "unknown", "worktreeId": "s-task"}})
        pins = []
        for domain, reference in (("management_worktree", "s-task"),
                ("management_operation", key), ("management_operation", unknown_key)):
            row = state.get(domain, reference)
            pins.append({"domain": domain, "key": reference, "version": row["version"],
                         "payloadDigest": runner.sha(runner.encoded(row["payload"]))})
        identities = {role: {"pid": index + 100, "creationTime": str(index + 1001),
            "executable": str(root / (role + ".exe")), "commandDigest": str(index + 1) * 64}
            for index, role in enumerate(("supervisor", "controller", "agent", "keeper"))}
        old_selected = {"driverDigest": "d" * 64, "runtimeOperationId": "old-runtime"}
        current_selected = {"driverDigest": actor.config["driverDigest"], "runtimeOperationId": "runtime"}
        old_operation, operation = {"operationId": "old-runtime"}, {"operationId": "runtime"}
        old_host, host = {"prior": True}, {"current": True}
        old_runtime = {"state": "running", "operation": old_operation, "host": old_host,
            "selected": old_selected, "owner": {role: identities[role] for role in ("supervisor", "controller", "agent")}}
        old_release = {"path": "old-release", "sha256": "1" * 64}
        old_receipt_ref = {"path": "old-receipt", "sha256": "2" * 64}
        release_ref = {"path": "new-release", "sha256": "3" * 64}
        old_descriptor = {"schemaVersion": 1, "kind": keeper.DESCRIPTOR_KIND, "admission": "idle",
            "runtime": old_runtime, "keeper": identities["keeper"], "bootFileTime": "1000",
            "runtimeReceipt": old_receipt_ref, "release": old_release}
        old_receipt = {"kind": "recovery-keeper-runtime", "runtime": old_runtime,
            "keeper": identities["keeper"], "release": old_release, "bootFileTime": "1000",
            **{name: False for name in ("runtimeAcceptance", "cargoAcceptance", "nativeProofSynthesized",
                                       "historicalHoldsReleased", "queueReplayRequested")}}
        runtime = {"state": "running", "operation": operation, "host": host, "selected": current_selected}
        descriptor = {"schemaVersion": 1, "kind": keeper.DESCRIPTOR_KIND, "admission": "idle",
            "automaticRestart": False, "runtimeAcceptance": False, "cargoAcceptance": False,
            "historicalHoldsReleased": False, "runtime": runtime, "bootFileTime": "1000",
            "release": release_ref, "lostRecoveryRuntimeInterruption": {"path": "interruption", "sha256": "4" * 64}}
        plan = {"recoveryVariant": lost.SAME_BOOT_VARIANT, "originalCommandStatus": "committed",
            "operationId": "old-runtime", "previousBootFileTime": "1000", "bootFileTime": "1000",
            "previousRuntimeReceipt": old_receipt_ref, "previousRecoveryRelease": old_release,
            "selectedDriverDigest": runner.sha(runner.encoded(old_selected)),
            "owner": old_runtime["owner"], "departedKeeper": identities["keeper"],
            "departedIdentities": {role: {"pid": value["pid"], "expectedCreationTime": value["creationTime"],
                                        "status": "absent"} for role, value in identities.items()},
            "managementQuarantine": {"records": pins, "currentRecords": [
                {"domain": value["domain"], "key": value["key"]} for value in pins],
                "nativeProofIssued": False, "artifactAcceptance": False, "executionHoldsReleaseAllowed": False}}
        archived_values = {"originalDescriptor": old_descriptor, "previousRuntimeReceipt": old_receipt,
            "previousOperation": old_operation, "previousHost": old_host, "previousSelector": old_selected}
        archives = {name: {"archive": {"path": name, "sha256": "5" * 64},
                           "original": {"sha256": "6" * 64}} for name in archived_values}
        preparation_ref = {"path": "preparation", "sha256": "7" * 64}
        preparation = {"previousSelectedDriver": old_selected, "previousSelectorSha256": "6" * 64}
        release = {"preparation": preparation_ref, "lostRecoveryRuntime": {
            "plan": {"path": "plan", "sha256": "8" * 64}}}
        actor.manager = Mock()
        actor.manager._read.return_value = operation
        stack = ExitStack()
        self.addCleanup(stack.close)
        def read(path):
            relative = Path(path).relative_to(actor.repo / ".jenkins/state").as_posix()
            return {"formal-lifecycle/recovery-keeper.json": descriptor, "recovery-lifecycle/command.json": {},
                    "deployment/host.json": host, "deployment/driver.json": current_selected}[relative]
        stack.enter_context(patch.object(keeper, "_read", side_effect=read))
        stack.enter_context(patch.object(actor, "worktree_resume_release", return_value=(release, current_selected,
            {"kind": "historical-runtime-configuration-observation", "configurationChangedSinceRuntimeSpawn": False,
             "runtimeReissued": False, "serviceLifecyclePerformed": False})))
        stack.enter_context(patch.object(keeper, "_runtime_receipt", return_value={"current": True}))
        stack.enter_context(patch.object(keeper, "_reference", side_effect=lambda state, ref:
            preparation if ref == preparation_ref else archived_values[ref["path"]]))
        stack.enter_context(patch.object(lost, "reviewed_marker", return_value=plan))
        stack.enter_context(patch.object(lost, "validate_interruption_receipt", return_value={"originalArtifacts": archives}))
        stack.enter_context(patch.object(upgrade, "_committed_command"))
        departed = stack.enter_context(patch.object(upgrade, "_departure", side_effect=lambda identity:
            {"pid": identity["pid"], "expectedCreationTime": identity["creationTime"], "status": "absent"}))
        stack.enter_context(patch.object(boot_module, "windows_boot_filetime", return_value=1000))
        actor._resume_metadata = {"descriptor": descriptor, "release": release, "preparation": preparation,
            "archived": archived_values, "archives": archives, "patches": stack}
        return actor, state, root, key, unknown_key, plan, departed

    def test_same_boot_resume_preserves_read_only_unknown_and_original_authority(self):
        with tempfile.TemporaryDirectory(prefix="worktree-same-boot-") as directory:
            actor, state, root, key, unknown_key, plan, departed = self.same_boot_fixture(directory)
            old = copy.deepcopy(state.get("management_worktree", "s-task"))
            creation = copy.deepcopy(state.get("management_operation", key))
            unknown = copy.deepcopy(state.get("management_operation", unknown_key))
            state.put("resource_reservation", "historical", {"status": "active", "owner": "unrelated-history"})
            historical = copy.deepcopy(state.get("resource_reservation", "historical"))
            result = actor.new_worktree()
            self.assertEqual(("s-task", root), result[:2])
            recovery = result[2]["resumedFrom"]["recovery"]
            self.assertEqual("managed-worktree-same-boot-resume", recovery["kind"])
            self.assertEqual([unknown_key], recovery["preservedUnknownInspectOperationKeys"])
            self.assertEqual("1000", recovery["previousBootFileTime"])
            self.assertEqual("1000", recovery["bootFileTime"])
            self.assertFalse(recovery["nativeProofIssued"])
            self.assertFalse(recovery["oldNativeTerminationClaimed"])
            self.assertFalse(recovery["historicalEofClaimed"])
            self.assertFalse(recovery["historicalHoldsReleased"])
            self.assertEqual(8, departed.call_count)
            self.assertEqual(old["payload"]["writerPrincipal"], result[2]["writerPrincipal"])
            self.assertEqual(old["payload"]["writerAuthorizationSnapshot"], result[2]["writerAuthorizationSnapshot"])
            self.assertEqual(creation, state.get("management_operation", key))
            self.assertEqual(unknown, state.get("management_operation", unknown_key))
            self.assertEqual(historical, state.get("resource_reservation", "historical"))
            self.assertEqual("released", state.get("managed_worktree_writer", "s-task")["payload"]["status"])
            actor.git.assert_not_called()

    def test_same_boot_resume_rejects_native_claim_or_reader_and_foreign_writer(self):
        for obstacle in ("native", "reader", "writer"):
            with self.subTest(obstacle=obstacle), tempfile.TemporaryDirectory(prefix="worktree-same-boot-busy-") as directory:
                actor, state, root, key, unknown_key, plan, departed = self.same_boot_fixture(directory)
                prior = copy.deepcopy(state.get("management_worktree", "s-task"))
                if obstacle == "native":
                    state.put("native_job", "old-inspect-native", {"owner": "management-" + unknown_key,
                        "status": "running", "pid": 12345})
                elif obstacle == "reader":
                    state.put("current_source_intent", "reader", {"worktreeId": "s-task"})
                else:
                    state.put("managed_worktree_writer", "s-task", {"worktreeId": "s-task",
                        "operationKey": "foreign", "inputDigest": "f" * 64, "status": "active"})
                message = {"native": "execution or native claim", "reader": "reader freezes",
                           "writer": "active or interrupted writer"}[obstacle]
                with self.assertRaisesRegex(Exception, message):
                    actor.new_worktree()
                self.assertEqual(prior, state.get("management_worktree", "s-task"))
                actor.source.assert_not_called()
                actor.git.assert_not_called()

    def test_same_boot_resume_rejects_each_changed_reviewed_state_pin(self):
        for domain, selector in (("management_worktree", "worktree"), ("management_operation", "creation"),
                                 ("management_operation", "unknown")):
            with self.subTest(selector=selector), tempfile.TemporaryDirectory(prefix="worktree-same-boot-drift-") as directory:
                actor, state, root, key, unknown_key, plan, departed = self.same_boot_fixture(directory)
                target = "s-task" if selector == "worktree" else key if selector == "creation" else unknown_key
                row = state.get(domain, target)
                state.put(domain, target, {**row["payload"], "foreignEdit": True}, expected_version=row["version"])
                before = copy.deepcopy(state.rows)
                message = "creation differs from the reviewed predecessor" if selector == "creation" else "reviewed State pin changed"
                with self.assertRaisesRegex(ValueError, message):
                    actor.new_worktree()
                self.assertEqual(before, state.rows)
                actor.source.assert_not_called()
                actor.git.assert_not_called()



    def test_same_boot_metadata_resume_accepts_only_stable_historical_configuration(self):
        from contextlib import ExitStack
        from types import SimpleNamespace
        from tools.jenkins.deployment import recovery_keeper as keeper
        from tools.jenkins.state import State

        real_reference, real_read = keeper._reference, keeper._read
        real_runtime_receipt, real_release = keeper._runtime_receipt, keeper._release
        failures = {
            "deployment-spec": "Recovery specification changed",
            "workflow-spec": "Recovery specification changed",
            "strict-profile": "strict native creation proof",
            "strict-config": "Actual strict file operations",
            "config-race": "changed during observation",
            "operator": "operator bytes changed",
            "owned-upgrade": "metadata-only lost-runtime recovery",
            "failed-own-start": "metadata-only lost-runtime recovery",
            "release-hash": "Immutable recovery evidence changed",
            "hold-missing": "four protected historical holds",
            "hold-version": "four protected historical holds",
            "hold-payload": "four protected historical holds",
            "extra-reader": "reader freezes",
            "extra-writer": "active or interrupted writer",
            "extra-native": "execution or native claim",
            "same-config-extra-holds": "four protected historical holds",
        }
        extra_boundaries = {"unrelated-holds", "hold-missing", "hold-version", "hold-payload",
                            "extra-reader", "extra-writer", "extra-native", "same-config-extra-holds"}
        for boundary in ("valid", "unrelated-holds", *failures):
            with self.subTest(boundary=boundary), tempfile.TemporaryDirectory(prefix="worktree-config-history-") as directory:
                actor, state, root, key, unknown_key, plan, departed = self.same_boot_fixture(directory)
                fixture = actor._resume_metadata
                with fixture["patches"], ExitStack() as observations:
                    saved_mock = actor.__dict__.pop("worktree_resume_release")
                    observations.callback(setattr, actor, "worktree_resume_release", saved_mock)
                    observations.enter_context(patch.object(keeper, "_reference", real_reference))
                    observations.enter_context(patch.object(keeper, "_read", real_read))
                    observations.enter_context(patch.object(keeper, "_runtime_receipt", real_runtime_receipt))
                    state_root = actor.repo / ".jenkins/state"
                    hold_state = State(state_root / "coordination.sqlite3")

                    def hold_rows():
                        with hold_state.connect() as connection:
                            return [tuple(row) for row in connection.execute(
                                "SELECT domain,key,version,payload,updated_at FROM records ORDER BY domain,key")]

                    config_path = actor.repo / ".codex/config.toml"
                    spec_path = actor.repo / ".jenkins/deployment-spec.json"
                    workflow_path = actor.repo / ".jenkins/workflow-spec.json"

                    def write_bytes(path, raw):
                        path.parent.mkdir(parents=True, exist_ok=True)
                        path.write_bytes(raw)
                        return {"path": str(path), "sha256": runner.sha(raw)}

                    def archive(name, value):
                        return write_bytes(state_root / "recovery-lifecycle/history" / (name + ".json"),
                                           runner.encoded(value))

                    historical_config = write_bytes(config_path, b"[fixture]\nrevision = 'runtime-spawn'\n")
                    historical_config_bytes = config_path.read_bytes()
                    current_config = write_bytes(config_path, b"[fixture]\nrevision = 'metadata-resume'\n")
                    spec = write_bytes(spec_path, b'{"fixture":"deployment"}\n')
                    workflow = write_bytes(workflow_path, b'{"fixture":"workflow"}\n')
                    descriptor, release = fixture["descriptor"], fixture["release"]
                    preparation, archived = fixture["preparation"], fixture["archived"]
                    target = {"driverDigest": actor.config["driverDigest"],
                              "root": str(actor.repo / ".jenkins/runtime/drivers" / actor.config["driverDigest"])}
                    old_selected = archived["previousSelector"]
                    old_target = {"driverDigest": old_selected["driverDigest"],
                                  "root": str(actor.repo / ".jenkins/runtime/drivers" / old_selected["driverDigest"])}
                    old_selected.update(old_target)
                    selected_ref = archive("previous-selector", old_selected)
                    preparation.update(schemaVersion=1, state="prepared-not-selected", driver=target,
                        previousSelectorSha256=selected_ref["sha256"])
                    plan["selectedDriverDigest"] = runner.sha(runner.encoded(old_selected))
                    holds = [{"domain": "resource_reservation", "key": "retained-" + str(i), "version": 1,
                              "payload": {"status": "active", "owner": "retained-" + str(i)}} for i in range(4)]
                    for hold in holds:
                        state.put(hold["domain"], hold["key"], hold["payload"])
                        hold_state.put(hold["domain"], hold["key"], hold["payload"])
                    extra_holds = []
                    if boundary in extra_boundaries:
                        # Independent work owns both rows; neither belongs to s-task.
                        for domain, reference in (("resource_reservation", "active-other-reservation"),
                                                  ("pool_writer_hold", "active-other-writer")):
                            value = {"status": "active", "owner": "management-other-validation",
                                     "worktreeId": "other-worktree", "sourceRoot": str(root.parent / "other-worktree")}
                            state.put(domain, reference, value)
                            hold_state.put(domain, reference, value)
                            extra_holds.append((domain, reference))
                        self.assertEqual(6, len(keeper.protected_holds(state_root)))
                    preparation["protectedHolds"] = holds
                    required = (*keeper._OPERATOR_FILES, "tools/jenkins/deployment/lost_recovery_runtime.py")
                    operators, source_files = {}, {
                        ".jenkins/deployment-spec.json": spec["sha256"],
                        ".jenkins/workflow-spec.json": workflow["sha256"],
                    }
                    for name in (*required, ".jenkins/state/migration-baseline/jenkins_desktop_holder.py"):
                        canonical = actor.repo / name
                        physical = Path(target["root"]) / name if canonical.is_relative_to(actor.repo / "tools/jenkins") else canonical
                        # Opaque fixture evidence only; no production source tree is copied.
                        reference = write_bytes(physical, ("reviewed fixture " + name + "\n").encode())
                        operators[str(canonical)] = reference["sha256"]
                        source_files[name] = reference["sha256"]
                    regression_ref = archive("support-regression", {"fixture": "reviewed-support"})
                    preparation["supportRegression"] = regression_ref
                    fence = {name: "6" * 64 for name in {
                        "deployment-operation.json", "deployment/host.json", "deployment/driver.json",
                        *keeper._LEGACY_FILES}}
                    fence["deployment/driver.json"] = selected_ref["sha256"]
                    release.update(schemaVersion=1, kind=keeper.RELEASE_KIND, scope="execution-runtime",
                        requestId="1" * 32, config=historical_config, deploymentSpec=spec, workflowSpec=workflow,
                        targetDriver=target, actorCodeRoot=target["root"], operatorFiles=operators,
                        regression=regression_ref, protectedHolds=holds,
                        expectedPreviousOwner={"operation": archived["previousOperation"]},
                        expectedLifecycleFence=fence, runtimeAcceptance=False, cargoAcceptance=False,
                        automaticRestart=False, queueReplayRequested=False)
                    release["preparation"] = archive("prepared-target", preparation)
                    previous_release = archive("previous-release", {"config": historical_config})
                    archived["originalDescriptor"]["release"] = previous_release
                    archived["previousRuntimeReceipt"]["release"] = previous_release
                    previous_receipt = archive("previous-runtime-receipt", archived["previousRuntimeReceipt"])
                    archived["originalDescriptor"]["runtimeReceipt"] = previous_receipt
                    plan.update(previousRuntimeReceipt=previous_receipt, previousRecoveryRelease=previous_release)
                    for name, value in archived.items():
                        fixture["archives"][name]["archive"] = archive("archived-" + name, value)
                    fixture["archives"]["previousSelector"]["original"] = selected_ref
                    release["lostRecoveryRuntime"]["plan"] = archive("reviewed-plan", plan)
                    if boundary == "owned-upgrade":
                        release["kind"] = keeper.OWNED_UPGRADE_RELEASE_KIND
                        release["ownedUpgrade"] = {"fixture": "different-lifecycle"}
                    elif boundary == "failed-own-start":
                        release["failedOwnStart"] = {"fixture": "different-lifecycle"}
                    release_ref = archive("current-release", release)
                    runtime = descriptor["runtime"]
                    runtime["selected"] = {**target, "runtimeOperationId": actor.config["runtimeOperationId"]}
                    live = {role: {"pid": i + 200, "creationTime": str(i + 2001),
                        "executable": str(root / ("current-" + role + ".exe")), "commandDigest": "9" * 64}
                        for i, role in enumerate(("supervisor", "controller", "agent", "keeper"))}
                    runtime["owner"] = {role: live[role] for role in ("supervisor", "controller", "agent")}
                    descriptor.update(release=release_ref, keeper=live["keeper"], protectedHolds=holds)
                    probe = {"requestedProfile": "zircon-strict", "holderIdentity": live["keeper"],
                        "configuration": {"path": historical_config["path"], "updateOpenDenied": True,
                            "sha256Before": historical_config["sha256"], "sha256After": historical_config["sha256"]},
                        "allowedProbe": {"path": str(state_root / "desktop-holder" / ("probe-" + "4" * 32 + ".tmp")),
                                         "exclusiveCreateWrite": True, "removed": True},
                        "associatedJob": {"query": "current-process-associated-job", "holderMember": True,
                                          "memberPids": [live["keeper"]["pid"], live["supervisor"]["pid"]]}}
                    receipt = {"schemaVersion": 1, "kind": "recovery-keeper-runtime", "runtime": runtime,
                        "keeper": live["keeper"], "release": release_ref, "bootFileTime": descriptor["bootFileTime"],
                        "protectedHolds": holds, "lostRecoveryRuntimeInterruption": descriptor["lostRecoveryRuntimeInterruption"],
                        **{name: False for name in ("runtimeAcceptance", "cargoAcceptance", "historicalHoldsReleased",
                                                   "nativeProofSynthesized", "queueReplayRequested")},
                        "strictSpawn": {"creator": live["keeper"], "child": live["supervisor"],
                            "creationFlags": keeper.CREATION_FLAGS, "requestedProfile": "zircon-strict",
                            "creatorInJob": True, "childInJob": True,
                            "actualRestrictionPreflight": copy.deepcopy(probe),
                            "associatedJobAtSpawn": copy.deepcopy(probe)}}
                    if boundary == "strict-profile":
                        receipt["strictSpawn"]["requestedProfile"] = "full-access"
                    elif boundary == "strict-config":
                        receipt["strictSpawn"]["actualRestrictionPreflight"]["configuration"]["sha256Before"] = current_config["sha256"]
                    descriptor["runtimeReceipt"] = archive("current-runtime-receipt", receipt)
                    for name, value in (("formal-lifecycle/recovery-keeper.json", descriptor),
                            ("recovery-lifecycle/command.json", {}), ("deployment/host.json", runtime["host"]),
                            ("deployment/driver.json", runtime["selected"])):
                        write_bytes(state_root / name, runner.encoded(value))
                    actor.manager.paths = SimpleNamespace(repo=actor.repo, state=state_root, home=actor.repo / ".jenkins/home")
                    actor.manager.spec = SimpleNamespace(path=spec_path, controller={"listenAddress": "127.0.0.1", "httpPort": 18080})
                    for action in ("start", "stop", "restart", "_request"):
                        getattr(actor.manager, action).side_effect = AssertionError("Metadata resume cannot perform service actions")
                    changed_during_read = []
                    def verify_driver(path, *, expected_digest):
                        value = target if expected_digest == target["driverDigest"] else old_target
                        self.assertEqual(Path(value["root"]), path)
                        if boundary == "config-race" and not changed_during_read:
                            changed_during_read.append(True)
                            config_path.write_bytes(b"[fixture]\nrevision = 'changed-during-observation'\n")
                        return value
                    observations.enter_context(patch.object(keeper, "__file__", str(Path(target["root"]) / "tools/jenkins/deployment/recovery_keeper.py")))
                    observations.enter_context(patch.object(keeper, "verify_driver", side_effect=verify_driver))
                    observations.enter_context(patch.object(keeper, "_historical_regression", return_value={"sourceFiles": source_files}))
                    observations.enter_context(patch.object(keeper, "observe_live_identity"))
                    observations.enter_context(patch.object(keeper, "observe_endpoint_owner"))
                    if boundary == "deployment-spec":
                        spec_path.write_bytes(b'{"fixture":"foreign-deployment"}\n')
                    elif boundary == "workflow-spec":
                        workflow_path.write_bytes(b'{"fixture":"foreign-workflow"}\n')
                    elif boundary == "operator":
                        (Path(target["root"]) / "tools/jenkins/deployment/recovery_keeper.py").write_bytes(b"foreign operator")
                    elif boundary == "release-hash":
                        Path(release_ref["path"]).write_bytes(runner.encoded(release) + b"\n")
                    if boundary in {"hold-missing", "hold-version", "hold-payload"}:
                        original = holds[0]
                        with hold_state.connect() as connection:
                            if boundary == "hold-missing":
                                connection.execute("DELETE FROM records WHERE domain=? AND key=?",
                                                   (original["domain"], original["key"]))
                            elif boundary == "hold-version":
                                connection.execute("UPDATE records SET version=version+1 WHERE domain=? AND key=?",
                                                   (original["domain"], original["key"]))
                            else:
                                connection.execute("UPDATE records SET payload=? WHERE domain=? AND key=?",
                                    (runner.encoded({**original["payload"], "foreignEdit": True}).decode(),
                                     original["domain"], original["key"]))
                    elif boundary == "same-config-extra-holds":
                        config_path.write_bytes(historical_config_bytes)
                    elif boundary in {"extra-reader", "extra-writer", "extra-native"}:
                        # The metadata reader accepts six holds before the unchanged WT guards run.
                        actor.worktree_resume_release(descriptor["release"])
                        if boundary == "extra-reader":
                            state.put("current_source_intent", "reader", {"worktreeId": "s-task"})
                        elif boundary == "extra-writer":
                            state.put("managed_worktree_writer", "s-task", {"worktreeId": "s-task",
                                "operationKey": "foreign", "inputDigest": "f" * 64, "status": "active"})
                        else:
                            state.put("native_job", "old-inspect-native", {"owner": "management-" + unknown_key,
                                "status": "running", "pid": 12345})
                    before_state = copy.deepcopy(state.rows)
                    before_holds = hold_rows()
                    before_extras = {binding: hold_state.get(*binding) for binding in extra_holds}
                    if boundary in {"valid", "unrelated-holds"}:
                        # The lifecycle Start/Stop release reader must keep rejecting the changed config.
                        with self.assertRaisesRegex(Exception, "configuration or specification changed"):
                            real_release(actor.manager, release_ref, current_source=False)
                        result = actor.new_worktree()
                        self.assertEqual(("s-task", root), result[:2])
                        recovery = result[2]["resumedFrom"]["recovery"]
                        evidence = recovery["configurationEvidence"]
                        self.assertEqual(historical_config, evidence["historicalConfig"])
                        self.assertEqual(current_config, evidence["currentConfig"])
                        self.assertEqual(spec, evidence["currentDeploymentSpec"])
                        self.assertEqual(workflow, evidence["currentWorkflowSpec"])
                        self.assertTrue(evidence["configurationChangedSinceRuntimeSpawn"])
                        self.assertFalse(evidence["runtimeReissued"])
                        self.assertFalse(evidence["serviceLifecyclePerformed"])
                        current_identity = config_path.stat()
                        self.assertEqual({"device": current_identity.st_dev, "inode": current_identity.st_ino},
                                         evidence["currentConfigFileIdentity"])
                        self.assertEqual([unknown_key], recovery["preservedUnknownInspectOperationKeys"])
                        self.assertEqual(before_state[("management_worktree", "s-task")]["payload"]["writerPrincipal"],
                                         result[2]["writerPrincipal"])
                        self.assertEqual(before_state[("management_worktree", "s-task")]["payload"]["writerAuthorizationSnapshot"],
                                         result[2]["writerAuthorizationSnapshot"])
                        self.assertEqual("released", state.get("managed_worktree_writer", "s-task")["payload"]["status"])
                        self.assertEqual(8, departed.call_count)
                    else:
                        with self.assertRaisesRegex(Exception, failures[boundary]):
                            actor.new_worktree()
                        self.assertEqual(before_state, state.rows)
                        actor.source.assert_not_called()
                    for binding in (("management_operation", key), ("management_operation", unknown_key),
                                    *((hold["domain"], hold["key"]) for hold in holds)):
                        self.assertEqual(before_state[binding], state.rows[binding])
                    self.assertEqual(before_holds, hold_rows())
                    for binding, original in before_extras.items():
                        self.assertEqual(original, hold_state.get(*binding))
                    actor.git.assert_not_called()
                    for action in ("start", "stop", "restart", "_request"):
                        getattr(actor.manager, action).assert_not_called()


if __name__ == "__main__":
    unittest.main()

from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock, patch

import pytest

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment import independent_service as service
from tools.jenkins.deployment.manager import DeploymentManager, _supervisor_creation_flags


def test_pending_intake_keeps_original_identity_without_native_adoption():
    rows = [{"domain": "execution", "key": "original", "payload": {"status": "preparing"}},
            {"domain": "resource_reservation", "key": "original:original", "payload": {"owner": "original", "status": "active"}}]
    service._pending_without_launch(rows, "original")
    assert rows[1]["payload"]["status"] == "active"


@pytest.mark.parametrize("domain", ["execution_launch", "execution_host", "native_job"])
def test_pending_native_spawn_is_never_resubmitted(domain):
    rows = [{"domain": "execution", "key": "original", "payload": {"status": "preparing"}},
            {"domain": domain, "key": "original", "payload": {"status": "unknown"}}]
    with pytest.raises(JenkinsError, match="native launch"):
        service._pending_without_launch(rows, "original")


def test_replacement_stops_before_inventory_when_native_owner_is_live():
    snapshot = {"operation": {}, "host": {}, "selected": {"driverDigest": "driver", "root": "runtime/drivers/driver"}}
    manager = SimpleNamespace(paths=SimpleNamespace(runtime=Path("runtime"), state=Path("state")))
    with patch.object(service, "_snapshot", return_value=snapshot), \
            patch.object(service, "verify_driver"), patch.object(Path, "exists", return_value=False), \
            patch.object(service, "_owner", return_value={"supervisor": {}, "controller": {}, "agent": {}}), \
            patch.object(service, "_native_process_snapshot", side_effect=JenkinsError("live", "owner live")), \
            patch.object(service, "observe_runtime_owner_inventory") as inventory:
        with pytest.raises(JenkinsError, match="owner live"):
            service.capture_replacement(manager, {"driverDigest": "driver", "predecessorDigest": service.digest(snapshot)}, "policy")
        inventory.assert_not_called()
    # A controlled successor preserves the verified predecessor policy digest;
    # unrelated history still cannot be admitted by its exact receipt validator.
    import hashlib
    import json
    from tools.jenkins.deployment import job_provisioning, recovery

    raw_refs = {}
    def evidence(name, value):
        raw = json.dumps(value, sort_keys=True).encode("utf-8")
        raw_refs[name] = raw
        return {"path": name, "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}

    old_implementation = {"serviceSha256": "1" * 64, "jobProvisioningSha256": "2" * 64}
    implementation = {"serviceSha256": "3" * 64, "jobProvisioningSha256": "4" * 64}
    old_installer = evidence("old-installer.ps1", {"source": "old-reviewed-adapter"})
    old_policy = {"previousPolicySha256": ["5" * 64, "6" * 64],
                  "runtimeRebind": {"schemaVersion": 1, "controlImplementation": old_implementation,
                      "managementInstaller": old_installer, "reviewedRuntimeRebindInstallerSha256": old_installer["sha256"],
                      "codex": evidence("codex.exe", {"input": "unchanged-codex"}),
                      "activationRuntimeBefore": evidence("runtime-before", {"input": "unchanged-activation"})},
                  "foreignPolicy": {"preserve": True}}
    policy_ref = evidence("previous-policy", old_policy)
    initial_ref = evidence("previous-handoff", {"controlImplementation": old_implementation})
    previous = {"phase": "completed", "desiredState": "running",
                "startedOperationId": "original", "initialHandoffRef": initial_ref}
    previous_ref = evidence("previous-desired", previous)
    identity = {"pid": 20, "creationTime": "100", "executable": "python.exe", "commandDigest": "7" * 64}
    binding_ref = evidence("stop-binding", {"nativeProofSynthesized": False, "terminalHost": {}, "hostDeparture": {}})
    result_ref = evidence("stop-result", {})
    stopped_ref = evidence("stopped", {"state": "stopped", "operationId": "original",
                                     "stopBinding": binding_ref, "stopResult": result_ref})
    process_ref = evidence("resident-process", {
        "kind": "windows-native-resident-process-exit", "identity": identity,
        "taskInstanceStopAttempted": True, "rootProcessTerminal": True, "waitResult": 0,
        "exitTime": "101", "jobProofIssued": False, "eofProofIssued": False})
    manager = SimpleNamespace(paths=SimpleNamespace(repo=Path("repository"), runtime=Path("runtime"),
                                                   state=Path("state"), home=Path("home")))
    previous_refs = {"previousDesiredRef": previous_ref, "previousInitialHandoffRef": initial_ref,
                     "previousPolicyRef": policy_ref}
    plan_ref = evidence("next-plan", {
        "kind": "independent-service-resident-handoff-v1", "repository": str(manager.paths.repo),
        "taskName": "ZirconEngine-Jenkins-Service", "controlImplementation": implementation,
        "runtimeOperationId": "original", "driverDigest": "old", "resident": identity, **previous_refs})
    next_policy = {**old_policy, "previousPolicySha256": [*old_policy["previousPolicySha256"], policy_ref["sha256"]],
                   "runtimeRebind": {**old_policy["runtimeRebind"], "controlImplementation": implementation}}
    next_ref = evidence("next-policy", next_policy)
    receipt = {"kind": "independent-service-resident-handoff", "status": "predecessor-retired",
               "controlImplementation": implementation, "planRef": plan_ref, "residentProcessProof": process_ref,
               "stoppedRuntimeRef": stopped_ref, "policyRef": next_ref, **previous_refs}
    desired = {"initialHandoffRef": evidence("next-handoff", receipt)}
    old_driver = {"root": str(manager.paths.runtime / "drivers/old"), "driverDigest": "old"}
    verify = service._verify_initial_handoff
    read_ref = job_provisioning._service_ref_read
    with patch.object(service, "_control_implementation", return_value=implementation), \
            patch.object(service, "verify_driver", return_value=old_driver), \
            patch.object(service, "_verify_initial_handoff", return_value={"controlImplementation": old_implementation}) as previous_verify, \
            patch.object(service, "_raw", side_effect=lambda path: raw_refs[str(path)]), \
            patch.object(job_provisioning, "_service_ref_read", wraps=read_ref), \
            patch.object(service, "_sha", side_effect=lambda path: hashlib.sha256(raw_refs[str(path)]).hexdigest()), \
            patch.object(service, "_read", side_effect=lambda path: json.loads(raw_refs[str(path)])), \
            patch.object(recovery, "has_terminal_lifecycle_proofs", return_value=True):
        assert verify(manager, manager.paths.state / "independent-service", desired) == receipt
        previous_verify.assert_called_once_with(
            manager, manager.paths.state / "independent-service", previous, previous_driver=old_driver)
        # The explicit reviewed adapter changes exactly its two registration pins.
        reviewed = evidence("reviewed-installer.ps1", {"source": "reviewed-client-preserving-adapter"})
        reviewed_plan = {**json.loads(raw_refs[plan_ref["path"]]), "reviewedManagementInstallerRef": reviewed}
        reviewed_policy = {**next_policy, "runtimeRebind": {**next_policy["runtimeRebind"],
            "managementInstaller": reviewed, "reviewedRuntimeRebindInstallerSha256": reviewed["sha256"]}}
        reviewed_receipt = {**receipt, "planRef": evidence("reviewed-plan", reviewed_plan),
                            "policyRef": evidence("reviewed-policy", reviewed_policy)}
        reviewed_desired = {"initialHandoffRef": evidence("reviewed-handoff", reviewed_receipt)}
        assert verify(manager, manager.paths.state / "independent-service", reviewed_desired) == reviewed_receipt
        assert old_policy["runtimeRebind"]["managementInstaller"] == old_installer
        for key in ("codex", "activationRuntimeBefore"):
            assert reviewed_policy["runtimeRebind"][key] == old_policy["runtimeRebind"][key]
        forged = {**reviewed, "sha256": "0" * 64}
        forged_plan = {**reviewed_plan, "reviewedManagementInstallerRef": forged}
        forged_policy = {**reviewed_policy, "runtimeRebind": {**reviewed_policy["runtimeRebind"],
            "managementInstaller": forged, "reviewedRuntimeRebindInstallerSha256": forged["sha256"]}}
        forged_receipt = {**reviewed_receipt, "planRef": evidence("forged-plan", forged_plan),
                          "policyRef": evidence("forged-policy", forged_policy)}
        with pytest.raises(JenkinsError, match="Runtime rebind input changed"):
            verify(manager, manager.paths.state / "independent-service",
                   {"initialHandoffRef": evidence("forged-handoff", forged_receipt)})
        wrong_ref = evidence("unrelated-policy", {
            **next_policy, "previousPolicySha256": [*next_policy["previousPolicySha256"], "8" * 64]})
        wrong_desired = {"initialHandoffRef": evidence("unrelated-handoff", {**receipt, "policyRef": wrong_ref})}
        with pytest.raises(JenkinsError, match="Successor policy or previous completed binding differs"):
            verify(manager, manager.paths.state / "independent-service", wrong_desired)
    assert old_policy["previousPolicySha256"] == ["5" * 64, "6" * 64]



def test_capability_is_consumed_once_and_checks_fresh_evidence():
    manager = SimpleNamespace(paths=SimpleNamespace(state=Path("state")))
    evidence = {"original": "unchanged"}
    capability = service.IndependentServiceStart(service._ISSUER, manager, {}, "policy", evidence)
    with patch.object(service, "capture_replacement", return_value=evidence):
        capability.consume(manager)
        with pytest.raises(JenkinsError, match="already consumed"):
            capability.consume(manager)
    with patch.object(service, "capture_replacement", return_value={"original": "changed"}):
        with pytest.raises(JenkinsError, match="evidence changed"):
            capability.revalidate(manager)


def test_fake_capability_cannot_bypass_existing_keeper_authority():
    manager = object.__new__(DeploymentManager)
    with pytest.raises(JenkinsError) as error:
        manager._start_locked(Path("lock"), independent_service_start=object())
    assert error.value.code == "independent_service_invalid"


def test_normal_request_client_inventory_does_not_block_fresh_departure_proof():
    manager = SimpleNamespace(paths=SimpleNamespace(state=Path("state")))
    evidence = {"snapshot": "original", "departures": "confirmed", "runtimeInventoryDigest": "before"}
    current = {**evidence, "runtimeInventoryDigest": "after"}
    capability = service.IndependentServiceStart(service._ISSUER, manager, {}, "policy", evidence)
    with patch.object(service, "capture_replacement", return_value=current):
        capability.consume(manager)
    assert capability.evidence == current and capability.attempt["evidenceDigest"] == service.digest(current)


def test_resident_retries_after_health_or_disk_evidence_errors(tmp_path):
    manager = SimpleNamespace(paths=SimpleNamespace(state=tmp_path), _read=Mock(side_effect=OSError("temporary storage error")))
    written = []
    with patch.object(service, "process_lock") as lock, \
            patch.object(service, "current_identity", return_value=SimpleNamespace(to_dict=lambda: {"pid": 123})), \
            patch.object(service, "atomic_json", side_effect=lambda path, value: written.append(value)):
        lock.return_value.__enter__.return_value = None
        service.supervise(manager, tmp_path / "policy.json", interval=0, max_ticks=2)
    assert manager._read.call_count == 2
    assert len(written) == 2 and all(value["status"] == "recovery-pending" for value in written)


def test_service_supervisor_explicitly_breaks_caller_job_lifetime():
    import os
    import subprocess
    if os.name == "nt":
        assert _supervisor_creation_flags() & subprocess.CREATE_BREAKAWAY_FROM_JOB


def test_native_windows_task_keeps_its_own_service_job():
    import os
    import subprocess
    if os.name != "nt":
        pytest.skip("Windows native scheduler contract")
    capability = object.__new__(service.IndependentServiceStart)
    processes = {os.getpid(): {"parentPid": 20}, 20: {"pid": 20, "creationTime": 100, "imageName": "svchost.exe"}}
    with patch.object(service, "_native_process_snapshot", return_value=processes), \
            patch.object(service, "_system_process_image", return_value="native-image"), \
            patch.object(service, "_physical_dos_path", return_value="native-image"), \
            patch.object(service, "current_identity", side_effect=AssertionError("SYSTEM process handles are not required")):
        flags = capability.creation_flags()
    assert not flags & subprocess.CREATE_BREAKAWAY_FROM_JOB
    assert flags & subprocess.CREATE_NO_WINDOW


def test_scheduler_birth_change_rejects_native_task_context():
    import os
    if os.name != "nt":
        pytest.skip("Windows native scheduler contract")
    capability = object.__new__(service.IndependentServiceStart)
    first = {os.getpid(): {"parentPid": 20}, 20: {"pid": 20, "creationTime": 100, "imageName": "svchost.exe"}}
    second = {**first, 20: {**first[20], "creationTime": 101}}
    with patch.object(service, "_native_process_snapshot", side_effect=[first, second]), \
            patch.object(service, "_system_process_image", return_value="native-image"), \
            patch.object(service, "_physical_dos_path", return_value="native-image"):
        with pytest.raises(JenkinsError, match="identity changed"):
            capability.creation_flags()


def test_desktop_client_cannot_claim_native_task_lifetime():
    import os
    if os.name != "nt":
        pytest.skip("Windows native scheduler contract")
    capability = object.__new__(service.IndependentServiceStart)
    processes = {os.getpid(): {"parentPid": 20}, 20: {"pid": 20, "imageName": "codex.exe"}}
    with patch.object(service, "_native_process_snapshot", return_value=processes):
        with pytest.raises(JenkinsError, match="Task Scheduler child"):
            capability.creation_flags()


def test_failed_spawn_retry_retains_one_original_lifecycle_archive(tmp_path):
    import json
    original = {"operationId": "original", "state": "running"}
    host = {"status": "running"}
    previous = {"kind": "independent-native-service-interruption", "snapshot": {"operation": original, "host": host}}
    record = {"operation": original, "host": host, "deploymentRecoveryEvidence": previous}
    history = tmp_path / "deployment/history"
    history.mkdir(parents=True)
    archive = history / (service.digest("original") + "-" + service.digest(record) + ".json")
    archive.write_text(json.dumps(record), encoding="utf8")
    original_bytes = archive.read_bytes()
    manager = SimpleNamespace(paths=SimpleNamespace(state=tmp_path), _archive_deployment=Mock())
    capability = object.__new__(service.IndependentServiceStart)
    capability.manager = manager
    capability.archive_predecessor({**original, "state": "interrupted", "deploymentRecoveryEvidence": previous}, host, {})
    manager._archive_deployment.assert_not_called()
    assert list(history.iterdir()) == [archive] and archive.read_bytes() == original_bytes


def test_multiple_precreate_failures_follow_preserved_attempt_to_original_archive(tmp_path):
    import json
    original = {"operationId": "original", "state": "running"}
    host = {"status": "running"}
    first = {"kind": "independent-native-service-interruption", "snapshot": {"operation": original, "host": host}}
    interrupted = {**original, "state": "interrupted", "deploymentRecoveryEvidence": first}
    second = {"kind": "independent-native-service-interruption", "snapshot": {"operation": interrupted, "host": host}}
    history = tmp_path / "deployment/history"
    history.mkdir(parents=True)
    record = {"operation": original, "host": host, "deploymentRecoveryEvidence": first}
    archive = history / (service.digest("original") + "-" + service.digest(record) + ".json")
    archive.write_text(json.dumps(record), encoding="utf8")
    attempts = tmp_path / "independent-service/attempts"
    attempts.mkdir(parents=True)
    proof = attempts / "second-predecessor.json"
    proof.write_text(json.dumps(second), encoding="utf8")
    manager = SimpleNamespace(paths=SimpleNamespace(state=tmp_path), _archive_deployment=Mock())
    capability = object.__new__(service.IndependentServiceStart)
    capability.manager = manager
    current = {**interrupted, "deploymentRecoveryEvidence": second}
    capability.archive_predecessor(current, host, {})
    manager._archive_deployment.assert_not_called()
    assert len(list(history.iterdir())) == 1
    proof.write_text("{}", encoding="utf8")
    with pytest.raises(JenkinsError, match="no original attempt"):
        capability.archive_predecessor(current, host, {})


def test_failed_independent_controller_start_can_retry_without_invented_agent(tmp_path):
    import json
    identity = {"pid": 20, "creationTime": "100", "executable": "python.exe", "commandDigest": "a" * 64}
    home = tmp_path / "home"
    operation = {**identity, "operationId": "original", "generation": "original", "hostPid": 20,
                 "component": "lifecycle-host", "homePath": str(home.absolute()), "controlPlaneOnly": False,
                 "state": "failed"}
    host = {"status": "failed", "operationId": "original", "generation": "original", "hostPid": 20,
            "identity": {**identity, "pid": 21}, "agentLaunchAttempted": False, "agentIdentity": None}
    selected = {"runtimeOperationId": "original", "driverDigest": "driver"}
    state = tmp_path / "state"
    (state / "deployment").mkdir(parents=True)
    (state / "independent-service").mkdir()
    journal = state / "operation.json"
    journal.write_text(json.dumps(operation), encoding="utf8")
    (state / "deployment/host.json").write_text(json.dumps(host), encoding="utf8")
    (state / "deployment/driver.json").write_text(json.dumps(selected), encoding="utf8")
    current = state / "independent-service/current.json"
    attempt = {"phase": "failed", "operationId": "original", "identity": identity}
    current.write_text(json.dumps(attempt), encoding="utf8")
    manager = SimpleNamespace(paths=SimpleNamespace(state=state, home=home), journal=journal)
    snapshot = service._snapshot(manager)
    assert service._owner(snapshot)["agent"] is None
    current.write_text(json.dumps({**attempt, "operationId": "foreign"}), encoding="utf8")
    with pytest.raises(JenkinsError, match="bound full Jenkins runtime"):
        service._snapshot(manager)

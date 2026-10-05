"""Recover ownership without treating recovery evidence as build acceptance."""
from __future__ import annotations

import time
from pathlib import Path

from ..contracts import JenkinsError, digest
from ..processes.boot import windows_boot_filetime


# This immutable driver was reviewed for anonymous, non-inherited,
# non-breakaway, atomic KILL_ON_JOB_CLOSE containment of the controller.
# Unknown historical launch implementations do not inherit this exception.
_AUDITED_PRE_AGENT_DRIVERS = {
    "a71820c8fd7b298ff2f821f414500298f799bf092526cd2f34c3c30e785d6029",
}


def _verified_startup_containment(selected: dict, operation: dict, repository: Path) -> str:
    from .driver import verify_driver
    driver_digest = selected.get("driverDigest")
    root = repository / ".jenkins/runtime/drivers" / str(driver_digest)
    if (driver_digest not in _AUDITED_PRE_AGENT_DRIVERS
            or selected.get("runtimeOperationId") != operation.get("operationId")
            or Path(selected.get("root", "")).absolute() != root.absolute()):
        raise JenkinsError("deployment_startup_containment_unproven", "The historical controller containment has not been verified")
    verify_driver(root, expected_digest=driver_digest)
    return driver_digest


def abandoned_startup_evidence(operation: dict, host: dict, home_path: str,
                               repository: Path, selected: dict, address: str, port: int) -> dict:
    """Permit a quiet control-plane start after a contained, pre-agent failure.

    The departed host's last private Job handle kills its controller tree.
    This is not an EOF receipt and cannot release historical execution holds.
    """
    from .lifecycle import check_port_available
    from .startup_observations import observe_departed_identity, runtime_owner_inventory, observe_home_available
    repository = Path(repository).absolute()
    controller = host.get("controllerIdentity") or host.get("identity")
    if (operation.get("homePath") != home_path
            or operation.get("operationId") != host.get("operationId")
            or not operation.get("operationId")
            or operation.get("generation") != operation.get("operationId")
            or operation.get("generation") != host.get("generation")
            or operation.get("pid") != operation.get("hostPid")
            or operation.get("hostPid") != host.get("hostPid")
            or host.get("status") != "starting"
            or host.get("agentLaunchAttempted") is not False or host.get("agentIdentity")):
        raise JenkinsError("deployment_recovery_owner_mismatch", "This was not a verified pre-agent controller startup")
    for identity, executable in ((operation, repository / ".jenkins/runtime/python/python.exe"),
                                 (controller, repository / ".jenkins/runtime/jdk/bin/java.exe")):
        if (not isinstance(identity, dict) or type(identity.get("pid")) is not int or identity["pid"] <= 0
                or not str(identity.get("creationTime", "")).isdigit() or int(identity["creationTime"]) <= 0
                or Path(identity.get("executable", "")).absolute() != executable.absolute()
                or len(str(identity.get("commandDigest", ""))) != 64):
            raise JenkinsError("deployment_recovery_identity_missing", "The exact startup identities are unavailable")
    containment = _verified_startup_containment(selected, operation, repository)
    observations = [observe_departed_identity(identity) for identity in (operation, controller)]
    inventory = runtime_owner_inventory(repository)
    if inventory:
        raise JenkinsError("deployment_home_owner_live", "Another repository runtime still owns the deployment", retryable=True)
    home = observe_home_available(Path(home_path))
    check_port_available(address, port)
    # Re-observe after inventory and exclusive file probes; PID-only absence
    # and inaccessible observations are never sufficient.
    observations = [observe_departed_identity(identity) for identity in (operation, controller)]
    return {"kind": "same-boot-control-plane-recovery", "operationId": operation["operationId"],
            "homePath": home_path, "hostRecordDigest": digest(host), "operationDigest": digest(operation),
            "containedDriverDigest": containment, "departedIdentities": observations,
            "runtimeOwnerInventory": inventory, "inventoryDigest": digest(inventory),
            "homeObservation": home, "endpointAvailable": {"address": address, "port": port},
            "agentLaunchAttempted": False, "controlPlaneOnly": True, "nativeProofIssued": False,
            "validationAcceptanceAllowed": False, "executionHoldsReleaseAllowed": False,
            "observedAt": time.time(), "outcome": "interrupted"}


def replacement_deployment_evidence(operation: dict, host: dict, home_path: str,
                                    repository: Path, selected: dict, address: str, port: int) -> dict:
    if (host.get("status") == "starting" and host.get("agentLaunchAttempted") is False
            and not host.get("agentIdentity") and selected.get("driverDigest") in _AUDITED_PRE_AGENT_DRIVERS):
        return abandoned_startup_evidence(operation, host, home_path, repository, selected, address, port)
    return previous_boot_deployment_evidence(operation, host, home_path)


def has_terminal_deployment_proofs(operation: dict, host: dict, home_path: str) -> bool:
    """A terminal label alone cannot authorize replacing a lifecycle owner."""
    if (not isinstance(operation.get("operationId"), str) or not operation["operationId"]
            or not isinstance(operation.get("generation"), str) or not operation["generation"]
            or operation.get("homePath") != home_path
            or operation.get("operationId") != host.get("operationId")
            or operation.get("generation") != host.get("generation")
            or host.get("status") not in {"stopped", "failed"}):
        return False

    def complete(proof):
        identity = proof.get("identity") if isinstance(proof, dict) else None
        return bool(identity and type(identity.get("pid")) is int and identity["pid"] > 0
            and str(identity.get("creationTime", "")).isdigit() and int(identity["creationTime"]) > 0
            and identity.get("executable") and proof.get("complete") is True
            and proof.get("childrenGone") is True and type(proof.get("activeProcesses")) is int
            and proof["activeProcesses"] == 0
            and proof.get("stdoutEof") is True and proof.get("stderrEof") is True
            and type(proof.get("processExitCode")) is int)

    controller = host.get("nativeTerminationProof")
    identity = host.get("controllerIdentity") or host.get("identity")
    if not complete(controller) or not identity or controller["identity"] != identity:
        return False
    agent = host.get("agentTerminationProof")
    if agent is not None:
        return bool(host.get("agentIdentity") and host.get("agentLaunchAttempted") is not False
                    and complete(agent) and agent["identity"] == host["agentIdentity"])
    return host.get("agentLaunchAttempted") is False and not host.get("agentIdentity")


def has_terminal_lifecycle_proofs(operation: dict, host: dict, home_path: str,
                                  host_observation: dict | None = None) -> bool:
    """Require native component proofs and an exact supervisor departure observation."""
    if not has_terminal_deployment_proofs(operation, host, home_path) or not isinstance(host_observation, dict):
        return False
    pid, birth = operation.get("hostPid"), operation.get("creationTime")
    if (type(pid) is not int or pid <= 0 or operation.get("pid") != pid
            or host.get("hostPid") != pid or not str(birth or "").isdigit() or int(birth) <= 0):
        return False
    if host_observation.get("pid") != pid or str(host_observation.get("expectedCreationTime")) != str(birth):
        return False
    if host_observation.get("status") == "absent":
        return True
    observed = host_observation.get("observedIdentity")
    return (host_observation.get("status") == "pid-reused" and isinstance(observed, dict)
            and observed.get("pid") == pid and str(observed.get("creationTime", "")).isdigit()
            and int(observed["creationTime"]) > 0 and str(observed["creationTime"]) != str(birth))


def previous_boot_deployment_evidence(operation: dict, host: dict, home_path: str) -> dict:
    if (operation.get("homePath") != home_path
            or operation.get("operationId") != host.get("operationId")
            or operation.get("generation") != host.get("generation")
            or operation.get("hostPid") != host.get("hostPid")):
        raise JenkinsError("deployment_recovery_owner_mismatch", "Lost deployment ownership is unproven")
    identities = [operation, host.get("controllerIdentity") or host.get("identity")]
    if host.get("agentIdentity") or host.get("agentLaunchAttempted") is not False:
        identities.append(host.get("agentIdentity"))
    if any(not isinstance(i, dict) or not i.get("pid") or not str(i.get("creationTime", "")).isdigit()
           for i in identities):
        raise JenkinsError("deployment_recovery_identity_missing", "Recovery requires every durable component identity")
    boot = windows_boot_filetime()
    if any(int(i["creationTime"]) <= 0 or int(i["creationTime"]) >= boot for i in identities):
        raise JenkinsError("deployment_termination_unproven",
            "The lost lifecycle owner belongs to this Windows boot; preserve the deployment until a real reboot",
            retryable=True)
    return {"kind": "windows-boot-boundary", "operationId": operation["operationId"],
        "homePath": home_path, "bootFileTime": str(boot),
        "identities": [{k: i.get(k) for k in ("pid", "creationTime", "executable", "commandDigest")}
                       for i in identities], "hostRecordDigest": digest(host),
        "observedAt": time.time(), "outcome": "interrupted", "validationAcceptanceAllowed": False}

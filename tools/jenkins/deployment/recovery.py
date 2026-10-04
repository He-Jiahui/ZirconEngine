"""Recover a lost lifecycle owner only across a verified Windows boot."""
from __future__ import annotations

import time

from ..contracts import JenkinsError, digest
from ..processes.boot import windows_boot_filetime


def has_terminal_deployment_proofs(operation: dict, host: dict, home_path: str) -> bool:
    """A terminal label alone cannot authorize replacing a lifecycle owner."""
    if (operation.get("homePath") != home_path
            or operation.get("operationId") != host.get("operationId")
            or operation.get("generation") != host.get("generation")
            or host.get("status") not in {"stopped", "failed"}):
        return False

    def complete(proof):
        identity = proof.get("identity") if isinstance(proof, dict) else None
        return bool(identity and type(identity.get("pid")) is int and identity["pid"] > 0
            and str(identity.get("creationTime", "")).isdigit() and int(identity["creationTime"]) > 0
            and identity.get("executable") and proof.get("complete") is True
            and proof.get("childrenGone") is True and proof.get("activeProcesses") == 0
            and proof.get("stdoutEof") is True and proof.get("stderrEof") is True
            and type(proof.get("processExitCode")) is int)

    controller = host.get("nativeTerminationProof")
    identity = host.get("controllerIdentity") or host.get("identity")
    if not complete(controller) or not identity or controller["identity"] != identity:
        return False
    agent = host.get("agentTerminationProof")
    if agent is not None:
        return complete(agent) and (not host.get("agentIdentity") or agent["identity"] == host["agentIdentity"])
    return host.get("agentLaunchAttempted") is False and not host.get("agentIdentity")


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

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



# This is an owner-issued, immutable audit for one lost management generation.
# It grants no terminal proof and cannot authorize releasing native build holds.
_CONTROL_PLANE_MANAGEMENT_AUDITS = {
    "7ec2af2f6dd0b96ef7dc99a155f7626cec902fd7a08aeca452288690ebfe427d": (
        '.jenkins/state/deployment/history/control-plane-client-audit-c28a8efd6f33d4d51958a6bc6cd8978a0898c0ba6c2bebb740ab08927c92529a.json',
        '98fabbb691ee8304eb9cd61db8bfe596ade1079db9c5bbd3417d9a8045bd8193',
    ),
}

# Each explicit owner review applies only to its exact lost generation.
_CONTROL_PLANE_MANAGEMENT_GENERATION_AUDITS = {
    ("7ec2af2f6dd0b96ef7dc99a155f7626cec902fd7a08aeca452288690ebfe427d",
     "start-1791461349169992100"): (
        ".jenkins/state/deployment/history/control-plane-audit-bb3bde857913ba82b84afc258d7d7f2cfbf83c09f9036b94157eb06e67c6dadd.json",
        "69cd849483f91ce709a16877013a98b08704022290fe598af1e663ae563bed8c",
    ),
    ("7ec2af2f6dd0b96ef7dc99a155f7626cec902fd7a08aeca452288690ebfe427d",
     "start-1791396148879125100"): (
        ".jenkins/state/deployment/history/control-plane-audit-10a51d28bd833259b7dfbf298664fa7160b3c44e023a61e00b790703c1864545.json",
        "73139ea32ca7aadea2fe1f964d7206c2dddeb88aa8ff6703fe79f5d908387c45",
    ),
    ("7ec2af2f6dd0b96ef7dc99a155f7626cec902fd7a08aeca452288690ebfe427d",
     "start-1791319602046794700"): (
        ".jenkins/state/deployment/history/control-plane-audit-501812cebc919a6895d2ae30d0cf41a17e268809f22716ddfa65d15c709d494e.json",
        "37f9d8a8228f048d305a0681be033ef5083b915a9c35026c93973a9fbf607074",
    ),
}


def _management_audit(repository: Path, operation: dict, host: dict,
                      home_path: str, selected: dict) -> tuple[dict, dict]:
    import hashlib
    import json
    import re
    import xml.etree.ElementTree as ET
    from .driver import _plain, verify_driver

    driver = selected.get("driverDigest")
    entry = _CONTROL_PLANE_MANAGEMENT_GENERATION_AUDITS.get((driver, operation.get("operationId")))
    if entry is None:
        entry = _CONTROL_PLANE_MANAGEMENT_AUDITS.get(driver)
    if entry is None:
        raise JenkinsError("deployment_management_audit_missing", "This management generation has no verified audit")
    relative, expected_sha = entry
    audit_path = repository / relative
    _plain(repository)
    _plain(audit_path)
    try:
        raw = audit_path.read_bytes()
        if hashlib.sha256(raw).hexdigest() != expected_sha:
            raise ValueError("audit bytes")
        audit = json.loads(raw)
        root = repository / ".jenkins/runtime/drivers" / driver
        if (audit.get("schemaVersion") != 1 or audit.get("kind") != "sealed-control-plane-management-audit"
                or audit.get("scope") != "quiet-management-only"
                or audit.get("driverDigest") != driver
                or audit.get("runtimeOperationId") != operation.get("operationId")
                or selected.get("runtimeOperationId") != operation.get("operationId")
                or Path(selected.get("root", "")).absolute() != root
                or Path(audit.get("repositoryRoot", "")).absolute() != repository
                or Path(audit.get("homePath", "")).absolute() != Path(home_path).absolute()):
            raise ValueError("audit context")
        for field in ("nativeProofIssued", "historicalEofClaimed",
                      "validationAcceptanceAllowed", "executionHoldsReleaseAllowed"):
            if audit.get(field) is not False:
                raise ValueError("audit acceptance boundary")
        for field, observed in (("operationBinding", operation), ("hostBinding", host)):
            binding = audit.get(field)
            if not isinstance(binding, dict) or not binding:
                raise ValueError("audit owner binding missing")
            if any(observed.get(key) != value for key, value in binding.items()):
                raise ValueError("audit owner changed")
        controller = host.get("controllerIdentity") or host.get("identity")
        if (not operation.get("operationId") or operation.get("generation") != operation.get("operationId")
                or host.get("operationId") != operation.get("operationId")
                or host.get("generation") != operation.get("generation")
                or operation.get("homePath") != home_path
                or operation.get("pid") != operation.get("hostPid")
                or operation.get("hostPid") != host.get("hostPid")
                or operation.get("controlPlaneOnly") is not True
                or host.get("controlPlaneOnly") is not True
                or host.get("agentLaunchAttempted") is not False
                or host.get("agentIdentity") is not None):
            raise ValueError("management owner context")
        for identity, executable in ((operation, repository / ".jenkins/runtime/python/python.exe"),
                                     (controller, repository / ".jenkins/runtime/jdk/bin/java.exe")):
            if (not isinstance(identity, dict) or type(identity.get("pid")) is not int or identity["pid"] <= 0
                    or not str(identity.get("creationTime", "")).isdigit()
                    or int(identity["creationTime"]) <= 0
                    or Path(identity.get("executable", "")).absolute() != executable
                    or not re.fullmatch("[0-9a-f]{64}", str(identity.get("commandDigest", "")))):
                raise ValueError("management exact identity")
        verify_driver(root, expected_digest=driver)
        if hashlib.sha256((root / "driver-manifest.json").read_bytes()).hexdigest() != audit.get("manifestSha256"):
            raise ValueError("audited manifest changed")
        critical = audit.get("criticalFiles")
        required = {"tools/jenkins/processes/native.py", "tools/jenkins/processes/job.py",
                    "tools/jenkins/deployment/host.py", "tools/jenkins/deployment/lifecycle.py"}
        if not isinstance(critical, dict) or set(critical) != required:
            raise ValueError("audited containment sources missing")
        for name, source_sha in critical.items():
            if hashlib.sha256((root / name).read_bytes()).hexdigest() != source_sha:
                raise ValueError("audited containment source changed")
        probe_ref = audit.get("probe") or {}
        probe_path = Path(probe_ref.get("path", "")).absolute()
        probe_root = repository / ".jenkins/state/migration-baseline"
        _plain(probe_path)
        if probe_path.parent != probe_root or not probe_path.name.startswith("controller-containment-probe-"):
            raise ValueError("probe path")
        probe_raw = probe_path.read_bytes()
        if hashlib.sha256(probe_raw).hexdigest() != probe_ref.get("sha256"):
            raise ValueError("probe bytes")
        probe = json.loads(probe_raw)
        if (probe.get("driverDigest") != driver or probe.get("manifestSha256") != audit.get("manifestSha256")
                or probe.get("criticalFiles") != critical):
            raise ValueError("probe source identity")
        for name in ("accepted", "rootAssignedBeforeResume", "rootJobMembershipBeforeResume",
                     "anonymousJob", "lastHandleCloseObserved", "stdoutEof", "stderrEof"):
            if probe.get(name) is not True:
                raise ValueError("probe containment missing")
        for name in ("historicalNativeProofIssued", "validationAcceptanceAllowed", "executionHoldsReleaseAllowed"):
            if probe.get(name) is not False:
                raise ValueError("probe acceptance boundary")
        announcement = probe.get("fixtureAnnouncement") or {}
        members = probe.get("membersBeforeClose")
        exits = probe.get("retainedMemberExitCodes")
        if (type(probe.get("jobHandleFlags")) is not int or probe["jobHandleFlags"] & 1
                or type(probe.get("limitFlags")) is not int or probe["limitFlags"] != 0x2000
                or announcement.get("breakawayRejected") is not True or announcement.get("breakawayWinerror") != 5
                or type(probe.get("activeProcessesBeforeClose")) is not int or probe["activeProcessesBeforeClose"] != 2
                or not isinstance(members, list) or len(members) != 2
                or not isinstance(exits, dict) or len(exits) != 2
                or any(type(code) is not int or code == 259 for code in exits.values())):
            raise ValueError("probe kernel policy")
        policy = audit.get("controlPolicy") or {}
        home = Path(home_path).absolute()
        init = home / "init.groovy.d/90-zircon-repository-build-root.groovy"
        _plain(home)
        _plain(init)
        _plain(home / "config.xml")
        init_bytes = init.read_bytes()
        if (policy.get("controllerExecutors") != 0 or policy.get("agentLaunchAttempted") is not False
                or policy.get("brokerDisabled") is not True
                or policy.get("requiredStartEnvironment") != {"ZIRCON_JENKINS_RECOVERY_CONTROL_ONLY": "1"}
                or hashlib.sha256(init_bytes).hexdigest() != policy.get("quietInitSha256")
                or b"doQuietDown" not in init_bytes
                or ET.parse(home / "config.xml").getroot().findtext("numExecutors") != "0"):
            raise ValueError("quiet management policy changed")
    except (OSError, ValueError, KeyError, TypeError, ET.ParseError) as error:
        raise JenkinsError("deployment_management_audit_invalid", "The management recovery audit does not match") from error
    return audit, {"path": str(audit_path), "sha256": expected_sha}


def _management_execution_inventory(repository: Path, operation_id: str) -> dict:
    """Read existing records without initializing State or changing a quarantine."""
    import json
    import sqlite3
    from .driver import _plain
    path = repository / ".jenkins/state/coordination.sqlite3"
    _plain(path)
    if not path.exists():
        return {"recordsDigest": digest([]), "protectedHistoricalRows": [], "nativeIdentityBindings": []}
    active = {"pending", "admitted", "claimed", "launching", "started", "running", "cancel_requested", "reconciling"}
    try:
        from contextlib import closing
        with closing(sqlite3.connect(path.as_uri() + "?mode=ro", uri=True)) as connection:
            rows = [(domain, key, json.loads(payload)) for domain, key, payload in connection.execute(
                "SELECT domain,key,payload FROM records WHERE domain IN "
                "('execution','execution_host','execution_launch','native_job','resource_reservation','pool_writer_hold') "
                "ORDER BY domain,key")]
        protected = []
        native_bindings = []

        def collect_native_bindings(value, domain, key, location=""):
            if isinstance(value, dict):
                for pid_key in ("pid", "hostPid", "ownerPid"):
                    pid = value.get(pid_key)
                    if type(pid) is int and pid > 0:
                        native_bindings.append({"domain": domain, "key": key,
                            "location": location + "/" + pid_key, "pid": pid,
                            "creationTime": value.get("creationTime")})
                for field in sorted(value):
                    collect_native_bindings(value[field], domain, key, location + "/" + field)
            elif isinstance(value, list):
                for index, child in enumerate(value):
                    collect_native_bindings(child, domain, key, location + "/" + str(index))
        for domain, key, payload in rows:
            if not isinstance(payload, dict):
                raise ValueError("execution quarantine payload is not an object")
            binding = payload.get("operationId", payload.get("runtimeOperationId"))
            generation = payload.get("generation")
            if domain in {"execution", "execution_host", "execution_launch"}:
                if binding == operation_id or generation == operation_id:
                    raise JenkinsError("deployment_management_execution_active",
                                       "The lost management generation has an execution owner", retryable=True)
                if payload.get("status") in active and (not binding or not generation):
                    raise JenkinsError("deployment_management_execution_ambiguous",
                                       "Active execution ownership is unavailable", retryable=True)
            protected.append({"domain": domain, "key": key, "payloadDigest": digest(payload)})
            collect_native_bindings(payload, domain, key)
    except (sqlite3.Error, ValueError, TypeError) as error:
        raise JenkinsError("deployment_management_state_unavailable",
                           "The existing execution quarantine cannot be inspected", retryable=True) from error
    return {"recordsDigest": digest(rows), "protectedHistoricalRows": protected,
            "nativeIdentityBindings": native_bindings}


def _management_lifecycle_fence(repository: Path, operation: dict, host: dict, selected: dict) -> dict:
    """Observe the files serialized by the caller's deployment-start lock."""
    import hashlib
    import json
    from .driver import _plain
    paths = {"operation": repository / ".jenkins/state/deployment-operation.json",
             "host": repository / ".jenkins/state/deployment/host.json",
             "selector": repository / ".jenkins/state/deployment/driver.json"}
    raw = {}
    values = {}
    try:
        for name, path in paths.items():
            _plain(path)
            raw[name] = path.read_bytes()
            values[name] = json.loads(raw[name])
        fields = ("operationId", "generation", "pid", "hostPid", "creationTime",
                  "executable", "commandDigest", "homePath", "controlPlaneOnly")
        if (values["selector"] != selected or values["host"] != host
                or any(values["operation"].get(field) != operation.get(field) for field in fields)):
            raise ValueError("lifecycle binding changed")
    except (OSError, ValueError, TypeError, AttributeError) as error:
        raise JenkinsError("deployment_management_lifecycle_changed",
                           "The serialized management lifecycle binding changed", retryable=True) from error
    return {name: hashlib.sha256(content).hexdigest() for name, content in raw.items()}



def _management_runtime_observation(repository: Path, audit: dict) -> dict:
    """Keep identity-verified clients separate from possible Home owners.

    This fixed, quiet-only audit also records Python clients with no durable
    Jenkins native binding. The caller holds the startup lock and checks the
    selector, journal, host and quarantine before and after this observation.
    Java owners and bound or ambiguous Python owners remain blocking. No
    client is stopped, adopted, or issued a native receipt.
    """
    import hashlib
    import json
    from .driver import _plain
    from .startup_observations import runtime_owner_inventory

    inventory = runtime_owner_inventory(repository)
    clients = audit.get("reviewedManagementClients", [])
    if not isinstance(clients, list):
        raise JenkinsError("deployment_management_client_invalid", "Management client review is invalid")
    preserved = []
    owners = []
    execution = None
    for observed in inventory:
        matching = [item for item in clients if isinstance(item, dict)
                    and isinstance(item.get("identity"), dict)
                    and item["identity"].get("pid") == observed.get("pid")]
        if not matching and (audit.get("driverDigest") in _CONTROL_PLANE_MANAGEMENT_AUDITS
                and audit.get("scope") == "quiet-management-only"
                and audit.get("validationAcceptanceAllowed") is False
                and audit.get("executionHoldsReleaseAllowed") is False
                and (repository / ".jenkins/state/coordination.sqlite3").is_file()
                and type(observed.get("pid")) is int and observed["pid"] > 0
                and str(observed.get("creationTime", "")).isascii()
                and str(observed.get("creationTime", "")).isdigit()
                and int(observed["creationTime"]) > 0
                and Path(str(observed.get("executable", ""))).absolute()
                    == repository / ".jenkins/runtime/python/python.exe"):
            if execution is None:
                execution = _management_execution_inventory(repository, audit["runtimeOperationId"])
            bindings = [binding for binding in execution["nativeIdentityBindings"]
                        if binding["pid"] == observed["pid"]
                        and (not str(binding.get("creationTime", "")).isascii()
                             or not str(binding.get("creationTime", "")).isdigit()
                             or int(binding["creationTime"]) <= 0
                             or str(binding["creationTime"]) == str(observed["creationTime"]))]
            if not bindings:
                preserved.append({"identity": observed, "kind": "unbound-python-observation",
                    "nativeBindingInventoryDigest": execution["recordsDigest"],
                    "stopped": False, "adopted": False, "homeOwnershipClaimed": False})
                continue
        if len(matching) != 1:
            owners.append(observed)
            continue
        item = matching[0]
        expected = item["identity"]
        valid = (type(expected.get("pid")) is int and expected["pid"] > 0
                 and str(expected.get("creationTime", "")).isascii()
                 and str(expected.get("creationTime", "")).isdigit()
                 and expected.get("creationTime") == observed.get("creationTime")
                 and Path(str(expected.get("executable", ""))).absolute()
                     == Path(str(observed.get("executable", ""))).absolute())
        if not valid:
            owners.append(observed)
            continue
        evidence = item.get("evidence") or {}
        proof_path = Path(str(evidence.get("path", ""))).absolute()
        _plain(proof_path)
        try:
            proof_bytes = proof_path.read_bytes()
            if hashlib.sha256(proof_bytes).hexdigest() != evidence.get("sha256"):
                raise ValueError("client evidence changed")
            executable = Path(expected["executable"]).absolute()
            if item.get("kind") == "unrelated-workbench-client":
                if (executable != repository / ".jenkins/runtime/python/python.exe"
                        or proof_path.suffix != ".py"
                        or proof_path.is_relative_to(repository / ".jenkins")
                        or item.get("jenkinsNativeRecordReferences") != []
                        or not isinstance(item.get("commandLineSha256"), str)
                        or len(item["commandLineSha256"]) != 64):
                    raise ValueError("workbench client context")
            elif item.get("kind") == "formal-tray-client":
                receipt = json.loads(proof_bytes)
                ui = receipt.get("uiIdentity") or {}
                if (executable != repository / ".jenkins/runtime/python/pythonw.exe"
                        or proof_path.parent != repository / ".jenkins/state/migration-baseline"
                        or receipt.get("scope") != "formal-tray-rightclick-current-runtime"
                        or receipt.get("status") != "accepted"
                        or receipt.get("controllerLifecycleUnchanged") is not True
                        or any(ui.get(k) != expected.get(k)
                               for k in ("pid", "creationTime", "executable"))):
                    raise ValueError("tray client context")
                for name, source_sha in receipt.get("sourceHashes", {}).items():
                    source = repository / name
                    _plain(source)
                    if not source.is_relative_to(repository / "tools/jenkins/tray"):
                        raise ValueError("tray source context")
                    if hashlib.sha256(source.read_bytes()).hexdigest() != source_sha:
                        raise ValueError("tray source changed")
            else:
                raise ValueError("unknown client review kind")
        except (OSError, ValueError, KeyError, TypeError) as error:
            raise JenkinsError("deployment_management_client_invalid",
                               "The live client no longer matches its immutable review", retryable=True) from error
        preserved.append({"identity": observed, "kind": item["kind"],
                          "reviewDigest": digest(item), "evidence": evidence,
                          "stopped": False, "adopted": False})
    if owners:
        raise JenkinsError("deployment_home_owner_live",
                           "A repository runtime has no verified management-client identity",
                           retryable=True, details={"possibleOwners": owners})
    return {"runtimeOwnerInventory": owners, "preservedManagementClients": preserved,
            "observedRuntimeProcesses": inventory}


def control_only_management_evidence(operation: dict, host: dict, home_path: str,
                                     repository: Path, selected: dict, address: str, port: int) -> dict:
    """Reopen quiet management while all lost-generation acceptance stays closed."""
    from .lifecycle import check_port_available
    from .management_boot import observe_management_boot_stamp
    from .startup_observations import observe_departed_identity, runtime_owner_inventory, observe_home_available
    repository = Path(repository).absolute()
    audit, reference = _management_audit(repository, operation, host, home_path, selected)
    lifecycle_fence = _management_lifecycle_fence(repository, operation, host, selected)
    stamp = observe_management_boot_stamp()
    if stamp != audit.get("managementBootStamp"):
        raise JenkinsError("deployment_management_boot_changed", "The audited Windows management boot has changed",
                           retryable=True)
    controller = host.get("controllerIdentity") or host.get("identity")
    for expected in (operation, controller):
        observe_departed_identity(expected)
    runtime = _management_runtime_observation(repository, audit)
    inventory = runtime["runtimeOwnerInventory"]
    execution = _management_execution_inventory(repository, operation["operationId"])
    home = observe_home_available(Path(home_path))
    check_port_available(address, port)
    departed = [observe_departed_identity(expected) for expected in (operation, controller)]
    if observe_management_boot_stamp() != stamp:
        raise JenkinsError("deployment_management_boot_changed", "The Windows management boot changed during observation",
                           retryable=True)
    if _management_runtime_observation(repository, audit) != runtime:
        raise JenkinsError("deployment_management_runtime_changed", "Runtime client identities changed during observation", retryable=True)
    if _management_execution_inventory(repository, operation["operationId"]) != execution:
        raise JenkinsError("deployment_management_state_changed", "The execution quarantine changed during observation",
                           retryable=True)
    if _management_lifecycle_fence(repository, operation, host, selected) != lifecycle_fence:
        raise JenkinsError("deployment_management_lifecycle_changed",
                           "The serialized management lifecycle changed during observation", retryable=True)
    return {"kind": "same-boot-control-plane-management-recovery", "operationId": operation["operationId"],
            "homePath": home_path, "operationDigest": digest(operation), "hostRecordDigest": digest(host),
            "auditReceipt": reference, "containedDriverDigest": selected["driverDigest"],
            "managementBootStamp": stamp, "lifecycleFence": lifecycle_fence,
            "departedIdentities": departed, "runtimeOwnerInventory": inventory,
            "inventoryDigest": digest(inventory), "runtimeObservation": runtime, "executionQuarantine": execution, "homeObservation": home,
            "endpointAvailable": {"address": address, "port": port}, "controlPlaneOnly": True,
            "agentLaunchAttempted": False, "brokerDisabled": True, "nativeProofIssued": False,
            "validationAcceptanceAllowed": False, "executionHoldsReleaseAllowed": False,
            "outcome": "interrupted", "observedAt": time.time()}


def replacement_deployment_evidence(operation: dict, host: dict, home_path: str,
                                    repository: Path, selected: dict, address: str, port: int) -> dict:
    if (host.get("status") == "starting" and host.get("agentLaunchAttempted") is False
            and not host.get("agentIdentity") and selected.get("driverDigest") in _AUDITED_PRE_AGENT_DRIVERS):
        return abandoned_startup_evidence(operation, host, home_path, repository, selected, address, port)
    if selected.get("driverDigest") in _CONTROL_PLANE_MANAGEMENT_AUDITS:
        return control_only_management_evidence(operation, host, home_path, repository, selected, address, port)
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

"""Recover one activation transaction without changing process or execution ownership."""
from __future__ import annotations

from pathlib import Path

from ..contracts import JenkinsError, digest
from ..state.locks import process_lock
from .execution_activation import (
    _driver, _json, _sha, _write_json_atomic, protected_holds, verify_regression,
)
from .recovery import has_terminal_lifecycle_proofs
from .startup_observations import observe_departed_identity
from .lifecycle import check_port_available


def _hex(value) -> bool:
    return isinstance(value, str) and len(value) == 64 and all(c in "0123456789abcdef" for c in value)


def _waiting(reason: str) -> dict:
    return {"status": "waiting", "reasonCode": reason, "retryable": True,
            "runtimeAcceptance": False, "nativeExecutionHoldsReleaseAllowed": False}


def _ref(path: Path) -> dict:
    return {"path": str(path), "sha256": _sha(path)}


def _reference(state: Path, reference: dict, *, name: str | None = None) -> dict:
    if (not isinstance(reference, dict) or not isinstance(reference.get("path"), str)
            or not _hex(reference.get("sha256"))):
        raise JenkinsError("execution_recovery_archive", "Recovery evidence reference is incomplete")
    path = Path(reference["path"]).absolute()
    history = (state / "deployment/history").absolute()
    if path.parent != history or (name is not None and path.name != name) or _sha(path) != reference["sha256"]:
        raise JenkinsError("execution_recovery_archive", "Recovery evidence path or content changed")
    return _json(path)


def _receipt(state: Path, body: dict) -> dict:
    path = state / "deployment/history" / ("execution-recovery-" + digest(body) + ".json")
    if path.exists():
        if _json(path) != body:
            raise JenkinsError("execution_recovery_archive", "Immutable recovery receipt changed")
    else:
        _write_json_atomic(path, body)
    return _ref(path)


def _context(manager, journal: dict, transition: str) -> tuple[dict, dict]:
    if not _hex(transition) or journal.get("transitionDigest") != transition:
        raise JenkinsError("execution_recovery_identity", "Recovery digest does not match the activation journal")
    state = manager.paths.state
    evidence = _reference(state, journal.get("archiveRef"),
                          name="execution-activation-" + transition + ".json")
    if digest(evidence) != transition:
        raise JenkinsError("execution_recovery_archive", "Activation archive identity changed")
    if (evidence.get("repository") != str(manager.paths.repo.absolute())
            or evidence.get("home") != str(manager.paths.home.absolute())
            or evidence.get("specPath") != str(manager.spec.path.absolute())
            or evidence.get("specSha256") != _sha(manager.spec.path)
            or evidence.get("runtimeAcceptance") is not False
            or evidence.get("nativeExecutionHoldsReleaseAllowed") is not False):
        raise JenkinsError("execution_recovery_archive", "Activation repository, Home or specification changed")
    if (journal.get("previous") != evidence.get("previous")
            or journal.get("protectedHolds") != evidence.get("protectedHolds")):
        raise JenkinsError("execution_recovery_archive", "Activation journal differs from its archive")
    candidate_driver = _driver(manager.paths.repo, evidence["candidateDriver"])
    if candidate_driver != evidence["candidateDriver"]:
        raise JenkinsError("execution_recovery_selector", "Archived candidate driver identity changed")
    _driver(manager.paths.repo, evidence["oldDriver"])
    candidate = {**candidate_driver, "state": "selected-for-runtime-start",
                 "executionActivationDigest": transition,
                 "executionActivationReceipt": journal["archiveRef"]["path"],
                 "supportRegression": evidence["regression"], "runtimeAcceptance": False}
    if journal.get("candidate") != candidate:
        raise JenkinsError("execution_recovery_selector", "Candidate selector differs from its archived transition")
    return evidence, candidate


def _holds(state: Path, evidence: dict) -> None:
    if protected_holds(state) != evidence["protectedHolds"]:
        raise JenkinsError("execution_recovery_holds", "Protected reservations or writer holds changed", retryable=True)


def _terminal(manager, operation: dict, host: dict) -> dict:
    observation = observe_departed_identity(operation)
    if not has_terminal_lifecycle_proofs(operation, host, str(manager.paths.home.absolute()), observation):
        raise JenkinsError("execution_recovery_terminal", "Complete native and supervisor departure proofs are required",
                           retryable=True)
    check_port_available(manager.spec.controller["listenAddress"], int(manager.spec.controller["httpPort"]))
    return observation


def _snapshot(manager) -> tuple[dict, dict, dict]:
    state = manager.paths.state
    operation = manager._read()
    if not isinstance(operation, dict):
        raise JenkinsError("execution_recovery_owner", "Lifecycle operation is missing", retryable=True)
    return operation, _json(state / "deployment/host.json"), _json(state / "deployment/driver.json")


def _unchanged(manager, evidence: dict, journal: dict, operation: dict, host: dict, selector: dict) -> None:
    current, current_host, current_selector = _snapshot(manager)
    if (current != operation or current_host != host or current_selector != selector
            or _json(manager.paths.state / "deployment/execution-activation.json") != journal):
        raise JenkinsError("execution_recovery_changed", "Recovery ownership or selector changed", retryable=True)
    _holds(manager.paths.state, evidence)


def _candidate_matches(selector: dict, candidate: dict, operation: dict | None = None) -> bool:
    expected = dict(candidate)
    if operation is not None:
        expected["runtimeOperationId"] = operation.get("operationId")
    return selector == expected


def _identity_complete(value: dict, executable: Path) -> bool:
    return (isinstance(value, dict) and type(value.get("pid")) is int and value["pid"] > 0
            and str(value.get("creationTime", "")).isdigit() and int(value["creationTime"]) > 0
            and Path(value.get("executable", "")).absolute() == executable.absolute()
            and _hex(value.get("commandDigest")))


def _runtime(manager, evidence: dict, candidate: dict, operation: dict, host: dict, selector: dict) -> dict:
    from .live_observations import observe_live_identity, observe_endpoint_owner
    op = operation.get("operationId")
    repo = manager.paths.repo
    controller = host.get("controllerIdentity") or host.get("identity")
    agent = host.get("agentIdentity")
    if (not isinstance(op, str) or not op or operation.get("generation") != op
            or operation.get("pid") != operation.get("hostPid")
            or host.get("hostPid") != operation.get("hostPid")
            or host.get("operationId") != op or host.get("generation") != op
            or host.get("status") != "running"
            or host.get("controlPlaneOnly") is not False or operation.get("controlPlaneOnly") is not False
            or operation.get("homePath") != str(manager.paths.home.absolute())
            or host.get("agentLaunchAttempted") is not True
            or not _identity_complete(operation, repo / ".jenkins/runtime/python/python.exe")
            or not _identity_complete(controller, repo / ".jenkins/runtime/jdk/bin/java.exe")
            or not _identity_complete(agent, repo / ".jenkins/runtime/jdk/bin/java.exe")
            or not _candidate_matches(selector, candidate, operation)):
        raise JenkinsError("execution_recovery_owner", "Running candidate ownership is incomplete", retryable=True)
    verify_regression(repo, evidence["regression"], evidence["candidateDriver"])
    identities = [observe_live_identity(value) for value in (operation, controller, agent)]
    endpoint = observe_endpoint_owner(manager.spec.controller["listenAddress"],
                                      int(manager.spec.controller["httpPort"]), controller["pid"])
    health = manager.health()
    if not all(health.get(key) is True for key in ("controller", "plugins", "agent", "ready")):
        raise JenkinsError("execution_recovery_health", "Running candidate health is incomplete", retryable=True)
    # Re-observe after HTTP. A reused PID or moved listener cannot inherit the earlier proof.
    if identities != [observe_live_identity(value) for value in (operation, controller, agent)]:
        raise JenkinsError("execution_recovery_owner", "Lifecycle identity changed during reconciliation", retryable=True)
    if endpoint != observe_endpoint_owner(manager.spec.controller["listenAddress"],
                                         int(manager.spec.controller["httpPort"]), controller["pid"]):
        raise JenkinsError("execution_recovery_owner", "Controller listener changed during reconciliation", retryable=True)
    return {"identities": identities, "endpointOwner": endpoint, "health": health}


def _rollback_receipt(manager, journal: dict, evidence: dict) -> tuple[dict, dict]:
    ref = journal.get("recoveryReceipt") or journal.get("rollbackIntent")
    body = _reference(manager.paths.state, ref)
    if (body.get("kind") != "execution-activation-rollback"
            or body.get("transitionDigest") != journal["transitionDigest"]
            or body.get("archiveRef") != journal["archiveRef"]
            or body.get("oldSelector") != evidence["oldDriver"]
            or body.get("protectedHolds") != evidence["protectedHolds"]
            or body.get("runtimeAcceptance") is not False
            or body.get("nativeExecutionHoldsReleaseAllowed") is not False
            or Path(ref["path"]).name != "execution-recovery-" + digest(body) + ".json"):
        raise JenkinsError("execution_recovery_archive", "Rollback receipt is not bound to this transition")
    return ref, body


def _resume_rollback(manager, journal: dict, evidence: dict) -> dict:
    state = manager.paths.state
    ref, body = _rollback_receipt(manager, journal, evidence)
    operation, host, selector = _snapshot(manager)
    if operation != body.get("operation") or host != body.get("host"):
        return _waiting("execution_recovery_owner_changed")
    if selector not in (body.get("fromSelector"), evidence["oldDriver"]):
        return _waiting("execution_recovery_selector_changed")
    _terminal(manager, operation, host)
    _unchanged(manager, evidence, journal, operation, host, selector)
    if selector != evidence["oldDriver"]:
        _write_json_atomic(state / "deployment/driver.json", evidence["oldDriver"])
    # On a crash after selector replacement the durable intent admits this exact old selector.
    _unchanged(manager, evidence, journal, operation, host, evidence["oldDriver"])
    _write_json_atomic(state / "deployment/execution-activation.json",
                       {**journal, "status": "rolled_back", "recoveryReceipt": ref, "runtimeAcceptance": False})
    return {"status": "rolled_back", "controlPlaneOnly": True, "recoveryReceipt": ref,
            "runtimeAcceptance": False, "nativeExecutionHoldsReleaseAllowed": False}


def assert_recovery_start_allowed(manager) -> dict | None:
    path = manager.paths.state / "deployment/execution-activation.json"
    if not path.is_file():
        return None
    journal = _json(path)
    if journal.get("status") != "rolled_back":
        raise JenkinsError("execution_recovery_unresolved", "Activation rollback is unresolved")
    evidence, _ = _context(manager, journal, journal.get("transitionDigest"))
    _holds(manager.paths.state, evidence)
    ref, body = _rollback_receipt(manager, journal, evidence)
    operation, host, selector = _snapshot(manager)
    if operation == body.get("operation"):
        if host != body.get("host") or selector != evidence["oldDriver"]:
            raise JenkinsError("execution_recovery_unresolved", "Rolled back ownership changed")
        _terminal(manager, operation, host)
    else:
        op = operation.get("operationId")
        expected = {**evidence["oldDriver"], "runtimeOperationId": op, "state": "selected-for-runtime-start"}
        if (operation.get("executionActivationRecoveryReceipt") != ref
                or operation.get("controlPlaneOnly") is not True
                or not (host.get("controlPlaneOnly") is True or
                        (host.get("status") in {"stopped", "failed"} and host.get("agentLaunchAttempted") is False
                         and not host.get("agentIdentity")))
                or operation.get("homePath") != str(manager.paths.home.absolute())
                or not isinstance(op, str) or not op or operation.get("generation") != op
                or operation.get("pid") != operation.get("hostPid") or host.get("hostPid") != operation.get("hostPid")
                or host.get("operationId") != op or host.get("generation") != op or selector != expected):
            raise JenkinsError("execution_recovery_unresolved", "Controller-only recovery generation lost its receipt chain")
    return {"controlPlaneOnly": True, "recoveryReceipt": ref}


def recover_execution_activation(manager, transition_digest: str) -> dict:
    state = manager.paths.state
    path = state / "deployment/execution-activation.json"
    with process_lock(state / "deployment-start.lock", reason="deployment_operation_busy"):
        journal = _json(path)
        evidence, candidate = _context(manager, journal, transition_digest)
        try:
            _holds(state, evidence)
            status = journal.get("status")
            if status == "rollback_pending":
                return _resume_rollback(manager, journal, evidence)
            if status == "rolled_back":
                directive = assert_recovery_start_allowed(manager)
                return {"status": "rolled_back", **directive, "runtimeAcceptance": False,
                        "nativeExecutionHoldsReleaseAllowed": False}
            operation, host, selector = _snapshot(manager)
            if status not in {"pending", "unknown", "failed", "succeeded"}:
                return _waiting("execution_recovery_journal_unknown")
            no_spawn = operation == evidence["previous"] and host == evidence["host"]
            selected_no_spawn = selector == evidence["oldDriver"] or _candidate_matches(selector, candidate)
            selected_runtime = _candidate_matches(selector, candidate, operation)
            if status != "succeeded" and ((no_spawn and selected_no_spawn) or selected_runtime):
                try:
                    observation = _terminal(manager, operation, host)
                except (JenkinsError, KeyError, TypeError) as error:
                    if no_spawn:
                        return _waiting(getattr(error, "code", "execution_recovery_terminal"))
                else:
                    body = {"kind": "execution-activation-rollback", "transitionDigest": transition_digest,
                            "archiveRef": journal["archiveRef"], "oldSelector": evidence["oldDriver"],
                            "fromSelector": selector, "operation": operation, "host": host,
                            "hostObservation": observation, "protectedHolds": evidence["protectedHolds"],
                            "runtimeAcceptance": False, "nativeExecutionHoldsReleaseAllowed": False}
                    _unchanged(manager, evidence, journal, operation, host, selector)
                    intent = _receipt(state, body)
                    _unchanged(manager, evidence, journal, operation, host, selector)
                    journal = {**journal, "status": "rollback_pending", "rollbackIntent": intent}
                    _write_json_atomic(path, journal)
                    return _resume_rollback(manager, journal, evidence)
            if not selected_runtime:
                return _waiting("execution_recovery_selector_changed")
            observed = _runtime(manager, evidence, candidate, operation, host, selector)
            _unchanged(manager, evidence, journal, operation, host, selector)
            if status == "succeeded":
                if journal.get("operationId") != operation["operationId"]:
                    return _waiting("execution_recovery_generation_changed")
                return {"status": "resolved", "operationId": operation["operationId"],
                        "runtimeAcceptance": False, "nativeExecutionHoldsReleaseAllowed": False,
                        "reconciliationReceipt": journal.get("reconciliationReceipt")}
            body = {"kind": "execution-activation-reconciliation", "transitionDigest": transition_digest,
                    "archiveRef": journal["archiveRef"], "operation": operation, "host": host,
                    "selector": selector, "observed": observed, "protectedHolds": evidence["protectedHolds"],
                    "runtimeAcceptance": False, "nativeExecutionHoldsReleaseAllowed": False}
            receipt = _receipt(state, body)
            _unchanged(manager, evidence, journal, operation, host, selector)
            _write_json_atomic(path, {**journal, "status": "succeeded", "operationId": operation["operationId"],
                                     "reconciliationReceipt": receipt, "runtimeAcceptance": False})
            return {"status": "succeeded", "operationId": operation["operationId"],
                    "reconciliationReceipt": receipt, "runtimeAcceptance": False,
                    "nativeExecutionHoldsReleaseAllowed": False}
        except (JenkinsError, OSError, ValueError, KeyError) as error:
            return _waiting(getattr(error, "code", "execution_recovery_observation_unavailable"))

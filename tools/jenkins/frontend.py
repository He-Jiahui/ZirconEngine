"""Public Jenkins adapters for ordinary management and formal M8 requests.

Ordinary build/test capability verifies a healthy selected runtime and the
complete installed capsule. It remains independent of formal M8 acceptance.
Explicit functional requests retain their original bounded validation gate.
"""
from __future__ import annotations

import json
from pathlib import Path
from typing import Mapping

from .contracts import JenkinsError, digest, file_digest, identifier
from .submission import _parameter_fields, _submission_fields, request_payload, submit
from .state import State


def management_entry(repo_root: str | Path):
    """Load the installed client's canonical adapter; its boot verifies pins."""
    import importlib.util
    path = Path(repo_root).absolute() / "tools/jenkins-management/entry.py"
    if not path.is_file():
        raise JenkinsError("ordinary_entry_pending", "Ordinary Jenkins client is pending installation")
    spec = importlib.util.spec_from_file_location("zircon_ordinary_management_entry", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _m8_enforced(repo_root: str | Path) -> bool:
    try:
        return sole_entry_enforced(repo_root)
    except JenkinsError:
        # Ordinary capability has its own complete capsule/runtime proof.
        return False


def _ordinary_dispatch(repo_root, identity, parameters=None) -> dict:
    if not isinstance(identity, Mapping):
        raise JenkinsError("ordinary_identity_pending", "Original management identity and action input are required")
    entry = management_entry(repo_root)
    if parameters is not None and (not isinstance(parameters, Mapping)
                                   or set(parameters) - entry.RECIPE_FIELDS):
        raise JenkinsError("ordinary_recipe_unsupported", "Every caller option needs an explicit management recipe field")
    try:
        return entry.dispatch(Path(repo_root).absolute(), dict(identity),
                              recipe=dict(parameters) if parameters else None)
    except Exception as error:
        raise JenkinsError(getattr(error, "code", "ordinary_entry_pending"), str(error)) from error


_IDENTITY_FIELDS = ("repositoryId", "sessionId", "requestId")
_ENTRY_FIELDS = {
    "currentSourceRef": "CURRENT_SOURCE_REF", "sourcePreparationRef": "SOURCE_PREPARATION_REF",
    "patchRequestRef": "PATCH_REQUEST_REF", "patchOperationRef": "PATCH_OPERATION_REF",
    "attemptId": "ATTEMPT_ID", "generation": "GENERATION",
    "stageImplementationDigest": "STAGE_IMPLEMENTATION_DIGEST",
    "runtimeOperationId": "RUNTIME_OPERATION_ID", "buildRoot": "BUILD_ROOT",
}
_PROTECTED_PARAMETERS = frozenset((*_ENTRY_FIELDS.values(), "REPOSITORY_ID", "SESSION_ID", "REQUEST_ID",
                                   "SEALED_INPUT_REF", "SOURCE_INPUT_DIGEST", "COVERAGE_REF", "COVERAGE_DIGEST"))


def _first(identity: Mapping[str, object], *names: str) -> object:
    for name in names:
        value = identity.get(name)
        if value not in (None, ""):
            return value
    return None


def _read_json(path: Path) -> dict:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        raise JenkinsError("entry_gate_unavailable", "Jenkins entry gate record is unreadable") from exc
    if not isinstance(value, dict):
        raise JenkinsError("entry_gate_invalid", "Jenkins entry gate record must be an object")
    return value


def sole_entry_enforced(repo_root: str | Path) -> bool:
    """Return the persisted M8 gate; missing or malformed records fail closed."""
    root = Path(repo_root).absolute()
    inventory = _read_json(root / ".jenkins" / "state" / "entry-inventory.json")
    activation = inventory.get("activation")
    if not isinstance(activation, Mapping):
        raise JenkinsError("entry_gate_invalid", "entry inventory has no activation record")
    if not bool(activation.get("soleEntryEnforcementApplied")):
        return False
    # The deployment acceptance record is separate from the inventory so an
    # edited inventory cannot activate the gate by itself.
    acceptance = _read_json(root / ".jenkins" / "state" / "deployment" / "acceptance.json")
    return bool(acceptance.get("m8Accepted") and acceptance.get("soleEntryEnforcementApplied"))


def validate_identity(identity: Mapping[str, object] | None) -> dict[str, str]:
    """Normalize legacy and current State identity field names.

    State requests use ``sealedInputRef``/``sourceInputDigest`` while the
    original adapter used ``sourceRef``/``sourceDigest``.  Accept both at the
    boundary, then emit one canonical identity for request matching.
    """
    if not isinstance(identity, Mapping):
        raise JenkinsError("identity_missing", "Jenkins entry requires request, source and coverage identity")
    missing = [name for name in _IDENTITY_FIELDS if not identity.get(name)]
    if missing:
        raise JenkinsError("identity_missing", "Jenkins entry identity is missing: " + ",".join(missing))
    source_ref = _first(identity, "sealedInputRef", "sourceInputDigest", "sourceRef", "sourceDigest")
    source_digest = _first(identity, "sourceInputDigest", "sourceDigest", "sealedInputRef", "sourceRef")
    coverage_digest = _first(identity, "coverageDigest", "coverageRef")
    if not source_ref or not source_digest or not coverage_digest:
        raise JenkinsError("identity_missing", "Jenkins entry identity is missing source or coverage identity")
    result = {
        "repositoryId": str(identity["repositoryId"]),
        "sessionId": str(identity["sessionId"]),
        "requestId": str(identity["requestId"]),
        "sourceRef": str(source_ref), "sourceDigest": str(source_digest),
        "coverageRef": str(_first(identity, "coverageRef", "coverageDigest") or coverage_digest),
        "coverageDigest": str(coverage_digest),
    }
    for name in ("repositoryId", "sessionId", "requestId", "sourceRef", "coverageRef"):
        identifier(result[name], name)
    fields = _submission_fields(identity)
    result.update({name: fields[parameter] for name, parameter in _ENTRY_FIELDS.items() if parameter in fields})
    return result


def _verified_runtime_binding(repo_root: str | Path) -> dict[str, str]:
    """Verify the selected bytes without claiming a sealed control context."""
    from .deployment.driver import _plain, verify_driver
    root = Path(repo_root).absolute()
    selector = root / ".jenkins/state/deployment/driver.json"
    _plain(selector)
    record = _read_json(selector)
    driver = record.get("driverDigest")
    runtime = record.get("runtimeOperationId")
    if (not isinstance(driver, str) or len(driver) != 64 or any(ch not in "0123456789abcdef" for ch in driver)
            or not isinstance(runtime, str) or not runtime):
        raise JenkinsError("active_driver_unavailable", "Entry requires a selected driver and runtime operation")
    expected_root = root / ".jenkins/runtime/drivers" / driver
    expected = {"root": expected_root, "launcher": expected_root / "launcher.py",
                "steps": expected_root / "pipeline/steps.groovy"}
    if any(not record.get(name) or Path(record[name]).absolute() != path for name, path in expected.items()):
        raise JenkinsError("active_driver_unavailable", "Selected driver paths differ from their physical owner")
    verified = verify_driver(expected_root, expected_digest=driver)
    for path in (Path(verified["launcher"]), Path(verified["steps"]), root / ".jenkins/runtime/python/python.exe"):
        _plain(path)
        if not path.is_file():
            raise JenkinsError("active_driver_unavailable", "Selected driver entry or dedicated Python is missing")
    return {"stageImplementationDigest": driver, "runtimeOperationId": runtime}


def _registered_entry(state: State, identity: Mapping[str, object] | None, repo_root: str | Path) -> tuple[dict, dict]:
    supplied = validate_identity(identity)
    root = Path(repo_root).absolute()
    from .deployment.driver import _plain
    _plain(state.path)
    if state.path != root / ".jenkins/state/coordination.sqlite3":
        raise JenkinsError("identity_untrusted", "Entry must use the repository coordination State")
    request = state.get_request(supplied["repositoryId"], supplied["sessionId"], supplied["requestId"])
    if request is None:
        raise JenkinsError("identity_untrusted", "entry identity is not a matching State request")
    if request["status"] == "cancelled":
        raise JenkinsError("request_cancelled", "Cancelled entry cannot be submitted again")
    payload = request["payload"]
    if request.get("payloadDigest") != digest(payload):
        raise JenkinsError("identity_untrusted", "Registered State request payload is corrupt")
    expected = validate_identity(payload)
    for name in (*_IDENTITY_FIELDS, "sourceRef", "sourceDigest", "coverageRef", "coverageDigest"):
        if supplied[name] != expected[name]:
            raise JenkinsError("identity_untrusted", "entry identity is not a matching State request")
    runtime = _verified_runtime_binding(root)
    if (not expected.get("attemptId") or expected.get("generation") != str(request["generation"])
            or expected.get("stageImplementationDigest") != runtime["stageImplementationDigest"]):
        raise JenkinsError("identity_untrusted", "State request differs from the selected driver identity")
    preparation_ref = expected.get("sourcePreparationRef")
    if ((expected.get("currentSourceRef") and expected.get("patchRequestRef"))
            or (preparation_ref and expected.get("patchOperationRef"))):
        raise JenkinsError("source_preparation_untrusted", "Entry source preparation kinds are mutually exclusive")
    if expected.get("currentSourceRef") and not preparation_ref:
        raise JenkinsError("source_preparation_untrusted", "Current-source entry requires its completed source preparation")
    if preparation_ref:
        operation = state.get_operation(preparation_ref)
        prepared = operation.get("payload", {}) if operation else {}
        result = operation.get("result", {}) if operation else {}
        bound_identity = prepared.get("identity", {})
        aliases = {"sourceInputDigest": "sourceDigest", "sealedInputRef": "sourceRef"}
        if (not operation or operation["kind"] != "source_preparation" or operation["status"] != "complete"
                or operation["payloadDigest"] != digest(prepared) or not isinstance(bound_identity, Mapping) or not bound_identity
                or any(str(expected.get(aliases.get(key, key))) != str(value) for key, value in bound_identity.items())
                or not result or result.get("sourceDigest") != expected["sourceDigest"]
                or result.get("coverageDigest") != expected["coverageDigest"]):
            raise JenkinsError("source_preparation_untrusted", "Entry source differs from its completed State preparation")
        current_ref = expected.get("currentSourceRef")
        if current_ref:
            intent = state.get("current_source_intent", current_ref)
            if (not intent or digest(intent["payload"]) != current_ref or intent["payload"] != prepared
                    or prepared.get("kind") != "current-source" or preparation_ref != "current-source-" + current_ref
                    or Path(prepared.get("repositoryRoot", "")).absolute() != root):
                raise JenkinsError("source_preparation_untrusted", "Current-source references differ from immutable intake")
        prepared_runtime = prepared.get("runtimeOperationId")
        if (not prepared_runtime or prepared_runtime != runtime["runtimeOperationId"]
                or expected.get("runtimeOperationId", prepared_runtime) != prepared_runtime):
            raise JenkinsError("runtime_binding_mismatch", "Entry source preparation belongs to another runtime operation")
    else:
        registered_runtime = expected.get("runtimeOperationId")
        if expected.get("patchRequestRef"):
            intent = state.get("patch_intent", expected["patchRequestRef"])
            bound = intent["payload"] if intent else {}
            if (not intent or digest(bound) != expected["patchRequestRef"]
                    or any(str(expected.get(key)) != str(value) for key, value in bound.get("identity", {}).items())
                    or not bound.get("runtimeOperationId")
                    or registered_runtime and registered_runtime != bound["runtimeOperationId"]):
                raise JenkinsError("identity_untrusted", "Entry patch source differs from immutable State intake")
            registered_runtime = bound["runtimeOperationId"]
        if registered_runtime != runtime["runtimeOperationId"]:
            raise JenkinsError("runtime_binding_mismatch", "Entry request belongs to another runtime operation")
    expected.update(runtime)
    for name in _ENTRY_FIELDS:
        if name in supplied and supplied[name] != expected.get(name):
            raise JenkinsError("identity_untrusted", "Entry cannot replace immutable State or runtime identity")
    return payload, expected


def _entry_parameters(payload: Mapping[str, object], identity: dict, layers, *, job: str) -> dict[str, str]:
    authoritative = _submission_fields(identity)
    merged = _parameter_fields(payload.get("parameters", {}))
    for parameters in layers:
        if parameters is None:
            continue
        for name, value in _parameter_fields(parameters).items():
            if name.replace("_", "").casefold() in {"command", "trusted", "formalacceptance", "m8accepted",
                                                    "soleentryenforcementapplied"}:
                raise JenkinsError("entry_parameter_forbidden", "raw command and acceptance flags are not entry parameters")
            if name in _PROTECTED_PARAMETERS and authoritative.get(name) != value:
                raise JenkinsError("submission_identity_override", "Entry parameters cannot override State or runtime identity")
            merged[name] = value
    # Do not let saved State parameters reintroduce a different delivery fence.
    for name in _PROTECTED_PARAMETERS:
        if name in merged and merged[name] != authoritative.get(name):
            raise JenkinsError("submission_identity_override", "State parameters differ from immutable entry identity")
    merged.update(authoritative)
    if any(name.replace("_", "").casefold() in {"command", "trusted", "formalacceptance", "m8accepted",
                                                 "soleentryenforcementapplied"} for name in merged):
        raise JenkinsError("entry_parameter_forbidden", "raw command and acceptance flags are not entry parameters")
    if not merged.get("BUILD_ROOT"):
        raise JenkinsError("build_root_missing", "entry identity requires the State buildRoot")
    merged.setdefault("RUST_EDITION", "2021")
    if job == "zircon-execution" and not merged.get("RECIPE_REF"):
        raise JenkinsError("recipe_identity_missing", "execution entry requires recipeRef")
    if job == "zircon-flow":
        merged.pop("RECIPE_REF", None)
    return merged


def _require_functional_runtime(state: State, payload: dict, identity: dict, repo_root, manager,
                                expected_driver_digest, expected_runtime_operation_id) -> None:
    """Read-only proof for an explicitly bounded pre-M8 validation dispatch."""
    from .current_source import _authorize
    from .deployment.driver import _plain, verify_driver
    from .deployment.execution_recovery import _identity_complete
    from .deployment.live_observations import observe_endpoint_owner, observe_live_identity
    from .deployment.manager import DeploymentManager
    root = Path(repo_root).absolute()
    if (identity.get("stageImplementationDigest") != expected_driver_digest
            or identity.get("runtimeOperationId") != expected_runtime_operation_id
            or not identity.get("currentSourceRef") or not identity.get("sourcePreparationRef")):
        raise JenkinsError("functional_identity_untrusted", "Functional entry requires exact selected runtime and typed current-source references")
    intent = state.get("current_source_intent", identity["currentSourceRef"])["payload"]
    authorization = _authorize(state, intent)
    if (not intent.get("authorizationSnapshot") or intent["authorizationSnapshot"] != authorization["snapshotDigest"]
            or payload.get("authorizationSnapshot") != authorization["snapshotDigest"]):
        raise JenkinsError("functional_authorization_changed", "Functional validation must retain the original author's scoped grant")
    if (not isinstance(manager, DeploymentManager)
            or manager.paths.repo.absolute() != root or manager.paths.state.absolute() != root / ".jenkins/state"
            or manager.paths.home.absolute() != root / ".jenkins/jenkins_home" or manager.spec.repository_root != root):
        raise JenkinsError("entry_controller_mismatch", "Functional entry must use the managed repository deployment")
    paths = (root / ".jenkins/state/deployment-operation.json", root / ".jenkins/state/deployment/host.json",
             root / ".jenkins/state/deployment/driver.json")
    for path in paths:
        _plain(path)
    operation, host, selected = (_read_json(path) for path in paths)
    runtime = identity["runtimeOperationId"]
    controller = host.get("controllerIdentity") or host.get("identity")
    agent = host.get("agentIdentity")
    if (operation.get("state") != "running" or operation.get("component") != "lifecycle-host"
            or operation.get("operationId") != operation.get("generation") or operation.get("operationId") != runtime
            or selected.get("runtimeOperationId") != runtime or selected.get("driverDigest") != expected_driver_digest
            or operation.get("pid") != operation.get("hostPid") or operation.get("hostPid") != host.get("hostPid")
            or host.get("operationId") != runtime or host.get("generation") != runtime or host.get("status") != "running"
            or operation.get("homePath") != str(manager.paths.home.absolute())
            or operation.get("controlPlaneOnly") is not False or host.get("controlPlaneOnly") is not False
            or host.get("agentLaunchAttempted") is not True
            or not _identity_complete(operation, root / ".jenkins/runtime/python/python.exe")
            or not _identity_complete(controller, root / ".jenkins/runtime/jdk/bin/java.exe")
            or not _identity_complete(agent, root / ".jenkins/runtime/jdk/bin/java.exe")
            or len({operation["pid"], controller["pid"], agent["pid"]}) != 3):
        raise JenkinsError("functional_owner_untrusted", "Functional entry requires complete running native ownership")
    release_ref = selected.get("recoveryRelease", {})
    release_path = Path(release_ref.get("path", "")).absolute()
    _plain(release_path)
    if (not release_path.is_relative_to(root / ".jenkins/state/deployment")
            or not release_path.is_file() or file_digest(release_path) != release_ref.get("sha256")):
        raise JenkinsError("functional_release_untrusted", "Functional entry requires its verified deployment recovery release")
    release = _read_json(release_path)
    target = verify_driver(selected["root"], expected_digest=expected_driver_digest)
    if (release.get("schemaVersion") != 1 or release.get("kind") != "formal-owned-recovery-upgrade-release"
            or release.get("targetDriver") != target):
        raise JenkinsError("functional_release_untrusted", "Functional entry release differs from the selected immutable driver")
    identities = [observe_live_identity(value) for value in (operation, controller, agent)]
    endpoint = observe_endpoint_owner(manager.spec.controller["listenAddress"], int(manager.spec.controller["httpPort"]), controller["pid"])
    health = manager.health()
    if (not all(health.get(key) is True for key in ("controller", "agent", "plugins", "ready", "acceptingRequests"))
            or health.get("quietingDown") is not False):
        raise JenkinsError("functional_runtime_not_ready", "Functional entry requires healthy full runtime with open admission")
    if (identities != [observe_live_identity(value) for value in (operation, controller, agent)]
            or endpoint != observe_endpoint_owner(manager.spec.controller["listenAddress"], int(manager.spec.controller["httpPort"]), controller["pid"])
            or (operation, host, selected) != tuple(_read_json(path) for path in paths)
            or file_digest(release_path) != release_ref["sha256"]
            or _authorize(state, intent) != authorization):
        raise JenkinsError("functional_runtime_changed", "Functional ownership changed during readiness verification")


def submit_entry(state: State, *, identity: Mapping[str, object] | None,
                 parameters: Mapping[str, object] | None = None,
                 repo_root: str | Path, job: str = "zircon-flow",
                 base_url: str | None = None, token: str | None = None,
                 transport=None, manager=None, functional: bool = False,
                 expected_driver_digest: str | None = None,
                 expected_runtime_operation_id: str | None = None) -> dict:
    """Submit a registered entry through M8 or explicit bounded validation.

    Functional validation requires exact expected driver/runtime identities;
    raw commands and caller acceptance flags are rejected in both modes.
    """
    if type(functional) is not bool:
        raise JenkinsError("functional_scope_invalid", "Functional entry must be explicitly selected")
    if functional and (job != "zircon-flow" or not expected_driver_digest or not expected_runtime_operation_id):
        raise JenkinsError("functional_scope_invalid", "Functional entry permits only bounded zircon-flow validation with an exact runtime fence")
    if not functional and not _m8_enforced(repo_root):
        if (base_url is not None or token is not None or transport is not None or manager is not None
                or expected_driver_digest is not None or expected_runtime_operation_id is not None):
            raise JenkinsError("ordinary_runtime_override", "Ordinary entry uses its verified installed runtime and credentials")
        if job not in {"zircon-flow", "zircon-management"}:
            raise JenkinsError("ordinary_recipe_unsupported", "Ordinary entry accepts the pinned management validation job")
        return _ordinary_dispatch(repo_root, identity, parameters)
    registered, normalized = _registered_entry(state, identity, repo_root)
    params = _entry_parameters(registered, normalized, (parameters,), job=job)
    # The original State request carries source ownership and planner inputs.
    # Keep it immutable and claim only the distinct external delivery record.
    # Flow has always used the "flow" marker. Retain it so a legacy unknown
    # delivery cannot be bypassed by selecting a new marker namespace.
    params["submissionRequestId"] = ("flow" if job == "zircon-flow"
                                     else f"{normalized['requestId']}:entry:{job}")
    payload = request_payload(
        repository_id=normalized["repositoryId"], session_id=normalized["sessionId"],
        request_id=normalized["requestId"], source_ref=normalized["sourceRef"],
        source_digest=normalized["sourceDigest"], coverage_ref=normalized["coverageRef"],
        coverage_digest=normalized["coverageDigest"], parameters=params)
    payload.update({name: normalized[name] for name in _ENTRY_FIELDS if name in normalized})
    if manager is None and (functional or transport is None and token is None):
        from .deployment.spec import load_spec
        from .deployment.paths import resolve_paths
        from .deployment.manager import DeploymentManager
        spec = load_spec(Path(repo_root) / ".jenkins" / "deployment-spec.json")
        manager = DeploymentManager(spec, resolve_paths(spec))
        if base_url is not None and base_url.rstrip("/") != manager.base_url:
            raise JenkinsError("entry_controller_mismatch", "Entry controller differs from managed deployment")
        base_url = manager.base_url
    if functional:
        if token is not None or (base_url is not None and base_url.rstrip("/") != manager.base_url):
            raise JenkinsError("entry_controller_mismatch", "Functional entry uses only managed deployment credentials")
        for name in tuple(params):
            if name.replace("_", "").casefold() in {"allowcommit", "allowpush", "allownotify"}:
                if params.pop(name).casefold() != "false":
                    raise JenkinsError("functional_action_forbidden", "Functional entry permits validation only")
        params.update(ALLOW_COMMIT="false", ALLOW_PUSH="false", ALLOW_NOTIFY="false")
        payload["parameters"] = dict(params)
        payload["functionalScope"] = "validation-only"
        _require_functional_runtime(state, registered, normalized, repo_root, manager,
                                    expected_driver_digest, expected_runtime_operation_id)
    return submit(state, payload, base_url=base_url or "http://127.0.0.1:18080",
                  job=job, token=token, transport=transport, manager=manager)


def dispatch_or_local(*, repo_root: str | Path, state: State | None,
                      identity: Mapping[str, object] | None,
                      parameters: Mapping[str, object] | None = None,
                      local_result=None, **kwargs) -> dict:
    """Dispatch through verified ordinary capability or the formal M8 gate."""
    if not _m8_enforced(repo_root):
        return {"mode": "jenkins-management", "formalAcceptance": False, "m8Accepted": False,
                "result": submit_entry(state, identity=identity, parameters=parameters,
                                       repo_root=repo_root, **kwargs)}
    if state is None:
        raise JenkinsError("state_missing", "Jenkins entry requires the coordination State")
    return {"mode": "jenkins", "formalAcceptance": False,
            "result": submit_entry(state, identity=identity, parameters=parameters,
                                    repo_root=repo_root, **kwargs)}


def entry_from_environment(repo_root: str | Path, *, job: str = "zircon-flow",
                           parameters: Mapping[str, object] | None = None,
                           transport=None, manager=None, functional: bool = False,
                           expected_driver_digest: str | None = None,
                           expected_runtime_operation_id: str | None = None) -> dict | None:
    """Dispatch a known entry using the caller's immutable State identity."""
    ordinary = not functional and not _m8_enforced(repo_root)
    raw = __import__("os").environ.get("ZIRCON_JENKINS_IDENTITY_FILE", "")
    path = Path(raw)
    if not path.is_file():
        raise JenkinsError("ordinary_identity_pending", "Jenkins entry requires the original ZIRCON_JENKINS_IDENTITY_FILE")
    if ordinary:
        entry = management_entry(repo_root)
        identity_file, _raw = entry.document(path)
        return submit_entry(None, identity=identity_file, parameters=parameters,
                            repo_root=repo_root, job=job, transport=transport, manager=manager,
                            expected_driver_digest=expected_driver_digest,
                            expected_runtime_operation_id=expected_runtime_operation_id)
    identity_file = _read_json(path)
    state = State(Path(repo_root) / ".jenkins" / "state" / "coordination.sqlite3")
    payload, normalized = _registered_entry(state, identity_file, repo_root)
    merged = _entry_parameters(payload, normalized, (identity_file.get("parameters"), parameters), job=job)
    return submit_entry(state, identity=normalized, parameters=merged,
                        repo_root=repo_root, job=job, transport=transport, manager=manager, functional=functional,
                        expected_driver_digest=expected_driver_digest,
                        expected_runtime_operation_id=expected_runtime_operation_id)


def main(argv=None) -> int:
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("gate", "dispatch", "capability"))
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--job", default="zircon-flow")
    args = parser.parse_args(argv)
    if args.action == "gate":
        print("true" if sole_entry_enforced(args.repo_root) else "false")
        return 0
    try:
        if args.action == "capability":
            context = management_entry(args.repo_root).capability(Path(args.repo_root).absolute())
            result = {"status": "ready", "ordinaryEntry": context["config"]["ordinaryEntry"],
                      "driverDigest": context["config"]["driverDigest"],
                      "runtimeOperationId": context["config"]["runtimeOperationId"],
                      "formalAcceptance": False, "m8Accepted": False}
        else:
            result = entry_from_environment(args.repo_root, job=args.job)
        print(json.dumps(result, ensure_ascii=False, sort_keys=True))
        return 2 if result.get("status") == "pending" else 0
    except Exception as error:
        print(json.dumps({"status": "pending", "reasonCode": getattr(error, "code", "ordinary_entry_pending"),
                          "message": str(error), "formalAcceptance": False, "m8Accepted": False}))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())

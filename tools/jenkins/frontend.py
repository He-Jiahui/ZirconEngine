"""Single-entry adapter for known local build and test launchers.

The adapter is deliberately inert until the M8 acceptance record enables sole
entry enforcement.  Before that point callers may continue to use their local
debug path.  Once enabled, every build/test request must carry the immutable
request identity, sealed source reference and coverage reference and is sent
through the Jenkins submission adapter.
"""
from __future__ import annotations

import json
from pathlib import Path
from typing import Mapping

from .contracts import JenkinsError, identifier
from .submission import request_payload, submit
from .state import State


_IDENTITY_FIELDS = ("repositoryId", "sessionId", "requestId")


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
    return result


def submit_entry(state: State, *, identity: Mapping[str, object] | None,
                 parameters: Mapping[str, object] | None = None,
                 repo_root: str | Path, job: str = "zircon-flow",
                 base_url: str | None = None, token: str | None = None,
                 transport=None, manager=None) -> dict:
    """Submit one known entry after the M8 gate is active.

    The caller cannot provide a raw Cargo command or a trusted acceptance flag;
    those are recipe fields produced by the workflow planner.
    """
    if not sole_entry_enforced(repo_root):
        raise JenkinsError("sole_entry_not_active", "Jenkins sole-entry enforcement is not active")
    normalized = validate_identity(identity)
    params = {str(k): str(v) for k, v in (parameters or {}).items()}
    # Normalize planner-facing names before the generic submission adapter
    # serializes Jenkins parameters.
    aliases = {"sealedInputRef": "SEALED_INPUT_REF", "sourceInputDigest": "SOURCE_INPUT_DIGEST",
               "patchOperationRef": "PATCH_OPERATION_REF", "rustEdition": "RUST_EDITION",
               "driverDigest": "STAGE_IMPLEMENTATION_DIGEST"}
    for old, new in aliases.items():
        if old in params and new not in params:
            params[new] = params.pop(old)
    if "command" in params or "trusted" in params or "formalAcceptance" in params:
        raise JenkinsError("entry_parameter_forbidden", "raw command and acceptance flags are not entry parameters")
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
    if manager is None and transport is None and token is None:
        from .deployment.spec import load_spec
        from .deployment.paths import resolve_paths
        from .deployment.manager import DeploymentManager
        spec = load_spec(Path(repo_root) / ".jenkins" / "deployment-spec.json")
        manager = DeploymentManager(spec, resolve_paths(spec))
        if base_url is not None and base_url.rstrip("/") != manager.base_url:
            raise JenkinsError("entry_controller_mismatch", "Entry controller differs from managed deployment")
        base_url = manager.base_url
    return submit(state, payload, base_url=base_url or "http://127.0.0.1:18080",
                  job=job, token=token, transport=transport, manager=manager)


def dispatch_or_local(*, repo_root: str | Path, state: State | None,
                      identity: Mapping[str, object] | None,
                      parameters: Mapping[str, object] | None = None,
                      local_result=None, **kwargs) -> dict:
    """Choose local debug evidence before M8 and Jenkins after M8."""
    if not sole_entry_enforced(repo_root):
        return {"mode": "local", "formalAcceptance": False, "result": local_result}
    if state is None:
        raise JenkinsError("state_missing", "Jenkins entry requires the coordination State")
    return {"mode": "jenkins", "formalAcceptance": False,
            "result": submit_entry(state, identity=identity, parameters=parameters,
                                    repo_root=repo_root, **kwargs)}


def entry_from_environment(repo_root: str | Path, *, job: str = "zircon-flow",
                           parameters: Mapping[str, object] | None = None,
                           transport=None, manager=None) -> dict | None:
    """Dispatch a known entry using the caller's immutable State identity."""
    if not sole_entry_enforced(repo_root):
        return None
    raw = __import__("os").environ.get("ZIRCON_JENKINS_IDENTITY_FILE", "")
    path = Path(raw)
    if not path.is_file():
        raise JenkinsError("identity_missing", "Jenkins sole-entry mode requires ZIRCON_JENKINS_IDENTITY_FILE")
    identity_file = _read_json(path)
    state = State(Path(repo_root) / ".jenkins" / "state" / "coordination.sqlite3")
    normalized = validate_identity(identity_file)
    request = state.get_request(normalized["repositoryId"], normalized["sessionId"], normalized["requestId"])
    payload = request.get("payload", {}) if request else {}
    expected_source = _first(payload, "sealedInputRef", "sourceInputDigest", "sourceDigest", "sourceRef")
    expected_coverage = _first(payload, "coverageDigest", "coverageRef")
    if request is None or expected_source != normalized["sourceDigest"] or expected_coverage != normalized["coverageDigest"]:
        raise JenkinsError("identity_untrusted", "entry identity is not a matching State request")
    supplied = identity_file.get("parameters")
    if supplied is not None and not isinstance(supplied, Mapping):
        raise JenkinsError("identity_invalid", "entry parameters must be an object")
    merged = dict(payload.get("parameters", {}) if isinstance(payload.get("parameters"), Mapping) else {})
    merged.update(dict(supplied or {}))
    merged.update(dict(parameters or {}))
    # State-owned values are always authoritative for a known entry.
    merged.update({
        "sealedInputRef": str(expected_source),
        "coverageDigest": str(expected_coverage),
        "buildRoot": str(_first(payload, "buildRoot") or _first(identity_file, "buildRoot") or ""),
        "rustEdition": str(_first(identity_file, "rustEdition", "edition") or merged.get("rustEdition") or "2021"),
    })
    for name in ("patchOperationRef", "attemptId", "generation", "driverDigest"):
        value = _first(identity_file, name) or _first(payload, name) or merged.get(name)
        if value not in (None, ""):
            merged[name] = value
    if not merged.get("buildRoot"):
        raise JenkinsError("build_root_missing", "entry identity requires the State buildRoot")
    if job == "zircon-execution" and not merged.get("recipeRef"):
        raise JenkinsError("recipe_identity_missing", "execution entry requires recipeRef")
    # Flow receives planner inputs only; a caller cannot inject a raw command
    # or manufacture an execution recipe.
    if job == "zircon-flow":
        merged.pop("recipeRef", None)
        merged.pop("command", None)
    return submit_entry(state, identity=normalized, parameters=merged,
                        repo_root=repo_root, job=job, transport=transport, manager=manager)


def main(argv=None) -> int:
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("gate", "dispatch"))
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--job", default="zircon-flow")
    args = parser.parse_args(argv)
    if args.action == "gate":
        print("true" if sole_entry_enforced(args.repo_root) else "false")
        return 0
    result = entry_from_environment(args.repo_root, job=args.job)
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

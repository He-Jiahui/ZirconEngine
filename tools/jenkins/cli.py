"""Structured local control entry shared with the three Jenkins jobs."""

from __future__ import annotations

import argparse
import importlib
import json
import sys
import sqlite3
from pathlib import Path
import uuid

from .contracts import JenkinsError, canonical_json, digest, response
from .state import State
import os

DOMAIN_MODULES = {
    "candidate": "source", "flow": "workflow", "stage": "workflow",
    "patch": "patches",
    "admission": "resources", "artifact": "artifacts", "receipt": "validation",
    "accept": "validation", "integration": "gitops", "notification": "notifications",
    "deployment": "deployment", "execution": "workflow", "maintenance": "maintenance",
}

READ_ACTIONS = {"query", "get", "status", "health", "validate", "observe", "observe-execution", "observe-flow"}
RECOVERY_ACTIONS = {'cancel', 'cancel-flow', 'cancel-execution', 'reconcile-flow', 'reconcile-patch', 'reconcile-execution',
                    'reconcile-maintenance', 'compensate', 'source-compensate', 'release', 'release-claim'}


def configured_build_payload(domain: str, action: str, payload: dict, repo_root: Path) -> dict:
    """Fence new writes to the current root while keeping historical recovery identities."""
    spec_path = repo_root / '.jenkins/deployment-spec.json'
    if action in READ_ACTIONS | RECOVERY_ACTIONS or domain in {'integration', 'notification', 'deployment', 'accept', 'receipt'} or not spec_path.is_file():
        return payload
    from .deployment.paths import resolve_paths
    from .deployment.spec import load_spec
    selected = resolve_paths(load_spec(spec_path), payload.get('buildRoot')).build_root
    return {**payload, 'buildRoot': str(selected)}


def normalized_payload(payload: dict) -> dict:
    """Expose the workflow identity to authorization without accepting conflicts."""
    identity = payload.get("identity")
    if identity is None:
        return payload
    if not isinstance(identity, dict):
        raise JenkinsError("identity_invalid", "Workflow identity must be an object")
    value = dict(payload)
    for key in ("repositoryId", "sessionId", "requestId", "attemptId", "generation"):
        if key not in identity:
            continue
        if key in value and value[key] != identity[key]:
            raise JenkinsError("identity_conflict", "Control and workflow identities differ")
        value[key] = identity[key]
    return value


def authorization_action(domain: str, action: str, payload: dict) -> str:
    if domain == "patch" and action in {"register", "submit", "prepare", "prepare-patch", "reconcile", "reconcile-patch", "cancel"}:
        return "implementation"
    if domain == "deployment" or (domain == "admission" and action == "activate-policy"):
        return "deployment"
    if domain == "notification":
        return "notify"
    if domain in {"maintenance", "flow"} and action in {"gc", "gc-maintenance", "cache-maintenance"}:
        return "gc"
    if domain == "artifact" and action in {"gc", "delete", "collect"}:
        return "gc"
    if domain == "candidate" and action in {"claim", "apply", "patch", "compensate", "release", "release-claim"}:
        return "implementation"
    if domain == "integration" and action in {"commit", "commit-flow", "integrate", "publish", "revert", "forward_revert", "forward-revert-flow", "repair", "commit_flow", "reconcile", "reconcile-flow", "reconcile_flow", "compensate", "forward_revert_flow"}:
        return "commit"
    if domain == "integration" and action == "push":
        return "push"
    if domain == "stage" and payload.get("stage") in {"git_commit", "git-commit", "commit"}:
        return "commit"
    if domain == "flow" and action in {"source-claim", "source-apply", "source-patch", "source-compensate", "source-release", "prepare-patch", "reconcile-patch"}:
        return "implementation"
    return "validation"


def runtime_path(repo_root: Path, value: Path, *, area: str | None = None) -> Path:
    """Keep protocol files in their declared physical runtime domain."""
    value = value if value.is_absolute() else repo_root / value
    if ".." in value.parts:
        raise JenkinsError("runtime_path_rejected", "Runtime paths cannot contain parent traversal")
    value = value.absolute()
    approved = repo_root.absolute() / ".jenkins"
    if area:
        approved /= area
    if not value.is_relative_to(approved):
        raise JenkinsError("runtime_path_rejected", "Protocol files must stay in their declared runtime directory")
    for part in (value, *value.parents):
        if part.exists() or part.is_symlink():
            if part.is_symlink() or getattr(part.lstat(), "st_file_attributes", 0) & 0x400:
                raise JenkinsError("runtime_path_rejected", "Protocol file paths must have a physical identity")
    return value


def read_payload(path: Path | None) -> dict:
    if path is None:
        return {}
    if path.stat().st_size > 8 * 1024 * 1024:
        raise JenkinsError("input_too_large", "Control input exceeds the configured size bound")
    value = json.loads(path.read_text(encoding="utf-8-sig"))
    if not isinstance(value, dict):
        raise JenkinsError("input_not_object", "Control input must be a JSON object")
    return value


def request_operation(action: str, payload: dict, state: State, repo_root: Path) -> dict:
    repo = payload.get("repositoryId", digest(str(repo_root).casefold()))
    session = payload.get("sessionId")
    request = payload.get("requestId")
    if action == "submit":
        auth = state.check_authorization(repo, session, "validation", payload.get("ownedPaths", []))
        actions = payload.get("requestedActions", [])
        for requested in actions:
            state.check_authorization(repo, session, requested, payload.get("ownedPaths", []))
        value = state.submit_request({**payload, "repositoryId": repo,
                                      "authorizationSnapshot": auth["snapshotDigest"]})
        return response(value["status"], request_id=request, observed_generation=value["generation"], request=value)
    value = state.get_request(repo, session, request)
    if value is None:
        raise JenkinsError("request_not_found", "Request does not exist")
    if action == "query":
        return response(value["status"], request_id=request, observed_generation=value["generation"], request=value)
    if action == "cancel":
        state.check_authorization(repo, session, "validation", value["payload"].get("ownedPaths", []))
        with state.transaction() as connection:
            connection.execute("UPDATE requests SET status='cancel_requested' WHERE repository_id=? AND session_id=? AND request_id=?",
                               (repo, session, request))
            state.put("cancellation", f"{repo}:{session}:{request}",
                      {"repositoryId": repo, "sessionId": session, "requestId": request,
                       "generation": value["generation"]}, connection=connection)
            state.event("cancel_requested", {"repositoryId": repo, "sessionId": session, "requestId": request},
                        connection=connection)
        return response("cancel_requested", request_id=request, observed_generation=value["generation"])
    raise JenkinsError("operation_unknown", "Unknown request action")


def dispatch(domain: str, action: str, payload: dict, state: State, repo_root: Path) -> dict:
    payload = normalized_payload(payload)
    if domain == "request":
        if action == 'submit':
            payload = configured_build_payload(domain, action, payload, repo_root)
        return request_operation(action, payload, state, repo_root)
    if action not in READ_ACTIONS:
        operation = authorization_action(domain, action, payload)
        state.check_authorization(payload.get("repositoryId", digest(str(repo_root).casefold())),
                                  payload.get("sessionId"), operation, payload.get("ownedPaths", []))
        # Patch preparation mutates source state and also needs a validation
        # scope for the resulting sealed input.  Keep both checks at the
        # protocol boundary so a raw patch cannot bypass validation authority.
        if (domain, action) in {("flow", "prepare-patch"), ("flow", "reconcile-patch"),
                                ("patch", "prepare"), ("patch", "prepare-patch"),
                                ("patch", "reconcile"), ("patch", "reconcile-patch")}:
            state.check_authorization(payload.get("repositoryId", digest(str(repo_root).casefold())),
                                      payload.get("sessionId"), "validation", payload.get("ownedPaths", []))
        payload = configured_build_payload(domain, action, payload, repo_root)
    module_name = DOMAIN_MODULES.get(domain)
    if module_name is None:
        raise JenkinsError("operation_unknown", "Unknown operation family")
    try:
        module = importlib.import_module(f"tools.jenkins.{module_name}")
    except ImportError as error:
        raise JenkinsError("module_unavailable", "Required support module has not been installed") from error
    handler = getattr(module, "handle", None)
    if handler is None:
        raise JenkinsError("operation_unavailable", "Support operation is not yet registered")
    value = handler(action, payload, state, repo_root, domain=domain)
    if not isinstance(value, dict):
        raise JenkinsError("invalid_owner_response", "Support owner returned an invalid control response")
    return {**response(value.get("status", "complete"), request_id=payload.get("requestId"),
                       operation_id=payload.get("operationId")), **value}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Repository-owned Jenkins control protocol")
    parser.add_argument("--repository", type=Path, default=Path(os.environ.get("ZIRCON_REPO_ROOT", Path(__file__).resolve().parents[2])))
    parser.add_argument("--state", type=Path)
    parser.add_argument("domain", choices=["request", *DOMAIN_MODULES])
    parser.add_argument("action")
    parser.add_argument("--input-file", type=Path)
    parser.add_argument("--output-file", type=Path)
    args = parser.parse_args(argv)
    output = None
    try:
        repo = args.repository.absolute()
        if not repo.is_dir():
            raise JenkinsError("repository_missing", "The declared repository root does not exist")
        if args.output_file:
            output = runtime_path(repo, args.output_file, area="state/responses")
        state_path = runtime_path(repo, args.state or repo / ".jenkins/state/coordination.sqlite3", area="state")
        state = State(state_path)
        result = dispatch(args.domain, args.action, read_payload(args.input_file), state, repo)
        exit_code = 0
    except JenkinsError as error:
        result = response("blocked" if error.retryable else "failed", reason_code=error.code,
                          retryable=error.retryable, message=str(error))
        exit_code = 2
    except (OSError, ValueError, TypeError, KeyError, sqlite3.Error):
        result = response("failed", reason_code="invalid_control_input",
                          message="Control input or required file could not be read")
        exit_code = 2
    encoded = canonical_json(result) + b"\n"
    if output is not None:
        try:
            output.parent.mkdir(parents=True, exist_ok=True)
            temporary = output.with_name(output.name + ".pending-" + uuid.uuid4().hex)
            temporary.write_bytes(encoded)
            temporary.replace(output)
        except (JenkinsError, OSError) as error:
            result = response("failed", reason_code=getattr(error, "code", "response_write_failed"),
                              message="Control response could not be safely persisted")
            encoded = canonical_json(result) + b"\n"
            exit_code = 2
    sys.stdout.buffer.write(encoded)
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())

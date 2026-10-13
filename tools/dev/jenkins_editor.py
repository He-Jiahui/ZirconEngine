"""Submit existing editor source postimages to the selected Jenkins coordinator.

This entry writes request metadata only. The coordinator owns source sealing,
materialization, compilation and product publication.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import re
import sqlite3
import uuid
from contextlib import closing
from datetime import datetime, timezone
from pathlib import Path

from tools.dev.jenkins_validate import (
    EntryError, HEX64, build_root, canonical_json, digest, plain_path, protocol,
    read_json, readonly_flow, relative_path, repository_id, require_identifier,
    save_json, selected_driver,
)

_BASE_HEAD = re.compile(r"(?:[0-9a-f]{40}|[0-9a-f]{64})\Z")
_TEST_ID = re.compile(r"[A-Za-z_][A-Za-z0-9_:]{0,511}\Z")
_UNKNOWN = frozenset({"delivery_unknown", "protocol_delivery_unknown"})


def _timeouts(args):
    registration = args.registration_timeout if args.registration_timeout is not None else args.timeout
    for value, maximum, name in ((args.timeout, 300, "Timeout"), (registration, 900, "Registration timeout")):
        if isinstance(value, bool) or not isinstance(value, (float, int)) or not math.isfinite(value) or not 0 < value <= maximum:
            raise EntryError("timeout_invalid", f"{name} must be greater than zero and at most {maximum} seconds")
    return registration


def _file_reference(value):
    path = Path(value)
    if not path.is_absolute():
        raise EntryError("registration_evidence_invalid", "Evidence must use an absolute physical file path")
    plain_path(path)
    if not path.is_file() or path.stat().st_size > 8 * 1024 * 1024:
        raise EntryError("registration_evidence_invalid", "Evidence must be a bounded ordinary metadata file")
    return {"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


def _protocol_implementation():
    return _file_reference(Path(protocol.__code__.co_filename).absolute())["sha256"]


def _terminal_registration(args, folder, prior, intent):
    if not args.registration_terminal_receipt:
        raise EntryError("registration_terminal_required", "Recovery requires evidence that the original bounded registration call is terminal")
    reference = _file_reference(args.registration_terminal_receipt)
    receipt = read_json(reference["path"], "registration terminal receipt")
    expected = {"identity": prior["identity"], "runtimeOperationId": prior["runtimeOperationId"],
                "buildRoot": prior["buildRoot"], "intentDigest": digest(intent),
                "registrationCallId": prior.get("registrationCallId", "original"),
                "protocolInputSha256": _file_reference(folder / "current-source-register.json")["sha256"],
                "protocolImplementationSha256": _protocol_implementation()}
    if (not isinstance(receipt, dict) or type(receipt.get("schemaVersion")) is not int or receipt["schemaVersion"] != 1
            or receipt.get("kind") != "jenkins-editor-registration-terminal"
            or any(receipt.get(key) != value for key, value in expected.items())
            or read_json(folder / "current-source-register.json", "original registration input") != intent):
        raise EntryError("registration_evidence_mismatch", "Terminal evidence must bind this exact immutable intent and registration call")
    handle, registration = receipt.get("operatorHandle"), receipt.get("registration")
    if (not isinstance(handle, dict) or handle.get("kind") != "unified-exec"
            or type(handle.get("id")) is not int or handle["id"] <= 0
            or handle.get("status") != "terminal" or type(handle.get("exitCode")) is not int or handle["exitCode"] != 0
            or not isinstance(registration, dict) or registration.get("domain") != "current-source"
            or registration.get("action") != "register" or registration.get("status") != "terminal"
            or registration.get("cause") != "timeout" or registration.get("processReaped") is not True
            or type(registration.get("timeoutSeconds")) not in (int, float)
            or not math.isfinite(registration["timeoutSeconds"]) or not 0 < registration["timeoutSeconds"] <= 900
            or prior.get("registrationTimeoutSeconds", registration["timeoutSeconds"]) != registration["timeoutSeconds"]):
        raise EntryError("registration_not_terminal", "Only the completed bounded control call and its reaped child permit recovery", retryable=True)
    evidence = receipt.get("evidence")
    if not isinstance(evidence, dict):
        raise EntryError("registration_evidence_invalid", "Original dispatch result and intent references are required")
    artifacts = {}
    for name in ("dispatchResult", "dispatchIntent"):
        item = evidence.get(name)
        if not isinstance(item, dict) or set(item) != {"path", "sha256"} or _file_reference(item["path"]) != item:
            raise EntryError("registration_evidence_mismatch", "Original dispatch evidence bytes differ")
        artifacts[name] = item
    result = read_json(artifacts["dispatchResult"]["path"], "original dispatch result")
    outcome = result.get("operatorResult", {}) if isinstance(result, dict) else {}
    if (not isinstance(outcome, dict) or type(result.get("exitCode")) is not int or result["exitCode"] != 2
            or result.get("dispatchIntentSha256") != artifacts["dispatchIntent"]["sha256"]
            or outcome.get("reasonCode") != "protocol_delivery_unknown" or outcome.get("retryable") is not True
            or outcome.get("formalAcceptance") is not False):
        raise EntryError("registration_evidence_mismatch", "Evidence must reference the original unknown registration control result")
    return reference


def _recovery_event(folder, recovery_id, phase, **facts):
    path = folder / "registration-recovery" / f"{recovery_id}-{phase}.json"
    save_json(path, {"schemaVersion": 1, "kind": "jenkins-editor-registration-recovery",
                     "recordedUtc": datetime.now(timezone.utc).isoformat(), "phase": phase, **facts}, immutable=True)
    return _file_reference(path)


def _registered_reference(root, ref, prior, intent):
    """Read the authoritative immutable intent and dispatch marker without State writes."""
    database = root / ".jenkins/state/coordination.sqlite3"
    plain_path(database)
    if not database.is_file():
        raise EntryError("registration_reconciliation_unknown", "Registered input cannot be reconciled with authoritative State", retryable=True)
    try:
        with closing(sqlite3.connect(database.absolute().as_uri() + "?mode=ro", uri=True, timeout=3)) as connection:
            connection.execute("PRAGMA query_only=ON")
            connection.execute("BEGIN")
            row = connection.execute("SELECT payload FROM records WHERE domain=? AND key=?", ("current_source_intent", ref)).fetchone()
            recorded = json.loads(row[0]) if row else None
            slot = digest([prior["identity"][name] for name in ("repositoryId", "sessionId", "requestId")])
            source_request = connection.execute("SELECT payload FROM records WHERE domain=? AND key=?", ("current_source_request", slot)).fetchone()
            expected = {**intent, "schemaVersion": 1, "kind": "current-source", "repositoryRoot": str(root),
                        "runtimeOperationId": prior["runtimeOperationId"],
                        "authorizationSnapshot": recorded.get("authorizationSnapshot") if isinstance(recorded, dict) else None}
            if (not isinstance(recorded, dict) or not HEX64.fullmatch(str(recorded.get("authorizationSnapshot", "")))
                    or digest(recorded) != ref or recorded != expected or source_request is None
                    or json.loads(source_request[0]) != {"currentSourceRef": ref}):
                raise EntryError("registration_reference_mismatch", "Returned registration reference differs from the saved immutable intent")
            key = ":".join(prior["identity"][name] for name in ("repositoryId", "sessionId", "requestId")) + ":flow"
            return connection.execute("SELECT 1 FROM records WHERE domain=? AND key=?", ("jenkins_submission", key)).fetchone() is not None
    except (sqlite3.Error, ValueError) as error:
        raise EntryError("registration_reconciliation_unknown", "Authoritative registration state cannot be read", retryable=True) from error


def _paths(root, values, name):
    paths = [relative_path(root, value) for value in values]
    if len(paths) != len(set(paths)):
        raise EntryError("source_scope_invalid", f"{name} contains duplicate paths")
    return sorted(paths)


def _path_input(args, root, name):
    repeated = getattr(args, name + "_path")
    filename = getattr(args, name + "_paths")
    if filename:
        if repeated:
            raise EntryError("source_scope_invalid", f"Use either --{name}-path or --{name}-paths")
        values = read_json(filename, name + " paths")
        if not isinstance(values, list) or any(not isinstance(value, str) for value in values):
            raise EntryError("source_scope_invalid", f"--{name}-paths requires a JSON list of paths")
    else:
        values = repeated
    return _paths(root, values, name + " paths")


def _external_inputs(args, root):
    if not args.external_inputs:
        return []
    inputs = read_json(args.external_inputs, "external inputs")
    if not isinstance(inputs, list) or len(inputs) > 1:
        raise EntryError("external_inputs_invalid", "Provide at most one explicit zr_vm dependency")
    result = []
    for item in inputs:
        if (not isinstance(item, dict) or
                set(item) != {"name", "repositoryRoot", "baseHead", "currentHashes", "untrackedPaths"} or
                item["name"] != "zr_vm"):
            raise EntryError("external_inputs_invalid", "External inputs require the exact zr_vm dependency fields")
        external = Path(item["repositoryRoot"]).absolute()
        if external != root.parent / "zr_vm" or external.resolve() != external or not external.is_dir():
            raise EntryError("external_inputs_invalid", "zr_vm must be the physical canonical repository sibling")
        plain_path(external)
        base = item["baseHead"]
        if not isinstance(base, str) or not _BASE_HEAD.fullmatch(base):
            raise EntryError("external_inputs_invalid", "External inputs require an immutable Git base")
        values = item["currentHashes"]
        if not isinstance(values, dict) or not values:
            raise EntryError("external_inputs_invalid", "External hashes must be a nonempty object")
        hashes = {}
        for path, checksum in values.items():
            name = relative_path(external, path)
            if (name in hashes or name.split("/", 1)[0] in {".git", ".codex", ".jenkins", ".opencode", "target"} or
                    checksum is not None and (not isinstance(checksum, str) or not HEX64.fullmatch(checksum))):
                raise EntryError("external_inputs_invalid", "External hashes require unique source paths and SHA-256 or null")
            hashes[name] = checksum
        untracked = item["untrackedPaths"]
        if not isinstance(untracked, list) or any(not isinstance(path, str) for path in untracked):
            raise EntryError("external_inputs_invalid", "External untracked paths must be a JSON list")
        untracked = _paths(external, untracked, "external untracked paths")
        if not set(untracked) <= {path for path, checksum in hashes.items() if checksum is not None}:
            raise EntryError("external_inputs_invalid", "External untracked paths must be selected present inputs")
        result.append({"name": "zr_vm", "repositoryRoot": str(external), "baseHead": base,
                       "currentHashes": dict(sorted(hashes.items())), "untrackedPaths": untracked})
    return result


def intent_payload(args, root, identity):
    if not args.current_hashes:
        raise EntryError("current_hashes_missing", "--current-hashes is required")
    original = read_json(args.current_hashes, "current hashes")
    if not isinstance(original, dict) or not original:
        raise EntryError("current_hashes_invalid", "Current hashes must be a nonempty object")
    hashes = {}
    for path, checksum in original.items():
        name = relative_path(root, path)
        if name in hashes or (checksum is not None and
                              (not isinstance(checksum, str) or not HEX64.fullmatch(checksum))):
            raise EntryError("current_hashes_invalid", "Use unique paths with SHA-256 or null")
        hashes[name] = checksum
    owned = _path_input(args, root, "owned")
    untracked = _path_input(args, root, "untracked")
    if not owned or not set(owned) <= set(hashes):
        raise EntryError("source_scope_invalid", "Owned paths must be an explicit nonempty subset of current hashes")
    if not set(untracked) <= {p for p, value in hashes.items() if value is not None}:
        raise EntryError("source_scope_invalid", "Untracked paths must name explicitly selected present files")
    if not args.base_head or not _BASE_HEAD.fullmatch(args.base_head):
        raise EntryError("source_base_invalid", "--base-head requires an immutable Git revision")
    tests = args.required_test
    if (not tests or len(tests) != len(set(tests)) or
            any(not isinstance(test, str) or not _TEST_ID.fullmatch(test) for test in tests)):
        raise EntryError("product_test_scope_invalid", "Provide unique exact Cargo test IDs")
    return {
        "identity": identity, "baseHead": args.base_head,
        "buildRoot": build_root(root, args.build_root),
        "ownedPaths": owned, "currentHashes": dict(sorted(hashes.items())),
        "untrackedPaths": untracked,
        "externalInputs": _external_inputs(args, root),
        "coverage": {"template": "editor_product_" + (args.profile or "development"),
                     "requiredTests": sorted(tests), "noDefaultFeatures": True},
    }


def _query(root, driver, folder, saved, args, call):
    ref = args.current_source_ref or saved.get("currentSourceRef")
    if (args.current_source_ref and saved.get("currentSourceRef") and
            args.current_source_ref != saved["currentSourceRef"]):
        raise EntryError("request_identity_conflict", "Query must retain the saved source reference")
    control = {"identity": saved["identity"], "buildRoot": saved["buildRoot"],
               "runtimeOperationId": saved["runtimeOperationId"]}
    if ref:
        control["currentSourceRef"] = require_identifier(ref, "currentSourceRef")
    source = call(root, driver, folder, "current-source", "query", control, args.timeout)
    preparation = source.get("preparation") or {}
    prepared = (preparation.get("payload") or {}).get("result") or {}
    flow = (readonly_flow(root, saved["identity"], prepared["identity"], saved["buildRoot"])
            if prepared.get("identity") else None)
    return {"status": flow["status"] if flow else source.get("status", "pending"),
            "formalAcceptance": bool(flow and flow.get("formalAcceptance") is True),
            **control, "source": source, "flow": flow}


def _execute_locked(args, root, driver, identity, folder, *, call):
    lifecycle = folder / "lifecycle.json"
    prior = read_json(lifecycle, "request lifecycle") if lifecycle.is_file() else None
    if prior:
        if not isinstance(prior, dict) or prior.get("identity") != identity:
            raise EntryError("request_identity_conflict", "Saved request, attempt or driver differs")
        if prior.get("runtimeOperationId") != driver["runtimeOperationId"]:
            raise EntryError("runtime_binding_changed", "Observe this request through its original runtime")
        if args.build_root and prior.get("buildRoot") != build_root(root, args.build_root):
            raise EntryError("source_build_root_mismatch", "Retain the saved physical build root")
        if args.action == "resume-registration":
            return _resume_registration(args, root, driver, folder, prior, call=call)
        if args.current_hashes:
            save_json(folder / "intent.json", intent_payload(args, root, identity), immutable=True)
        # Replays observe the same request. They never perform a second admission.
        return _query(root, driver, folder, prior, args, call)
    if args.action == "resume-registration":
        raise EntryError("registration_recovery_missing", "Recovery requires the existing original lifecycle and intent")
    if args.action == "query":
        saved = {"identity": identity, "buildRoot": build_root(root, args.build_root),
                 "runtimeOperationId": driver["runtimeOperationId"]}
        query_path = folder / "current-source-query.json"
        plain_path(query_path)
        if query_path.exists():
            control = dict(saved)
            if args.current_source_ref:
                control["currentSourceRef"] = require_identifier(args.current_source_ref, "currentSourceRef")
            recorded = read_json(query_path, "saved current-source query")
            try:
                same_binding = (isinstance(recorded, dict)
                                and canonical_json(recorded) == canonical_json(control))
            except (TypeError, ValueError):
                same_binding = False
            if not same_binding:
                raise EntryError("query_binding_conflict", "Existing query does not match the requested binding")
        return _query(root, driver, folder, saved, args, call)
    payload = intent_payload(args, root, identity)
    save_json(folder / "intent.json", payload, immutable=True)
    saved = {"identity": identity, "buildRoot": payload["buildRoot"],
             "runtimeOperationId": driver["runtimeOperationId"], "phase": "registering",
             "registrationCallId": uuid.uuid4().hex, "registrationTimeoutSeconds": _timeouts(args)}
    save_json(lifecycle, saved)
    return _register_and_submit(args, root, driver, folder, saved, payload, call=call)


def _register_and_submit(args, root, driver, folder, saved, payload, *, call):
    lifecycle = folder / "lifecycle.json"
    try:
        registered = call(root, driver, folder, "current-source", "register", payload, _timeouts(args))
        if not isinstance(registered, dict) or registered.get("status") != "registered" or registered.get("retryable") is True:
            raise EntryError("protocol_delivery_unknown", "Registration response is not authoritative", retryable=True)
        ref = registered.get("currentSourceRef")
        if not isinstance(ref, str) or not HEX64.fullmatch(ref):
            raise EntryError("protocol_delivery_unknown", "Registration returned an invalid source digest", retryable=True)
    except EntryError as error:
        unknown = error.code in _UNKNOWN
        save_json(lifecycle, {**saved, "phase": "registration_unknown" if unknown else "rejected",
                              "reasonCode": error.code})
        raise
    return _submit_registered(args, root, driver, folder, saved, payload, ref, call=call)


def _submit_registered(args, root, driver, folder, saved, payload, ref, *, call):
    lifecycle = folder / "lifecycle.json"
    control = {"identity": saved["identity"], "currentSourceRef": ref,
               "buildRoot": payload["buildRoot"], "runtimeOperationId": saved["runtimeOperationId"],
               "ownedPaths": payload["ownedPaths"]}
    saved = {**saved, **control, "phase": "dispatching"}
    save_json(lifecycle, saved)
    try:
        submitted = call(root, driver, folder, "current-source", "submit", control, args.timeout)
    except EntryError as error:
        unknown = error.code in _UNKNOWN
        save_json(lifecycle, {**saved, "phase": "delivery_unknown" if unknown else "rejected",
                              "reasonCode": error.code})
        if not unknown:
            raise
        return {"status": "delivery_unknown", "formalAcceptance": False, **control,
                "reasonCode": error.code,
                "nextAction": "query with the original session, request, attempt, driver and build root"}
    save_json(lifecycle, {**saved, "phase": "submitted"})
    return {"status": "submitted", "formalAcceptance": False, **control, "submission": submitted,
            "nextAction": "query with the original session, request, attempt, driver and build root"}


def _resume_registration(args, root, driver, folder, prior, *, call):
    if (args.current_hashes or args.base_head or args.profile or args.required_test or args.owned_path
            or args.untracked_path or args.owned_paths or args.untracked_paths or args.external_inputs or args.current_source_ref):
        raise EntryError("registration_recovery_input_conflict", "Recovery uses only the saved immutable intent; source, coverage and reference replacements are forbidden")
    if prior.get("phase") != "registration_unknown" or prior.get("reasonCode") not in _UNKNOWN:
        raise EntryError("registration_recovery_phase", "Only a reconciled unknown registration can be resumed")
    plain_path(folder / "intent.json")
    intent = read_json(folder / "intent.json", "original immutable intent")
    if not isinstance(intent, dict) or intent.get("identity") != prior["identity"] or intent.get("buildRoot") != prior["buildRoot"]:
        raise EntryError("registration_recovery_input_conflict", "Saved intent and lifecycle identity differ")
    terminal = _terminal_registration(args, folder, prior, intent)
    recovery_id = uuid.uuid4().hex
    evidence = _recovery_event(folder, recovery_id, "terminal", previousLifecycle=prior,
                               terminalReceipt=terminal, intentDigest=digest(intent))
    try:
        observed = _query(root, driver, folder, prior, args, call)
    except EntryError as error:
        _recovery_event(folder, recovery_id, "query-unknown", reasonCode=error.code, terminalEvidence=evidence)
        raise EntryError("registration_reconciliation_unknown", "Original registration query is unresolved; recovery cannot register", retryable=True) from error
    query = _recovery_event(folder, recovery_id, "query", observation=observed, terminalEvidence=evidence)
    source = observed.get("source")
    if (not isinstance(source, dict) or source.get("retryable") is not False
            or source.get("status") not in {"not_found", "registered", "prepared"}):
        raise EntryError("registration_reconciliation_unknown", "Original registration remains unknown or retryable", retryable=True)
    if source["status"] == "not_found":
        if any(source.get(key) is not None for key in ("currentSourceRef", "preparation", "request")) or source.get("reasonCode"):
            raise EntryError("registration_reconciliation_unknown", "Not-found observation contains conflicting registration evidence", retryable=True)
        saved = {**prior, "phase": "registering", "registrationCallId": uuid.uuid4().hex,
                 "registrationTimeoutSeconds": _timeouts(args), "registrationRecovery": query}
        saved.pop("reasonCode", None)
        save_json(folder / "lifecycle.json", saved)
        return _register_and_submit(args, root, driver, folder, saved, intent, call=call)
    ref = source.get("currentSourceRef")
    if not isinstance(ref, str) or not HEX64.fullmatch(ref) or prior.get("currentSourceRef", ref) != ref:
        raise EntryError("registration_reference_mismatch", "Late registration must provide its exact original source digest")
    dispatched = _registered_reference(root, ref, prior, intent)
    saved = {**prior, "phase": "registered_observed", "currentSourceRef": ref, "registrationRecovery": query}
    save_json(folder / "lifecycle.json", saved)
    if dispatched or source.get("preparation") is not None or source.get("request") is not None:
        return {**observed, "currentSourceRef": ref, "registrationRecovery": query}
    return _submit_registered(args, root, driver, folder, saved, intent, ref, call=call)


def execute(args, *, call=protocol, driver_loader=selected_driver):
    _timeouts(args)
    root = Path(args.repository).absolute()
    plain_path(root)
    if not root.is_dir():
        raise EntryError("repository_missing", "Repository root does not exist")
    driver = driver_loader(root)
    identity = {"repositoryId": repository_id(root),
                "sessionId": require_identifier(args.session_id, "sessionId"),
                "requestId": require_identifier(args.request_id, "requestId"),
                "attemptId": require_identifier(args.attempt_id or args.request_id + "-attempt-1", "attemptId"),
                "generation": 1, "stageImplementationDigest": driver["digest"]}
    if args.action == "dry-check":
        payload = intent_payload(args, root, identity)
        return {"status": "dry-check", "formalAcceptance": False, "identity": identity,
                "buildRoot": payload["buildRoot"], "intentDigest": digest(payload),
                "selectedFileCount": len(payload["currentHashes"]),
                "ownedPaths": payload["ownedPaths"], "untrackedPaths": payload["untrackedPaths"],
                "coverage": payload["coverage"], "protocol": ["current-source/register", "current-source/submit"]}
    folder = root / ".jenkins/state/requests/operator-editor" / digest(
        [identity[k] for k in ("repositoryId", "sessionId", "requestId")])
    plain_path(folder)
    from tools.jenkins.state.locks import process_lock
    folder.mkdir(parents=True, exist_ok=True)
    with process_lock(folder / "operator.lock", reason="operator_request_busy"):
        return _execute_locked(args, root, driver, identity, folder, call=call)


def parser():
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("action", nargs="?", choices=("dry-check", "submit", "query", "resume-registration"), default="dry-check")
    result.add_argument("--repository", "--repo-root", default=str(Path(__file__).resolve().parents[2]))
    result.add_argument("--session-id", required=True)
    result.add_argument("--request-id", required=True)
    result.add_argument("--attempt-id")
    result.add_argument("--current-hashes")
    result.add_argument("--owned-path", action="append", default=[])
    result.add_argument("--untracked-path", action="append", default=[])
    result.add_argument("--owned-paths", help="JSON list of task-owned paths")
    result.add_argument("--untracked-paths", help="JSON list of explicitly selected untracked paths")
    result.add_argument("--external-inputs", help="JSON metadata for an explicit canonical sibling zr_vm dependency")
    result.add_argument("--base-head")
    result.add_argument("--profile", choices=("development", "shipping"))
    result.add_argument("--required-test", action="append", default=[])
    result.add_argument("--build-root")
    result.add_argument("--current-source-ref")
    result.add_argument("--timeout", type=float, default=60)
    result.add_argument("--registration-timeout", type=float, help="Separate bounded registration budget, at most 900 seconds; defaults to --timeout")
    result.add_argument("--registration-terminal-receipt", help="Absolute root-owned evidence for the terminal original registration call; required for resume-registration")
    return result


def main(argv=None):
    args = parser().parse_args(argv)
    try:
        value = execute(args)
        code = 3 if value["status"] == "delivery_unknown" else 0
    except EntryError as error:
        value = {"status": "blocked" if error.retryable else "failed", "formalAcceptance": False,
                 "reasonCode": error.code, "message": str(error), "retryable": error.retryable}
        code = 2
    except (OSError, ValueError, KeyError, TypeError, UnicodeError):
        value = {"status": "failed", "formalAcceptance": False, "reasonCode": "invalid_input",
                 "message": "Required input or selected driver cannot be read"}
        code = 2
    print(canonical_json(value).decode("utf-8"))
    return code


if __name__ == "__main__":
    raise SystemExit(main())

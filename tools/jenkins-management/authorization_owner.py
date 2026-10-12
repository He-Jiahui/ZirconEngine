"""Jenkins-owned issuance of preapproved validation identities and scope succession.

The operator seals actual user-task evidence and exact scopes in the capsule.
API authentication alone never supplies repository action authorization.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import stat
import sys

ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,127}\Z")
SHA = re.compile(r"[0-9a-f]{64}\Z")


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"),
                      ensure_ascii=False, allow_nan=False).encode("utf-8")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def physical(value):
    path = Path(os.path.abspath(value))
    for part in (path, *path.parents):
        try:
            info = part.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise ValueError("authorization_path_alias")
    return path


def bounded_path(repo, value, area):
    path = physical(value)
    root = physical(repo / area)
    if path == root or not path.is_relative_to(root):
        raise ValueError("authorization_path_outside_owned_area")
    return path


def read_json(path):
    value = json.loads(physical(path).read_bytes())
    if not isinstance(value, dict):
        raise ValueError("authorization_record_not_object")
    return value


def write_json(path, value):
    path = physical(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + ".pending")
    data = encoded(value) + b"\n"
    if temporary.exists():
        if temporary.read_bytes() != data:
            raise ValueError("authorization_response_pending_requires_review")
    else:
        with temporary.open("xb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
    temporary.replace(path)
    return {"path": str(path), "sha256": sha(path.read_bytes())}


def boot(capsule):
    capsule = physical(capsule)
    raw = physical(capsule / "capsule.json").read_bytes()
    config = json.loads(raw)
    if (capsule.name != sha(raw) or config.get("schemaVersion") != 1
            or config.get("job") != "zircon-authorization"
            or sha(physical(capsule / "authorization_owner.py").read_bytes())
            != config.get("actorSha256")):
        raise ValueError("authorization_capsule_identity_mismatch")
    repo = physical(config["repositoryRoot"])
    if capsule.parent != physical(repo / ".jenkins/runtime/authorization"):
        raise ValueError("authorization_capsule_path_mismatch")
    issued = bounded_path(repo, config["userGrant"]["path"], ".jenkins/state/authorizations")
    grant_raw = issued.read_bytes()
    if sha(grant_raw) != config["userGrant"]["sha256"]:
        raise ValueError("authorization_user_task_evidence_changed")
    user = json.loads(grant_raw)
    if (user.get("source") != "user" or not user.get("task")
            or user.get("rootThreadId") != config.get("rootThreadId")
            or sha(user.get("objective", "").encode("utf-8")) != user.get("objectiveSha256")
            or user.get("approvedGrants") != config.get("grants")):
        raise ValueError("authorization_user_task_scope_mismatch")
    installed = read_json(repo / ".jenkins/state/management/installed.json")
    if (installed.get("driverDigest") != config.get("driverDigest")
            or installed.get("runtimeOperationId") != config.get("runtimeOperationId")
            or not SHA.fullmatch(installed.get("capsuleSha256", ""))):
        raise ValueError("authorization_runtime_binding_changed")
    management = physical(repo / ".jenkins/runtime/management" / installed["capsuleSha256"])
    if physical(installed["capsule"]) != management:
        raise ValueError("authorization_management_capsule_path_mismatch")
    management_raw = physical(management / "capsule.json").read_bytes()
    metadata = json.loads(management_raw)
    if (sha(management_raw) != installed["capsuleSha256"]
            or sha(physical(management / "runner.py").read_bytes()) != metadata["files"]["runner.py"]):
        raise ValueError("authorization_management_actor_integrity_mismatch")
    spec = importlib.util.spec_from_file_location("jenkins_authorization_management", management / "runner.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    management_config, actual_repo, _ = module.boot(management)
    if actual_repo != repo or management_config["driverDigest"] != config["driverDigest"]:
        raise ValueError("authorization_owner_runtime_not_current")
    from tools.jenkins.state import State
    from tools.jenkins.source import repository_id
    if any(grant.get("repositoryId") != repository_id(repo) for grant in config["grants"].values()):
        raise ValueError("authorization_repository_scope_mismatch")
    return config, user, repo, State(repo / ".jenkins/state/coordination.sqlite3")


def scope_plan(grant, state, repo):
    from tools.jenkins.state import _relative_path
    if (grant.get("actions") != ["validation"] or not grant.get("task")
            or not ID.fullmatch(grant.get("sessionId", ""))
            or not grant.get("principal")
            or not SHA.fullmatch(grant.get("handoff", {}).get("sha256", ""))):
        raise ValueError("authorization_catalog_scope_invalid")
    paths = [_relative_path(path) for path in grant["paths"]]
    if not paths or len(paths) != len(set(paths)):
        raise ValueError("authorization_catalog_paths_invalid")
    predecessor = grant.get("predecessor")
    if predecessor:
        old = state.check_authorization(grant["repositoryId"], predecessor["sessionId"], "validation")
        if (old["snapshotDigest"] != predecessor["snapshotDigest"]
                or old["principal"] != grant["principal"]
                or sorted(old["paths"]) != sorted(predecessor["paths"])
                or sorted(paths) != sorted(set(old["paths"]) | set(grant["addedPaths"]))):
            raise ValueError("authorization_predecessor_snapshot_mismatch")
    for name, expected in grant.get("currentPins", {}).items():
        path = physical(repo / name)
        if not path.is_relative_to(repo) or sha(path.read_bytes()) != expected:
            raise ValueError("authorization_owned_source_pin_changed")
    return paths


def consume_handoff(repo, grant, result, state, key):
    if not grant.get("consumeHandoffAfterAuthorization") or result.get("handoffConsumed"):
        return result
    path = bounded_path(repo, grant["handoff"]["path"], ".jenkins/records")
    if not path.exists():
        observed = {**result, "handoffCleanup": "missing-requires-reconciliation"}
        state.put("authorization_owner_result", key, observed)
        return observed
    data = path.read_bytes()
    if sha(data) != grant["handoff"]["sha256"]:
        return {**result, "handoffCleanup": "changed-preserved"}
    response_path = bounded_path(repo, grant["responsePath"], ".jenkins/records")
    if response_path.read_bytes() != encoded(result) + b"\n":
        raise ValueError("authorization_completion_response_missing")
    # The immutable input text, State result and exact response are durable before deletion.
    if state.get("authorization_owner_result", key)["payload"].get("snapshotDigest") != result["snapshotDigest"]:
        raise ValueError("authorization_completion_receipt_missing")
    path.unlink()
    result = {**result, "handoffConsumed": True, "handoffCleanup": "completed",
              "removedBytes": len(data)}
    state.put("authorization_owner_result", key, result)
    state.event("authorization_handoff_consumed", {"operationId": key, "path": str(path),
                "sha256": grant["handoff"]["sha256"], "removedBytes": len(data)})
    return result


def execute(config, user, repo, state, task_key, *, check_only=False):
    grant = config["grants"].get(task_key)
    if not isinstance(grant, dict):
        raise ValueError("authorization_task_not_preapproved")
    key = grant["operationId"]
    if not ID.fullmatch(key):
        raise ValueError("authorization_operation_id_invalid")
    if grant.get("sourceReady") is False:
        path = bounded_path(repo, grant["handoff"]["path"], ".jenkins/records")
        raw = path.read_bytes()
        if sha(raw) != grant["handoff"]["sha256"]:
            raise ValueError("authorization_handoff_changed")
        result = {"status": "pending", "reasonCode": "source_freeze_pending",
                  "operationId": key, "task": grant["task"], "handoffRef": grant["handoff"],
                  "authorizationWritten": False, "requestSubmitted": False,
                  "driverDigest": config["driverDigest"], "runtimeOperationId": config["runtimeOperationId"],
                  "nextCanonicalAdmission": grant.get("nextCanonicalAdmission"),
                  "formalAcceptance": False, "m8Accepted": False, "handoffConsumed": False,
                  "removedBytes": 0}
        if not check_only:
            state.put("record_intake_receipt", key, result)
            write_json(bounded_path(repo, grant["responsePath"], ".jenkins/records"), result)
        return result
    prior = state.get("authorization_owner_result", key)
    if prior:
        result = prior["payload"]
        if (result.get("catalogDigest") != sha(encoded(grant))
                or result.get("userGrant") != config["userGrant"]):
            raise ValueError("authorization_operation_input_conflict")
        actual = state.check_authorization(grant["repositoryId"], grant["sessionId"], "validation", grant["paths"])
        if actual["snapshotDigest"] != result["snapshotDigest"]:
            raise ValueError("authorization_completed_snapshot_changed")
        if check_only:
            return result
        response_path = bounded_path(repo, grant["responsePath"], ".jenkins/records")
        write_json(response_path, result)
        result = consume_handoff(repo, grant, result, state, key)
        write_json(response_path, result)
        return result
    paths = scope_plan(grant, state, repo)
    handoff_path = bounded_path(repo, grant["handoff"]["path"], ".jenkins/records")
    handoff_bytes = handoff_path.read_bytes()
    if sha(handoff_bytes) != grant["handoff"]["sha256"]:
        raise ValueError("authorization_handoff_changed")
    if check_only:
        return {"status": "ready", "operationId": key, "sessionId": grant["sessionId"],
                "paths": paths, "actions": ["validation"], "authorizationWritten": False}
    evidence = {"source": "user", "task": grant["task"], "rootThreadId": config["rootThreadId"],
                "issuedFromCurrentUserGoal": True, "objectiveSha256": user["objectiveSha256"],
                "userGrantRef": config["userGrant"], "handoffRef": grant["handoff"],
                "predecessor": grant.get("predecessor"), "operationId": key}
    authorization = state.authorize_session(grant["repositoryId"], grant["sessionId"],
                                            grant["principal"], paths, ["validation"], evidence)
    result = {"status": "completed", "operationId": key, "repositoryId": grant["repositoryId"],
              "sessionId": grant["sessionId"], "principal": grant["principal"],
              "snapshotDigest": authorization["snapshotDigest"], "authorizationStatus": authorization["status"],
              "paths": paths, "actions": ["validation"], "predecessor": grant.get("predecessor"),
              "userGrant": config["userGrant"], "catalogDigest": sha(encoded(grant)),
              "handoffRef": grant["handoff"], "handoffText": handoff_bytes.decode("utf-8"),
              "handoffConsumed": False, "removedBytes": 0, "driverDigest": config["driverDigest"],
              "runtimeOperationId": config["runtimeOperationId"], "formalAcceptance": False,
              "m8Accepted": False, "rejectedRequestsPreserved": grant.get("rejectedRequests", []),
              "nextCanonicalAdmission": grant.get("nextCanonicalAdmission")}
    state.put("authorization_owner_result", key, result, expected_version=0)
    state.event("validation_identity_issued", {"operationId": key, "sessionId": grant["sessionId"],
                "snapshotDigest": result["snapshotDigest"], "predecessor": grant.get("predecessor")})
    response_path = bounded_path(repo, grant["responsePath"], ".jenkins/records")
    write_json(response_path, result)
    result = consume_handoff(repo, grant, result, state, key)
    write_json(response_path, result)
    return result


def main(argv=None):
    parser = argparse.ArgumentParser()
    parser.add_argument("--capsule", type=Path, required=True)
    parser.add_argument("--task-key", required=True)
    parser.add_argument("--output-file", type=Path, required=True)
    parser.add_argument("--check-only", action="store_true")
    args = parser.parse_args(argv)
    config, user, repo, state = boot(args.capsule)
    output = bounded_path(repo, args.output_file, ".jenkins/state/controls")
    result = execute(config, user, repo, state, args.task_key, check_only=args.check_only)
    write_json(output, result)
    print(json.dumps({k: v for k, v in result.items() if k != "handoffText"}, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

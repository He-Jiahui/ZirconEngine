"""Bounded host-governance dispatcher using the existing Jenkins State."""
from __future__ import annotations

import base64
import json
import os
from pathlib import Path
import re
import shutil
import sys
import time

from .authority import Governance
from .paths import build_namespace, digest, encoded, file_fact, physical


ACTIONS = {"scan", "register", "register-inventory", "configure-repository", "open-round", "claim", "inspect-scope", "record-response", "close-round", "adopt",
           "seal", "prepare-integration", "validate-integration", "finish-integration", "recover-integration", "apply-patch",
           "cleanup-plan", "cleanup-apply", "cleanup-reconcile", "permission-plan", "permission-apply",
           "permission-probe", "cancel-scan", "test", "reconcile", "audit"}
ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,127}\Z")


def _json(path: Path, value: dict):
    physical(path, exists=False)
    temporary = path.with_name(path.name + ".pending")
    if temporary.exists():
        raise ValueError("original_pending_report_requires_reconciliation")
    with temporary.open("xb") as stream:
        stream.write(encoded(value))
        stream.flush()
        os.fsync(stream.fileno())
    os.replace(temporary, path)


def native(authority: Governance, command: list[str], cwd: Path, output: Path, *, timeout: float, environment=None) -> dict:
    """This function is reachable only from the hash-pinned Jenkins dispatcher."""
    output = build_namespace(output)
    output.mkdir(parents=True, exist_ok=False)
    from tools.dev.local_cargo import _run_owned_command
    env = {**os.environ, "PYTHONDONTWRITEBYTECODE": "1", "GIT_TERMINAL_PROMPT": "0",
           "GIT_NO_LAZY_FETCH": "1", "TEMP": str(output), "TMP": str(output), **(environment or {})}
    result = _run_owned_command(command, physical(cwd), env, output, timeout)
    _json(output / "native-receipt.json", result)
    authority.put("native", digest(str(output)), {**result, "receipt": file_fact(output / "native-receipt.json"),
                                                "operationId": authority.runtime["operationId"], **authority.runtime}, expected_version=0)
    return result


def register_artifacts(authority, items):
    """Register paths before a Jenkins recipe creates outputs outside its receipt directory."""
    root = Path("E:/cargo-targets/zircon-local/reports") / authority.runtime["taskId"]
    ledger = physical(root / "artifacts.json")
    with authority.state.transaction() as connection:
        value = json.loads(ledger.read_text(encoding="utf-8"))
        for item in items:
            physical(item["path"], exists=False)
            value["artifacts"].append({**item, "owner": authority.runtime,
                "dependencies": {"requestId": authority.runtime["requestId"], "attemptId": authority.runtime["attemptId"],
                                 "driverDigest": authority.runtime["driverDigest"]}})
        _json(ledger, value)
        authority.state.put("governance_artifact_manifest", authority.runtime["taskId"],
                            {"path": str(ledger), "sha256": file_fact(ledger)["sha256"]}, connection=connection)


MANAGED_DOMAINS = ("managed_storage", "managed_storage_registration", "governance_repository",
    "generated_output", "artifact_object", "artifact_ref", "resource_reservation", "pool_writer_hold",
    "management_worktree", "managed_worktree_reader", "managed_worktree_writer", "path_claims",
    "patch_owner", "execution", "native_job", "management_task_scratch", "governance_claim")
MANAGED_FIELDS = ("path", "root", "namespace", "worktree", "worktreeRoot", "buildNamespace", "buildRoot",
    "sourceRoot", "sourceCheckout", "canonicalCheckout", "commonDir", "outputRoot", "targetRoot",
    "targetDir", "executionRoot", "preparationRoot", "objectRoot", "scratchRoot")


def _managed_binding(authority):
    paths = [authority.root / ".jenkins/state/deployment/driver.json",
             authority.root / ".jenkins/state/deployment-operation.json"]
    facts = [file_fact(path) for path in paths]
    selected, runtime = [json.loads(path.read_text(encoding="utf-8")) for path in paths]
    if (selected.get("driverDigest") != authority.runtime["driverDigest"]
            or not selected.get("runtimeOperationId")
            or runtime.get("operationId") != selected["runtimeOperationId"]
            or runtime.get("state") != "running"):
        raise ValueError("managed_inventory_runtime_changed")
    if facts != [file_fact(path) for path in paths]:
        raise ValueError("managed_inventory_runtime_changed")
    return {"controlRoot": str(authority.root), "databasePath": str(physical(authority.state.path)),
            "driverDigest": selected["driverDigest"], "capsuleDigest": authority.runtime["capsuleDigest"],
            "runtimeOperationId": selected["runtimeOperationId"], "runtimeRefs": facts}


def _managed_index(authority):
    """One RO snapshot; project only whitelisted ownership fields, never journals."""
    import sqlite3
    binding = _managed_binding(authority)
    database = physical(authority.state.path)
    before = database.stat()
    result = {"schemaVersion": 1, "inventoryPolicy": "managed-roots-v1", "binding": binding,
              "mode": "ro", "queryOnly": True, "records": [], "omissions": [], "status": "complete"}
    connection = sqlite3.connect(database.as_uri() + "?mode=ro", uri=True, isolation_level=None, timeout=3)
    used, deadline = 0, time.monotonic() + 15
    try:
        connection.execute("PRAGMA query_only=ON")
        if connection.execute("PRAGMA query_only").fetchone()[0] != 1:
            raise ValueError("managed_inventory_requires_query_only")
        connection.execute("BEGIN")
        if connection.execute("SELECT value FROM metadata WHERE key='schemaVersion'").fetchone() != ("1",):
            raise ValueError("managed_inventory_state_schema")
        connection.set_progress_handler(lambda: int(time.monotonic() >= deadline), 1000)
        query = ("SELECT domain,key,version,updated_at,length(CAST(payload AS BLOB)),"
                 "CASE WHEN length(CAST(payload AS BLOB))<=65536 THEN payload END FROM records "
                 "WHERE domain IN (" + ",".join("?" for _ in MANAGED_DOMAINS) + ") ORDER BY domain,key LIMIT 100001")
        for domain, key, version, updated, size, body in connection.execute(query, MANAGED_DOMAINS):
            if len(result["records"]) >= 100000 or used + (len(body.encode("utf-8")) if body else 128) > 8 * 1024 * 1024:
                result["omissions"].append({"reason": "managed-index-budget"})
                result["status"] = "partial"
                break
            used += len(body.encode("utf-8")) if body else 128
            item = {"domain": domain, "key": key, "version": version, "updatedAt": updated,
                    "payloadBytes": size, "paths": [], "protection": "protected-unknown"}
            if body is None:
                item["reason"] = "oversized-record-protected-without-loading"
            else:
                value = json.loads(body)
                item["recordDigest"] = digest(value)
                state = value.get("status")
                current = value.get("driverDigest") == binding["driverDigest"]
                if current and state in {"active", "reserved", "held", "running", "preparing", "updating",
                                          "materializing", "pending", "waiting", "started", "claimed", "ready"}:
                    item["protection"] = "protected-active"
                elif (state in {"released", "complete", "terminal", "deleted", "failed", "interrupted"}
                      or value.get("driverDigest") and not current):
                    item["protection"] = "protected-history"
                if domain == "artifact_ref" and state == "active":
                    item["protection"] = "protected-active-reference"
                for field in ("owner", "ownerSessionId", "sessionId", "operationId", "digest"):
                    if isinstance(value.get(field), (str, int, float, bool)):
                        item[field] = value[field]
                paths = [(field, value[field]) for field in MANAGED_FIELDS if isinstance(value.get(field), str)]
                for field in ("writeScopes", "outputRoots"):
                    paths += [(field, path) for path in value.get(field, []) if isinstance(path, str)] if isinstance(value.get(field), list) else []
                for field, raw in paths:
                    path = Path(raw)
                    if not path.is_absolute() or ".." in path.parts:
                        continue
                    source = field in {"sourceRoot", "sourceCheckout", "canonicalCheckout", "commonDir", "worktree", "worktreeRoot"}
                    namespace = ((domain in {"managed_storage", "managed_storage_registration", "governance_repository"}
                                  and field in {"root", "path", "namespace", "buildRoot", "buildNamespace"})
                                 or (domain in {"execution", "resource_reservation", "management_task_scratch"}
                                     and field == "buildRoot" and path in {Path(drive + ":/cargo-targets") for drive in "DEF"}))
                    item["paths"].append({"path": str(path), "field": field,
                        "role": "source" if source else "namespace" if namespace else "retained",
                        "protection": item["protection"]})
            result["records"].append(item)
    except sqlite3.OperationalError as error:
        if "interrupt" not in str(error).lower():
            raise
        result["status"] = "partial"
        result["omissions"].append({"reason": "managed-index-time-budget"})
    finally:
        connection.close()
    after = database.stat()
    if (before.st_dev, before.st_ino) != (after.st_dev, after.st_ino) or binding != _managed_binding(authority):
        raise ValueError("managed_inventory_authority_changed")
    result["indexDigest"] = digest(result)
    return result


def _scan(authority, payload, report):
    from .inventory import inventory_roots
    policy = payload.get("inventoryPolicy", "whole-volume-v1")
    roots = inventory_roots(policy)
    output = report / "inventory.json"
    command = [sys.executable, "-B", "-X", "utf8", "-m", "tools.jenkins.governance.scan_child",
               "--output", str(output), "--roots-base64", base64.b64encode(encoded(roots)).decode(),
               "--registrations-base64", base64.b64encode(encoded({
                   row["payload"]["canonicalCheckout"].casefold(): row["payload"]
                   for row in authority.state.list("governance_repository")})).decode()]
    if policy == "managed-roots-v1":
        if payload.get("resumeOperationId"):
            raise ValueError("managed_inventory_resume_not_supported")
        index = _managed_index(authority)
        _json(report / "managed-index.json", index)
        index_ref = file_fact(report / "managed-index.json")
        command += ["--inventory-policy", policy, "--managed-index", index_ref["path"],
                    "--managed-index-sha256", index_ref["sha256"]]
    if payload.get("resumeOperationId"):
        from .resumption import prepare
        specification = prepare(authority, payload, report.parent)
        _json(report / "resume-source.json", specification)
        command += ["--resume-base64", base64.b64encode(encoded(specification)).decode()]
    evidence = native(authority, command, authority.root, report / "native-scan", timeout=600 if policy == "managed-roots-v1" else 21600)
    if not _terminal(evidence) or evidence.get("rootExitCode") != 0:
        raise ValueError("inventory_native_execution_incomplete")
    observation = json.loads(output.read_text(encoding="utf-8"))
    reference = file_fact(output)
    result = {"status": observation["status"], "inventoryRef": reference,
              "inventoryDigest": observation["inventoryDigest"],
              "coverage": {key: observation.get(key) for key in ("roots", "rootCoverage", "directories", "files", "logicalBytes", "durationSeconds")},
              "repositoryCount": len(observation["repositories"]), "errors": len(observation["errors"]),
              "links": len(observation["links"]), "sourceCandidates": len(observation["sourceCandidates"]),
              "outputCandidates": len(observation["outputCandidates"]), "acceptance": "inventory-only"}
    if policy == "managed-roots-v1":
        if (observation.get("inventoryPolicy") != policy or observation.get("roots") != [str(Path(root)) for root in roots]
                or observation.get("stateIndex") != index
                or digest({k: v for k, v in observation.items() if k != "inventoryDigest"}) != observation["inventoryDigest"]):
            raise ValueError("managed_inventory_child_binding_changed")
        result.update(inventoryPolicy=policy, inventoryId=payload["operationId"], binding=index["binding"],
                      managedIndexRef=index_ref, nativeReceipt=file_fact(report / "native-scan/native-receipt.json"),
                      nativeRecordKey=digest(str(report / "native-scan")), acceptance="bounded-inventory-only")
    authority.put("inventory", payload["operationId"], result, expected_version=0)
    return result


def _terminal(value):
    # Match the retained Windows Job receipt, rather than treating Jenkins SUCCESS as native acceptance.
    tree = value.get("processTree", {})
    accounting = value.get("nativeJobEvidence", {})
    return (tree.get("terminal") is True and tree.get("nativeWaiterJoined") is True
            and tree.get("pipeEOF") is True and not value.get("terminalErrors")
            and not accounting.get("errors")
            and accounting.get("records", {}).get("afterClose", {}).get("activeProcesses") == 0)


def _dispatch(authority, request, report):
    action = request["action"]
    payload = {**request.get("payload", {}), "operationId": request["operationId"]}
    if action == "scan":
        return _scan(authority, payload, report)
    if action == "cancel-scan":
        from .cancellation import cancel_scan
        return cancel_scan(authority, payload, report)
    if action == "inspect-scope":
        from .scopes import preview
        return preview(authority, payload, report)
    if action in {"register-inventory", "configure-repository"}:
        from .registration import run, configure
        return (run if action == "register-inventory" else configure)(authority, payload, report)
    handlers = {"register": authority.register, "open-round": authority.open_round, "claim": authority.claim,
                "record-response": authority.record_response, "close-round": authority.close_round,
                "adopt": authority.adopt_legacy_claim}
    if action in handlers:
        return handlers[action](payload)
    if action in {"seal", "prepare-integration", "finish-integration", "recover-integration", "apply-patch"}:
        from . import versioning
        if action == "seal":
            scratch = report / "private-index"
            scratch.mkdir()
            return versioning.seal(authority, payload, scratch)
        if action == "prepare-integration":
            return versioning.prepare_integration(authority, payload)
        if action == "finish-integration":
            return versioning.finish_integration(authority, payload)
        if action == "recover-integration":
            return versioning.recover_integration(authority, payload)
        return versioning.apply_patch(authority, payload, report)
    if action in {"cleanup-plan", "cleanup-apply", "cleanup-reconcile"}:
        from . import cleanup
        return {"cleanup-plan": cleanup.plan, "cleanup-apply": cleanup.apply,
                "cleanup-reconcile": cleanup.reconcile}[action](authority, payload, report)
    if action in {"validate-integration", "test"}:
        from . import validation
        return validation.run(authority, payload, report, self_test=action == "test")
    if action in {"permission-plan", "permission-apply", "permission-probe"}:
        from . import permissions
        return {"permission-plan": permissions.plan, "permission-apply": permissions.apply,
                "permission-probe": permissions.probe}[action](authority, payload, report)
    if action == "audit":
        from .audit import run
        return run(authority, payload, report)
    raise ValueError("unimplemented_governance_action")


def execute(pins: dict, request: dict, *, capsule_digest: str) -> int:
    from tools.jenkins.deployment.driver import verify_driver
    from tools.jenkins.processes.identity import current_identity
    from tools.jenkins.state import State
    root = physical(pins["controlRoot"])
    selected = json.loads((root / ".jenkins/state/deployment/driver.json").read_text(encoding="utf-8"))
    if selected["driverDigest"] != pins["driverDigest"]:
        raise ValueError("selected_driver_changed")
    verify_driver(physical(selected["root"]), expected_digest=pins["driverDigest"])
    for field in ("taskId", "sessionId", "requestId", "operationId", "attemptId"):
        if not isinstance(request.get(field), str) or not ID.fullmatch(request[field]):
            raise ValueError("invalid_governance_identity:" + field)
    if request["taskId"] != pins["taskId"] or request["action"] not in ACTIONS:
        raise ValueError("governance_task_or_action_not_authorized")
    report_root = build_namespace(pins["reportRoot"])
    if report_root != Path("E:/cargo-targets/zircon-local/reports") / request["taskId"]:
        raise ValueError("governance_report_root_changed")
    state = State(root / ".jenkins/state/coordination.sqlite3")
    identity = {"taskId": request["taskId"], "sessionId": request["sessionId"], "requestId": request["requestId"],
        "operationId": request["operationId"], "attemptId": request["attemptId"], "driverDigest": pins["driverDigest"],
        "capsuleDigest": capsule_digest, "jenkinsBuildUrl": os.environ.get("BUILD_URL"),
        "processIdentity": current_identity(os.getpid()).to_dict(), "acceptedAt": time.time()}
    authority = Governance(state, root, identity)
    if request["action"] == "reconcile":
        result = state.get_operation(request["operationId"])
        print(json.dumps({"status": "observed", "originalOperation": result}, ensure_ascii=True))
        return 0
    gated = {"adopt", "seal", "prepare-integration", "validate-integration", "finish-integration", "recover-integration", "apply-patch",
             "cleanup-plan", "cleanup-apply", "permission-apply", "configure-repository"}
    if request["action"] in gated:
        capability = state.get("governance_capability", capsule_digest)
        if not capability or capability["payload"].get("status") != "passed":
            raise ValueError("exact_governance_capsule_functional_tests_required")
    intent = {"request": request, "capsuleDigest": capsule_digest, "driverDigest": pins["driverDigest"]}
    with state.transaction() as connection:
        old = state.get_operation(request["operationId"], connection=connection)
        if old:
            if old["payloadDigest"] != digest(intent):
                raise ValueError("original_operation_input_changed")
            print(json.dumps({"status": "original-operation", "originalOperation": old}, ensure_ascii=True))
            return 0
        state.put_operation("host_governance", request["operationId"], intent, connection=connection)
        state.transition_operation(request["operationId"], "prepared", "running", identity, connection=connection)
    report_root.mkdir(parents=True, exist_ok=True)
    report = report_root / request["operationId"]
    manifest = {"schemaVersion": 1, "taskId": request["taskId"], "owner": identity,
        "artifacts": [{"path": str(report), "role": "governance-evidence-and-owned-temporaries",
                       "dependencies": intent, "retainUntil": "claim, integration and cleanup requests are reconciled"}],
        "logicalDeletedBytes": 0, "actualFreedBytes": 0, "status": "open"}
    # Record intended output before creating this operation's output directory.
    _json(report_root / (request["operationId"] + ".artifacts.json"), manifest)
    with state.transaction() as connection:
        ledger = report_root / "artifacts.json"
        current = json.loads(ledger.read_text(encoding="utf-8")) if ledger.exists() else {"schemaVersion": 1, "taskId": request["taskId"], "artifacts": []}
        current["artifacts"].extend(manifest["artifacts"])
        current.update(status="open", logicalDeletedBytes=0, actualFreedBytes=0)
        _json(ledger, current)
        state.put("governance_artifact_manifest", request["taskId"], {"path": str(ledger), "sha256": file_fact(ledger)["sha256"]}, connection=connection)
    report.mkdir(exist_ok=False)
    _json(report / "request.json", intent)
    try:
        from .audit import admission
        _json(report / "admission.json", admission(authority, request))
        result = _dispatch(authority, request, report)
        _json(report / "closeout-roots.json", admission(authority, request))
        if result.get("status") == "failed":
            phase, code = "blocked", 1
        else:
            phase = "complete" if result.get("status") not in {"needs-review", "conflict", "validation-required", "awaiting-user-claims"} else "waiting"
            code = 0
    except Exception as error:
        result = {"status": "needs-review", "reason": type(error).__name__ + ":" + str(error)[:400],
                  "sourcePreserved": True, **identity}
        phase, code = "blocked", 1
    receipt = {"identity": identity, "action": request["action"], "result": result,
               "finishedAt": time.time(), "sourcePins": pins["sourcePins"], "formalAcceptance": False}
    _json(report / "receipt.json", receipt)
    state.transition_operation(request["operationId"], "running", phase,
                               {"receipt": file_fact(report / "receipt.json"), "result": result})
    workspace = build_namespace(os.environ["WORKSPACE"])
    archive = workspace / "governance" / request["operationId"]
    archive.mkdir(parents=True, exist_ok=False)
    # Archive all evidence, including native process-tree receipts and failed logs.
    # Never archive temporary source/index fixtures or credential files.
    for directory, children, files in os.walk(report, followlinks=False):
        children[:] = [name for name in children if name not in {"fixtures", "private-index"}
                       and not (Path(directory) / name).is_symlink()]
        for name in files:
            source = Path(directory) / name
            if source.suffix not in {".json", ".md", ".bin"}:
                continue
            physical(source)
            destination = archive / source.relative_to(report)
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)
    shutil.copy2(report_root / (request["operationId"] + ".artifacts.json"), archive / "artifacts.json")
    print(json.dumps({"status": phase, "receipt": str(report / "receipt.json"), "result": result}, ensure_ascii=True))
    return code

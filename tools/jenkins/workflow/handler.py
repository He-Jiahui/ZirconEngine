"""CLI handlers for flow/stage/execution domains.

Handlers persist only small orchestration records.  Source, resource, process,
artifact and validation authorities remain in their own support modules.
"""
from __future__ import annotations

from pathlib import Path
import subprocess
from typing import Any

from ..contracts import JenkinsError, digest, response
from ..state import State
from ..validation import TrustedReceiptAuthority
from ..source import handle as source_handle, get_sealed_consumer, require_sealed_consumer, repository_id
from .classification import classify_change
from .orchestrator import TEMPLATES
from .syntax import syntax_receipt, format_receipt
from ..submission import request_payload as jenkins_request_payload, submit as jenkins_submit

def _stage_recipe(stage: str, identity: dict) -> str:
    return digest({"stage": stage, "implementation": identity.get("stageImplementationDigest")})
from .classification import classify_rust_source, rust_non_doc_tokens


def _identity(payload: dict) -> dict:
    value = payload.get("identity")
    if not isinstance(value, dict):
        raise JenkinsError("identity_missing", "Workflow identity is required")
    required = {"sessionId", "requestId", "attemptId", "generation", "sourceInputDigest",
                "coverageDigest", "stageImplementationDigest"}
    missing = sorted(required - set(value))
    if missing:
        raise JenkinsError("identity_missing", "Workflow identity fields missing: " + ",".join(missing))
    if type(value["generation"]) is not int or value["generation"] < 1:
        raise JenkinsError("generation_invalid", "workflow generation must be a positive integer")
    return value


def _trusted_driver(repo_root: Path, identity: dict) -> str:
    """Use the deployed sealed driver when one is available.

    Direct fixture tests intentionally run without deployment state.  In that
    mode the caller's digest is accepted only as a controlled test binding.
    A deployed invocation always derives the digest from the active driver
    record and rejects a caller supplied substitute.
    """
    from ..deployment.driver import active_driver_binding
    try:
        binding = active_driver_binding(repo_root)
    except (FileNotFoundError, JenkinsError, ValueError, OSError) as error:
        # Formal deployments must fail closed.  Only an explicitly isolated
        # fixture (without a deployment marker) may supply its sealed driver.
        marker = repo_root / ".jenkins" / "state" / "deployment" / "driver.json"
        if marker.exists():
            raise JenkinsError("driver_binding_missing", "formal workflow has no valid sealed driver") from error
        supplied = identity.get("driverDigest") or identity.get("stageImplementationDigest")
        if not isinstance(supplied, str) or len(supplied) != 64:
            raise JenkinsError("driver_context_missing", "isolated workflow fixture requires a driver digest")
        return supplied
    digest_value = str(binding["digest"])
    supplied = identity.get("driverDigest")
    if supplied and supplied != digest_value:
        raise JenkinsError("driver_context_mismatch", "workflow identity driver differs from active sealed driver")
    return digest_value

def _sealed_classification(record: dict) -> dict:
    """Classify only changed sealed Rust objects by before/after token proof."""
    manifest = record["payload"].get("sourceManifest") or {}
    root = record["payload"].get("objectRoot")
    if not isinstance(root, str):
        raise JenkinsError("sealed_object_root_missing", "sealed object root is required")
    objects = Path(root).absolute() / "inputs" / "objects"
    files: dict[str, str] = {}
    changed = False
    for entry in manifest.get("entries", []):
        if not str(entry.get("path", "")).endswith(".rs") or entry.get("status") not in {"present", "base"}:
            continue
        before = entry.get("beforeHash")
        after = entry.get("sha256")
        if before == after and not manifest.get("baseHead"):
            continue
        changed = True
        obj = objects / str(entry.get("objectDigest"))
        if not obj.is_file():
            raise JenkinsError("sealed_object_missing", "sealed source object is unavailable")
        text = obj.read_text(encoding="utf-8")
        base_head = manifest.get("baseHead")
        if base_head:
            try:
                base_text = subprocess.run(["git", "-C", str(record["payload"].get("repoRoot", ".")), "show", f"{base_head}:{entry['path']}"], capture_output=True, check=True).stdout.decode("utf-8")
            except (OSError, subprocess.CalledProcessError, UnicodeDecodeError):
                return {"kind": "blocked", "reason": "sealed_base_unavailable"}
            if rust_non_doc_tokens(base_text) != rust_non_doc_tokens(text):
                return {"kind": "semantic", "reason": "non_comment_token_change"}
        result = classify_rust_source(text)
        if result.get("kind") == "blocked":
            return result
        files[entry["path"]] = text
    if not changed:
        return {"kind": "blocked", "reason": "no_changed_rust_source"}
    return {"kind": "comments_only", "files": files}


def _light_receipt(state: State, repo_root: Path, execution_id: str, stage: str,
                   identity: dict, evidence: dict, journal: dict | None = None) -> dict:
    authority = TrustedReceiptAuthority(state, repo_root=repo_root)
    manifest = {"executionId": execution_id, "stage": stage,
                "sourceDigest": identity["sourceInputDigest"],
                "recipeDigest": _stage_recipe(stage, identity),
                "coverageDigest": identity["coverageDigest"],
                "driverDigest": identity.get("driverDigest") or identity.get("stageImplementationDigest")}
    authority.register_attempt(manifest)
    state.put("light_attempt", execution_id, {"status":"registered", "sourceDigest": identity["sourceInputDigest"],
                "driverDigest": manifest["driverDigest"], "journal": journal or {},
                "tool": evidence.get("tool"), "commandDigest": evidence.get("commandDigest")})
    output_root = repo_root / ".jenkins" / "tmp"
    output_root.mkdir(parents=True, exist_ok=True)
    return authority.record_light_terminal(execution_id, evidence,
                                          {"root": str(output_root), "files": [], "digest": digest([])})



def _ensure_execution_job(state: State, *, identity: dict, source_ref: str,
                          coverage_digest: str, recipe_ref: str, repo_root: Path,
                          base_url: str | None = None, token: str | None = None,
                          transport=None) -> dict:
    """Persist an execution request before making the Jenkins side effect.

    The immutable request is the source of truth.  Replaying this operation
    returns the recorded queue/build reference and never submits a second
    Jenkins build.  ``transport`` is only used by bounded integration tests;
    production callers use the normal Jenkins HTTP transport.
    """
    repository = str(identity.get("repositoryId") or "repository")
    session = str(identity["sessionId"])
    request_id = str(identity["requestId"])
    submission_id = f"{request_id}:execution:{recipe_ref[:16]}"
    execution_id = identity.get("executionId")
    if not isinstance(execution_id, str) or not execution_id:
        raise JenkinsError("execution_id_missing", "authoritative executionId is required for Jenkins dispatch")
    request = jenkins_request_payload(
        repository_id=repository, session_id=session, request_id=request_id,
        source_ref=source_ref, source_digest=source_ref,
        coverage_ref=coverage_digest, coverage_digest=coverage_digest,
        parameters={"recipeRef": recipe_ref, "generation": str(identity["generation"]),
                    "attemptId": str(identity["attemptId"]), "submissionRequestId": submission_id,
                    "executionId": execution_id},
    )
    # State.submit_request is deliberately called by submit() before HTTP.
    # A caller can disable network delivery while provisioning a flow; the
    # pending request remains durable and reconcile can safely retry it.
    marker = f"{repository}:{session}:{request_id}:{submission_id}"
    previous = state.get("jenkins_submission", marker)
    if previous is not None:
        if previous["payload"].get("delivery") in {"dispatching", "unknown"}:
            raise JenkinsError("delivery_unknown", "Existing Jenkins dispatch must be reconciled", retryable=True)
        return {"request": state.get_request(repository, session, request_id),
                **previous["payload"], "reused": True}
    # Reaching this helper is itself the controlled dispatch authorization:
    # the workflow has already validated source ownership, generation, driver,
    # and recipe identity.  Always persist then submit through the adapter.
    manager = None
    if transport is None:
        from ..deployment.spec import load_spec
        from ..deployment.paths import resolve_paths
        from ..deployment.manager import DeploymentManager
        spec = load_spec(repo_root / ".jenkins" / "deployment-spec.json")
        manager = DeploymentManager(spec, resolve_paths(spec))
    result = jenkins_submit(state, request,
                            base_url=manager.base_url if manager else (base_url or "http://127.0.0.1:18080"),
                            job="zircon-execution", token=None, transport=transport, manager=manager)
    return result


def _project_execution(state: State, repo_root: Path, workflow: dict, *, key: str) -> tuple[dict, list[dict]]:
    execution_id = workflow.get("executionId")
    if not isinstance(execution_id, str):
        return workflow, list(workflow.get("stages", []))
    from .execution import ExecutionEngine
    try:
        observed = ExecutionEngine(state, repo_root=repo_root,
                                   build_root=workflow.get("buildRoot")).observe(execution_id)
    except JenkinsError as error:
        if (repo_root / ".jenkins/state/deployment/driver.json").exists():
            raise
        execution = state.get("execution", execution_id)
        if execution is None:
            return workflow, list(workflow.get("stages", []))
        observed = {**execution["payload"], "executionId": execution_id}
    stage_receipts = observed.get("stageReceipts", {}) or {}
    stages = [dict(item) for item in workflow.get("stages", [])]
    required = {"build", "unit_test", "integration_test", "regression_test"}
    receipts = dict(workflow.get("stageReceipts", {}))
    for item in stages:
        stage = item.get("name")
        receipt_id = stage_receipts.get(stage) if stage in required else None
        if receipt_id:
            receipt = state.get("validation_receipt", receipt_id)
            if receipt and receipt["payload"].get("status") == "passed":
                item["status"] = "passed"; item["receipt"] = receipt["payload"]; receipts[stage] = receipt_id
    projected = {**workflow, "executionStatus": observed.get("status", "waiting"),
                 "stageReceipts": receipts, "stages": stages}
    consumer = state.get("execution_consumer", f"{execution_id}:{key}")
    if consumer and consumer["payload"].get("status") == "released" and workflow.get("status") != "accepted":
        projected.update(status="cancelled", executionStatus="cancelled")
    if observed.get("status") in {"failed", "cancelled", "rejected"} and workflow.get("status") != "accepted":
        projected["status"] = "cancelled" if observed["status"] == "cancelled" else "failed"
        for item in stages:
            if item.get("status") not in {"passed", "reused"}:
                item["status"] = "failed" if item.get("name") in required else "blocked"
    return projected, stages

def handle(action: str, payload: dict, state: State, repo_root: Path, *, domain: str = "flow") -> dict:
    if action in {"source-claim", "source-apply", "source-seal", "source-materialize"}:
        return source_handle(action.removeprefix("source-"), payload, state, repo_root)
    # Raw patch requests have no source digest until the patch is applied and
    # sealed. Dispatch them before normal full identity validation.
    if action in {"prepare-patch", "reconcile-patch"}:
        from ..patches import prepare_patch, reconcile_patch
        partial = payload.get("identity")
        if not isinstance(partial, dict):
            partial = {key: payload[key] for key in
                       ("repositoryId", "sessionId", "requestId", "attemptId", "generation",
                        "stageImplementationDigest", "driverDigest", "buildRoot") if key in payload}
        if action == "prepare-patch":
            trusted = _trusted_driver(repo_root, partial)
            return prepare_patch(state, repo_root, payload, trusted_driver_digest=trusted)
        try:
            trusted = _trusted_driver(repo_root, partial)
        except JenkinsError:
            trusted = partial.get("driverDigest") or partial.get("stageImplementationDigest")
        return reconcile_patch(state, repo_root, payload, trusted_driver_digest=trusted)
    identity = _identity(payload) if action not in {"inventory-maintenance", "gc-maintenance", "reconcile-maintenance"} else payload.get("identity", {})
    key = f"{identity.get('sessionId','maintenance')}:{identity.get('requestId','maintenance')}:{identity.get('attemptId','') }"
    bound = state.get("workflow", key)
    if bound and bound["payload"].get("identity") != identity:
        raise JenkinsError("workflow_identity_mismatch", "workflow identity cannot change during recovery or execution")
    if action == "register-flow":
        sealed_ref = payload.get("sealedInputRef") or identity.get("sourceInputDigest")
        sealed = state.get("sealed_input", sealed_ref)
        if sealed is None or sealed["payload"].get("status") != "sealed":
            raise JenkinsError("sealed_input_missing", "workflow requires a source-owned sealed input reference")
        if sealed_ref != identity.get("sourceInputDigest"):
            raise JenkinsError("sealed_input_digest_mismatch", "sealed input ref is not bound to request identity")
        sealed_payload = sealed["payload"]
        request = state.get_request(identity.get("repositoryId", ""), identity["sessionId"], identity["requestId"])
        if request is None and (repo_root / ".jenkins/state/deployment/driver.json").exists():
            raise JenkinsError("request_not_registered", "formal flow requires its immutable registered request")
        if request:
            intent = request["payload"]
            source = intent.get("sealedInputRef") or intent.get("sourceInputDigest") or intent.get("sourceDigest") or intent.get("sourceRef")
            if request["generation"] != identity["generation"] or source != sealed_ref or intent.get("coverageDigest") != identity["coverageDigest"]:
                raise JenkinsError("request_identity_mismatch", "flow differs from the registered request generation or input")
        # A sealed input is consumable only by the source owner that sealed it.
        # Legacy hand-written fixtures may omit owner metadata; formal records
        # always carry all three fields and therefore require this binding.
        owner = identity.get("attemptId")
        sealed_repo = sealed_payload.get("repositoryId")
        sealed_session = identity.get("sessionId")
        if all(isinstance(value, str) and value for value in (owner, sealed_repo, sealed_session)):
            consumer = require_sealed_consumer(state, repository_id_value=sealed_repo,
                                           session_id=sealed_session, owner=owner,
                                           source_digest=sealed_ref)
            if consumer is None or consumer["payload"].get("status") != "active":
                raise JenkinsError("sealed_consumer_missing", "workflow requires a source-owned sealed consumer binding")
            if consumer["payload"].get("manifestDigest") != sealed_payload.get("sourceDigest"):
                raise JenkinsError("sealed_consumer_mismatch", "sealed consumer manifest does not match input")
        sealed_coverage = sealed_payload.get("coverageDigest")
        if isinstance(sealed_coverage, str) and identity.get("coverageDigest") != sealed_coverage:
            raise JenkinsError("coverage_identity_mismatch", "coverage digest is not bound to sealed source")
        for field in ("repositoryId",):
            bound = sealed_payload.get(field)
            if bound is not None and identity.get(field) is not None and identity.get(field) != bound:
                raise JenkinsError("sealed_identity_mismatch", f"identity {field} differs from sealed source owner")
        driver_digest = _trusted_driver(repo_root, identity)
        patch_refs = list(sealed_payload.get("patchOperationRefs", []))
        supplied_patch = payload.get("patchOperationRef")
        if supplied_patch and patch_refs and supplied_patch not in patch_refs:
            raise JenkinsError("patch_identity_mismatch", "workflow patch differs from sealed source journal")
        if not patch_refs and supplied_patch:
            patch_refs = [supplied_patch]
        from ..resources import canonical_build_root
        selected_root = canonical_build_root(payload.get("buildRoot", sealed_payload.get("buildRoot")))
        sealed_root = sealed_payload.get("buildRoot")
        if sealed_root and selected_root.path != canonical_build_root(sealed_root).path:
            raise JenkinsError("build_root_identity_mismatch", "workflow must retain its sealed input build root")
        execution_id = digest({"source": identity.get("sourceInputDigest"), "coverage": identity.get("coverageDigest"),
                               "recipe": payload.get("recipe", {})})[:32]
        existing = state.get("workflow", key)
        if existing:
            old = existing["payload"]
            immutable = {"identity": old.get("identity"), "sourceDigest": old.get("sourceDigest"),
                         "coverageDigest": old.get("coverageDigest"), "driverDigest": old.get("driverDigest"),
                         "executionId": old.get("executionId"), "sourceManifest": old.get("sourceManifest"),
                         "objectRoot": old.get("objectRoot"), "repoRoot": old.get("repoRoot")}
            requested = {"identity": identity, "sourceDigest": sealed["payload"].get("sourceDigest", sealed_ref),
                         "coverageDigest": identity.get("coverageDigest"), "driverDigest": driver_digest,
                         "executionId": execution_id, "sourceManifest": sealed["payload"]["manifest"],
                         "objectRoot": sealed["payload"].get("objectRoot"), "repoRoot": sealed["payload"].get("repositoryRoot", str(repo_root))}
            if any(immutable.get(k) != requested.get(k) for k in immutable):
                raise JenkinsError("workflow_identity_mismatch", "register-flow cannot replace an immutable contract")
            return response(old.get("status", "pending"), request_id=identity.get("requestId"), observed_generation=identity.get("generation"), executionId=old.get("executionId"))
        state.put("workflow", key, {"identity": identity, "repositoryId": sealed_payload.get("repositoryId") or identity.get("repositoryId"), "sourceDigest": sealed["payload"].get("sourceDigest", sealed_ref), "coverageDigest": identity.get("coverageDigest"), "driverDigest": identity.get("driverDigest") or identity.get("stageImplementationDigest"), "payloadDigest": digest(payload),
                                     "status": "pending", "executionId": execution_id,
                                     "sourceManifest": sealed["payload"]["manifest"], "coverage": sealed["payload"]["coverage"],
                                     "objectRoot": sealed["payload"].get("objectRoot"), "repoRoot": sealed["payload"].get("repositoryRoot", str(repo_root)),
                                     "ownedPaths": list(sealed_payload.get("ownedPaths", ())),
                                     "patchOperationRefs": patch_refs,
                                     "buildRoot": str(selected_root.path), "stages": [], "driverDigest": driver_digest})
        return response("pending", request_id=identity.get("requestId"), observed_generation=identity.get("generation"), executionId=execution_id)
    if action == "compose-flow":
        record = state.get("workflow", key)
        if record is None:
            raise JenkinsError("workflow_not_found", "register-flow must precede compose-flow")
        if payload.get("requestedTemplate") is not None or payload.get("changeSet") is not None:
            raise JenkinsError("caller_change_forbidden", "template and change set must come from source-owned records")
        if record["payload"].get("status") in {"ready", "accepted"}:
            return response(record["payload"]["status"], request_id=identity.get("requestId"), observed_generation=identity.get("generation"), executionId=record["payload"].get("executionId"), template=record["payload"].get("template"), stages=[s["name"] for s in record["payload"].get("stages", [])])
        result = _sealed_classification(record)
        if result["kind"] == "blocked":
            raise JenkinsError("classification_blocked", "change impact cannot be proven", details=result)
        coverage = record["payload"].get("coverage") or {}
        hinted = coverage.get("template") if isinstance(coverage, dict) else None
        # A source-owned comments hint cannot suppress a proven semantic Rust
        # token change.  Keep classification authoritative over caller hints.
        if hinted == "comments_only" and result["kind"] != "comments_only":
            hinted = None
        if hinted in TEMPLATES and hinted != "cache_maintenance":
            template = hinted
        elif result["kind"] == "comments_only":
            template = "comments_only"
        else:
            changed_count = len(result.get("files", {}))
            template = "cross_module" if changed_count > 1 else "module_unit"
        if result["kind"] == "blocked":
            raise JenkinsError("classification_blocked", "sealed source classification is blocked", details=result)
        if template not in TEMPLATES:
            raise JenkinsError("template_unknown", f"Unknown workflow template: {template}")
        stages = list(TEMPLATES[template])
        required_stages = [stage for stage in stages if stage in {"patch", "syntax", "format", "build", "unit_test", "integration_test", "regression_test"}]
        state.put("workflow", key, {**record["payload"], "status": "ready", "template": template,
                                     "stages": [{"name": stage, "status": "pending"} for stage in stages],
                                     "requiredStages": required_stages, "stageReceipts": {}, "stageRecipeDigests": {s: _stage_recipe(s, identity) for s in stages}, "generation": identity.get("generation"), "dagDigest": digest(stages), "requiredOutputs": []})
        return response("ready", request_id=identity.get("requestId"), observed_generation=identity.get("generation"),
                        executionId=record["payload"]["executionId"], template=template, stages=stages)
    if action == "run-stage":
        record = state.get("workflow", key)
        if record is None:
            raise JenkinsError("workflow_not_found", "workflow is not registered")
        if record["payload"].get("status") in {"failed", "cancelled"}:
            raise JenkinsError("workflow_terminal", "failed or cancelled flow cannot run further stages")
        stage_name = payload.get("stage")
        stages = list(record["payload"].get("stages", []))
        names = [item.get("name") for item in stages]
        if stage_name not in names:
            raise JenkinsError("stage_unknown", "stage is not in the composed DAG")
        index = names.index(stage_name)
        # Jenkins retries are idempotent: a terminal stage receipt is reused
        # instead of registering a second light attempt with the same ID.
        if stages[index].get("status") in {"passed", "reused"}:
            existing_receipt = stages[index].get("receipt")
            return response(record["payload"].get("status", "pending"),
                            request_id=identity.get("requestId"),
                            observed_generation=identity.get("generation"),
                            receipt=existing_receipt,
                            executionId=record["payload"].get("executionId"))
        if any(item.get("status") not in {"passed", "reused"} for item in stages[:index]):
            raise JenkinsError("stage_precondition", "prior stage has no accepted receipt", retryable=True)
        sources: list[tuple[str, str]] = []
        manifest = record["payload"].get("sourceManifest") or {}
        root = record["payload"].get("objectRoot")
        if isinstance(root, str):
            for entry in manifest.get("entries", []):
                if str(entry.get("path", "")).endswith(".rs") and entry.get("objectDigest") and entry.get("status") == "present":
                    obj = Path(root).absolute() / "inputs" / "objects" / str(entry["objectDigest"])
                    if obj.is_file():
                        sources.append((str(entry.get("path")), obj.read_text(encoding="utf-8")))
        if stage_name in {"build", "unit_test", "integration_test", "regression_test", "commit", "notify", "gc"}:
            raise JenkinsError("stage_requires_authority", "heavy and side-effect stages require their authoritative adapter", retryable=True)
        if stage_name in {"syntax", "format"} and not sources:
            raise JenkinsError("sealed_source_missing", "syntax and format require sealed source text")
        if stage_name == "patch":
            journal = payload.get("patchOperationRef") or payload.get("patchJournal") or payload.get("operationId")
            if not isinstance(journal, str):
                raise JenkinsError("patch_journal_missing", "source apply operation journal is required")
            references = record["payload"].get("patchOperationRefs") or [journal]
            if journal not in references:
                raise JenkinsError("patch_identity_mismatch", "patch receipt differs from sealed input")
            operations = [state.get_operation(ref) for ref in references]
            for operation in operations:
                if operation is None or operation.get("kind") != "source_patch" or operation.get("status") != "complete":
                    raise JenkinsError("patch_journal_untrusted", "patch stage requires completed source-owned journals")
                after = operation.get("result", {}).get("afterHashes", {})
                entries = {str(e["path"]).casefold(): e.get("sha256") for e in manifest.get("entries", [])}
                if any(entries.get(p) != h for p, h in after.items()):
                    raise JenkinsError("patch_source_mismatch", "patch journal differs from sealed source bytes")
            execution_id = digest({"flow": key, "stage": stage_name, "journal": journal})[:32]
            receipt = _light_receipt(state, repo_root, execution_id, stage_name, identity,
                                     {"status": "passed", "exitCode": 0, "tool": "source-parser",
                                      "commandDigest": digest(["source", "patch"]), "operationId": journal},
                                     {"operations": [{"operationId": op["operationId"], "result": op["result"]} for op in operations]})
            stages[index]["status"] = "passed"
            stages[index]["receipt"] = receipt
            receipts = dict(record["payload"].get("stageReceipts", {})); receipts[stage_name] = execution_id
            state.put("workflow", key, {**record["payload"], "status": "pending", "stages": stages,
                                         "stageReceipts": receipts}, expected_version=record["version"])
            return response("pending", request_id=identity.get("requestId"), observed_generation=identity.get("generation"), receipt=receipt)
        if stage_name == "acceptance":
            authority = TrustedReceiptAuthority(state, repo_root=repo_root)
            flow = {**record["payload"], "flowId": key, "generation": identity.get("generation"),
                    "sourceInputDigest": identity.get("sourceInputDigest")}
            receipt = authority.accept_flow(flow)
            stages[index]["status"] = "passed"; stages[index]["receipt"] = receipt
            state.put("workflow", key, {**record["payload"], "status": "accepted", "stages": stages,
                                         "acceptance": receipt}, expected_version=record["version"])
            return response("accepted", request_id=identity.get("requestId"), observed_generation=identity.get("generation"), receipt=receipt)
        try:
            if stage_name == "syntax":
                checks = [syntax_receipt(text, edition=str(payload.get("edition", "2021")), source_digest=identity.get("sourceInputDigest")) for _, text in sources]
                raw_receipt = {"commandDigest": digest([c.get("commandDigest") for c in checks]), "edition": payload.get("edition", "2021")}
            elif stage_name == "format":
                checks = [format_receipt(text, edition=str(payload.get("edition", "2021")), source_digest=identity.get("sourceInputDigest")) for _, text in sources]
                raw_receipt = {"commandDigest": digest([c.get("commandDigest") for c in checks]), "edition": payload.get("edition", "2021")}
            else:
                raw_receipt = {"status": "passed", "kind": stage_name,
                               "sourceDigest": identity.get("sourceInputDigest")}
            execution_id = digest({"flow": key, "stage": stage_name})[:32]
            receipt = _light_receipt(state, repo_root, execution_id, stage_name, identity,
                                     {"status": "passed", "exitCode": 0, "tool": "rustfmt",
                                      "commandDigest": raw_receipt.get("commandDigest", digest(["rustfmt"])),
                                      "edition": raw_receipt.get("edition", payload.get("edition", "2021"))})
        except JenkinsError as error:
            stages[index]["status"] = "failed"
            for item in stages[index + 1:]: item["status"] = "blocked"
            state.put("workflow", key, {**record["payload"], "status": "failed", "stages": stages}, expected_version=record["version"])
            raise
        stages[index]["status"] = "passed"
        stages[index]["receipt"] = receipt
        receipts = dict(record["payload"].get("stageReceipts", {}))
        receipts[stage_name] = execution_id
        status = "ready" if all(item["status"] in {"passed", "reused"} for item in stages if item["name"] in {"patch", "syntax", "format"}) else "pending"
        state.put("workflow", key, {**record["payload"], "status": status, "stages": stages, "stageReceipts": receipts}, expected_version=record["version"])
        return response(status, request_id=identity.get("requestId"), observed_generation=identity.get("generation"), receipt=receipt)
    if action == "observe-flow":
        record = state.get("workflow", key)
        if record is None: raise JenkinsError("workflow_not_found", "workflow is not registered")
        projected, stages = _project_execution(state, repo_root, record["payload"], key=key)
        if projected != record["payload"]:
            state.put("workflow", key, projected, expected_version=record["version"])
        heavy = [x for x in stages if x.get("name") in {"build", "unit_test", "integration_test", "regression_test"}]
        status = projected.get("executionStatus", projected.get("status", "waiting"))
        if projected.get("status") not in {"failed", "cancelled"} and heavy and all(x.get("status") in {"passed", "reused"} for x in heavy): status = "ready"
        return response(status, request_id=identity.get("requestId"), observed_generation=identity.get("generation"),
                        executionId=projected.get("executionId"), stages=stages)
    if action == "accept-flow":
        record = state.get("workflow", key)
        if record is None:
            raise JenkinsError("workflow_not_found", "workflow is not registered")
        stages = record["payload"].get("stages", [])
        required = set(record["payload"].get("requiredStages", []))
        if not stages or any(item.get("status") not in {"passed", "reused"} for item in stages if item.get("name") in required):
            raise JenkinsError("acceptance_pending", "required authoritative stage receipts are not complete", retryable=True)
        authority = TrustedReceiptAuthority(state, repo_root=repo_root)
        flow = {**record["payload"], "flowId": key,
                "generation": identity.get("generation"),
                "sourceInputDigest": identity.get("sourceInputDigest")}
        receipt = authority.accept_flow(flow)
        stage_items = list(stages)
        for item in stage_items:
            if item.get("name") == "acceptance":
                item["status"] = "passed"
                item["receipt"] = receipt
        state.put("workflow", key, {**record["payload"], "status": "accepted",
                                     "stages": stage_items, "acceptance": receipt},
                  expected_version=record["version"])
        return response("accepted", request_id=identity.get("requestId"),
                        observed_generation=identity.get("generation"), receipt=receipt)
    if action == "finalize-flow":
        record = state.get("workflow", key)
        if record is None or record["payload"].get("status") != "accepted":
            raise JenkinsError("flow_unaccepted", "finalization requires formal acceptance")
        flow = record["payload"]
        stages = [dict(item) for item in flow.get("stages", [])]
        for stage in stages:
            name = stage.get("name")
            if name not in {"commit", "notify", "gc"} or stage.get("status") == "passed":
                continue
            if name == "commit" and flow.get("gitCommitSha"):
                stage.update(status="passed", commitSha=flow["gitCommitSha"])
                continue
            try:
                state.check_authorization(flow["repositoryId"], identity["sessionId"], name if name != "notify" else "notify", flow.get("ownedPaths", []))
            except JenkinsError as error:
                if error.code not in {"action_not_authorized", "path_not_authorized"}:
                    raise
                stage.update(status="skipped", reasonCode="action_not_authorized")
                continue
            if name == "gc" and isinstance(payload.get("cleanupScope"), dict):
                from ..maintenance import garbage_collect
                result = garbage_collect({"identity": identity, "buildRoot": flow["buildRoot"],
                    "operationId": digest({"flow": key, "stage": "gc"}),
                    "cleanupScope": payload["cleanupScope"], "ownedPaths": flow.get("ownedPaths", [])}, state, repo_root)
                stage.update(status="passed", receipt=result)
            elif name == "commit" and not payload.get("commitRequested"):
                stage.update(status="skipped", reasonCode="action_not_requested")
            else:
                stage.update(status="pending", reasonCode="explicit_delivery_required" if name == "notify" else "bounded_scope_required")
        # Enqueue a completion notification so the Notify stage has something to deliver.
        if flow.get("gitCommitSha") or payload.get("commitRequested"):
            event_id = f"{identity['sessionId']}:{identity['requestId']}:{identity['attemptId']}:complete"
            destination_id = payload.get("notifyDestination") or "default"
            try:
                from ..notifications import NotificationOutbox
                outbox = NotificationOutbox(state)
                outbox.enqueue(event_id, destination_id, {
                    "kind": "flow_complete",
                    "sessionId": identity["sessionId"],
                    "requestId": identity["requestId"],
                    "attemptId": identity["attemptId"],
                    "repositoryId": flow.get("repositoryId"),
                    "gitCommitSha": flow.get("gitCommitSha"),
                    "template": flow.get("template"),
                    "acceptanceReceipt": flow.get("acceptance"),
                })
            except JenkinsError:
                pass  # notification enqueue failure must never block finalization
        from ..source import PathClaim, release_paths
        if flow.get("gitCommitSha"):
            claims = [PathClaim.from_record(row) for row in state.list("path_claims")
                      if row["payload"].get("owner") == identity["attemptId"]]
            release_paths(state, identity["attemptId"], claims)
        current = state.get("workflow", key)
        state.put("workflow", key, {**current["payload"], "stages": stages,
                  "postAcceptanceStatus": "complete" if all(s.get("status") in {"passed", "reused", "skipped"} for s in stages) else "pending"}, expected_version=current["version"])
        return response("complete", request_id=identity.get("requestId"), executionId=flow["executionId"])
    if action == "claim-execution":
        raise JenkinsError("legacy_execution_claim_removed",
                           "execution claims must be created by submit-execution from an immutable recipeRef")
    if action == "ensure-execution-job":
        execution = payload.get("executionId")
        if not isinstance(execution, str):
            raise JenkinsError("execution_id_missing", "executionId is required")
        record = state.get("workflow", key)
        if record is None or record["payload"].get("executionId") != execution:
            raise JenkinsError("execution_identity_mismatch", "execution is not bound to this workflow")
        recipe_ref = record["payload"].get("recipeRef")
        if not isinstance(recipe_ref, str):
            raise JenkinsError("recipe_plan_missing", "workflow recipe is missing")
        dispatch = _ensure_execution_job(state, identity={**identity, "executionId": execution},
            source_ref=record["payload"]["sourceDigest"], coverage_digest=record["payload"]["coverageDigest"],
            recipe_ref=recipe_ref, repo_root=repo_root)
        state.put("workflow", key, {**record["payload"], "jenkinsSubmission": dispatch}, expected_version=record["version"])
        return response("pending", request_id=identity.get("requestId"), executionId=execution,
                        recipeRef=recipe_ref, jenkinsSubmission=dispatch)
    if action == "submit-execution":
        record = state.get("workflow", key)
        if record is None:
            raise JenkinsError("workflow_not_found", "workflow is not registered")
        sealed_ref = record["payload"].get("sourceDigest")
        if not isinstance(sealed_ref, str) or sealed_ref != identity.get("sourceInputDigest"):
            raise JenkinsError("sealed_input_identity_mismatch", "execution must use the workflow sealed input")
        from .planning import RecipePlanner, get_recipe_plan
        from .execution import ExecutionEngine
        driver_digest = record["payload"].get("driverDigest") or _trusted_driver(repo_root, identity)
        driver = {"digest": driver_digest, "generation": str(identity.get("generation"))}
        try:
            from ..deployment.driver import active_driver_binding
            binding = active_driver_binding(repo_root)
            driver["runtimeOperationId"] = str(binding.get("generation") or "")
        except Exception as error:
            raise JenkinsError("runtime_binding_missing", "Formal execution requires its live runtime binding") from error
        template = str(record["payload"].get("template", "module_unit"))
        existing_ref = record["payload"].get("recipeRef")
        if existing_ref:
            plan_payload = get_recipe_plan(state, existing_ref)
        else:
            planner = RecipePlanner(state, repo_root=repo_root,
                                     build_root=record["payload"].get("buildRoot"))
            manifest = record["payload"].get("sourceManifest") or {}
            owned = set(record["payload"].get("ownedPaths", ()))
            paths = [str(e.get("path")) for e in manifest.get("entries", ())
                      if isinstance(e, dict) and e.get("path") in owned]
            if not paths:
                raise JenkinsError("owned_change_missing", "Cargo planning requires a proven owned change scope")
            plan = planner.plan(sealed_input_ref=sealed_ref, driver=driver,
                                changed_paths=paths, template=template)
            plan_payload = {"recipe": plan.recipe, "recipeDigest": plan.recipe_ref,
                            "sourceRef": plan.source_ref, "coverageRef": plan.coverage_ref,
                            "driver": plan.driver, "generation": plan.generation, "status": "planned"}
        recipe = dict(plan_payload["recipe"])
        engine = ExecutionEngine(state, repo_root=repo_root,
                                 build_root=record["payload"].get("buildRoot"))
        claimed = engine.claim(source=sealed_ref, recipe=plan_payload["recipeDigest"], driver=driver,
                               consumer_id=key)
        execution_id = claimed["executionId"]
        request = state.get_request(str(record["payload"].get("repositoryId") or ""),
                                    str(identity["sessionId"]), str(identity["requestId"])) if hasattr(state, "get_request") else None
        if request is not None:
            state.transition_request(request["repositoryId"], request["sessionId"], request["requestId"],
                                     int(identity["generation"]), request["status"], "running",
                                     {"executionId": execution_id, "recipeRef": plan_payload["recipeDigest"]})
        registered = state.put("workflow", key, {**record["payload"], "executionId": execution_id,
                                     "recipeRef": plan_payload["recipeDigest"],
                                     "executionStatus": claimed.get("status", "claimed"),
                                     "stageRecipeDigests": {**dict(record["payload"].get("stageRecipeDigests", {})),
                                                               **{stage: plan_payload["recipeDigest"] for stage in record["payload"].get("requiredStages", ())
                                                                  if stage in {"build", "unit_test", "integration_test", "regression_test"}}},
                                     "requiredOutputsByStage": {stage: ["stdout.log", "stderr.log"] for stage in record["payload"].get("requiredStages", ())
                                                               if stage in {"build", "unit_test", "integration_test", "regression_test"}},
                                      }, expected_version=record["version"])
        dispatch = _ensure_execution_job(
            state, identity={**identity, "executionId": execution_id}, source_ref=sealed_ref,
            coverage_digest=str(record["payload"].get("coverageDigest") or identity["coverageDigest"]),
            recipe_ref=plan_payload["recipeDigest"], repo_root=repo_root,
            transport=payload.get("transport"),
        )
        fresh = state.get("workflow", key)
        state.put("workflow", key, {**fresh["payload"],
                  "jenkinsSubmission": {k: v for k, v in dispatch.items() if k != "request"}}, expected_version=fresh["version"])
        return response("pending", request_id=identity.get("requestId"),
                        observed_generation=identity.get("generation"), executionId=execution_id,
                        recipeRef=plan_payload["recipeDigest"], jenkinsSubmission=dispatch)
    if action == "run-execution":
        execution = payload.get("executionId")
        if not isinstance(execution, str):
            raise JenkinsError("execution_id_missing", "executionId is required")
        record = state.get("execution", execution)
        if record is None:
            raise JenkinsError("execution_not_found", "execution must be claimed first")
        workflow = state.get("workflow", key)
        if workflow is None or workflow["payload"].get("executionId") != execution:
            raise JenkinsError("execution_identity_mismatch", "execution is not bound to this workflow")
        ep = record["payload"]
        expected = {"sourceDigest": workflow["payload"].get("sourceDigest"),
                    "coverageDigest": workflow["payload"].get("coverageDigest"),
                    "driverDigest": workflow["payload"].get("driverDigest"),
                    "generation": identity.get("generation")}
        plan = ep.get("recipeRef")
        if not isinstance(plan, str) or ep.get("sealedInputRef") != expected["sourceDigest"]:
            raise JenkinsError("execution_identity_mismatch", "execution recipe or source binding differs")
        from .planning import get_recipe_plan
        plan_record = get_recipe_plan(state, plan)
        if str(plan_record.get("generation")) != str(expected["generation"]):
            raise JenkinsError("execution_generation_mismatch", "execution generation differs from workflow")
        driver = plan_record.get("driver") or {}
        if isinstance(driver, dict) and driver.get("digest") != expected["driverDigest"]:
            raise JenkinsError("execution_driver_mismatch", "execution driver differs from workflow")
        coverage_ref = plan_record.get("coverageRef") or plan_record.get("recipe", {}).get("coverageDigest")
        if coverage_ref is not None and coverage_ref != expected["coverageDigest"]:
            raise JenkinsError("execution_coverage_mismatch", "execution coverage differs from workflow")
        from .execution import ExecutionEngine
        engine = ExecutionEngine(state, repo_root=repo_root,
                                 build_root=workflow["payload"].get("buildRoot"))
        result = engine.run(execution, source=ep.get("sealedInputRef"))
        status = result.get("status", "waiting")
        req = state.get_request(str(workflow["payload"].get("repositoryId") or ""),
                                str(identity["sessionId"]), str(identity["requestId"])) if hasattr(state, "get_request") else None
        if req is not None and status in {"passed", "failed", "cancelled", "pending"}:
            target = "waiting" if status == "pending" else ("failed" if status == "failed" else "running")
            state.transition_request(req["repositoryId"], req["sessionId"], req["requestId"],
                                     int(identity["generation"]), req["status"], target,
                                     {"executionId": execution, "status": status})
        return response(status, executionId=execution, terminalProof=result.get("terminalProof"),
                        retryable=status in {"pending", "failed"})
    if action in {"observe-execution", "reconcile-execution"}:
        execution = payload.get("executionId")
        if not isinstance(execution, str):
            raise JenkinsError("execution_id_missing", "executionId is required")
        workflow = state.get("workflow", key)
        if workflow is None or workflow["payload"].get("executionId") != execution:
            raise JenkinsError("execution_identity_mismatch", "The execution is not bound to this workflow")
        from .execution import ExecutionEngine
        result = ExecutionEngine(state, repo_root=repo_root,
            build_root=workflow["payload"]["buildRoot"]).reconcile(execution)
        return response(result.get("status", "reconciling"), request_id=identity.get("requestId"),
                        executionId=execution, terminalProof=result.get("terminalProof"),
                        retryable=result.get("status") in {"claimed", "running", "pending", "waiting"})
    if action == "finalize-execution":
        execution = payload.get("executionId")
        record = state.get("execution", execution) if isinstance(execution, str) else None
        if record is None:
            raise JenkinsError("execution_not_found", "execution is not registered")
        proof = record["payload"].get("terminalProof")
        if not isinstance(proof, dict) or not proof.get("complete"):
            raise JenkinsError("terminal_proof_missing", "execution has no complete native termination proof", retryable=True)
        return response(record["payload"].get("status", "waiting"), executionId=execution, terminalProof=proof)
    if action in {"cancel-execution", "cancel-flow"}:
        execution = payload.get("executionId")
        if not isinstance(execution, str):
            raise JenkinsError("execution_id_missing", "executionId is required")
        workflow = state.get("workflow", key)
        if workflow is None or workflow["payload"].get("executionId") != execution:
            raise JenkinsError("execution_identity_mismatch", "The execution is not bound to this workflow")
        from .execution import ExecutionEngine
        result = ExecutionEngine(state, repo_root=repo_root,
                                 build_root=workflow["payload"]["buildRoot"]).cancel(
                                     execution, key)
        fresh = state.get("workflow", key)
        if fresh["payload"].get("status") != "accepted":
            state.put("workflow", key, {**fresh["payload"], "status": "cancelled", "consumerCancelled": True}, expected_version=fresh["version"])
        return response(result.get("status", "cancel_requested"), executionId=execution,
                        retryable=result.get("status") not in {"cancelled", "failed", "passed"})
    if action == "reconcile-flow":
        workflow = state.get("workflow", key)
        execution = payload.get("executionId")
        if workflow is None or (execution is not None and workflow["payload"].get("executionId") != execution):
            raise JenkinsError("execution_identity_mismatch", "The execution is not bound to this workflow")
        if payload.get("error") or workflow["payload"].get("status") in {"failed", "cancelled"}:
            # Write a durable failure evidence file before attempting compensation.
            # This is best-effort: a write failure must never block the recovery path.
            try:
                import json as _json
                import time as _time
                _flow = workflow["payload"]
                _build_root = _flow.get("buildRoot") or str(repo_root / ".jenkins" / "builds" / "zircon-jenkins")
                _failures_dir = Path(_build_root) / "failures"
                _failures_dir.mkdir(parents=True, exist_ok=True)
                _evidence = {
                    "failedAt": _time.time(),
                    "sessionId": identity.get("sessionId"),
                    "requestId": identity.get("requestId"),
                    "attemptId": identity.get("attemptId"),
                    "generation": identity.get("generation"),
                    "executionId": _flow.get("executionId"),
                    "template": _flow.get("template"),
                    "status": _flow.get("status"),
                    "failedStages": [s for s in _flow.get("stages", []) if s.get("status") in {"failed", "blocked"}],
                    "error": str(payload.get("error") or "")[:2000],
                }
                _evidence_path = _failures_dir / f"failure-{identity.get('requestId', 'unknown')}-{identity.get('attemptId', 'unknown')}.json"
                _evidence_path.write_text(_json.dumps(_evidence, ensure_ascii=False, indent=2), encoding="utf-8")
            except Exception:
                pass  # evidence write failure must never block reconciliation
            from .recovery import recover_flow
            return recover_flow(state, repo_root, key, error=payload.get("error"))
        if workflow["payload"].get("template") == "comments_only":
            return response(workflow["payload"]["status"], request_id=identity.get("requestId"))
        execution = workflow["payload"].get("executionId")
        if state.get("execution", execution) is None:
            return response(workflow["payload"].get("status", "pending"), request_id=identity.get("requestId"))
        from .execution import ExecutionEngine
        result = ExecutionEngine(state, repo_root=repo_root,
                                 build_root=workflow["payload"]["buildRoot"]).reconcile(execution)
        workflow = state.get("workflow", key)
        if workflow is not None and workflow["payload"].get("executionId") == execution:
            status = result.get("status", "reconciling")
            projected, _stages = _project_execution(state, repo_root, workflow["payload"], key=key)
            state.put("workflow", key, projected, expected_version=workflow["version"])
        return response(result.get("status", "reconciling"), request_id=identity.get("requestId"),
                        executionId=execution, retryable=result.get("status") in {"claimed", "running", "pending", "waiting", "cancel_requested"})
    if action in {"inventory-maintenance", "gc-maintenance", "reconcile-maintenance"}:
        raise JenkinsError("maintenance_requires_authority", "maintenance must be submitted through zircon-maintenance")
    raise JenkinsError("operation_unknown", f"Unknown workflow action: {action}")


def handle_gap(action: str, payload: dict, state: State, repo_root: Path) -> dict:
    """Handler for the 'gap' domain used by zircon-gap.groovy.

    Bridges Pipeline control calls to the GapState SQLite layer.
    Only validate-claim and accept are needed at runtime; CLI commands own
    the other state transitions (claim, release, submit).
    """
    from ..gap.state import GapState
    from ..contracts import JenkinsError, response

    gap = GapState(repo_root / ".jenkins" / "state" / "coordination.sqlite3")
    gap.ensure_schema()

    if action == "validate-claim":
        issue_id = payload.get("issueId")
        session_id = payload.get("sessionId")
        if not issue_id or not session_id:
            raise JenkinsError("gap_identity_missing", "issueId and sessionId are required")
        issue = gap.get_issue(issue_id)
        if issue is None:
            raise JenkinsError("gap_not_found", f"Issue {issue_id!r} not found")
        if issue["status"] != "claimed":
            raise JenkinsError("gap_not_claimed",
                               f"Issue {issue_id!r} is {issue['status']!r}, not claimed")
        # Verify the active claim belongs to this session.
        import sqlite3
        con = sqlite3.connect(str(gap._path), timeout=10)
        try:
            row = con.execute(
                "SELECT session_id FROM gap_claims WHERE issue_id=? AND outcome IS NULL",
                (issue_id,)
            ).fetchone()
        finally:
            con.close()
        if row is None or row[0] != session_id:
            owner = row[0] if row else "unknown"
            raise JenkinsError("gap_not_owner",
                               f"Issue {issue_id!r} is claimed by {owner!r}, not {session_id!r}")
        return {"status": "claimed", "issueId": issue_id, "sessionId": session_id}

    if action == "accept":
        issue_id = payload.get("issueId")
        receipt_ref = payload.get("receiptRef")
        if not issue_id or not receipt_ref:
            raise JenkinsError("gap_identity_missing", "issueId and receiptRef are required")
        gap.accept(issue_id, receipt_ref)
        return {"status": "accepted", "issueId": issue_id, "receiptRef": receipt_ref}

    raise JenkinsError("gap_action_unknown", f"Unknown gap action: {action}")

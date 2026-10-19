"""CLI handlers for flow/stage/execution domains.

Handlers persist only small orchestration records.  Source, resource, process,
artifact and validation authorities remain in their own support modules.
"""
from __future__ import annotations

import os

from pathlib import Path
import difflib
import hashlib
import subprocess
from typing import Any

from ..contracts import JenkinsError, digest, response
from ..state import State
from ..validation import TrustedReceiptAuthority
from ..validation.receipts import HEAVY_STAGES, LIGHT_STAGES
from ..source import handle as source_handle, get_sealed_consumer, require_sealed_consumer, repository_id
from .classification import classify_change
from .orchestrator import TEMPLATES
from .syntax import REVIEW_SUFFIXES, format_receipt, review_receipt, syntax_receipt
from ..submission import request_payload as jenkins_request_payload, submit as jenkins_submit

def _stage_recipe(stage: str, identity: dict) -> str:
    return digest({"stage": stage, "implementation": identity.get("stageImplementationDigest")})
from .classification import classify_rust_source, rust_lexical_evidence


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

def _require_registered_execution(state: State, repo_root: Path, flow: dict, consumer_id: str) -> None:
    """Keep the planner's execution binding separate from registration replay."""
    recipe_ref = flow.get("recipeRef")
    if not recipe_ref:
        if flow.get("executionId") != flow.get("registrationExecutionId", flow.get("executionId")):
            raise JenkinsError("execution_identity_mismatch", "An unsubmitted flow changed its registration identity")
        return
    from .execution import ExecutionKey, _execution_identity
    from .planning import get_recipe_plan
    from ..deployment.driver import active_driver_binding
    from ..resources import canonical_build_root

    plan = get_recipe_plan(state, recipe_ref)
    execution_id = flow.get("executionId")
    execution = state.get("execution", execution_id)
    key_digest = _execution_identity(execution, execution_id)
    payload = execution["payload"]
    driver = plan.get("driver") or {}
    active = active_driver_binding(repo_root)
    expected_key = ExecutionKey(flow.get("sourceDigest"), plan["recipe"].get("preparationKey"),
                                recipe_ref, flow.get("driverDigest"))
    if (key_digest != expected_key.digest or plan.get("sourceRef") != flow.get("sourceDigest")
            or plan.get("coverageRef") != flow.get("coverageDigest")
            or payload.get("coverageDigest") != flow.get("coverageDigest")
            or str(plan.get("generation")) != str(flow.get("generation"))
            or str(payload.get("generation")) != str(flow.get("generation"))
            or driver != payload.get("driver") or driver.get("digest") != flow.get("driverDigest")
            or driver.get("digest") != active.get("digest")
            or str(driver.get("generation")) != str(flow.get("generation"))
            or driver.get("runtimeOperationId") != (active.get("runtimeOperationId") or active.get("generation"))):
        raise JenkinsError("execution_identity_mismatch", "Registered flow differs from its immutable execution plan")
    root = canonical_build_root(flow.get("buildRoot"))
    if (canonical_build_root(payload.get("buildRoot")).path != root.path
            or canonical_build_root(plan["recipe"].get("buildRoot")).path != root.path):
        raise JenkinsError("execution_root_mismatch", "Registered execution differs from its approved root")
    binding = state.get("execution_binding", digest({"consumer": consumer_id, "executionKey": key_digest}))
    consumer = state.get("execution_consumer", f"{execution_id}:{consumer_id}")
    if (not binding or not consumer or binding["payload"].get("executionId") != execution_id
            or binding["payload"].get("consumerId") != consumer_id
            or consumer["payload"].get("executionId") != execution_id
            or consumer["payload"].get("consumerId") != consumer_id):
        raise JenkinsError("execution_identity_mismatch", "Registered flow lacks its exact execution consumer binding")


def _require_flow_source_preparation(state: State, reference: str, *, identity: dict,
                                     source_digest: str, preparation_kind: str) -> dict:
    if preparation_kind == 'already-applied-patch':
        from ..already_applied import require_already_applied_preparation
        return require_already_applied_preparation(state, reference, identity=identity,
                                                  source_digest=source_digest)
    if preparation_kind == 'current-source':
        from ..current_source import require_source_preparation
        operation = require_source_preparation(state, reference, identity=identity,
                                               source_digest=source_digest)
        if operation['payload'].get('kind') != 'current-source':
            raise JenkinsError('source_preparation_mismatch', 'Current source requires its own preparation type')
        return operation
    raise JenkinsError('source_preparation_mismatch', 'Unknown coordinator source preparation type')


def _sealed_classification(record: dict, state: State | None = None) -> dict:
    """Classify changed sealed Rust objects only after an original-byte proof.

    A sealed manifest's beforeHash is the claimed preimage identity, while
    sha256/objectDigest identify the candidate bytes. Equal hashes describe
    an unchanged entry and are skipped even when a Git base is present. For a
    changed entry, the patch journal's beforeObjects is authoritative; a Git
    base is only a legacy fallback when its blob bytes hash to that preimage.
    """
    payload = record.get("payload") or {}
    manifest = payload.get("sourceManifest") or {}
    root = payload.get("objectRoot")
    if not isinstance(root, str):
        raise JenkinsError("sealed_object_root_missing", "sealed object root is required")
    object_root = Path(root).absolute()
    objects = object_root / "inputs" / "objects"
    files: dict[str, str] = {}
    changed = False
    patch_refs = payload.get("patchOperationRefs") or []
    journal_before: dict[str, tuple[object, str | None]] = {}
    journal_after: dict[str, str | None] = {}
    applied_preparation = manifest.get('preparationKind') == 'already-applied-patch'
    if applied_preparation:
        reference = payload.get('sourcePreparationRef')
        if state is None or not isinstance(reference, str) or patch_refs:
            return {'kind': 'blocked', 'reason': 'source_preparation_untrusted'}
        try:
            operation = _require_flow_source_preparation(
                state, reference, identity=payload.get('identity') or {},
                source_digest=payload.get('sourceDigest'), preparation_kind='already-applied-patch')
        except JenkinsError as error:
            return {'kind': 'blocked', 'reason': error.code}
        before = operation['payload'].get('beforeObjects')
        after = operation['payload'].get('expectedAfterHashes')
        if (not isinstance(before, dict) or not isinstance(after, dict)
                or set(before) != set(after) or operation['payload'].get('objectRoot') != root):
            return {'kind': 'blocked', 'reason': 'source_preparation_mismatch'}
        journal_before = {path: (checksum, root) for path, checksum in before.items()}
        journal_after = dict(after)
    if state is not None and patch_refs:
        for reference in patch_refs:
            if not isinstance(reference, str):
                return {"kind": "blocked", "reason": "sealed_before_unavailable"}
            operation = state.get_operation(reference)
            if (not operation or operation.get("kind") != "source_patch"
                    or operation.get("status") != "complete"):
                return {"kind": "blocked", "reason": "sealed_before_unavailable"}
            operation_payload = operation.get("payload")
            if not isinstance(operation_payload, dict):
                return {"kind": "blocked", "reason": "sealed_before_unavailable"}
            before_objects = operation_payload.get("beforeObjects")
            if not isinstance(before_objects, dict):
                return {"kind": "blocked", "reason": "sealed_before_unavailable"}
            operation_root = operation_payload.get("objectRoot")
            if not isinstance(operation_root, str):
                return {"kind": "blocked", "reason": "sealed_before_object_root_missing"}
            for raw_path, captured in before_objects.items():
                path_key = str(raw_path).replace(chr(92), "/").casefold()
                prior = journal_before.get(path_key)
                if prior is not None and prior[0] != captured:
                    return {"kind": "blocked", "reason": "sealed_before_hash_mismatch"}
                journal_before[path_key] = (captured, operation_root)
            expected_after = operation_payload.get("expectedAfterHashes")
            if isinstance(expected_after, dict):
                for raw_path, captured in expected_after.items():
                    path_key = str(raw_path).replace(chr(92), "/").casefold()
                    prior = journal_after.get(path_key)
                    if prior is not None and prior != captured:
                        return {"kind": "blocked", "reason": "sealed_after_hash_mismatch"}
                    journal_after[path_key] = captured

    def _valid_digest(value: object) -> bool:
        return (isinstance(value, str) and len(value) == 64
                and all(char in "0123456789abcdef" for char in value.lower()))

    manifest_entries = {
        str(entry.get("path", "")).replace(chr(92), "/").casefold(): entry
        for entry in manifest.get("entries", [])
        if isinstance(entry, dict)
    }
    # A comments-only flow must account for every journal/manifest path. A
    # non-Rust postimage is semantic, while a Rust creation/deletion has no
    # trustworthy before token stream and therefore remains blocked.
    for path_key, (captured, _operation_root) in journal_before.items():
        if path_key not in journal_after:
            return {"kind": "blocked", "reason": "sealed_after_unavailable"}
        candidate_after = journal_after[path_key]
        if captured == candidate_after:
            continue
        if not path_key.endswith(".rs"):
            return {"kind": "semantic", "reason": "non_rust_source_change"}
        entry = manifest_entries.get(path_key)
        if applied_preparation and (captured is None or candidate_after is None):
            if (not isinstance(entry, dict) or entry.get('sha256') != candidate_after
                    or entry.get('beforeHash') != captured):
                return {'kind': 'blocked', 'reason': 'sealed_after_hash_mismatch'}
            return {'kind': 'semantic', 'reason': 'rust_source_added_or_deleted'}
        if not isinstance(entry, dict) or entry.get("status") not in {"present", "base"}:
            return {"kind": "blocked", "reason": "sealed_before_unavailable"}
    for path_key in journal_after:
        if path_key not in journal_before:
            return {"kind": "blocked", "reason": "sealed_before_unavailable"}
    for entry in manifest_entries.values():
        entry_path = str(entry.get("path", ""))
        before_value = entry.get("beforeHash")
        after_value = entry.get("sha256")
        if before_value == after_value:
            continue
        if not entry_path.endswith(".rs"):
            return {"kind": "semantic", "reason": "non_rust_source_change"}
        if entry.get("status") not in {"present", "base"}:
            return {"kind": "blocked", "reason": "sealed_before_unavailable"}

    def _preimage(path: str, expected: object, operation_root: str | None) -> tuple[bytes | None, str | None]:
        if not _valid_digest(expected):
            return None, "sealed_before_unavailable"
        roots = [object_root]
        if operation_root is not None:
            operation_store = Path(operation_root).absolute()
            if operation_store.as_posix().casefold() != object_root.as_posix().casefold():
                roots.insert(0, operation_store)
        for candidate_root in roots:
            candidate = candidate_root / "inputs" / "objects" / expected
            if not candidate.is_file():
                continue
            try:
                original = candidate.read_bytes()
            except OSError:
                return None, "sealed_before_object_unreadable"
            if hashlib.sha256(original).hexdigest() == expected:
                return original, None
            return None, "sealed_before_object_corrupt"
        base_head = manifest.get("baseHead")
        if not isinstance(base_head, str) or not base_head:
            return None, "sealed_before_unavailable"
        try:
            base_result = subprocess.run(
                ["git", "-c", "core.fsmonitor=false", "-c", "core.hooksPath=" + os.devnull, "-C", str(payload.get("repoRoot", ".")),
                 "show", f"{base_head}:{path}"],
                capture_output=True, check=True)
            base_bytes = base_result.stdout
        except (OSError, subprocess.CalledProcessError):
            return None, "sealed_base_unavailable"
        if not isinstance(base_bytes, bytes) or hashlib.sha256(base_bytes).hexdigest() != expected:
            return None, "sealed_base_mismatch"
        return base_bytes, None

    for entry in manifest.get("entries", []):
        path = str(entry.get("path", ""))
        if not path.endswith(".rs") or entry.get("status") not in {"present", "base"}:
            continue
        path_key = path.replace(chr(92), "/").casefold()
        before = entry.get("beforeHash")
        after = entry.get("sha256")
        journal = journal_before.get(path_key)
        if journal is not None:
            preimage_hash, operation_root = journal
            if journal_after.get(path_key, after) != after:
                return {"kind": "blocked", "reason": "sealed_after_hash_mismatch"}
            # A journal entry whose original and candidate bytes are equal is
            # an unchanged/no-op patch and must not trigger Git or object reads.
            if preimage_hash == after:
                continue
        else:
            operation_root = None
            preimage_hash = before
            # A base tree is not evidence that an equal-hash entry changed.
            if before == after:
                continue
        changed = True
        object_digest = entry.get("objectDigest")
        if not _valid_digest(object_digest):
            raise JenkinsError("sealed_object_missing", "sealed source object digest is unavailable")
        obj = objects / object_digest
        if not obj.is_file():
            raise JenkinsError("sealed_object_missing", "sealed source object is unavailable")
        try:
            candidate_bytes = obj.read_bytes()
        except OSError as error:
            raise JenkinsError("sealed_object_unreadable", "sealed source object cannot be read") from error
        if (not _valid_digest(after) or after != object_digest
                or hashlib.sha256(candidate_bytes).hexdigest() != object_digest):
            raise JenkinsError("sealed_object_corrupt", "sealed source object hash does not match its manifest")
        try:
            text = candidate_bytes.decode("utf-8")
        except UnicodeDecodeError:
            return {"kind": "blocked", "reason": "sealed_source_not_utf8"}
        before_bytes, before_reason = _preimage(path, preimage_hash, operation_root)
        if before_reason is not None:
            return {"kind": "blocked", "reason": before_reason}
        try:
            before_text = before_bytes.decode("utf-8")
            before_evidence = rust_lexical_evidence(before_text)
            after_evidence = rust_lexical_evidence(text)
            before_docs = tuple(before_text[start:end] for start, end in before_evidence["doc_comment_spans"])
            after_docs = tuple(text[start:end] for start, end in after_evidence["doc_comment_spans"])
            if before_docs != after_docs:
                return {"kind": "semantic", "reason": "documentation_comment"}
            if before_evidence["tokens"] != after_evidence["tokens"]:
                return {"kind": "semantic", "reason": "non_comment_token_change"}
        except (UnicodeDecodeError, ValueError):
            return {"kind": "blocked", "reason": "sealed_source_tokenization_failed"}
        # Token equality proves ordinary inline comments too. Source-observing
        # macros and directives need compilation even when the tokens match.
        for evidence in (before_evidence, after_evidence):
            if evidence["line_sensitive_macros"]:
                return {"kind": "semantic", "reason": "line_sensitive_macro"}
            if (evidence["source_embedding_macros"] or evidence["conditional_build_sensitive"]
                    or evidence["rustc_directive_comments"]):
                return {"kind": "semantic", "reason": "build_or_conditional_directive"}
        files[path] = text
    if not changed:
        return {"kind": "blocked", "reason": "no_changed_rust_source"}
    # A different sealed module can embed the changed file's bytes. Inspect
    # that same input closure before granting the no-compilation template.
    for entry in manifest.get("entries", []):
        path = str(entry.get("path", ""))
        if path in files or not path.endswith(".rs") or entry.get("status") not in {"present", "base"}:
            continue
        object_digest = entry.get("objectDigest")
        if not _valid_digest(object_digest):
            raise JenkinsError("sealed_object_missing", "sealed source closure object is unavailable")
        try:
            closure_bytes = (objects / object_digest).read_bytes()
        except OSError as error:
            raise JenkinsError("sealed_object_unreadable", "sealed source closure cannot be read") from error
        if hashlib.sha256(closure_bytes).hexdigest() != object_digest or entry.get("sha256") != object_digest:
            raise JenkinsError("sealed_object_corrupt", "sealed source closure hash does not match")
        try:
            evidence = rust_lexical_evidence(closure_bytes.decode("utf-8"))
        except (UnicodeDecodeError, ValueError):
            return {"kind": "blocked", "reason": "sealed_source_tokenization_failed"}
        if evidence["source_embedding_macros"]:
            return {"kind": "semantic", "reason": "source_embedding_in_input_closure"}
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
    # Preserve the immutable typed identity on this secondary delivery.  The
    # source/coverage and attempt/generation above still come from the caller
    # and are checked by submission.submit against the original request.
    original = state.get_request(repository, session, request_id)
    if original is not None:
        from ..submission import _submission_fields
        bound = original["payload"]
        if original.get("payloadDigest") != digest(bound):
            raise JenkinsError("submission_identity_mismatch", "Registered request payload is corrupt")
        fields = _submission_fields(bound)
        supplied_driver = identity.get("stageImplementationDigest") or identity.get("driverDigest")
        if fields.get("STAGE_IMPLEMENTATION_DIGEST") and str(supplied_driver) != fields["STAGE_IMPLEMENTATION_DIGEST"]:
            raise JenkinsError("submission_identity_mismatch", "Submission driver differs from registered request")
        for name in ("CURRENT_SOURCE_REF", "SOURCE_PREPARATION_REF", "PATCH_REQUEST_REF",
                     "PATCH_OPERATION_REF", "STAGE_IMPLEMENTATION_DIGEST", "RUNTIME_OPERATION_ID", "BUILD_ROOT"):
            if name in fields:
                request["parameters"][name] = fields[name]
        if "RUNTIME_OPERATION_ID" not in fields and fields.get("SOURCE_PREPARATION_REF"):
            preparation = state.get_operation(fields["SOURCE_PREPARATION_REF"])
            if (preparation and preparation["kind"] == "source_preparation"
                    and preparation["status"] == "complete"
                    and preparation["payloadDigest"] == digest(preparation["payload"])
                    and preparation["payload"].get("runtimeOperationId")):
                request["parameters"]["RUNTIME_OPERATION_ID"] = preparation["payload"]["runtimeOperationId"]
        if "RUNTIME_OPERATION_ID" not in request["parameters"] and fields.get("PATCH_REQUEST_REF"):
            intent = state.get("patch_intent", fields["PATCH_REQUEST_REF"])
            if (intent and digest(intent["payload"]) == fields["PATCH_REQUEST_REF"]
                    and intent["payload"].get("runtimeOperationId")):
                request["parameters"]["RUNTIME_OPERATION_ID"] = intent["payload"]["runtimeOperationId"]
    # State.submit_request is deliberately called by submit() before HTTP.
    # A caller can disable network delivery while provisioning a flow; the
    # pending request remains durable and reconcile can safely retry it.
    # submit() verifies the full original request and marker even on replay.
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
    if workflow.get("status") == "accepted":
        return workflow, list(workflow.get("stages", []))
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
    required = HEAVY_STAGES
    receipts = dict(workflow.get("stageReceipts", {}))
    consumer = state.get("execution_consumer", f"{execution_id}:{key}")
    active = (consumer is not None and consumer["payload"].get("status") == "active"
              and consumer["payload"].get("consumerId") == key
              and consumer["payload"].get("executionId") == execution_id
              and workflow.get("status") not in {"failed", "cancelled", "rejected"}
              and not workflow.get("consumerCancelled"))
    for item in stages:
        stage = item.get("name")
        receipt_id = stage_receipts.get(stage) if stage in required and active and observed.get("status") == "passed" else None
        if receipt_id:
            receipt = state.get("validation_receipt", receipt_id)
            if receipt and receipt["payload"].get("status") == "passed":
                item["status"] = "passed"; item["receipt"] = receipt["payload"]; receipts[stage] = receipt_id
    projected = {**workflow, "executionStatus": observed.get("status", "waiting"),
                 "stageReceipts": receipts, "stages": stages}
    if workflow.get("status") == "cancelled" or workflow.get("consumerCancelled"):
        projected.update(status="cancelled", executionStatus="cancelled")
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
    if action == "prepare-current-source":
        from ..current_source import prepare_current_source
        from ..patches import _identity as current_identity
        return prepare_current_source(state, repo_root, payload,
                                      trusted_driver_digest=_trusted_driver(repo_root, current_identity(payload)))
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
            marker = repo_root / ".jenkins" / "state" / "deployment" / "driver.json"
            if marker.exists():
                # A formal deployment must not recover by trusting an
                # arbitrary caller digest after the selected driver becomes
                # unavailable or changes.  Preserve the pending evidence and
                # let the caller reconcile once the same sealed binding is
                # restored.
                raise
            trusted = partial.get("driverDigest") or partial.get("stageImplementationDigest")
        return reconcile_patch(state, repo_root, payload, trusted_driver_digest=trusted)
    identity = _identity(payload) if action not in {"inventory-maintenance", "gc-maintenance", "reconcile-maintenance"} else payload.get("identity", {})
    key = f"{identity.get('sessionId','maintenance')}:{identity.get('requestId','maintenance')}:{identity.get('attemptId','') }"
    bound = state.get("workflow", key)
    if bound and bound["payload"].get("identity") != identity:
        raise JenkinsError("workflow_identity_mismatch", "workflow identity cannot change during recovery or execution")
    if action == "source-release":
        from ..source.closeout import release_accepted_claims
        # Issuer and accepted-source driver identities are independent. This
        # closeout can verify an older accepted flow without rewriting it.
        issuer = _trusted_driver(repo_root, {"stageImplementationDigest": identity["stageImplementationDigest"]})
        return release_accepted_claims(state, payload, identity=identity, issuer_driver_digest=issuer)
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
            if intent.get("checkoutBindingRef") is not None or intent.get("jointGroupId") is not None:
                from ..source.consumer_checkout import resolve_consumer_checkout
                with state.transaction() as connection:
                    checked_root, _ = resolve_consumer_checkout(
                        state, connection, repo_root, intent, intent, identity, sealed_ref)
                    if str(checked_root) != sealed_payload.get("repositoryRoot"):
                        raise JenkinsError("consumer_checkout_mismatch", "Flow differs from its original source authority")
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
        preparation_ref = payload.get("sourcePreparationRef")
        # Jenkins emits an empty string for an unset StringParameterDefinition.
        # It represents the same absent optional preparation as omitted/null.
        if preparation_ref == "":
            preparation_ref = None
        if preparation_ref is not None and not isinstance(preparation_ref, str):
            raise JenkinsError("source_preparation_mismatch", "Source preparation must be a receipt reference")
        preparation = None
        preparation_kind = (sealed_payload.get('sourceManifest') or sealed_payload.get('manifest', {})).get('preparationKind')
        if preparation_kind in {'current-source', 'already-applied-patch'}:
            if not isinstance(preparation_ref, str) or patch_refs:
                raise JenkinsError("source_preparation_missing", "Flow requires its typed coordinator preparation receipt")
            preparation = _require_flow_source_preparation(state, preparation_ref, identity=identity,
                                                          source_digest=sealed_ref, preparation_kind=preparation_kind)
            if (preparation["result"].get("coverageDigest") != identity["coverageDigest"]
                    or request and request["payload"].get("sourcePreparationRef") != preparation_ref):
                raise JenkinsError("source_preparation_mismatch", "Flow differs from its immutable source preparation request")
        elif preparation_ref:
            raise JenkinsError("source_preparation_mismatch", "A patch flow cannot adopt a current-source preparation receipt")
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
            registration_id = old.get("registrationExecutionId")
            if registration_id is None and not old.get("recipeRef"):
                registration_id = old.get("executionId")
            immutable = {"identity": old.get("identity"), "sourceDigest": old.get("sourceDigest"),
                         "coverageDigest": old.get("coverageDigest"), "driverDigest": old.get("driverDigest"),
                         "registrationExecutionId": registration_id, "sourceManifest": old.get("sourceManifest"),
                         "objectRoot": old.get("objectRoot"), "repoRoot": old.get("repoRoot"),
                         "sourcePreparationRef": None if old.get("sourcePreparationRef") == "" else old.get("sourcePreparationRef"),
                         "buildRoot": old.get("buildRoot"),
                         "patchOperationRefs": old.get("patchOperationRefs", []), "coverage": old.get("coverage")}
            requested = {"identity": identity, "sourceDigest": sealed["payload"].get("sourceDigest", sealed_ref),
                         "coverageDigest": identity.get("coverageDigest"), "driverDigest": driver_digest,
                         "registrationExecutionId": execution_id, "sourceManifest": sealed["payload"]["manifest"],
                         "objectRoot": sealed["payload"].get("objectRoot"), "repoRoot": sealed["payload"].get("repositoryRoot", str(repo_root)),
                         "sourcePreparationRef": preparation_ref, "buildRoot": str(selected_root.path),
                         "patchOperationRefs": patch_refs, "coverage": sealed_payload.get("coverage")}
            if any(immutable.get(k) != requested.get(k) for k in immutable):
                raise JenkinsError("workflow_identity_mismatch", "register-flow cannot replace an immutable contract")
            _require_registered_execution(state, repo_root, old, key)
            return response(old.get("status", "pending"), request_id=identity.get("requestId"), observed_generation=identity.get("generation"), executionId=old.get("executionId"))
        state.put("workflow", key, {"identity": identity, "repositoryId": sealed_payload.get("repositoryId") or identity.get("repositoryId"), "sourceDigest": sealed["payload"].get("sourceDigest", sealed_ref), "coverageDigest": identity.get("coverageDigest"), "driverDigest": identity.get("driverDigest") or identity.get("stageImplementationDigest"), "payloadDigest": digest(payload),
                                     "status": "pending", "executionId": execution_id, "registrationExecutionId": execution_id,
                                     "sourceManifest": sealed["payload"]["manifest"], "coverage": sealed["payload"]["coverage"],
                                     "objectRoot": sealed["payload"].get("objectRoot"), "repoRoot": sealed["payload"].get("repositoryRoot", str(repo_root)),
                                     "ownedPaths": list(preparation["result"]["ownedPaths"] if preparation else sealed_payload.get("ownedPaths", ())),
                                     "patchOperationRefs": patch_refs,
                                     "sourcePreparationRef": preparation_ref,
                                     "buildRoot": str(selected_root.path), "stages": [], "driverDigest": driver_digest})
        return response("pending", request_id=identity.get("requestId"), observed_generation=identity.get("generation"), executionId=execution_id)
    if action == "compose-flow":
        record = state.get("workflow", key)
        if record is None:
            raise JenkinsError("workflow_not_found", "register-flow must precede compose-flow")
        if payload.get("requestedTemplate") is not None or payload.get("changeSet") is not None:
            raise JenkinsError("caller_change_forbidden", "template and change set must come from source-owned records")
        if record["payload"].get("template") and record["payload"].get("stages"):
            flow = record["payload"]
            names = [item.get("name") for item in flow["stages"]]
            required = [stage for stage in names if stage in HEAVY_STAGES | (LIGHT_STAGES - {"acceptance"})]
            if (flow["template"] not in TEMPLATES or names != list(TEMPLATES[flow["template"]])
                    or flow.get("dagDigest") != digest(names) or flow.get("requiredStages") != required):
                raise JenkinsError("workflow_dag_corrupt", "Composed workflow differs from its immutable DAG")
            return response(record["payload"]["status"], request_id=identity.get("requestId"), observed_generation=identity.get("generation"), executionId=record["payload"].get("executionId"), template=record["payload"].get("template"), stages=[s["name"] for s in record["payload"].get("stages", [])])
        if record["payload"].get("status") in {"failed", "cancelled", "rejected", "unknown"}:
            raise JenkinsError("workflow_terminal", "A terminal or unknown workflow cannot be recomposed")
        from ..editor_product import TEMPLATES as PRODUCT_TEMPLATES
        if record["payload"].get("sourcePreparationRef"):
            preparation_kind = record['payload'].get('sourceManifest', {}).get('preparationKind')
            _require_flow_source_preparation(state, record["payload"]["sourcePreparationRef"], identity=identity,
                                             source_digest=identity["sourceInputDigest"],
                                             preparation_kind=preparation_kind)
            result = ({'kind': 'semantic', 'sourcePreparationKind': 'current-source'}
                      if preparation_kind == 'current-source' else _sealed_classification(record, state))
        else:
            result = _sealed_classification(record, state)
        if result["kind"] == "blocked":
            raise JenkinsError("classification_blocked", "change impact cannot be proven", details=result)
        coverage = record["payload"].get("coverage") or {}
        hinted = coverage.get("template") if isinstance(coverage, dict) else None
        if hinted in PRODUCT_TEMPLATES and record['payload'].get('sourceManifest', {}).get('preparationKind') != 'current-source':
            raise JenkinsError("source_preparation_missing", "Fixed editor product flows require typed current-source preparation")
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
        required_stages = [stage for stage in stages if stage in HEAVY_STAGES | (LIGHT_STAGES - {"acceptance"})]
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
        if stage_name == "source":
            from ..current_source import require_source_preparation
            journal = record["payload"].get("sourcePreparationRef")
            if payload.get("sourcePreparationRef", journal) != journal:
                raise JenkinsError("source_preparation_mismatch", "Source stage cannot replace its preparation receipt")
            operation = require_source_preparation(state, journal, identity=identity,
                                                   source_digest=identity["sourceInputDigest"])
            entries = {str(entry["path"]).casefold(): entry.get("sha256")
                       for entry in record["payload"]["sourceManifest"].get("entries", [])}
            if entries != operation["result"].get("currentHashes"):
                raise JenkinsError("source_preparation_mismatch", "Source preparation differs from its complete sealed closure")
            execution_id = digest({"flow": key, "stage": "source", "journal": journal})[:32]
            receipt = _light_receipt(state, repo_root, execution_id, "source", identity,
                {"status": "passed", "exitCode": 0, "tool": "current-source-preparation",
                 "commandDigest": digest(["source", "prepare-current-source"]), "operationId": journal},
                {"operationId": journal, "result": operation["result"]})
            stages[index].update(status="passed", receipt=receipt)
            receipts = dict(record["payload"].get("stageReceipts", {})); receipts["source"] = execution_id
            state.put("workflow", key, {**record["payload"], "status": "ready", "stages": stages,
                                         "stageReceipts": receipts}, expected_version=record["version"])
            return response("ready", request_id=identity.get("requestId"), receipt=receipt,
                            observed_generation=identity.get("generation"))
        sources: list[tuple[str, str]] = []
        manifest = record["payload"].get("sourceManifest") or {}
        root = record["payload"].get("objectRoot")
        if isinstance(root, str):
            for entry in manifest.get("entries", []):
                path = str(entry.get("path", ""))
                if Path(path).suffix.lower() not in REVIEW_SUFFIXES:
                    continue
                if not entry.get("objectDigest") or entry.get("status") != "present":
                    continue
                obj = Path(root).absolute() / "inputs" / "objects" / str(entry["objectDigest"])
                if obj.is_file():
                    sources.append((path, obj.read_text(encoding="utf-8")))
        if stage_name in HEAVY_STAGES | {"commit", "notify", "gc"}:
            raise JenkinsError("stage_requires_authority", "heavy and side-effect stages require their authoritative adapter", retryable=True)
        if stage_name in {"syntax", "format", "review"} and not sources:
            raise JenkinsError("sealed_source_missing", "syntax, format, and review require sealed source text")
        if stage_name == "patch":
            applied_preparation = manifest.get('preparationKind') == 'already-applied-patch'
            if applied_preparation:
                journal = record['payload'].get('sourcePreparationRef')
                if (payload.get('sourcePreparationRef', journal) != journal
                        or payload.get('patchOperationRef') or payload.get('patchJournal')
                        or record['payload'].get('patchOperationRefs')):
                    raise JenkinsError('source_preparation_mismatch', 'Applied patch receipt cannot adopt a source writer journal')
                operation = _require_flow_source_preparation(state, journal, identity=identity,
                    source_digest=identity['sourceInputDigest'], preparation_kind='already-applied-patch')
                entries = {str(e['path']).casefold(): e.get('sha256') for e in manifest.get('entries', [])}
                if entries != operation['result'].get('currentHashes'):
                    raise JenkinsError('patch_source_mismatch', 'Applied patch preparation differs from its complete sealed closure')
                operations = [operation]
                tool = 'coordinator-source-preparation'
                command = ['source', 'prepare-already-applied-patch']
            else:
                journal = payload.get("patchOperationRef") or payload.get("patchJournal") or payload.get("operationId")
                if journal is None and manifest.get("preparationKind") == "joint-input":
                    frozen_refs = record["payload"].get("patchOperationRefs") or []
                    journal = frozen_refs[0] if frozen_refs else None
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
                tool = 'source-parser'
                command = ['source', 'patch']
            execution_id = digest({"flow": key, "stage": stage_name, "journal": journal})[:32]
            receipt = _light_receipt(state, repo_root, execution_id, stage_name, identity,
                                     {"status": "passed", "exitCode": 0, "tool": tool,
                                      "commandDigest": digest(command), "operationId": journal},
                                     {"operations": [{"operationId": op["operationId"], "result": op["result"],
                                                       "authorization": op["payload"].get("authorization")}
                                                      for op in operations],
                                      **({"jointGroupId": (state.get("sealed_input", identity["sourceInputDigest"]) or {}).get("payload", {}).get("jointGroupId"),
                                          "authorInputs": manifest.get("authorInputs")}
                                         if manifest.get("preparationKind") == "joint-input" else {})})
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
            return response("accepted", request_id=identity.get("requestId"), observed_generation=identity.get("generation"), receipt=receipt)
        try:
            if stage_name == "syntax":
                checks = [syntax_receipt(text, edition=str(payload.get("edition", "2021")), source_digest=identity.get("sourceInputDigest")) for _, text in sources]
                raw_receipt = {"commandDigest": digest([c.get("commandDigest") for c in checks]), "edition": payload.get("edition", "2021")}
            elif stage_name == "format":
                checks = [format_receipt(text, edition=str(payload.get("edition", "2021")), source_digest=identity.get("sourceInputDigest")) for _, text in sources]
                raw_receipt = {"commandDigest": digest([c.get("commandDigest") for c in checks]), "edition": payload.get("edition", "2021")}
            elif stage_name == "review":
                checks = [review_receipt(path, text, edition=str(payload.get("edition", "2021")),
                                         source_digest=identity.get("sourceInputDigest"))
                          for path, text in sources]
                languages = sorted({c.get("tool") for c in checks})
                raw_receipt = {"commandDigest": digest([c.get("commandDigest") for c in checks]),
                               "edition": payload.get("edition", "2021"),
                               # A review covers several languages at once, so
                               # the recorded tool must be the set actually used
                               # rather than the Rust-only default.
                               "tool": ",".join(languages) or "review",
                               "languages": languages, "reviewedFiles": len(checks)}
            else:
                raw_receipt = {"status": "passed", "kind": stage_name,
                               "sourceDigest": identity.get("sourceInputDigest")}
            execution_id = digest({"flow": key, "stage": stage_name})[:32]
            receipt = _light_receipt(state, repo_root, execution_id, stage_name, identity,
                                     {"status": "passed", "exitCode": 0,
                                      "tool": raw_receipt.get("tool", "rustfmt"),
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
        # The filter runs over the stages this template actually declares, so
        # naming review here cannot stall a template that has no review stage.
        status = "ready" if all(item["status"] in {"passed", "reused"} for item in stages
                                if item["name"] in {"patch", "syntax", "format", "review"}) else "pending"
        state.put("workflow", key, {**record["payload"], "status": status, "stages": stages, "stageReceipts": receipts}, expected_version=record["version"])
        return response(status, request_id=identity.get("requestId"), observed_generation=identity.get("generation"), receipt=receipt)
    if action == "observe-flow":
        record = state.get("workflow", key)
        if record is None: raise JenkinsError("workflow_not_found", "workflow is not registered")
        projected, stages = _project_execution(state, repo_root, record["payload"], key=key)
        if projected != record["payload"]:
            state.put("workflow", key, projected, expected_version=record["version"])
        heavy = [x for x in stages if x.get("name") in HEAVY_STAGES]
        status = projected.get("executionStatus", projected.get("status", "waiting"))
        if (projected.get("status") not in {"failed", "cancelled", "rejected"}
                and not projected.get("consumerCancelled") and projected.get("executionStatus") == "passed"
                and heavy and all(x.get("status") in {"passed", "reused"} for x in heavy)):
            status = "ready"
        if projected.get("status") == "accepted":
            status = "accepted"
        elif projected.get("status") in {"failed", "cancelled", "rejected"}:
            status = projected["status"]
        return response(status, request_id=identity.get("requestId"), observed_generation=identity.get("generation"),
                        executionId=projected.get("executionId"), stages=stages)
    if action == "accept-flow":
        record = state.get("workflow", key)
        if record is None:
            raise JenkinsError("workflow_not_found", "workflow is not registered")
        authority = TrustedReceiptAuthority(state, repo_root=repo_root)
        flow = {**record["payload"], "flowId": key,
                "generation": identity.get("generation"),
                "sourceInputDigest": identity.get("sourceInputDigest")}
        receipt = authority.accept_flow(flow)
        return response("accepted", request_id=identity.get("requestId"),
                        observed_generation=identity.get("generation"), receipt=receipt)
    if action == "finalize-flow":
        record = state.get("workflow", key)
        if record is None or record["payload"].get("status") != "accepted":
            raise JenkinsError("flow_unaccepted", "finalization requires formal acceptance")
        flow = record["payload"]
        stages = [dict(item) for item in flow.get("stages", [])]
        notify_eligible = False
        for stage in stages:
            name = stage.get("name")
            if name not in {"commit", "notify", "gc"} or stage.get("status") == "passed":
                continue
            if name == "commit" and flow.get("gitCommitSha"):
                stage.update(status="passed", commitSha=flow["gitCommitSha"])
                continue
            if name == "commit" and payload.get("commitRequested") is not True:
                stage.update(status="skipped", reasonCode="action_not_requested")
                continue
            if name == "notify" and (payload.get("notifyRequested") is not True or flow.get("template") == "comments_only"):
                stage.update(status="skipped", reasonCode="action_not_requested")
                continue
            try:
                state.check_authorization(flow["repositoryId"], identity["sessionId"], name, flow.get("ownedPaths", []))
            except JenkinsError as error:
                if error.code not in {"action_not_authorized", "path_not_authorized"}:
                    raise
                stage.update(status="skipped", reasonCode="action_not_authorized")
                continue
            if name == "notify":
                from ..notifications import NotificationOutbox
                destination = payload.get("notifyDestination")
                if not isinstance(destination, str) or not destination:
                    raise JenkinsError("notification_destination_missing", "Requested notification requires its explicit destination")
                event_id = f"{identity['sessionId']}:{identity['requestId']}:{identity['attemptId']}:complete"
                notification = NotificationOutbox(state).enqueue(event_id, destination, {
                    "kind": "flow_complete", "sessionId": identity["sessionId"],
                    "requestId": identity["requestId"], "attemptId": identity["attemptId"],
                    "repositoryId": flow["repositoryId"], "gitCommitSha": flow.get("gitCommitSha"),
                    "template": flow.get("template"), "acceptanceReceipt": flow.get("acceptance")})
                delivered = notification.status == "delivered"
                notify_eligible = notification.status in {"pending", "retryable"}
                stage.update(status="passed" if delivered else "pending", eventId=event_id,
                             destinationId=destination, notificationStatus=notification.status,
                             reasonCode="delivery_confirmed" if delivered else "explicit_delivery_required")
            elif name == "gc" and isinstance(payload.get("cleanupScope"), dict):
                from ..maintenance import garbage_collect
                result = garbage_collect({"identity": identity, "buildRoot": flow["buildRoot"],
                    "operationId": digest({"flow": key, "stage": "gc"}),
                    "cleanupScope": payload["cleanupScope"], "ownedPaths": flow.get("ownedPaths", [])}, state, repo_root)
                stage.update(status="passed", receipt=result)
            else:
                stage.update(status="pending", reasonCode="bounded_scope_required")
        from ..source import PathClaim, release_paths
        if flow.get("gitCommitSha"):
            claims = [PathClaim.from_record(row) for row in state.list("path_claims")
                      if row["payload"].get("owner") == identity["attemptId"]]
            release_paths(state, identity["attemptId"], claims)
        current = state.get("workflow", key)
        state.put("workflow", key, {**current["payload"], "stages": stages,
                  "postAcceptanceStatus": "complete" if all(s.get("status") in {"passed", "reused", "skipped"} for s in stages) else "pending"}, expected_version=current["version"])
        return response("complete", request_id=identity.get("requestId"), executionId=flow["executionId"],
                        notifyEligible=notify_eligible)
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
        supplied_runtime = payload.get("runtimeOperationId") or identity.get("runtimeOperationId")
        supplied_driver = payload.get("driverDigest") or identity.get("driverDigest") or identity.get("stageImplementationDigest")
        from ..deployment.driver import active_driver_binding
        binding = active_driver_binding(repo_root)
        if str(supplied_runtime or "") != str(binding.get("runtimeOperationId") or binding.get("generation")):
            raise JenkinsError("runtime_binding_mismatch", "execution dispatch runtime operation differs from selected driver")
        if str(supplied_driver or "") != str(binding["digest"]):
            raise JenkinsError("driver_context_mismatch", "execution dispatch driver differs from selected driver")
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
                      if isinstance(e, dict) and str(e.get("path", "")).casefold() in {path.casefold() for path in owned}]
            if not paths:
                raise JenkinsError("owned_change_missing", "Cargo planning requires a proven owned change scope")
            options = {}
            if manifest.get("preparationKind") == "joint-input":
                sealed_record = state.get("sealed_input", sealed_ref)
                if not sealed_record:
                    raise JenkinsError("sealed_input_missing", "Union source is unavailable")
                union_coverage = sealed_record["payload"].get("coverage", {})
                options = {"features": union_coverage.get("features", ()),
                           "profile": union_coverage.get("profile")}
            plan = planner.plan(sealed_input_ref=sealed_ref, driver=driver,
                                changed_paths=paths, template=template, **options)
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
                                                                  if stage in HEAVY_STAGES}},
                                     "requiredOutputsByStage": {stage: recipe.get("requiredOutputsByStage", {}).get(stage, ["stdout.log", "stderr.log"])
                                                               for stage in record["payload"].get("requiredStages", ()) if stage in HEAVY_STAGES},
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
        workflow = state.get("workflow", key)
        if (workflow is None or workflow["payload"].get("executionId") != execution
                or (payload.get("recipeRef") is not None
                    and payload["recipeRef"] != record["payload"].get("recipeRef"))):
            raise JenkinsError("execution_identity_mismatch", "Finalization must retain the exact workflow execution")
        from .execution import ExecutionEngine
        # Reconcile only an existing execution. Finalization must never launch
        # pending work, and must retain the original host failure status.
        result = ExecutionEngine(state, repo_root=repo_root,
            build_root=workflow["payload"]["buildRoot"]).observe(execution)
        proof = result.get("terminalProof")
        if not isinstance(proof, dict) or proof.get("complete") is not True:
            raise JenkinsError("terminal_proof_missing", "execution has no complete native termination proof", retryable=True)
        record = state.get("execution", execution)
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
                                     execution, key, cancel_request=True)
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
                _build_root = _flow.get("buildRoot")
                if not _build_root:
                    # A recovered flow must retain the same approved build
                    # root it was registered with.  Never invent a
                    # repository-local ``.jenkins/builds`` fallback.
                    raise JenkinsError("build_root_missing", "Failed flow has no registered buildRoot")
                from ..resources.paths import canonical_build_root
                _failures_dir = canonical_build_root(_build_root).namespace() / "failures"
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

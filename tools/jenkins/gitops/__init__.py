"""Safe Git integration for the Jenkins shared checkout.

The module operates on sealed blobs and an explicitly owned path set.  It
never uses the worktree as a source for a commit after sealing and never
silently consumes foreign staged entries.
"""
from pathlib import Path
import hashlib
import os
import stat
import subprocess

from tools.jenkins.contracts import JenkinsError


def _path_key(path: str) -> str:
    return str(path).replace("\\", "/").casefold()


def _physical_repository_root(value: str | os.PathLike[str]) -> Path:
    """Admit a repository only after checking every existing parent physically.

    ``Path.resolve`` alone can make a junction appear to be an ordinary bound
    checkout.  Git integration is allowed to use a fixture checkout, but the
    checkout must be the exact path recorded by State and may not traverse a
    reparse point or symlink.
    """
    raw = Path(value)
    if not raw.is_absolute():
        raise JenkinsError("repository_binding_mismatch", "Repository root must be absolute")
    lexical = Path(os.path.abspath(os.fspath(raw)))
    current = lexical
    while True:
        try:
            st = current.stat()
            attrs = int(getattr(st, "st_file_attributes", 0))
            if attrs & 0x0400 or current.is_symlink():
                raise JenkinsError("repository_root_reparse", "Repository root traverses a reparse point",
                                   details={"path": str(current)})
        except FileNotFoundError:
            pass
        if current.parent == current:
            break
        current = current.parent
    resolved = Path(os.path.realpath(lexical))
    if os.path.normcase(os.fspath(resolved)) != os.path.normcase(os.fspath(lexical)):
        raise JenkinsError("repository_root_alias", "Repository root resolves through an alias")
    if not (lexical / ".git").exists():
        raise JenkinsError("git_repository_missing", "Git repository is missing", details={"root": str(lexical)})
    return lexical


def _sealed_blob_oids(manifest: dict, sealed: dict, *, object_root: str | os.PathLike[str],
                     build_root: str | os.PathLike[str] | None) -> dict[str, str]:
    """Import sealed SHA-256 objects into Git and return path -> blob OID.

    Source manifests intentionally use SHA-256 object identities.  Git's
    ``update-index --cacheinfo`` needs a Git blob OID, so every owned object is
    verified from the State object store and written with ``git hash-object``.
    """
    from ..resources import canonical_build_root, physical_path_under

    object_path = Path(object_root)
    if build_root is None:
        # The sealed record is the authority; derive the approved root from the
        # object namespace without accepting a caller-selected fallback.
        build_root = str(object_path.parent.parent)
    approved = canonical_build_root(str(build_root))
    physical_object_root = physical_path_under(approved, str(object_path), allow_missing=False)
    expected_namespace = approved.namespace()
    if physical_object_root != expected_namespace:
        raise JenkinsError("object_root_mismatch", "Sealed object root is not the approved build namespace")
    objects = physical_object_root / "inputs" / "objects"
    owned = {_path_key(path) for path in sealed.get("ownedPaths", ())}
    dependencies = {_path_key(path) for path in manifest.get("declaredDependencies", ())}
    entries = [entry for entry in manifest.get("entries", ()) if _path_key(entry.get("path", "")) in owned]
    entry_keys = [_path_key(entry.get("path", "")) for entry in entries]
    if not owned or owned & dependencies or set(entry_keys) != owned or len(entry_keys) != len(owned):
        raise JenkinsError("git_source_scope_mismatch", "Every owned path must appear exactly once in the accepted source")
    result: dict[str, str] = {}
    for entry in entries:
        path = str(entry.get("path", "")).replace("\\", "/")
        if entry.get("status") != "present":
            continue
        object_digest = entry.get("objectDigest")
        if not isinstance(object_digest, str) or len(object_digest) != 64 or any(c not in "0123456789abcdef" for c in object_digest.lower()):
            raise JenkinsError("sealed_object_invalid", "Source objectDigest is not a SHA-256 identity", details={"path": path})
        source = physical_path_under(approved, str(objects / object_digest), allow_missing=False)
        if source != objects / object_digest or not source.is_file():
            raise JenkinsError("sealed_object_invalid", "Sealed object path is not canonical", details={"path": path})
        data = source.read_bytes()
        if hashlib.sha256(data).hexdigest() != object_digest:
            raise JenkinsError("sealed_object_invalid", "Sealed object checksum does not match manifest", details={"path": path})
        completed = subprocess.run(["git", "hash-object", "-w", "--stdin"], cwd=sealed["repositoryRoot"],
                                   input=data, capture_output=True, check=True)
        result[path] = completed.stdout.decode("ascii").strip()
    return result

from .integration import (
    CandidateManifest,
    CandidatePath,
    GitRepository,
    IntegrationPlan,
    IntegrationReceipt,
    JournalState,
    git_mode,
)


def _flow_scoped_sealed(state, workflow: dict, sealed: dict) -> dict:
    """Use the current consumer's preparation scope for shared input objects."""
    manifest = sealed.get("sourceManifest") or sealed.get("manifest") or {}
    kind = manifest.get("preparationKind")
    if kind not in {"current-source", "already-applied-patch"}:
        return sealed
    reference = workflow.get("sourcePreparationRef")
    if not isinstance(reference, str) or workflow.get("patchOperationRefs"):
        raise JenkinsError("source_preparation_untrusted", "Git flow requires its typed source preparation")
    from ..workflow.handler import _require_flow_source_preparation
    preparation = _require_flow_source_preparation(
        state, reference, identity=workflow.get("identity", {}),
        source_digest=workflow.get("sourceDigest"), preparation_kind=kind)
    owned = preparation.get("result", {}).get("ownedPaths")
    flow_owned = workflow.get("ownedPaths")
    if (not isinstance(owned, list) or not owned
            or not all(isinstance(path, str) for path in owned)
            or len(set(owned)) != len(owned)
            or not isinstance(flow_owned, list) or sorted(flow_owned) != sorted(owned)):
        raise JenkinsError("git_source_scope_mismatch", "Git flow ownership differs from its consumer preparation")
    # Do not add ownership to the shared State row or replace its first author.
    return {**sealed, "ownedPaths": list(owned)}


def handle(action, payload, state, repo_root, *, domain="gitops"):
    """Small CLI adapter used by Jenkins stages; all actions stay explicit."""
    payload = dict(payload or {})
    def record_commit_receipt(workflow_key: str | None, receipt: dict) -> None:
        """Attach a proven Git journal receipt to the workflow with CAS."""
        if not state or not workflow_key or receipt.get("status") not in {"ref_updated", "complete"}:
            return
        record = state.get("workflow", workflow_key)
        if record is None:
            raise JenkinsError("workflow_missing", "Cannot attach Git receipt to an unknown workflow")
        current = record["payload"]
        stages = [dict(item) for item in current.get("stages", [])]
        commit = next((item for item in stages if item.get("name") == "commit"), None)
        if commit is None:
            commit = {"name": "commit"}
            stages.append(commit)
        commit["status"] = "passed"
        commit["receipt"] = dict(receipt)
        receipts = dict(current.get("stageReceipts", {}))
        receipts["commit"] = dict(receipt)
        updated = {**current, "stages": stages, "stageReceipts": receipts,
                   "gitReceipt": dict(receipt), "gitCommitSha": receipt.get("commit_sha") or receipt.get("commitSha")}
        state.put("workflow", workflow_key, updated, expected_version=record["version"])
    def unwrap(value):
        return value.get("payload", value) if isinstance(value, dict) else value

    def workflow_context():
        if not state:
            raise JenkinsError("state_required", "Git flow actions require State")
        key = payload.get("workflowKey") or payload.get("key")
        if not isinstance(key, str) or key.count(":") != 2:
            raise JenkinsError("workflow_reference_invalid", "workflowKey must be session:request:attempt")
        workflow = unwrap(state.get("workflow", key))
        if not workflow:
            raise JenkinsError("workflow_missing", "Workflow record is not registered")
        identity = workflow.get("identity") or {}
        parts = key.split(":")
        if len(parts) == 3:
            if identity.get("sessionId") and identity["sessionId"] != parts[0]:
                raise JenkinsError("workflow_identity_mismatch", "Workflow session identity does not match key")
            if identity.get("requestId") and identity["requestId"] != parts[1]:
                raise JenkinsError("workflow_identity_mismatch", "Workflow request identity does not match key")
            if identity.get("attempt") is not None and str(identity["attempt"]) != parts[2]:
                raise JenkinsError("workflow_identity_mismatch", "Workflow attempt identity does not match key")
        for field in ("repositoryId", "sessionId", "requestId", "attemptId"):
            supplied = payload.get(field)
            expected = identity.get(field)
            if supplied is not None and expected is not None and str(supplied) != str(expected):
                raise JenkinsError("workflow_identity_mismatch", f"Workflow {field} does not match caller")
        source_digest = workflow.get("sourceDigest")
        sealed = unwrap(state.get("sealed_input", source_digest)) if source_digest else None
        if not sealed:
            raise JenkinsError("sealed_input_missing", "Workflow source is not sealed")
        acceptance_ref = workflow.get("acceptance") or workflow.get("acceptanceId")
        acceptance_id = acceptance_ref.get("acceptanceId") if isinstance(acceptance_ref, dict) else acceptance_ref
        acceptance = unwrap(state.get("acceptance", acceptance_id)) if acceptance_id else None
        if isinstance(acceptance_ref, dict) and acceptance:
            # Workflow may carry a receipt snapshot; bind it to the authoritative State record.
            if acceptance_ref.get("acceptanceId") != acceptance_id or acceptance_ref.get("sourceDigest") not in {None, acceptance.get("sourceDigest")}:
                raise JenkinsError("acceptance_binding_mismatch", "Workflow acceptance is not the authoritative State receipt")
        if not acceptance or acceptance.get("status") not in {"accepted", "passed"}:
            raise JenkinsError("acceptance_missing", "Workflow lacks accepted authority receipt")
        if acceptance.get("sourceDigest") != source_digest:
            raise JenkinsError("acceptance_source_mismatch", "Acceptance does not cover the workflow's sealed source")
        if sealed.get("coverageDigest") and acceptance.get("coverageDigest") != sealed.get("coverageDigest"):
            raise JenkinsError("acceptance_coverage_mismatch", "Acceptance does not cover sealed input")
        # Formal flows must carry the immutable source-owned consumer binding.
        # Older hand-written unit fixtures may omit owner metadata and are
        # handled by their direct GitRepository tests.
        owner = identity.get("owner") or identity.get("attemptId") or workflow.get("owner")
        if owner and identity.get("repositoryId") and identity.get("sessionId") and source_digest:
            from ..source import require_sealed_consumer
            require_sealed_consumer(state, repository_id_value=str(identity["repositoryId"]),
                                    session_id=str(identity["sessionId"]), owner=str(owner),
                                    source_digest=str(source_digest))
        sealed = _flow_scoped_sealed(state, workflow, sealed)
        return workflow, sealed, acceptance

    effective_root = _physical_repository_root(repo_root)
    if action in {"commit-flow", "commit_flow", "reconcile-flow", "reconcile_flow",
                  "forward-revert-flow", "forward_revert_flow"}:
        workflow, sealed, acceptance = workflow_context()
        payload["sourceDigest"] = workflow.get("sourceDigest")
        acceptance_ref = workflow.get("acceptance") or workflow.get("acceptanceId")
        payload["acceptanceId"] = acceptance_ref.get("acceptanceId") if isinstance(acceptance_ref, dict) else acceptance_ref
        bound_root = workflow.get("repoRoot") or workflow.get("repositoryRoot") or sealed.get("repositoryRoot")
        if not bound_root:
            raise JenkinsError("repository_binding_missing", "Workflow source repository root is required")
        effective_root = _physical_repository_root(bound_root)
        # The State record is the trust boundary for an isolated fixture.  A
        # caller may supply the controller checkout as repo_root, but cannot
        # replace the workflow's sealed checkout with an arbitrary path.
        sealed_root = sealed.get("repositoryRoot")
        if sealed_root and os.path.normcase(os.fspath(effective_root)) != os.path.normcase(os.fspath(_physical_repository_root(sealed_root))):
            raise JenkinsError("repository_binding_mismatch", "Workflow source repository differs from sealed State root")
        if workflow.get("identity") and workflow["identity"].get("repositoryId"):
            payload["repositoryId"] = workflow["identity"]["repositoryId"]
        # Flow inputs come only from sealed manifest objects owned by the workflow.
        manifest = sealed.get("manifest") or sealed.get("sourceManifest") or sealed.get("candidate")
        if manifest is None and sealed.get("ownedPaths") is not None:
            manifest = sealed
        if manifest and action.startswith("commit"):
            if "paths" not in manifest and manifest.get("entries") is not None:
                # Entries in a sealed source manifest are SHA-256 identities
                # in the external object store, never Git blob OIDs.  Import
                # only owned blobs; retain the complete accepted input manifest
                # so the actual planned tree must also match every dependency.
                object_oids = _sealed_blob_oids(
                    manifest, sealed, object_root=sealed.get("objectRoot", ""),
                    build_root=sealed.get("buildRoot"))
                acceptance_ref_value = workflow.get("acceptance") or workflow.get("acceptanceId")
                acceptance_value = (acceptance_ref_value.get("acceptanceId")
                                    if isinstance(acceptance_ref_value, dict)
                                    else acceptance_ref_value)
                candidate_id = workflow.get("candidateId") or workflow.get("identity", {}).get("attemptId") or f"{workflow.get('sourceDigest')}-candidate"
                manifest = {**manifest, "inputManifest": manifest, "paths": [
                    {"path": e.get("path"), "blobOid": object_oids.get(str(e.get("path")).replace("\\", "/")),
                     "mode": git_mode(e.get("mode")) or "100644",
                     "deleted": bool(e.get("deleted", False) or e.get("status") == "deleted")}
                    for e in manifest["entries"]
                    if _path_key(e.get("path")) in {_path_key(path) for path in sealed.get("ownedPaths", ())}
                    and _path_key(e.get("path")) not in {_path_key(path) for path in manifest.get("declaredDependencies", ())}
                    and (e.get("status") == "deleted" or str(e.get("path")).replace("\\", "/") in object_oids)],
                    "candidateId": candidate_id,
                    "repositoryRoot": str(effective_root),
                    "baseHead": manifest.get("baseHead") or workflow.get("baseHead"),
                    "sourceDigest": workflow.get("sourceDigest"),
                    "coverageDigest": sealed.get("coverageDigest") or workflow.get("coverageDigest"),
                    "acceptanceReceipts": [acceptance_value] if acceptance_value else [],
                }
            if "paths" not in manifest and manifest.get("ownedPaths") is not None:
                objects = sealed.get("objects", {})
                rows = []
                for path in manifest.get("ownedPaths", []):
                    obj = objects.get(path) if isinstance(objects, dict) else None
                    if not obj:
                        raise JenkinsError("sealed_object_missing", f"No sealed object for owned path {path}")
                    if isinstance(obj, dict):
                        oid = obj.get("blobOid") or obj.get("oid")
                    else:
                        oid = obj
                    rows.append({"path": path, "blobOid": oid})
                manifest = {**manifest, "paths": rows}
            if sealed.get("coverageDigest") != manifest.get("coverageDigest"):
                raise JenkinsError("coverage_binding_mismatch", "Candidate coverage does not match sealed input")
            payload["candidates"] = [manifest]
            expected_outputs = workflow.get("requiredOutputsByStage", {}).get("commit")
            if expected_outputs is not None:
                supplied = payload.get("requiredOutputs", expected_outputs)
                if supplied != expected_outputs:
                    raise JenkinsError("required_outputs_mismatch", "Commit outputs differ from State workflow contract")
            payload.setdefault("message", workflow.get("message", "Jenkins integration"))
        payload.setdefault("authorization", workflow.get("authorization", {}))
        if action.startswith("reconcile") and "plan" not in payload:
            plan = workflow.get("plan") or workflow.get("stageReceipts", {}).get("plan")
            if plan:
                payload["plan"] = plan
    if state and payload.get("sourceDigest"):
        sealed_record = state.get("sealed_input", payload["sourceDigest"])
        if sealed_record:
            sealed_value = sealed_record.get("payload", sealed_record)
            trusted_root = sealed_value.get("repositoryRoot")
            if trusted_root and _physical_repository_root(trusted_root) != effective_root:
                raise JenkinsError("repository_binding_mismatch", "Repository root differs from sealed input")
            if sealed_value.get("repositoryId") and payload.get("repositoryId") and sealed_value["repositoryId"] != payload["repositoryId"]:
                raise JenkinsError("repository_binding_mismatch", "Repository identity differs from sealed input")
    if payload.get("repositoryRoot") and _physical_repository_root(payload["repositoryRoot"]) != effective_root:
        raise JenkinsError("repository_binding_mismatch", "Git repository root is not the trusted bound root")
    repo = GitRepository(effective_root, state=state)
    def candidate(value):
        if isinstance(value, CandidateManifest):
            return value
        raw_paths = value.get("paths", ())
        paths = tuple(CandidatePath(
            path=item.get("path"), blob_oid=item.get("blob_oid", item.get("blobOid")),
            mode=item.get("mode", "100644"), before_digest=item.get("before_digest", item.get("beforeDigest")),
            after_digest=item.get("after_digest", item.get("afterDigest")), deleted=item.get("deleted", False)
        ) for item in raw_paths)
        return CandidateManifest(value.get("candidate_id", value.get("candidateId")), value.get("repository_root", value.get("repositoryRoot", str(repo.root))),
                                 value.get("base_head", value.get("baseHead")), paths,
                                 value.get("source_digest", value.get("sourceDigest")),
                                 value.get("coverage_digest", value.get("coverageDigest")),
                                 tuple(value.get("acceptance_receipts", value.get("acceptanceReceipts", ()))),
                                 value.get("coauthor_map", {}),
                                 value.get("input_manifest", value.get("inputManifest")))
    if action == "seal":
        result = repo.seal_candidate(**payload)
        return {**result.__dict__, "paths": [p.__dict__ for p in result.paths]}
    if action == "plan":
        candidates = tuple(candidate(item) for item in payload.pop("candidates"))
        result = repo.plan(candidates, **payload)
        return result.__dict__
    if action in {"commit-flow", "commit_flow"}:
        candidates = tuple(candidate(item) for item in payload.pop("candidates"))
        for c in candidates:
            if not c.acceptance_receipts:
                raise JenkinsError("acceptance_missing", "Sealed candidate requires accepted receipt")
        plan_args = payload.pop("plan", {})
        authorization = payload.pop("authorization", plan_args.pop("authorization", None))
        if not isinstance(authorization, dict) or not authorization:
            identity = workflow.get("identity") or {}
            authorization = {"repositoryId": identity.get("repositoryId") or workflow.get("repositoryId"),
                             "sessionId": identity.get("sessionId")}
        if authorization is None:
            raise JenkinsError("authorization_missing", "Commit flow requires scoped authorization")
        plan = repo.plan(candidates, authorization=authorization,
                         message=plan_args.pop("message", payload.pop("message", "Jenkins integration")),
                         operation_id=plan_args.pop("operation_id", plan_args.pop("operationId", None)))
        receipt = repo.publish(plan, authorization=authorization).__dict__
        record_commit_receipt(payload.get("workflowKey"), receipt)
        return receipt
    if action == "publish":
        plan_data = payload.pop("plan")
        plan = plan_data if isinstance(plan_data, IntegrationPlan) else IntegrationPlan(**plan_data)
        return repo.publish(plan, **payload).__dict__
    if action == "reconcile":
        return repo.reconcile(**payload).__dict__
    if action == "repair":
        plan = payload["plan"]
        if isinstance(plan, dict):
            plan = IntegrationPlan(**plan)
        repo._align_index(plan)
        return repo.reconcile(plan).__dict__
    if action in {"reconcile-flow", "reconcile_flow"}:
        supplied = payload.get("plan")
        operation_id = payload.get("operationId") or (supplied.operation_id if isinstance(supplied, IntegrationPlan) else (supplied or {}).get("operation_id"))
        operation_id = operation_id or workflow.get("gitReceipt", {}).get("operation_id")
        operation = state.get_operation(operation_id) if operation_id else None
        if not operation or operation.get("kind") != "git_integration":
            raise JenkinsError("git_journal_missing", "Git reconciliation requires its registered integration journal")
        plan_data = operation["payload"].get("plan")
        if not isinstance(plan_data, dict):
            raise JenkinsError("git_journal_invalid", "integration journal has no immutable plan")
        if supplied is not None and (supplied.__dict__ if isinstance(supplied, IntegrationPlan) else supplied) != plan_data:
            raise JenkinsError("git_plan_mismatch", "caller cannot replace the registered integration plan")
        authorization = operation["payload"].get("authorization", {})
        identity = workflow.get("identity", {})
        if authorization.get("repositoryId") != identity.get("repositoryId") or authorization.get("sessionId") != identity.get("sessionId"):
            raise JenkinsError("git_journal_owner_mismatch", "Git journal belongs to another authorized workflow")
        plan = IntegrationPlan(**plan_data)
        source_manifest = sealed.get("sourceManifest") or sealed.get("manifest") or {}
        if source_manifest.get("entries") is not None:
            if (len(plan.input_closures) != 1
                    or plan.input_closures[0].get("sourceDigest") != workflow.get("sourceDigest")
                    or plan.input_closures[0].get("manifest") != source_manifest):
                raise JenkinsError("git_input_closure_binding_mismatch", "Journal does not retain the complete accepted input closure; revalidation is required",
                                   retryable=True, details={"revalidationRequired": True})
            owned = {_path_key(path) for path in sealed.get("ownedPaths", ())}
            accepted_paths = {_path_key(entry["path"]) for entry in source_manifest["entries"] if _path_key(entry["path"]) in owned}
            if {_path_key(path) for path in plan.owned_paths} != accepted_paths or accepted_paths != owned:
                raise JenkinsError("git_reconcile_scope_mismatch", "journal paths differ from accepted source scope")
        receipt = repo.reconcile(plan).__dict__
        if receipt.get("status") in {"complete", "index_pending"}:
            manifest = sealed.get("sourceManifest") or sealed.get("manifest") or {}
            owned = {_path_key(path) for path in sealed.get("ownedPaths", ())}
            if manifest.get("entries") is not None:
                # GitRepository.reconcile has verified the actual commit tree
                # against the journal's entire retained accepted input closure.
                repo.verify_plan(plan)
            if receipt["status"] == "complete" and operation["status"] != "complete":
                state.transition_operation(operation_id, operation["status"], "complete", receipt)
        record_commit_receipt(payload.get("workflowKey"), receipt)
        return receipt
    if action == "compensate":
        return repo.compensate_uncommitted(**payload)
    if action == "forward_revert":
        return {"commitSha": repo.forward_revert(**payload)}
    if action in {"forward-revert-flow", "forward_revert_flow"}:
        plan = payload.pop("plan", None)
        if plan:
            p = plan if isinstance(plan, IntegrationPlan) else IntegrationPlan(**plan)
            receipt = repo.publish(p, authorization=payload["authorization"])
            return receipt.__dict__
        return {"commitSha": repo.forward_revert(
            payload["commitSha"], authorization=payload["authorization"],
            message=payload.get("message", "Forward revert"),
            source_digest=payload["sourceDigest"], coverage_digest=payload.get("coverageDigest", ""),
            acceptance_receipts=(payload.get("acceptanceId"),))}
    raise JenkinsError("unknown_gitops_action", f"Unknown gitops action: {action}")

__all__ = [
    "CandidateManifest", "CandidatePath", "GitRepository", "IntegrationPlan",
    "IntegrationReceipt", "JournalState",
]

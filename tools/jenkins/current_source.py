"""Coordinator intake of exact, already-applied checkout bytes.

Registration reads the selected files and records an immutable intent. Only
the coordinator preparation operation writes its content-addressed input store;
neither operation applies, compensates, or claims files in the shared checkout.
"""
from __future__ import annotations

import re
import subprocess
from pathlib import Path
from typing import Mapping

from .contracts import JenkinsError, digest, require_digest, response
from .patches import _build_root, _ensure_patch_metadata, _head, _identity, _lock, _runtime_binding
from .resources import canonical_build_root
from .source import (PathClaim, SealedManifest, _before_hash, _thaw,
                     canonical_path, register_sealed_consumer, repository_id,
                     seal_candidate)


PRODUCT_TEMPLATES = frozenset({"editor_product_development", "editor_product_shipping"})
SOURCE_ROOT_CAPABILITY = 'managed-worktree-reference-v1'
_WORKTREE_ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,159}\Z")


def validate_coverage(value: Mapping) -> dict:
    if (not isinstance(value, dict) or value.get("template") not in PRODUCT_TEMPLATES
            or set(value) - {"template", "requiredTests", "noDefaultFeatures"}
            or value.get("noDefaultFeatures", True) is not True):
        raise JenkinsError("current_source_coverage_invalid", "Current source requires a fixed editor product coverage template")
    tests = value.get("requiredTests")
    if (not isinstance(tests, list) or not tests
            or any(not isinstance(test, str) or not test or "\n" in test or "\r" in test for test in tests)
            or len(set(tests)) != len(tests)):
        raise JenkinsError("current_source_tests_missing", "Product coverage requires unique, exact Cargo test IDs")
    return {"template": value["template"], "requiredTests": sorted(tests), "noDefaultFeatures": True}


def _paths(root: Path, values, name: str) -> list[str]:
    if not isinstance(values, list) or any(not isinstance(value, str) for value in values):
        raise JenkinsError("current_source_scope_invalid", f"{name} must be an explicit file list")
    result = [canonical_path(root, value) for value in values]
    if len(set(result)) != len(result):
        raise JenkinsError("current_source_scope_invalid", f"{name} contains duplicate paths")
    return sorted(result)


def _hashes(root: Path, values) -> dict:
    if not isinstance(values, dict) or not values:
        raise JenkinsError("current_source_hashes_missing", "Provide the complete explicit current file closure")
    result = {}
    for path, checksum in values.items():
        if not isinstance(path, str):
            raise JenkinsError("current_source_scope_invalid", "Current file paths must be strings")
        relative = canonical_path(root, path)
        if relative in result:
            raise JenkinsError("current_source_scope_invalid", "Current file paths collide")
        if checksum is not None:
            require_digest(checksum, "currentHash")
        result[relative] = checksum
    return dict(sorted(result.items()))


def _git_selection(root: Path, paths: list[str]) -> tuple[set[str], set[str]]:
    selected = set(paths)
    try:
        def names(*flags):
            # A complete engine closure exceeds Windows' command-line limit.
            # Inventory is metadata only; intersect it with the explicit scope.
            result = subprocess.run(["git", "ls-files", "-z", *flags],
                                    cwd=root, capture_output=True, check=True)
            inventory = {value.decode("utf-8").replace("\\", "/").casefold()
                         for value in result.stdout.split(b"\0") if value}
            return {canonical_path(root, value) for value in inventory & selected}
        return names("--cached"), names("--others", "--exclude-standard")
    except (OSError, subprocess.CalledProcessError, UnicodeError) as error:
        raise JenkinsError("current_source_git_unavailable", "Cannot verify selected tracked and untracked source files") from error


def _source_root(root: Path, value, worktree_id=None) -> Path:
    """A noncanonical byte source must name one physical managed worktree."""
    if value in (None, "") and worktree_id is None:
        return root
    # An explicit physical canonical checkout retains the historical intent.
    # Only a separate managed byte source needs the paired worktree identity.
    if (worktree_id is None and isinstance(value, str) and value
            and Path(value).is_absolute() and Path(value).absolute() == root):
        from .deployment.driver import _plain
        _plain(Path(value))
        if root.resolve() != root:
            raise JenkinsError("current_source_repository_alias", "Use the physical canonical checkout")
        return root
    if (not isinstance(value, str) or not value or not isinstance(worktree_id, str)
            or not _WORKTREE_ID.fullmatch(worktree_id) or worktree_id.endswith(".")):
        raise JenkinsError("current_source_worktree_required", "Pair sourceRoot with an explicit managed worktreeId")
    from .deployment.driver import _plain
    source = Path(value)
    if not source.is_absolute():
        raise JenkinsError("current_source_root_invalid", "Managed sourceRoot must be an absolute physical directory")
    source = source.absolute()
    _plain(source)
    _plain(source / ".git")
    if (source != root / ".jenkins/worktrees" / worktree_id
            or source.resolve() != source or not source.is_dir()):
        raise JenkinsError("current_source_root_invalid", "Source must be the exact direct managed worktree child")
    def git_path(flag):
        completed = subprocess.run(["git", "-C", str(source), "rev-parse",
                                    "--path-format=absolute", flag],
                                   capture_output=True, text=True, check=False)
        if completed.returncode:
            raise JenkinsError("current_source_root_invalid", "Managed source is not a Git worktree")
        path = Path(completed.stdout.strip()).absolute()
        _plain(path)
        return path
    if git_path("--git-common-dir") != root / ".git" or git_path("--show-toplevel") != source:
        raise JenkinsError("current_source_root_mismatch", "Managed source has a different physical Git identity")
    return source


def _managed_worktree(state, root, intent, reader, *, connection=None):
    """Bind a reader to the ready row, its completed creator and active writer."""
    source = _source_root(root, intent.get("sourceRoot"), intent.get("worktreeId"))
    identity, reference = intent["identity"], intent["worktreeId"]
    row = state.get("management_worktree", reference, connection=connection)
    value = row["payload"] if row else {}
    def require(condition, message):
        if not condition:
            raise JenkinsError("current_source_worktree_unproven", message)
    require(row is not None and value.get("status") == "ready"
            and value.get("path") == str(source)
            and value.get("repositoryId") == identity["repositoryId"]
            and value.get("driverDigest") == identity["stageImplementationDigest"]
            and isinstance(intent.get("runtimeOperationId"), str) and bool(intent["runtimeOperationId"])
            and value.get("runtimeOperationId") == intent["runtimeOperationId"]
            and value.get("baseHead") == intent["baseHead"] == _head(source),
            "Managed source lacks its exact ready row, runtime or Git base")
    owned = _paths(root, value.get("ownedPaths"), "writerOwnedPaths")
    require(owned == intent["ownedPaths"] and bool(owned)
            and isinstance(value.get("sessionId"), str)
            and value["sessionId"] != identity["sessionId"],
            "The validation reader must retain a separate narrow writer identity")
    writer = state.check_authorization(identity["repositoryId"], value["sessionId"],
                                       "implementation", owned, connection=connection)
    validation = state.check_authorization(identity["repositoryId"], value["sessionId"],
                                           "validation", owned, connection=connection)
    from .source.external_inputs import authorization_paths
    closure = sorted(list(intent["currentHashes"]) + authorization_paths(intent.get("externalInputs", [])))
    require(writer["actions"] == ["implementation", "validation"]
            and sorted(writer["paths"]) == owned
            and writer["snapshotDigest"] == validation["snapshotDigest"] == value.get("writerAuthorizationSnapshot")
            and writer["principal"] == value.get("writerPrincipal") == reader["principal"]
            and reader["actions"] == ["validation"] and sorted(reader["paths"]) == closure,
            "Managed source writer or explicit validation reader authorization changed")
    names = ("repositoryId", "sessionId", "requestId", "operationId")
    require(all(isinstance(value.get(name), str) and value[name] for name in names),
            "Managed source has no complete creation identity")
    key = digest({name: value[name] for name in names})
    operation = state.get("management_operation", key, connection=connection)
    created = operation["payload"] if operation else {}
    original, result = created.get("payload", {}), created.get("result", {})
    require(operation is not None and value.get("creationOperationKey") == key
            and created.get("phase") == "complete" and created.get("action") == "create-worktree"
            and all(created.get(name) == value[name] for name in names[1:])
            and all(original.get(name) == value[name] for name in names)
            and original.get("action") == "create-worktree"
            and original.get("driverDigest") == value["driverDigest"]
            and original.get("runtimeOperationId") == value["runtimeOperationId"]
            and isinstance(original.get("name"), str)
            and value["sessionId"] + "-" + original["name"] == reference
            and original.get("baseHead") == value["baseHead"]
            and _paths(root, original.get("ownedPaths"), "creatorOwnedPaths") == owned
            and created.get("inputDigest") == digest({k: v for k, v in original.items() if k != "observationOnly"})
            and result.get("status") == "complete" and result.get("worktreeId") == reference
            and result.get("operationKey") == key
            and all(result.get(name) == value.get(name) for name in
                    (*names, "path", "baseHead", "branch", "ownedPaths", "driverDigest", "runtimeOperationId",
                     "creationOperationKey", "writerPrincipal", "writerAuthorizationSnapshot")),
            "Managed source has no exact completed creation operation")
    from .managed_worktree import assert_no_writer
    assert_no_writer(state, reference, connection=connection)
    proof = {"worktreeId": reference, "recordVersion": row["version"], "recordDigest": digest(value),
            "creationOperationKey": key, "creationOperationVersion": operation["version"],
            "creationOperationDigest": digest(created), "writerSessionId": value["sessionId"],
            "writerAuthorizationSnapshot": writer["snapshotDigest"], "writerPrincipal": writer["principal"]}
    if intent.get("worktreePreparationRef"):
        from .managed_worktree import reader_binding
        proof["sourcePreparation"] = reader_binding(state, root, intent, reader, connection=connection)
    return proof


def _verify_current(root: Path, intent: Mapping) -> None:
    source = _source_root(root, intent.get("sourceRoot"), intent.get("worktreeId"))
    if _head(source) != intent["baseHead"]:
        raise JenkinsError("current_source_base_changed", "Current checkout HEAD differs from its registered base")
    expected = intent["currentHashes"]
    tracked, untracked = _git_selection(source, list(expected))
    selected_untracked = {path for path, checksum in expected.items() if checksum is not None and path not in tracked}
    if selected_untracked != set(intent["untrackedPaths"]) or not selected_untracked <= untracked:
        raise JenkinsError("current_source_untracked_mismatch", "Explicit untracked files differ from Git's selected current closure")
    for path, checksum in expected.items():
        target = source.joinpath(*path.split("/"))
        from .deployment.driver import _plain
        _plain(target)
        canonical_path(root, path)
        if source != root:
            canonical_path(source, path)
        if checksum is None:
            if path not in tracked or target.exists():
                raise JenkinsError("current_source_deletion_invalid", "Null hashes may name only absent tracked files", details={"path": path})
        elif not target.is_file():
            raise JenkinsError("current_source_file_missing", "Selected source must be a present ordinary file", details={"path": path})
        if _before_hash(source, path) != checksum:
            raise JenkinsError("current_source_hash_mismatch", "Current source differs from its registered hash", details={"path": path})


def _authorize(state, intent, *, connection=None) -> dict:
    identity = intent["identity"]
    from .source.external_inputs import authorization_paths
    return state.check_authorization(identity["repositoryId"], identity["sessionId"],
                                     "validation", list(intent["currentHashes"]) + authorization_paths(intent.get("externalInputs", [])),
                                     connection=connection)


def register_current_source(state, repo_root: Path, payload: dict, *, trusted_driver_digest: str) -> dict:
    root = repo_root.absolute()
    from .deployment.driver import _plain
    _plain(root)
    if root.resolve() != root:
        raise JenkinsError("current_source_repository_alias", "Current source requires the physical canonical checkout")
    identity = _identity(payload)
    if identity["repositoryId"] != repository_id(root):
        raise JenkinsError("current_source_repository_mismatch", "Current source repository identity differs from its physical checkout")
    if identity["stageImplementationDigest"] != trusted_driver_digest:
        raise JenkinsError("driver_context_mismatch", "Current source intake must use its selected sealed driver")
    hashes = _hashes(root, payload.get("currentHashes"))
    source = _source_root(root, payload.get("sourceRoot"), payload.get("worktreeId"))
    owned = _paths(root, payload.get("ownedPaths"), "ownedPaths")
    from .product_inputs import compiler_controls, validate_hash_paths
    controls = compiler_controls(payload.get("compilerControlInputs"), hashes, owned, managed=source != root)
    validate_hash_paths(hashes, controls)
    untracked = _paths(root, payload.get("untrackedPaths"), "untrackedPaths")
    if not owned or not set(owned) <= set(hashes) or not set(untracked) <= set(hashes):
        raise JenkinsError("current_source_scope_invalid", "Owned and untracked paths must be explicit subsets of the current closure")
    base = payload.get("baseHead")
    if not isinstance(base, str) or len(base) not in {40, 64} or any(c not in "0123456789abcdef" for c in base):
        raise JenkinsError("current_source_base_missing", "Current source requires an immutable Git base revision")
    from .source.external_inputs import normalize_current
    prepared_ref = payload.get("worktreePreparationRef")
    if prepared_ref is not None:
        if source == root or controls is None:
            raise JenkinsError("managed_source_preparation_mismatch", "Prepared product inputs require the managed source pair")
        from .managed_worktree import preparation
        prepared = preparation(state, prepared_ref, payload["worktreeId"])["payload"]
        external = prepared["externalInputs"]
        if (payload.get("externalInputs") != external
                or prepared["objectRoot"] != str(canonical_build_root(_build_root(root, payload.get("buildRoot"))).namespace())):
            raise JenkinsError("managed_source_preparation_mismatch", "Flow must retain the frozen external closure and object domain")
    else:
        external = normalize_current(root, payload.get("externalInputs", []))
    supplied = {"schemaVersion": 1, "kind": "current-source", "identity": identity,
                "repositoryRoot": str(root), "baseHead": base, "currentHashes": hashes,
                "ownedPaths": owned, "untrackedPaths": untracked,
                "coverage": validate_coverage(payload.get("coverage")),
                "buildRoot": _build_root(root, payload.get("buildRoot")),
                "runtimeOperationId": _runtime_binding(root, identity, payload),
                "externalInputs": external}
    # Recorded only when a worktree actually supplies the bytes: an ordinary
    # checkout keeps the exact intent shape it has always had, so historical
    # currentSourceRef digests stay verifiable.
    if source != root:
        supplied.update(sourceRoot=str(source), worktreeId=payload["worktreeId"])
        if controls is not None:
            supplied["compilerControlInputs"] = controls
        if prepared_ref is not None:
            supplied.update(worktreePreparationRef=prepared_ref,
                            patchOperationKeys=payload.get("patchOperationKeys", []))
    slot = digest([identity[key] for key in ("repositoryId", "sessionId", "requestId")])
    with _lock(root):
        authorization = _authorize(state, supplied)
        if source != root:
            supplied["managedWorktree"] = _managed_worktree(state, root, supplied, authorization)
        intent = {**supplied, "authorizationSnapshot": authorization["snapshotDigest"]}
        reference = digest(intent)
        if source != root and state.get("managed_worktree_reader_retirement", reference):
            raise JenkinsError("current_source_reader_retired", "Reuse the worktree with a fresh reader request")
        previous = state.get("current_source_request", slot)
        if previous:
            if previous["payload"].get("currentSourceRef") != reference:
                raise JenkinsError("current_source_request_conflict", "This request already owns different current-source input")
            return response("registered", currentSourceRef=reference, reused=True)
        if state.get_request(identity["repositoryId"], identity["sessionId"], identity["requestId"]):
            raise JenkinsError("current_source_request_conflict", "An existing request cannot be adopted by current-source intake")
        if state.get("patch_request", slot):
            raise JenkinsError("current_source_request_conflict", "An existing patch intake cannot be adopted by current-source intake")
        _verify_current(root, intent)
        _verify_external(state, intent)
        with state.transaction() as connection:
            actual = _authorize(state, intent, connection=connection)
            if actual["snapshotDigest"] != intent["authorizationSnapshot"]:
                raise JenkinsError("current_source_authorization_changed", "Reader authorization changed before intake commit")
            if source != root and _managed_worktree(state, root, intent, actual, connection=connection) != intent["managedWorktree"]:
                raise JenkinsError("current_source_worktree_changed", "Managed source changed before intake commit")
            if (state.get("current_source_request", slot, connection=connection)
                    or state.get_request(identity["repositoryId"], identity["sessionId"], identity["requestId"], connection=connection)
                    or state.get("patch_request", slot, connection=connection)):
                raise JenkinsError("current_source_request_conflict", "A competing request was admitted before intake commit")
            state.put("current_source_intent", reference, intent, expected_version=0, connection=connection)
            state.put("current_source_request", slot, {"currentSourceRef": reference}, expected_version=0, connection=connection)
        return response("registered", currentSourceRef=reference, reused=False)


def _load_intent(state, root: Path, payload: dict) -> tuple[str, dict]:
    reference = require_digest(payload.get("currentSourceRef"), "currentSourceRef")
    record = state.get("current_source_intent", reference)
    if not record or digest(record["payload"]) != reference:
        raise JenkinsError("current_source_intent_missing", "Immutable current-source intake is missing or corrupt")
    intent = record["payload"]
    if intent.get("worktreeId") and state.get("managed_worktree_reader_retirement", reference):
        raise JenkinsError("current_source_reader_retired", "A retired source reference cannot start or resume work")
    if intent["identity"] != _identity(payload) or intent["repositoryRoot"] != str(root.absolute()):
        raise JenkinsError("current_source_identity_mismatch", "Current-source intake belongs to another request or checkout")
    if payload.get("buildRoot") and _build_root(root, payload["buildRoot"]) != intent["buildRoot"]:
        raise JenkinsError("current_source_build_root_mismatch", "Current-source preparation must retain its build root")
    runtime = _runtime_binding(root, intent["identity"], payload)
    if runtime != intent.get("runtimeOperationId"):
        raise JenkinsError("current_source_runtime_mismatch", "Current-source control must retain its registered runtime operation")
    authorization = _authorize(state, intent)
    if authorization["snapshotDigest"] != intent["authorizationSnapshot"]:
        raise JenkinsError("current_source_authorization_changed", "Current-source authorization differs from its registered scope")
    if intent.get("sourceRoot") or intent.get("worktreeId") is not None:
        if _managed_worktree(state, root, intent, authorization) != intent.get("managedWorktree"):
            raise JenkinsError("current_source_worktree_changed", "Registered managed source binding changed")
    return reference, intent


def _verify_external(state, intent):
    if intent.get("worktreePreparationRef"):
        from .managed_worktree import prepared_external
        return prepared_external(state, intent)
    from .source.external_inputs import verify_current
    verify_current(intent["externalInputs"])
    return None


def _request(intent: dict, prepared: dict) -> dict:
    return {**prepared["identity"], "sealedInputRef": prepared["sealedInputRef"],
            "sourcePreparationRef": prepared["sourcePreparationRef"], "currentSourceRef": prepared["currentSourceRef"],
            "ownedPaths": intent["ownedPaths"], "buildRoot": intent["buildRoot"],
            "driverDigest": intent["identity"]["stageImplementationDigest"],
            "authorizationSnapshot": intent["authorizationSnapshot"]}


def require_source_preparation(state, reference: str, *, identity: Mapping, source_digest: str) -> dict:
    operation = state.get_operation(reference)
    bound_identity = operation["payload"].get("identity", {}) if operation else {}
    if (not operation or operation["kind"] != "source_preparation" or operation["status"] != "complete"
            or not bound_identity or any(identity.get(key) != value for key, value in bound_identity.items())
            or not operation.get("result") or operation["result"].get("sourceDigest") != source_digest):
        raise JenkinsError("source_preparation_untrusted", "Source stage requires its exact completed coordinator preparation")
    return operation


def prepare_current_source(state, repo_root: Path, payload: dict, *, trusted_driver_digest: str) -> dict:
    root = repo_root.absolute()
    reference, intent = _load_intent(state, root, payload)
    identity = intent["identity"]
    if identity["stageImplementationDigest"] != trusted_driver_digest:
        raise JenkinsError("driver_context_mismatch", "Current-source preparation differs from its registered sealed driver")
    with _lock(root):
        previous = state.get("current_source_preparation", reference)
        if previous and previous["payload"].get("status") == "prepared":
            result = previous["payload"]["result"]
            require_source_preparation(state, result["sourcePreparationRef"], identity=identity,
                                       source_digest=result["sealedInputRef"])
            sealed_record = state.get("sealed_input", result["sealedInputRef"])
            if not sealed_record or digest(sealed_record["payload"]["sourceManifest"]) != result["sealedInputRef"]:
                raise JenkinsError("current_source_sealed_missing", "Prepared source manifest is unavailable or corrupt")
            objects = Path(sealed_record["payload"]["objectRoot"]) / "inputs/objects"
            from .contracts import file_digest
            for entry in sealed_record["payload"]["sourceManifest"]["entries"]:
                if entry["status"] == "present":
                    obj = objects / entry["objectDigest"]
                    canonical_path(objects, entry["objectDigest"])
                    if not obj.is_file() or file_digest(obj) != entry["sha256"]:
                        raise JenkinsError("current_source_object_corrupt", "Prepared source object differs from its immutable manifest")
            from .source.external_inputs import verify_objects
            verify_objects(sealed_record["payload"]["sourceManifest"], Path(sealed_record["payload"]["objectRoot"]))
            state.submit_request(_request(intent, result))
            return result
        operation_id = "current-source-" + reference
        operation = state.put_operation("source_preparation", operation_id, intent)
        # A partial preparation may retry these reads and CAS writes; it never
        # compensates or modifies the canonical source files.
        _verify_current(root, intent)
        from .source.external_inputs import seal as seal_external
        frozen_external = _verify_external(state, intent)
        claims = [PathClaim(path, identity["attemptId"] if path in intent["ownedPaths"] else "dependency",
                            0, checksum, 0, identity["repositoryId"])
                  for path, checksum in intent["currentHashes"].items()]
        object_root = canonical_build_root(intent["buildRoot"]).namespace()
        # Identity comes from the canonical checkout; bytes come from the
        # registered source root, which is the same repository by construction.
        manifest = seal_candidate(root, claims, object_root=object_root, base_head=intent["baseHead"],
                                  source_root=intent.get("sourceRoot"))
        plain = _thaw(manifest.payload)
        plain.update(preparationKind="current-source", validationCoverageDigest=digest(intent["coverage"]),
                     external=(frozen_external if frozen_external is not None else
                               seal_external(intent["externalInputs"], object_root)))
        if intent.get("worktreePreparationRef"):
            plain.update(worktreePreparationRef=intent["worktreePreparationRef"],
                         compilerControlInputs=intent["compilerControlInputs"])
        manifest = SealedManifest(plain, digest(plain))
        _verify_current(root, intent)
        _verify_external(state, intent)
        coverage = {**intent["coverage"], "sourceDigest": manifest.source_digest}
        coverage["coverageDigest"] = digest(coverage)
        sealed = {"status": "sealed", "repositoryRoot": str(root), "repositoryId": identity["repositoryId"],
                  "sourceDigest": manifest.source_digest, "sourceManifest": plain, "manifest": plain,
                  "coverage": coverage, "coverageDigest": coverage["coverageDigest"],
                  "objectRoot": str(object_root), "buildRoot": intent["buildRoot"],
                  "owner": identity["attemptId"], "sessionId": identity["sessionId"],
                  "ownedPaths": intent["ownedPaths"], "claims": [], "patchOperationRefs": [],
                  "sourcePreparationRefs": [operation_id]}
        old = state.get("sealed_input", manifest.source_digest)
        if old:
            for name in ("repositoryId", "repositoryRoot", "objectRoot", "sourceManifest", "coverageDigest"):
                if old["payload"].get(name) != sealed[name]:
                    raise JenkinsError("sealed_input_conflict", "Current-source identity cannot replace different sealed inputs")
        else:
            state.put("sealed_input", manifest.source_digest, sealed, expected_version=0)
        register_sealed_consumer(state, repository_id_value=identity["repositoryId"], session_id=identity["sessionId"],
                                 owner=identity["attemptId"], source_digest=manifest.source_digest,
                                 manifest_digest=manifest.source_digest)
        _ensure_patch_metadata(state, root, intent, sealed, policy_root=root)
        _verify_current(root, intent)
        _verify_external(state, intent)
        result = response("prepared", identity={**identity, "sourceInputDigest": manifest.source_digest,
                                                 "coverageDigest": coverage["coverageDigest"]},
                          sealedInputRef=manifest.source_digest, currentSourceRef=reference,
                          sourcePreparationRef=operation_id, buildRoot=intent["buildRoot"])
        if operation["status"] != "complete":
            state.transition_operation(operation_id, operation["status"], "complete", {
                "sourceDigest": manifest.source_digest, "coverageDigest": coverage["coverageDigest"],
                "currentHashes": intent["currentHashes"], "ownedPaths": intent["ownedPaths"],
                "untrackedPaths": intent["untrackedPaths"], "externalInputs": intent["externalInputs"],
                "canonicalSourceModified": False})
        state.put("current_source_preparation", reference, {"status": "prepared", "result": result},
                  expected_version=previous["version"] if previous else 0)
        state.submit_request(_request(intent, result))
        return result


def submit_current_source(state, root: Path, payload: dict, *, transport=None, manager=None) -> dict:
    from .submission import dispatch_registered, DEFAULT_BASE_URL
    reference, intent = _load_intent(state, root, payload)
    identity = intent["identity"]
    runtime = _runtime_binding(root, identity, payload)
    parameters = {"currentSourceRef": reference, "attemptId": identity["attemptId"], "generation": "1",
                  "buildRoot": intent["buildRoot"], "stageImplementationDigest": identity["stageImplementationDigest"]}
    # Authorization travels with the submission rather than being assumed by
    # the pipeline: without these the flow reaches Acceptance and silently
    # stops short of the commit stage, which reads as success.
    authorization = payload.get("authorization")
    if isinstance(authorization, Mapping):
        if authorization.get("allowCommit") is True:
            parameters["ALLOW_COMMIT"] = "true"
            commit_token = authorization.get("commitAuthorization")
            if not isinstance(commit_token, str) or not commit_token:
                raise JenkinsError("commit_authorization_missing",
                                   "Commit authorization requires its State scope identity")
            parameters["COMMIT_AUTHORIZATION"] = commit_token
        if authorization.get("allowNotify") is True:
            parameters["ALLOW_NOTIFY"] = "true"
            destination = authorization.get("notifyDestination")
            if not isinstance(destination, str) or not destination:
                raise JenkinsError("notification_destination_missing",
                                   "Notification authorization requires its destination")
            parameters["NOTIFY_DESTINATION"] = destination
    # An absent marker keeps the long-standing "flow" namespace, so existing
    # submissions retain their exact external delivery identity.
    marker = payload.get("submissionRequestId")
    if isinstance(marker, str) and marker:
        parameters["submissionRequestId"] = marker
    fields = {"REPOSITORY_ID": identity["repositoryId"], "SESSION_ID": identity["sessionId"],
              "REQUEST_ID": identity["requestId"], "ATTEMPT_ID": identity["attemptId"], "GENERATION": "1",
              "CURRENT_SOURCE_REF": reference, "BUILD_ROOT": intent["buildRoot"],
              "STAGE_IMPLEMENTATION_DIGEST": identity["stageImplementationDigest"]}
    if runtime:
        parameters["runtimeOperationId"] = runtime
        fields["RUNTIME_OPERATION_ID"] = runtime
    if manager is None and transport is None:
        from .deployment.spec import load_spec
        from .deployment.paths import resolve_paths
        from .deployment.manager import DeploymentManager
        spec = load_spec(root / ".jenkins/deployment-spec.json")
        manager = DeploymentManager(spec, resolve_paths(spec))
    dispatch = {key: identity[key] for key in ("repositoryId", "sessionId", "requestId")}
    dispatch["parameters"] = parameters
    return dispatch_registered(state, dispatch, fields, registered_request=state.get("current_source_intent", reference),
                               base_url=manager.base_url if manager else DEFAULT_BASE_URL,
                               job="zircon-flow", transport=transport, manager=manager)


def handle(action, payload, state, repo_root, *, domain="current-source"):
    root = repo_root.absolute()
    if action == "register":
        from .workflow.handler import _trusted_driver
        return register_current_source(state, root, payload, trusted_driver_digest=_trusted_driver(root, _identity(payload)))
    if action == "submit":
        return submit_current_source(state, root, payload)
    if action == "query":
        if not payload.get("currentSourceRef"):
            identity = _identity(payload)
            slot = digest([identity[key] for key in ("repositoryId", "sessionId", "requestId")])
            previous = state.get("current_source_request", slot)
            if not previous:
                return response("not_found", currentSourceRef=None, preparation=None)
            payload = {**payload, "currentSourceRef": previous["payload"]["currentSourceRef"]}
        reference, intent = _load_intent(state, root, payload)
        preparation = state.get("current_source_preparation", reference)
        identity = intent["identity"]
        return response("prepared" if preparation else "registered", currentSourceRef=reference,
                        preparation=preparation,
                        request=state.get_request(identity["repositoryId"], identity["sessionId"], identity["requestId"]))
    raise JenkinsError("operation_unknown", "Unknown current-source action")

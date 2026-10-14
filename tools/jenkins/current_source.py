"""Coordinator intake of exact, already-applied checkout bytes.

Registration reads the selected files and records an immutable intent. Only
the coordinator preparation operation writes its content-addressed input store;
neither operation applies, compensates, or claims files in the shared checkout.
"""
from __future__ import annotations

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
SOURCE_ROOT_CAPABILITY = 'same-repository-worktree-v1'


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


def _source_root(root: Path, value) -> Path:
    """Admit a same-repository worktree as the byte source for this intake.

    Repository identity stays bound to the canonical checkout, so a worktree may
    supply source bytes without ever becoming the repository it is registered
    against.  Sharing the checkout's Git common directory is what makes it the
    same repository; a merely similar path would silently re-point the intake at
    foreign content.  Managed worktrees live inside the checkout, so containment
    is not evidence either way and is not tested.
    """
    if value in (None, ""):
        return root
    source = Path(value).absolute()
    if source == root or not source.is_dir() or source.is_symlink():
        raise JenkinsError("current_source_root_invalid",
                           "Current source root must be a separate physical directory",
                           details={"sourceRoot": str(source)})
    completed = subprocess.run(["git", "-C", str(source), "rev-parse",
                                "--path-format=absolute", "--git-common-dir"],
                               capture_output=True, text=True, check=False)
    if completed.returncode:
        raise JenkinsError("current_source_root_invalid",
                           "Current source root is not a Git worktree",
                           details={"sourceRoot": str(source)})
    if Path(completed.stdout.strip()).absolute() != (root / ".git"):
        raise JenkinsError("current_source_root_mismatch",
                           "Current source root belongs to a different repository",
                           details={"sourceRoot": str(source)})
    return source


def _verify_current(root: Path, intent: Mapping) -> None:
    source = _source_root(root, intent.get("sourceRoot"))
    if _head(source) != intent["baseHead"]:
        raise JenkinsError("current_source_base_changed", "Current checkout HEAD differs from its registered base")
    expected = intent["currentHashes"]
    tracked, untracked = _git_selection(source, list(expected))
    selected_untracked = {path for path, checksum in expected.items() if checksum is not None and path not in tracked}
    if selected_untracked != set(intent["untrackedPaths"]) or not selected_untracked <= untracked:
        raise JenkinsError("current_source_untracked_mismatch", "Explicit untracked files differ from Git's selected current closure")
    for path, checksum in expected.items():
        target = source.joinpath(*path.split("/"))
        # Relative paths are validated against the identity root; both roots
        # share the same repository layout, so one canonical check covers both.
        canonical_path(root, path)
        if checksum is None:
            if path not in tracked or target.exists():
                raise JenkinsError("current_source_deletion_invalid", "Null hashes may name only absent tracked files", details={"path": path})
        elif not target.is_file():
            raise JenkinsError("current_source_file_missing", "Selected source must be a present ordinary file", details={"path": path})
        if _before_hash(source, path) != checksum:
            raise JenkinsError("current_source_hash_mismatch", "Current source differs from its registered hash", details={"path": path})


def _authorize(state, intent) -> dict:
    identity = intent["identity"]
    from .source.external_inputs import authorization_paths
    return state.check_authorization(identity["repositoryId"], identity["sessionId"],
                                     "validation", list(intent["currentHashes"]) + authorization_paths(intent.get("externalInputs", [])))


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
    if any(path.split("/", 1)[0] in {".git", ".codex", ".jenkins", ".opencode", "target"} for path in hashes):
        raise JenkinsError("current_source_control_forbidden", "Product inputs cannot include checkout control or compiler output files")
    owned = _paths(root, payload.get("ownedPaths"), "ownedPaths")
    untracked = _paths(root, payload.get("untrackedPaths"), "untrackedPaths")
    if not owned or not set(owned) <= set(hashes) or not set(untracked) <= set(hashes):
        raise JenkinsError("current_source_scope_invalid", "Owned and untracked paths must be explicit subsets of the current closure")
    base = payload.get("baseHead")
    if not isinstance(base, str) or len(base) not in {40, 64} or any(c not in "0123456789abcdef" for c in base):
        raise JenkinsError("current_source_base_missing", "Current source requires an immutable Git base revision")
    source = _source_root(root, payload.get("sourceRoot"))
    from .source.external_inputs import normalize_current
    supplied = {"schemaVersion": 1, "kind": "current-source", "identity": identity,
                "repositoryRoot": str(root), "baseHead": base, "currentHashes": hashes,
                "ownedPaths": owned, "untrackedPaths": untracked,
                "coverage": validate_coverage(payload.get("coverage")),
                "buildRoot": _build_root(root, payload.get("buildRoot")),
                "runtimeOperationId": _runtime_binding(root, identity, payload),
                "externalInputs": normalize_current(root, payload.get("externalInputs", []))}
    # Recorded only when a worktree actually supplies the bytes: an ordinary
    # checkout keeps the exact intent shape it has always had, so historical
    # currentSourceRef digests stay verifiable.
    if source != root:
        supplied["sourceRoot"] = str(source)
    slot = digest([identity[key] for key in ("repositoryId", "sessionId", "requestId")])
    with _lock(root):
        authorization = _authorize(state, supplied)
        intent = {**supplied, "authorizationSnapshot": authorization["snapshotDigest"]}
        reference = digest(intent)
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
        from .source.external_inputs import verify_current as verify_external
        verify_external(intent["externalInputs"])
        with state.transaction() as connection:
            state.put("current_source_intent", reference, intent, expected_version=0, connection=connection)
            state.put("current_source_request", slot, {"currentSourceRef": reference}, expected_version=0, connection=connection)
        return response("registered", currentSourceRef=reference, reused=False)


def _load_intent(state, root: Path, payload: dict) -> tuple[str, dict]:
    reference = require_digest(payload.get("currentSourceRef"), "currentSourceRef")
    record = state.get("current_source_intent", reference)
    if not record or digest(record["payload"]) != reference:
        raise JenkinsError("current_source_intent_missing", "Immutable current-source intake is missing or corrupt")
    intent = record["payload"]
    if intent["identity"] != _identity(payload) or intent["repositoryRoot"] != str(root.absolute()):
        raise JenkinsError("current_source_identity_mismatch", "Current-source intake belongs to another request or checkout")
    # Re-admit the registered byte source on every load: it must still be a
    # live worktree of this same repository, not merely the path it named once.
    if intent.get("sourceRoot") and _source_root(root, intent["sourceRoot"]) != Path(intent["sourceRoot"]).absolute():
        raise JenkinsError("current_source_root_changed", "Registered current-source worktree moved or changed identity")
    if payload.get("buildRoot") and _build_root(root, payload["buildRoot"]) != intent["buildRoot"]:
        raise JenkinsError("current_source_build_root_mismatch", "Current-source preparation must retain its build root")
    runtime = _runtime_binding(root, intent["identity"], payload)
    if runtime != intent.get("runtimeOperationId"):
        raise JenkinsError("current_source_runtime_mismatch", "Current-source control must retain its registered runtime operation")
    if _authorize(state, intent)["snapshotDigest"] != intent["authorizationSnapshot"]:
        raise JenkinsError("current_source_authorization_changed", "Current-source authorization differs from its registered scope")
    return reference, intent


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
        from .source.external_inputs import verify_current as verify_external, seal as seal_external
        verify_external(intent["externalInputs"])
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
                     external=seal_external(intent["externalInputs"], object_root))
        manifest = SealedManifest(plain, digest(plain))
        _verify_current(root, intent)
        verify_external(intent["externalInputs"])
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
        verify_external(intent["externalInputs"])
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

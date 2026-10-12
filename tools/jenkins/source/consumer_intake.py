"""Authorize a registered request to consume existing, immutable sealed input.

This intake writes references only. It never claims, prepares, materializes or
executes source. Request, grant and consumer checks use one State transaction;
the existing helper's consumer representation remains unchanged.
"""
from __future__ import annotations

from pathlib import Path

from ..contracts import JenkinsError, digest, file_digest, identifier, require_digest, response
from ..deployment.driver import _plain, active_driver_binding
from ..resources import canonical_build_root, physical_path_under
from . import _consumer_key, canonical_paths, compute_source_digest, repository_id
from .external_inputs import verify_objects as verify_external_objects
from .consumer_checkout import resolve_consumer_checkout


_IDENTIFIERS = ("repositoryId", "sessionId", "requestId", "attemptId")
_DIGESTS = ("sourceInputDigest", "coverageDigest", "stageImplementationDigest")
_IDENTITY_FIELDS = (*_IDENTIFIERS, "generation", *_DIGESTS)
_ALLOWED = frozenset((*_IDENTITY_FIELDS, "identity", "driverDigest", "owner", "sourceDigest",
                      "manifestDigest", "sealedInputRef", "ownedPaths", "buildRoot", "objectRoot",
                      "operationId", "runtimeOperationId", "checkoutBindingRef", "repositoryRoot", "jointGroupId"))


def _identity(payload: dict) -> dict:
    supplied = payload.get("identity")
    if not isinstance(supplied, dict):
        raise JenkinsError("identity_missing", "Consumer intake requires a registered request identity")
    if set(supplied) - set((*_IDENTITY_FIELDS, "driverDigest")):
        raise JenkinsError("identity_invalid", "Consumer identity contains unsupported fields")
    identity = {key: supplied.get(key) for key in _IDENTITY_FIELDS}
    for key in _IDENTIFIERS:
        identifier(identity[key], key)
    if type(identity["generation"]) is not int or identity["generation"] < 1:
        raise JenkinsError("generation_invalid", "Consumer generation must be a positive integer")
    for key in _DIGESTS:
        require_digest(identity[key], key)
    if "driverDigest" in supplied:
        identity["driverDigest"] = require_digest(supplied["driverDigest"], "driverDigest")
    for key, value in identity.items():
        if key in payload and payload[key] != value:
            raise JenkinsError("identity_conflict", "Consumer control and request identities differ")
    return identity


def _current_runtime(context: Path, payload: dict, intent: dict, identity: dict) -> dict | None:
    formal = ((context / ".jenkins/deployment-spec.json").exists()
              or (context / ".jenkins/state/deployment/driver.json").exists())
    if not formal and not any(key in payload or key in intent for key in
                              ("runtimeOperationId", "driverDigest", "checkoutBindingRef", "jointGroupId")):
        return None
    binding = active_driver_binding(context)
    driver = require_digest(binding.get("digest"), "active driver digest")
    operation = identifier(binding.get("runtimeOperationId"), "active runtime operation")
    supplied_driver = require_digest(payload.get("driverDigest"), "driverDigest")
    supplied_operation = identifier(payload.get("runtimeOperationId"), "runtimeOperationId")
    if (supplied_driver != driver or identity["stageImplementationDigest"] != driver
            or identity.get("driverDigest", driver) != driver):
        raise JenkinsError("driver_context_mismatch", "Consumer must use the current verified sealed driver")
    if supplied_operation != operation:
        raise JenkinsError("consumer_runtime_mismatch", "Consumer must use the current verified runtime operation")
    if (intent.get("driverDigest", driver) != driver
            or intent.get("runtimeOperationId", operation) != operation):
        raise JenkinsError("request_identity_mismatch", "Registered consumer runtime differs from the current driver")
    return {"driverDigest": driver, "runtimeOperationId": operation}


def _repository(root: Path, value: object) -> Path:
    if not isinstance(value, str) or not value or not Path(value).is_absolute():
        raise JenkinsError("sealed_repository_missing", "Sealed input requires its physical repository root")
    selected = Path(value)
    _plain(root)
    _plain(selected)
    if (not root.is_dir() or not selected.is_dir() or root.resolve() != root
            or selected.resolve() != selected or str(selected).casefold() != str(root).casefold()):
        raise JenkinsError("sealed_repository_mismatch", "Sealed input belongs to another physical repository")
    return selected


def _scope(root: Path, values: object) -> list[str]:
    if not isinstance(values, list) or not values or any(not isinstance(value, str) for value in values):
        raise JenkinsError("sealed_scope_missing", "Sealed input requires explicit owned paths")
    paths = canonical_paths(root, values)
    if len(paths) != len(values):
        raise JenkinsError("sealed_scope_invalid", "Sealed owned paths contain duplicate identities")
    return paths


def _sealed_input(state, connection, root: Path, identity: dict, source: str) -> tuple[dict, list[str]]:
    record = state.get("sealed_input", source, connection=connection)
    if record is None:
        raise JenkinsError("sealed_input_missing", "Consumer requires an existing sealed input")
    sealed = record["payload"]
    if not isinstance(sealed, dict):
        raise JenkinsError("sealed_input_corrupt", "Sealed input record must be an object")
    if sealed.get("status") != "sealed":
        raise JenkinsError("sealed_input_missing", "Consumer requires an existing sealed input")
    manifest = sealed.get("sourceManifest", sealed.get("manifest"))
    if (not isinstance(manifest, dict) or sealed.get("sourceDigest") != source
            or compute_source_digest(manifest) != source
            or "manifest" in sealed and sealed["manifest"] != manifest
            or sealed.get("manifestDigest", source) != source):
        raise JenkinsError("sealed_input_corrupt", "Sealed manifest differs from its immutable source identity")
    repo = identity["repositoryId"]
    if sealed.get("repositoryId") != repo or manifest.get("repositoryId") != repo:
        raise JenkinsError("sealed_input_repository_mismatch", "Sealed source belongs to another repository")
    _repository(root, sealed.get("repositoryRoot"))
    coverage = sealed.get("coverage")
    coverage_ref = identity["coverageDigest"]
    if (not isinstance(coverage, dict) or sealed.get("coverageDigest") != coverage_ref
            or coverage.get("coverageDigest") != coverage_ref or coverage.get("sourceDigest") != source
            or digest({key: value for key, value in coverage.items() if key != "coverageDigest"}) != coverage_ref):
        raise JenkinsError("coverage_identity_mismatch", "Consumer coverage differs from the sealed input")
    owned = _scope(root, sealed.get("ownedPaths"))
    entries = manifest.get("entries")
    if not isinstance(entries, list) or not entries or any(not isinstance(entry, dict) for entry in entries):
        raise JenkinsError("invalid_source_manifest", "Sealed entries must be a nonempty object list")
    paths = _scope(root, [entry.get("path") for entry in entries])
    if not all(any(path == owned_path or path.startswith(owned_path + "/") for path in paths)
               for owned_path in owned):
        raise JenkinsError("sealed_scope_invalid", "Sealed owned scope has no corresponding manifest entry")
    return sealed, owned


def _objects(sealed: dict, payload: dict) -> tuple[str, str]:
    build = sealed.get("buildRoot")
    objects = sealed.get("objectRoot")
    if not isinstance(build, str) or not build or not isinstance(objects, str) or not objects:
        raise JenkinsError("sealed_object_binding_missing", "Sealed input requires registered physical object roots")
    approved = canonical_build_root(build)
    store = physical_path_under(approved, objects, allow_missing=False)
    if store != physical_path_under(approved, approved.namespace(), allow_missing=False):
        raise JenkinsError("object_root_mismatch", "Consumer must use the sealed input's approved object store")
    if "buildRoot" in payload and canonical_build_root(payload["buildRoot"]).path != approved.path:
        raise JenkinsError("consumer_identity_conflict", "Consumer cannot replace the sealed build root")
    if "objectRoot" in payload and physical_path_under(approved, payload["objectRoot"], allow_missing=False) != store:
        raise JenkinsError("consumer_identity_conflict", "Consumer cannot replace the sealed object root")
    manifest = sealed.get("sourceManifest", sealed.get("manifest"))
    for entry in manifest["entries"]:
        if entry.get("status") == "deleted":
            if entry.get("sha256") is not None or entry.get("objectDigest") is not None:
                raise JenkinsError("sealed_input_corrupt", "Deleted sealed entry cannot carry object bytes")
            continue
        if entry.get("status") != "present":
            raise JenkinsError("sealed_input_corrupt", "Sealed entry has an unsupported status")
        checksum = require_digest(entry.get("objectDigest"), "objectDigest")
        content_hash = require_digest(entry.get("sha256"), "sha256")
        if entry.get("kind") != "directory" and checksum != content_hash:
            raise JenkinsError("sealed_input_corrupt", "Sealed object differs from its content identity")
        obj = physical_path_under(approved, store / "inputs" / "objects" / checksum, allow_missing=False)
        if not obj.is_file() or file_digest(obj) != checksum:
            raise JenkinsError("input_object_missing", "Sealed input object is missing or corrupt")
    verify_external_objects(manifest, store)
    return str(approved.path), str(store)


def register_consumer(state, repo_root: Path, payload: dict) -> dict:
    """Attach one authorized request/attempt to an immutable shared source."""
    if not isinstance(payload, dict) or set(payload) - _ALLOWED:
        raise JenkinsError("consumer_input_invalid", "Consumer intake accepts source references only")
    identity = _identity(payload)
    source = require_digest(payload.get("sourceDigest"), "sourceDigest")
    manifest_ref = require_digest(payload.get("manifestDigest"), "manifestDigest")
    owner = identifier(payload.get("owner"), "owner")
    if owner != identity["attemptId"]:
        raise JenkinsError("consumer_owner_mismatch", "Consumer owner must be the registered request attempt")
    if source != identity["sourceInputDigest"] or manifest_ref != source or payload.get("sealedInputRef", source) != source:
        raise JenkinsError("sealed_input_digest_mismatch", "Consumer cannot replace the registered sealed source")
    context = Path(repo_root).absolute()
    if "operationId" in payload:
        identifier(payload["operationId"], "operationId")
    repo, session, request_id = (identity[key] for key in _IDENTIFIERS[:3])
    with state.transaction() as connection:
        request = state.get_request(repo, session, request_id, connection=connection)
        if request is None:
            raise JenkinsError("request_not_registered", "Consumer requires its immutable registered request")
        intent = request["payload"]
        if not isinstance(intent, dict) or request["payloadDigest"] != digest(intent):
            raise JenkinsError("request_identity_mismatch", "Registered request payload is corrupt")
        if request["generation"] != identity["generation"]:
            raise JenkinsError("request_generation_mismatch", "Consumer request generation changed")
        if (type(intent.get("generation")) is not int
                or any(intent.get(key) != value for key, value in identity.items())):
            raise JenkinsError("request_identity_mismatch", "Consumer differs from its registered request identity")
        if intent.get("sealedInputRef", source) != source:
            raise JenkinsError("request_identity_mismatch", "Registered source references conflict")
        if (request["status"] in {"cancel_requested", "cancelled"}
                or state.get("cancellation", f"{repo}:{session}:{request_id}", connection=connection) is not None):
            raise JenkinsError("request_cancelled", "Cancelled requests cannot add sealed consumers")
        runtime = _current_runtime(context, payload, intent, identity)
        root, checkout = resolve_consumer_checkout(state, connection, context, payload, intent, identity, source)
        sealed, owned = _sealed_input(state, connection, root, identity, source)
        grant = state.check_authorization(repo, session, "validation", owned, connection=connection)
        if intent.get("authorizationSnapshot", grant["snapshotDigest"]) != grant["snapshotDigest"]:
            raise JenkinsError("authorization_conflict", "Consumer authorization differs from its registered request")
        if "ownedPaths" in intent and _scope(root, intent["ownedPaths"]) != owned:
            raise JenkinsError("request_identity_mismatch", "Registered owned scope differs from sealed source")
        if "ownedPaths" in payload:
            supplied_paths = payload["ownedPaths"]
            if not isinstance(supplied_paths, list):
                raise JenkinsError("consumer_input_invalid", "Consumer ownedPaths must be a path list")
            if supplied_paths and _scope(root, supplied_paths) != owned:
                raise JenkinsError("consumer_identity_conflict", "Consumer cannot replace the sealed owned scope")
        build, objects = _objects(sealed, payload)
        if "buildRoot" in intent and canonical_build_root(intent["buildRoot"]).path != Path(build):
            raise JenkinsError("request_identity_mismatch", "Registered build root differs from sealed source")
        key = _consumer_key(repo, session, owner, source)
        consumer_payload = {"repositoryId": repo, "sessionId": session, "owner": owner,
                            "sourceDigest": source, "manifestDigest": manifest_ref, "status": "active"}
        binding = {"identity": identity, "requestPayloadDigest": request["payloadDigest"],
                   "authorizationSnapshot": grant["snapshotDigest"], "repositoryRoot": str(root),
                   "sourceDigest": source, "manifestDigest": manifest_ref, "ownedPaths": owned,
                   "buildRoot": build, "objectRoot": objects}
        if runtime is not None:
            binding.update(runtime)
        if checkout is not None:
            binding["sourceCheckout"] = checkout
            binding["contextRoot"] = str(context)
        if "operationId" in payload:
            binding["operationId"] = payload["operationId"]
        old = state.get("sealed_consumer", key, connection=connection)
        prior = state.get("sealed_consumer_intake", key, connection=connection)
        if ((old is not None and old["payload"] != consumer_payload)
                or (prior is not None and prior["payload"] != binding)):
            raise JenkinsError("consumer_identity_conflict", "Sealed consumer request binding is immutable")
        if old is None:
            old = state.put("sealed_consumer", key, consumer_payload, expected_version=0, connection=connection)
        if prior is None:
            state.put("sealed_consumer_intake", key, binding, expected_version=0, connection=connection)
        return response("registered", request_id=request_id, observed_generation=identity["generation"],
                        sealedInputRef=source, sourceDigest=source, manifestDigest=manifest_ref,
                        consumer=old, reused=prior is not None)

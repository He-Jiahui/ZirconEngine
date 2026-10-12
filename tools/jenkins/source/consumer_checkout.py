"""Resolve historical acceptance source for a validation-only consumer.

The original deployment binding and completed source journal establish where
the immutable input came from. They grant no new source ownership and never
select or adopt a historical runtime.
"""
from __future__ import annotations

from pathlib import Path

from ..contracts import JenkinsError, digest, identifier, require_digest
from ..deployment.driver import _plain
from . import canonical_path, canonical_paths, repository_id


_AUTHOR_FIELDS = ("repositoryId", "sessionId", "requestId", "attemptId", "generation",
                  "stageImplementationDigest")


def _need(condition: bool, message: str) -> None:
    if not condition:
        raise JenkinsError("consumer_checkout_mismatch", message)


def _physical(context: Path, value: object) -> Path:
    _need(isinstance(value, str) and bool(value) and Path(value).is_absolute(),
          "Original checkout requires an absolute physical root")
    root = Path(value)
    permitted = context / ".jenkins/tmp/acceptance"
    for part in (context, permitted, root):
        _plain(part)
    _need(context.is_dir() and root.is_dir() and root.parent == permitted
          and root.resolve() == root and context.resolve() == context,
          "Consumer source must be the registered original acceptance checkout")
    return root


def _paths(root: Path, values: object) -> list[str]:
    _need(isinstance(values, list) and bool(values) and all(isinstance(value, str) for value in values),
          "Original source authority requires an explicit path scope")
    paths = canonical_paths(root, values)
    _need(len(paths) == len(values), "Original source scope contains duplicate paths")
    return paths


def _hashes(root: Path, values: object) -> dict:
    _need(isinstance(values, dict) and bool(values), "Complete source journal requires hash evidence")
    result = {}
    for path, checksum in values.items():
        _need(isinstance(path, str), "Source journal path must be a string")
        canonical = canonical_path(root, path)
        _need(canonical not in result, "Source journal contains duplicate path identities")
        result[canonical] = None if checksum is None else require_digest(checksum, "source journal hash")
    return result


def _journal(state, connection, root: Path, sealed: dict, author: dict, owned: list[str]) -> list[dict]:
    references = sealed.get("patchOperationRefs")
    _need(isinstance(references, list) and bool(references)
          and all(isinstance(value, str) and value for value in references)
          and len(set(references)) == len(references), "Original sealed patch requires complete journal references")
    manifest = sealed.get("sourceManifest", sealed.get("manifest"))
    _need(isinstance(manifest, dict) and isinstance(manifest.get("entries"), list),
          "Original source manifest is unavailable")
    entries = {}
    for entry in manifest["entries"]:
        _need(isinstance(entry, dict) and isinstance(entry.get("path"), str), "Original sealed entry is invalid")
        path = canonical_path(root, entry["path"])
        _need(path not in entries, "Original manifest contains duplicate path identities")
        entries[path] = entry
    covered = set()
    evidence = []
    for reference in references:
        identifier(reference, "patch operation reference")
        row = state.get_operation(reference, connection=connection)
        _need(row is not None and row["kind"] == "source_patch" and row["status"] == "complete",
              "Original source journal is missing or incomplete")
        value = row["payload"]
        _need(isinstance(value, dict) and row["payloadDigest"] == digest(value),
              "Original source journal payload is corrupt")
        authorization = value.get("authorization")
        _need(isinstance(authorization, dict)
              and type(authorization.get("generation")) is int
              and all(authorization.get(key) == author[key] for key in _AUTHOR_FIELDS)
              and authorization.get("owner") == author["attemptId"]
              and value.get("owner") == author["attemptId"]
              and value.get("repositoryRoot") == str(root)
              and value.get("objectRoot") == sealed.get("objectRoot"),
              "Original source journal belongs to another author or checkout")
        paths = _paths(root, value.get("paths"))
        before = _hashes(root, value.get("beforeHashes"))
        expected = _hashes(root, value.get("expectedAfterHashes"))
        result = row.get("result")
        _need(isinstance(result, dict), "Original source journal lacks terminal hash evidence")
        after = _hashes(root, result.get("afterHashes"))
        _need(set(paths) == set(before) == set(expected) == set(after)
              and _hashes(root, result.get("beforeHashes")) == before and after == expected,
              "Original completed source journal changed its immutable scope or result")
        for path, checksum in after.items():
            entry = entries.get(path)
            _need(path in owned and entry is not None
                  and entry.get("sha256") == checksum
                  and entry.get("status") == ("deleted" if checksum is None else "present"),
                  "Original source journal does not explain the sealed owned bytes")
            covered.add(path)
        evidence.append({"operationId": reference, "payloadDigest": row["payloadDigest"],
                         "resultDigest": digest(result)})
    _need(covered == set(owned), "Original source journals do not cover the sealed owned scope")
    return evidence


def resolve_consumer_checkout(state, connection, context: Path, payload: dict,
                              intent: dict, identity: dict, source: str) -> tuple[Path, dict | None]:
    """Select only the request's existing source authority; never caller paths."""
    if intent.get("jointGroupId") is not None or payload.get("jointGroupId") is not None:
        from ..workflow.group_protocol import resolve_joint_checkout
        return resolve_joint_checkout(state, connection, context, payload, intent, identity, source)
    reference = payload.get("checkoutBindingRef")
    registered = intent.get("checkoutBindingRef")
    if reference is None and registered is None:
        if identity["repositoryId"] != repository_id(context):
            raise JenkinsError("source_repository_mismatch", "Consumer must use its physical repository identity")
        for values in (intent, payload):
            if "repositoryRoot" in values:
                value = values["repositoryRoot"]
                _need(isinstance(value, str) and value == str(context),
                      "Consumer cannot select an unregistered source checkout")
        return context, None
    reference = require_digest(reference, "checkoutBindingRef")
    _need(reference == registered, "Consumer source checkout differs from its registered request")
    record = state.get("acceptance_checkout", reference, connection=connection)
    _need(record is not None and isinstance(record["payload"], dict), "Original acceptance checkout is unavailable")
    binding = record["payload"]
    author = binding.get("identity")
    _need(isinstance(author, dict) and set(author) == set(_AUTHOR_FIELDS), "Original checkout identity is invalid")
    for key in _AUTHOR_FIELDS[:4]:
        identifier(author[key], key)
    _need(type(author["generation"]) is int and author["generation"] >= 1,
          "Original checkout generation is invalid")
    require_digest(author["stageImplementationDigest"], "original driver digest")
    _need(reference == digest({key: author[key] for key in _AUTHOR_FIELDS[:5]})
          and type(binding.get("schemaVersion")) is int and binding["schemaVersion"] == 1
          and binding.get("kind") == "original-acceptance-checkout"
          and binding.get("contextRoot") == str(context)
          and binding.get("driverDigest") == author["stageImplementationDigest"]
          and binding.get("productRepositoryAcceptance") is False,
          "Original deployment checkout authority changed")
    identifier(binding.get("runtimeOperationId"), "original runtime operation")
    require_digest(binding.get("authorizationSnapshot"), "original authorization snapshot")
    root = _physical(context, binding.get("checkoutRoot"))
    _need(identity["repositoryId"] == author["repositoryId"] == repository_id(root)
          and intent.get("repositoryRoot") == str(root)
          and ("repositoryRoot" not in payload or payload["repositoryRoot"] == str(root)),
          "Consumer request is not bound to the original physical checkout")
    source_row = state.get("sealed_input", source, connection=connection)
    _need(source_row is not None and isinstance(source_row["payload"], dict), "Original sealed source is unavailable")
    sealed = source_row["payload"]
    _need(sealed.get("status") == "sealed" and sealed.get("sourceDigest") == source
          and sealed.get("repositoryId") == author["repositoryId"] and sealed.get("repositoryRoot") == str(root)
          and sealed.get("sessionId") == author["sessionId"] and sealed.get("owner") == author["attemptId"],
          "Sealed input does not belong to the original checkout author")
    owned = _paths(root, sealed.get("ownedPaths"))
    allowed = _paths(root, binding.get("ownedPaths"))
    _need(all(any(path == scope or path.startswith(scope + "/") for scope in allowed) for path in owned),
          "Original acceptance scope does not cover the sealed owned paths")
    journals = _journal(state, connection, root, sealed, author, owned)
    return root, {"checkoutBindingRef": reference, "checkoutBindingDigest": digest(binding),
                  "originalIdentity": author, "patchJournals": journals}

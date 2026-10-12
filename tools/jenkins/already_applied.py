"""Prepare an explicitly authorized patch whose postimage is already present.

The immutable Git commit supplies the preimage. Registration retains only the
patch, hashes and provenance; the coordinator owns CAS publication during
preparation. No path in the canonical checkout is written or compensated here.
"""
from __future__ import annotations

from dataclasses import asdict
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import tomllib
from typing import Mapping

from .contracts import JenkinsError, digest, identifier, response
from .resources import canonical_build_root, physical_path_under
from .resources.paths import build_root_containing
from .source import (PathClaim, _atomic_write, _thaw, canonical_path,
                     compute_source_digest, get_sealed_manifest,
                     register_sealed_consumer, repository_id, seal_candidate)
from .source import baselines
from .source.unified_patch import _normalized_path, parse_unified_patch, render_patch_bytes

_KIND = "already-applied-patch"
_CONTROL = {".git", ".codex", ".jenkins", ".opencode", "target"}
_MAX_CLOSURE_FILES = 512
_MAX_CLOSURE_BYTES = 64 * 1024 * 1024
_DYNAMIC_RUST = re.compile(
    r"\b(?:include|include_str|include_bytes|env|option_env|asm|global_asm)\s*!|"
    r"#\s*\[\s*(?:path\s*=|link\s*\()|\bOUT_DIR\b")


def _hashes(root: Path, value, *, nullable: bool, empty: bool = False) -> dict:
    if not isinstance(value, Mapping) or (not value and not empty):
        raise JenkinsError("patch_scope_mismatch", "Exact input hashes are required")
    result = {}
    for supplied, checksum in value.items():
        if not isinstance(supplied, str):
            raise JenkinsError("patch_path_invalid", "Input hash paths must be canonical strings")
        path = _normalized_path(supplied)
        if path.split("/", 1)[0] in _CONTROL:
            raise JenkinsError("patch_control_forbidden", "Patch input cannot include checkout control or compiler output")
        if path in result:
            raise JenkinsError("patch_path_collision", "Input hashes contain aliased paths")
        if checksum is None:
            if not nullable:
                raise JenkinsError("patch_dependency_invalid", "Declared dependency hashes cannot be null")
        elif not isinstance(checksum, str) or not re.fullmatch(r"[0-9a-f]{64}", checksum):
            raise JenkinsError("patch_hash_invalid", "Input hashes must be lowercase SHA-256 or permitted null")
        if canonical_path(root, path) != path:
            raise JenkinsError("patch_path_collision", "Input path is not canonical")
        result[path] = checksum
    return dict(sorted(result.items()))


def _current_bytes(root: Path, hashes: Mapping) -> dict:
    # Reuse the resolver's component checks even for null deletion paths.
    baselines._scope(root, hashes)
    result, total = {}, 0
    for path, checksum in hashes.items():
        target = root.joinpath(*path.split("/"))
        if checksum is None:
            if target.exists():
                raise JenkinsError("foreign_edit_detected", "Expected deletion is no longer absent", details={"path": path})
            result[path] = None
            continue
        if not target.is_file() or target.stat().st_size > baselines._MAX_OBJECT_BYTES:
            raise JenkinsError("source_scope_incomplete", "Input must be a bounded ordinary file", details={"path": path})
        data = target.read_bytes()
        total += len(data)
        if total > _MAX_CLOSURE_BYTES:
            raise JenkinsError("source_scope_incomplete", "Declared closure exceeds the bounded intake")
        if hashlib.sha256(data).hexdigest() != checksum:
            raise JenkinsError("foreign_edit_detected", "Current input differs from its exact hash", details={"path": path})
        result[path] = data
    return result


def _provenance(trusted) -> dict:
    return _thaw(asdict(trusted.provenance))


def _verify_current(root: Path, intent: Mapping) -> tuple[object, dict]:
    trusted = baselines.resolve_baseline(
        root, repository_id_value=intent["identity"]["repositoryId"],
        base_head=intent["baseHead"], before_hashes=intent["beforeHashes"],
        baseline_ref=intent.get("baselineRef"))
    if _provenance(trusted) != intent["baselineProvenance"]:
        raise JenkinsError("baseline_provenance_changed", "Git preimage provenance differs from immutable registration")
    rendered = render_patch_bytes(intent["unifiedPatch"], trusted.before_bytes, intent["beforeHashes"])
    if {path: None if data is None else hashlib.sha256(data).hexdigest()
            for path, data in rendered.items()} != intent["afterHashes"]:
        raise JenkinsError("patch_after_mismatch", "Patch rendering differs from the registered postimage hashes")
    current = _current_bytes(root, {**intent["afterHashes"], **intent["dependencyHashes"]})
    if any(current[path] != data for path, data in rendered.items()):
        raise JenkinsError("patch_after_mismatch", "Patch rendering differs from canonical owned bytes")
    return trusted, current


def _relative(root: Path, directory: Path, value: str, *, check_physical: bool = True) -> str:
    if not isinstance(value, str) or not value or "\\" in value or ":" in value:
        raise JenkinsError("source_scope_incomplete", "Cargo path input is not a bounded relative path")
    target = Path(os.path.abspath(directory.joinpath(*value.split("/"))))
    try:
        relative = target.relative_to(root).as_posix()
    except ValueError as error:
        raise JenkinsError("baseline_required", "Cargo input outside this repository needs authoritative provenance") from error
    path = _normalized_path(relative)
    if check_physical:
        canonical_path(root, path)
        baselines._scope(root, {path: None})
    return path


def _cargo_closure(root: Path, current: Mapping, *, inventory: bool = True) -> None:
    """Admit a bounded local Cargo closure, rejecting unproven dynamic inputs.

    Metadata is also collected by the deployment-owned collector below. It does
    not prove build-script, procedural-macro or generated-input dependencies, so
    those cases remain blocked rather than trusting a client completeness flag.
    """
    if len(current) > _MAX_CLOSURE_FILES or current.get("cargo.toml") is None:
        raise JenkinsError("source_scope_incomplete", "Cargo preparation requires its explicit root manifest")
    manifests = {}
    for path, data in current.items():
        if path.rsplit("/", 1)[-1] != "cargo.toml" or data is None:
            continue
        try:
            manifests[path] = tomllib.loads(data.decode("utf-8"))
        except (UnicodeError, tomllib.TOMLDecodeError) as error:
            raise JenkinsError("source_scope_incomplete", "Cargo manifests must be valid sealed UTF-8 TOML") from error
    required = {"cargo.toml"}
    workspace_dependencies = manifests["cargo.toml"].get("workspace", {}).get("dependencies", {})
    for path, value in manifests.items():
        directory = root.joinpath(*path.split("/")).parent
        workspace = value.get("workspace", {})
        members = workspace.get("members", [])
        if not isinstance(members, list):
            raise JenkinsError("source_scope_incomplete", "Workspace members must be explicit")
        for member in members:
            if not isinstance(member, str) or any(c in member for c in "*?[]"):
                raise JenkinsError("source_scope_incomplete", "Glob workspace membership needs authoritative closure evidence")
            required.add(_relative(root, directory, member.rstrip("/") + "/Cargo.toml", check_physical=inventory))
        package = value.get("package")
        if package is None:
            continue
        if not isinstance(package, Mapping):
            raise JenkinsError("source_scope_incomplete", "Cargo package declaration is malformed")
        build = package.get("build")
        build_path = _normalized_path((directory / "build.rs").relative_to(root).as_posix())
        if (build is not None and build is not False) or (
                build is None and (current.get(build_path) is not None
                                   or inventory and (directory / "build.rs").exists())):
            raise JenkinsError("baseline_required", "Build-script input closure has no authoritative resolver")
        if value.get("lib", {}).get("proc-macro"):
            raise JenkinsError("baseline_required", "Procedural-macro input closure has no authoritative resolver")
        tables = [value.get(name, {}) for name in ("dependencies", "dev-dependencies", "build-dependencies")]
        for target in value.get("target", {}).values():
            if not isinstance(target, Mapping):
                raise JenkinsError("source_scope_incomplete", "Cargo target dependency declaration is malformed")
            tables.extend(target.get(name, {}) for name in ("dependencies", "dev-dependencies", "build-dependencies"))
        for table in tables:
            if not isinstance(table, Mapping):
                raise JenkinsError("source_scope_incomplete", "Cargo dependency declaration is malformed")
            for name, dependency in table.items():
                if isinstance(dependency, Mapping) and dependency.get("workspace") is True:
                    dependency = workspace_dependencies.get(name)
                    dependency_directory = root
                else:
                    dependency_directory = directory
                if (not isinstance(dependency, Mapping) or not isinstance(dependency.get("path"), str)
                        or any(key in dependency for key in ("git", "registry", "registry-index"))):
                    raise JenkinsError("baseline_required", "External Cargo dependencies need authoritative input provenance")
                required.add(_relative(root, dependency_directory, dependency["path"].rstrip("/") + "/Cargo.toml",
                                       check_physical=inventory))
        # Every ordinary Rust file in each selected local package is explicit.
        # This bounds normal module resolution without pretending to evaluate
        # dynamic macros or generated source.
        visited = 0
        for walk_root, directories, files in os.walk(directory, followlinks=False) if inventory else ():
            base = Path(walk_root)
            directories[:] = [name for name in directories if name.casefold() not in _CONTROL]
            for name in [*directories, *files]:
                candidate = base / name
                info = candidate.lstat()
                if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
                    raise JenkinsError("source_scope_incomplete", "Cargo package inventory contains a path alias")
                visited += 1
                if visited > 10000:
                    raise JenkinsError("source_scope_incomplete", "Cargo package inventory exceeds the bounded scope")
            for name in files:
                if name.casefold().endswith(".rs"):
                    required.add(canonical_path(root, (base / name).relative_to(root)))
        for section in ("lib", "bin", "test", "bench", "example"):
            targets = value.get(section, [] if section != "lib" else {})
            if section == "lib":
                targets = [targets] if targets else []
            if not isinstance(targets, list) or not all(isinstance(target, Mapping) for target in targets):
                raise JenkinsError("source_scope_incomplete", "Cargo target declaration is malformed")
            for target in targets:
                if "path" in target:
                    required.add(_relative(root, directory, target["path"], check_physical=inventory))
    if not required <= {path for path, data in current.items() if data is not None}:
        raise JenkinsError("source_scope_incomplete", "Declared hashes omit a required Cargo or Rust input",
                           details={"missingPaths": sorted(required - {p for p, data in current.items() if data is not None})})
    if inventory and (root / "Cargo.lock").exists() and current.get("cargo.lock") is None:
        raise JenkinsError("source_scope_incomplete", "The existing Cargo lockfile must be explicitly hash-bound")
    # Config and dynamic source cannot be proved by the ordinary module list.
    if (any(current.get(".cargo/" + name) is not None for name in ("config", "config.toml"))
            or inventory and any((root / ".cargo" / name).exists() for name in ("config", "config.toml"))):
        raise JenkinsError("baseline_required", "Cargo configuration inputs need authoritative closure evidence")
    for path, data in current.items():
        if path.endswith(".rs") and data is not None:
            try:
                text = data.decode("utf-8")
            except UnicodeError as error:
                raise JenkinsError("source_scope_incomplete", "Rust inputs must be sealed UTF-8") from error
            if _DYNAMIC_RUST.search(text):
                raise JenkinsError("baseline_required", "Dynamic Rust input closure has no authoritative resolver",
                                   details={"path": path})


def register_already_applied_intent(state, repo_root: Path, payload: dict, *,
                                    trusted_driver_digest: str) -> dict:
    from .patch_context import patch_checkout_for_registration
    from .patches import _authorize, _build_root, _identity, _lock, _runtime_binding
    context = repo_root.absolute()
    root = patch_checkout_for_registration(state, context, payload)
    identity = _identity(payload)
    if payload.get("inputMode") != "already-applied":
        raise JenkinsError("patch_input_mode_invalid", "Already-applied intake requires its explicit input mode")
    if identity["stageImplementationDigest"] != trusted_driver_digest:
        raise JenkinsError("driver_context_mismatch", "Patch intake differs from its trusted sealed driver")
    before = _hashes(root, payload.get("beforeHashes"), nullable=True)
    after = _hashes(root, payload.get("afterHashes"), nullable=True)
    text = payload.get("unifiedPatch")
    if not isinstance(text, str) or len(text.encode("utf-8")) > 4 * 1024 * 1024:
        raise JenkinsError("patch_input_invalid", "Bounded unified patch text is required")
    parsed = parse_unified_patch(text)
    if {item.path for item in parsed.files} != set(before) or set(before) != set(after):
        raise JenkinsError("patch_scope_mismatch", "Patch, before and after hash scopes must match exactly")
    dependencies = payload.get("declaredDependencies")
    if not isinstance(dependencies, list) or not all(isinstance(value, str) for value in dependencies):
        raise JenkinsError("patch_dependency_invalid", "Declared dependencies must be explicit")
    declared = [_normalized_path(value) for value in dependencies]
    if len(set(declared)) != len(declared) or set(declared) & set(before):
        raise JenkinsError("patch_dependency_invalid", "Dependencies cannot alias or overlap owned patch paths")
    dependency_hashes = _hashes(root, payload.get("dependencyHashes"), nullable=False, empty=True)
    if set(declared) != set(dependency_hashes):
        raise JenkinsError("patch_dependency_invalid", "Declared dependencies require exactly their nonnull hashes")
    coverage = payload.get("coverage")
    if (not isinstance(coverage, dict) or not coverage
            or {"sourceDigest", "coverageDigest"} & set(coverage)):
        raise JenkinsError("coverage_missing", "Source-independent validation coverage is required")
    if payload.get("externalInputs") not in (None, []):
        raise JenkinsError("baseline_required", "External input closure has no authoritative already-applied resolver")
    trusted = baselines.resolve_baseline(root, repository_id_value=identity["repositoryId"],
                                        before_hashes=before, base_head=payload.get("baseHead"),
                                        baseline_ref=payload.get("baselineRef"))
    supplied = {"schemaVersion": 1, "inputMode": "already-applied", "identity": identity,
                "repositoryRoot": str(root), "contextRoot": str(context),
                "unifiedPatch": text, "patchDigest": digest(text),
                "baseHead": trusted.provenance.base_head, "beforeHashes": before,
                "afterHashes": after, "declaredDependencies": sorted(declared),
                "dependencyHashes": dependency_hashes, "coverage": coverage,
                "externalInputs": [], "buildRoot": _build_root(context, payload.get("buildRoot")),
                "runtimeOperationId": _runtime_binding(context, identity, payload),
                "baselineProvenance": _provenance(trusted)}
    supplied["inputDigest"] = digest(supplied)
    slot = digest([identity[key] for key in ("repositoryId", "sessionId", "requestId")])
    owner_slot = digest([repository_id(root), identity["attemptId"]])
    with _lock(root):
        authorization = _authorize(state, supplied)
        _verify_current(root, supplied)
        intent = {**supplied, "authorizationSnapshot": authorization["snapshotDigest"]}
        reference = digest(intent)
        prior = state.get("patch_request", slot)
        if prior:
            if prior["payload"] != {"patchRequestRef": reference, "inputDigest": supplied["inputDigest"]}:
                raise JenkinsError("patch_request_conflict", "Request already owns a different immutable patch intent")
            old = state.get("patch_intent", reference)
            if not old or old["payload"] != intent:
                raise JenkinsError("patch_intent_missing", "Registered patch intent is missing or corrupt")
            return response("registered", patchRequestRef=reference, reused=True)
        if state.get_request(identity["repositoryId"], identity["sessionId"], identity["requestId"]):
            raise JenkinsError("patch_request_conflict", "An existing source request cannot be adopted")
        owner = state.get("patch_owner", owner_slot)
        if owner or any(row["payload"].get("repositoryId") == identity["repositoryId"]
                        and row["payload"].get("owner") == identity["attemptId"]
                        for row in state.list("path_claims")):
            raise JenkinsError("patch_owner_conflict", "An existing source owner cannot be adopted")
        with state.transaction() as connection:
            if state.get_request(identity["repositoryId"], identity["sessionId"], identity["requestId"], connection=connection):
                raise JenkinsError("patch_request_conflict", "Request appeared during immutable intake")
            state.put("patch_intent", reference, intent, expected_version=0, connection=connection)
            state.put("patch_request", slot, {"patchRequestRef": reference, "inputDigest": supplied["inputDigest"]},
                      expected_version=0, connection=connection)
            state.put("patch_owner", owner_slot, {"identity": identity, "patchRequestRef": reference},
                      expected_version=0, connection=connection)
        return response("registered", patchRequestRef=reference, request_id=identity["requestId"],
                        observed_generation=1, reused=False)


def _own_intent(state, reference: str) -> dict:
    from .patches import _authorize
    identifier(reference, "patchRequestRef")
    record = state.get("patch_intent", reference)
    if not record or digest(record["payload"]) != reference:
        raise JenkinsError("patch_intent_missing", "Immutable already-applied intent is missing or corrupt")
    intent = record["payload"]
    if intent.get("inputMode") != "already-applied" or intent.get("schemaVersion") != 1:
        raise JenkinsError("source_preparation_untrusted", "Preparation is not an already-applied patch")
    identity = intent["identity"]
    slot = digest([identity[key] for key in ("repositoryId", "sessionId", "requestId")])
    request = state.get("patch_request", slot)
    owner = state.get("patch_owner", digest([identity["repositoryId"], identity["attemptId"]]))
    if (not request or request["payload"] != {"patchRequestRef": reference, "inputDigest": intent["inputDigest"]}
            or not owner or owner["payload"] != {"identity": identity, "patchRequestRef": reference}
            or digest(intent["unifiedPatch"]) != intent["patchDigest"]):
        raise JenkinsError("patch_intent_corrupt", "Patch consumer binding differs from immutable intake")
    _authorize(state, intent)
    return intent


def _object_root(intent) -> Path:
    selected = canonical_build_root(intent["buildRoot"])
    return physical_path_under(selected, selected.namespace())


def _cas_bytes(object_root: Path, checksum: str) -> bytes:
    if not isinstance(checksum, str) or not re.fullmatch(r"[0-9a-f]{64}", checksum):
        raise JenkinsError("input_object_corrupt", "CAS object identity is invalid")
    selected = build_root_containing(object_root)
    target = physical_path_under(selected, object_root / "inputs/objects" / checksum, allow_missing=False)
    if not target.is_file() or target.stat().st_size > baselines._MAX_OBJECT_BYTES:
        raise JenkinsError("input_object_corrupt", "CAS object is absent or exceeds the input bound")
    data = target.read_bytes()
    if hashlib.sha256(data).hexdigest() != checksum:
        raise JenkinsError("input_object_corrupt", "CAS object bytes differ from their immutable identity")
    return data


def _write_cas(object_root: Path, data: bytes) -> str:
    checksum = hashlib.sha256(data).hexdigest()
    target = physical_path_under(build_root_containing(object_root), object_root / "inputs/objects" / checksum)
    if target.exists():
        if _cas_bytes(object_root, checksum) != data:
            raise JenkinsError("input_object_corrupt", "Existing CAS object cannot be replaced")
    else:
        _atomic_write(target, data)
        _cas_bytes(object_root, checksum)
    return checksum


def _git_tree(intent: Mapping) -> tuple[Path, str, dict]:
    # Deliberately verify the specified immutable commit, not today's HEAD.
    root = baselines._plain_root(intent["repositoryRoot"], intent["identity"]["repositoryId"])
    base = intent["baseHead"]
    if not isinstance(base, str) or not re.fullmatch(r"(?:[0-9a-f]{40}|[0-9a-f]{64})", base):
        raise JenkinsError("baseline_provenance_changed", "Immutable baseline commit is invalid")
    cache = {}
    if baselines._git(root, "config", "--local", "--includes", "--get-regexp",
                      r"^(extensions\.partialclone|remote\..*\.promisor)$", allow_missing=True):
        raise JenkinsError("baseline_required", "Partial-clone object reads cannot provide a strictly read-only baseline")
    commit = baselines._object(root, "commit", base, cache)
    first = commit.split(b"\n", 1)[0]
    if not first.startswith(b"tree "):
        raise JenkinsError("baseline_provenance_changed", "Immutable commit has no tree binding")
    tree = first[5:].decode("ascii")
    provenance = intent["baselineProvenance"]
    if (provenance.get("kind") != "git-commit" or provenance.get("repository_id") != intent["identity"]["repositoryId"]
            or provenance.get("repository_root") != str(root) or provenance.get("base_head") != base
            or provenance.get("tree_object_id") != tree):
        raise JenkinsError("baseline_provenance_changed", "Immutable Git provenance changed")
    return root, tree, cache


def _request(intent, prepared) -> dict:
    return {**prepared["identity"], "sealedInputRef": prepared["sealedInputRef"],
            "sourceInputDigest": prepared["sealedInputRef"], "patchRequestRef": prepared["patchRequestRef"],
            "sourcePreparationRef": prepared["sourcePreparationRef"],
            "sourcePreparationRefs": [prepared["sourcePreparationRef"]], "patchOperationRefs": [],
            "ownedPaths": sorted(intent["beforeHashes"]), "buildRoot": intent["buildRoot"],
            "driverDigest": intent["identity"]["stageImplementationDigest"],
            "authorizationSnapshot": intent["authorizationSnapshot"], "inputMode": "already-applied"}


def _operation_payload(intent, reference: str, identity: dict, object_root: Path) -> dict:
    return {"preparationKind": _KIND, "patchRequestRef": reference, "identity": identity,
            "repositoryRoot": intent["repositoryRoot"], "objectRoot": str(object_root),
            "buildRoot": intent["buildRoot"], "beforeObjects": intent["beforeHashes"],
            "expectedAfterHashes": intent["afterHashes"], "afterHashes": intent["afterHashes"],
            "ownedPaths": sorted(intent["beforeHashes"]), "baselineProvenance": intent["baselineProvenance"],
            "runtimeOperationId": intent.get("runtimeOperationId"), "canonicalSourceModified": False}


def _prepared_result(intent, reference: str, identity: dict, source_digest: str) -> dict:
    operation_id = "already-applied-" + reference
    return response("prepared", identity=identity, sealedInputRef=source_digest,
                    sourcePreparationRef=operation_id, sourcePreparationRefs=[operation_id],
                    patchOperationRefs=[], patchRequestRef=reference, buildRoot=intent["buildRoot"],
                    canonicalSourceModified=False)


def _verify_cargo_binding(state, intent, source_digest: str, manifest: Mapping, current: Mapping) -> None:
    record = state.get("sealed_cargo_evidence", source_digest)
    formal = state.get("environment_policy", "default") is not None or (
        Path(intent["contextRoot"]) / ".jenkins/state/deployment/driver.json").is_file()
    if record is None:
        if formal:
            raise JenkinsError("cargo_evidence_missing", "Formal preparation requires authoritative Cargo evidence")
        return
    from .patches import _validate_metadata_workspace
    from .workflow.planning import _cargo_evidence
    raw = record["payload"]
    if (raw.get("repositoryId") != intent["identity"]["repositoryId"]
            or raw.get("buildRoot") != intent["buildRoot"]
            or raw.get("metadataDigest") != digest(raw.get("cargoMetadata") or raw.get("metadata"))
            or raw.get("toolchainDigest") != digest(raw.get("toolchain"))):
        raise JenkinsError("cargo_evidence_conflict", "Cargo metadata or toolchain binding differs from sealed evidence")
    _validate_metadata_workspace(raw, intent["buildRoot"], source_digest, manifest=manifest)
    evidence = _cargo_evidence(state, source_digest)
    metadata = evidence["cargoMetadata"]
    packages = metadata.get("packages")
    if not isinstance(packages, list) or not packages:
        raise JenkinsError("source_scope_incomplete", "Cargo evidence has no complete local package declaration")
    for package in packages:
        if not isinstance(package, Mapping) or not isinstance(package.get("manifest_path"), str):
            raise JenkinsError("source_scope_incomplete", "Cargo package path proof is malformed")
        paths = [package["manifest_path"]]
        targets = package.get("targets")
        if not isinstance(targets, list) or not targets:
            raise JenkinsError("source_scope_incomplete", "Cargo target path proof is missing")
        for target in targets:
            if not isinstance(target, Mapping) or not isinstance(target.get("src_path"), str):
                raise JenkinsError("source_scope_incomplete", "Cargo target path proof is malformed")
            paths.append(target["src_path"])
        if any(current.get(_normalized_path(path)) is None for path in paths):
            raise JenkinsError("source_scope_incomplete", "Cargo evidence references unsealed manifest or target input")
    inputs = raw.get("inputDigests")
    if (not isinstance(inputs, Mapping) or not inputs
            or raw.get("manifestInputsDigest") != digest(inputs)
            or any(not isinstance(path, str) or _normalized_path(path) not in current
                   or current[_normalized_path(path)] is None
                   or hashlib.sha256(current[_normalized_path(path)]).hexdigest() != checksum
                   for path, checksum in inputs.items())
            or current.get("cargo.lock") is None
            or raw.get("lockDigest") != hashlib.sha256(current["cargo.lock"]).hexdigest()):
        raise JenkinsError("cargo_evidence_conflict", "Cargo input or lock hashes differ from the complete sealed closure")


def require_already_applied_preparation(state, reference: str, *, identity: Mapping,
                                        source_digest: str) -> dict:
    operation = state.get_operation(reference)
    if (not operation or operation["kind"] != "source_preparation" or operation["status"] != "complete"
            or operation.get("payloadDigest") != digest(operation["payload"])
            or operation["payload"].get("preparationKind") != _KIND):
        raise JenkinsError("source_preparation_untrusted", "Exact completed already-applied preparation is required")
    patch_ref = operation["payload"].get("patchRequestRef")
    intent = _own_intent(state, patch_ref)
    if reference != "already-applied-" + patch_ref:
        raise JenkinsError("source_preparation_untrusted", "Preparation reference differs from immutable intake")
    preparation = state.get("patch_preparation", patch_ref)
    if preparation and preparation["payload"].get("status") == "cancelled":
        raise JenkinsError("patch_cancelled", "Cancelled consumer preparation cannot be reused")
    own = intent["identity"]
    manifest = get_sealed_manifest(state, repository_id_value=own["repositoryId"],
                                  session_id=own["sessionId"], owner=own["attemptId"],
                                  source_digest=source_digest)
    plain = _thaw(manifest.payload)
    coverage = {**intent["coverage"], "sourceDigest": source_digest}
    coverage["coverageDigest"] = digest(coverage)
    final_identity = {**own, "sourceInputDigest": source_digest, "coverageDigest": coverage["coverageDigest"]}
    object_root = _object_root(intent)
    if (not preparation or preparation["payload"].get("status") != "prepared"
            or preparation["payload"].get("result") != _prepared_result(intent, patch_ref, final_identity, source_digest)):
        raise JenkinsError("source_preparation_untrusted", "Durable preparation differs from its own immutable consumer")
    if (any(identity.get(key) != value for key, value in final_identity.items())
            or operation["payload"] != _operation_payload(intent, patch_ref, final_identity, object_root)):
        raise JenkinsError("source_preparation_untrusted", "Preparation consumer identity or payload differs")
    sealed = state.get("sealed_input", source_digest)
    expected_shared = {"status": "sealed", "preparationKind": _KIND, "repositoryId": own["repositoryId"],
                       "repositoryRoot": intent["repositoryRoot"], "buildRoot": intent["buildRoot"],
                       "objectRoot": str(object_root), "sourceDigest": source_digest,
                       "coverage": coverage, "coverageDigest": coverage["coverageDigest"]}
    if not sealed or any(sealed["payload"].get(key) != value for key, value in expected_shared.items()):
        raise JenkinsError("sealed_input_conflict", "Shared sealed source binding differs")
    if (plain.get("preparationKind") != _KIND or plain.get("repositoryId") != own["repositoryId"]
            or plain.get("baseHead") != intent["baseHead"] or plain.get("external") != []
            or plain.get("validationCoverageDigest") != digest(intent["coverage"])
            or plain.get("declaredDependencies") != intent["declaredDependencies"]):
        raise JenkinsError("source_preparation_untrusted", "Manifest is not bound to the complete declared closure")
    manifest_path = physical_path_under(canonical_build_root(intent["buildRoot"]),
                                       object_root / "inputs" / (source_digest + ".json"), allow_missing=False)
    try:
        disk_manifest = json.loads(manifest_path.read_bytes())
    except (OSError, ValueError) as error:
        raise JenkinsError("sealed_input_corrupt", "Persisted source manifest is absent or corrupt") from error
    if disk_manifest != plain or compute_source_digest(disk_manifest) != source_digest:
        raise JenkinsError("sealed_input_corrupt", "Persisted manifest differs from the authoritative sealed record")
    root, tree, cache = _git_tree(intent)
    expected_current = {**intent["afterHashes"], **intent["dependencyHashes"]}
    entries = plain.get("entries")
    if not isinstance(entries, list):
        raise JenkinsError("source_preparation_untrusted", "Manifest entries are malformed")
    by_path, current = {}, {}
    for entry in entries:
        if not isinstance(entry, Mapping):
            raise JenkinsError("source_preparation_untrusted", "Manifest entry is malformed")
        if not isinstance(entry.get("path"), str):
            raise JenkinsError("source_preparation_untrusted", "Manifest entry path is malformed")
        path = _normalized_path(entry["path"])
        if path in by_path or path not in expected_current or entry.get("kind") == "directory":
            raise JenkinsError("source_preparation_untrusted", "Manifest contains an unexpected or aliased entry")
        _, mode, oid = baselines._lookup(root, tree, path, cache)
        if entry.get("baseBlob") != oid:
            raise JenkinsError("baseline_provenance_changed", "Manifest base blob differs from its immutable Git commit")
        expected = expected_current[path]
        before = intent["beforeHashes"].get(path, expected)
        if entry.get("beforeHash") != before:
            raise JenkinsError("source_preparation_untrusted", "Manifest preimage differs from trusted Git or declared dependency")
        if expected is None:
            if (entry.get("status") != "deleted" or entry.get("sha256") is not None
                    or entry.get("size") != 0 or entry.get("objectDigest") is not None):
                raise JenkinsError("source_preparation_untrusted", "Deleted entry has inconsistent bytes")
            current[path] = None
        else:
            if entry.get("status") != "present" or entry.get("sha256") != expected or entry.get("objectDigest") != expected:
                raise JenkinsError("source_preparation_untrusted", "Present entry differs from declared current bytes")
            data = _cas_bytes(object_root, expected)
            if type(entry.get("size")) is not int or entry["size"] != len(data):
                raise JenkinsError("source_preparation_untrusted", "Manifest size differs from CAS bytes")
            current[path] = data
        by_path[path] = entry
    if set(by_path) != set(expected_current):
        raise JenkinsError("source_preparation_untrusted", "Manifest omits declared closure entries")
    _cargo_closure(root, current, inventory=False)
    _verify_cargo_binding(state, intent, source_digest, plain, current)
    # Verify every CAS entry before considering addition/deletion semantics.
    before_bytes, provenance_paths = {}, intent["baselineProvenance"].get("paths", [])
    expected_provenance = {}
    for path, checksum in intent["beforeHashes"].items():
        git_path, mode, oid = baselines._lookup(root, tree, path, cache)
        if checksum is None:
            if oid is not None:
                raise JenkinsError("baseline_provenance_changed", "Claimed addition exists in the trusted commit")
            before_bytes[path] = None
        else:
            original = baselines._object(root, "blob", oid, cache)
            if hashlib.sha256(original).hexdigest() != checksum or _cas_bytes(object_root, checksum) != original:
                raise JenkinsError("before_object_corrupt", "Prepared preimage differs from its immutable Git blob")
            before_bytes[path] = original
        expected_provenance[path] = {"path": path, "git_path": git_path, "mode": mode,
                                     "object_id": oid, "sha256": checksum}
    if provenance_paths != [expected_provenance[path] for path in sorted(expected_provenance)]:
        raise JenkinsError("baseline_provenance_changed", "Typed per-path Git provenance differs")
    rendered = render_patch_bytes(intent["unifiedPatch"], before_bytes, intent["beforeHashes"])
    if any(rendered[path] != current[path] for path in rendered):
        raise JenkinsError("patch_after_mismatch", "Sealed owned bytes differ from the authorized patch")
    expected_result = {"sourceDigest": source_digest, "coverageDigest": coverage["coverageDigest"],
                       "beforeHashes": intent["beforeHashes"], "afterHashes": intent["afterHashes"],
                       "currentHashes": expected_current, "ownedPaths": sorted(intent["beforeHashes"]),
                       "canonicalSourceModified": False}
    if operation.get("result") != expected_result:
        raise JenkinsError("source_preparation_untrusted", "Completed preparation result differs from its complete source closure")
    return operation


def prepare_already_applied(state, context_root: Path, payload: dict, *,
                           trusted_driver_digest: str, policy_root: Path | None = None) -> dict:
    from .patch_context import resolve_patch_checkout
    from .patches import _ensure_patch_metadata, _load_intent, _lock, _runtime_binding
    context = context_root.absolute()
    intent = _load_intent(state, context, payload)
    reference = payload["patchRequestRef"]
    _own_intent(state, reference)
    if intent["identity"]["stageImplementationDigest"] != trusted_driver_digest:
        raise JenkinsError("driver_context_mismatch", "Preparation differs from its registered sealed driver")
    if _runtime_binding(context, intent["identity"], payload) != intent.get("runtimeOperationId"):
        raise JenkinsError("runtime_binding_mismatch", "Preparation changed its registered runtime owner")
    root = resolve_patch_checkout(state, context, intent)
    operation_id = "already-applied-" + reference
    with _lock(root):
        previous = state.get("patch_preparation", reference)
        if previous and previous["payload"].get("status") == "cancelled":
            raise JenkinsError("patch_cancelled", "Cancelled input cannot be prepared or dispatched")
        if previous and previous["payload"].get("status") == "prepared":
            prepared = previous["payload"]["result"]
            require_already_applied_preparation(state, operation_id, identity=prepared["identity"],
                                                source_digest=prepared["sealedInputRef"])
            _ensure_patch_metadata(state, root, intent, {"sourceDigest": prepared["sealedInputRef"]},
                                   policy_root=policy_root or context)
            state.submit_request(_request(intent, prepared))
            return prepared
        try:
            trusted, current = _verify_current(root, intent)
            _cargo_closure(root, current)
            object_root = _object_root(intent)
            claims = [PathClaim(path, "dependency", 0, checksum, 0, intent["identity"]["repositoryId"])
                      for path, checksum in {**intent["afterHashes"], **intent["dependencyHashes"]}.items()]
            candidate = seal_candidate(root, claims, object_root=object_root, base_head=intent["baseHead"],
                                       declared_dependencies=intent["declaredDependencies"])
            plain = _thaw(candidate.payload)
            _, tree, cache = _git_tree(intent)
            for entry in plain["entries"]:
                path = _normalized_path(entry["path"])
                entry["beforeHash"] = intent["beforeHashes"].get(path, entry["sha256"])
                entry["baseBlob"] = baselines._lookup(root, tree, path, cache)[2]
            plain.update(preparationKind=_KIND, validationCoverageDigest=digest(intent["coverage"]))
            source_digest = compute_source_digest(plain)
            for data in trusted.before_bytes.values():
                if data is not None:
                    _write_cas(object_root, data)
            manifest_path = physical_path_under(canonical_build_root(intent["buildRoot"]),
                                               object_root / "inputs" / (source_digest + ".json"))
            encoded = json.dumps(plain, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")
            if manifest_path.exists():
                if manifest_path.read_bytes() != encoded:
                    raise JenkinsError("sealed_input_corrupt", "Existing immutable manifest cannot be replaced")
            else:
                _atomic_write(manifest_path, encoded)
            coverage = {**intent["coverage"], "sourceDigest": source_digest}
            coverage["coverageDigest"] = digest(coverage)
            final_identity = {**intent["identity"], "sourceInputDigest": source_digest,
                              "coverageDigest": coverage["coverageDigest"]}
            shared = {"status": "sealed", "preparationKind": _KIND,
                      "repositoryRoot": str(root), "repositoryId": final_identity["repositoryId"],
                      "sourceDigest": source_digest, "sourceManifest": plain, "manifest": plain,
                      "coverage": coverage, "coverageDigest": coverage["coverageDigest"],
                      "objectRoot": str(object_root), "buildRoot": intent["buildRoot"]}
            old = state.get("sealed_input", source_digest)
            if old:
                if old["payload"] != shared:
                    raise JenkinsError("sealed_input_conflict", "Shared sealed source is immutable")
            else:
                state.put("sealed_input", source_digest, shared, expected_version=0)
            register_sealed_consumer(state, repository_id_value=final_identity["repositoryId"],
                                    session_id=final_identity["sessionId"], owner=final_identity["attemptId"],
                                    source_digest=source_digest, manifest_digest=source_digest)
            operation_payload = _operation_payload(intent, reference, final_identity, object_root)
            operation = state.put_operation("source_preparation", operation_id, operation_payload)
            if operation["status"] not in {"prepared", "complete"}:
                raise JenkinsError("source_preparation_untrusted", "Preparation has an unresolved operation outcome")
            _ensure_patch_metadata(state, root, intent, {"sourceDigest": source_digest},
                                   policy_root=policy_root or context)
            _verify_current(root, intent)
            result = {"sourceDigest": source_digest, "coverageDigest": coverage["coverageDigest"],
                      "beforeHashes": intent["beforeHashes"], "afterHashes": intent["afterHashes"],
                      "currentHashes": {**intent["afterHashes"], **intent["dependencyHashes"]},
                      "ownedPaths": sorted(intent["beforeHashes"]), "canonicalSourceModified": False}
            if operation["status"] == "complete":
                if operation["result"] != result:
                    raise JenkinsError("source_preparation_untrusted", "Completed preparation changed its outcome")
            else:
                state.transition_operation(operation_id, "prepared", "complete", result)
            prepared = _prepared_result(intent, reference, final_identity, source_digest)
            state.put("patch_preparation", reference, {"status": "prepared", "result": prepared},
                      expected_version=previous["version"] if previous else 0)
            require_already_applied_preparation(state, operation_id, identity=final_identity, source_digest=source_digest)
            state.submit_request(_request(intent, prepared))
            return prepared
        except BaseException as error:
            latest = state.get("patch_preparation", reference)
            if not latest or latest["payload"].get("status") != "prepared":
                state.put("patch_preparation", reference,
                          {"status": "failed", "operationId": operation_id,
                           "reasonCode": getattr(error, "code", type(error).__name__),
                           "canonicalSourceModified": False},
                          expected_version=latest["version"] if latest else 0)
            raise


def _cancel_guard(state, intent, reference: str, preparation) -> None:
    refs = {reference, "already-applied-" + reference}
    if preparation and preparation["payload"].get("result"):
        refs.add(preparation["payload"]["result"]["sealedInputRef"])
    own = intent["identity"]

    def belongs(value) -> bool | None:
        if not isinstance(value, Mapping):
            return None
        identity = value.get("identity", value)
        keys = ("repositoryId", "sessionId", "requestId")
        if isinstance(identity, Mapping) and all(isinstance(identity.get(key), str) for key in keys):
            if all(identity[key] == own[key] for key in keys):
                return True
            return False
        if any(isinstance(value.get(key), str) and value[key] in refs
               for key in ("patchRequestRef", "sourcePreparationRef", "sourceInputDigest",
                           "sealedInputRef", "sourceDigest", "inputDigest", "operationId")):
            return True
        if isinstance(value.get("executionId"), str):
            execution = state.get("execution", value["executionId"])
            return belongs(execution["payload"]) if execution else None
        return None

    for domain in ("execution", "execution_consumer", "native_job", "workflow", "writer_reservation"):
        for row in state.list(domain):
            value = row["payload"]
            binding = belongs(value)
            if binding is False:
                continue
            status = value.get("status")
            if domain == "native_job":
                from .deployment.drain_observations import native_terminal
                if not native_terminal(row):
                    raise JenkinsError("shared_source_active", "Native outcome is not proven terminal for cancellation")
            elif domain == "execution":
                from .deployment.drain_observations import native_terminal
                native = [job for job in state.list("native_job")
                          if job["payload"].get("executionId") in {row["key"], value.get("executionId")}]
                if (status not in {"succeeded", "failed", "cancelled", "complete", "terminal"}
                        or not native or any(not native_terminal(job) for job in native)):
                    raise JenkinsError("shared_source_active", "Execution has no complete native termination proof for cancellation")
            elif domain == "execution_consumer" and status not in {"released", "terminal", "complete"}:
                raise JenkinsError("shared_source_active", "Execution consumer still references this input")
            elif domain == "workflow" and status not in {"failed", "cancelled", "accepted", "complete"}:
                raise JenkinsError("shared_source_active", "Workflow still references this input")
            elif domain == "writer_reservation" and status not in {"released", "terminal"}:
                raise JenkinsError("shared_source_active", "Writer ownership is unresolved")


def reconcile_already_applied(state, context_root: Path, payload: dict, *,
                             trusted_driver_digest: str | None = None, cancel: bool = False) -> dict:
    from .patch_context import resolve_patch_checkout
    from .patches import _load_intent, _lock, _runtime_binding
    context = context_root.absolute()
    intent = _load_intent(state, context, payload)
    reference = payload["patchRequestRef"]
    _own_intent(state, reference)
    if trusted_driver_digest is not None and trusted_driver_digest != intent["identity"]["stageImplementationDigest"]:
        raise JenkinsError("driver_context_mismatch", "Recovery differs from its trusted sealed driver")
    if _runtime_binding(context, intent["identity"], payload) != intent.get("runtimeOperationId"):
        raise JenkinsError("runtime_binding_mismatch", "Recovery changed its registered runtime owner")
    if not cancel:
        trusted = trusted_driver_digest or intent["identity"]["stageImplementationDigest"]
        return prepare_already_applied(state, context, payload, trusted_driver_digest=trusted, policy_root=context)
    root = resolve_patch_checkout(state, context, intent)
    with _lock(root):
        previous = state.get("patch_preparation", reference)
        if previous and previous["payload"].get("status") == "cancelled":
            return response("cancelled", canonicalSourceModified=False, patchRequestRef=reference, reused=True)
        _cancel_guard(state, intent, reference, previous)
        identity = intent["identity"]
        request = state.get_request(identity["repositoryId"], identity["sessionId"], identity["requestId"])
        if request and request["status"] not in {"pending", "waiting", "blocked", "reconciling", "cancelled"}:
            raise JenkinsError("shared_source_active", "Request is running or has a terminal outcome that cannot be rewritten")
        if request and request["status"] != "cancelled":
            state.transition_request(identity["repositoryId"], identity["sessionId"], identity["requestId"], 1,
                                     request["status"], "cancelled", {"canonicalSourceModified": False})
        state.put("patch_preparation", reference,
                  {**(previous["payload"] if previous else {}), "status": "cancelled",
                   "canonicalSourceModified": False},
                  expected_version=previous["version"] if previous else 0)
        return response("cancelled", canonicalSourceModified=False, patchRequestRef=reference, reused=False)


__all__ = ["register_already_applied_intent", "prepare_already_applied",
           "reconcile_already_applied", "require_already_applied_preparation"]

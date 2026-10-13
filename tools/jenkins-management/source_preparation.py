"""Jenkins source sealing and shared preparation for ordinary management tasks.

This module is part of the pinned management capsule. It reuses the formal CAS,
sealed-consumer and preparation-pool authorities; it never emits acceptance.
"""
from __future__ import annotations

from collections.abc import Mapping
import hashlib
import json
from pathlib import Path
import platform
import re
import sys
import time


def encoded(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")


def digest(value):
    return hashlib.sha256(encoded(value)).hexdigest()


def source_files(snapshot, prefix=""):
    """Flatten registered submodule bytes without copying Git control files."""
    result = {}
    for name, value in snapshot["files"].items():
        path = prefix + name
        if (not isinstance(name, str) or not name or "\\" in name or ":" in name
                or name.startswith("/") or any(part in {"", ".", ".."} for part in name.split("/"))
                or any(part.casefold() == ".git" for part in name.split("/"))):
            raise ValueError("Noncanonical source file")
        if isinstance(value, dict):
            if set(value) != {"gitlinkCommit", "source"}:
                raise ValueError("Unregistered source directory")
            result.update(source_files(value["source"], path + "/"))
        elif value is None or isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value):
            if path in result:
                raise ValueError("Duplicate flattened source path")
            result[path] = value
        else:
            raise ValueError("Source file lacks its actual hash")
    return result


def preparation_key(repository_id, trusted, environment, packages, features, profile,
                    no_default_features, dependency_cache, external_names):
    """Compatibility identity excludes producer worktree and session paths."""
    replaced = {"TEMP", "TMP", "TMPDIR", "CARGO_TARGET_DIR", "CARGO_BUILD_BUILD_DIR",
                "CARGO_HOME", "RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER",
                "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_NET_OFFLINE"}
    return digest({
        "schemaVersion": 1, "repositoryId": repository_id,
        "host": {"platform": sys.platform, "architecture": platform.machine()},
        "target": environment.get("CARGO_BUILD_TARGET", "toolchain-host"),
        "trustedToolsAndPolicy": trusted,
        "environment": {key: value for key, value in environment.items() if key.upper() not in replaced},
        "compilerWrappers": {"rustc": "", "workspace": ""},
        "compilerFlags": {"rustflags": "", "encodedRustflags": "", "rustdocflags": ""},
        "packages": sorted(set(packages)), "features": sorted(set(features)),
        "noDefaultFeatures": no_default_features, "profile": profile,
        "dependencyCache": dependency_cache, "externalLayout": sorted(external_names),
    })


def verify_sealed_capture(captured, manifest):
    """Bind every sealed byte, deletion and base to the exact declared capture."""
    expected = {path.casefold(): checksum for path, checksum in source_files(captured).items()}
    entries = manifest["entries"]
    actual = {}
    for entry in entries:
        if entry["status"] == "present" and entry.get("objectDigest") == entry["sha256"]:
            actual[entry["path"].casefold()] = entry["sha256"]
        elif entry["status"] == "deleted" and entry["sha256"] is None:
            actual[entry["path"].casefold()] = None
        else:
            raise ValueError("Sealed entry differs from its captured source bytes")
    if (manifest["baseHead"] != captured["head"] or len(actual) != len(entries)
            or actual != expected):
        raise ValueError("Sealed source inventory differs from the complete declared capture")


def _reusable_sealed_capture(state, *, source_root, snapshot, repository_root,
                             repository_id, approved, objects):
    """Reuse only a complete captured input with its original producer binding."""
    from tools.jenkins.source import require_sealed_consumer, repository_id as physical_repository_id
    from tools.jenkins.source.external_inputs import sources, flattened, verify_objects
    from tools.jenkins.resources.paths import canonical_build_root, physical_path_under
    from tools.jenkins.workflow import pools

    # Execution records are small references. Do not load every historical
    # sealed_input manifest merely to find the same complete capture.
    for row in state.list("execution"):
        execution = row["payload"]
        if (execution.get("kind") != "management_owned_cargo"
                or execution.get("repositoryId") != repository_id
                or execution.get("sourceRoot") != str(source_root)
                or execution.get("capturedSourceDigest") != snapshot["sourceDigest"]):
            continue
        producer = row["key"]
        reference = execution.get("sealedInputRef")
        session = execution.get("sessionId")
        if (not producer.startswith("management-") or not isinstance(session, str) or not session
                or not isinstance(reference, str) or not re.fullmatch(r"[0-9a-f]{64}", reference)
                or execution.get("sourceDigest") != reference):
            raise ValueError("Captured preparation producer identity differs")
        previous = state.get("sealed_input", reference)
        if not previous:
            raise ValueError("Captured preparation sealed input is missing")
        payload = previous["payload"]
        build_root = payload.get("buildRoot")
        if not isinstance(build_root, str) or not build_root:
            raise ValueError("Captured preparation build root identity differs")
        candidate_root = canonical_build_root(build_root)
        if candidate_root.path != approved.path:
            # A legal foreign store is not an input for the selected root.
            # Check its physical binding without reading or copying its CAS.
            foreign_objects = physical_path_under(candidate_root, payload.get("objectRoot", ""), allow_missing=False)
            foreign_namespace = physical_path_under(candidate_root, candidate_root.namespace(), allow_missing=False)
            if foreign_objects != foreign_namespace:
                raise ValueError("Foreign preparation object namespace differs")
            continue
        captured = state.get("management_source", producer[len("management-"):])
        if not captured or captured["payload"] != snapshot:
            raise ValueError("Preparation producer differs from the complete captured input")
        manifest = payload.get("sourceManifest")
        if (payload.get("status") != "sealed" or payload.get("sourceDigest") != reference
                or payload.get("repositoryId") != repository_id
                or payload.get("repositoryRoot") != str(repository_root)
                or not isinstance(manifest, Mapping) or payload.get("manifest") != manifest
                or digest(manifest) != reference or manifest.get("schemaVersion") != 1
                or manifest.get("repositoryId") != repository_id
                or manifest.get("preparationKind") != "current-source"
                or manifest.get("declaredDependencies") != []):
            raise ValueError("Captured preparation sealed manifest differs")
        coverage_digest = digest(payload.get("coverage"))
        if (payload.get("coverageDigest") != coverage_digest
                or manifest.get("validationCoverageDigest") != coverage_digest):
            raise ValueError("Captured preparation coverage identity differs")
        consumer = require_sealed_consumer(state, repository_id_value=repository_id,
            session_id=session, owner=producer, source_digest=reference, manifest_digest=reference)["payload"]
        binding = {"repositoryId": repository_id, "sessionId": session, "owner": producer,
                   "sourceDigest": reference, "manifestDigest": reference}
        if any(consumer.get(name) != value for name, value in binding.items()):
            raise ValueError("Captured preparation producer consumer differs")
        actual_objects = physical_path_under(approved, payload.get("objectRoot", ""), allow_missing=False)
        expected_objects = physical_path_under(approved, objects, allow_missing=False)
        if (actual_objects != expected_objects
                or manifest.get("selectedObjectNamespace", str(expected_objects)) != str(expected_objects)):
            raise ValueError("Captured preparation object namespace differs")

        external = sources(manifest)
        if len(external) != len(snapshot["externalRoots"]):
            raise ValueError("Captured preparation external closure differs")
        verify_sealed_capture(snapshot["source"], manifest)
        for captured_external, sealed_external in zip(snapshot["externalRoots"], external):
            root = Path(captured_external["path"])
            dependency = sealed_external["manifest"]
            if (root != repository_root.parent / "zr_vm"
                    or sealed_external.get("repositoryRoot") != str(root)
                    or dependency.get("schemaVersion") != 1
                    or dependency.get("repositoryId") != physical_repository_id(root)
                    or dependency.get("declaredDependencies") != []):
                raise ValueError("Captured preparation external identity differs")
            verify_sealed_capture(captured_external["source"], dependency)
        # Validate original paths before prefixing them into the sibling layout.
        # Then retain the formal physical-path and every-byte CAS checks before
        # publishing any new metadata or consumer reference.
        for value in [manifest, *[item["manifest"] for item in external]]:
            if not isinstance(value.get("entries"), list):
                raise ValueError("Captured preparation manifest entries differ")
            for entry in value["entries"]:
                if not isinstance(entry, Mapping):
                    raise ValueError("Captured preparation manifest entry differs")
                pools._relative(entry.get("path"))
        verify_objects(manifest, actual_objects)
        pools._manifest(flattened(manifest), actual_objects, approved)
        return previous
    return None


def seal_source(state, *, source_root, snapshot, repository_root, repository_id,
                session_id, owner, approved, coverage):
    """Publish exact captured bytes through the existing formal input CAS."""
    from tools.jenkins.source import PathClaim, seal_candidate, register_sealed_consumer, _thaw
    from tools.jenkins.source.external_inputs import KIND
    from tools.jenkins.resources.paths import canonical_build_root, physical_path_under

    started = time.perf_counter()
    source_root, repository_root = Path(source_root), Path(repository_root)
    if snapshot.get("sourceRoot") != str(source_root):
        raise ValueError("Captured source root differs")
    objects = approved.namespace()
    reusable = _reusable_sealed_capture(state, source_root=source_root, snapshot=snapshot,
        repository_root=repository_root, repository_id=repository_id, approved=approved, objects=objects)
    reused_reference = reusable["key"] if reusable else None
    if reusable:
        print("[management] Reusing verified captured input " + reused_reference, flush=True)
        manifest = reusable["payload"]["sourceManifest"]
        external = manifest["external"]
    else:
        # Seal the smaller foreign dependency before the large owned engine. Its
        # mutable checkout is no longer an execution input after exact CAS sealing.
        external = []
        for item in snapshot["externalRoots"]:
            root = Path(item["path"])
            if root != repository_root.parent / "zr_vm":
                raise ValueError("External source differs from the canonical zr_vm dependency")
            captured = item["source"]
            claims = [PathClaim(path, "dependency", 0, checksum, 0)
                      for path, checksum in source_files(captured).items()]
            print("[management] Sealing external input " + str(root), flush=True)
            dependency = seal_candidate(root, claims, object_root=objects, base_head=captured["head"])
            dependency_manifest = _thaw(dependency.payload)
            verify_sealed_capture(captured, dependency_manifest)
            external.append({"kind": KIND, "name": "zr_vm", "repositoryRoot": str(root),
                             "manifest": dependency_manifest, "sourceDigest": dependency.source_digest})
        engine = snapshot["source"]
        claims = [PathClaim(path, owner, 0, checksum, 0, repository_id)
                  for path, checksum in source_files(engine).items()]
        print("[management] Sealing owned input " + str(source_root), flush=True)
        sealed = seal_candidate(source_root, claims, object_root=objects, base_head=engine["head"])
        manifest = _thaw(sealed.payload)
        verify_sealed_capture(engine, manifest)
    manifest = {**manifest, "repositoryId": repository_id, "preparationKind": "current-source",
                "validationCoverageDigest": digest(coverage), "external": external}
    source_digest = digest(manifest)
    previous = reusable if reused_reference == source_digest else state.get("sealed_input", source_digest)
    if previous and reusable is None:
        prior = previous["payload"]
        prior_build = prior.get("buildRoot")
        if not isinstance(prior_build, str) or not prior_build:
            raise ValueError("Existing sealed build root identity differs")
        prior_root = canonical_build_root(prior_build)
        if prior_root.path != approved.path:
            # The legacy digest is already bound to another physical store.
            # Preserve it; only this cold input gains a namespace-bound identity.
            if (prior.get("status") != "sealed" or prior.get("sourceDigest") != source_digest
                    or prior.get("repositoryId") != repository_id
                    or prior.get("repositoryRoot") != str(repository_root)
                    or prior.get("sourceManifest") != manifest or prior.get("manifest") != manifest
                    or digest(prior["sourceManifest"]) != source_digest
                    or prior.get("coverage") != coverage or prior.get("coverageDigest") != digest(coverage)):
                raise ValueError("Foreign sealed source collision differs")
            prior_objects = physical_path_under(prior_root, prior.get("objectRoot", ""), allow_missing=False)
            prior_namespace = physical_path_under(prior_root, prior_root.namespace(), allow_missing=False)
            if (prior_objects != prior_namespace
                    or manifest.get("selectedObjectNamespace", str(prior_namespace)) != str(prior_namespace)):
                raise ValueError("Foreign sealed source namespace differs")
            manifest = {**manifest, "selectedObjectNamespace": str(objects)}
            source_digest = digest(manifest)
            previous = state.get("sealed_input", source_digest)
    record = {"status": "sealed", "repositoryRoot": str(repository_root),
              "repositoryId": repository_id, "sourceDigest": source_digest,
              "sourceManifest": manifest, "manifest": manifest,
              "objectRoot": str(objects), "buildRoot": str(approved.path),
              "coverage": coverage, "coverageDigest": digest(coverage)}
    if previous:
        if previous["payload"] != record:
            raise ValueError("Existing sealed source differs")
    else:
        state.put("sealed_input", source_digest, record, expected_version=0)
    register_sealed_consumer(state, repository_id_value=repository_id,
                            session_id=session_id, owner=owner,
                            source_digest=source_digest, manifest_digest=source_digest)
    return {"sealedInputRef": source_digest, "sourceManifest": manifest,
            "sourceRoot": str(source_root), "capturedSourceDigest": snapshot["sourceDigest"],
            "capturedInputVerified": True, "coverageDigest": digest(coverage),
            "metrics": {"elapsedSeconds": time.perf_counter() - started,
                        "reused": reusable is not None, "reusedSealedInputRef": reused_reference,
                        "rootOrder": [value["repositoryRoot"] for value in external] + [str(source_root)],
                        "fileCount": sum(entry["status"] == "present" for entry in manifest["entries"])
                                     + sum(entry["status"] == "present" for value in external
                                           for entry in value["manifest"]["entries"]),
                        "sourceBytes": sum(entry.get("size", 0) for entry in manifest["entries"])
                                       + sum(entry.get("size", 0) for value in external
                                             for entry in value["manifest"]["entries"]),
                        "objectRoot": str(objects / "inputs" / "objects")}}


def materialize_source(state, *, approved, owner, preparation_key_value, source_path,
                       reservation_key, sealed, identity, command, environment_digest):
    """Update the existing formal pool journal, preserving unchanged mtimes."""
    from tools.jenkins.workflow import pools
    from tools.jenkins.source.external_inputs import sources, flattened, verify_objects
    from tools.jenkins.resources.paths import physical_path_under

    started = time.perf_counter()
    execution = {**identity, "kind": "management_owned_cargo", "status": "preparing",
                 "preparationKey": preparation_key_value, "preparationRoot": str(source_path),
                 "sealedInputRef": sealed["sealedInputRef"], "sourceDigest": sealed["sealedInputRef"],
                 "reservationKey": reservation_key, "sourceRoot": sealed["sourceRoot"],
                 "capturedSourceDigest": sealed["capturedSourceDigest"],
                 "command": command, "environmentDigest": environment_digest,
                 "formalAcceptance": False}
    if state.get("execution", owner):
        raise ValueError("Management execution already exists; reconcile its original operation")
    state.put("execution", owner, execution, expected_version=0)
    pools._hold(state, owner, preparation_key_value)
    source = physical_path_under(approved, source_path, allow_missing=True)
    pools._check_tree(source)
    record = state.get("sealed_input", sealed["sealedInputRef"])
    if not record or record["payload"].get("sourceManifest") != sealed["sourceManifest"]:
        raise ValueError("Actual sealed preparation input differs")
    objects = physical_path_under(approved, record["payload"]["objectRoot"], allow_missing=False)
    if objects != physical_path_under(approved, approved.namespace(), allow_missing=False):
        raise ValueError("Sealed objects differ from this build namespace")
    manifest = sealed["sourceManifest"]
    if digest(manifest) != sealed["sealedInputRef"]:
        raise ValueError("Sealed manifest digest differs")
    destination = source
    if sources(manifest):
        # Prefixing must not hide an invalid original engine schema or path.
        entries = manifest.get("entries")
        if not isinstance(entries, list):
            raise pools._error("sealed_manifest_missing", "sealed input manifest entries are required")
        for entry in entries:
            if not isinstance(entry, Mapping):
                raise pools._error("manifest_entry_invalid", "sealed input entry is not an object")
            pools._relative(entry.get("path"))
        if source.name != "source":
            raise ValueError("Sibling dependency requires the fixed inputs/source layout")
        verify_objects(manifest, objects)
        destination = physical_path_under(approved, source.parent)
        normalized, desired = pools._manifest(flattened(manifest), objects, approved)
        pools._check_tree(destination)
    else:
        normalized, desired = pools._manifest(manifest, objects, approved)
    def prune_empty_owned_parents(paths):
        # File deletions may leave parents previously owned by this exact
        # manifest. Remove only those parents, only while physically empty.
        for relative in sorted(paths):
            parent = physical_path_under(approved, destination / relative).parent
            while parent != destination:
                physical_path_under(approved, parent)
                try:
                    parent.rmdir()
                except OSError:
                    break
                parent = parent.parent
    key = preparation_key_value
    pools._recover(state, key, destination, objects)
    recovered = state.get("preparation_update", key)
    if recovered and recovered["payload"].get("status") == "complete":
        journal = recovered["payload"]
        prune_empty_owned_parents(set(journal.get("oldOwnedFiles", {})) - set(journal.get("ownedFiles", {})))
    previous_row = state.get("preparation_source", key)
    previous = previous_row["payload"] if previous_row else None
    old = previous.get("ownedFiles", {}) if previous else {}
    actual = pools._inventory(destination, old if previous else None)
    if previous and previous["sourceDigest"] == sealed["sealedInputRef"]:
        if actual != desired:
            raise ValueError("Same source identity differs from the actual pool")
        return {"physicalPath": str(source), "reused": True, "incremental": False,
                "elapsedSeconds": time.perf_counter() - started,
                "changedFileCount": 0, "preservedFileCount": len(desired),
                "sourceDigest": sealed["sealedInputRef"], "preparationKey": key,
                "sealedInputRef": sealed["sealedInputRef"], "sourceRoot": sealed["sourceRoot"],
                "capturedSourceDigest": sealed["capturedSourceDigest"], "formalAcceptance": False}
    # Every previous file was checked by _inventory. Keep all paths in the
    # journal because _recover verifies them after an interrupted update.
    files = [{"path": path, "oldHash": old.get(path), "newHash": desired.get(path),
              "applied": old.get(path) == desired.get(path)} for path in sorted(set(old) | set(desired))]
    intent = {"executionId": owner, "preparationKey": key, "sourceDigest": sealed["sealedInputRef"],
              "oldSourceDigest": previous.get("sourceDigest") if previous else None,
              "status": "updating", "files": files, "oldOwnedFiles": old,
              "ownedFiles": desired, "manifest": normalized, "objectRoot": str(objects),
              "manifestDigest": digest(normalized), "createdAt": time.time()}
    state.put("preparation_update", key, intent, expected_version=None)
    changed = 0
    for item in files:
        target = physical_path_under(approved, destination / item["path"])
        if pools._hash(target) != item["oldHash"]:
            raise ValueError("Preparation file changed after its verified inventory")
        if item["oldHash"] == item["newHash"]:
            # Neither replace nor touch a file whose bytes already match.
            continue
        if item["newHash"] is None:
            if item["path"] not in old or not target.is_file():
                raise ValueError("Preparation deletion lacks its owned preimage")
            target.unlink()
        else:
            pools._atomic_copy(objects / "inputs" / "objects" / item["newHash"], target)
        if pools._hash(target) != item["newHash"]:
            raise ValueError("Prepared output differs from its exact new hash")
        item["applied"] = True
        changed += 1
        # _recover verifies every journaled path against old/new hashes, even
        # when a copy finished before its progress bit was checkpointed.
        # Bound full-journal writes instead of rewriting it for every file.
        if changed % 512 == 0:
            row = state.get("preparation_update", key)
            state.put("preparation_update", key, {**intent, "files": files}, expected_version=row["version"])
    prune_empty_owned_parents(set(old) - set(desired))
    if pools._inventory(destination, desired) != desired:
        raise ValueError("Completed pool differs from its exact source closure")
    row = state.get("preparation_update", key)
    state.put("preparation_update", key, {**intent, "status": "complete", "files": files,
              "completedAt": time.time()}, expected_version=row["version"])
    state.put("preparation_source", key, {"executionId": owner, "preparationKey": key,
              "sourceDigest": sealed["sealedInputRef"], "manifest": normalized,
              "objectRoot": str(objects), "ownedFiles": desired,
              "physicalPath": str(destination), "updatedAt": time.time()})
    return {"physicalPath": str(source), "reused": False, "incremental": bool(previous),
            "elapsedSeconds": time.perf_counter() - started,
            "changedFileCount": changed,
            "preservedFileCount": sum(old.get(path) == checksum for path, checksum in desired.items()),
            "sourceDigest": sealed["sealedInputRef"], "preparationKey": key,
            "sealedInputRef": sealed["sealedInputRef"], "sourceRoot": sealed["sourceRoot"],
            "capturedSourceDigest": sealed["capturedSourceDigest"], "formalAcceptance": False}


def verify_source(manifest, source_path, approved):
    """Hash the entire actual execution layout, including its sealed sibling."""
    from tools.jenkins.contracts import file_digest
    from tools.jenkins.resources.paths import physical_path_under
    from tools.jenkins.source.external_inputs import sources, flattened

    source = physical_path_under(approved, source_path, allow_missing=False)
    with_external = bool(sources(manifest))
    root = physical_path_under(approved, source.parent if with_external else source, allow_missing=False)
    expected_manifest = flattened(manifest) if with_external else manifest
    expected = {entry["path"]: entry["sha256"] for entry in expected_manifest["entries"]
                if entry["status"] == "present"}
    actual = {}
    for path in root.rglob("*"):
        physical_path_under(approved, path, allow_missing=False)
        if path.is_file():
            actual[path.relative_to(root).as_posix()] = file_digest(path)
    if actual != expected:
        raise ValueError("Actual prepared source bytes differ from the sealed execution input")
    return {"physicalPath": str(source), "sourceDigest": digest(manifest),
            "filesDigest": digest(actual), "fileCount": len(actual)}

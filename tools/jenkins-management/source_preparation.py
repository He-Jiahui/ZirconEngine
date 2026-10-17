"""Jenkins source sealing and shared preparation for ordinary management tasks.

This module is part of the pinned management capsule. It reuses the formal CAS,
sealed-consumer and preparation-pool authorities; it never emits acceptance.
"""
from __future__ import annotations

from collections.abc import Mapping
import hashlib
import json
import os
import stat
from pathlib import Path
import platform
import re
import sys
import time


def encoded(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")


def digest(value):
    return hashlib.sha256(encoded(value)).hexdigest()



def _index_sysroot_path(folded_paths, relative):
    """Index one relative sysroot name while preserving case-alias rejection."""
    folded = relative.casefold()
    prior = folded_paths.get(folded)
    if prior is not None and prior != relative:
        raise ValueError("Toolchain sysroot contains a case alias")
    folded_paths[folded] = relative


def trusted_toolchain_facts(trusted, file_fact, *, environment=None):
    """Capture the exact policy-pinned toolchain and its complete local inputs."""
    if not isinstance(trusted, Mapping) or not callable(file_fact):
        raise ValueError("Trusted toolchain policy is unavailable")
    host = trusted.get("host")
    if not isinstance(host, str) or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", host):
        raise ValueError("Trusted toolchain host is invalid")
    if environment is not None and not isinstance(environment, Mapping):
        raise ValueError("Trusted toolchain environment is invalid")

    def same_path(left, right):
        return os.path.normcase(os.path.abspath(os.fspath(left))) == os.path.normcase(os.path.abspath(os.fspath(right)))

    cargo_value, rustc_value = trusted.get("cargoPath"), trusted.get("rustcPath")
    if not isinstance(cargo_value, str) or not isinstance(rustc_value, str):
        raise ValueError("Trusted Cargo and rustc paths are required")
    cargo_path, rustc_path = Path(cargo_value), Path(rustc_value)
    if (not cargo_path.is_absolute() or not rustc_path.is_absolute()
            or cargo_path.name.casefold() != "cargo.exe" or rustc_path.name.casefold() != "rustc.exe"
            or cargo_path.parent.name.casefold() != "bin" or not same_path(cargo_path.parent, rustc_path.parent)):
        raise ValueError("Trusted compiler paths must share one absolute toolchain bin directory")
    root = cargo_path.parent.parent
    if root.parent.name.casefold() != "toolchains" or not root.name or not re.fullmatch(
            r"[A-Za-z0-9][A-Za-z0-9._+-]*", root.name):
        raise ValueError("Trusted compiler path is not under one Rustup toolchain root")
    rustup_home = root.parent.parent
    if environment is not None:
        configured_home = environment.get("RUSTUP_HOME")
        if configured_home not in (None, ""):
            if not isinstance(configured_home, str) or not same_path(configured_home, rustup_home):
                raise ValueError("RUSTUP_HOME conflicts with the trusted toolchain root")
        configured_toolchain = environment.get("RUSTUP_TOOLCHAIN")
        if configured_toolchain not in (None, "", root.name):
            raise ValueError("RUSTUP_TOOLCHAIN conflicts with the trusted toolchain root")

    def require_plain(path):
        path = Path(path)
        if not path.is_absolute():
            raise ValueError("Toolchain inputs must use absolute paths")
        for ancestor in (path, *path.parents):
            info = ancestor.lstat()
            if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
                raise ValueError("Toolchain inputs cannot cross filesystem links")
        return path

    plain_files = {}
    folded = {}

    def add_file(path, expected_sha=None):
        path = require_plain(path)
        fact = file_fact(path)
        if not isinstance(fact, Mapping) or set(("path", "sha256", "byteLength")) - set(fact):
            raise ValueError("Toolchain file fact is incomplete")
        if fact["path"] != str(path) or not isinstance(fact["sha256"], str) or not re.fullmatch(
                r"[0-9a-f]{64}", fact["sha256"]):
            raise ValueError("Toolchain file fact identity is invalid")
        if type(fact["byteLength"]) is not int or fact["byteLength"] < 0:
            raise ValueError("Toolchain file fact length is invalid")
        if expected_sha is not None and fact["sha256"] != expected_sha:
            raise ValueError("Installed compiler differs from the trusted toolchain")
        prior = folded.get(str(path).casefold())
        if prior is not None and prior != str(path):
            raise ValueError("Toolchain input paths contain a case alias")
        folded[str(path).casefold()] = str(path)
        plain_files[str(path)] = dict(fact)
        return fact["sha256"]

    binaries = {}
    for name in ("cargo", "rustc", "rustdoc"):
        path = root / "bin" / (name + ".exe")
        if name == "cargo" and not same_path(path, cargo_path):
            raise ValueError("Trusted Cargo path differs from its toolchain root")
        if name == "rustc" and not same_path(path, rustc_path):
            raise ValueError("Trusted rustc path differs from its toolchain root")
        expected_sha = trusted.get(name + "Sha256") if name in {"cargo", "rustc"} else None
        if not isinstance(expected_sha, str) or not re.fullmatch(r"[0-9a-f]{64}", expected_sha):
            if name in {"cargo", "rustc"}:
                raise ValueError("Trusted compiler hash is missing")
        identity_path = cargo_value if name == "cargo" else rustc_value if name == "rustc" else str(path)
        binaries[name] = {"path": identity_path, "sha256": add_file(path, expected_sha)}

    bin_directory = require_plain(root / "bin")
    if not bin_directory.is_dir():
        raise ValueError("Trusted toolchain bin directory is unavailable")
    libraries = {}
    for path in sorted(bin_directory.iterdir(), key=lambda item: item.name.casefold()):
        path = require_plain(path)
        if path.is_dir():
            continue
        if not path.is_file():
            raise ValueError("Toolchain bin contains an unsupported filesystem entry")
        if path.suffix.casefold() == ".dll":
            libraries[path.relative_to(root).as_posix()] = add_file(path)

    sysroot_inputs = {}
    folded_sysroot_paths = {}
    relative_roots = (Path("lib/rustlib") / host / "lib",
                      Path("lib/rustlib") / host / "bin", Path("lib/rustlib/bin"))
    for relative_root in relative_roots:
        directory = root / relative_root
        try:
            require_plain(directory)
        except FileNotFoundError:
            continue
        if not directory.is_dir():
            raise ValueError("Trusted toolchain sysroot input is not a directory")
        pending = [directory]
        while pending:
            current = pending.pop()
            for path in sorted(current.iterdir(), key=lambda item: item.name.casefold(), reverse=True):
                path = require_plain(path)
                if path.is_dir():
                    pending.append(path)
                elif path.is_file():
                    relative = path.relative_to(root).as_posix()
                    _index_sysroot_path(folded_sysroot_paths, relative)
                    sysroot_inputs[relative] = add_file(path)
                else:
                    raise ValueError("Toolchain sysroot contains an unsupported filesystem entry")
    if not any(name.casefold().startswith((Path("lib/rustlib") / host / "lib").as_posix().casefold() + "/")
               for name in sysroot_inputs):
        raise ValueError("Installed host sysroot libraries are required")

    python_path = require_plain(Path(sys.executable))
    python_sha = add_file(python_path)
    settings = rustup_home / "settings.toml"
    settings_sha = add_file(settings)
    return {"channel": root.name, "host": host, "binaries": binaries,
            "libraries": dict(sorted(libraries.items())),
            "sysrootInputs": dict(sorted(sysroot_inputs.items())),
            "python": {"path": str(python_path), "sha256": python_sha},
            "settingsSha256": settings_sha, "plainFileFacts": plain_files}


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
                session_id, owner, approved, coverage, management=None):
    """Publish exact captured bytes through the existing formal input CAS."""
    from tools.jenkins.source import PathClaim, seal_candidate, register_sealed_consumer, _thaw
    from tools.jenkins.source.external_inputs import KIND
    from tools.jenkins.resources.paths import canonical_build_root, physical_path_under

    started = time.perf_counter()
    source_root, repository_root = Path(source_root), Path(repository_root)
    if snapshot.get("sourceRoot") != str(source_root):
        raise ValueError("Captured source root differs")
    objects = approved.namespace()
    prepared_context = None
    if snapshot.get("externalInputPolicy") == "prepared-sealed-CAS":
        if management is None or management.repo_id != repository_id or management.session != session_id:
            raise ValueError("Prepared sealing requires the actual management authority")
        engine = management.source(source_root, payload=management.payload)
        prepared_context = prepared_validation_context(management, source_root, engine)
        if (engine != snapshot["source"] or prepared_context["captures"] != snapshot["externalRoots"]
                or prepared_context["provenance"] != snapshot.get("preparedSealedInput")
                or prepared_context["objects"] != objects):
            raise ValueError("Prepared sealing differs from its original verified capture")
    reusable = None if prepared_context is not None else _reusable_sealed_capture(state,
        source_root=source_root, snapshot=snapshot, repository_root=repository_root,
        repository_id=repository_id, approved=approved, objects=objects)
    reused_reference = reusable["key"] if reusable else None
    if prepared_context is not None:
        manifest = prepared_execution_manifest(management, source_root, engine, prepared_context)
        external = manifest["external"]
    elif reusable:
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


def _pool_path_index(paths):
    """Map unique logical file identities back to their original manifest keys."""
    from tools.jenkins.workflow import pools
    result = {}
    for path in paths:
        relative = pools._relative(path)
        if relative != path:
            raise pools._error("manifest_path_invalid", "Pool file path is not canonical", path=path)
        folded = relative.casefold()
        if folded in result:
            raise pools._error("manifest_path_collision", "Pool file identities collide", path=path)
        result[folded] = path
    return result


def _pool_inventory(destination, previous, *, approved=None, allowed_hashes=None, preserve_cache=True):
    """Keep physical scope/hash checks while returning original manifest keys."""
    from tools.jenkins.workflow import pools
    from tools.jenkins.resources.paths import physical_path_under
    names = _pool_path_index(previous or {})
    if allowed_hashes is not None and set(allowed_hashes) != set(previous or {}):
        raise pools._error("preparation_recovery_ambiguous", "Recovery file closure differs")
    directories = {"/".join(path.split("/")[:index]).casefold()
                   for path in previous or {} for index in range(1, len(path.split("/")))}
    actual, seen = {}, {}
    if destination.exists():
        pools._check_tree(destination)
        for candidate in destination.rglob("*"):
            if approved is not None:
                physical_path_under(approved, candidate, allow_missing=False)
            relative = candidate.relative_to(destination).as_posix()
            if preserve_cache and relative.split("/", 1)[0].casefold() in pools._PRESERVED_DIRS:
                continue
            folded = relative.casefold()
            if folded in seen:
                raise pools._error("preparation_path_collision", "Actual pool identities collide", path=relative)
            seen[folded] = relative
            if candidate.is_dir():
                if folded not in directories:
                    raise pools._error("preparation_foreign_file", "preparation contains an unregistered directory", path=relative)
                continue
            if not candidate.is_file():
                continue
            original = names.get(folded)
            if original is None:
                raise pools._error("preparation_foreign_file", "preparation contains an unregistered file", path=relative)
            checksum = pools._hash(candidate)
            permitted = allowed_hashes[original] if allowed_hashes is not None else (previous[original],)
            if checksum not in permitted:
                code = "preparation_recovery_ambiguous" if allowed_hashes is not None else "preparation_hash_mismatch"
                raise pools._error(code, "Owned preparation file has an unrecognised hash", path=relative)
            actual[original] = checksum
    for original, checksum in (previous or {}).items():
        permitted = allowed_hashes[original] if allowed_hashes is not None else (checksum,)
        if original not in actual and None not in permitted:
            code = "preparation_recovery_ambiguous" if allowed_hashes is not None else "preparation_hash_mismatch"
            raise pools._error(code, "Owned preparation file is missing", path=original)
    return actual


def _recover_pool(state, key, destination, objects, approved):
    """Recover only journaled old/new bytes; preserve raw keys and CAS versions."""
    from tools.jenkins.workflow import pools
    from tools.jenkins.resources.paths import physical_path_under
    row = state.get("preparation_update", key)
    if not row or row["payload"].get("status") not in {"complete", "updating"}:
        return
    destination = physical_path_under(approved, destination, allow_missing=True)
    intent = row["payload"]
    if intent["status"] == "complete":
        if not state.get("preparation_source", key) and intent.get("manifest") and intent.get("ownedFiles") is not None:
            _pool_inventory(destination, intent["ownedFiles"])
            state.put("preparation_source", key, {"executionId": intent.get("executionId"),
                "preparationKey": key, "sourceDigest": intent.get("sourceDigest"),
                "manifest": intent["manifest"], "objectRoot": intent.get("objectRoot"),
                "ownedFiles": intent["ownedFiles"], "physicalPath": str(destination), "updatedAt": time.time()})
        return
    files = intent.get("files", [])
    _pool_path_index(item.get("path") for item in files)
    old_owned = intent.get("oldOwnedFiles", {})
    old_names = _pool_path_index(old_owned)
    desired = {item["path"]: item.get("newHash") for item in files}
    permitted = {item["path"]: (item.get("oldHash"), item.get("newHash")) for item in files}
    # This checks every actual member, directory and old/new hash before writes.
    _pool_inventory(destination, desired, allowed_hashes=permitted)
    version, updates = row["version"], 0
    for item in files:
        target = destination / item["path"]
        observed = pools._hash(target)
        old, new = item.get("oldHash"), item.get("newHash")
        if observed not in (old, new):
            raise pools._error("preparation_recovery_ambiguous", "interrupted update has an unrecognised file hash", path=item["path"])
        copied = False
        if observed != new:
            target = physical_path_under(approved, target)
            if objects is None:
                raise pools._error("preparation_recovery_incomplete", "interrupted update needs its sealed object root")
            if new is None:
                owned_name = old_names.get(item["path"].casefold())
                if owned_name is None or old_owned[owned_name] != old:
                    raise pools._error("preparation_recovery_scope", "journalled deletion is not owned", path=item["path"])
                if target.exists():
                    target.unlink()
            else:
                if not isinstance(new, str) or not re.fullmatch(r"[0-9a-f]{64}", new):
                    raise pools._error("manifest_object_invalid", "Recovery object digest is invalid", path=item["path"])
                source = physical_path_under(approved, objects / "inputs" / "objects" / new, allow_missing=False)
                if pools._hash(source) != new:
                    raise pools._error("input_object_missing", "Recovery object is missing or corrupt", path=item["path"])
                pools._atomic_copy(source, target)
                if pools._hash(target) != new:
                    raise pools._error("preparation_recovery_hash", "recovered object hash does not match", path=item["path"])
            copied = True
        changed = not item.get("applied") or copied
        item["applied"] = True
        if changed:
            updates += 1
            if updates % 512 == 0:
                state.put("preparation_update", key, {**intent, "files": files}, expected_version=version)
                version += 1
    state.put("preparation_update", key, {**intent, "status": "complete", "files": files,
              "recoveredAt": time.time()}, expected_version=version)
    if intent.get("manifest") and intent.get("ownedFiles") is not None:
        state.put("preparation_source", key, {"executionId": intent.get("executionId"),
            "preparationKey": key, "sourceDigest": intent.get("sourceDigest"),
            "manifest": intent["manifest"], "objectRoot": intent.get("objectRoot"),
            "ownedFiles": intent["ownedFiles"], "physicalPath": str(destination), "updatedAt": time.time()})


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
    _recover_pool(state, key, destination, objects, approved)
    recovered = state.get("preparation_update", key)
    if recovered and recovered["payload"].get("status") == "complete":
        journal = recovered["payload"]
        prune_empty_owned_parents(set(journal.get("oldOwnedFiles", {})) - set(journal.get("ownedFiles", {})))
    previous_row = state.get("preparation_source", key)
    previous = previous_row["payload"] if previous_row else None
    old = previous.get("ownedFiles", {}) if previous else {}
    adoption_rows = None
    if (previous and previous["sourceDigest"] != sealed["sealedInputRef"] and old == desired
            and recovered and recovered["payload"].get("status") == "complete"):
        if (previous.get("preparationKey") != key or previous.get("physicalPath") != str(destination)
                or previous.get("objectRoot") != str(objects)
                or recovered["payload"].get("preparationKey") != key
                or recovered["payload"].get("ownedFiles") != old):
            raise ValueError("Same-byte preparation lacks its completed physical binding")
        adoption_rows = [(domain, row_key, state.get(domain, row_key)) for domain, row_key in (
            ("execution", owner), ("pool_writer_hold", key),
            ("resource_reservation", reservation_key))]
    actual = _pool_inventory(destination, old if previous else None)
    if adoption_rows is not None and old == desired == actual:
        current_execution, hold, reservation = [((row or {}).get("payload", {}))
                                                for _domain, _row_key, row in adoption_rows]
        if (current_execution.get("kind") != "management_owned_cargo"
                or current_execution.get("status") != "preparing"
                or current_execution.get("preparationKey") != key
                or current_execution.get("reservationKey") != reservation_key
                or hold.get("status") != "active" or hold.get("owner") != owner
                or hold.get("reservationKey") != reservation_key
                or reservation.get("status") != "active" or reservation.get("owner") != owner):
            raise ValueError("Same-byte preparation requires its active writer and reservation")
        pins = [("preparation_source", key, previous_row), ("preparation_update", key, recovered),
                ("sealed_input", sealed["sealedInputRef"], record), *adoption_rows]
        # No source bytes change. Publish the new seal only after the complete
        # inventory, without creating a per-file update journal for identical bytes.
        adopted = {"executionId": owner, "preparationKey": key,
                   "sourceDigest": sealed["sealedInputRef"], "manifest": normalized,
                   "objectRoot": str(objects), "ownedFiles": desired,
                   "physicalPath": str(destination), "updatedAt": time.time()}
        with state.transaction() as connection:
            if any(state.get(domain, row_key, connection=connection) != pinned
                   for domain, row_key, pinned in pins):
                raise ValueError("Same-byte preparation authority changed before identity adoption")
            state.put("preparation_source", key, adopted,
                      expected_version=previous_row["version"], connection=connection)
        previous = adopted
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
    if _pool_inventory(destination, desired) != desired:
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
    from tools.jenkins.contracts import JenkinsError
    from tools.jenkins.resources.paths import physical_path_under
    from tools.jenkins.source.external_inputs import sources, flattened

    source = physical_path_under(approved, source_path, allow_missing=False)
    with_external = bool(sources(manifest))
    root = physical_path_under(approved, source.parent if with_external else source, allow_missing=False)
    expected_manifest = flattened(manifest) if with_external else manifest
    _pool_path_index(entry["path"] for entry in expected_manifest["entries"])
    expected = {entry["path"]: entry["sha256"] for entry in expected_manifest["entries"]
                if entry["status"] == "present"}
    try:
        actual = _pool_inventory(root, expected, approved=approved, preserve_cache=False)
    except JenkinsError as error:
        raise ValueError("Actual prepared source bytes differ from the sealed execution input") from error
    return {"physicalPath": str(source), "sourceDigest": digest(manifest),
            "filesDigest": digest(actual), "fileCount": len(actual)}

# These two management actions belong to the hash-pinned management capsule.
# The service's sealed support modules remain the State, authorization and CAS
# authorities; none of their module globals or import paths are replaced.
from contextlib import contextmanager
from concurrent.futures import ThreadPoolExecutor
import ctypes
import ntpath
import threading


@contextmanager
def _managed_phase(name):
    started = time.perf_counter()
    print("[management] " + name + " started", flush=True)
    try:
        yield
    finally:
        print("[management] " + name + " elapsed=" + format(time.perf_counter() - started, ".3f") + "s", flush=True)


def _managed_native():
    from tools.jenkins.pilot.native import windows_storage
    if not windows_storage.is_available():
        raise ValueError("Managed input inspection requires native Windows read pins")
    return windows_storage


def _managed_relative(value):
    from tools.jenkins.contracts import JenkinsError
    raw = str(value).replace("\\", "/")
    if (ntpath.isabs(raw) or len(raw) >= 2 and raw[1] == ":"
            or not raw or any(part in {"", ".", ".."} for part in raw.split("/"))):
        raise JenkinsError("invalid_source_path", "source path must be repository-relative without traversal",
                           details={"path": raw})
    return raw


class _ManagedInputStage:
    """Cache only directory proofs backed by still-held, non-writable handles.

    A fresh instance is required at each existing before/after checkpoint.
    Files and absences are never cached. READ sharing on every directory below
    the volume prevents rename/delete and conversion to a reparse point.
    """

    def __init__(self, root, phase):
        self.root, self.phase = Path(root).absolute(), phase
        self.native = _managed_native()
        self.directories, self.order = {}, []
        self.lock, self.readers = threading.RLock(), threading.local()
        self.observed = self.hashed = self.byte_count = 0
        self.started = time.perf_counter()
        self.drive, tail = ntpath.splitdrive(str(self.root))
        if (self.drive.upper() not in {"D:", "E:", "F:"} or not tail.startswith("\\")
                or any(part in {"", ".", ".."} for part in tail[1:].split("\\"))):
            raise ValueError("Managed inputs require a literal physical D/E/F root")
        self.drive = self.drive.upper()
        self.expected_device = None

    def __enter__(self):
        print("[management] " + self.phase + " pins started: " + str(self.root), flush=True)
        try:
            self._directory(str(self.root))
            return self
        except BaseException:
            self._close()
            raise

    def _directory(self, path):
        key = ntpath.normcase(path)
        cached = self.directories.get(key)
        if cached is not None:
            return cached[0]
        drive, tail = ntpath.splitdrive(path)
        if tail == "\\":
            from tools.jenkins.pilot.native.paths import _verify_local_drive_mapping
            self.expected_device = _verify_local_drive_mapping(drive)
            handle = self.native._open_volume_root(drive)
            try:
                identity = self.native._ensure_directory_handle(handle)
                observed = ntpath.normcase(ntpath.normpath(self.native._final_nt_path(handle)))
                expected = ntpath.normcase(ntpath.normpath(self.expected_device + "\\"))
                if observed != expected:
                    raise ValueError("Managed input volume identity differs")
            except BaseException:
                self.native._close_handle(handle)
                raise
        else:
            parent = self._directory(ntpath.dirname(path))
            handle = self.native._open_relative_handle(
                parent, ntpath.basename(path), access=self.native._DIRECTORY_READ_ACCESS,
                share=self.native._FILE_SHARE_READ,
                options=(self.native._FILE_DIRECTORY_FILE | self.native._FILE_OPEN_REPARSE_POINT
                         | self.native._FILE_SYNCHRONOUS_IO_NONALERT))
            try:
                identity = self.native._ensure_directory_handle(handle)
            except BaseException:
                self.native._close_handle(handle)
                raise
        self.directories[key] = (handle, identity, path)
        self.order.append(handle)
        return handle

    @contextmanager
    def _leaf(self, value, *, content):
        from tools.jenkins.contracts import JenkinsError
        raw = _managed_relative(value)
        path = ntpath.join(str(self.root), *raw.split("/"))
        handle = None
        try:
            with self.lock:
                self.observed += 1
                parent = self._directory(ntpath.dirname(path))
            access = self.native._FILE_READ_ATTRIBUTES | self.native._SYNCHRONIZE
            if content:
                access |= self.native._FILE_READ_DATA
            handle = self.native._open_relative_handle(
                parent, ntpath.basename(path), access=access, share=self.native._FILE_SHARE_READ,
                options=self.native._FILE_OPEN_REPARSE_POINT | self.native._FILE_SYNCHRONOUS_IO_NONALERT)
            attributes = int(self.native._attributes(handle).FileAttributes)
            if attributes & self.native._FILE_ATTRIBUTE_REPARSE_POINT:
                raise JenkinsError("reparse_path", "source paths cannot cross reparse points",
                                   details={"path": path})
            if content and attributes & self.native._FILE_ATTRIBUTE_DIRECTORY:
                raise JenkinsError("managed_source_directory", "Source preparation requires explicit ordinary files",
                                   details={"path": path})
        except OSError as error:
            number = getattr(error, "winerror", None) or getattr(error, "errno", None)
            if handle is not None:
                self.native._close_handle(handle)
                handle = None
            if number not in {2, 3}:
                raise JenkinsError("managed_source_unavailable", "Cannot acquire an exact native input read pin",
                                   retryable=number in {5, 32, 33},
                                   details={"path": path, "nativeError": number}) from error
        except BaseException:
            if handle is not None:
                self.native._close_handle(handle)
            raise
        try:
            yield handle
        finally:
            if handle is not None:
                self.native._close_handle(handle)

    def canonical(self, value):
        raw = _managed_relative(value)
        with self._leaf(raw, content=False):
            return raw.casefold()

    def checksum(self, value):
        return self._read(value, retain_bytes=False)[0]

    def capture(self, value):
        """Return one pinned file's hash, payload and mode from the same read."""
        return self._read(value, retain_bytes=True)

    def _read(self, value, *, retain_bytes):
        from tools.jenkins.contracts import JenkinsError
        with self._leaf(value, content=True) as handle:
            if handle is None:
                return None, None, None
            before = self.native.identity_from_handle(handle)
            api, _ntdll, _msvcrt = self.native._apis()
            buffer = getattr(self.readers, "buffer", None)
            if buffer is None:
                buffer = self.readers.buffer = ctypes.create_string_buffer(256 * 1024)
            checksum, byte_count, chunks = hashlib.sha256(), 0, []
            while True:
                count = ctypes.c_ulong()
                if not api.ReadFile(handle, buffer, len(buffer), ctypes.byref(count), None):
                    raise ctypes.WinError(ctypes.get_last_error())
                if not count.value:
                    break
                data = memoryview(buffer)[:count.value]
                checksum.update(data)
                if retain_bytes:
                    chunks.append(bytes(data))
                byte_count += count.value
            after = self.native.identity_from_handle(handle)
            if not before.same_file_state(after) or byte_count != after.size:
                raise JenkinsError("managed_source_changed", "Input changed during its native read",
                                   details={"path": str(self.root / str(value))})
            mode = (0o444 if int(self.native._attributes(handle).FileAttributes) & 1 else 0o666) if retain_bytes else None
            with self.lock:
                self.hashed += 1
                self.byte_count += byte_count
            return checksum.hexdigest(), b"".join(chunks) if retain_bytes else None, mode

    def verify(self):
        from tools.jenkins.pilot.native.paths import _verify_local_drive_mapping
        if _verify_local_drive_mapping(self.drive) != self.expected_device:
            raise ValueError("Managed input drive mapping changed")
        for handle, expected, path in self.directories.values():
            if not expected.same_object(self.native._ensure_directory_handle(handle)):
                raise ValueError("A pinned managed input directory changed: " + path)

    def _close(self):
        first = None
        while self.order:
            handle = self.order.pop()
            try:
                self.native._close_handle(handle)
            except BaseException as error:
                first = first or error
        self.directories.clear()
        if first is not None:
            raise first

    def __exit__(self, kind, value, trace):
        try:
            if kind is None:
                self.verify()
        finally:
            directories = len(self.directories)
            try:
                self._close()
            finally:
                print("[management] " + self.phase + " pins released: paths=" + str(self.observed)
                      + " directories=" + str(directories) + " files=" + str(self.hashed)
                      + " bytes=" + str(self.byte_count)
                      + " elapsed=" + format(time.perf_counter() - self.started, ".3f") + "s", flush=True)


def managed_input_stage(root, phase):
    return _ManagedInputStage(root, phase)


def _managed_map(stage, names, *, metadata=False):
    names = list(names)
    method = stage.canonical if metadata else stage.checksum
    result = {}
    with ThreadPoolExecutor(max_workers=4) as readers:
        for start in range(0, len(names), 128):
            selected = names[start:start + 128]
            for name, observed in zip(selected, readers.map(method, selected)):
                result[name] = observed
            if start and start % 4096 == 0:
                print("[management] " + stage.phase + " checked " + str(start)
                      + "/" + str(len(names)), flush=True)
    return result


def _managed_checksums(root, names, phase):
    with managed_input_stage(root, phase) as stage:
        return _managed_map(stage, names)


def _managed_hashes(root, values, phase):
    from tools.jenkins.contracts import JenkinsError, require_digest
    if not isinstance(values, dict) or not values:
        raise JenkinsError("current_source_hashes_missing", "Provide the complete explicit current file closure")
    result = {}
    for path, checksum in values.items():
        if not isinstance(path, str):
            raise JenkinsError("current_source_scope_invalid", "Current file paths must be strings")
        relative = _managed_relative(path).casefold()
        if relative in result:
            raise JenkinsError("current_source_scope_invalid", "Current file paths collide")
        if checksum is not None:
            require_digest(checksum, "currentHash")
        result[relative] = checksum
    if root is not None:
        with managed_input_stage(root, phase) as stage:
            _managed_map(stage, values, metadata=True)
    return dict(sorted(result.items()))


def _managed_drift(expected, observed, code, message):
    from tools.jenkins.contracts import JenkinsError
    if expected == observed:
        return
    old, new = set(expected), set(observed)
    added, removed = sorted(new - old), sorted(old - new)
    changed = sorted(path for path in old & new if isinstance(expected, dict)
                     and isinstance(observed, dict) and expected[path] != observed[path])
    details = {"addedCount": len(added), "removedCount": len(removed), "changedCount": len(changed),
               "addedPaths": added[:16], "removedPaths": removed[:16], "changedPaths": changed[:16]}
    print("[management] input drift " + json.dumps(details, sort_keys=True), flush=True)
    raise JenkinsError(code, message, details=details)


def _managed_membership(management, base, prior):
    from tools.jenkins import managed_worktree as formal
    names = management.git(management.repo, "ls-files", "--cached", "--others",
                           "--exclude-standard", "-z").stdout
    raw = {name.decode("utf-8", "strict").replace("\\", "/") for name in names.split(b"\0") if name}
    raw.update(str(name).replace("\\", "/") for name in prior["files"]
               if formal.product_path(name.casefold()))
    # Retain absence identities only from the already verified WT owner scope.
    raw.update(management.owned)
    with managed_input_stage(management.repo, "canonical-membership") as stage:
        values = set(_managed_map(stage, sorted(raw), metadata=True).values())
    values.update(formal._git_base_tree(management.repo, base))
    values.update(formal.PROJECT_TEMPLATE_INPUTS)
    values.update(formal.COMPILER_CONTROL_PATHS)
    return {name for name in values if formal.product_path(name)}


def _managed_verify_canonical(management, head, hashes, base, prior):
    from tools.jenkins import managed_worktree as formal
    formal._require(management.text(management.repo, "rev-parse", "HEAD") == head,
                    "managed_source_changed", "Canonical input HEAD changed during preparation")
    membership = _managed_membership(management, base, prior)
    _managed_drift(set(hashes), membership, "managed_source_membership_changed",
                   "The complete canonical source membership changed")
    actual = _managed_checksums(management.repo, hashes, "canonical-bytes")
    _managed_drift(hashes, actual, "managed_source_changed",
                   "Canonical source differs from its declared hash")



_MANAGED_SEALED_INVENTORY = "jenkins-managed-sealed-inventory-v1"


def _managed_owned_hashes(payload, owned):
    from tools.jenkins import managed_worktree as formal
    if "sourceHashes" not in payload:
        return None
    hashes = _managed_hashes(None, payload["sourceHashes"], "declared-owned-input")
    formal._require(set(hashes) == set(owned), "managed_owned_scope_invalid",
                    "Declared source hashes must match every owned path exactly")
    return hashes


def _managed_publish_object(store, relative, data, checksum):
    """Expose a complete CAS object only after closing its exclusive handle."""
    from tools.jenkins.contracts import JenkinsError
    from tools.jenkins import managed_worktree as formal
    formal._require(relative == "inputs/objects/" + checksum
                    and hashlib.sha256(data).hexdigest() == checksum,
                    "managed_source_object_changed", "Native publication requires its exact captured object")
    path = ntpath.join(str(store.root), *_managed_relative(relative).split("/"))
    with store.lock:
        parent = store._directory(ntpath.dirname(path))
    native = store.native
    try:
        handle = native._open_relative_handle(
            parent, checksum,
            access=(native._FILE_READ_DATA | native._FILE_WRITE_DATA | native._FILE_READ_ATTRIBUTES
                    | native._DELETE | native._SYNCHRONIZE),
            share=0, disposition=native._FILE_CREATE,
            options=(native._FILE_NON_DIRECTORY_FILE | native._FILE_OPEN_REPARSE_POINT
                     | native._FILE_SYNCHRONOUS_IO_NONALERT))
    except OSError as error:
        number = native._windows_error_code(error)
        if number not in {80, 183}:
            raise
        existing = store.checksum(relative)
        if existing is None:
            raise JenkinsError("managed_source_unavailable", "CAS creation race requires a fresh exact read pin",
                               retryable=True, details={"path": path, "nativeError": number}) from error
        formal._require(existing == checksum, "managed_source_object_changed",
                        "An existing CAS object differs from its captured digest")
        return
    complete = False
    try:
        before = native._ensure_regular_file_handle(handle)
        native._write_handle(handle, memoryview(data))
        kernel32, _ntdll, _msvcrt = native._apis()
        if not kernel32.FlushFileBuffers(ctypes.c_void_p(handle)):
            native._raise_last_error("FlushFileBuffers")
        written = native.identity_from_handle(handle)
        formal._require(before.same_object(written) and written.size == len(data),
                        "managed_source_object_changed", "Native object changed during its write")
        complete = True
    finally:
        try:
            if not complete:
                native._mark_delete(handle)
        finally:
            native._close_handle(handle)


def _managed_capture_files(root, names, object_root, phase, *, expected=None):
    """Publish the bytes read through each native pin, never a later path read."""
    from tools.jenkins import managed_worktree as formal
    objects = object_root / "inputs/objects"
    formal._plain(objects)
    objects.mkdir(parents=True, exist_ok=True)
    expected = expected or {}
    result, metadata = {}, {}
    publication = threading.Lock()
    with managed_input_stage(root, phase) as inputs:
        with managed_input_stage(object_root, phase + "-objects") as store:
            def capture(name):
                checksum, data, mode = inputs.capture(name)
                folded = _managed_relative(name).casefold()
                if folded in expected:
                    formal._require(checksum == expected[folded], "managed_owned_source_changed",
                                    "Owned source differs from its declared request hash")
                if checksum is not None:
                    formal._require(hashlib.sha256(data).hexdigest() == checksum,
                                    "managed_source_object_changed", "Captured bytes differ from their native hash")
                    relative = "inputs/objects/" + checksum
                    # Keep native source reads parallel; duplicate objects share publication.
                    with publication:
                        existing = store.checksum(relative)
                        formal._require(existing in (None, checksum), "managed_source_object_changed",
                                        "An existing source object is corrupt")
                        if existing is None:
                            _managed_publish_object(store, relative, data, checksum)
                        formal._require(store.checksum(relative) == checksum, "managed_source_object_changed",
                                        "Published source object differs from its captured bytes")
                return folded, checksum, {"size": len(data) if data is not None else 0, "mode": mode}
            with ThreadPoolExecutor(max_workers=4) as readers:
                ordered = list(names)
                for start in range(0, len(ordered), 128):
                    for name, checksum, facts in readers.map(capture, ordered[start:start + 128]):
                        formal._require(name not in result, "managed_source_scope_invalid",
                                        "Captured source paths collide")
                        result[name], metadata[name] = checksum, facts
                    if start and start % 4096 == 0:
                        print("[management] " + phase + " sealed " + str(start) + "/" + str(len(ordered)), flush=True)
    return dict(sorted(result.items())), metadata


def _managed_captured_manifest(root, base, hashes, metadata, *, external=(), product=False):
    from tools.jenkins import managed_worktree as formal
    from tools.jenkins.source import repository_id
    tree, entries = formal._git_base_tree(root, base), []
    for name, checksum in sorted(hashes.items()):
        original = tree.get(name.casefold())
        entry = {"path": original[0] if original else name, "status": "deleted" if checksum is None else "present",
                 "sha256": checksum, "size": metadata[name]["size"], "beforeHash": checksum,
                 "baseBlob": original[1] if original else None}
        if checksum is not None:
            entry.update(mode=metadata[name]["mode"], objectDigest=checksum)
        entries.append(entry)
    manifest = {"schemaVersion": 1, "repositoryId": repository_id(root), "baseHead": base,
                "entries": sorted(entries, key=lambda entry: entry["path"]), "external": list(external),
                "declaredDependencies": []}
    if product:
        manifest["preparationKind"] = "current-source"
    return manifest


_SOURCE_EVIDENCE_V2 = "full-file-index-v2"


def full_file_index_requested(payload):
    if "sourceEvidencePolicy" not in payload:
        return False
    if payload["sourceEvidencePolicy"] != _SOURCE_EVIDENCE_V2:
        raise ValueError("Unsupported explicit source evidence policy")
    return True


def source_evidence_payload(payload, source):
    """Select the original evidence version without translating its identity."""
    checked_source_evidence(source)
    value = dict(payload)
    value.pop("sourceEvidencePolicy", None)
    if source.get("sourceEvidencePolicy") == _SOURCE_EVIDENCE_V2:
        value["sourceEvidencePolicy"] = _SOURCE_EVIDENCE_V2
    return value


def _source_index_entries(raw):
    entries, seen, aliases = [], set(), {}
    for row in raw.split(b"\0"):
        if not row:
            continue
        fields, name = row.split(b"\t", 1)
        mode, obj, stage = fields.decode("ascii", "strict").split()
        name = name.decode("utf-8", "strict")
        if (_managed_relative(name) != name or "\\" in name or not re.fullmatch(r"[0-7]{6}", mode)
                or not re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", obj) or stage not in {"0", "1", "2", "3"}
                or (name, int(stage)) in seen or aliases.get(name.casefold(), name) != name):
            raise ValueError("Complete Git index contains a noncanonical or duplicate entry")
        seen.add((name, int(stage)))
        aliases[name.casefold()] = name
        entries.append({"path": name, "mode": mode, "object": obj, "stage": int(stage)})
    return sorted(entries, key=lambda item: (item["path"], item["stage"]))


def full_file_index_metadata(management, root):
    """Capture index objects/stages plus all flags, and the complete file membership."""
    head = management.text(root, "rev-parse", "HEAD")
    if not re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", head):
        raise ValueError("Full source evidence requires an immutable HEAD")
    raw = management.git(root, "ls-files", "--cached", "--others", "--exclude-standard", "-z").stdout
    names = list(dict.fromkeys(name.decode("utf-8", "strict") for name in raw.split(b"\0") if name))
    aliases = {}
    for name in names:
        if (_managed_relative(name) != name or "\\" in name
                or aliases.get(name.casefold(), name) != name):
            raise ValueError("Full source membership contains a noncanonical or aliased path")
        aliases[name.casefold()] = name
    stage = management.git(root, "ls-files", "--stage", "-z").stdout
    debug = management.git(root, "ls-files", "--stage", "--debug", "-z").stdout
    return {"head": head, "names": sorted(names), "gitIndex": {
        "entries": _source_index_entries(stage), "metadataSha256": hashlib.sha256(debug).hexdigest()}}


def checked_source_evidence(source):
    """Recognize exact v1 or v2 shapes; a v2 source never invents a diff digest."""
    from tools.jenkins import managed_worktree as formal
    v2 = isinstance(source, dict) and source.get("sourceEvidencePolicy") == _SOURCE_EVIDENCE_V2
    keys = {"head", "files", "sourceDigest"} | (
        {"sourceEvidencePolicy", "gitIndex"} if v2 else {"diffSha256"})
    formal._require(isinstance(source, dict) and set(source) == keys and isinstance(source.get("files"), dict)
                    and isinstance(source.get("head"), str)
                    and re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", source["head"])
                    and source.get("sourceDigest") == digest(
                        {name: value for name, value in source.items() if name != "sourceDigest"}),
                    "source_evidence_invalid", "Source evidence shape or complete digest differs")
    source_files(source)
    if not v2:
        formal._require(isinstance(source["diffSha256"], str)
                        and re.fullmatch(r"[0-9a-f]{64}", source["diffSha256"]),
                        "source_evidence_invalid", "Legacy source requires its actual original binary-diff digest")
    else:
        index = source["gitIndex"]
        formal._require(isinstance(index, dict) and set(index) == {"entries", "metadataSha256"}
                        and isinstance(index["entries"], list)
                        and isinstance(index["metadataSha256"], str)
                        and re.fullmatch(r"[0-9a-f]{64}", index["metadataSha256"]),
                        "source_evidence_invalid", "Full source requires complete index metadata")
        raw = []
        for entry in index["entries"]:
            formal._require(isinstance(entry, dict) and set(entry) == {"path", "mode", "object", "stage"}
                            and type(entry["stage"]) is int and isinstance(entry["path"], str)
                            and isinstance(entry["mode"], str) and isinstance(entry["object"], str),
                            "source_evidence_invalid", "Full index entry differs")
            raw.append((entry["mode"] + " " + entry["object"] + " " + str(entry["stage"])
                        + "\t" + entry["path"]).encode("utf-8") + b"\0")
        formal._require(_source_index_entries(b"".join(raw)) == index["entries"]
                        and all(entry["path"] in source["files"] for entry in index["entries"]),
                        "source_evidence_invalid", "Full source omits an indexed path or canonical entry")
    for name, value in source["files"].items():
        if isinstance(value, dict):
            formal._require(set(value) == {"gitlinkCommit", "source"}
                            and isinstance(value["gitlinkCommit"], str)
                            and re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", value["gitlinkCommit"]),
                            "source_evidence_invalid", "Nested Gitlink identity differs")
            checked_source_evidence(value["source"])
            if v2:
                formal._require(any(entry == {"path": name, "mode": "160000",
                    "object": value["gitlinkCommit"], "stage": 0} for entry in source["gitIndex"]["entries"]),
                    "source_evidence_invalid", "Nested Gitlink differs from its complete index entry")
    return source


def finish_full_file_index(management, root, files, before):
    after = full_file_index_metadata(management, root)
    if after != before or set(files) != set(before["names"]):
        raise ValueError("HEAD, full index or complete source membership changed during capture")
    value = {"sourceEvidencePolicy": _SOURCE_EVIDENCE_V2, "head": before["head"],
             "files": files, "gitIndex": before["gitIndex"]}
    return checked_source_evidence({**value, "sourceDigest": digest(value)})


def source_diff_digest(management, root):
    result = management.git(root, "diff", "--no-ext-diff", "--no-textconv", "--binary", "--full-index", "HEAD", "--")
    return result.stdoutSha256 if hasattr(result, "stdoutSha256") else hashlib.sha256(result.stdout).hexdigest()


def matching_target_source(management, target, prior):
    return management.source(target, payload=source_evidence_payload(management.payload, prior))


class _ManagedCapturedExternal:
    """Supply the existing Gitlink inventory protocol with already sealed reads."""

    def __init__(self, management, object_root):
        self.management, self.object_root, self.metadata = management, object_root, {}

    def __getattr__(self, name):
        return getattr(self.management, name)

    def source(self, root, *, payload=None):
        from tools.jenkins import managed_worktree as formal
        payload = self.payload if payload is None else payload
        names = self.git(root, "ls-files", "--cached", "--others", "--exclude-standard", "-z").stdout
        names = list(dict.fromkeys(name.decode("utf-8", "strict") for name in names.split(b"\0") if name))
        head, gitlinks = self.text(root, "rev-parse", "HEAD"), {}
        if root == self.repo.parent / "zr_vm":
            stage = self.git(root, "ls-files", "--stage", "-z").stdout
            indexed = {}
            for row in stage.split(b"\0"):
                if row.startswith(b"160000 "):
                    fields, name = row.split(b"\t", 1)
                    indexed[name.decode("utf-8", "strict")] = fields.decode("ascii").split()[1]
            requested = payload.get("externalGitlinks")
            formal._require(isinstance(requested, dict) and set(requested) == set(indexed)
                            and all(isinstance(value, dict) and value.get("indexCommit") == indexed[name]
                                    for name, value in requested.items()),
                            "external_scope_invalid", "External Gitlinks require their exact registered index")
            for name, registered in requested.items():
                child = root / _managed_relative(name)
                formal._plain(child)
                formal._require(child.is_dir() and Path(self.text(child, "rev-parse", "--show-toplevel")).absolute() == child,
                                "external_repository_invalid", "Registered Gitlink must retain its physical root")
                captured = self.source(child, payload=payload)
                formal._require(captured["head"] == registered.get("head"), "external_capture_changed",
                                "Registered external Gitlink HEAD changed")
                gitlinks[name] = {"gitlinkCommit": indexed[name], "source": captured}
        ordinary = [name for name in names if name not in gitlinks]
        formal._require(not any(_managed_relative(name).split("/", 1)[0].casefold()
                                in {".git", ".codex", ".jenkins", ".opencode", "target"} for name in ordinary),
                        "external_control_forbidden", "External checkout controls are not compiler inputs")
        files, metadata = _managed_capture_files(root, ordinary, self.object_root, "external-capture")
        self.metadata[str(root)] = metadata
        files.update(gitlinks)
        value = {"head": head, "files": files, "diffSha256": source_diff_digest(self, root)}
        formal._require(self.text(root, "rev-parse", "HEAD") == head, "external_capture_changed",
                        "External Git base changed while its tuple was captured")
        return {**value, "sourceDigest": digest(value)}

    def flattened_metadata(self, root, source, prefix=""):
        result = {}
        for name, checksum in source["files"].items():
            if isinstance(checksum, dict):
                result.update(self.flattened_metadata(root / name, checksum["source"], prefix + name.casefold() + "/"))
            else:
                result[prefix + name.casefold()] = self.metadata[str(root)][name.casefold()]
        return result


def _managed_sealed_inventory(management, record, inputs, inspection_payload):
    """Verify a completed immutable tuple without opening the mutable source roots."""
    from tools.jenkins import managed_worktree as formal
    from tools.jenkins.source import repository_id
    from tools.jenkins.source.external_inputs import KIND, sources
    sealed = inputs.get("sealedSource")
    object_root = formal.canonical_build_root(management.build_root()).namespace()
    formal._require(isinstance(sealed, dict)
                    and set(sealed) == {"kind", "objectRoot", "sourceManifest", "sourceDigest", "externalCapture"}
                    and sealed.get("kind") == _MANAGED_SEALED_INVENTORY
                    and sealed.get("objectRoot") == str(object_root),
                    "managed_sealed_inventory_invalid", "Sealed inventory must use its exact managed object namespace")
    formal._plain(object_root)
    manifest = sealed["sourceManifest"]
    formal._require(isinstance(manifest, dict) and digest(manifest) == sealed.get("sourceDigest"),
                    "managed_sealed_inventory_changed", "Sealed inventory manifest differs from its exact digest")
    hashes = _managed_hashes(None, inputs.get("currentHashes"), "sealed-current-scope")
    owned = _managed_owned_hashes(inspection_payload, record["ownedPaths"])
    formal._require(owned is None or all(hashes.get(path) == checksum for path, checksum in owned.items()),
                    "managed_owned_source_changed", "Sealed owned bytes differ from their original request")
    objects = {}

    def checked_manifest(value, expected, root, base, *, product=False):
        keys = {"schemaVersion", "repositoryId", "baseHead", "entries", "external", "declaredDependencies"}
        if product:
            keys.add("preparationKind")
        formal._require(isinstance(value, dict) and set(value) == keys and value.get("schemaVersion") == 1
                        and value.get("repositoryId") == repository_id(root) and value.get("baseHead") == base
                        and value.get("declaredDependencies") == []
                        and (value.get("preparationKind") == "current-source" if product else value.get("external") == []),
                        "managed_sealed_inventory_invalid", "Sealed manifest authority or schema differs")
        entries, actual = value.get("entries"), {}
        formal._require(isinstance(entries, list) and entries, "managed_sealed_inventory_invalid",
                        "Sealed manifest needs its complete explicit entries")
        paths = []
        for entry in entries:
            formal._require(isinstance(entry, dict) and isinstance(entry.get("path"), str),
                            "managed_sealed_inventory_invalid", "Sealed source entry has no canonical path")
            name = _managed_relative(entry["path"]).casefold()
            checksum = entry.get("sha256")
            present = checksum is not None
            fields = {"path", "status", "sha256", "size", "beforeHash", "baseBlob"}
            if present:
                fields.update(("mode", "objectDigest"))
            formal._require(set(entry) == fields and name not in actual and name in expected
                            and checksum == expected[name] and entry.get("beforeHash") == checksum
                            and entry.get("status") == ("present" if present else "deleted")
                            and type(entry.get("size")) is int and entry["size"] >= 0
                            and (entry.get("baseBlob") is None or isinstance(entry["baseBlob"], str)
                                 and re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", entry["baseBlob"])),
                            "managed_sealed_inventory_invalid", "Sealed entry differs from its complete hash closure")
            if present:
                formal._require(entry.get("objectDigest") == checksum and type(entry.get("mode")) is int
                                and 0 <= entry["mode"] <= 0o7777
                                and (checksum not in objects or objects[checksum] == entry["size"]),
                                "managed_sealed_inventory_invalid", "Sealed object metadata differs")
                objects[checksum] = entry["size"]
            else:
                formal._require(entry["size"] == 0, "managed_sealed_inventory_invalid",
                                "Deleted source cannot carry object bytes")
            actual[name] = checksum
            paths.append(entry["path"])
        formal._require(actual == expected and paths == sorted(paths), "managed_sealed_inventory_invalid",
                        "Sealed manifest omits, adds or reorders captured source paths")

    checked_manifest(manifest, hashes, management.repo, record["baseHead"], product=True)
    external, captures = inputs.get("externalInputs"), sealed["externalCapture"]
    roots = inputs.get("externalRoots")
    formal._require(isinstance(external, list) and isinstance(captures, list) and isinstance(roots, list)
                    and len(external) == len(captures) == len(roots) <= 1,
                    "external_scope_invalid", "Sealed inventory must retain its exact optional sibling")
    captured_gitlinks = {}

    def checked_capture(source, prefix=""):
        checked_source_evidence(source)
        for name, value in source["files"].items():
            if isinstance(value, dict):
                formal._require(set(value) == {"gitlinkCommit", "source"}
                                and isinstance(value.get("gitlinkCommit"), str)
                                and re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", value["gitlinkCommit"]),
                                "external_capture_changed", "Captured Gitlink identity differs")
                relative = prefix + _managed_relative(name)
                checked_capture(value["source"], relative + "/")
                captured_gitlinks[relative] = {"indexCommit": value["gitlinkCommit"], "head": value["source"]["head"]}

    frozen_external = sources(manifest)
    formal._require(len(frozen_external) == len(external), "external_scope_invalid",
                    "Sealed dependency closure differs from its inventory")
    for value, capture, frozen, raw_root in zip(external, captures, frozen_external, roots):
        expected_root = management.repo.parent / "zr_vm"
        formal._require(isinstance(value, dict)
                        and set(value) == {"name", "repositoryRoot", "baseHead", "currentHashes", "untrackedPaths"}
                        and value.get("name") == "zr_vm" and value.get("repositoryRoot") == str(expected_root)
                        and isinstance(raw_root, str) and Path(raw_root).is_absolute()
                        and Path(raw_root).absolute() == expected_root
                        and isinstance(capture, dict) and set(capture) == {"path", "source"}
                        and capture.get("path") == str(expected_root) and isinstance(capture.get("source"), dict)
                        and frozen.get("kind") == KIND and frozen.get("repositoryRoot") == str(expected_root),
                        "external_scope_invalid", "Sealed sibling identity or physical source reference differs")
        expected = _managed_hashes(None, value["currentHashes"], "sealed-external-scope")
        formal._require(not any(name.split("/", 1)[0] in {".git", ".codex", ".jenkins", ".opencode", "target"}
                                for name in expected), "external_control_forbidden",
                        "Sealed external closure contains checkout controls")
        untracked = value["untrackedPaths"]
        formal._require(isinstance(untracked, list) and all(isinstance(name, str) for name in untracked)
                        and untracked == sorted(set(untracked)) and set(untracked) <= set(expected)
                        and all(_managed_relative(name).casefold() == name for name in untracked),
                        "external_scope_invalid", "Sealed untracked membership is not its exact canonical subset")
        source = capture["source"]
        checked_capture(source)
        formal._require(source["head"] == value["baseHead"]
                        and source["sourceDigest"] == digest({key: item for key, item in source.items() if key != "sourceDigest"})
                        and {name.casefold(): checksum for name, checksum in source_files(source).items()} == expected,
                        "external_capture_changed", "Sealed external snapshot differs from its original captured hashes")
        checked_manifest(frozen["manifest"], expected, expected_root, value["baseHead"])
        formal._require(digest(frozen["manifest"]) == frozen.get("sourceDigest"),
                        "external_capture_changed", "Sealed dependency manifest digest differs")
    formal._require(inputs.get("externalGitlinks") == captured_gitlinks,
                    "external_capture_changed", "Sealed dependency snapshot differs from its registered Gitlinks")
    names = ["inputs/objects/" + checksum for checksum in sorted(objects)]
    observed = _managed_checksums(object_root, names, "sealed-inventory-objects")
    formal._require(all(observed[name] == name.rsplit("/", 1)[1] for name in names),
                    "managed_source_object_changed", "A sealed source object is absent or corrupt")
    return manifest, object_root, external, captures



def prepared_validation_context(management, root, engine, *, payload=None):
    """Resolve immutable prepared CAS provenance without observing its old source roots."""
    from types import SimpleNamespace
    from tools.jenkins import managed_worktree as formal
    from tools.jenkins.resources.paths import canonical_build_root, physical_path_under
    payload = management.payload if payload is None else payload
    state, root = management.state, Path(root)
    reference = payload.get("worktreePreparationRef")
    row, operation = _managed_route_preparation(state, reference, payload.get("worktreeId"),
                                               management.repo_id, management.config)
    prepared = row["payload"]
    worktree = state.get("management_worktree", payload["worktreeId"])
    record = worktree["payload"] if worktree else {}
    formal._require(payload.get("sourceView") == "worktree"
        and full_file_index_requested(payload)
        and prepared.get("sourceKind") == "sealed-observed-canonical-input"
        and prepared.get("sourcePreparation") is True
        and prepared.get("implementationOwnershipTransferred") is False
        and prepared.get("path") == str(root) == record.get("path")
        and prepared.get("writerSessionId") == management.session == record.get("sessionId")
        and record.get("status") == "ready"
        and all(record.get(name) == prepared.get(name) for name in
                ("repositoryId", "driverDigest", "runtimeOperationId", "baseHead", "writerAuthorizationSnapshot"))
        and sorted(management.owned) == sorted(record.get("ownedPaths", [])),
        "prepared_validation_unproven", "Validation requires its exact complete sealed worktree preparation")
    checked_source_evidence(engine)
    files = source_files(engine)
    formal._require(engine.get("sourceEvidencePolicy") == _SOURCE_EVIDENCE_V2
        and engine == prepared.get("source") and engine["head"] == record["baseHead"]
        and len({name.casefold() for name in files}) == len(files),
        "prepared_validation_source_changed", "The complete WT HEAD, index or file evidence differs from preparation")
    approved = canonical_build_root(management.build_root())
    objects = physical_path_under(approved, prepared.get("objectRoot", ""), allow_missing=False)
    formal._require(objects == physical_path_under(approved, approved.namespace(), allow_missing=False),
        "prepared_validation_store_changed", "Prepared CAS differs from the selected physical build namespace")
    inspection, inputs = formal._inventory_reference(state, objects, prepared["inspectionOperationKey"],
        prepared["preparationInputRef"], payload["worktreeId"], management.repo_id,
        management.config["driverDigest"], management.config["runtimeOperationId"])
    manifest, actual_objects, external, captures = _managed_sealed_inventory(
        management, record, inputs, inspection["payload"]["payload"])
    formal._require(actual_objects == objects
        and prepared.get("sourceManifest") == manifest
        and prepared.get("externalInputs") == external and prepared.get("externalRoots") == captures
        and all(prepared.get(name) == inputs.get(name) for name in
                ("sourceHead", "currentHashes", "beforeHashes", "compilerControlInputs"))
        and payload.get("externalRoots", []) == [value["path"] for value in captures]
        and ("externalGitlinks" not in payload or payload["externalGitlinks"] == inputs.get("externalGitlinks")),
        "prepared_validation_inventory_changed", "Prepared validation inventory or explicit dependency binding differs")
    reader_actor = SimpleNamespace(state=state, repo_id=management.repo_id,
        session=prepared["readerSessionId"], owned=management.owned)
    reader = formal._reader(reader_actor, record, inputs["currentHashes"], external)
    formal._require(reader["snapshotDigest"] == prepared.get("readerAuthorizationSnapshot"),
        "prepared_validation_authority_changed", "The original narrow writer or exact reader snapshot changed")
    pins = [formal._pin(value) for value in (row, operation, worktree, inspection)]
    formal._require(formal._pins_current(state, pins),
        "prepared_validation_authority_changed", "Prepared input authority changed during CAS verification")
    proof = {"kind": "prepared-sealed-CAS", "worktreePreparationRef": reference,
        "worktreeId": payload["worktreeId"], "authorityPins": pins,
        "objectRoot": str(objects), "buildRoot": str(approved.path),
        "sourceManifestDigest": digest(manifest), "preparedEngineDigest": engine["sourceDigest"],
        "externalCaptureDigest": digest(captures),
        "externalManifestDigests": [value["sourceDigest"] for value in manifest["external"]],
        "liveExternalObserved": False}
    return {"manifest": manifest, "objects": objects, "captures": captures, "provenance": proof}


def prepared_execution_manifest(management, root, engine, context):
    """Reuse verified CAS bytes and seal only extra current WT members."""
    original = context["manifest"]
    files = source_files(engine)
    names = {name.casefold(): name for name in files}
    prior = {entry["path"].casefold(): entry for entry in original["entries"]}
    entries, added = [], []
    for folded, name in sorted(names.items()):
        entry = prior.get(folded)
        if entry is None:
            added.append(name)
        else:
            if entry["sha256"] != files[name]:
                raise ValueError("Prepared engine intersection differs from its exact CAS bytes")
            entries.append(dict(entry))
    if added:
        expected = {name.casefold(): files[name] for name in added}
        hashes, metadata = _managed_capture_files(root, added, context["objects"],
            "prepared-execution-extra-inputs", expected=expected)
        extra = _managed_captured_manifest(root, engine["head"], hashes, metadata)
        entries.extend(extra["entries"])
    manifest = {**original, "entries": sorted(entries, key=lambda entry: entry["path"])}
    verify_sealed_capture(engine, manifest)
    if management.source(root, payload=management.payload) != engine:
        raise ValueError("The complete WT source changed while sealing execution inputs")
    return manifest

def _managed_prewrite_check(management, target, before, hashes, object_root):
    from tools.jenkins import managed_worktree as formal
    with managed_input_stage(target, "target-prewrite") as targets:
        with managed_input_stage(object_root, "cas-prewrite") as objects:
            def checked(path):
                formal._require(targets.checksum(path) == before[path],
                                "managed_worktree_target_changed", "Target changed after source sealing")
                checksum = hashes[path]
                if checksum is not None:
                    formal._require(objects.checksum("inputs/objects/" + checksum) == checksum,
                                    "managed_source_object_changed", "A sealed source object is absent or corrupt")
            with ThreadPoolExecutor(max_workers=4) as readers:
                names = list(hashes)
                for start in range(0, len(names), 128):
                    list(readers.map(checked, names[start:start + 128]))
                    if start and start % 4096 == 0:
                        print("[management] target/CAS prewrite checked " + str(start)
                              + "/" + str(len(names)), flush=True)


def managed_inspect_worktree(management, *, preparation_helper):
    """Capsule-owned input inspection; preserve the service reader receipt contract."""

    from tools.jenkins import managed_worktree as formal
    _require, digest, _plain = formal._require, formal.digest, formal._plain
    canonical_build_root = formal.canonical_build_root
    compiler_controls, require_product_inputs = formal.compiler_controls, formal.require_product_inputs
    _target_source, _external_capture = formal._target_source, formal._external_capture
    assert_no_writer = formal.assert_no_writer
    SOURCE_EVIDENCE_MAX_BYTES = formal.SOURCE_EVIDENCE_MAX_BYTES
    COMPILER_CONTROL_PATHS = formal.COMPILER_CONTROL_PATHS
    record, target = management.wt(management.payload["worktreeId"], own=False)
    caller = management.state.check_authorization(management.repo_id, management.session,
                                                   "validation", management.owned)
    writer = management.state.check_authorization(management.repo_id, record["sessionId"],
                                                   "implementation", record["ownedPaths"])
    _require(record.get("status") == "ready" and record.get("repositoryId") == management.repo_id
             and record.get("driverDigest") == management.config["driverDigest"]
             and record.get("runtimeOperationId") == management.config["runtimeOperationId"]
             and record.get("baseHead") == management.text(target, "rev-parse", "HEAD")
             and caller["principal"] == writer["principal"] == record.get("writerPrincipal")
             and writer["snapshotDigest"] == record.get("writerAuthorizationSnapshot")
             and sorted(management.owned) == sorted(record["ownedPaths"]) == sorted(writer["paths"]),
             "managed_inventory_authority_invalid", "Inspection retains the actual narrow WT owner and current runtime")
    assert_no_writer(management.state, management.payload["worktreeId"])
    prior_row, prior = _target_source(management, management.payload.get("targetSourceOperationKey"))
    _require(matching_target_source(management, target, prior) == prior, "managed_worktree_target_changed",
             "Target must retain its exact previous complete source operation")
    head = management.text(management.repo, "rev-parse", "HEAD")
    membership = _managed_membership(management, record["baseHead"], prior)
    expected_owned = _managed_owned_hashes(management.payload, record["ownedPaths"])
    _require(set(record["ownedPaths"]) <= membership, "managed_owned_scope_invalid",
             "Every owned path must belong to the complete captured closure")
    before = _managed_checksums(target, sorted(membership), "target-before")
    object_root = canonical_build_root(management.build_root()).namespace()
    report = management.outputs() / "reports" / management.key
    manifest_ref = management.artifact_manifest(target, {"reports": report, "inputObjects": object_root / "inputs"})
    hashes, metadata = _managed_capture_files(management.repo, sorted(membership), object_root,
                                             "canonical-capture", expected=expected_owned)
    controls = {name: hashes[name] for name in sorted(COMPILER_CONTROL_PATHS)}
    compiler_controls(controls, hashes, record["ownedPaths"], managed=True)
    require_product_inputs(hashes, controls)
    capture = _ManagedCapturedExternal(management, object_root)
    external, external_captures, gitlinks = _external_capture(capture, preparation_helper, inspection=True)
    if "externalGitlinks" in management.payload:
        _require(isinstance(management.payload["externalGitlinks"], dict)
                 and management.payload["externalGitlinks"] == gitlinks,
                 "external_scope_invalid", "Declared Gitlinks must match every actual index and HEAD exactly")
    from tools.jenkins.source.external_inputs import KIND
    frozen_external = []
    for value, captured_external in zip(external, external_captures):
        root, source = Path(captured_external["path"]), captured_external["source"]
        manifest = _managed_captured_manifest(root, source["head"], value["currentHashes"],
                                             capture.flattened_metadata(root, source))
        preparation_helper.verify_sealed_capture(source, manifest)
        frozen_external.append({"kind": KIND, "name": "zr_vm", "repositoryRoot": str(root),
                                "manifest": manifest, "sourceDigest": digest(manifest)})
    source_manifest = _managed_captured_manifest(management.repo, record["baseHead"], hashes, metadata,
                                                external=frozen_external, product=True)
    sealed_source = {"kind": _MANAGED_SEALED_INVENTORY, "objectRoot": str(object_root),
                     "sourceManifest": source_manifest, "sourceDigest": digest(source_manifest),
                     "externalCapture": external_captures}
    value = {"sourceHead": head, "currentHashes": hashes, "beforeHashes": before,
             "compilerControlInputs": controls, "externalRoots": management.payload["externalRoots"],
             "externalGitlinks": gitlinks, "externalInputs": external,
             "targetSourceOperationKey": prior_row["key"], "sealedSource": sealed_source}
    _managed_sealed_inventory(management, record, value, management.payload)
    _require(management.text(management.repo, "rev-parse", "HEAD") == head,
             "managed_source_changed", "Canonical Git base changed during captured input inspection")
    _require(matching_target_source(management, target, prior) == prior, "managed_worktree_target_changed",
             "Target changed during input inspection")
    assert_no_writer(management.state, management.payload["worktreeId"])
    path = report / "source-inventory.json"
    _plain(path)
    report.mkdir(parents=True, exist_ok=True)
    inventory = {"preparationInput": value, "inventoryDigest": digest(value),
                 "externalCapture": external_captures, "targetSourceDigest": prior["sourceDigest"]}
    raw = json.dumps(inventory, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
    _require(len(raw) <= SOURCE_EVIDENCE_MAX_BYTES, "managed_inventory_size",
             "Source inventory exceeds the fixed 64 MiB evidence bound")
    with path.open("xb") as stream:
        stream.write(raw)
        stream.flush()
        os.fsync(stream.fileno())
    reference = {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}
    return {"status": "complete", "worktreeId": management.payload["worktreeId"],
            "preparationInputRef": reference, "inventoryDigest": digest(value),
            "artifactManifest": manifest_ref, "targetSourceDigest": prior["sourceDigest"],
            "sourceAcceptanceCreated": False, "implementationOwnershipTransferred": False,
            "canonicalSourceModified": False, "commitPerformed": False, "pushPerformed": False,
            "formalAcceptance": False}


def _recovery_document(management, reference, *, roots=None, json_document=True):
    from tools.jenkins import managed_worktree as formal
    roots = roots or [management.repo / ".jenkins/state/management/reports"]
    formal._require(isinstance(reference, dict) and set(reference) <= {"path", "sha256", "size", "bytes"}
                    and isinstance(reference.get("path"), str) and isinstance(reference.get("sha256"), str)
                    and re.fullmatch(r"[0-9a-f]{64}", reference["sha256"]),
                    "managed_recovery_terminal_unproven", "Recovery requires an exact evidence reference")
    path = Path(reference["path"])
    formal._require(path.is_absolute() and any(path.is_relative_to(root) for root in roots),
                    "managed_recovery_terminal_unproven", "Recovery evidence escapes its formal report location")
    formal._plain(path)
    formal._require(path.stat().st_size <= 64 * 1024**2,
                    "managed_recovery_terminal_unproven", "Recovery evidence exceeds its bounded input")
    raw = path.read_bytes()
    formal._require(hashlib.sha256(raw).hexdigest() == reference["sha256"]
                    and all(reference[name] == len(raw) for name in ("size", "bytes") if name in reference),
                    "managed_recovery_terminal_unproven", "Recovery evidence differs from its exact bytes")
    return json.loads(raw) if json_document else raw


def _native_original_absent(identity):
    """Reject a live original birth through one retained query handle, including PID reuse."""
    import ctypes
    from ctypes import wintypes
    from tools.jenkins import managed_worktree as formal
    formal._require(type(identity.get("pid")) is int and identity["pid"] > 0
                    and isinstance(identity.get("creationTime"), str)
                    and identity["creationTime"].isascii() and identity["creationTime"].isdigit()
                    and int(identity["creationTime"]) > 0
                    and isinstance(identity.get("executable"), str) and Path(identity["executable"]).is_absolute(),
                    "managed_recovery_terminal_unproven", "Recovery native identity is incomplete")
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    kernel.OpenProcess.restype = wintypes.HANDLE
    kernel.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
    kernel.GetProcessTimes.restype = wintypes.BOOL
    kernel.GetExitCodeProcess.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.DWORD)]
    kernel.GetExitCodeProcess.restype = wintypes.BOOL
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel.CloseHandle.restype = wintypes.BOOL
    handle = kernel.OpenProcess(0x1000, False, identity["pid"])
    if not handle:
        error = ctypes.get_last_error()
        formal._require(error in {87, 1168}, "managed_recovery_terminal_unproven",
                        "Original process cannot be proven absent through Windows")
        return
    try:
        times = [wintypes.FILETIME() for _ in range(4)]
        if not kernel.GetProcessTimes(handle, *[ctypes.byref(value) for value in times]):
            raise ctypes.WinError(ctypes.get_last_error())
        birth = str((times[0].dwHighDateTime << 32) | times[0].dwLowDateTime)
        if birth != identity["creationTime"]:
            return
        status = wintypes.DWORD()
        if not kernel.GetExitCodeProcess(handle, ctypes.byref(status)):
            raise ctypes.WinError(ctypes.get_last_error())
        formal._require(status.value != 259, "managed_recovery_original_active",
                        "The original native process is still active")
    finally:
        kernel.CloseHandle(handle)


def _recovery_no_descendants(native):
    import ctypes
    from ctypes import wintypes
    from tools.jenkins import managed_worktree as formal

    class Entry(ctypes.Structure):
        _fields_ = [("dwSize", wintypes.DWORD), ("cntUsage", wintypes.DWORD),
            ("th32ProcessID", wintypes.DWORD), ("th32DefaultHeapID", ctypes.c_size_t),
            ("th32ModuleID", wintypes.DWORD), ("cntThreads", wintypes.DWORD),
            ("th32ParentProcessID", wintypes.DWORD), ("pcPriClassBase", wintypes.LONG),
            ("dwFlags", wintypes.DWORD), ("szExeFile", wintypes.WCHAR * 260)]

    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.CreateToolhelp32Snapshot.argtypes = [wintypes.DWORD, wintypes.DWORD]
    kernel.CreateToolhelp32Snapshot.restype = wintypes.HANDLE
    for name in ("Process32FirstW", "Process32NextW"):
        getattr(kernel, name).argtypes = [wintypes.HANDLE, ctypes.POINTER(Entry)]
        getattr(kernel, name).restype = wintypes.BOOL
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    handle = kernel.CreateToolhelp32Snapshot(2, 0)
    if handle == ctypes.c_void_p(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        entry, parents = Entry(), {item["pid"] for item in native}
        entry.dwSize = ctypes.sizeof(entry)
        more = kernel.Process32FirstW(handle, ctypes.byref(entry))
        while more:
            formal._require(entry.th32ParentProcessID not in parents,
                            "managed_recovery_original_active", "An original native descendant remains active")
            more = kernel.Process32NextW(handle, ctypes.byref(entry))
        if ctypes.get_last_error() != 18:
            raise ctypes.WinError(ctypes.get_last_error())
    finally:
        kernel.CloseHandle(handle)


def _recovery_terminal(management, operation, preparation):
    from tools.jenkins import managed_worktree as formal
    from tools.jenkins.deployment.drain_observations import owner_observation
    proof = _recovery_document(management, management.payload.get("prepareRecoveryTerminalRef"))
    op = operation["payload"]
    original, key = op["payload"], operation["key"]
    build = proof.get("build", {})
    formal._require(proof.get("schemaVersion") == 1
        and proof.get("kind") == "jenkins-prepare-recovery-terminal-v1"
        and proof.get("status") == "original-attempt-terminal"
        and all(proof.get(name) == original.get(name) for name in
                ("worktreeId", "driverDigest", "runtimeOperationId"))
        and proof.get("operationKey") == key and build.get("job") == "zircon-management"
        and type(build.get("number")) is int and build["number"] > 0
        and type(build.get("queueId")) is int and build["queueId"] > 0
        and build.get("building") is False and build.get("result") in {"FAILURE", "ABORTED"}
        and proof.get("fullPipelineFinished") is True and proof.get("serviceOwnerUnchanged") is True
        and proof.get("keeperUnchanged") is True
        and proof.get("recordCurrentOperationPin") == {"key": key, "version": operation["version"],
            "payloadDigest": digest(op), "inputDigest": op["inputDigest"]}
        and proof.get("originalPreparationPin") == {"key": key, "version": preparation["version"],
            "payloadDigest": digest(preparation["payload"])},
        "managed_recovery_terminal_unproven", "Terminal proof differs from the original bound operation and preparation")
    owner = owner_observation(management.manager)
    formal._require(proof.get("serviceOwner") == owner
                    and owner["driverDigest"] == management.config["driverDigest"]
                    and owner["operationId"] == management.config["runtimeOperationId"],
                    "managed_recovery_terminal_unproven", "Recovery must retain the verified lifecycle owner")
    url = "job/zircon-management/" + str(build["number"]) + "/"
    status, raw = management.manager._request(url + "api/json?tree=number,queueId,building,result")
    current = json.loads(raw)
    formal._require(status == 200 and all(current.get(name) == build[name] for name in
                    ("number", "queueId", "building", "result")),
                    "managed_recovery_terminal_unproven", "The original Jenkins build is not actually terminal")
    console = _recovery_document(management, proof.get("consoleRef"), json_document=False)
    status, actual_console = management.manager._request(url + "consoleText")
    formal._require(status == 200 and actual_console == console
                    and b"[Pipeline] End of Pipeline" in console
                    and console.rstrip().endswith(("Finished: " + build["result"]).encode("ascii"))
                    and original["operationId"].encode("utf-8") in console and key.encode("ascii") in console,
                    "managed_recovery_terminal_unproven", "Original Pipeline EOF or identity is unproven")
    local = _recovery_document(management, proof.get("receiptRef"),
        roots=[management.repo / ".jenkins/state/controls"], json_document=False)
    home_root = management.repo / ".jenkins/jenkins_home/jobs/zircon-management/builds" / str(build["number"]) / "archive"
    home = _recovery_document(management, proof.get("homeReceiptRef"), roots=[home_root], json_document=False)
    home_path = Path(proof["homeReceiptRef"]["path"])
    formal._require(home_path.parent == home_root, "managed_recovery_terminal_unproven",
                    "Original artifact must be in its exact Jenkins build")
    status, api = management.manager._request(url + "artifact/" + home_path.name)
    receipt = json.loads(local)
    formal._require(status == 200 and api == home == local
                    and all(receipt.get(name) == value for name, value in op["result"].items()),
                    "managed_recovery_terminal_unproven", "Original result differs across State, local, API and Home")
    observation = _recovery_document(management, proof.get("originalNativeObservationRef"))
    native = proof.get("native")
    fields = ("role", "pid", "creationTime", "executable")
    formal._require(isinstance(native, list) and len(native) == 3
                    and {item.get("role") for item in native} == {"runner", "git-wrapper", "git-child"}
                    and len({item.get("pid") for item in native}) == 3
                    and observation.get("kind") == "recorded-earlier-live-process-observation"
                    and observation.get("operationKey") == key
                    and observation.get("buildNumber") == build["number"]
                    and observation.get("queueId") == build["queueId"]
                    and [{name: item.get(name) for name in fields} for item in observation.get("native", [])]
                        == [{name: item.get(name) for name in fields} for item in native]
                    and all(item.get("observation") == "original-process-absent"
                        and item.get("retainedHandleSignaled") is None and item.get("exitCode") is None for item in native),
                    "managed_recovery_terminal_unproven", "Recovery needs the original independently observed native births")
    for identity in native:
        _native_original_absent(identity)
    _recovery_no_descendants(native)
    return proof


@contextmanager
def materialized_preparation_recovery_hold(management):
    """Keep the interrupted reservation intact until one atomic read-only reconciliation."""
    from tools.jenkins import managed_worktree as formal
    from tools.jenkins.state.locks import process_lock
    key = management.payload.get("resumePreparationFromOperationKey")
    formal._require(full_file_index_requested(management.payload)
                    and isinstance(key, str) and re.fullmatch(r"[0-9a-f]{64}", key)
                    and key != management.key,
                    "managed_recovery_identity_invalid", "Preparation recovery requires a new v2 request identity")
    with process_lock(management.repo / ".jenkins/state/locks" / ("management-operation-" + key + ".lock"), timeout=0):
        operation = management.state.get("management_operation", key)
        preparation = management.state.get("managed_worktree_preparation", key)
        op, prepared = operation["payload"] if operation else {}, preparation["payload"] if preparation else {}
        original = op.get("payload", {})
        formal._require(op.get("phase") == "unknown" and op.get("action") == "prepare-worktree"
            and op.get("result", {}).get("status") == "unknown"
            and op.get("result", {}).get("reasonCode") == "management_operation_interrupted"
            and prepared.get("status") == "materializing"
            and key == digest({name: original.get(name) for name in
                               ("repositoryId", "sessionId", "requestId", "operationId")})
            and op.get("inputDigest") == digest({name: value for name, value in original.items() if name != "observationOnly"})
            and prepared.get("inputDigest") == op["inputDigest"]
            and all(original.get(name) == management.payload.get(name) for name in
                ("repositoryId", "sessionId", "driverDigest", "runtimeOperationId", "worktreeId",
                 "inspectionOperationKey", "preparationInputRef", "ownedPaths", "buildRoot"))
            and prepared.get("readerSessionId") == management.session,
            "managed_recovery_identity_invalid", "Recovery must retain the interrupted request's exact input and reader")
        writer = management.state.get("managed_worktree_writer", management.payload["worktreeId"])
        expected = {"worktreeId": management.payload["worktreeId"], "operationKey": key,
                    "inputDigest": op["inputDigest"], "status": "interrupted"}
        formal._require(writer is not None and writer["payload"] == expected,
                        "managed_recovery_writer_changed", "Only the original interrupted reservation can be reconciled")
        record, target = management.wt(management.payload["worktreeId"], own=False)
        formal._require(record["path"] == prepared.get("path") and record["baseHead"] == prepared.get("baseHead")
            and record["sessionId"] == prepared.get("writerSessionId")
            and record["writerAuthorizationSnapshot"] == prepared.get("writerAuthorizationSnapshot"),
            "managed_recovery_identity_invalid", "Recovery retains the original worktree and writer")
        reader = formal._reader(management, record, prepared["currentHashes"], prepared["externalInputs"])
        formal._require(reader["snapshotDigest"] == prepared.get("readerAuthorizationSnapshot"),
                        "managed_recovery_authority_changed", "Original reader snapshot changed")
        from tools.jenkins.managed_worktree_lifetime import active_readers
        formal._require(not active_readers(management.state, management.payload["worktreeId"], target),
                        "managed_worktree_reader_active", "Existing source readers still freeze this worktree")
        proof = _recovery_terminal(management, operation, preparation)
        management._materialized_recovery_context = {
            "operation": operation, "preparation": preparation, "writer": writer, "proof": proof}
        try:
            yield
        finally:
            del management._materialized_recovery_context


def _materialized_index(management, target, source, hashes):
    """Retain every base index entry; only the interrupted intent-to-add closure may be new."""
    from tools.jenkins import managed_worktree as formal
    entries = {}
    raw = management.git(target, "ls-tree", "-r", "-z", "HEAD").stdout
    for row in raw.split(b"\0"):
        if row:
            fields, path = row.split(b"\t", 1)
            mode, _kind, obj = fields.decode("ascii").split()
            name = path.decode("utf-8", "strict")
            entries[name.casefold()] = {"path": name, "mode": mode, "object": obj, "stage": 0}
    actual = source["gitIndex"]["entries"]
    formal._require(all(entry["stage"] == 0 for entry in actual),
                    "managed_recovery_index_changed", "Recovery rejects unresolved index stages")
    observed = {entry["path"].casefold(): entry for entry in actual}
    added = {path for path, checksum in hashes.items() if checksum is not None and path not in entries}
    empty = (hashlib.sha1 if len(source["head"]) == 40 else hashlib.sha256)(b"blob 0\0").hexdigest()
    formal._require(len(observed) == len(actual) and set(observed) == set(entries) | added
        and all(observed[name] == entry for name, entry in entries.items())
        and all(observed[name]["object"] == empty and observed[name]["mode"] in {"100644", "100755"}
                for name in added),
        "managed_recovery_index_changed", "Worktree index differs from its base plus the declared intent-to-add paths")


def resume_materialized_preparation(management, record, target, inspection_row, inputs, *,
                                    source_manifest, object_root, external, external_captures, reader):
    from tools.jenkins import managed_worktree as formal
    from tools.jenkins.managed_worktree_lifetime import active_readers
    context = getattr(management, "_materialized_recovery_context", None)
    formal._require(context is not None and "sealedSource" in inputs,
                    "managed_recovery_identity_invalid", "Recovery requires its locked original reservation and sealed inventory")
    operation, old_row, writer = context["operation"], context["preparation"], context["writer"]
    old, hashes = old_row["payload"], inputs["currentHashes"]
    equal = {"worktreeId": management.payload["worktreeId"], "path": str(target), "baseHead": record["baseHead"],
        "repositoryId": management.repo_id, "driverDigest": management.config["driverDigest"],
        "runtimeOperationId": management.config["runtimeOperationId"], "inspectionOperationKey": inspection_row["key"],
        "preparationInputRef": management.payload["preparationInputRef"], "sourceManifest": source_manifest,
        "objectRoot": str(object_root), "externalInputs": external, "externalRoots": external_captures,
        "readerSessionId": management.session, "readerAuthorizationSnapshot": reader["snapshotDigest"],
        "writerSessionId": record["sessionId"], "writerAuthorizationSnapshot": record["writerAuthorizationSnapshot"],
        **{name: inputs[name] for name in ("sourceHead", "currentHashes", "beforeHashes", "compilerControlInputs")}}
    formal._require(all(old.get(name) == value for name, value in equal.items())
                    and old.get("sourceKind") == "sealed-observed-canonical-input",
                    "managed_recovery_input_changed", "Recovery differs from the original sealed closure or authority")
    prior_row, prior = formal._target_source(management, inputs.get("targetSourceOperationKey"))
    formal._require(prior["head"] == record["baseHead"], "managed_worktree_base_changed", "Original base differs")
    worktree_row = management.state.get("management_worktree", management.payload["worktreeId"])
    postimages = _managed_checksums(target, hashes, "recovery-postimages")
    _managed_drift(hashes, postimages, "managed_worktree_postimage_changed",
                   "Interrupted materialization has an incomplete or changed postimage")
    source = management.source(target)
    formal._require(source.get("sourceEvidencePolicy") == _SOURCE_EVIDENCE_V2 and source["head"] == record["baseHead"],
                    "managed_recovery_index_changed", "Recovery needs the complete current v2 source and original HEAD")
    checked_source_evidence(source)
    expected = {name.casefold(): value for name, value in source_files(prior).items()}
    expected.update(hashes)
    observed = {name.casefold(): value for name, value in source_files(source).items()}
    formal._require(all(name in expected and expected[name] == value for name, value in observed.items())
                    and all(observed.get(name) == value for name, value in expected.items()),
                    "managed_worktree_postimage_changed", "Recovery rejects foreign changes outside the complete postimage closure")
    _materialized_index(management, target, source, hashes)
    _recovery_terminal(management, operation, old_row)
    pins = [formal._pin(row) for row in (operation, old_row, writer, prior_row, worktree_row, inspection_row)]
    resumed = {"operationKey": operation["key"], "operationPin": formal._pin(operation),
               "preparationPin": formal._pin(old_row), "writerPin": formal._pin(writer),
               "writerPayload": writer["payload"], "terminalRef": management.payload["prepareRecoveryTerminalRef"]}
    binding = {"worktreeId": management.payload["worktreeId"], "operationKey": management.key,
               "inputDigest": management.input_digest, "status": "active", "resumedFrom": resumed}
    with management.state.transaction() as connection:
        current_reader = formal._reader(management, record, hashes, external, connection=connection)
        formal._require(current_reader["snapshotDigest"] == reader["snapshotDigest"]
            and formal._pins_current(management.state, pins, connection=connection)
            and not active_readers(management.state, management.payload["worktreeId"], target, connection=connection)
            and management.state.get("managed_worktree_preparation", management.key, connection=connection) is None,
            "managed_recovery_authority_changed", "Recovery authority changed before its atomic completion")
        reserved = management.state.put("managed_worktree_writer", management.payload["worktreeId"], binding,
            expected_version=writer["version"], connection=connection)
        management.state.put("managed_worktree_preparation", management.key, {
            **old, "status": "complete", "inputDigest": management.input_digest, "source": source,
            "sourceEvidencePolicy": _SOURCE_EVIDENCE_V2, "resumedFrom": resumed,
            "sourceFilesRewritten": False}, expected_version=0, connection=connection)
        management.state.put("managed_worktree_writer", management.payload["worktreeId"],
            {**binding, "status": "released"}, expected_version=reserved["version"], connection=connection)
    return {"status": "complete", "worktreeId": management.payload["worktreeId"],
        "worktreePreparationRef": management.key, "source": source, "beforeHashes": old["beforeHashes"],
        "afterHashes": hashes, "sourceHead": old["sourceHead"], "baseHead": old["baseHead"],
        "sealedCandidateDigest": digest(source_manifest), "sourcePreparation": True,
        "resumedFrom": resumed, "sourceFilesRewritten": False, "implementationOwnershipTransferred": False,
        "canonicalSourceModified": False, "commitPerformed": False, "pushPerformed": False, "formalAcceptance": False}


def managed_prepare_worktree(management, *, preparation_helper):
    """Capsule-owned managed preparation with the existing State and CAS authorities."""

    from tools.jenkins import managed_worktree as formal
    _require, digest, _plain = formal._require, formal.digest, formal._plain
    canonical_build_root = formal.canonical_build_root
    compiler_controls, require_product_inputs = formal.compiler_controls, formal.require_product_inputs
    _target_source, _external_capture = formal._target_source, formal._external_capture

    _inventory_reference, _reader = formal._inventory_reference, formal._reader
    validate_hash_paths = formal.validate_hash_paths
    _pin, _pins_current = formal._pin, formal._pins_current
    PathClaim, seal_candidate, _thaw = formal.PathClaim, formal.seal_candidate, formal._thaw
    _seal_external = formal._seal_external
    reference = management.payload["worktreeId"]
    inspection_row, inputs = _inventory_reference(management.state,
        canonical_build_root(management.build_root()).namespace(),
        management.payload.get("inspectionOperationKey"), management.payload.get("preparationInputRef"),
        reference, management.repo_id, management.config["driverDigest"], management.config["runtimeOperationId"])
    record, target = management.wt(reference, own=False)
    _require(record.get("status") == "ready" and record.get("repositoryId") == management.repo_id
             and record.get("driverDigest") == management.config["driverDigest"]
             and record.get("runtimeOperationId") == management.config["runtimeOperationId"]
             and management.text(target, "rev-parse", "HEAD") == record.get("baseHead"),
             "managed_worktree_unproven", "Source preparation needs the exact ready current-runtime worktree")
    sealed_inventory = "sealedSource" in inputs
    hashes = _managed_hashes(None if sealed_inventory else management.repo, inputs.get("currentHashes"), "declared-current-scope")
    controls = compiler_controls(inputs.get("compilerControlInputs"), hashes,
                                  record["ownedPaths"], managed=True)
    _require(controls is not None, "compiler_control_scope_invalid", "Complete product controls are explicit")
    validate_hash_paths(hashes, controls)
    require_product_inputs(hashes, controls)
    head = inputs.get("sourceHead")
    _require(isinstance(head, str) and len(head) in {40, 64}
             and all(c in "0123456789abcdef" for c in head),
             "managed_source_head_missing", "Canonical input HEAD must be immutable")
    if sealed_inventory:
        source_manifest, object_root, external, external_captures = _managed_sealed_inventory(
            management, record, inputs, inspection_row["payload"]["payload"])
    else:
        external, external_captures, _ = _external_capture(management, preparation_helper, payload=inputs)
    reader = _reader(management, record, hashes, external)
    if "resumePreparationFromOperationKey" in management.payload:
        return resume_materialized_preparation(management, record, target, inspection_row, inputs,
            source_manifest=source_manifest if sealed_inventory else None, object_root=object_root if sealed_inventory else None,
            external=external, external_captures=external_captures, reader=reader)
    prior_row, prior = _target_source(management, inputs.get("targetSourceOperationKey"))
    worktree_row = management.state.get("management_worktree", reference)
    _require(prior.get("head") == record["baseHead"], "managed_worktree_base_changed",
             "The previous source receipt must retain this WT base")
    _require(matching_target_source(management, target, prior) == prior,
             "managed_worktree_target_changed", "Target differs from its previous complete source receipt")
    before = _managed_hashes(None if sealed_inventory else management.repo, inputs.get("beforeHashes"), "declared-before-scope")
    _require(set(before) == set(hashes), "managed_source_scope_invalid", "Before/post scopes must be identical")
    _require(all(len(str(target / path)) < 260 for path, checksum in hashes.items()
                 if checksum is not None and checksum != before[path]),
             "managed_worktree_path_invalid", "Target file write violates its physical path budget")
    actual_before = _managed_checksums(target, before, "target-preimages")
    _managed_drift(before, actual_before, "managed_worktree_target_changed",
                   "Target preimage differs from the declared before hash")
    if not sealed_inventory:
        _managed_verify_canonical(management, head, hashes, record["baseHead"], prior)
    object_root = canonical_build_root(management.build_root()).namespace()
    # Register artifacts before publishing any new CAS object or report.
    report = management.outputs() / "reports" / management.key
    manifest_ref = management.artifact_manifest(target, {"candidate": target, "reports": report,
                                                        "inputObjects": object_root / "inputs"})
    if not sealed_inventory:
        claims = [PathClaim(path, "dependency", 0, checksum, 0, management.repo_id)
                  for path, checksum in hashes.items()]
        with _managed_phase("formal-candidate-seal"):
            sealed = seal_candidate(management.repo, claims, object_root=object_root,
                                    base_head=record["baseHead"])
        with _managed_phase("formal-external-seal"):
            frozen_external = _seal_external(external_captures, preparation_helper, object_root)
        source_manifest = _thaw(sealed.payload)
        source_manifest.update(preparationKind="current-source", external=frozen_external)
        _managed_verify_canonical(management, head, hashes, record["baseHead"], prior)
    _require(matching_target_source(management, target, prior) == prior,
             "managed_worktree_target_changed", "Target changed while the input objects were sealed")
    with management.state.transaction() as connection:
        current_reader = _reader(management, record, hashes, external, connection=connection)
        _require(current_reader["snapshotDigest"] == reader["snapshotDigest"]
                 and _pins_current(management.state, [_pin(prior_row), _pin(worktree_row), _pin(inspection_row)], connection=connection),
                 "managed_source_authority_changed", "Preparation input authority changed")
        old = management.state.get("managed_worktree_preparation", management.key, connection=connection)
        _require(old is None, "managed_source_reconciliation_required", "An interrupted preparation cannot be replayed")
        management.state.put("managed_worktree_preparation", management.key, {
            "status": "materializing", "worktreeId": reference, "path": str(target),
            "inputDigest": management.input_digest, "sourceHead": head, "baseHead": record["baseHead"],
            "inspectionOperationKey": inspection_row["key"],
            "preparationInputRef": management.payload["preparationInputRef"],
            "currentHashes": hashes, "beforeHashes": before, "compilerControlInputs": controls,
            "externalInputs": external, "externalRoots": external_captures,
            "sourceManifest": source_manifest, "objectRoot": str(object_root),
            "readerSessionId": management.session, "readerAuthorizationSnapshot": reader["snapshotDigest"],
            "sourceKind": "sealed-observed-canonical-input" if sealed_inventory else "observed-canonical-input",
            "sourcePreparation": True,
            "implementationOwnershipTransferred": False,
            "provenanceReferences": {"kind": "unasserted-input-metadata",
                                     "references": management.payload.get("provenanceRefs", [])},
            "writerSessionId": record["sessionId"],
            "writerAuthorizationSnapshot": record["writerAuthorizationSnapshot"],
            "repositoryId": management.repo_id, "driverDigest": management.config["driverDigest"],
            "runtimeOperationId": management.config["runtimeOperationId"],
            "artifactManifest": manifest_ref}, expected_version=0, connection=connection)
    # Recheck all preimages and sealed objects before the first target write.
    if sealed_inventory:
        _managed_sealed_inventory(management, record, inputs, inspection_row["payload"]["payload"])
    _managed_prewrite_check(management, target, before, hashes, object_root)
    # Every target preimage and every source object has passed before any target
    # write. Each handle rechecks its own preimage immediately before mutation.
    with managed_input_stage(target, "materialization-preimages") as preimages:
        for path, checksum in hashes.items():
            destination = target / path
            _require(preimages.checksum(path) == before[path],
                     "managed_worktree_target_changed", "Target preimage changed before its write")
            if checksum == before[path]:
                continue
            _plain(destination)
            management._worktree_mutation_started = True
            if checksum is None:
                destination.unlink()
                continue
            obj = object_root / "inputs/objects" / checksum
            _plain(obj)
            data = obj.read_bytes()
            _require(hashlib.sha256(data).hexdigest() == checksum,
                     "managed_source_object_changed", "Prepared source object is corrupt")
            if before[path] is None:
                destination.parent.mkdir(parents=True, exist_ok=True)
                _plain(destination)
                with destination.open("xb") as stream:
                    stream.write(data)
                    stream.flush()
                    os.fsync(stream.fileno())
            else:
                with destination.open("r+b") as stream:
                    _require(hashlib.sha256(stream.read()).hexdigest() == before[path],
                             "managed_worktree_target_changed", "Target handle preimage changed")
                    stream.seek(0)
                    stream.write(data)
                    stream.truncate()
                    stream.flush()
                    os.fsync(stream.fileno())
    tracked = {name.decode("utf-8", "strict").casefold() for name in
               management.git(target, "ls-files", "--cached", "-z").stdout.split(b"\0") if name}
    added = sorted(path for path, checksum in hashes.items() if checksum is not None and path not in tracked)
    # Intent-to-add preserves deleted base entries while admitting newly copied
    # compiler data even when the historical base .gitignore would ignore it.
    for start in range(0, len(added), 128):
        management._worktree_mutation_started = True
        management.git(target, "add", "--intent-to-add", "--force", "--", *added[start:start + 128])
    postimages = _managed_checksums(target, hashes, "target-postimages")
    _managed_drift(hashes, postimages, "managed_worktree_postimage_changed",
                   "Materialized WT differs from its declared input hashes")
    source = management.source(target)
    row = management.state.get("managed_worktree_preparation", management.key)
    management.state.put("managed_worktree_preparation", management.key,
                         {**row["payload"], "status": "complete", "source": source},
                         expected_version=row["version"])
    return {"status": "complete", "worktreeId": reference, "worktreePreparationRef": management.key,
            "source": source, "beforeHashes": before, "afterHashes": hashes,
            "sourceHead": head, "baseHead": record["baseHead"],
            "sealedCandidateDigest": digest(source_manifest), "sourcePreparation": True,
            "implementationOwnershipTransferred": False, "canonicalSourceModified": False,
            "commitPerformed": False, "pushPerformed": False, "formalAcceptance": False}

# Local validation admission belongs to the management capsule. A request is
# evidence of local validation intent; it is never a fabricated user grant.
_LOCAL_REQUEST_POLICY = "local-validation-v1"
_LOCAL_CONTROL_ROOTS = {".git", ".jenkins", ".codex", ".opencode", "dev", "target", "build", "agents.md"}


def local_request_principal(manager):
    """Read the verified live supervisor's Windows owner through one handle."""
    from ctypes import wintypes
    from tools.jenkins.deployment.drain_observations import owner_observation
    owner = owner_observation(manager)["supervisor"]
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    advapi = ctypes.WinDLL("advapi32", use_last_error=True)
    kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    kernel.OpenProcess.restype = wintypes.HANDLE
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel.CloseHandle.restype = wintypes.BOOL
    kernel.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
    kernel.GetProcessTimes.restype = wintypes.BOOL
    kernel.GetExitCodeProcess.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.DWORD)]
    kernel.GetExitCodeProcess.restype = wintypes.BOOL
    kernel.LocalFree.argtypes = [wintypes.HLOCAL]
    kernel.LocalFree.restype = wintypes.HLOCAL
    advapi.OpenProcessToken.argtypes = [wintypes.HANDLE, wintypes.DWORD, ctypes.POINTER(wintypes.HANDLE)]
    advapi.OpenProcessToken.restype = wintypes.BOOL
    advapi.GetTokenInformation.argtypes = [wintypes.HANDLE, wintypes.DWORD, wintypes.LPVOID, wintypes.DWORD, ctypes.POINTER(wintypes.DWORD)]
    advapi.GetTokenInformation.restype = wintypes.BOOL
    advapi.ConvertSidToStringSidW.argtypes = [wintypes.LPVOID, ctypes.POINTER(wintypes.LPWSTR)]
    advapi.ConvertSidToStringSidW.restype = wintypes.BOOL
    process = kernel.OpenProcess(0x1000, False, owner["pid"])
    if not process:
        raise ctypes.WinError(ctypes.get_last_error())
    token, text = wintypes.HANDLE(), wintypes.LPWSTR()
    try:
        created, exited, cpu_kernel, cpu_user = (wintypes.FILETIME() for _ in range(4))
        if not kernel.GetProcessTimes(process, ctypes.byref(created), ctypes.byref(exited),
                                      ctypes.byref(cpu_kernel), ctypes.byref(cpu_user)):
            raise ctypes.WinError(ctypes.get_last_error())
        birth = (created.dwHighDateTime << 32) | created.dwLowDateTime
        if str(birth) != str(owner["creationTime"]):
            raise ValueError("Local request supervisor birth changed")
        if not advapi.OpenProcessToken(process, 8, ctypes.byref(token)):
            raise ctypes.WinError(ctypes.get_last_error())
        size = wintypes.DWORD()
        advapi.GetTokenInformation(token, 1, None, 0, ctypes.byref(size))
        if not size.value:
            raise ctypes.WinError(ctypes.get_last_error())
        buffer = ctypes.create_string_buffer(size.value)
        if not advapi.GetTokenInformation(token, 1, buffer, size, ctypes.byref(size)):
            raise ctypes.WinError(ctypes.get_last_error())
        sid = ctypes.cast(buffer, ctypes.POINTER(ctypes.c_void_p)).contents.value
        if not advapi.ConvertSidToStringSidW(sid, ctypes.byref(text)):
            raise ctypes.WinError(ctypes.get_last_error())
        status = wintypes.DWORD()
        if not kernel.GetExitCodeProcess(process, ctypes.byref(status)) or status.value != 259:
            raise ValueError("Local request supervisor is no longer live")
        principal = "windows:" + text.value
        if not re.fullmatch(r"windows:S-1-[0-9-]+", principal):
            raise ValueError("Local request owner SID is invalid")
        return principal
    finally:
        if text:
            kernel.LocalFree(ctypes.cast(text, wintypes.HLOCAL))
        if token:
            kernel.CloseHandle(token)
        kernel.CloseHandle(process)


class _ScopedAuthorizationState:
    """Delegate authority to State, matching immutable scopes without a cross product."""

    def __init__(self, authority):
        self._authority = authority

    def __getattr__(self, name):
        return getattr(self._authority, name)

    def check_authorization(self, repository_id, session_id, action, paths=None, *, connection=None):
        from tools.jenkins.contracts import JenkinsError
        from tools.jenkins.state import _relative_path
        if connection is None:
            with self._authority.connect() as current:
                return self.check_authorization(repository_id, session_id, action, paths, connection=current)
        result = self._authority.check_authorization(
            repository_id, session_id, action, paths=None, connection=connection)
        if not paths:
            return result
        scopes = result["paths"]
        # Keep the driver's behavior for malformed historical rows as well.
        if not all(isinstance(scope, str) for scope in scopes):
            return self._authority.check_authorization(
                repository_id, session_id, action, paths, connection=connection)
        allowed = set(scopes)
        for path in paths:
            current = _relative_path(path)
            while current not in allowed:
                current, separator, _ = current.rpartition("/")
                if not separator:
                    raise JenkinsError("path_not_authorized", "Path lies outside task authorization")
        return result


def scoped_authorization_state(state):
    """Wrap one supplied State; never replace the driver class or cache authority."""
    return state if isinstance(state, _ScopedAuthorizationState) else _ScopedAuthorizationState(state)


def _local_context_row(row):
    from tools.jenkins.contracts import JenkinsError, digest as contract_digest
    value = {"repositoryId": row["repository_id"], "sessionId": row["session_id"],
             "principal": row["principal"], "paths": json.loads(row["paths"]),
             "actions": json.loads(row["actions"]), "evidence": json.loads(row["evidence"])}
    if contract_digest(value) != row["snapshot_digest"] or row["status"] != "active":
        raise JenkinsError("local_context_conflict", "Existing local context is revoked or corrupt")
    return value


def ensure_local_context(state, repo, session_id, *, principal, paths, actions, evidence, connection=None):
    """Insert an ordinary State row, preserving existing scoped authorizations."""
    from contextlib import nullcontext
    from tools.jenkins.contracts import JenkinsError, canonical_json, digest as contract_digest, identifier
    from tools.jenkins.source import repository_id
    from tools.jenkins.state import _relative_path
    state = scoped_authorization_state(state)
    repo = Path(repo).absolute()
    identifier(session_id, "sessionId")
    repository = evidence.get("repositoryId")
    identities = {repository_id(repo), hashlib.sha256(str(repo).casefold().encode()).hexdigest(),
                  digest(str(repo).casefold())}
    actions, paths = sorted(set(actions)), sorted({_relative_path(path) for path in paths})
    if (Path(state.path) != repo / ".jenkins/state/coordination.sqlite3" or repository not in identities
            or not isinstance(principal, str) or not principal or not paths
            or actions not in (["validation"], ["implementation", "validation"])
            or evidence.get("source") != "automatic-local-request"
            or evidence.get("policy") != _LOCAL_REQUEST_POLICY
            or evidence.get("kind") not in {"ordinary-validation", "managed-worktree-reader", "managed-worktree-writer"}
            or not re.fullmatch(r"[0-9a-f]{64}", str(evidence.get("inputDigest", "")))):
        raise JenkinsError("local_context_invalid", "Local validation needs a bound owner, policy and exact context")
    snapshot = {"repositoryId": repository, "sessionId": session_id, "principal": principal,
                "paths": paths, "actions": actions, "evidence": evidence}
    with (state.transaction() if connection is None else nullcontext(connection)) as connection:
        row = connection.execute("SELECT * FROM authorizations WHERE repository_id=? AND session_id=?",
                                 (repository, session_id)).fetchone()
        if row is None:
            checksum = contract_digest(snapshot)
            connection.execute("INSERT INTO authorizations VALUES (?,?,?,?,?,?,?,'active',?)",
                               (repository, session_id, principal, canonical_json(paths).decode(),
                                canonical_json(actions).decode(), canonical_json(evidence).decode(), checksum, time.time()))
            state.event("local_validation_context_admitted", {"repositoryId": repository, "sessionId": session_id,
                        "snapshotDigest": checksum, "source": evidence["source"]}, connection=connection)
        else:
            existing = _local_context_row(row)
            if (existing["evidence"].get("source") == "automatic-local-request"
                    and existing != snapshot):
                raise JenkinsError("local_context_conflict", "An automatic context already has different ownership or scope")
        for action in actions:
            result = state.check_authorization(repository, session_id, action, paths, connection=connection)
    return result


def admit_ordinary_context(state, repo, payload, *, principal, driver_digest, runtime_operation_id, actions=None, record_request=True):
    """Freeze one public source scope per local session; pin each request separately."""
    from tools.jenkins.contracts import JenkinsError, digest as contract_digest, identifier, require_digest
    from tools.jenkins.deployment.driver import _plain
    from tools.jenkins.state import _relative_path
    state = scoped_authorization_state(state)
    repo = Path(repo).absolute()
    identity = payload.get("identity", payload)
    if not isinstance(identity, dict):
        raise JenkinsError("identity_invalid", "Local validation identity must be an object")
    for name in ("repositoryId", "sessionId", "requestId"):
        identifier(identity.get(name), name)
    require_digest(driver_digest, "driverDigest")
    if not isinstance(runtime_operation_id, str) or not runtime_operation_id:
        raise JenkinsError("runtime_binding_missing", "Local validation needs its current runtime")
    owned = payload.get("ownedPaths")
    if not isinstance(owned, list) or not owned:
        raise JenkinsError("invalid_source_path", "Local validation requires explicit owned paths")
    requested = set()
    for path in [*owned, *payload.get("declaredDependencies", [])]:
        relative = _relative_path(path)
        if relative.split("/")[0] in _LOCAL_CONTROL_ROOTS or relative.startswith("."):
            raise JenkinsError("invalid_source_path", "Local validation excludes control and Git runtime roots")
        _plain(repo / relative)
        requested.add(relative)
    for field in ("beforeHashes", "afterHashes", "dependencyHashes", "currentHashes", "sourceHashes"):
        hashes = payload.get(field)
        if hashes is None:
            continue
        if not isinstance(hashes, dict):
            raise JenkinsError("local_input_invalid", "Source hashes must be explicit objects")
        for path, checksum in hashes.items():
            relative = _relative_path(path)
            if relative.split("/")[0] in _LOCAL_CONTROL_ROOTS or relative.startswith("."):
                raise JenkinsError("invalid_source_path", "Control files are outside ordinary local validation")
            _plain(repo / relative)
            if checksum is not None:
                require_digest(checksum, field)
            requested.add(relative)
    actions = actions or ["implementation", "validation"]
    with state.connect() as connection:
        row = connection.execute("SELECT * FROM authorizations WHERE repository_id=? AND session_id=?",
                                 (identity["repositoryId"], identity["sessionId"])).fetchone()
        existing = _local_context_row(row) if row is not None else None
    if existing is not None:
        # A manual narrow writer is never expanded. Ordinary contexts retain
        # their first scope and digest across new requests and service restarts.
        evidence, scopes = existing["evidence"], existing["paths"]
        if evidence.get("source") == "automatic-local-request":
            if (evidence.get("kind") not in {"ordinary-validation", "managed-worktree-writer"}
                    or existing["principal"] != principal):
                raise JenkinsError("local_context_conflict", "This identity already belongs to another local context")
            context = ensure_local_context(state, repo, identity["sessionId"], principal=principal,
                                           paths=scopes, actions=existing["actions"], evidence=evidence)
        else:
            context = state.check_authorization(identity["repositoryId"], identity["sessionId"], actions[0], sorted(requested))
    else:
        scopes = []
        for child in repo.iterdir():
            name = child.name.casefold()
            if name.startswith(".") or name in _LOCAL_CONTROL_ROOTS:
                continue
            _plain(child)
            scopes.append(name)
        evidence = {"source": "automatic-local-request", "policy": _LOCAL_REQUEST_POLICY,
                    "kind": "ordinary-validation", "repositoryId": identity["repositoryId"],
                    "repositoryRoot": str(repo), "ownerPrincipal": principal,
                    "createdDriverDigest": driver_digest, "createdRuntimeOperationId": runtime_operation_id,
                    "inputDigest": contract_digest({"repositoryId": identity["repositoryId"],
                                     "sessionId": identity["sessionId"], "principal": principal, "policy": _LOCAL_REQUEST_POLICY})}
        context = ensure_local_context(state, repo, identity["sessionId"], principal=principal,
                                       paths=scopes, actions=["implementation", "validation"], evidence=evidence)
    for action in actions:
        state.check_authorization(identity["repositoryId"], identity["sessionId"], action, sorted(requested))
    if not record_request:
        return context
    admission = {"schemaVersion": 1, "source": "automatic-local-request", "policy": _LOCAL_REQUEST_POLICY,
                 "identity": {name: identity[name] for name in ("repositoryId", "sessionId", "requestId", "attemptId",
                              "operationId", "generation") if name in identity},
                 "inputDigest": contract_digest(payload), "requestedPaths": sorted(requested),
                 "driverDigest": driver_digest, "runtimeOperationId": runtime_operation_id,
                 "contextSnapshot": context["snapshotDigest"], "principal": context["principal"]}
    key = contract_digest(admission["identity"])
    with state.transaction() as connection:
        prior = state.get("local_request_admission", key, connection=connection)
        if prior is not None and prior["payload"] != admission:
            raise JenkinsError("local_request_conflict", "This local request already has different exact input")
        if prior is None:
            state.put("local_request_admission", key, admission, expected_version=0, connection=connection)
    return context

_MANAGED_ROUTE_ACTIONS = frozenset({
    "create-worktree", "prepare-merge", "inspect-worktree", "apply-patch", "generate",
    "prepare-worktree", "prepare-flow", "submit-flow",
})


def _managed_route_identity(payload):
    from tools.jenkins.contracts import identifier
    return {name: identifier(payload.get(name), name)
            for name in ("repositoryId", "sessionId", "requestId", "operationId")}


def _managed_route_state(repo, state, *, query_only):
    from contextlib import contextmanager
    import sqlite3
    from tools.jenkins.contracts import JenkinsError
    from tools.jenkins.deployment.driver import _plain
    from tools.jenkins.state import State
    path = repo / ".jenkins/state/coordination.sqlite3"
    _plain(path)
    if state is not None:
        if Path(state.path) != path:
            raise JenkinsError("managed_context_invalid", "Managed routing needs the canonical State database")
        return state
    if not path.is_file():
        if query_only:
            return None
        raise JenkinsError("managed_context_invalid", "Managed routing requires the existing State database")
    state = object.__new__(State)
    state.path, state.busy_timeout_ms = path, 3000
    if query_only:
        @contextmanager
        def readonly():
            connection = sqlite3.connect(path.as_uri() + "?mode=ro", uri=True, isolation_level=None)
            connection.row_factory = sqlite3.Row
            try:
                yield connection
            finally:
                connection.close()
        state.connect = readonly
    return state


def _managed_route_replay(state, payload, key, *, connection=None):
    from tools.jenkins.contracts import JenkinsError
    row = state.get("management_context_route", key, connection=connection)
    if row is None:
        alias = state.get("management_context_route_alias", key, connection=connection)
        if alias is not None:
            row = state.get("management_context_route", alias["payload"].get("routeRef", ""),
                            connection=connection)
            if row is None:
                raise JenkinsError("managed_context_route_conflict", "Managed routing alias lost its immutable route")
    if row is None:
        return None
    route = row["payload"]
    clean = {name: value for name, value in payload.items() if name != "observationOnly"}
    identity = _managed_route_identity(payload)
    if (route.get("routeRef") != row["key"] or route.get("action") != payload.get("action")
            or route.get("source") != "automatic-local-request"
            or route.get("policy") != _LOCAL_REQUEST_POLICY):
        raise JenkinsError("managed_context_route_conflict", "Managed routing provenance changed")
    if identity == route.get("originIdentity") and digest(clean) == route.get("originInputDigest"):
        routed = {**payload, **route["routeFields"]}
    elif identity == route.get("routedIdentity") and digest(clean) == route.get("routedInputDigest"):
        routed = dict(payload)
    else:
        raise JenkinsError("managed_context_route_conflict", "This managed request already has different exact input")
    if (_managed_route_identity(routed) != route["routedIdentity"]
            or digest({name: value for name, value in routed.items() if name != "observationOnly"})
                != route["routedInputDigest"]):
        raise JenkinsError("managed_context_route_conflict", "Stored managed routing fields are inconsistent")
    return routed, route


def _managed_route_object_root(manager, payload):
    from tools.jenkins.resources.paths import canonical_build_root, physical_path_under
    requested = payload.get("requestedTargetDir")
    explicit = payload.get("buildRoot")
    if requested is not None:
        target = Path(requested)
        approved = canonical_build_root(str(Path(target.anchor) / "cargo-targets"))
        physical_path_under(approved, target)
        if explicit is not None and canonical_build_root(explicit).path != approved.path:
            raise ValueError("Requested target and buildRoot choose different physical volumes")
    else:
        storage = getattr(getattr(manager, "spec", None), "storage", {})
        approved = canonical_build_root(explicit or storage.get("buildRoot", r"D:\cargo-targets"))
    return approved.namespace()


def _managed_route_worktree(state, repo, config, reference, owned):
    from tools.jenkins.contracts import JenkinsError, require_digest
    from tools.jenkins.deployment.driver import _plain
    row = state.get("management_worktree", reference)
    record = row["payload"] if row else {}
    recorded_path = record.get("path")
    if (not isinstance(reference, str) or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,159}", reference)
            or record.get("status") != "ready"
            or record.get("repositoryId") != config["repositoryId"]
            or record.get("driverDigest") != config["driverDigest"]
            or record.get("runtimeOperationId") != config["runtimeOperationId"]
            or sorted(record.get("ownedPaths", [])) != owned
            or not isinstance(recorded_path, str)
            or Path(recorded_path) != repo / ".jenkins/worktrees" / reference):
        raise JenkinsError("managed_context_worktree_unproven", "Managed routing needs the exact ready current-runtime worktree")
    _plain(Path(recorded_path))
    require_digest(record.get("creationOperationKey"), "creationOperationKey")
    writer = state.check_authorization(config["repositoryId"], record["sessionId"], "implementation", owned)
    validation = state.check_authorization(config["repositoryId"], record["sessionId"], "validation", owned)
    if (writer["actions"] != ["implementation", "validation"] or sorted(writer["paths"]) != owned
            or writer["snapshotDigest"] != validation["snapshotDigest"]
            or writer["snapshotDigest"] != record.get("writerAuthorizationSnapshot")
            or writer["principal"] != record.get("writerPrincipal")):
        raise JenkinsError("managed_context_writer_changed", "Managed routing preserves the original exact writer snapshot")
    return row, writer


def _managed_route_preparation(state, key, reference, repository, config):
    from tools.jenkins.contracts import JenkinsError, require_digest
    key = require_digest(key, "worktreePreparationRef")
    row = state.get("managed_worktree_preparation", key)
    operation = state.get("management_operation", key)
    value, op = row["payload"] if row else {}, operation["payload"] if operation else {}
    original, result = op.get("payload", {}), op.get("result", {})
    if (value.get("status") != "complete" or value.get("worktreeId") != reference
            or op.get("phase") != "complete" or op.get("action") != "prepare-worktree"
            or op.get("inputDigest") != value.get("inputDigest")
            or op.get("inputDigest") != digest({name: item for name, item in original.items() if name != "observationOnly"})
            or key != digest(_managed_route_identity(original))
            or original.get("sessionId") != value.get("readerSessionId")
            or result.get("status") != "complete" or result.get("worktreePreparationRef") != key
            or result.get("source") != value.get("source")
            or result.get("sealedCandidateDigest") != digest(value.get("sourceManifest"))
            or value.get("sourceManifest", {}).get("baseHead") != value.get("baseHead")
            or any(value.get(name) != expected for name, expected in {
                "repositoryId": repository, "driverDigest": config["driverDigest"],
                "runtimeOperationId": config["runtimeOperationId"]}.items())
            or any(original.get(name) != value.get(name) for name in
                   ("repositoryId", "driverDigest", "runtimeOperationId", "worktreeId",
                    "inspectionOperationKey", "preparationInputRef"))):
        raise JenkinsError("managed_context_preparation_unproven", "Routing requires the original completed preparation")
    return row, operation


def route_managed_context(config, repo, manager, payload, *, state=None, query_only=False):
    """Route one local managed request before its immutable operation claim.

    Routing creates technical contexts, never acceptance or implementation scope
    for dependencies. Original chat contexts and historical writer rows remain intact.
    """
    from tools.jenkins.contracts import JenkinsError, require_digest
    from tools.jenkins.state import _relative_path
    from tools.jenkins import managed_worktree as formal
    from tools.jenkins.source.external_inputs import authorization_paths
    if payload.get("action") not in _MANAGED_ROUTE_ACTIONS:
        return payload, None
    query_only = query_only or payload.get("observationOnly") is True
    repo = Path(repo).absolute()
    state = _managed_route_state(repo, state, query_only=query_only)
    if state is None:
        return payload, None
    state = scoped_authorization_state(state)
    origin = _managed_route_identity(payload)
    key = digest(origin)
    replay = _managed_route_replay(state, payload, key)
    if replay is not None:
        return replay
    if query_only:
        return payload, None
    if (payload.get("driverDigest") != config["driverDigest"]
            or payload.get("runtimeOperationId") != config["runtimeOperationId"]):
        raise JenkinsError("managed_context_invalid", "Managed routing retains the selected driver and runtime")
    owned = payload.get("ownedPaths")
    if not isinstance(owned, list) or not owned or any(not isinstance(path, str) for path in owned):
        raise JenkinsError("managed_context_invalid", "Managed routing requires explicit owned paths")
    normalized = [_relative_path(path) for path in owned]
    if normalized != owned or len(set(owned)) != len(owned):
        raise JenkinsError("managed_context_invalid", "Managed routing requires canonical unique owned paths")
    owned = sorted(owned)
    requester = local_request_principal(manager)
    with state.connect() as connection:
        original_row = connection.execute("SELECT * FROM authorizations WHERE repository_id=? AND session_id=?",
                                         (origin["repositoryId"], origin["sessionId"])).fetchone()
        original_context = _local_context_row(original_row) if original_row is not None else None
    if original_context is None:
        admit_ordinary_context(state, repo, payload, principal=requester,
            driver_digest=config["driverDigest"], runtime_operation_id=config["runtimeOperationId"], record_request=False)
        with state.connect() as connection:
            original_row = connection.execute("SELECT * FROM authorizations WHERE repository_id=? AND session_id=?",
                                             (origin["repositoryId"], origin["sessionId"])).fetchone()
            original_context = _local_context_row(original_row)
    evidence = original_context["evidence"]
    if (evidence.get("source") == "automatic-local-request"
            and (evidence.get("ownerPrincipal") != requester
                 or evidence.get("policy") != _LOCAL_REQUEST_POLICY
                 or evidence.get("repositoryId") != origin["repositoryId"]
                 or evidence.get("repositoryRoot") != str(repo)
                 or evidence.get("kind") not in {"ordinary-validation", "managed-worktree-writer", "managed-worktree-reader"}
                 or not re.fullmatch(r"[0-9a-f]{64}", str(evidence.get("inputDigest", "")))
                 or original_context["actions"] != (["validation"] if evidence.get("kind") == "managed-worktree-reader"
                                                     else ["implementation", "validation"]))):
        raise JenkinsError("managed_context_owner_changed", "The original local request belongs to another owner")
    action = payload["action"]
    original_authority = state.check_authorization(origin["repositoryId"], origin["sessionId"],
        "implementation" if action in {"create-worktree", "prepare-merge", "apply-patch", "generate"} else "validation", owned)
    routed, contexts, pins = dict(payload), [], []
    writer_session, reader_session = None, None
    writer_snapshot, reader_snapshot = None, None
    derivation_digest = None
    worktree_id, inspection_key, prepared_key = payload.get("worktreeId"), None, None
    if action in {"create-worktree", "prepare-merge"}:
        if original_authority["actions"] == ["implementation", "validation"] and sorted(original_authority["paths"]) == owned:
            return payload, None
        if (evidence.get("source") != "automatic-local-request"
                or evidence.get("kind") != "ordinary-validation"
                or original_authority["principal"] != requester):
            raise JenkinsError("managed_context_writer_changed", "A legacy writer cannot be expanded or rebound")
        if payload.get("resumeFromOperationKey") is not None:
            raise JenkinsError("managed_context_writer_changed", "An existing worktree resume retains its original writer identity")
        binding = {"repositoryId": origin["repositoryId"], "originIdentity": origin,
                   "originSnapshot": original_authority["snapshotDigest"], "ownedPaths": owned,
                   "driverDigest": config["driverDigest"], "runtimeOperationId": config["runtimeOperationId"]}
        derivation_digest = digest(binding)
        writer_session = "w" + derivation_digest[:10]
        routed["sessionId"] = writer_session
        contexts.append((writer_session, requester, owned, ["implementation", "validation"], {
            "source": "automatic-local-request", "policy": _LOCAL_REQUEST_POLICY, "kind": "managed-worktree-writer",
            "repositoryId": origin["repositoryId"], "repositoryRoot": str(repo), "ownerPrincipal": requester,
            "originSessionId": origin["sessionId"], "originSnapshot": original_authority["snapshotDigest"],
            "createdDriverDigest": config["driverDigest"], "createdRuntimeOperationId": config["runtimeOperationId"],
            "inputDigest": digest(binding)}))
    else:
        prepared = None
        intent_row = None
        if action == "submit-flow":
            flow = payload.get("flowPayload")
            reference = flow.get("currentSourceRef") if isinstance(flow, dict) else None
            intent_row = state.get("current_source_intent", require_digest(reference, "currentSourceRef"))
            intent = intent_row["payload"] if intent_row else {}
            if (digest(intent) != reference or flow.get("identity") != intent.get("identity")
                    or intent.get("repositoryRoot") != str(repo)
                    or intent.get("identity", {}).get("repositoryId") != origin["repositoryId"]
                    or intent.get("identity", {}).get("stageImplementationDigest") != config["driverDigest"]
                    or intent.get("runtimeOperationId") != config["runtimeOperationId"]
                    or worktree_id is not None and worktree_id != intent.get("worktreeId")):
                raise JenkinsError("managed_context_preparation_unproven", "Flow routing requires its immutable current-source intent")
            worktree_id = intent.get("worktreeId")
            prepared_key = intent.get("worktreePreparationRef")
            pins.append(formal._pin(intent_row))
        elif action == "prepare-flow":
            prepared_key = payload.get("worktreePreparationRef")
        row, writer = _managed_route_worktree(state, repo,
            {**config, "repositoryId": origin["repositoryId"]}, worktree_id, owned)
        record = row["payload"]
        pins.append(formal._pin(row))
        writer_session, writer_snapshot = record["sessionId"], writer["snapshotDigest"]
        if (original_authority["principal"] != writer["principal"]
                and not (evidence.get("source") == "automatic-local-request"
                         and evidence.get("kind") == "ordinary-validation"
                         and original_authority["principal"] == requester)):
            raise JenkinsError("managed_context_owner_changed", "The requester cannot continue this managed writer")
        if action in {"inspect-worktree", "apply-patch", "generate"}:
            routed["sessionId"] = writer_session
        else:
            if action in {"prepare-flow", "submit-flow"}:
                prepared_row, operation = _managed_route_preparation(
                    state, prepared_key, worktree_id, origin["repositoryId"], config)
                prepared = prepared_row["payload"]
                pins.extend([formal._pin(prepared_row), formal._pin(operation)])
                inspection_key, inventory_ref = prepared["inspectionOperationKey"], prepared["preparationInputRef"]
            else:
                inspection_key, inventory_ref = payload.get("inspectionOperationKey"), payload.get("preparationInputRef")
            object_root = _managed_route_object_root(manager, payload)
            if prepared is not None and prepared.get("objectRoot") != str(object_root):
                raise JenkinsError("managed_context_preparation_unproven", "Routing retains the original CAS namespace")
            inspection, inputs = formal._inventory_reference(state, object_root, inspection_key, inventory_ref,
                worktree_id, origin["repositoryId"], config["driverDigest"], config["runtimeOperationId"])
            pins.append(formal._pin(inspection))
            if (sorted(inspection["payload"]["payload"].get("ownedPaths", [])) != owned
                    or prepared is not None and any(inputs.get(name) != prepared.get(name) for name in
                        ("sourceHead", "currentHashes", "beforeHashes", "compilerControlInputs", "externalInputs"))):
                raise JenkinsError("managed_context_preparation_unproven", "Routing retains its exact completed inventory closure")
            paths = sorted(list(inputs["currentHashes"]) + authorization_paths(inputs["externalInputs"]))
            if len(set(paths)) != len(paths) or not set(owned) <= set(inputs["currentHashes"]):
                raise JenkinsError("managed_context_invalid", "The trusted input inventory contains overlapping or missing paths")
            binding = {"repositoryId": origin["repositoryId"], "worktreeId": worktree_id,
                       "creationOperationKey": record["creationOperationKey"], "baseHead": record["baseHead"],
                       "writerSessionId": writer_session, "writerPrincipal": writer["principal"],
                       "writerAuthorizationSnapshot": writer_snapshot,
                       "inspectionOperationKey": inspection_key, "inventoryDigest": digest(inputs),
                       "driverDigest": config["driverDigest"], "runtimeOperationId": config["runtimeOperationId"]}
            derivation_digest = digest(binding)
            reader_session = prepared["readerSessionId"] if prepared is not None else "mr-" + derivation_digest[:12]
            if reader_session == writer_session:
                raise JenkinsError("managed_context_invalid", "The validation reader must remain separate from the writer")
            reader_evidence = {"source": "automatic-local-request", "policy": _LOCAL_REQUEST_POLICY,
                "kind": "managed-worktree-reader", "repositoryId": origin["repositoryId"], "repositoryRoot": str(repo),
                "ownerPrincipal": requester, "writerPrincipal": writer["principal"],
                "worktreeId": worktree_id, "writerSessionId": writer_session,
                "writerAuthorizationSnapshot": writer_snapshot, "inspectionOperationKey": inspection_key,
                "inventoryDigest": digest(inputs), "createdDriverDigest": config["driverDigest"],
                "createdRuntimeOperationId": config["runtimeOperationId"], "inputDigest": digest(binding)}
            if prepared is not None:
                with state.connect() as connection:
                    reader_row = connection.execute("SELECT * FROM authorizations WHERE repository_id=? AND session_id=?",
                                                    (origin["repositoryId"], reader_session)).fetchone()
                    reader_context = _local_context_row(reader_row) if reader_row is not None else None
                if (reader_context is None or reader_context["evidence"].get("source") == "automatic-local-request"
                        and reader_context["evidence"].get("ownerPrincipal") != requester):
                    raise JenkinsError("managed_context_reader_changed", "The prepared reader belongs to another local owner")
                reader = state.check_authorization(origin["repositoryId"], reader_session, "validation", paths)
                if (reader["principal"] != writer["principal"] or reader["actions"] != ["validation"]
                        or sorted(reader["paths"]) != paths
                        or reader["snapshotDigest"] != prepared.get("readerAuthorizationSnapshot")
                        or prepared.get("writerSessionId") != writer_session
                        or prepared.get("writerAuthorizationSnapshot") != writer_snapshot):
                    raise JenkinsError("managed_context_reader_changed", "The completed preparation reader cannot be replaced")
                reader_snapshot = reader["snapshotDigest"]
                if intent_row is not None and (intent.get("authorizationSnapshot") != reader_snapshot
                        or intent["identity"]["sessionId"] != reader_session
                        or intent.get("ownedPaths") != owned):
                    raise JenkinsError("managed_context_reader_changed", "Flow retains the prepared validation reader")
            else:
                contexts.append((reader_session, writer["principal"], paths, ["validation"], reader_evidence))
            if action == "prepare-flow":
                flow_identity = payload.get("flowIdentity")
                if not isinstance(flow_identity, dict) or flow_identity.get("generation") not in (1, "1"):
                    raise JenkinsError("managed_context_invalid", "Prepared Flow needs its explicit generation-one identity")
                routed["sessionId"] = writer_session
                routed["flowIdentity"] = {**flow_identity, "sessionId": reader_session}
            else:
                routed["sessionId"] = reader_session
    routed_identity = _managed_route_identity(routed)
    routed_key = digest(routed_identity)
    route = {"schemaVersion": 1, "routeRef": key, "source": "automatic-local-request",
             "policy": _LOCAL_REQUEST_POLICY, "action": action, "originIdentity": origin,
             "routedIdentity": routed_identity, "originInputDigest": digest({name: value for name, value in payload.items()
                                                                          if name != "observationOnly"}),
             "routedInputDigest": digest({name: value for name, value in routed.items() if name != "observationOnly"}),
             "routeFields": {name: value for name, value in routed.items() if payload.get(name) != value},
             "ownerPrincipal": requester, "originSnapshot": original_authority["snapshotDigest"],
             "repositoryRoot": str(repo), "driverDigest": config["driverDigest"],
             "runtimeOperationId": config["runtimeOperationId"], "worktreeId": worktree_id,
             "writerSessionId": writer_session, "writerAuthorizationSnapshot": writer_snapshot,
             "readerSessionId": reader_session, "readerAuthorizationSnapshot": reader_snapshot,
             "inspectionOperationKey": inspection_key, "worktreePreparationRef": prepared_key,
             "derivationDigest": derivation_digest, "authorityPins": pins, "formalAcceptance": False}
    with state.transaction() as connection:
        replay = _managed_route_replay(state, payload, key, connection=connection)
        if replay is not None:
            return replay
        current_origin = connection.execute("SELECT * FROM authorizations WHERE repository_id=? AND session_id=?",
                                           (origin["repositoryId"], origin["sessionId"])).fetchone()
        if (current_origin is None or _local_context_row(current_origin) != original_context
                or pins and not formal._pins_current(state, pins, connection=connection)):
            raise JenkinsError("managed_context_route_conflict", "Managed routing authority changed before its commit")
        if state.get("management_operation", key, connection=connection) or state.get("management_operation", routed_key, connection=connection):
            raise JenkinsError("managed_context_already_claimed", "A claimed operation must be queried with its original identity")
        for session, principal, paths, actions, context_evidence in contexts:
            old_context = connection.execute("SELECT * FROM authorizations WHERE repository_id=? AND session_id=?",
                                             (origin["repositoryId"], session)).fetchone()
            if old_context is not None and _local_context_row(old_context)["evidence"].get("source") != "automatic-local-request":
                raise JenkinsError("managed_context_route_conflict", "An existing manual identity cannot become a technical context")
            context = ensure_local_context(state, repo, session, principal=principal, paths=paths,
                                           actions=actions, evidence=context_evidence, connection=connection)
            if context["principal"] != principal or context["actions"] != actions or sorted(context["paths"]) != paths:
                raise JenkinsError("managed_context_route_conflict", "A technical context must retain its exact frozen scope")
            if session == writer_session:
                route["writerAuthorizationSnapshot"] = context["snapshotDigest"]
            if session == reader_session:
                route["readerAuthorizationSnapshot"] = context["snapshotDigest"]
        if writer_snapshot is not None:
            actual = state.check_authorization(origin["repositoryId"], writer_session, "implementation", owned, connection=connection)
            if actual["snapshotDigest"] != writer_snapshot:
                raise JenkinsError("managed_context_writer_changed", "The original writer changed before routing commit")
        if reader_snapshot is not None:
            actual = state.check_authorization(origin["repositoryId"], reader_session, "validation", connection=connection)
            if actual["snapshotDigest"] != reader_snapshot:
                raise JenkinsError("managed_context_reader_changed", "The prepared reader changed before routing commit")
        if routed_key != key:
            alias = state.get("management_context_route_alias", routed_key, connection=connection)
            if alias is not None:
                raise JenkinsError("managed_context_route_conflict", "The technical identity already belongs to another route")
            state.put("management_context_route_alias", routed_key, {"routeRef": key}, expected_version=0, connection=connection)
        state.put("management_context_route", key, route, expected_version=0, connection=connection)
    return routed, route


class _PreparedWorktreeFlowProof:
    """Reuse one invocation's validated closure while retaining fresh authority."""
    def __init__(self, management, supplied, prepared_row):
        from tools.jenkins import managed_worktree as formal
        self.management = management
        self.prepared = prepared_row["payload"]
        self.worktree = supplied["worktreeId"]
        state = management.state
        row = state.get("management_worktree", self.worktree)
        formal._require(row is not None, "current_source_worktree_changed",
                        "Prepared source worktree disappeared before validation")
        pairs = [("management_worktree", self.worktree),
                 ("managed_worktree_preparation", prepared_row["key"]),
                 ("management_operation", prepared_row["key"]),
                 ("management_operation", self.prepared["inspectionOperationKey"]),
                 ("management_operation", row["payload"]["creationOperationKey"])]
        pairs.extend(("management_operation", key) for key in supplied.get("patchOperationKeys", []))
        pairs.append(("managed_worktree_writer", self.worktree))
        self.pins = []
        for domain, key in dict.fromkeys(pairs):
            actual = state.get(domain, key)
            formal._require(actual is not None or domain == "managed_worktree_writer",
                            "current_source_worktree_changed", "Prepared authority row disappeared")
            self.pins.append((domain, key, formal._pin(actual) if actual else None))
        formal._require(next(pin for domain, key, pin in self.pins
                             if domain == "managed_worktree_preparation") == formal._pin(prepared_row),
                        "current_source_worktree_changed", "Prepared authority changed before validation")

    def bind(self, intent):
        self.intent_digest = digest(intent)
        self.check(intent)

    def check(self, intent, *, connection=None):
        from tools.jenkins import current_source as current, managed_worktree as formal
        state, identity = self.management.state, intent["identity"]
        formal._require(digest(intent) == self.intent_digest,
                        "current_source_worktree_changed", "Prepared invocation input changed")
        for domain, key, pin in self.pins:
            row = state.get(domain, key, connection=connection)
            formal._require((row is None if pin is None else row is not None and formal._pin(row) == pin),
                            "current_source_worktree_changed", "Prepared authority version or payload changed")
        reader = current._authorize(state, intent, connection=connection)
        binding = intent["managedWorktree"]
        writer = state.check_authorization(identity["repositoryId"], binding["writerSessionId"],
                                           "implementation", intent["ownedPaths"], connection=connection)
        validation = state.check_authorization(identity["repositoryId"], binding["writerSessionId"],
                                               "validation", intent["ownedPaths"], connection=connection)
        formal._require(reader["snapshotDigest"] == intent["authorizationSnapshot"]
                        and writer["snapshotDigest"] == validation["snapshotDigest"]
                        == binding["writerAuthorizationSnapshot"],
                        "current_source_worktree_changed", "Prepared reader or writer authority changed")
        formal._require(not state.get("managed_worktree_reader_retirement", self.intent_digest,
                                      connection=connection),
                        "current_source_reader_retired", "Reuse the worktree with a fresh reader request")
        formal.assert_no_writer(state, self.worktree, connection=connection)
        formal._require(current._head(Path(intent["sourceRoot"])) == intent["baseHead"],
                        "current_source_base_changed", "Prepared worktree HEAD changed")
        return self.prepared


def _verify_prepared_worktree_flow(management, root, intent, *, proof=None):
    """Keep every source checkpoint, reusing only its held directory proofs."""
    from subprocess import SubprocessError
    from tools.jenkins import current_source as current
    from tools.jenkins import managed_worktree as formal
    from tools.jenkins.contracts import JenkinsError
    from tools.jenkins.source import _git_base_tree
    source = current._source_root(root, intent.get("sourceRoot"), intent.get("worktreeId"))
    prepared = (proof.check(intent) if proof is not None else
                formal.preparation(management.state, intent["worktreePreparationRef"],
                                   intent["worktreeId"])["payload"])
    formal._require(source != root and prepared.get("sourceKind") == "sealed-observed-canonical-input"
        and prepared.get("sourcePreparation") is True
        and prepared.get("implementationOwnershipTransferred") is False,
        "current_source_prepared_absence_unproven", "Absent owned inputs require a completed sealed WT preparation")
    if proof is None:
        authorization = current._authorize(management.state, intent)
        formal._require(authorization["snapshotDigest"] == intent["authorizationSnapshot"]
            and current._managed_worktree(management.state, root, intent, authorization) == intent["managedWorktree"],
            "current_source_worktree_changed", "Prepared source authority changed during Flow sealing")
    if current._head(source) != intent["baseHead"]:
        raise JenkinsError("current_source_base_changed", "Current checkout HEAD differs from its registered base")
    expected = intent["currentHashes"]
    # Only Git membership is metadata. Every selected path is pinned below.
    try:
        def selected_names(*flags):
            result = management.git(source, "ls-files", "-z", *flags, check=False)
            if result.returncode:
                raise JenkinsError("current_source_git_unavailable",
                                   "Cannot verify selected tracked and untracked source files")
            inventory = {raw.decode("utf-8", "strict").replace("\\", "/").casefold()
                         for raw in result.stdout.split(b"\0") if raw}
            return inventory & set(expected)
        tracked, untracked = selected_names("--cached"), selected_names("--others", "--exclude-standard")
    except (OSError, UnicodeError, SubprocessError) as error:
        raise JenkinsError("current_source_git_unavailable",
                           "Cannot verify selected tracked and untracked source files") from error
    selected = {path for path, checksum in expected.items() if checksum is not None and path not in tracked}
    if selected != set(intent["untrackedPaths"]) or not selected <= untracked:
        raise JenkinsError("current_source_untracked_mismatch", "Explicit untracked files differ from Git's selected current closure")
    checked_source_evidence(prepared["source"])
    absent, prepared_files = [], {name.casefold() for name in prepared["source"]["files"]}
    base_paths = _git_base_tree(root, intent["baseHead"])
    # Stages end at this checkpoint; neither leaf hashes nor absences survive it.
    with managed_input_stage(root, "flow-canonical-paths") as canonical, \
            managed_input_stage(source, "flow-source-bytes") as pinned:
        _managed_map(canonical, expected, metadata=True)
        try:
            actual = _managed_map(pinned, expected)
        except JenkinsError as error:
            if error.code != "managed_source_directory":
                raise
            path = Path(error.details["path"]).relative_to(source).as_posix().casefold()
            if expected[path] is None:
                raise JenkinsError("current_source_deletion_invalid", "Null source must remain absent",
                                   details={"path": path}) from error
            raise JenkinsError("current_source_file_missing", "Selected source must be a present ordinary file",
                               details={"path": path}) from error
        for path, checksum in expected.items():
            if checksum is None:
                if actual[path] is not None:
                    raise JenkinsError("current_source_deletion_invalid", "Null source must remain absent", details={"path": path})
                if path not in tracked:
                    formal._require(path in prepared["beforeHashes"]
                        and path in prepared["currentHashes"] and prepared["beforeHashes"][path] is None
                        and prepared["currentHashes"][path] is None and path not in prepared_files
                        and path not in base_paths,
                        "current_source_prepared_absence_unproven", "Untracked null input is not a proved unchanged owned absence")
                    # A frozen dependency absence is readable, without acquiring writer ownership.
                    if path in intent["ownedPaths"]:
                        absent.append(path)
            elif actual[path] is None:
                raise JenkinsError("current_source_file_missing", "Selected source must be a present ordinary file", details={"path": path})
            if actual[path] != checksum:
                raise JenkinsError("current_source_hash_mismatch", "Current source differs from its registered hash", details={"path": path})
    return sorted(absent)


def _ensure_prepared_flow_metadata(state, root, intent, sealed_result):
    """Keep the driver's metadata proof, passing its collector JSON-safe bytes."""
    from collections.abc import Mapping
    from tools.jenkins import patches
    from tools.jenkins.source import _thaw
    from tools.jenkins.workflow.planning import _cargo_evidence
    if state.get("environment_policy", "default") is None and not (
            root / ".jenkins/state/deployment/driver.json").is_file():
        return
    manifest, sealed, expected_build, objects = patches._sealed_metadata_binding(
        state, root, intent, sealed_result)
    environment, policy, cargo, rustc = patches._verified_metadata_inputs(state, root)
    policy_digest = digest(policy)
    source_digest = str(sealed_result["sourceDigest"])
    existing = state.get("sealed_cargo_evidence", source_digest)
    if existing is not None:
        patches._validate_existing_metadata_evidence(
            state, existing, identity=intent["identity"], source_digest=source_digest,
            build_root=expected_build, policy_digest=policy_digest)
        return
    # get_sealed_manifest deliberately freezes nested external manifests. The
    # unchanged collector's external JSON digest requires the equivalent plain
    # representation; its original binding, materialization and CAS checks stay.
    plain_manifest = _thaw(manifest.payload)
    evidence = patches.collect_cargo_evidence(
        plain_manifest, objects, expected_build, cargo=cargo, rustc=rustc,
        environment=environment, environment_policy_digest=policy_digest)
    if (not isinstance(evidence, Mapping)
            or evidence.get("sourceDigest") != source_digest
            or evidence.get("manifestDigest", source_digest) != source_digest
            or evidence.get("buildRoot") != expected_build
            or evidence.get("environmentPolicyDigest") != policy_digest):
        from tools.jenkins.contracts import JenkinsError
        raise JenkinsError("cargo_evidence_identity_mismatch",
                           "collected Cargo evidence is not bound to the sealed patch")
    patches._validate_metadata_workspace(evidence, expected_build, source_digest, manifest=manifest)
    patches.persist_cargo_evidence(
        state, evidence, repository_id_value=intent["identity"]["repositoryId"],
        session_id=intent["identity"]["sessionId"], owner=intent["identity"]["attemptId"],
        manifest=plain_manifest)
    _cargo_evidence(state, source_digest)


def register_prepared_worktree_flow(management, payload):
    """Register and genuinely prepare the narrow absent-owned WT Flow input.

    The original Flow consumes this typed preparation through its existing
    prepared branch. Neither the immutable driver nor its module globals change.
    """
    from tools.jenkins import current_source as current
    from tools.jenkins import managed_worktree as formal
    from tools.jenkins.contracts import JenkinsError, response
    from tools.jenkins.deployment.driver import _plain
    from tools.jenkins.product_inputs import compiler_controls, validate_hash_paths
    from tools.jenkins.source import PathClaim, SealedManifest, _thaw, register_sealed_consumer, seal_candidate
    from tools.jenkins.source.external_inputs import verify_objects
    from tools.jenkins.workflow.handler import _trusted_driver
    state, root = management.state, management.repo.absolute()
    _plain(root)
    formal._require(root.resolve() == root, "current_source_repository_alias",
                    "Current source requires the physical canonical checkout")
    identity = current._identity(payload)
    formal._require(identity["repositoryId"] == current.repository_id(root)
        and identity["stageImplementationDigest"] == management.config["driverDigest"]
        == _trusted_driver(root, identity), "driver_context_mismatch",
        "Prepared Flow source must retain its selected repository and sealed driver")
    hashes = _managed_hashes(root, payload.get("currentHashes"), "flow-intake-paths")
    source = current._source_root(root, payload.get("sourceRoot"), payload.get("worktreeId"))
    owned = current._paths(root, payload.get("ownedPaths"), "ownedPaths")
    controls = compiler_controls(payload.get("compilerControlInputs"), hashes, owned, managed=source != root)
    validate_hash_paths(hashes, controls)
    untracked = current._paths(root, payload.get("untrackedPaths"), "untrackedPaths")
    formal._require(source != root and controls is not None and owned
        and set(owned) <= set(hashes) and set(untracked) <= set(hashes),
        "current_source_scope_invalid", "Prepared Flow requires its complete managed source and narrow writer scope")
    base = payload.get("baseHead")
    formal._require(isinstance(base, str) and re.fullmatch(r"(?:[0-9a-f]{40}|[0-9a-f]{64})", base),
                    "current_source_base_missing", "Current source requires an immutable Git base revision")
    prepared_ref = payload.get("worktreePreparationRef")
    prepared_row, _ = _managed_route_preparation(
        state, prepared_ref, payload["worktreeId"], identity["repositoryId"], management.config)
    prepared = prepared_row["payload"]
    build = current._build_root(root, payload.get("buildRoot"))
    objects = current.canonical_build_root(build).namespace()
    formal._require(payload.get("externalInputs") == prepared["externalInputs"]
        and prepared["objectRoot"] == str(objects), "managed_source_preparation_mismatch",
        "Flow must retain the frozen external closure and object domain")
    supplied = {"schemaVersion": 1, "kind": "current-source", "identity": identity,
        "repositoryRoot": str(root), "baseHead": base, "currentHashes": hashes,
        "ownedPaths": owned, "untrackedPaths": untracked,
        "coverage": current.validate_coverage(payload.get("coverage")), "buildRoot": build,
        "runtimeOperationId": current._runtime_binding(root, identity, payload),
        "externalInputs": prepared["externalInputs"], "sourceRoot": str(source),
        "worktreeId": payload["worktreeId"], "compilerControlInputs": controls,
        "worktreePreparationRef": prepared_ref, "patchOperationKeys": payload.get("patchOperationKeys", [])}
    slot = digest([identity[name] for name in ("repositoryId", "sessionId", "requestId")])
    with current._lock(root):
        authorization = current._authorize(state, supplied)
        proof = _PreparedWorktreeFlowProof(management, supplied, prepared_row)
        supplied["managedWorktree"] = current._managed_worktree(state, root, supplied, authorization)
        intent = {**supplied, "authorizationSnapshot": authorization["snapshotDigest"]}
        # The full original validator accepted exactly the pinned rows above.
        proof.bind(intent)
        reference = digest(intent)
        if state.get("managed_worktree_reader_retirement", reference):
            raise JenkinsError("current_source_reader_retired", "Reuse the worktree with a fresh reader request")
        registered = state.get("current_source_request", slot)
        reused = registered is not None
        absent = None
        if registered:
            formal._require(registered["payload"].get("currentSourceRef") == reference,
                "current_source_request_conflict", "This request already owns different current-source input")
            stored = state.get("current_source_intent", reference)
            formal._require(stored is not None and stored["payload"] == intent,
                "current_source_intent_missing", "Existing current-source intent is missing or corrupt")
        else:
            formal._require(not state.get_request(identity["repositoryId"], identity["sessionId"], identity["requestId"])
                and not state.get("patch_request", slot), "current_source_request_conflict",
                "An existing request or patch cannot be adopted by current-source intake")
            absent = _verify_prepared_worktree_flow(management, root, intent, proof=proof)
            with state.transaction() as connection:
                proof.check(intent, connection=connection)
                formal._require(not state.get("current_source_request", slot, connection=connection)
                    and not state.get_request(identity["repositoryId"], identity["sessionId"], identity["requestId"], connection=connection)
                    and not state.get("patch_request", slot, connection=connection),
                    "current_source_request_conflict", "A competing request was admitted before intake commit")
                state.put("current_source_intent", reference, intent, expected_version=0, connection=connection)
                state.put("current_source_request", slot, {"currentSourceRef": reference}, expected_version=0, connection=connection)
        previous = state.get("current_source_preparation", reference)
        if not previous or previous["payload"].get("status") != "prepared":
            operation_id = "current-source-" + reference
            operation = state.put_operation("source_preparation", operation_id, intent)
            if absent is None:
                absent = _verify_prepared_worktree_flow(management, root, intent, proof=proof)
            proof.check(intent)
            formal._require(bool(absent), "current_source_prepared_absence_unproven",
                            "This adapter only prepares proved unchanged owned absences")
            external = prepared["sourceManifest"].get("external", [])
            claims = [PathClaim(path, identity["attemptId"] if path in owned else "dependency",
                                0, checksum, 0, identity["repositoryId"]) for path, checksum in hashes.items()]
            manifest = seal_candidate(root, claims, object_root=objects, base_head=base, source_root=source)
            plain = _thaw(manifest.payload)
            plain.update(preparationKind="current-source", validationCoverageDigest=digest(intent["coverage"]),
                         external=external, worktreePreparationRef=prepared_ref,
                         compilerControlInputs=controls, preparedAbsentOwnedPaths=absent)
            manifest = SealedManifest(plain, digest(plain))
            formal._require(_verify_prepared_worktree_flow(management, root, intent, proof=proof) == absent,
                "current_source_preparation_changed", "Prepared WT or frozen external inputs changed while sealing")
            # Reject corrupt frozen dependencies before publishing a sealed input.
            verify_objects(plain, objects)
            coverage = {**intent["coverage"], "sourceDigest": manifest.source_digest}
            coverage["coverageDigest"] = digest(coverage)
            sealed = {"status": "sealed", "repositoryRoot": str(root), "repositoryId": identity["repositoryId"],
                "sourceDigest": manifest.source_digest, "sourceManifest": plain, "manifest": plain,
                "coverage": coverage, "coverageDigest": coverage["coverageDigest"], "objectRoot": str(objects),
                "buildRoot": build, "owner": identity["attemptId"], "sessionId": identity["sessionId"],
                "ownedPaths": owned, "claims": [], "patchOperationRefs": [], "sourcePreparationRefs": [operation_id]}
            old = state.get("sealed_input", manifest.source_digest)
            if old:
                formal._require(all(old["payload"].get(name) == sealed[name] for name in
                    ("repositoryId", "repositoryRoot", "objectRoot", "sourceManifest", "coverageDigest")),
                    "sealed_input_conflict", "Current-source identity cannot replace different sealed inputs")
            else:
                state.put("sealed_input", manifest.source_digest, sealed, expected_version=0)
            register_sealed_consumer(state, repository_id_value=identity["repositoryId"],
                session_id=identity["sessionId"], owner=identity["attemptId"],
                source_digest=manifest.source_digest, manifest_digest=manifest.source_digest)
            _ensure_prepared_flow_metadata(state, root, intent, sealed)
            formal._require(_verify_prepared_worktree_flow(management, root, intent, proof=proof) == absent,
                "current_source_preparation_changed", "Prepared input changed before preparation completion")
            # Re-read the original inventory and all external CAS before completion.
            current._load_intent(state, root, {"identity": identity, "currentSourceRef": reference,
                                              "buildRoot": build, "runtimeOperationId": intent["runtimeOperationId"]})
            result = response("prepared", identity={**identity, "sourceInputDigest": manifest.source_digest,
                "coverageDigest": coverage["coverageDigest"]}, sealedInputRef=manifest.source_digest,
                currentSourceRef=reference, sourcePreparationRef=operation_id, buildRoot=build)
            if operation["status"] == "complete":
                formal._require(operation.get("result", {}).get("sourceDigest") == manifest.source_digest
                    and operation["result"].get("coverageDigest") == coverage["coverageDigest"],
                    "source_preparation_untrusted", "Completed preparation cannot change its sealed identity")
            else:
                state.transition_operation(operation_id, operation["status"], "complete", {
                    "sourceDigest": manifest.source_digest, "coverageDigest": coverage["coverageDigest"],
                    "currentHashes": hashes, "ownedPaths": owned, "untrackedPaths": untracked,
                    "externalInputs": intent["externalInputs"], "canonicalSourceModified": False,
                    "preparedAbsentOwnedPaths": absent})
            state.put("current_source_preparation", reference, {"status": "prepared", "result": result},
                      expected_version=previous["version"] if previous else 0)
    # Consume the genuine prepared branch, including its typed operation and
    # engine/external CAS checks. This also admits the original request exactly.
    prepared_result = current.prepare_current_source(state, root, {
        "identity": identity, "currentSourceRef": reference, "buildRoot": build,
        "runtimeOperationId": intent["runtimeOperationId"]},
        trusted_driver_digest=management.config["driverDigest"])
    return response("registered", currentSourceRef=reference, reused=reused,
                    sourcePreparation=prepared_result)


def resume_prepared_flow_execution(management, payload, *, transport=None):
    """Resume one typed Flow's existing claim, then collect its real receipts."""
    from tools.jenkins import current_source as current, managed_worktree as formal
    from tools.jenkins.contracts import JenkinsError, digest
    from tools.jenkins.deployment.driver import active_driver_binding
    from tools.jenkins.submission import _submission_fields, request_payload, submit
    from tools.jenkins.workflow import handler as flow_handler

    state, root = management.state, management.repo
    continuation = payload.get("resumeExecution")
    if (not isinstance(continuation, dict)
            or set(continuation) - {"executionId", "recipeRef", "phase"}
            or continuation.get("phase", "dispatch") not in {"dispatch", "collect"}):
        raise JenkinsError("execution_resume_invalid", "Resume requires its original execution and recipe")
    reference = payload.get("currentSourceRef")
    if not isinstance(reference, str) or len(reference) != 64:
        raise JenkinsError("execution_resume_identity_mismatch", "Resume requires its original current-source reference")
    intent_row = state.get("current_source_intent", reference)
    intent = intent_row["payload"] if intent_row else {}
    if (not intent_row or digest(intent) != reference
            or payload.get("identity") != intent.get("identity")
            or intent.get("repositoryId", intent.get("identity", {}).get("repositoryId")) != management.repo_id
            or intent.get("identity", {}).get("sessionId") != management.session):
        raise JenkinsError("execution_resume_identity_mismatch", "Resume must retain the original current-source author")
    authorization = current._authorize(state, intent)
    if authorization["snapshotDigest"] != intent.get("authorizationSnapshot"):
        raise JenkinsError("current_source_authorization_changed", "Resume authority differs from original intake")
    if any((payload.get("authorization") or {}).get(name) is True for name in ("allowCommit", "allowNotify", "allowPush")):
        raise JenkinsError("execution_resume_action_forbidden", "This continuation resumes validation and acceptance only")
    prepared_row = state.get("current_source_preparation", reference)
    prepared = prepared_row["payload"].get("result", {}) if prepared_row else {}
    operation_ref = prepared.get("sourcePreparationRef")
    operation = state.get_operation(operation_ref) if operation_ref else None
    if (not prepared_row or prepared_row["payload"].get("status") != "prepared"
            or operation_ref != "current-source-" + reference
            or not operation or operation["kind"] != "source_preparation" or operation["status"] != "complete"
            or operation["payloadDigest"] != digest(operation["payload"]) or operation["payload"] != intent
            or not operation.get("result")
            or operation["result"].get("sourceDigest") != prepared.get("sealedInputRef")
            or operation["result"].get("coverageDigest") != prepared.get("identity", {}).get("coverageDigest")
            or any(operation["result"].get(name) != intent.get(name)
                   for name in ("currentHashes", "ownedPaths", "untrackedPaths", "externalInputs"))):
        raise JenkinsError("source_preparation_untrusted", "Resume needs the exact completed typed preparation")
    identity = prepared.get("identity", {})
    key = ":".join(str(identity.get(name, "")) for name in ("sessionId", "requestId", "attemptId"))
    workflow_row = state.get("workflow", key)
    workflow = workflow_row["payload"] if workflow_row else {}
    execution_id, recipe_ref = continuation.get("executionId"), continuation.get("recipeRef")
    source = prepared.get("sealedInputRef")
    if (not isinstance(execution_id, str) or len(execution_id) != 32
            or not isinstance(recipe_ref, str) or len(recipe_ref) != 64
            or not workflow_row or workflow.get("identity") != identity
            or workflow.get("executionId") != execution_id or workflow.get("recipeRef") != recipe_ref
            or workflow.get("sourceDigest") != source
            or workflow.get("coverageDigest") != prepared.get("identity", {}).get("coverageDigest")
            or workflow.get("sourcePreparationRef") != operation_ref
            or workflow.get("status") in {"failed", "cancelled", "rejected"}):
        raise JenkinsError("execution_resume_identity_mismatch", "Resume cannot replace the registered Flow or claim")
    current.require_source_preparation(state, operation_ref, identity=identity, source_digest=source)
    active = active_driver_binding(root)
    runtime = active.get("runtimeOperationId") or active.get("generation")
    if (active.get("digest") != management.config["driverDigest"]
            or runtime != management.config["runtimeOperationId"] or intent.get("runtimeOperationId") != runtime
            or identity.get("stageImplementationDigest") != active.get("digest")):
        raise JenkinsError("runtime_binding_mismatch", "Resume must retain the selected driver and runtime")
    flow_handler._require_registered_execution(state, root, workflow, key)
    original = state.get_request(management.repo_id, management.session, identity["requestId"])
    expected = current._request(intent, prepared)
    if (not original or original.get("payloadDigest") != digest(original["payload"])
            or original["payload"] != expected or original["generation"] != identity["generation"]):
        raise JenkinsError("submission_identity_mismatch", "Resume must retain the immutable registered request")
    sealed_row = state.get("sealed_input", source)
    if (not sealed_row or sealed_row["payload"].get("sourceDigest") != source
            or digest(sealed_row["payload"].get("sourceManifest")) != source):
        raise JenkinsError("current_source_sealed_missing", "Resume requires the original sealed manifest")
    pins = []
    for domain, row_key in (("current_source_intent", reference), ("current_source_preparation", reference),
                            ("workflow", key), ("recipe_plan", recipe_ref), ("execution", execution_id), ("sealed_input", source)):
        row = state.get(domain, row_key)
        if row is None:
            raise JenkinsError("execution_resume_identity_mismatch", "Resume authority disappeared")
        pins.append({"domain": domain, "key": row_key, "version": row["version"],
                     "payloadDigest": digest(row["payload"])})
    submission_id = identity["requestId"] + ":execution:" + recipe_ref[:16]
    request = request_payload(repository_id=management.repo_id, session_id=management.session,
        request_id=identity["requestId"], source_ref=source, source_digest=source,
        coverage_ref=identity["coverageDigest"], coverage_digest=identity["coverageDigest"],
        parameters={"recipeRef": recipe_ref, "executionId": execution_id,
                    "attemptId": identity["attemptId"], "generation": str(identity["generation"]),
                    "submissionRequestId": submission_id})
    fields = _submission_fields(expected)
    for name in ("CURRENT_SOURCE_REF", "SOURCE_PREPARATION_REF", "PATCH_REQUEST_REF",
                 "PATCH_OPERATION_REF", "STAGE_IMPLEMENTATION_DIGEST", "RUNTIME_OPERATION_ID", "BUILD_ROOT"):
        if name in fields:
            request["parameters"][name] = fields[name]
    request["parameters"]["RUNTIME_OPERATION_ID"] = runtime
    with state.transaction() as connection:
        if not formal._pins_current(state, pins, connection=connection):
            raise JenkinsError("execution_resume_conflict", "Resume authority changed before dispatch", retryable=True)
        if current._authorize(state, intent, connection=connection)["snapshotDigest"] != intent["authorizationSnapshot"]:
            raise JenkinsError("current_source_authorization_changed", "Resume authority changed before continuation")
        checked = state.get_request(management.repo_id, management.session, identity["requestId"], connection=connection)
        typed = state.get_operation(operation_ref, connection=connection)
        if (not checked or checked["payloadDigest"] != original["payloadDigest"]
                or not typed or typed["version"] != operation["version"] or typed != operation):
            raise JenkinsError("execution_resume_conflict", "Typed request changed before continuation", retryable=True)
    if continuation.get("phase", "dispatch") == "dispatch":
        delivery = submit(state, request, job="zircon-execution", transport=transport,
                          manager=management.manager if transport is None else None)
        fresh = state.get("workflow", key)
        if (not fresh or fresh["payload"].get("identity") != identity
                or fresh["payload"].get("executionId") != execution_id or fresh["payload"].get("recipeRef") != recipe_ref):
            raise JenkinsError("execution_resume_conflict", "Original Flow changed while delivery was observed", retryable=True)
        state.put("workflow", key, {**fresh["payload"],
            "jenkinsSubmission": {name: value for name, value in delivery.items() if name != "request"}},
            expected_version=fresh["version"])
        return {"status": "submitted", "executionId": execution_id, "recipeRef": recipe_ref,
                "flowId": key, "submission": delivery, "formalAcceptance": False}
    from tools.jenkins.submission import _marker_key
    marker = state.get("jenkins_submission", _marker_key(request))
    if (not marker or marker["payload"].get("payloadDigest") != digest(request)
            or not marker["payload"].get("dispatchDigest")
            or marker["payload"].get("delivery") not in {"submitted", "accepted", "completed"}):
        raise JenkinsError("delivery_unknown", "Collect requires the exact reconciled execution delivery", retryable=True)
    observed = flow_handler.handle("observe-flow", {"identity": identity, "executionId": execution_id}, state, root)
    if observed.get("status") not in {"ready", "accepted"}:
        return {"status": observed["status"] if observed.get("status") in {"failed", "cancelled", "rejected", "unknown"} else "pending",
                "executionId": execution_id, "recipeRef": recipe_ref, "flowId": key,
                "observation": observed, "formalAcceptance": False}
    accepted = flow_handler.handle("run-stage", {"identity": identity, "executionId": execution_id,
        "flowId": key, "stage": "acceptance"}, state, root)
    if accepted.get("status") != "accepted":
        raise JenkinsError("flow_unaccepted", "Original Flow acceptance did not complete")
    finalized = flow_handler.handle("finalize-flow", {"identity": identity, "executionId": execution_id,
        "commitRequested": False, "notifyRequested": False}, state, root)
    return {"status": "accepted", "executionId": execution_id, "recipeRef": recipe_ref,
            "flowId": key, "acceptance": accepted, "finalization": finalized, "formalAcceptance": False}


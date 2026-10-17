"""Operator entry for the selected sealed Jenkins incremental patch protocol.

Use --already-applied for checkout edits, with immutable --base-head and exact
--before-hashes, --after-hashes and --dependency-hashes JSON objects (or paths).
An empty dependency set still requires --dependency-hashes '{}'. The coordinator
owns baseline verification, candidate preparation and build acceptance.
"""
from __future__ import annotations

import argparse
from contextlib import closing
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import sqlite3
import stat
import subprocess
import sys
import uuid

MAX_PATCH_BYTES = 4 * 1024 * 1024
DEFAULT_BUILD_ROOT = r"D:\cargo-targets"
HEX64 = re.compile(r"[0-9a-f]{64}\Z")
BASE_HEAD = re.compile(r"(?:[0-9a-f]{40}|[0-9a-f]{64})\Z")
IDENTIFIER = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.:\-]{0,127}\Z")
BUILD_ROOT = re.compile(r"(?i)[DEF]:\\cargo-targets\Z")
HUNK = re.compile(r"@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@(?: .*)?\Z")
ALREADY_APPLIED_DRIVER_FILES = frozenset({
    "tools/jenkins/already_applied.py", "tools/jenkins/source/baselines.py",
})


class EntryError(RuntimeError):
    def __init__(self, code, message, *, retryable=False, details=None):
        super().__init__(message)
        self.code, self.retryable, self.details = code, retryable, details or {}


def canonical_json(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True,
                      separators=(",", ":"), allow_nan=False).encode("utf-8")


def digest(value):
    return hashlib.sha256(canonical_json(value)).hexdigest()


def repository_id(root):
    return hashlib.sha256(str(root.absolute().resolve()).casefold().encode("utf-8")).hexdigest()[:32]


def require_identifier(value, name):
    if not isinstance(value, str) or not IDENTIFIER.fullmatch(value):
        raise EntryError("invalid_identity", f"Invalid {name}")
    return value


def plain_path(path):
    for part in (path, *path.parents):
        try:
            metadata = part.lstat()
        except (FileNotFoundError, NotADirectoryError):
            continue
        if stat.S_ISLNK(metadata.st_mode) or getattr(metadata, "st_file_attributes", 0) & 0x400:
            raise EntryError("path_alias_rejected", "Protocol paths must have a physical identity")


def read_json(value, name):
    text = os.fspath(value)
    try:
        if not text.lstrip().startswith(("{", "[")):
            path = Path(text)
            if path.stat().st_size > 8 * 1024 * 1024:
                raise EntryError("input_too_large", f"{name} exceeds the metadata limit")
            text = path.read_text(encoding="utf-8-sig")
        return json.loads(text)
    except (OSError, ValueError) as error:
        raise EntryError("invalid_input", f"Cannot read {name} as JSON") from error


def relative_path(root, value):
    if not isinstance(value, str):
        raise EntryError("invalid_source_path", "Source paths must be strings")
    raw = value.replace("\\", "/")
    if not raw or raw.startswith("/") or ":" in raw or any(p in {"", ".", ".."} for p in raw.split("/")):
        raise EntryError("invalid_source_path", "Source paths must be repository-relative")
    path = root.joinpath(*raw.split("/"))
    plain_path(path)
    if not path.resolve().is_relative_to(root.resolve()):
        raise EntryError("invalid_source_path", "Source path escapes repository")
    return raw.casefold()


def patch_paths(root, text, *, include_existence=False):
    """Validate unified headers and hunk counts without applying a patch."""
    if not text or len(text.encode("utf-8")) > MAX_PATCH_BYTES:
        raise EntryError("patch_input_invalid", "Patch is empty or exceeds 4 MiB")
    lines, paths, current, hunks = text.splitlines(), {}, None, 0
    index = 0
    while index < len(lines):
        line = lines[index]
        if line.startswith("--- "):
            if current is not None and not hunks:
                raise EntryError("patch_input_invalid", "Each patched file requires a hunk")
            if index + 1 >= len(lines) or not lines[index + 1].startswith("+++ "):
                raise EntryError("patch_input_invalid", "Patch requires paired file headers")
            old, new = line[4:], lines[index + 1][4:]
            def header(value, prefix):
                if value == "/dev/null":
                    return None
                if not value.startswith(prefix):
                    raise EntryError("patch_input_invalid", "Patch headers require a/ and b/ prefixes")
                return relative_path(root, value[len(prefix):])
            before, after = header(old, "a/"), header(new, "b/")
            current = after if after is not None else before
            if current is None or (before is not None and after is not None and before != after) or current in paths:
                raise EntryError("patch_scope_mismatch", "Patch paths are duplicated or renamed")
            paths[current] = (before is not None, after is not None) if include_existence else before is None
            hunks, index = 0, index + 2
            continue
        match = HUNK.fullmatch(line)
        if match:
            if current is None:
                raise EntryError("patch_input_invalid", "Hunk has no file headers")
            old_count = int(match[2]) if match[2] is not None else 1
            new_count = int(match[4]) if match[4] is not None else 1
            index += 1
            while old_count or new_count:
                if index >= len(lines) or not lines[index]:
                    raise EntryError("patch_input_invalid", "Hunk line counts do not match")
                content = lines[index]
                if content == "\\ No newline at end of file":
                    index += 1
                    continue
                prefix = content[0]
                old_count -= prefix in {" ", "-"}
                new_count -= prefix in {" ", "+"}
                if prefix not in {" ", "-", "+"} or min(old_count, new_count) < 0:
                    raise EntryError("patch_input_invalid", "Hunk line counts do not match")
                index += 1
            if index < len(lines) and lines[index] == "\\ No newline at end of file":
                index += 1
            hunks += 1
            continue
        if line and not line.startswith(("diff --git ", "index ", "new file mode ", "deleted file mode ")):
            raise EntryError("patch_input_invalid", "Unsupported patch metadata or content")
        index += 1
    if not paths or not hunks:
        raise EntryError("patch_input_invalid", "Patch requires file headers and hunks")
    return paths


def selected_driver(root):
    """Read and verify the current selector; do not seal, start, or authorize."""
    marker = root / ".jenkins/state/deployment/driver.json"
    plain_path(marker)
    record = read_json(marker, "selected driver")
    checksum = record.get("driverDigest")
    if not isinstance(checksum, str) or not HEX64.fullmatch(checksum):
        raise EntryError("driver_unavailable", "Selected driver digest is unavailable")
    driver_root = root / ".jenkins/runtime/drivers" / checksum
    plain_path(driver_root)
    expected = {"root": driver_root, "launcher": driver_root / "launcher.py",
                "steps": driver_root / "pipeline/steps.groovy"}
    for name, path in expected.items():
        if record.get(name) is not None and Path(record[name]).absolute() != path:
            raise EntryError("driver_context_mismatch", "Selected driver paths differ from its digest")
    runtime = record.get("runtimeOperationId")
    if not isinstance(runtime, str) or not runtime:
        raise EntryError("runtime_binding_missing", "Selected driver has no runtime operation identity")
    manifest = read_json(driver_root / "driver-manifest.json", "driver manifest")
    files = manifest.get("files")
    if (manifest.get("schemaVersion") != 1 or not isinstance(files, dict) or not files
            or digest({"schemaVersion": 1, "files": files}) != checksum
            or manifest.get("driverDigest") != checksum):
        raise EntryError("driver_integrity_failed", "Selected driver manifest identity differs")
    for name, entry in files.items():
        relative = PurePosixPath(name)
        if (relative.is_absolute() or not relative.parts or "\\" in name or ":" in name
                or any(p in {".", ".."} for p in relative.parts)):
            raise EntryError("driver_integrity_failed", "Driver manifest path is invalid")
        path = driver_root.joinpath(*relative.parts)
        plain_path(path)
        data = path.read_bytes()
        if len(data) != entry.get("bytes") or hashlib.sha256(data).hexdigest() != entry.get("sha256"):
            raise EntryError("driver_integrity_failed", "Selected driver bytes differ from its manifest")
    observed = set()
    for path in driver_root.rglob("*"):
        plain_path(path)
        if path.is_file():
            observed.add(path.relative_to(driver_root).as_posix())
    if observed != set(files) | {"driver-manifest.json"} or not {"launcher.py", "pipeline/steps.groovy"} <= set(files):
        raise EntryError("driver_integrity_failed", "Selected driver has undeclared or missing files")
    python = root / ".jenkins/runtime/python/python.exe"
    plain_path(python)
    if not python.is_file():
        raise EntryError("driver_unavailable", "Dedicated Jenkins Python runtime is unavailable")
    env = dict(os.environ)
    env.update({"ZIRCON_DRIVER_DIGEST": checksum, "ZIRCON_DRIVER_LAUNCHER": str(expected["launcher"]),
                "ZIRCON_SEALED_DRIVER": str(expected["steps"]), "JENKINS_PYTHON": str(python),
                "ZIRCON_REPO_ROOT": str(root), "ZIRCON_REPOSITORY_ID": digest(str(root).casefold()),
                "PYTHONDONTWRITEBYTECODE": "1"})
    return {"digest": checksum, "runtimeOperationId": runtime, "python": python,
            "launcher": expected["launcher"], "env": env, "manifestFiles": tuple(sorted(files))}


def build_root(root, requested):
    if requested is None:
        spec = root / ".jenkins/deployment-spec.json"
        requested = read_json(spec, "deployment specification").get("storage", {}).get("buildRoot") if spec.is_file() else None
    value = requested or DEFAULT_BUILD_ROOT
    if not isinstance(value, str) or not BUILD_ROOT.fullmatch(value):
        raise EntryError("build_root_rejected", "Build root must be D:, E:, or F: followed by \\cargo-targets")
    value = value[0].upper() + value[1:]
    if os.name == "nt":
        path = Path(value)
        plain_path(path)
        if str(path.resolve()).casefold() != value.casefold():
            raise EntryError("build_root_rejected", "Build root must have its declared physical identity")
    return value


def already_applied_mode(args):
    enabled = bool(getattr(args, "already_applied", False))
    if not enabled and any(getattr(args, name, None) is not None
                           for name in ("after_hashes", "dependency_hashes", "baseline_ref")):
        raise EntryError("already_applied_flags_conflict",
                         "--after-hashes, --dependency-hashes and --baseline-ref require --already-applied")
    return enabled


def require_already_applied_driver(driver):
    if not ALREADY_APPLIED_DRIVER_FILES <= set(driver.get("manifestFiles", ())):
        raise EntryError("patch_handoff_unavailable",
                         "The selected sealed driver does not declare already-applied patch and baseline support")


def supplied_hashes(root, value, name, code, *, allow_null=False, allow_empty=False):
    original = read_json(value, name)
    if not isinstance(original, dict) or (not original and not allow_empty):
        raise EntryError(code, f"{name.capitalize()} must be a JSON object" +
                         ("" if allow_empty else " with at least one path"))
    hashes = {}
    for source, checksum in original.items():
        path = relative_path(root, source)
        if (path in hashes or (checksum is None and not allow_null)
                or (checksum is not None and (not isinstance(checksum, str) or not HEX64.fullmatch(checksum)))):
            raise EntryError(code, f"{name.capitalize()} require unique paths and SHA-256" +
                             (" or null" if allow_null else ""))
        hashes[path] = checksum
    return hashes


def current_file_hash(root, source, code):
    path = root.joinpath(*source.split("/"))
    plain_path(path)
    if not path.is_file() or not stat.S_ISREG(path.lstat().st_mode):
        raise EntryError(code, "Declared content must be an existing ordinary file", details={"path": source})
    return hashlib.sha256(path.read_bytes()).hexdigest()


def intent_payload(args, root, identity):
    already_applied = already_applied_mode(args)
    if already_applied:
        if not isinstance(args.base_head, str) or not BASE_HEAD.fullmatch(args.base_head):
            raise EntryError("patch_base_invalid", "--already-applied requires an immutable --base-head Git revision")
        if getattr(args, "after_hashes", None) is None or getattr(args, "dependency_hashes", None) is None:
            raise EntryError("already_applied_inputs_missing",
                             "--already-applied requires --after-hashes and explicit --dependency-hashes (use '{}' when empty)")
    if not args.patch or not args.before_hashes:
        raise EntryError("patch_inputs_missing", "--patch and --before-hashes are required")
    path = Path(args.patch)
    if path.stat().st_size > MAX_PATCH_BYTES:
        raise EntryError("patch_input_invalid", "Patch exceeds 4 MiB")
    text = path.read_bytes().decode("utf-8-sig")
    before = read_json(args.before_hashes, "before hashes")
    if not isinstance(before, dict) or not before:
        raise EntryError("before_hashes_invalid", "Before hashes must be a nonempty JSON object")
    hashes = {}
    for source, checksum in before.items():
        name = relative_path(root, source)
        if name in hashes or (checksum is not None and (not isinstance(checksum, str) or not HEX64.fullmatch(checksum))):
            raise EntryError("before_hashes_invalid", "Before hashes require unique paths and SHA-256 or null")
        hashes[name] = checksum
    parsed = patch_paths(root, text, include_existence=already_applied)
    paths = {name: not exists[0] for name, exists in parsed.items()} if already_applied else parsed
    if set(paths) != set(hashes) or any(is_new != (hashes[path] is None) for path, is_new in paths.items()):
        raise EntryError("patch_scope_mismatch", "Patch paths and before hashes must match; additions require null")
    if args.coverage and args.coverage_template:
        raise EntryError("coverage_conflict", "Use --coverage or --coverage-template")
    coverage = read_json(args.coverage, "coverage") if args.coverage else {"template": args.coverage_template}
    if (not isinstance(coverage, dict) or not coverage or coverage == {"template": None}
            or {"sourceDigest", "coverageDigest"} & set(coverage)):
        raise EntryError("coverage_missing", "Provide coverage without source-derived digests")
    dependencies = sorted({relative_path(root, p) for p in args.dependency} - set(hashes))
    if any(not root.joinpath(*p.split("/")).is_file() for p in dependencies):
        raise EntryError("patch_dependency_invalid", "Dependencies must name existing files")
    payload = {"identity": identity, "unifiedPatch": text, "beforeHashes": hashes,
               "declaredDependencies": dependencies, "coverage": coverage, "externalInputs": [],
               "baseHead": args.base_head, "buildRoot": build_root(root, args.build_root),
               "ownedPaths": sorted(hashes)}
    if already_applied:
        after = supplied_hashes(root, args.after_hashes, "after hashes", "after_hashes_invalid", allow_null=True)
        if set(after) != set(hashes) or any(exists[1] != (after[name] is not None) for name, exists in parsed.items()):
            raise EntryError("patch_scope_mismatch", "After hashes must match patch paths; deletions require null")
        for source, checksum in after.items():
            if checksum is None:
                if root.joinpath(*source.split("/")).exists():
                    raise EntryError("patch_after_hash_mismatch", "Declared deletion is still present", details={"path": source})
            else:
                observed = current_file_hash(root, source, "patch_after_hash_mismatch")
                if observed != checksum:
                    raise EntryError("patch_after_hash_mismatch", "Current content differs from its declared after hash",
                                     details={"path": source, "expected": checksum, "observed": observed})
        dependency_hashes = supplied_hashes(root, args.dependency_hashes, "dependency hashes",
                                             "dependency_hashes_invalid", allow_empty=True)
        if set(dependency_hashes) != set(dependencies):
            raise EntryError("dependency_hashes_invalid", "Dependency hashes must exactly match declared dependencies excluding patch paths")
        for source, checksum in dependency_hashes.items():
            observed = current_file_hash(root, source, "patch_dependency_hash_mismatch")
            if observed != checksum:
                raise EntryError("patch_dependency_hash_mismatch", "Current dependency differs from its declared hash",
                                 details={"path": source, "expected": checksum, "observed": observed})
        payload.update(inputMode="already-applied", afterHashes=after, dependencyHashes=dependency_hashes)
        if getattr(args, "baseline_ref", None) is not None:
            payload["baselineRef"] = require_identifier(args.baseline_ref, "baselineRef")
    return payload


def save_json(path, value, *, immutable=False):
    plain_path(path)
    if immutable and path.exists():
        if read_json(path, "saved protocol input") != value:
            raise EntryError("request_input_conflict", "This identity already has different protocol input")
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + ".pending-" + uuid.uuid4().hex)
    try:
        temporary.write_bytes(canonical_json(value) + b"\n")
        temporary.replace(path)
    finally:
        temporary.unlink(missing_ok=True)


def protocol(root, driver, folder, domain, action, payload, timeout):
    path = folder / (domain + "-" + action + ".json")
    save_json(path, payload, immutable=action not in {"query", "get", "status"})
    if domain == "patch" and action == "register":
        admit_local_validation(root, driver, payload)
    command = [str(driver["python"]), "-B", str(driver["launcher"]), "--repository", str(root),
               domain, action, "--input-file", str(path)]
    try:
        result = subprocess.run(command, cwd=root, env=driver["env"], capture_output=True,
                                text=True, encoding="utf-8", timeout=timeout, check=False)
    except (subprocess.TimeoutExpired, OSError) as error:
        raise EntryError("protocol_delivery_unknown", "Protocol call was interrupted; query this identity",
                         retryable=True) from error
    try:
        value = json.loads(result.stdout)
        if not isinstance(value, dict):
            raise ValueError()
    except ValueError as error:
        raise EntryError("protocol_delivery_unknown", "Protocol response is unavailable; query this identity",
                         retryable=True) from error
    if result.returncode:
        raise EntryError(value.get("reasonCode", "protocol_failed"), value.get("message", "Protocol rejected the request"),
                         retryable=bool(value.get("retryable")), details={"protocol": value})
    return value


def readonly_flow(root, base_identity, final_identity, expected_build_root):
    """Project the actual State schema without running a mutating flow action."""
    if (not isinstance(final_identity, dict)
            or any(final_identity.get(key) != value for key, value in base_identity.items())
            or any(not isinstance(final_identity.get(key), str) or not HEX64.fullmatch(final_identity[key])
                   for key in ("sourceInputDigest", "coverageDigest"))):
        raise EntryError("workflow_identity_mismatch", "Prepared identity differs from this patch request")
    database = root / ".jenkins/state/coordination.sqlite3"
    plain_path(database)
    if not database.is_file():
        return {"status": "pending", "formalAcceptance": False, "reasonCode": "state_unavailable"}
    flow_key = ":".join(final_identity[key] for key in ("sessionId", "requestId", "attemptId"))
    try:
        with closing(sqlite3.connect(database.absolute().as_uri() + "?mode=ro", uri=True, timeout=3)) as connection:
            connection.execute("PRAGMA query_only=ON")
            connection.execute("BEGIN")
            def record(domain, key):
                row = connection.execute("SELECT payload FROM records WHERE domain=? AND key=?", (domain, key)).fetchone()
                return json.loads(row[0]) if row else None
            workflow = record("workflow", flow_key)
            if workflow is None:
                return {"status": "pending", "formalAcceptance": False, "reasonCode": "workflow_not_registered"}
            expected = {"repositoryId": final_identity["repositoryId"],
                        "sourceDigest": final_identity["sourceInputDigest"],
                        "coverageDigest": final_identity["coverageDigest"],
                        "driverDigest": final_identity["stageImplementationDigest"],
                        "buildRoot": expected_build_root}
            if (not isinstance(workflow, dict) or workflow.get("identity") != final_identity
                    or any(workflow.get(key) != value for key, value in expected.items())):
                raise EntryError("workflow_identity_mismatch", "Stored workflow differs from this exact source, coverage, driver or build root")
            status = workflow.get("status", "pending")
            value = {"status": status, "formalAcceptance": False, "flowId": flow_key,
                     "executionId": workflow.get("executionId")}
            if status != "accepted":
                return value
            receipt = workflow.get("acceptance")
            if (not isinstance(receipt, dict) or receipt.get("schemaVersion") != 1
                    or receipt.get("kind") != "jenkins.authoritative.acceptance-receipt"
                    or receipt.get("authority") != "jenkins-validation" or receipt.get("status") != "accepted"
                    or any(receipt.get(key) != expected[key] for key in ("sourceDigest", "coverageDigest", "driverDigest"))
                    or receipt.get("receiptDigest") != digest({k: v for k, v in receipt.items() if k != "receiptDigest"})):
                raise EntryError("acceptance_receipt_invalid", "Stored acceptance lacks the matching authoritative receipt digest")
            flow_identity = {key: workflow.get(key) for key in
                             ("sourceDigest", "coverageDigest", "driverDigest", "generation", "beforePatch", "dagDigest")}
            stages = workflow.get("requiredStages")
            acceptance_id = digest({"flow": flow_key, "identity": flow_identity, "stages": stages})
            stage_items = workflow.get("stages", [])
            if not isinstance(stage_items, list) or any(not isinstance(item, dict) for item in stage_items):
                raise EntryError("acceptance_receipt_invalid", "Accepted flow has an invalid stage contract")
            stage_status = {item.get("name"): item.get("status") for item in stage_items}
            if (not isinstance(stages, list) or not stages or receipt.get("stages") != stages
                    or receipt.get("flowIdentity") != flow_identity or receipt.get("acceptanceId") != acceptance_id
                    or any(stage_status.get(stage) not in {"passed", "reused"} for stage in stages)
                    or record("acceptance", acceptance_id) != receipt):
                raise EntryError("acceptance_receipt_invalid", "Acceptance receipt is not bound to this flow and its required stages")
            return {**value, "formalAcceptance": True, "acceptanceId": acceptance_id,
                    "receiptDigest": receipt["receiptDigest"]}
    except (sqlite3.Error, ValueError, TypeError, KeyError) as error:
        raise EntryError("state_read_unavailable", "Authoritative workflow state cannot be read", retryable=True) from error


SOURCE_RELEASE_DRIVER_FILES = frozenset({"tools/jenkins/source/closeout.py"})
SOURCE_RELEASE_IDENTITY_FIELDS = frozenset({
    "repositoryId", "sessionId", "requestId", "attemptId", "generation",
    "sourceInputDigest", "coverageDigest", "stageImplementationDigest",
})


def release_need(condition, message):
    if not condition:
        raise EntryError("source_release_unproven", message)


def release_selectors(args):
    release_need(not any(getattr(args, name, None) for name in
                        ("patch", "before_hashes", "after_hashes", "dependency_hashes", "coverage",
                         "coverage_template", "base_head", "baseline_ref", "already_applied", "dependency")),
                 "release-source accepts original request references, not new source inputs")
    return {"sessionId": require_identifier(args.session_id, "sessionId"),
            "requestId": require_identifier(args.request_id, "requestId"),
            "attemptId": require_identifier(args.attempt_id or args.request_id + "-attempt-1", "attemptId"),
            "patchRequestRef": require_identifier(args.patch_request_ref, "patchRequestRef"),
            "releaseRequestId": require_identifier(args.release_request_id, "releaseRequestId")}


def release_database(root):
    database = root / ".jenkins/state/coordination.sqlite3"
    plain_path(database)
    release_need(database.is_file(), "Authoritative coordination State is unavailable")
    connection = sqlite3.connect(database.absolute().as_uri() + "?mode=ro", uri=True, timeout=3)
    connection.execute("PRAGMA query_only=ON")
    connection.execute("BEGIN")
    return connection


def release_row(connection, domain, key):
    row = connection.execute("SELECT version,payload FROM records WHERE domain=? AND key=?", (domain, key)).fetchone()
    release_need(row is not None and type(row[0]) is int and row[0] > 0, f"Missing authoritative {domain}")
    payload = json.loads(row[1])
    release_need(isinstance(payload, dict), f"Malformed authoritative {domain}")
    return {"key": key, "version": row[0], "payload": payload}


def readonly_release_input(root, selectors, *, include_claims=True):
    """Read original authority and CAS metadata in one read-only State snapshot."""
    try:
        with closing(release_database(root)) as connection:
            ref = selectors["patchRequestRef"]
            intent = release_row(connection, "patch_intent", ref)
            prepared = release_row(connection, "patch_preparation", ref)
            result = prepared["payload"].get("result")
            release_need(prepared["payload"].get("status") == "prepared" and isinstance(result, dict)
                         and result.get("status") == "prepared" and result.get("patchRequestRef") == ref,
                         "Original patch has no completed preparation")
            identity = result.get("identity")
            release_need(isinstance(identity, dict) and set(identity) == SOURCE_RELEASE_IDENTITY_FIELDS
                         and type(identity.get("generation")) is int and identity["generation"] > 0,
                         "Original full prepared identity is missing")
            for field in ("repositoryId", "sessionId", "requestId", "attemptId"):
                require_identifier(identity.get(field), field)
            for field in ("sourceInputDigest", "coverageDigest", "stageImplementationDigest"):
                release_need(isinstance(identity.get(field), str) and HEX64.fullmatch(identity[field]),
                             "Original prepared source, coverage, or driver digest is invalid")
            release_need(all(identity[k] == selectors[k] for k in ("sessionId", "requestId", "attemptId"))
                         and selectors["releaseRequestId"] != identity["requestId"]
                         and isinstance(intent["payload"].get("identity"), dict)
                         and all(digest(identity.get(k)) == digest(v) for k, v in intent["payload"]["identity"].items()),
                         "Original request references or patch intent identity differ")
            flow_key = ":".join(identity[k] for k in ("sessionId", "requestId", "attemptId"))
            flow_row = release_row(connection, "workflow", flow_key)
            flow = flow_row["payload"]
            source_root = Path(intent["payload"].get("repositoryRoot", "")).absolute()
            plain_path(source_root)
            release_need(source_root.is_dir() and repository_id(source_root) == identity["repositoryId"]
                         and flow.get("repoRoot") == str(source_root), "Original physical repository binding differs")
            approved = build_root(source_root, result.get("buildRoot"))
            expected = {"repositoryId": identity["repositoryId"], "sourceDigest": identity["sourceInputDigest"],
                        "coverageDigest": identity["coverageDigest"], "driverDigest": identity["stageImplementationDigest"],
                        "buildRoot": approved, "generation": identity["generation"]}
            release_need(digest(flow.get("identity")) == digest(identity) and flow.get("status") == "accepted"
                         and all(digest(flow.get(k)) == digest(v) for k, v in expected.items())
                         and intent["payload"].get("buildRoot") == approved
                         and result.get("sealedInputRef") == identity["sourceInputDigest"],
                         "Only the exact formally accepted original flow can release source claims")
            receipt = flow.get("acceptance")
            stages = flow.get("requiredStages")
            stage_items = flow.get("stages")
            release_need(isinstance(receipt, dict) and isinstance(stages, list) and bool(stages)
                         and all(isinstance(s, str) for s in stages) and len(set(stages)) == len(stages)
                         and isinstance(stage_items, list) and all(isinstance(s, dict) for s in stage_items),
                         "Original canonical acceptance or stage contract is absent")
            original = {k: flow.get(k) for k in ("sourceDigest", "coverageDigest", "driverDigest", "generation", "beforePatch", "dagDigest")}
            acceptance_id = digest({"flow": flow_key, "identity": original, "stages": stages})
            accepted = release_row(connection, "acceptance", acceptance_id)
            stage_status = {s.get("name"): s.get("status") for s in stage_items}
            release_need(receipt.get("schemaVersion") == 1 and receipt.get("kind") == "jenkins.authoritative.acceptance-receipt"
                         and receipt.get("authority") == "jenkins-validation" and receipt.get("status") == "accepted"
                         and all(receipt.get(k) == expected[k] for k in ("sourceDigest", "coverageDigest", "driverDigest"))
                         and receipt.get("receiptDigest") == digest({k: v for k, v in receipt.items() if k != "receiptDigest"})
                         and receipt.get("acceptanceId") == acceptance_id and digest(receipt.get("flowIdentity")) == digest(original)
                         and receipt.get("stages") == stages and digest(accepted["payload"]) == digest(receipt)
                         and all(stage_status.get(s) in {"passed", "reused"} for s in stages),
                         "Original formal acceptance is not bound to canonical State and required stages")
            request = connection.execute("SELECT generation,payload,payload_digest FROM requests WHERE repository_id=? AND session_id=? AND request_id=?",
                                         tuple(identity[k] for k in ("repositoryId", "sessionId", "requestId"))).fetchone()
            release_need(request is not None and type(request[0]) is int and request[0] == identity["generation"],
                         "Original authoritative request generation is missing")
            request_payload = json.loads(request[1])
            release_need(isinstance(request_payload, dict) and digest(request_payload) == request[2]
                         and all(digest(request_payload.get(k)) == digest(v) for k, v in identity.items())
                         and request_payload.get("patchRequestRef") == ref,
                         "Original request payload does not bind this exact preparation")
            delivery_key = ":".join(identity[k] for k in ("repositoryId", "sessionId", "requestId")) + ":flow"
            delivery = release_row(connection, "jenkins_submission", delivery_key)
            value = delivery["payload"]
            dispatch = {k: value.get(k) for k in ("repositoryId", "sessionId", "requestId", "parameters")}
            parameters = value.get("parameters")
            release_need(isinstance(parameters, dict) and value.get("delivery") in {"submitted", "accepted", "completed"}
                         and type(value.get("status")) is int and 200 <= value["status"] < 300
                         and value.get("payloadDigest") == digest(dispatch)
                         and isinstance(value.get("dispatchDigest"), str) and HEX64.fullmatch(value["dispatchDigest"])
                         and all(value.get(k) == identity[k] for k in ("repositoryId", "sessionId", "requestId")),
                         "Original Jenkins delivery lacks its immutable parameter binding")
            bound = {"attemptId": identity["attemptId"], "generation": str(identity["generation"]), "patchRequestRef": ref,
                     "buildRoot": approved, "stageImplementationDigest": identity["stageImplementationDigest"]}
            release_need(set(parameters) == {*bound, "runtimeOperationId"}
                         and all(parameters.get(k) == v for k, v in bound.items())
                         and isinstance(parameters.get("runtimeOperationId"), str) and bool(parameters["runtimeOperationId"]),
                         "Original Jenkins parameters differ from accepted patch identity")
            if flow.get("executionId"):
                execution = release_row(connection, "execution", flow["executionId"])["payload"]
                release_need(execution.get("operationId") == parameters["runtimeOperationId"]
                             and (execution.get("driver") or {}).get("runtimeOperationId") == parameters["runtimeOperationId"]
                             and (execution.get("driver") or {}).get("digest") == identity["stageImplementationDigest"],
                             "Original execution and Jenkins runtime parameters differ")
            sealed_row = release_row(connection, "sealed_input", identity["sourceInputDigest"])
            sealed = sealed_row["payload"]
            owned = flow.get("ownedPaths")
            release_need(isinstance(owned, list) and bool(owned) and all(isinstance(p, str) for p in owned)
                         and owned == sorted({relative_path(source_root, p) for p in owned})
                         and sealed.get("ownedPaths") == owned and sealed.get("repositoryId") == identity["repositoryId"]
                         and sealed.get("repositoryRoot") == str(source_root) and sealed.get("owner") == identity["attemptId"]
                         and sealed.get("sessionId") == identity["sessionId"] and sealed.get("sourceDigest") == identity["sourceInputDigest"]
                         and sealed.get("buildRoot") == approved, "Original accepted source ownership is not exact")
            pins = []
            if include_claims:
                rows = connection.execute("SELECT key,version,payload FROM records WHERE domain='path_claims'").fetchall()
                claims = [{"key": k, "version": v, "payload": json.loads(p)} for k, v, p in rows]
                claims = [r for r in claims if r["payload"].get("repositoryId") == identity["repositoryId"]
                          and r["payload"].get("owner") == identity["attemptId"]]
                release_need({r["key"] for r in claims} == {identity["repositoryId"] + ":" + p for p in owned},
                             "All exact accepted owner claims are required")
                original_claims = sealed.get("claims")
                release_need(isinstance(original_claims, list) and len(original_claims) == len(owned)
                             and all(isinstance(c, dict) for c in original_claims), "Original sealed claim pins are incomplete")
                originals = {c.get("path"): c for c in original_claims}
                release_need(set(originals) == set(owned), "Original sealed claim scope differs")
                for row in claims:
                    q = row["payload"]
                    old = originals.get(q.get("path"), {})
                    release_need(type(row["version"]) is int and row["version"] > 0 and type(old.get("version")) is int
                                 and row["version"] == old["version"] and type(q.get("epoch")) is int and q["epoch"] > 0
                                 and type(old.get("epoch")) is int and q["epoch"] == old["epoch"]
                                 and q.get("owner") == old.get("owner") == identity["attemptId"]
                                 and q.get("repositoryRoot") == str(source_root) and q.get("status") == "applied"
                                 and q.get("beforeHash") == old.get("before_hash") and q.get("afterHash") == q.get("beforeHash"),
                                 "Current claim differs from original accepted sealed CAS binding")
                    pins.append({"key": row["key"], "version": row["version"], "payloadDigest": digest(q)})
            payload = {"identity": identity, "releaseRequestId": selectors["releaseRequestId"],
                       "sourceDigest": identity["sourceInputDigest"], "acceptanceReceiptDigest": receipt["receiptDigest"],
                       "buildRoot": approved, "expectedClaims": sorted(pins, key=lambda p: p["key"])}
            return payload, digest([intent, prepared, flow_row, accepted, delivery, sealed_row]), str(source_root)
    except (sqlite3.Error, ValueError, TypeError, KeyError) as error:
        raise EntryError("state_read_unavailable", "Original accepted source authority cannot be read", retryable=True) from error


def create_release_metadata(path, value):
    """CREATE_NEW only: a partial/unknown prior write is never replaced."""
    plain_path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("xb") as stream:
        stream.write(canonical_json(value) + b"\n")
        stream.flush()
        os.fsync(stream.fileno())


def saved_release_metadata(path):
    plain_path(path)
    raw = path.read_bytes()
    release_need(len(raw) <= 8 * 1024 * 1024, "Saved release metadata exceeds the bounded limit")
    value = json.loads(raw)
    release_need(isinstance(value, dict) and raw == canonical_json(value) + b"\n", "Saved immutable release metadata is not canonical")
    return value


def verify_release_receipt(value, packet):
    payload, op = packet["payload"], packet["operationId"]
    release_need(isinstance(value, dict) and value.get("schemaVersion") == 1
                 and value.get("kind") == "jenkins.source-claim-closeout-receipt" and value.get("status") == "complete"
                 and value.get("operationId") == op and value.get("releaseRequestId") == payload["releaseRequestId"]
                 and value.get("inputDigest") == digest(payload) and digest(value.get("identity")) == digest(payload["identity"])
                 and value.get("sourceDigest") == payload["sourceDigest"]
                 and value.get("acceptanceReceiptDigest") == payload["acceptanceReceiptDigest"]
                 and value.get("acceptedDriverDigest") == payload["identity"]["stageImplementationDigest"]
                 and isinstance(value.get("closeoutIssuerDriverDigest"), str) and HEX64.fullmatch(value["closeoutIssuerDriverDigest"])
                 and value.get("receiptDigest") == digest({k: v for k, v in value.items() if k != "receiptDigest"})
                 and all(value.get(k) is True for k in ("sourceClaimsReleased", "consumerReferencesRetained", "sourceObjectsRetained",
                                                       "artifactReferencesRetained", "acceptanceRetained"))
                 and value.get("sourceBytesWritten") is False, "Closeout response is not the exact authoritative source-preserving receipt")
    return value


def execute_source_release(args, root, driver, *, call):
    selectors = release_selectors(args)
    release_need(SOURCE_RELEASE_DRIVER_FILES.issubset(driver.get("manifestFiles", ())),
                 "Selected sealed CLI does not provide flow/source-release")
    folder = root / ".jenkins/state/requests/outbox" / ("source-release-" + digest([selectors["sessionId"], selectors["releaseRequestId"]]))
    packet_path, input_path = folder / "release-source.json", folder / "flow-source-release.json"
    if packet_path.exists():
        packet = saved_release_metadata(packet_path)
        release_need(packet.get("schemaVersion") == 1 and packet.get("kind") == "jenkins.operator.source-release-input"
                     and digest(packet.get("selectors")) == digest(selectors)
                     and packet.get("payloadDigest") == digest(packet.get("payload")), "Release ID already binds different immutable input")
        payload, _, _ = readonly_release_input(root, selectors, include_claims=False)
        release_need(not args.build_root or build_root(root, args.build_root) == payload["buildRoot"], "Release must preserve the original build root")
        release_need(digest({k: v for k, v in payload.items() if k != "expectedClaims"}) ==
                     digest({k: v for k, v in packet["payload"].items() if k != "expectedClaims"}),
                     "Original accepted identity changed after release issuance")
        release_need(packet.get("operationId") == digest({"kind": "source-claim-closeout", "repositoryId": payload["identity"]["repositoryId"],
                     "sessionId": payload["identity"]["sessionId"], "releaseRequestId": selectors["releaseRequestId"]}), "Saved release operation identity differs")
        if input_path.exists():
            release_need(digest(saved_release_metadata(input_path)) == packet["payloadDigest"], "Saved CLI release input changed")
        with closing(release_database(root)) as connection:
            row = connection.execute("SELECT payload FROM records WHERE domain='source_claim_closeout' AND key=?", (packet["operationId"],)).fetchone()
        if row is not None:
            receipt = verify_release_receipt(json.loads(row[0]), packet)
            return {"status": "complete", "formalAcceptance": True, "releaseRequestId": selectors["releaseRequestId"],
                    "operationId": packet["operationId"], "sourceClaimsReleased": True, "reused": True,
                    "identity": packet["payload"]["identity"], "receipt": receipt, "inputFile": str(input_path)}
        return {"status": "delivery_unknown", "formalAcceptance": True, "sourceClaimsReleased": False,
                "releaseRequestId": selectors["releaseRequestId"], "operationId": packet["operationId"],
                "inputFile": str(input_path), "reasonCode": "source_release_delivery_unknown",
                "nextAction": "repeat release-source with the same references for read-only original operation reconciliation"}
    payload, authority_digest, source_root = readonly_release_input(root, selectors)
    release_need(not args.build_root or build_root(root, args.build_root) == payload["buildRoot"], "Release must preserve the original build root")
    operation_id = digest({"kind": "source-claim-closeout", "repositoryId": payload["identity"]["repositoryId"],
                           "sessionId": payload["identity"]["sessionId"], "releaseRequestId": selectors["releaseRequestId"]})
    packet = {"schemaVersion": 1, "kind": "jenkins.operator.source-release-input", "selectors": selectors,
              "payload": payload, "payloadDigest": digest(payload), "operationId": operation_id,
              "originalAuthorityDigest": authority_digest, "sourceRoot": source_root, "issuerDriverDigest": driver["digest"]}
    try:
        create_release_metadata(packet_path, packet)
    except FileExistsError:
        return execute_source_release(args, root, driver, call=call)
    unknown = {"status": "delivery_unknown", "formalAcceptance": True, "sourceClaimsReleased": False,
               "releaseRequestId": selectors["releaseRequestId"], "operationId": operation_id, "inputFile": str(input_path),
               "nextAction": "repeat release-source with the same references for read-only original operation reconciliation"}
    try:
        create_release_metadata(input_path, payload)
        create_release_metadata(folder / "delivery-intent.json", {"operationId": operation_id, "inputDigest": digest(payload),
                                                                "issuerDriverDigest": driver["digest"], "automaticRetry": False})
    except OSError:
        return {**unknown, "reasonCode": "source_release_input_write_unknown", "cliDispatchAttempted": False}
    try:
        value = call(root, driver, folder, "flow", "source-release", payload, args.timeout)
        receipt = verify_release_receipt(value.get("receipt") if isinstance(value, dict) else None, packet)
        release_need(value.get("status") == "complete", "Selected CLI did not complete original source release")
        create_release_metadata(folder / "response.json", value)
    except (EntryError, OSError) as error:
        reason = error.code if isinstance(error, EntryError) else "source_release_response_write_unknown"
        if isinstance(error, EntryError):
            try:
                create_release_metadata(folder / "response.json", {"status": "delivery_unknown", "reasonCode": reason, "protocol": error.details.get("protocol")})
            except OSError:
                pass  # Original input and once-only dispatch reservation remain authoritative.
        return {**unknown, "reasonCode": reason, "responseFile": str(folder / "response.json"), "cliDispatchAttempted": True}
    return {"status": "complete", "formalAcceptance": True, "sourceClaimsReleased": True,
            "releaseRequestId": selectors["releaseRequestId"], "operationId": operation_id, "identity": payload["identity"],
            "receipt": receipt, "inputFile": str(input_path), "responseFile": str(folder / "response.json"), "reused": bool(value.get("reused"))}



def execute(args, *, call=protocol, driver_loader=selected_driver):
    already_applied = already_applied_mode(args)
    root = Path(args.repository).absolute()
    plain_path(root)
    if not root.is_dir():
        raise EntryError("repository_missing", "Repository root does not exist")
    driver = driver_loader(root)
    if args.action == 'release-source':
        return execute_source_release(args, root, driver, call=call)
    if getattr(args, 'release_request_id', None):
        raise EntryError('release_action_required', '--release-request-id requires release-source')
    identity = {"repositoryId": repository_id(root), "sessionId": require_identifier(args.session_id, "sessionId"),
                "requestId": require_identifier(args.request_id, "requestId"),
                "attemptId": require_identifier(args.attempt_id or args.request_id + "-attempt-1", "attemptId"),
                "generation": 1, "stageImplementationDigest": driver["digest"]}
    folder = root / ".jenkins/state/requests/operator-validation" / digest([identity[k] for k in ("repositoryId", "sessionId", "requestId")])
    lifecycle = folder / "lifecycle.json"
    if args.action == "dry-check":
        if already_applied:
            require_already_applied_driver(driver)
        payload = intent_payload(args, root, identity)
        return {"status": "dry-check", "formalAcceptance": False, "identity": identity,
                "buildRoot": payload["buildRoot"], "patchSha256": hashlib.sha256(payload["unifiedPatch"].encode()).hexdigest(),
                "ownedPaths": sorted(payload["beforeHashes"]), "declaredDependencies": payload["declaredDependencies"],
                "coverage": payload["coverage"], "authorization": "automatic local validation context; exact input gates retained",
                "protocol": ["patch/register", "patch/submit"]}
    prior = read_json(lifecycle, "request lifecycle") if lifecycle.is_file() else None
    identity, folder, prior = registration_continuation(args, root, driver, folder, identity, prior)
    lifecycle = folder / "lifecycle.json"
    if prior and any(prior["identity"][key] != identity[key] for key in identity if key != "stageImplementationDigest"):
        raise EntryError("request_identity_conflict", "Saved identity differs; query with its original attempt and selected driver")
    if prior:
        identity = prior["identity"]
        if args.action == "submit" and args.patch:
            save_json(folder / "intent.json", intent_payload(args, root, identity), immutable=True)
    # An interrupted dispatch is always observed, never dispatched a second time.
    if args.action == "query" or (prior and (prior.get("phase") in {"registering", "registration_unknown", "dispatching", "submitted", "delivery_unknown"}
                      or prior.get("patchRequestRef"))):
        ref = args.patch_request_ref or (prior or {}).get("patchRequestRef")
        if not ref:
            value = call(root, driver, folder, "request", "query", {"identity": identity}, args.timeout)
            return {"status": "pending", "formalAcceptance": False, "identity": identity, "request": value}
        control = {"identity": identity, "patchRequestRef": require_identifier(ref, "patchRequestRef"),
                   "buildRoot": (prior or {}).get("buildRoot") or build_root(root, args.build_root)}
        if args.build_root and control["buildRoot"] != build_root(root, args.build_root):
            raise EntryError("patch_build_root_mismatch", "Query must preserve the saved build root")
        # Inject the protocol dependency for offline tests without importing coordinator State.
        patch = call(root, driver, folder, "patch", "query", control, args.timeout)
        prepared = (patch.get("preparation") or {}).get("payload", {}).get("result", {})
        flow = readonly_flow(root, identity, prepared["identity"], control["buildRoot"]) if prepared.get("identity") else None
        accepted = bool(flow and flow.get("formalAcceptance") is True)
        return {"status": flow.get("status", "pending") if flow else patch.get("status", "pending"), "formalAcceptance": accepted,
                **control, "patch": patch, "flow": flow}
    if already_applied:
        require_already_applied_driver(driver)
    payload = intent_payload(args, root, identity)
    save_json(folder / "intent.json", payload, immutable=True)
    save_json(lifecycle, {"identity": identity, "buildRoot": payload["buildRoot"], "phase": "registering"})
    try:
        registered = call(root, driver, folder, "patch", "register", payload, args.timeout)
    except EntryError as error:
        phase = "registration_unknown" if error.code in {"delivery_unknown", "protocol_delivery_unknown"} else "rejected"
        save_json(lifecycle, {"identity": identity, "buildRoot": payload["buildRoot"], "phase": phase, "reasonCode": error.code})
        raise
    ref = require_identifier(registered.get("patchRequestRef"), "patchRequestRef")
    control = {"identity": identity, "patchRequestRef": ref, "buildRoot": payload["buildRoot"],
               "runtimeOperationId": driver["runtimeOperationId"], "ownedPaths": payload["ownedPaths"]}
    saved = {**control, "phase": "dispatching"}
    save_json(lifecycle, saved)
    try:
        submitted = call(root, driver, folder, "patch", "submit", control, args.timeout)
    except EntryError as error:
        if error.code in {"delivery_unknown", "protocol_delivery_unknown"}:
            save_json(lifecycle, {**saved, "phase": "delivery_unknown"})
            try:
                observed = call(root, driver, folder, "patch", "query", control, args.timeout)
            except EntryError:
                observed = None
            return {"status": "delivery_unknown", "formalAcceptance": False, **control,
                    "reasonCode": error.code, "observed": observed,
                    "nextAction": "query with the same session, request, attempt and build root"}
        save_json(lifecycle, {**saved, "phase": "rejected", "reasonCode": error.code})
        raise
    save_json(lifecycle, {**saved, "phase": "submitted"})
    return {"status": "submitted", "formalAcceptance": False, **control, "submission": submitted,
            "nextAction": "query with the same session, request, attempt and build root"}


def parser():
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("action", nargs="?", default="dry-check", choices=("dry-check", "submit", "query", "release-source"))
    result.add_argument("--repository", "--repo-root", default=str(Path(__file__).resolve().parents[2]))
    result.add_argument("--session-id", "--session", required=True)
    result.add_argument("--request-id", "--request", required=True)
    result.add_argument("--attempt-id", "--attempt")
    result.add_argument("--patch")
    result.add_argument("--before-hashes", "--before-hashes-json", dest="before_hashes")
    result.add_argument("--already-applied", action="store_true",
                        help="Hand off scoped checkout postimages; requires immutable base, after and dependency hashes")
    result.add_argument("--after-hashes", "--after-hashes-json", dest="after_hashes",
                        help="Already-applied SHA-256/null map matching every patch path (JSON or file path)")
    result.add_argument("--dependency-hashes", "--dependency-hashes-json", dest="dependency_hashes",
                        help="Already-applied dependency SHA-256 map; pass '{}' explicitly when empty")
    result.add_argument("--baseline-ref", help="Optional coordinator-issued baseline identity for already-applied mode")
    result.add_argument("--coverage", "--coverage-json", dest="coverage")
    result.add_argument("--coverage-template")
    result.add_argument("--dependency", action="append", default=[])
    result.add_argument("--base-head")
    result.add_argument("--build-root")
    result.add_argument("--patch-request-ref")
    result.add_argument("--release-request-id", help="Independent source-claim release operation; requires original formal acceptance")
    result.add_argument("--timeout", type=float, default=60)
    return result


def main(argv=None):
    args = parser().parse_args(argv)
    try:
        if not 0 < args.timeout <= 300:
            raise EntryError("timeout_invalid", "Timeout must be greater than zero and at most 300 seconds")
        value = execute(args)
        code = 0 if value["status"] != "delivery_unknown" else 3
    except EntryError as error:
        value = {"status": "blocked" if error.retryable else "failed", "formalAcceptance": False,
                 "reasonCode": error.code, "message": str(error), "retryable": error.retryable, **error.details}
        code = 2
    except (OSError, ValueError, KeyError, TypeError, UnicodeError):
        value = {"status": "failed", "formalAcceptance": False, "reasonCode": "invalid_input",
                 "message": "Required input or selected driver cannot be read"}
        code = 2
    print(canonical_json(value).decode("utf-8"))
    return code



def admit_local_validation(root, driver, payload):
    """Use the verified management capsule's local policy before sealed intake."""
    import importlib.util
    entry_path = root / "tools/jenkins-management/entry.py"
    plain_path(entry_path)
    spec = importlib.util.spec_from_file_location("local_validation_management_entry", entry_path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    context = module.capability(root)
    config = context["config"]
    if (config["driverDigest"] != driver["digest"]
            or config["runtimeOperationId"] != driver["runtimeOperationId"]):
        raise EntryError("runtime_binding_changed", "Local validation selected a different management runtime")
    helper = context["module"].source_helper()
    from tools.jenkins.state import State
    state = State(root / ".jenkins/state/coordination.sqlite3")
    try:
        return helper.admit_ordinary_context(state, root, payload,
            principal=helper.local_request_principal(context["manager"]),
            driver_digest=driver["digest"], runtime_operation_id=driver["runtimeOperationId"])
    except Exception as error:
        if hasattr(error, "code"):
            raise EntryError(error.code, str(error), retryable=getattr(error, "retryable", False)) from error
        raise


def registration_continuation(args, root, driver, folder, identity, prior):
    """Continue only a known rejected registration, preserving all predecessor bytes."""
    pointer = folder / "registration-continuation.json"
    if pointer.is_file():
        link = read_json(pointer, "registration continuation")
        old, current = link.get("predecessorIdentity", {}), link.get("identity", {})
        if (any(identity.get(name) != current.get(name) for name in ("repositoryId", "sessionId", "requestId", "generation"))
                or identity.get("attemptId") not in {old.get("attemptId"), current.get("attemptId")}
                or not HEX64.fullmatch(str(link.get("lineageDigest", "")))
                or digest({k: v for k, v in link.items() if k != "lineageDigest"}) != link["lineageDigest"]):
            raise EntryError("request_identity_conflict", "Saved automatic continuation differs from the original request")
        for filename, checksum in link["predecessorFiles"].items():
            path = folder / filename
            plain_path(path)
            if hashlib.sha256(path.read_bytes()).hexdigest() != checksum:
                raise EntryError("request_input_conflict", "Rejected predecessor evidence changed")
        next_folder = folder / "registration-lineage" / link["lineageDigest"]
        plain_path(next_folder)
        lifecycle = next_folder / "lifecycle.json"
        return current, next_folder, read_json(lifecycle, "continued request lifecycle") if lifecycle.is_file() else None
    can_continue = (args.action == "submit" and prior and prior.get("phase") == "rejected"
                    and not prior.get("patchRequestRef")
                    and prior.get("reasonCode") in {"action_not_authorized", "session_not_authorized", "path_not_authorized"})
    if not can_continue:
        return identity, folder, prior
    old = prior["identity"]
    if any(old.get(name) != identity.get(name) for name in identity if name != "stageImplementationDigest"):
        raise EntryError("request_identity_conflict", "Rejected registration belongs to another original request")
    if not args.patch:
        raise EntryError("patch_inputs_missing", "Continuing a rejected registration requires the same scoped patch inputs")
    # Checking immutable predecessor input happens before publishing a continuation.
    save_json(folder / "intent.json", intent_payload(args, root, old), immutable=True)
    predecessor = {name: hashlib.sha256((folder / name).read_bytes()).hexdigest()
                   for name in ("intent.json", "lifecycle.json")}
    suffix = digest({"predecessorFiles": predecessor, "driverDigest": driver["digest"],
                     "runtimeOperationId": driver["runtimeOperationId"]})[:16]
    current = {**old, "stageImplementationDigest": driver["digest"],
               "attemptId": old["attemptId"][:100] + "-local-" + suffix}
    require_identifier(current["attemptId"], "attemptId")
    link = {"schemaVersion": 1, "kind": "known-no-post-registration-continuation",
            "predecessorIdentity": old, "identity": current, "predecessorFiles": predecessor,
            "predecessorPhase": "rejected", "predecessorReasonCode": prior["reasonCode"],
            "runtimeOperationId": driver["runtimeOperationId"], "requestIdPreserved": True,
            "unknownDeliveryRetried": False}
    link["lineageDigest"] = digest(link)
    save_json(pointer, link, immutable=True)
    next_folder = folder / "registration-lineage" / link["lineageDigest"]
    plain_path(next_folder)
    return current, next_folder, None


if __name__ == "__main__":
    raise SystemExit(main())

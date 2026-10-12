"""Bounded maintenance of one explicitly authorized historical Cargo build tree.

Intake pins the original evidence; inventory pins each plain file. GC never
removes a directory recursively, a target sibling, or historical evidence.
Unknown native ownership or a lost deletion acknowledgement remains unknown.
"""
from __future__ import annotations

from contextlib import contextmanager
import ctypes
import hashlib
import json
import ntpath
import os
from pathlib import Path
import time

from ..contracts import JenkinsError, digest, identifier, require_digest, response
from ..deployment.driver import _plain, active_driver_binding
from ..processes.identity import ProcessIdentity, current_identity, identity_matches
from ..resources.paths import canonical_build_root, physical_path_under
from ..source import repository_id
from ..state.locks import process_lock
from . import artifact_protection


CACHE_KIND = "existing-build-cache"
RELATIVE_SUBTREE = artifact_protection.RELATIVE_SUBTREE
CANDIDATE = (r"D:\cargo-targets\zircon-local\engine-audit-20261009-01a12097"
             r"\runtime-check-d-r1\compile\build")
EVIDENCE_PINS = (
    ("runtime-check-d-r1", "06b23071874452c4111e84ae2617e46a97d9557a9559a57e5772e2be2bf5c800",
     "9ba81b12c567a55a3dc04780822424b5b100bcee518fbf36bafad9fddd680c1a", "native"),
    ("runtime-focus-tests-r1", "11963737681ab32384d82ea430637e6f2ccf45e45c0e0cc92348772f2dea06a9",
     "b90155a8ac656ab00cc2ce64e994d73579f1a6c65a97d345c94213d2df9ebe36", "compile"),
    ("runtime-focus-tests-r2", "467af9550e7467afd70f39a22443627b1c99333c066aafce23a481d8ffea9c6f",
     "7433c3d13d84d1dd6a51966335506f97b3b59fc076ffe477234f797dc17d1cfd", "compile"),
)
_IDS = ("repositoryId", "sessionId", "requestId", "attemptId")
_HASHES = ("sourceInputDigest", "coverageDigest", "stageImplementationDigest")
_IDENTITY = (*_IDS, "generation", *_HASHES)
_MAX_FILES, _MAX_BYTES, _MAX_SECONDS = 10000, 5 * 1024**3, 300
_TERMINAL = {"released", "deleted", "terminal", "complete", "completed", "passed", "failed", "cancelled"}
_WRITERS = {"cargo.exe", "rustc.exe", "rustdoc.exe", "link.exe", "lld-link.exe", "cl.exe",
            "clang.exe", "clang-cl.exe", "cmake.exe", "ninja.exe", "msbuild.exe"}
_OWNED_LOCATIONS = {"path", "physicalpath", "preparationroot", "targetdir", "targetdirectory",
                    "builddir", "builddirectory", "phaseoutputroot"}


def _fail(code, message):
    return JenkinsError(code, message, retryable=code in {"cache_writer_active", "cache_ownership_unknown"})


def _identity(payload):
    supplied = payload.get("identity")
    if not isinstance(supplied, dict) or set(supplied) != set(_IDENTITY):
        raise _fail("maintenance_identity_invalid", "Build-cache maintenance requires a full request identity")
    value = dict(supplied)
    for field in _IDS:
        identifier(value[field], field)
    for field in _HASHES:
        require_digest(value[field], field)
    if type(value["generation"]) is not int or value["generation"] < 1:
        raise _fail("maintenance_identity_invalid", "Generation must be a positive integer")
    if any(field in payload and payload[field] != value[field] for field in _IDENTITY):
        raise _fail("identity_conflict", "Control and maintenance identities differ")
    return value


def _scope(value):
    if (not isinstance(value, dict) or not {"candidatePath", "maxFiles", "maxBytes"} <= set(value)
            or set(value) - {"candidatePath", "maxFiles", "maxBytes", "maxSeconds", "relativeSubtree"}):
        raise _fail("cache_scope_invalid", "One exact candidate and explicit inventory/deletion limits are required")
    value = {"maxSeconds": _MAX_SECONDS, **value}
    if value["candidatePath"] != CANDIDATE:
        raise _fail("cache_candidate_rejected", "This operation admits only the named historical build directory")
    if "relativeSubtree" in value and value["relativeSubtree"] != RELATIVE_SUBTREE:
        raise _fail("cache_scope_invalid", "Only the fixed historical incremental subtree may narrow the candidate")
    for field, maximum in (("maxFiles", _MAX_FILES), ("maxBytes", _MAX_BYTES), ("maxSeconds", _MAX_SECONDS)):
        if type(value[field]) is not int or not 0 < value[field] <= maximum:
            raise _fail("cache_scope_invalid", "Cleanup limits exceed the bounded maintenance policy")
    return dict(value)


def _authorize(state, connection, repo_root, payload):
    identity = _identity(payload)
    _plain(repo_root)
    if not repo_root.is_dir() or repo_root.resolve() != repo_root or identity["repositoryId"] != repository_id(repo_root):
        raise _fail("maintenance_repository_mismatch", "Maintenance belongs to another physical repository")
    request = state.get_request(*(identity[field] for field in _IDS[:3]), connection=connection)
    if request is None:
        raise _fail("request_not_found", "Maintenance requires an existing registered request")
    intent = request["payload"]
    if request["payloadDigest"] != digest(intent):
        raise _fail("request_payload_mismatch", "Registered maintenance intent changed")
    if request["generation"] != identity["generation"]:
        raise _fail("request_generation_mismatch", "Maintenance generation changed")
    if any(intent.get(field) != identity[field] for field in _IDENTITY):
        raise _fail("request_identity_mismatch", "Maintenance differs from its registered identity")
    cancellation = state.get("cancellation", ":".join(identity[field] for field in _IDS[:3]), connection=connection)
    if request["status"] in {"cancel_requested", "cancelled", "failed"} or cancellation:
        raise _fail("maintenance_cancelled", "Cancelled maintenance cannot remove cache files")
    if payload.get("cacheKind") != CACHE_KIND or intent.get("cacheKind") != CACHE_KIND:
        raise _fail("cache_scope_invalid", "Registered intent must explicitly name existing-build-cache maintenance")
    scope = _scope(intent.get("cleanupScope"))
    if _scope(payload.get("cleanupScope")) != scope:
        raise _fail("request_payload_mismatch", "Caller cannot change registered cleanup scope")
    paths = intent.get("ownedPaths")
    if not isinstance(paths, list) or not paths:
        raise _fail("cache_scope_invalid", "Registered maintenance requires explicit granted paths")
    if "ownedPaths" in payload and payload["ownedPaths"] != paths:
        raise _fail("request_payload_mismatch", "Caller cannot replace registered authorization paths")
    authorization = state.check_authorization(identity["repositoryId"], identity["sessionId"], "gc", paths,
                                               connection=connection)
    operation = identifier(payload.get("operationId"), "operationId")
    if intent.get("operationId") != operation:
        raise _fail("request_payload_mismatch", "Maintenance operation must match its registered request")
    if (repo_root / ".jenkins/deployment-spec.json").exists() or "runtimeOperationId" in payload:
        binding = active_driver_binding(repo_root)
        if (payload.get("driverDigest") != binding["digest"] or identity["stageImplementationDigest"] != binding["digest"]
                or payload.get("runtimeOperationId") != binding["runtimeOperationId"]):
            raise _fail("driver_context_mismatch", "Maintenance must use the current verified driver and runtime")
    authority = {"identity": identity, "operationId": operation, "cleanupScope": scope,
                 "requestPayloadDigest": request["payloadDigest"], "authorizationSnapshot": authorization["snapshotDigest"]}
    if "reviewedProtection" in payload or "reviewedProtection" in intent:
        if payload.get("reviewedProtection") != intent.get("reviewedProtection"):
            raise _fail("request_payload_mismatch", "Caller cannot change registered provenance review")
        authority["reviewedProtection"] = artifact_protection.normalize_review(
            intent.get("reviewedProtection"), repo_root, str(_selected_path(scope)))
    return authority


def _selected_path(scope):
    return Path(CANDIDATE).joinpath(*scope["relativeSubtree"].split("/")) if "relativeSubtree" in scope else Path(CANDIDATE)


def _inventory_prefix(candidate):
    if Path(candidate) == Path(CANDIDATE):
        return "build"
    if Path(candidate) == _selected_path({"relativeSubtree": RELATIVE_SUBTREE}):
        return "build/" + RELATIVE_SUBTREE
    raise _fail("cache_scope_invalid", "Inventory must bind the original build tree or its exact fixed subtree")


def _physical():
    approved = canonical_build_root(r"D:\cargo-targets")
    candidate = physical_path_under(approved, CANDIDATE, allow_missing=False)
    if not candidate.is_dir():
        raise _fail("cache_candidate_missing", "Registered build directory is absent")
    return approved, candidate


def _read_evidence(approved, path, checksum):
    path = physical_path_under(approved, path, allow_missing=False)
    if not path.is_file() or path.stat().st_size > 32 * 1024**2:
        raise _fail("cache_evidence_invalid", "Historical evidence is missing or exceeds its read bound")
    before = path.stat()
    raw = path.read_bytes()
    after = path.stat()
    if (any(getattr(before, f) != getattr(after, f) for f in ("st_dev", "st_ino", "st_size", "st_mtime_ns", "st_ctime_ns"))
            or hashlib.sha256(raw).hexdigest() != checksum):
        raise _fail("cache_evidence_changed", "Historical request or receipt differs from its pinned bytes")
    try:
        value = json.loads(raw)
    except (ValueError, UnicodeError) as error:
        raise _fail("cache_evidence_invalid", "Historical evidence is not JSON") from error
    if not isinstance(value, dict):
        raise _fail("cache_evidence_invalid", "Historical evidence must be an object")
    return value


def _terminal(native, repo_root):
    if not isinstance(native, dict):
        raise _fail("native_termination_unproven", "A pinned native terminal record is required")
    tree, job = native.get("processTree", {}), native.get("nativeJobEvidence", {})
    if not isinstance(tree, dict) or not isinstance(job, dict):
        raise _fail("native_termination_unproven", "Native process tree and Job evidence must be structured records")
    records = job.get("records", {})
    if (not isinstance(records, dict) or not isinstance(tree.get("pipes", {}), dict)
            or any(not isinstance(records.get(stage, {}), dict)
                   for stage in ("beforeResume", "afterTerminal", "afterClose"))):
        raise _fail("native_termination_unproven", "Native Job accounting and pipe evidence have an unknown shape")
    command = native.get("command")
    pid, birth = native.get("pid"), native.get("creationTime")
    if (native.get("schemaVersion") != 1 or type(pid) is not int or pid <= 0
            or not isinstance(birth, str) or not birth.isdigit() or not birth
            or not isinstance(command, list) or not command or not all(isinstance(v, str) and v for v in command)
            or not ntpath.isabs(command[0]) or native.get("cwd") != str(repo_root)
            or type(native.get("rootExitCode")) is not int or native.get("terminalErrors") != []
            or tree.get("scope") != "retained-windows-job" or tree.get("terminal") is not True
            or tree.get("nativeWaiterJoined") is not True or tree.get("readersFinished") is not True
            or tree.get("pipeEOF") is not True or tree.get("jobHandleRetainedUntilCliExit") is not False
            or any(tree.get("pipes", {}).get(name) != {"eof": True, "closed": True} for name in ("stdout", "stderr"))
            or job.get("access") != "JOB_OBJECT_QUERY" or job.get("errors") != []
            or any(records.get(stage, {}).get("activeProcesses") != 0 for stage in ("afterTerminal", "afterClose"))):
        raise _fail("native_termination_unproven", "Owned Job accounting, tuple, readers and pipes must all be terminal")
    members = records.get("beforeResume", {}).get("members")
    if (records.get("beforeResume", {}).get("activeProcesses") != 1 or not isinstance(members, list) or len(members) != 1
            or members[0] != {"pid": pid, "creationTime": birth, "exeBasename": ntpath.basename(command[0]), "member": True}):
        raise _fail("native_identity_mismatch", "Native proof must identify the exact retained root process")
    expected = ProcessIdentity(pid, birth, command[0])
    if identity_matches(expected):
        raise _fail("cache_writer_active", "An original native writer is still live")
    return {"pid": pid, "creationTime": birth, "executable": command[0], "commandDigest": digest(command),
            "rootExitCode": native["rootExitCode"], "terminalProofDigest": digest(native)}


def _evidence(approved, candidate, repo_root):
    audit = candidate.parent.parent.parent
    target = str(candidate.parent / "target")
    results = []
    for directory, request_hash, receipt_hash, native_field in EVIDENCE_PINS:
        request_path, receipt_path = audit / directory / "request.json", audit / directory / "receipt.json"
        request = _read_evidence(approved, request_path, request_hash)
        receipt = _read_evidence(approved, receipt_path, receipt_hash)
        request_ref = receipt.get("request")
        if native_field == "native":
            linked = request_ref == str(request_path) and receipt.get("requestSha256") == request_hash
            bound = request.get("targetDir") == target
        else:
            linked = isinstance(request_ref, dict) and request_ref.get("path") == str(request_path) and request_ref.get("sha256") == request_hash
            paths = request.get("physicalPaths", {})
            bound = paths.get("target") == target and paths.get("build") == str(candidate)
        if not linked or not bound:
            raise _fail("cache_evidence_binding_mismatch", "Original request/receipt must bind this exact target and build sibling")
        native = receipt.get(native_field)
        proof = _terminal(native, repo_root)
        if native_field == "native" and native["command"] != request.get("command"):
            raise _fail("native_identity_mismatch", "Original request and native command differ")
        results.append({"request": str(request_path), "requestSha256": request_hash,
                        "receipt": str(receipt_path), "receiptSha256": receipt_hash, "native": proof})
    return results


def _native_writers():
    """Fail closed unless a complete Windows process enumeration is available."""
    if os.name != "nt":
        raise _fail("cache_ownership_unknown", "Current native writer observation requires Windows")
    from ctypes import wintypes as wt
    class Entry(ctypes.Structure):
        _fields_ = [("size", wt.DWORD), ("usage", wt.DWORD), ("pid", wt.DWORD), ("heap", ctypes.c_size_t),
                    ("module", wt.DWORD), ("threads", wt.DWORD), ("parent", wt.DWORD), ("priority", wt.LONG),
                    ("flags", wt.DWORD), ("exe", wt.WCHAR * 260)]
    api = ctypes.WinDLL("kernel32", use_last_error=True)
    api.CreateToolhelp32Snapshot.argtypes, api.CreateToolhelp32Snapshot.restype = [wt.DWORD, wt.DWORD], wt.HANDLE
    api.Process32FirstW.argtypes = api.Process32NextW.argtypes = [wt.HANDLE, ctypes.POINTER(Entry)]
    api.CloseHandle.argtypes = [wt.HANDLE]
    handle = api.CreateToolhelp32Snapshot(2, 0)
    if handle in (None, ctypes.c_void_p(-1).value):
        raise _fail("cache_ownership_unknown", "Complete process inventory is unavailable")
    writers = []
    try:
        entry = Entry(); entry.size = ctypes.sizeof(Entry)
        success = api.Process32FirstW(handle, ctypes.byref(entry))
        count = 0
        while success:
            count += 1
            if count > 65536:
                raise _fail("cache_ownership_unknown", "Native process inventory exceeds its bound")
            if entry.exe.casefold() in _WRITERS:
                writers.append(current_identity(int(entry.pid)).to_dict())
            success = api.Process32NextW(handle, ctypes.byref(entry))
        if ctypes.get_last_error() != 18:
            raise _fail("cache_ownership_unknown", "Native process enumeration is incomplete")
    finally:
        api.CloseHandle(handle)
    if writers:
        raise JenkinsError("cache_writer_active", "A live compiler may write the historical cache", retryable=True,
                           details={"writers": writers})
    return {"complete": True, "writers": [], "observedAt": time.time()}


def _paths(value):
    result = []
    if isinstance(value, dict):
        for key, item in value.items():
            if isinstance(item, str) and key.lower() in _OWNED_LOCATIONS and ntpath.isabs(item):
                result.append(item)
            elif isinstance(item, (dict, list)):
                result.extend(_paths(item))
    elif isinstance(value, list):
        for item in value:
            result.extend(_paths(item))
    return result


def _from_previous_boot(value):
    """Exclude a vanished historic process only from the current-writer check.

    A real boot boundary never grants native acceptance or releases its holds.
    The old row is retained unchanged and refs/reservations are checked apart.
    """
    birth = value.get("birthToken")
    executable = value.get("executable")
    if (type(value.get("pid")) is not int or value["pid"] <= 0
            or not isinstance(birth, str) or not birth.isdigit() or int(birth) <= 0
            or not isinstance(executable, str) or not ntpath.isabs(executable)):
        return False
    from ..processes.boot import windows_boot_filetime
    return int(birth) < windows_boot_filetime()


def _quiet_state(state, connection, candidate, *, protection=None):
    selected = ntpath.normcase(str(candidate))
    if protection:
        protected = state.get("artifact_ref", artifact_protection.REF_KEY, connection=connection)
        if protected is None:
            raise _fail("artifact_protection_row_changed", "The pinned historical reference is missing")
        artifact_protection.excludes_reference(state, connection, protected, **protection)
    domains = ("resource_reservation", "pool_writer_hold", "pool_ref", "pool_reference", "build_cache_ref",
               "execution", "native_job", "artifact_ref")
    for domain in domains:
        for row in state.list(domain, connection=connection):
            value = row["payload"]
            if value.get("status") in _TERMINAL:
                continue
            if domain == "native_job" and _from_previous_boot(value):
                continue
            if protection and domain == "artifact_ref" and row["key"] == artifact_protection.REF_KEY:
                continue
            paths = _paths(value)
            for ref, owner_domain in (("preparationKey", "preparation_source"), ("executionId", "execution"),
                                      ("owner", "execution"), ("reservationKey", "resource_reservation"),
                                      ("digest", "artifact_object")):
                linked = state.get(owner_domain, str(value.get(ref, "")), connection=connection)
                if linked:
                    paths.extend(_paths(linked["payload"]))
            if domain == "pool_writer_hold":
                linked = state.get("preparation_source", row["key"], connection=connection)
                if linked:
                    paths.extend(_paths(linked["payload"]))
            # Explicit compiler/preparation namespaces prove unrelated holds.
            specific = [p for p in paths if p.casefold().startswith(("d:\\cargo-targets\\", "e:\\cargo-targets\\", "f:\\cargo-targets\\"))]
            specific = [p for p in specific if len(ntpath.normpath(p).split("\\")) > 2]
            if not specific or any(selected == ntpath.normcase(ntpath.normpath(p))
                                   or selected.startswith(ntpath.normcase(ntpath.normpath(p)).rstrip("\\") + "\\")
                                   or ntpath.normcase(ntpath.normpath(p)).startswith(selected + "\\") for p in specific):
                raise JenkinsError("cache_ownership_unknown", "A live reference, hold or reservation may overlap the candidate",
                                   retryable=True, details={"domain": domain, "key": row["key"]})


@contextmanager
def _storage(approved, candidate):
    from ..pilot.native.windows_storage import WindowsStorage, WindowsPathError
    try:
        storage = WindowsStorage(candidate.parent, expected_device=approved.volume_identity)
        try:
            yield storage
        finally:
            storage.close()
    except WindowsPathError as error:
        raise JenkinsError("cache_alias_rejected", "Candidate violates native path or hard-link protection",
                           details={"candidatePath": str(candidate), "nativeReason": str(error)}) from error


def _file_fact(storage, relative, maximum):
    from ..pilot.native.windows_storage import identity_from_stream
    with storage.open_relative(relative, "rb") as stream:
        before = identity_from_stream(stream)
        if before.size > maximum:
            raise _fail("cache_inventory_limit", "Candidate bytes exceed the registered bound")
        checksum = hashlib.sha256(); size = 0
        while chunk := stream.read(1024 * 1024):
            size += len(chunk)
            if size > maximum:
                raise _fail("cache_inventory_limit", "Candidate bytes exceed the registered bound")
            checksum.update(chunk)
        after = identity_from_stream(stream)
        if not before.same_file_state(after) or size != after.size or after.link_count != 1:
            raise _fail("cache_inventory_changed", "Candidate file changed or has an unowned hardlink")
    return {"path": relative, "sha256": checksum.hexdigest(), "byteLength": size,
            "fileId": after.file_id.hex(), "device": after.device, "mtimeNs": after.mtime_ns,
            "linkCount": after.link_count}, after


def _inventory(storage, candidate, limits):
    from ..pilot.native.windows_storage import WindowsPathError
    files, total = [], 0
    deadline = time.monotonic() + limits["maxSeconds"]
    prefix = _inventory_prefix(candidate)
    relative = prefix
    try:
        for relative in _bounded_files(storage, {**limits, "inventoryPrefix": prefix}, deadline):
            fact, _ = _file_fact(storage, relative, limits["maxBytes"] - total)
            if fact["linkCount"] != 1:
                raise JenkinsError("cache_alias_rejected", "Selected file has a hard-link alias", details={"path": relative})
            files.append(fact); total += fact["byteLength"]
    except WindowsPathError as error:
        raise JenkinsError("cache_alias_rejected", "Selected file violates native path or hard-link protection",
                           details={"path": relative, "nativeReason": str(error)}) from error
    result = {"candidatePath": str(candidate), "files": sorted(files, key=lambda f: f["path"]),
              "fileCount": len(files), "totalBytes": total, "complete": True}
    return {**result, "inventoryDigest": digest(result)}


def _bounded_files(storage, limits, deadline):
    # Enumerate from pinned parent handles as well as hashing from them. This
    # keeps a concurrent junction replacement from redirecting enumeration.
    from ..pilot.native.windows_storage import (
        _directory_entries, _open_relative_handle, _ensure_directory_handle, _close_handle,
        _DIRECTORY_READ_ACCESS, _FILE_SHARE_READ, _FILE_SHARE_WRITE, _FILE_DIRECTORY_FILE,
        _FILE_OPEN_REPARSE_POINT, _FILE_SYNCHRONOUS_IO_NONALERT,
    )
    count, files = 0, 0
    def walk(handle, prefix, depth):
        nonlocal count, files
        if depth > 64:
            raise _fail("cache_inventory_limit", "Candidate directory depth exceeds its bound")
        for name, attributes in _directory_entries(handle):
            count += 1
            if count > limits["maxFiles"] * 3 or time.monotonic() > deadline:
                raise _fail("cache_inventory_limit", "Candidate inventory exceeds its entry or time bound")
            relative = prefix + "/" + name
            if attributes & 0x400:
                raise _fail("cache_reparse_rejected", "Candidate inventory contains a reparse point")
            if attributes & 0x10:
                child = _open_relative_handle(handle, name, access=_DIRECTORY_READ_ACCESS,
                    share=_FILE_SHARE_READ | _FILE_SHARE_WRITE,
                    options=_FILE_DIRECTORY_FILE | _FILE_OPEN_REPARSE_POINT | _FILE_SYNCHRONOUS_IO_NONALERT)
                try:
                    _ensure_directory_handle(child)
                    yield from walk(child, relative, depth + 1)
                finally:
                    _close_handle(child)
            else:
                files += 1
                if files > limits["maxFiles"]:
                    raise _fail("cache_inventory_limit", "Candidate inventory exceeds its file bound")
                yield relative
    with storage._lock:
        storage.verify_root()
        prefix = limits.get("inventoryPrefix", "build")
        if prefix not in {"build", "build/" + RELATIVE_SUBTREE}:
            raise _fail("cache_scope_invalid", "Native inventory must use an exact supported candidate prefix")
        _, handles = storage._directory_relative(prefix)
        try:
            yield from walk(handles[-1], prefix, 0)
        finally:
            for handle in reversed(handles):
                _close_handle(handle)


def _public(authority, phase, key, value):
    summary = {field: value[field] for field in (
        "status", "fileCount", "totalBytes", "reclaimedBytes", "unreclaimedBytes", "removedFileCount",
        "historyPreserved", "pendingDeletionUnacknowledged") if field in value}
    summary.update({"phase": phase, "operationId": authority["operationId"],
                    "candidateRef": key, "candidatePath": str(_selected_path(authority["cleanupScope"])),
                    "baseCandidatePath": CANDIDATE, "relativeSubtree": authority["cleanupScope"].get("relativeSubtree"),
                    "JenkinsAcceptance": False, "milestoneAcceptance": False})
    if phase in {"gc", "reconcile"}:
        pending = value.get("pendingFile")
        summary["pendingFile"] = {"path": pending["path"]} if isinstance(pending, dict) and "path" in pending else pending
    if "unknown" in value:
        summary["unknownFileCount"] = len(value["unknown"])
    incomplete = phase in {"gc", "reconcile"} and value.get("status") in {"partial", "unknown", "running"}
    return response("blocked" if incomplete else "waiting" if value.get("status") == "yielded" else "accepted",
                    operation_id=authority["operationId"],
                    reason_code=("cache_gc_partial" if value.get("status") == "partial" else "cache_deletion_unknown") if incomplete else None,
                    inventoryDigest=value.get("inventoryDigest"), result=summary)


def _acknowledged(state, operation, inventory, *, storage=None, details=False):
    expected = {item["path"]: item for item in inventory["files"]}
    size, count, removed, pending = 0, 0, set(), []
    for row in state.list("maintenance_cache_gc_file"):
        if not isinstance(row, dict) or not isinstance(row.get("payload"), dict):
            raise _fail("cache_deletion_unknown", "Deletion ledger payload must be a structured record")
        value = row["payload"]
        if not isinstance(value.get("operationId"), str):
            raise _fail("cache_deletion_unknown", "Deletion ledger operation identity is unknown")
        if value.get("operationId") != operation:
            continue
        item = value.get("file")
        if (not isinstance(item, dict) or not isinstance(item.get("path"), str)
                or not isinstance(value.get("status"), str) or not isinstance(row.get("key"), str)
                or type(row.get("version")) is not int):
            raise _fail("cache_deletion_unknown", "Deletion ledger file, status or row identity has an unknown type")
        if (item != expected.get(item["path"]) or value.get("inventoryDigest") != inventory["inventoryDigest"]
                or row["key"] != digest([operation, item.get("path")])
                or value.get("status") not in {"removed", "pending"}):
            raise _fail("cache_deletion_unknown", "A deletion acknowledgement differs from the registered file")
        if value["status"] == "pending":
            if row["version"] != 1 or set(value) != {"operationId", "inventoryDigest", "file", "status"}:
                raise _fail("cache_deletion_unknown", "Pending deletion ledger has an unknown shape or version")
            pending.append(item)
            continue
        if (row["version"] != 2 or set(value) != {"operationId", "inventoryDigest", "file", "status", "removedBytes", "physicalNameAbsent"}
                or type(value.get("removedBytes")) is not int
                or value.get("removedBytes") != item.get("byteLength") or value.get("physicalNameAbsent") is not True
                or storage is not None and storage.exists(item["path"])):
            raise _fail("cache_deletion_unknown", "An acknowledged deletion reappeared or has invalid actual bytes")
        size += value["removedBytes"]; count += 1; removed.add(item["path"])
    return (size, count, removed, pending) if details else (size, count)


def _validate_gc_result(value, authority, key, inventory, size, count, removed, pending):
    """A persisted terminal status alone never proves a completed deletion."""
    base = {"operationId", "authorityDigest", "candidateRef", "inventoryDigest", "status", "removedFileCount",
            "reclaimedBytes", "pendingFile", "unknown", "historyPreserved", "JenkinsAcceptance", "milestoneAcceptance"}
    status = value.get("status") if isinstance(value, dict) else None
    if not isinstance(status, str) or status not in {"running", "yielded", "complete", "partial", "unknown"}:
        raise _fail("cache_deletion_unknown", "Persisted GC status has an unknown shape")
    fields = base if status == "running" else base | {"unreclaimedBytes", "resultDigest"}
    if (set(value) != fields or value.get("operationId") != authority["operationId"]
            or value.get("authorityDigest") != digest(authority) or value.get("candidateRef") != key
            or value.get("inventoryDigest") != inventory["inventoryDigest"]
            or type(value.get("reclaimedBytes")) is not int or value["reclaimedBytes"] != size
            or type(value.get("removedFileCount")) is not int or value["removedFileCount"] != count
            or value.get("historyPreserved") is not True or value.get("JenkinsAcceptance") is not False
            or value.get("milestoneAcceptance") is not False or not isinstance(value.get("unknown"), list)
            or size > inventory["totalBytes"] or count > inventory["fileCount"]):
        raise _fail("cache_deletion_unknown", "Persisted GC result differs from its exact authority, schema or positive acknowledgements")
    if status != "running":
        try:
            require_digest(value["resultDigest"], "GC result digest")
            valid_digest = value["resultDigest"] == digest({k: v for k, v in value.items() if k != "resultDigest"})
        except (JenkinsError, TypeError, ValueError) as error:
            raise _fail("cache_deletion_unknown", "Persisted GC result digest is invalid") from error
        if (not valid_digest or type(value["unreclaimedBytes"]) is not int
                or value["unreclaimedBytes"] != inventory["totalBytes"] - size):
            raise _fail("cache_deletion_unknown", "Persisted GC result digest or actual byte accounting changed")
    selected = value["pendingFile"]
    expected = {item["path"]: item for item in inventory["files"]}
    if (selected is not None and (not isinstance(selected, dict) or not isinstance(selected.get("path"), str)
            or selected != expected.get(selected["path"]) or selected["path"] in removed)):
        raise _fail("cache_deletion_unknown", "Persisted pending deletion does not identify a remaining registered file")
    if len(pending) > 1 or pending and (selected is None or pending != [selected]):
        raise _fail("cache_deletion_unknown", "Persisted pending deletion differs from its durable ledger")
    unknown = value["unknown"]
    if status in {"complete", "yielded", "running"}:
        if unknown or status != "running" and (selected is not None or pending):
            raise _fail("cache_deletion_unknown", "Completed or yielded GC cannot contain unacknowledged or unknown deletion evidence")
        if status == "complete" and (count != inventory["fileCount"] or size != inventory["totalBytes"]):
            raise _fail("cache_deletion_unknown", "Completion requires positive acknowledgements for the entire inventory")
        if status == "yielded" and count >= inventory["fileCount"]:
            raise _fail("cache_deletion_unknown", "A yielded GC must retain registered work for its next explicit chunk")
    else:
        if (selected is None or len(unknown) != 1 or not isinstance(unknown[0], dict)
                or set(unknown[0]) != {"path", "reasonCode"} or unknown[0].get("path") != selected["path"]
                or not isinstance(unknown[0].get("reasonCode"), str) or not 0 < len(unknown[0]["reasonCode"]) <= 128
                or (status == "partial") != (count > 0)):
            raise _fail("cache_deletion_unknown", "Incomplete GC requires bounded unknown evidence and actual acknowledged progress")


def _remaining_inventory(storage, candidate, limits, inventory, removed):
    actual = _inventory(storage, candidate, limits)
    expected = [item for item in inventory["files"] if item["path"] not in removed]
    if actual["files"] != expected or actual["totalBytes"] != sum(item["byteLength"] for item in expected):
        raise _fail("cache_inventory_changed", "Complete remaining candidate inventory changed between GC chunks")
    return expected


def handle(phase, payload, state, repo_root, **_):
    """Called only by the existing maintenance/cache-maintenance public route."""
    if phase not in {"intake", "inventory", "gc", "reconcile"} or payload.get("phase") != phase:
        raise _fail("cache_phase_invalid", "Maintenance requires an explicit supported phase")
    repo_root = Path(repo_root).absolute()
    with state.transaction() as connection:
        authority = _authorize(state, connection, repo_root, payload)
    approved, base_candidate = _physical()
    candidate = physical_path_under(approved, _selected_path(authority["cleanupScope"]), allow_missing=False)
    if not candidate.is_dir():
        raise _fail("cache_candidate_missing", "Selected historical build subtree is absent")
    key = digest({"candidatePath": str(candidate)})
    # OS ownership serializes maintenance processes; BEGIN IMMEDIATE also
    # fences managed reservation/ref changes during each physical deletion.
    base_key = digest({"candidatePath": CANDIDATE})
    lock_path = physical_path_under(approved, approved.namespace() / "maintenance" / (base_key + ".lock"))
    with process_lock(lock_path, reason="cache_maintenance_busy"):
        evidence = _evidence(approved, base_candidate, repo_root)
        inv_key = digest(authority)
        old_inventory = state.get("maintenance_cache_inventory", inv_key)
        review = authority.get("reviewedProtection")
        with _storage(approved, base_candidate) as storage:
            observed = None
            # A missing legacy object's location alone cannot prove exclusion:
            # intake first hashes the entire candidate under native parent pins.
            if phase == "intake" and review and not old_inventory:
                _native_writers()
                observed = _inventory(storage, candidate, authority["cleanupScope"])
            inventory = old_inventory["payload"] if old_inventory else observed
            protection = None
            if review:
                if inventory is None:
                    raise _fail("cache_inventory_missing", "Reviewed protection requires a complete managed candidate inventory")
                protection = {"repo_root": repo_root, "approved": approved, "candidate": candidate,
                              "authority": authority, "review": review, "inventory": inventory,
                              "index_ref": artifact_protection._key(authority, review)}
            with state.transaction() as connection:
                checked = _authorize(state, connection, repo_root, payload)
                if checked != authority:
                    raise _fail("request_payload_mismatch", "Maintenance authority changed before claim")
                old = state.get("maintenance_build_cache", key, connection=connection)
                pinned = {**authority, "candidatePath": str(candidate), "buildRoot": str(approved.path),
                          "volumeIdentity": approved.volume_identity, "evidence": evidence}
                if old and old["payload"] != pinned:
                    raise _fail("cache_owner_conflict", "The candidate has a different immutable maintenance owner")
                if not old and phase != "intake":
                    raise _fail("cache_intake_missing", "Existing cache must be explicitly registered before inventory or deletion")
                if review and phase == "intake":
                    artifact_protection.register_index(state, connection, repo_root, approved, candidate, authority, review, inventory)
                _quiet_state(state, connection, candidate, protection=protection)
                _native_writers()
                if not old:
                    state.put("maintenance_build_cache", key, pinned, expected_version=0, connection=connection)
                if observed is not None and not old_inventory:
                    state.put("maintenance_cache_inventory", inv_key, observed, expected_version=0, connection=connection)
            if phase == "intake":
                return _public(authority, phase, key, {"status": "registered",
                                                      "inventoryDigest": inventory.get("inventoryDigest") if inventory else None})
            if phase == "inventory":
                observed = _inventory(storage, candidate, authority["cleanupScope"])
                with state.transaction() as connection:
                    _authorize(state, connection, repo_root, payload)
                    if review:
                        protection = {**protection, "inventory": observed}
                    _quiet_state(state, connection, candidate, protection=protection)
                    _native_writers()
                    if old_inventory and old_inventory["payload"] != observed:
                        raise _fail("cache_inventory_changed", "Registered inventory is immutable; candidate has changed")
                    if not old_inventory:
                        state.put("maintenance_cache_inventory", inv_key, observed, expected_version=0, connection=connection)
                return _public(authority, phase, key, {**observed, "status": "inventoried"})
            if not old_inventory:
                raise _fail("cache_inventory_missing", "Deletion requires a complete registered inventory")
            inventory = old_inventory["payload"]
            if payload.get("inventoryDigest") != inventory["inventoryDigest"]:
                raise _fail("cache_inventory_mismatch", "Caller must pin the exact registered inventory")
            operation = authority["operationId"]
            old_gc = state.get("maintenance_cache_gc", operation)
            if old_gc:
                result = old_gc["payload"]
                size, count, removed, pending = _acknowledged(state, operation, inventory, storage=storage, details=True)
                _validate_gc_result(result, authority, key, inventory, size, count, removed, pending)
                if result.get("pendingFile") or pending:
                    result = {**result, "status": "unknown", "pendingDeletionUnacknowledged": True}
                if result["status"] in {"partial", "unknown", "running"}:
                    return _public(authority, phase, key, result)
                remaining = _remaining_inventory(storage, candidate, authority["cleanupScope"], inventory, removed)
                if result["status"] == "complete" and remaining:
                    raise _fail("cache_deletion_unknown", "Completion requires a positive acknowledgement for every inventoried file")
                if result["status"] == "complete" or phase == "reconcile":
                    return _public(authority, phase, key, result)
                if result["status"] != "yielded":
                    raise _fail("cache_deletion_unknown", "GC phase does not permit a new deletion chunk")
                result = {k: v for k, v in result.items() if k not in {"resultDigest", "unreclaimedBytes"}}
                result = {**result, "status": "running"}
                state.put("maintenance_cache_gc", operation, result)
            elif phase == "reconcile":
                return _public(authority, phase, key, {"status": "not_started", "reclaimedBytes": 0,
                                                      "inventoryDigest": inventory["inventoryDigest"]})
            else:
                remaining = _remaining_inventory(storage, candidate, authority["cleanupScope"], inventory, set())
                result = {"operationId": operation, "authorityDigest": digest(authority), "candidateRef": key,
                          "inventoryDigest": inventory["inventoryDigest"], "status": "running", "removedFileCount": 0,
                          "reclaimedBytes": 0, "pendingFile": None, "unknown": [], "historyPreserved": True,
                          "JenkinsAcceptance": False, "milestoneAcceptance": False}
                state.put("maintenance_cache_gc", operation, result, expected_version=0)
            deadline = time.monotonic() + authority["cleanupScope"]["maxSeconds"]
            for item in remaining:
                if time.monotonic() > deadline:
                    result = {**result, "status": "yielded", "pendingFile": None}
                    break
                try:
                    # Durable intent precedes unlink. A crash after unlink and
                    # before acknowledgement is never credited during reconcile.
                    result = {**result, "pendingFile": item}
                    file_key = digest([operation, item["path"]])
                    intent = {"operationId": operation, "inventoryDigest": inventory["inventoryDigest"],
                              "file": item, "status": "pending"}
                    with state.transaction() as connection:
                        state.put("maintenance_cache_gc_file", file_key, intent, expected_version=0, connection=connection)
                        state.put("maintenance_cache_gc", operation, result, connection=connection)
                    with state.transaction() as connection:
                        _authorize(state, connection, repo_root, payload)
                        _quiet_state(state, connection, candidate, protection=protection)
                        _native_writers()
                        physical_path_under(approved, candidate, allow_missing=False)
                        fact, identity = _file_fact(storage, item["path"], item["byteLength"])
                        if fact != item or fact["linkCount"] != 1:
                            raise _fail("cache_inventory_changed", "File differs from registered content before deletion")
                        if review and fact["sha256"] == artifact_protection.BLOB_DIGEST:
                            raise _fail("artifact_protection_content_overlap", "Protected blob moved into a file selected for deletion")
                        storage.delete(item["path"], expected_identity=identity)
                        if storage.exists(item["path"]):
                            raise _fail("cache_deletion_unproven", "Deleted file still has a physical name")
                        state.put("maintenance_cache_gc_file", file_key,
                                  {**intent, "status": "removed", "removedBytes": fact["byteLength"],
                                   "physicalNameAbsent": True}, connection=connection)
                        result = {**result, "removedFileCount": result["removedFileCount"] + 1, "pendingFile": None,
                                  "reclaimedBytes": result["reclaimedBytes"] + fact["byteLength"]}
                        state.put("maintenance_cache_gc", operation, result, connection=connection)
                except (OSError, ValueError, JenkinsError) as error:
                    result = {**result, "status": "partial" if result["removedFileCount"] else "unknown",
                              "unknown": [*result["unknown"], {"path": item["path"],
                                           "reasonCode": getattr(error, "code", type(error).__name__)}]}
                    break
            else:
                result = {**result, "status": "complete"}
            size, count = _acknowledged(state, operation, inventory, storage=storage)
            result = {**result, "reclaimedBytes": size, "removedFileCount": count,
                      "unreclaimedBytes": inventory["totalBytes"] - size}
            result["resultDigest"] = digest(result)
            state.put("maintenance_cache_gc", operation, result)
            return _public(authority, phase, key, result)

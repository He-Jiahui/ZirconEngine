"""Terminal task cache assessment and bounded owned scratch reclamation.

Preparation inputs, compiler outputs, warm caches and historical evidence are
retained. Only a new execution's registered, isolated TEMP generation is
eligible, after its enclosing Job and every phase have terminal EOF proof.
"""
from __future__ import annotations

from contextlib import contextmanager
import hashlib
import math
import ntpath
from pathlib import Path
import time

from ..contracts import JenkinsError, digest, identifier, require_digest, response
from ..resources import Capacity, ResourceManager, canonical_build_root, physical_path_under
from ..state.locks import process_lock
from .planning import get_recipe_plan

_IDS = ("repositoryId", "sessionId", "requestId", "attemptId")
_HASHES = ("sourceInputDigest", "coverageDigest", "stageImplementationDigest")
_IDENTITY = (*_IDS, "generation", *_HASHES)
_TERMINAL = {"accepted", "failed", "cancelled", "rejected"}
_LIMITS = {"maxFiles": 64, "maxBytes": 256 * 1024**2, "maxSeconds": 10}
_TEMP = ("TEMP", "TMP", "TMPDIR")


def _fail(code, message, *, retryable=False):
    return JenkinsError(code, message, retryable=retryable)


def _same(a, b):
    return isinstance(a, str) and isinstance(b, str) and a.replace("/", "\\").casefold() == b.replace("/", "\\").casefold()


def _overlap(a, b):
    if not isinstance(a, str) or not isinstance(b, str):
        return False
    a, b = (ntpath.normcase(ntpath.normpath(value.replace("/", "\\"))).rstrip("\\") for value in (a, b))
    return a == b or a.startswith(b + "\\") or b.startswith(a + "\\")


def _directory_fact(identity):
    return {"device": identity.device, "fileId": identity.file_id.hex(), "creationTime": identity.creation_time}


def _scratch_contract(state, execution_id):
    from .execution import _execution_identity
    execution = state.get("execution", execution_id)
    key = _execution_identity(execution, execution_id)
    value = execution["payload"]
    plan = get_recipe_plan(state, value["recipeRef"])
    recipe = plan["recipe"]
    if not str(recipe.get("kind", "")).startswith("cargo") or not recipe.get("tempRoot"):
        return None
    approved = canonical_build_root(value["buildRoot"])
    prep = require_digest(value["preparationKey"], "preparationKey")
    base = approved.namespace() / "preparations" / prep / "tmp"
    if (not _same(recipe.get("tempRoot"), str(base)) or recipe.get("preparationKey") != prep
            or not _same(recipe.get("buildRoot"), str(approved.path))
            or plan.get("sourceRef") != value.get("sourceDigest")
            or plan.get("driver") != value.get("driver")
            or str(plan.get("generation")) != str(value.get("generation"))
            or any(not _same((recipe.get("executionEnvironment") or {}).get(name), str(base)) for name in _TEMP)):
        raise _fail("task_scratch_contract_invalid", "Scratch must derive from the immutable recipe TEMP contract")
    directory = physical_path_under(approved, base / execution_id)
    contract = {"executionId": execution_id, "executionKeyDigest": key, "recipeRef": value["recipeRef"],
                "preparationKey": prep, "buildRoot": str(approved.path), "volumeIdentity": approved.volume_identity,
                "generation": str(value["generation"]), "driverDigest": value["driverDigest"],
                "runtimeOperationId": value["operationId"], "templateTempRoot": str(base),
                "path": str(directory), "effectiveEnvironment": {name: str(directory) for name in _TEMP}}
    contract["scratchGeneration"] = digest(contract)
    return approved, directory, contract


def register_execution_scratch(state, execution_id):
    """Register intent before creating a new isolated execution TEMP directory."""
    binding = _scratch_contract(state, execution_id)
    if binding is None:
        return None
    approved, directory, contract = binding
    from .pools import _hold
    _hold(state, execution_id, contract["preparationKey"])
    old = state.get("execution_scratch", execution_id)
    if old:
        if any(old["payload"].get(key) != value for key, value in contract.items()) or old["payload"].get("status") != "owned":
            raise _fail("task_scratch_ownership_unknown", "An existing scratch generation cannot be adopted")
        return require_execution_scratch(state, execution_id)
    if directory.exists():
        raise _fail("task_scratch_ownership_unknown", "A preexisting scratch directory is protected")
    state.put("execution_scratch", execution_id, {**contract, "status": "creating"}, expected_version=0)
    from ..pilot.native.windows_storage import WindowsStorage
    # Opening with create=True would request ancestor write access outside the
    # zircon-local grant. Pin that existing writable namespace and create only
    # relative descendants through it.
    granted = physical_path_under(approved, approved.path / "zircon-local", allow_missing=False)
    relative = directory.relative_to(granted).as_posix()
    with WindowsStorage(granted, expected_device=approved.volume_identity) as storage:
        identity = storage.create_directory(relative)
    old = state.get("execution_scratch", execution_id)
    state.put("execution_scratch", execution_id, {**contract, "status": "owned", "directoryIdentity": _directory_fact(identity)},
              expected_version=old["version"])
    return require_execution_scratch(state, execution_id)


def require_execution_scratch(state, execution_id):
    binding = _scratch_contract(state, execution_id)
    if binding is None:
        return None
    approved, directory, contract = binding
    row = state.get("execution_scratch", execution_id)
    if (not row or row["payload"].get("status") != "owned"
            or any(row["payload"].get(key) != value for key, value in contract.items())):
        raise _fail("task_scratch_ownership_unknown", "Execution TEMP requires its prelaunch registered generation")
    physical_path_under(approved, directory, allow_missing=False)
    from ..pilot.native.windows_storage import WindowsStorage
    with WindowsStorage(directory, expected_device=approved.volume_identity, readonly_root=True) as storage:
        if _directory_fact(storage.root_identity) != row["payload"].get("directoryIdentity"):
            raise _fail("task_scratch_generation_changed", "The scratch directory physical generation changed")
    return row["payload"]


def _policy(state):
    row = state.get("resource_policy", "default")
    if not row:
        raise _fail("resource_policy_missing", "Post-task assessment requires trusted resource policy")
    policy = row["payload"]
    cache = policy.get("postTaskCache")
    if not isinstance(cache, dict) or cache.get("enabled") is not True or cache.get("everyTerminalTask") is not True:
        raise _fail("post_task_cache_disabled", "Automatic post-task cache policy is not enabled")
    limits = {key: cache.get(key) for key in _LIMITS}
    if (any(type(value) is not int or not 0 < value <= _LIMITS[key] for key, value in limits.items())
            or type(cache.get("minFreeBytes")) is not int or cache["minFreeBytes"] < policy.get("diskReserveBytes", 35 * 1024**3)
            or cache.get("preserveWarmPreparation") is not True or cache.get("preserveAcceptedReferences") is not True):
        raise _fail("post_task_cache_policy_invalid", "Cache maintenance requires bounded limits and retained warm/accepted data")
    return row, cache, limits


def _authority(payload, state):
    supplied = payload.get("identity")
    if not isinstance(supplied, dict) or not set(_IDENTITY) <= set(supplied):
        raise _fail("post_task_identity_invalid", "Post-task maintenance requires the original full request identity")
    for field in _IDS:
        identifier(supplied[field], field)
    for field in _HASHES:
        require_digest(supplied[field], field)
    if type(supplied["generation"]) is not int or supplied["generation"] < 1:
        raise _fail("post_task_identity_invalid", "Generation must be a positive integer")
    if any(field in payload and payload[field] != supplied[field] for field in _IDENTITY):
        raise _fail("identity_conflict", "Control and task identities differ")
    if any(field in payload for field in ("candidatePath", "candidateRef", "cleanupScope", "scratchPath")):
        raise _fail("post_task_scope_untrusted", "Cache candidates are derived exclusively from State")
    request = state.get_request(*(supplied[field] for field in _IDS[:3]))
    if not request or request["payloadDigest"] != digest(request["payload"]):
        raise _fail("request_not_found", "The original immutable request is required")
    intent = request["payload"]
    if (request["generation"] != supplied["generation"]
            or any(intent.get(field) != supplied[field] for field in (*_IDS, "generation", "coverageDigest"))
            or (intent.get("sealedInputRef") or intent.get("sourceInputDigest") or intent.get("sourceDigest")) != supplied["sourceInputDigest"]
            or (intent.get("driverDigest") or intent.get("stageImplementationDigest")) != supplied["stageImplementationDigest"]):
        raise _fail("request_identity_mismatch", "Post-task identity differs from the exact registered attempt")
    flow_id = ":".join(supplied[field] for field in _IDS[1:])
    workflow = state.get("workflow", flow_id)
    flow = workflow["payload"] if workflow else {}
    if (flow.get("identity") != supplied or flow.get("repositoryId") != supplied["repositoryId"]
            or flow.get("sourceDigest") != supplied["sourceInputDigest"] or flow.get("coverageDigest") != supplied["coverageDigest"]
            or flow.get("driverDigest") != supplied["stageImplementationDigest"] or flow.get("status") not in _TERMINAL):
        raise _fail("post_task_workflow_pending", "The exact workflow must be terminal before cache assessment", retryable=True)
    approved = canonical_build_root(flow.get("buildRoot"))
    if not _same(intent.get("buildRoot"), str(approved.path)) or ("buildRoot" in payload and not _same(payload["buildRoot"], str(approved.path))):
        raise _fail("post_task_root_mismatch", "Post-task maintenance retains its authoritative task build root")
    authorization = state.check_authorization(supplied["repositoryId"], supplied["sessionId"], "validation", flow.get("ownedPaths", []))
    if intent.get("authorizationSnapshot") and intent["authorizationSnapshot"] != authorization["snapshotDigest"]:
        raise _fail("request_authorization_mismatch", "The original task authorization changed")
    operation = digest({"kind": "post-task-cache", "identity": supplied, "workflowId": flow_id})
    if payload.get("operationId") is not None and payload["operationId"] != operation:
        raise _fail("post_task_operation_mismatch", "Maintenance operation must derive from its original task")
    execution = state.get("execution", flow.get("executionId")) if flow.get("recipeRef") else None
    if flow.get("recipeRef"):
        from .execution import _execution_identity
        key = _execution_identity(execution, flow["executionId"])
        ep = execution["payload"]
        binding = state.get("execution_binding", digest({"consumer": flow_id, "executionKey": key}))
        consumer = state.get("execution_consumer", f"{flow['executionId']}:{flow_id}")
        if (ep.get("recipeRef") != flow.get("recipeRef") or ep.get("sourceDigest") != flow["sourceDigest"]
                or ep.get("coverageDigest") != flow["coverageDigest"] or ep.get("driverDigest") != flow["driverDigest"]
                or str(ep.get("generation")) != str(supplied["generation"])
                or not _same(ep.get("buildRoot"), str(approved.path)) or not binding or not consumer
                or binding["payload"] != {"executionId": flow["executionId"], "consumerId": flow_id}
                or consumer["payload"].get("executionId") != flow["executionId"]
                or consumer["payload"].get("consumerId") != flow_id
                or not isinstance(ep.get("driver"), dict) or ep["driver"].get("digest") != ep.get("driverDigest")
                or ep["driver"].get("runtimeOperationId") != ep.get("operationId")
                or intent.get("runtimeOperationId") != ep.get("operationId")
                or str(ep["driver"].get("generation")) != str(supplied["generation"])):
            raise _fail("post_task_execution_mismatch", "Maintenance lacks the exact immutable execution consumer binding")
        if any(field in payload and payload[field] != expected for field, expected in
               {"driverDigest": ep["driverDigest"], "runtimeOperationId": ep["operationId"]}.items()):
            raise _fail("post_task_execution_mismatch", "Maintenance differs from the pinned task driver/runtime")
    return {"identity": supplied, "operationId": operation, "workflowId": flow_id,
            "requestPayloadDigest": request["payloadDigest"], "authorizationSnapshot": authorization["snapshotDigest"],
            "workflowStatus": flow["status"], "executionId": execution["key"] if execution else None,
            "buildRoot": str(approved.path), "ownedPaths": flow.get("ownedPaths", [])}, approved, flow, execution


def _native_evidence(state, execution):
    if execution is None:
        return {"status": "light", "nativeJobIds": [], "cleanupAllowed": False}
    eid, value = execution["key"], execution["payload"]
    jobs = [row for row in state.list("native_job") if row["payload"].get("executionId") == eid or row["payload"].get("owner") == eid]
    launch, host = state.get("execution_launch", eid), state.get("execution_host", eid)
    if not jobs and not host and (not launch or (launch["payload"].get("status") == "cancelled" and launch["payload"].get("cancelledBeforeClaim") is True)):
        return {"status": "unlaunched" if value.get("status") in {"cancelled", "failed"} else "pending",
                "nativeJobIds": [], "cleanupAllowed": value.get("status") in {"cancelled", "failed"}}
    guard = next((row for row in jobs if row["payload"].get("role") == "execution_host_guard"), None)
    from ..processes.guards import guard_phases_complete
    complete = value.get("status") in {"passed", "failed", "cancelled"} and guard is not None
    for row in jobs:
        job = row["payload"]
        proof = job.get("completeProof")
        identity = proof.get("identity") if isinstance(proof, dict) else {}
        complete = complete and (job.get("status") == "terminal" and job.get("owner") == eid
            and job.get("executionId") == eid and isinstance(proof, dict) and proof.get("complete") is True
            and job.get("terminalProof") == proof and type(proof.get("processExitCode")) is int
            and type(proof.get("activeProcesses")) is int and proof["activeProcesses"] == 0
            and all(proof.get(field) is True for field in ("stdoutEof", "stderrEof", "childrenGone"))
            and type(job.get("pid")) is int and job["pid"] > 0
            and isinstance(job.get("birthToken"), str) and job["birthToken"].isdecimal() and int(job["birthToken"]) > 0
            and isinstance(job.get("executable"), str) and bool(job["executable"])
            and all(identity.get(field) == expected for field, expected in
                    {"pid": job.get("pid"), "creationTime": job.get("birthToken"), "executable": job.get("executable")}.items()))
        if complete:
            from ..processes.identity import ProcessIdentity, identity_matches
            try:
                complete = not identity_matches(ProcessIdentity(job["pid"], job["birthToken"], job["executable"]))
            except (JenkinsError, OSError, ValueError):
                complete = False
    complete = complete and guard_phases_complete(state, guard)
    if launch and guard:
        complete = complete and launch["payload"].get("guardNativeJobId") == guard["key"]
    return {"status": "terminal" if complete else "pending", "nativeJobIds": sorted(row["key"] for row in jobs),
            "proofDigest": digest([row["payload"] for row in jobs]), "cleanupAllowed": bool(complete)}


def _scratch_process_binding(state, execution, scratch, native):
    expected = {"scratchGeneration": scratch["scratchGeneration"],
                "effectiveEnvironmentDigest": digest(scratch["effectiveEnvironment"])}
    if execution["payload"].get("taskScratch") != expected:
        raise _fail("task_scratch_contract_invalid", "The execution lacks its exact prelaunch TEMP binding")
    if native["status"] == "terminal":
        host = state.get("execution_host", execution["key"])
        expected_host = {"scratchGeneration": scratch["scratchGeneration"],
                         "effectiveEnvironment": scratch["effectiveEnvironment"]}
        phases = host["payload"].get("phaseRecords", []) if host else []
        if (not host or host["payload"].get("taskScratch") != expected_host or not phases
                or any(phase.get("taskScratch") != expected_host
                       or not isinstance(phase.get("effectiveEnvironmentDigest"), str)
                       or len(phase["effectiveEnvironmentDigest"]) != 64 for phase in phases)):
            raise _fail("task_scratch_environment_unproven", "Every native phase must retain the recorded effective TEMP generation")


def _references(state, candidate):
    from ..maintenance import _active_refs, _historical_digests
    protected = _active_refs(state) | _historical_digests(state)
    paths = []
    def walk(value):
        if isinstance(value, dict):
            for key, item in value.items():
                if isinstance(item, str) and (key.lower().endswith("digest") or key.lower() in {"digest", "sha256", "artifact"}) and len(item) == 64:
                    protected.add(item)
                # Drive-root identity is a policy binding, not a reference to
                # every child. Actual producer/input storage remains a pin.
                if isinstance(item, str) and key not in {"buildRoot", "repositoryRoot"} and (key.lower().endswith("path") or key.lower().endswith("root")):
                    if _overlap(item, candidate):
                        paths.append(item)
                walk(item)
        elif isinstance(value, list):
            for item in value:
                walk(item)
    for domain in ("artifact_object", "artifact_bundle", "artifact_ref", "acceptance", "validation_receipt",
                   "validation_attempt", "sealed_input", "preparation_source", "external_input", "request_result"):
        for row in state.list(domain):
            walk(row["payload"])
    return protected, sorted(set(paths))


@contextmanager
def _storage(approved, scratch):
    from ..pilot.native.windows_storage import WindowsStorage
    physical_path_under(approved, scratch["path"], allow_missing=False)
    with WindowsStorage(scratch["path"], expected_device=approved.volume_identity) as storage:
        if _directory_fact(storage.root_identity) != scratch["directoryIdentity"]:
            raise _fail("task_scratch_generation_changed", "Scratch physical generation changed")
        yield storage


def _bounded_files(storage, prefix, limits, deadline):
    from ..pilot.native.windows_storage import (_directory_entries, _open_relative_handle, _ensure_directory_handle,
        _close_handle, _DIRECTORY_READ_ACCESS, _FILE_SHARE_READ, _FILE_SHARE_WRITE, _FILE_DIRECTORY_FILE,
        _FILE_OPEN_REPARSE_POINT, _FILE_SYNCHRONOUS_IO_NONALERT)
    count = 0
    def walk(handle, relative, depth):
        nonlocal count
        if depth > 16:
            raise _fail("post_task_inventory_limit", "Scratch directory depth exceeds its bounded inventory")
        for name, attrs in _directory_entries(handle):
            count += 1
            if count > limits["maxFiles"] * 3 or time.monotonic() > deadline:
                raise _fail("post_task_inventory_limit", "Scratch inventory exceeds its entry/time bound")
            path = relative + "/" + name if relative else name
            if attrs & 0x400:
                raise _fail("task_scratch_alias", "Scratch contains a reparse point")
            if attrs & 0x10:
                child = _open_relative_handle(handle, name, access=_DIRECTORY_READ_ACCESS,
                    share=_FILE_SHARE_READ | _FILE_SHARE_WRITE,
                    options=_FILE_DIRECTORY_FILE | _FILE_OPEN_REPARSE_POINT | _FILE_SYNCHRONOUS_IO_NONALERT)
                try:
                    _ensure_directory_handle(child)
                    yield from walk(child, path, depth + 1)
                finally:
                    _close_handle(child)
            else:
                yield path
    if not prefix:
        yield from walk(storage._root_handle, "", 0)
        return
    _, handles = storage._directory_relative(prefix)
    try:
        yield from walk(handles[-1], prefix, 0)
    finally:
        for handle in reversed(handles):
            _close_handle(handle)


def _scratch_inventory(state, approved, scratch, limits):
    protected, references = _references(state, scratch["path"])
    if references:
        return {"status": "protected", "references": references, "files": [], "totalBytes": 0}
    files, total, retained = [], 0, []
    deadline = time.monotonic() + limits["maxSeconds"]
    with _storage(approved, scratch) as storage:
        for relative in _bounded_files(storage, "", limits, deadline):
            if len(files) + len(retained) >= limits["maxFiles"]:
                raise _fail("post_task_inventory_limit", "Scratch file inventory exceeds its bound")
            fact, _ = _file_fact(storage, relative, limits["maxBytes"] - total, deadline)
            total += fact["byteLength"]
            if fact["sha256"] in protected:
                retained.append({**fact, "reasonCode": "historical_or_active_digest_reference"})
            else:
                files.append(fact)
    value = {"status": "complete", "files": files, "retained": retained, "totalBytes": total,
             "scratchGeneration": scratch["scratchGeneration"], "directoryIdentity": scratch["directoryIdentity"]}
    return {**value, "inventoryDigest": digest(value)}


def _file_fact(storage, relative, maximum, deadline):
    from ..pilot.native.windows_storage import identity_from_stream
    with storage.open_relative(relative, "rb") as stream:
        before = identity_from_stream(stream)
        if before.size > maximum or before.link_count != 1:
            raise _fail("post_task_inventory_limit", "Scratch bytes or hard-link count exceed the bounded policy")
        checksum, size = hashlib.sha256(), 0
        while True:
            if time.monotonic() > deadline:
                raise _fail("post_task_inventory_limit", "Scratch file hashing exceeded its time bound")
            chunk = stream.read(1024 * 1024)
            if not chunk:
                break
            size += len(chunk)
            if size > maximum:
                raise _fail("post_task_inventory_limit", "Scratch file hashing exceeded its byte bound")
            checksum.update(chunk)
        after = identity_from_stream(stream)
        if not before.same_file_state(after) or size != after.size or after.link_count != 1:
            raise _fail("task_scratch_content_changed", "Scratch file changed during terminal inventory")
    return {"path": relative, "sha256": checksum.hexdigest(), "byteLength": size,
            "fileId": after.file_id.hex(), "device": after.device, "mtimeNs": after.mtime_ns,
            "linkCount": after.link_count}, after


def _pool_quiet(state, scratch, *, connection=None):
    hold = state.get("pool_writer_hold", scratch["preparationKey"], connection=connection)
    if hold and hold["payload"].get("status") == "active":
        raise _fail("post_task_writer_active", "A preparation writer still owns this pool", retryable=True)
    for row in state.list("resource_reservation", connection=connection):
        if row["payload"].get("status") == "active" and row["payload"].get("owner") == scratch["executionId"]:
            raise _fail("post_task_writer_active", "The original execution still has an active reservation", retryable=True)


def _verify_record(value, authority):
    if value.get("authorityDigest") != digest(authority) or value.get("resultDigest") != digest({key: item for key, item in value.items() if key != "resultDigest"}):
        raise _fail("post_task_receipt_changed", "The immutable task cache assessment changed")
    return value


def assess(payload, state, repo_root):
    authority, approved, flow, execution = _authority(payload, state)
    operation = authority["operationId"]
    old = state.get("post_task_cache", operation)
    if old:
        return _verify_record(old["payload"], authority)
    policy_row, cache, limits = _policy(state)
    policy = policy_row["payload"]
    manager = ResourceManager(state, Capacity(*(policy[field] for field in ("cpuBudget", "memoryBudget", "diskBudget"))))
    snapshot = manager.capture_inventory(str(approved.path))
    observed = snapshot.get("observedAt")
    if (snapshot.get("complete") is not True or not _same(snapshot.get("buildRoot"), str(approved.path))
            or type(observed) not in (int, float) or not math.isfinite(observed) or not 0 <= time.time() - observed <= 30
            or any(type(snapshot.get(field)) is not int or snapshot[field] <= 0 for field in ("cpu", "memoryBytes", "diskBytes"))):
        raise _fail("storage_snapshot_stale", "Post-task assessment requires fresh complete capacity", retryable=True)
    native = _native_evidence(state, execution)
    scratch = state.get("execution_scratch", execution["key"]) if execution else None
    disposition = {"status": "no_registered_task_scratch", "files": [], "totalBytes": 0}
    if scratch:
        if native["cleanupAllowed"]:
            try:
                registered = require_execution_scratch(state, execution["key"])
                _scratch_process_binding(state, execution, registered, native)
                _pool_quiet(state, registered)
                disposition = _scratch_inventory(state, approved, registered, limits)
            except (JenkinsError, OSError, ValueError) as error:
                disposition = {"status": "protected", "reasonCode": getattr(error, "code", type(error).__name__),
                               "files": [], "totalBytes": 0}
        else:
            disposition = {"status": "native_terminal_pending", "files": [], "totalBytes": 0}
    grant = False
    try:
        state.check_authorization(authority["identity"]["repositoryId"], authority["identity"]["sessionId"], "gc", authority["ownedPaths"])
        grant = True
    except JenkinsError:
        pass
    active = [row for row in state.list("resource_reservation") if row["payload"].get("status") == "active"]
    writers = [row for row in state.list("pool_writer_hold") if row["payload"].get("status") == "active"]
    head = manager.fair_queue()
    from ..resources.capacity import _stored_root
    root_growth = sum(row["payload"].get("diskBytes", 0) for row in active
                      if not _stored_root(row["payload"].get("buildRoot"))
                      or _stored_root(row["payload"].get("buildRoot")) == _stored_root(str(approved.path)))
    result = {"operationId": operation, "authorityDigest": digest(authority), "authority": authority,
              "policyVersion": policy_row["version"], "policyDigest": digest(policy), "limits": limits,
              "observedAt": snapshot["observedAt"], "capacity": snapshot,
              "activeReservationKeys": [row["key"] for row in active], "activeWriterKeys": [row["key"] for row in writers],
              "reservationUsage": {"cpu": sum(row["payload"].get("cpu", 0) for row in active),
                                   "memoryBytes": sum(row["payload"].get("memoryBytes", 0) for row in active),
                                   "globalDiskBytes": sum(row["payload"].get("diskBytes", 0) for row in active),
                                   "selectedRootGrowthBytes": root_growth,
                                   "selectedRootAvailableBytes": max(0, snapshot["diskBytes"] - root_growth - cache["minFreeBytes"])},
              "queueHead": {"key": head[0]["key"], "buildRoot": head[0]["payload"].get("buildRoot"),
                            "policy": "weighted_aging_skip_only_proven_root_disk_pressure" if policy.get("skipProvenRootDiskBlockedHeads") is True
                                      else "global_weighted_aging_no_fit_bypass"} if head else None,
              "fairDecisionKeys": [row["key"] for row in state.list("resource_fair_decision")
                                   if execution and row["payload"].get("reservationKey", "").startswith(execution["key"] + ":")],
              "nativeEvidence": native, "scratch": disposition,
              "warmPreparation": {"disposition": "preserve", "preparationKey": execution["payload"].get("preparationKey") if execution else None},
              "unknownOwnership": "preserve", "acceptedReferences": "preserve", "gcAuthorized": grant,
              "cleanupRequired": bool(disposition.get("files")), "diskPressure": snapshot["diskBytes"] < cache["minFreeBytes"],
              "status": "pending" if native["status"] == "pending" else "assessed",
              "JenkinsAcceptance": False, "milestoneAcceptance": False}
    result["resultDigest"] = digest(result)
    with state.transaction() as connection:
        _authority(payload, state)
        old = state.get("post_task_cache", operation, connection=connection)
        if old:
            return _verify_record(old["payload"], authority)
        if result["status"] == "pending":
            state.put("post_task_cache_pending", operation, result, connection=connection)
        else:
            state.put("post_task_cache", operation, result, expected_version=0, connection=connection)
    return result


def garbage_collect(payload, state, repo_root):
    authority, approved, flow, execution = _authority(payload, state)
    operation = authority["operationId"]
    state.check_authorization(authority["identity"]["repositoryId"], authority["identity"]["sessionId"], "gc", authority["ownedPaths"])
    row = state.get("post_task_cache", operation)
    if not row:
        raise _fail("post_task_assessment_required", "GC requires its original immutable post-task assessment")
    assessment = _verify_record(row["payload"], authority)
    inventory = assessment["scratch"]
    if not execution or inventory.get("status") != "complete" or not inventory.get("files"):
        return {"operationId": operation, "status": "no_eligible_files", "reclaimedBytes": 0,
                "removedFileCount": 0, "unknownProtected": True, "historyPreserved": True}
    scratch = require_execution_scratch(state, execution["key"])
    if inventory.get("scratchGeneration") != scratch["scratchGeneration"]:
        raise _fail("task_scratch_generation_changed", "Assessment belongs to another scratch generation")
    native = _native_evidence(state, execution)
    if not native["cleanupAllowed"]:
        raise _fail("native_termination_unproven", "Complete guard and phase EOF evidence is required", retryable=True)
    _scratch_process_binding(state, execution, scratch, native)
    _, _, limits = _policy(state)
    limits = {key: min(limits[key], assessment["limits"][key]) for key in _LIMITS}
    lock = state.path.parent / "cache-maintenance-locks" / (scratch["preparationKey"] + ".lock")
    with process_lock(lock, reason="post_task_cache_busy"):
        with state.transaction() as connection:
            _pool_quiet(state, scratch, connection=connection)
            hold = state.get("cache_maintenance_hold", scratch["preparationKey"], connection=connection)
            if hold and hold["payload"].get("status") == "active" and hold["payload"].get("operationId") != operation:
                raise _fail("post_task_cache_busy", "Another exact cache operation holds this preparation", retryable=True)
            state.put("cache_maintenance_hold", scratch["preparationKey"], {"operationId": operation, "status": "active",
                      "scratchGeneration": scratch["scratchGeneration"]}, connection=connection)
        result = {"operationId": operation, "assessmentDigest": assessment["resultDigest"], "status": "complete",
                  "removedFileCount": 0, "reclaimedBytes": 0, "unknown": [], "historyPreserved": True}
        deadline = time.monotonic() + limits["maxSeconds"]
        try:
            with _storage(approved, scratch) as storage:
                for fact in inventory["files"]:
                    key = digest({"operation": operation, "file": fact})
                    receipt = state.get("post_task_cache_file", key)
                    if receipt:
                        value = receipt["payload"]
                        if (value.get("file") != fact or value.get("assessmentDigest") != assessment["resultDigest"]
                                or value.get("status") != "removed" or value.get("physicalNameAbsent") is not True
                                or storage.exists(fact["path"])):
                            result["status"] = "unknown"
                            result["unknown"].append({"path": fact["path"], "reasonCode": "deletion_acknowledgement_unknown"})
                            break
                        result["removedFileCount"] += 1
                        result["reclaimedBytes"] += fact["byteLength"]
                        continue
                    if (result["removedFileCount"] >= limits["maxFiles"] or result["reclaimedBytes"] + fact["byteLength"] > limits["maxBytes"]
                            or time.monotonic() > deadline):
                        result["status"] = "yielded"
                        break
                    # Hash outside SQLite's writer transaction. The durable
                    # preparation hold fences this pool while independent
                    # pools continue admission during bounded file reads.
                    current, identity = _file_fact(storage, fact["path"], fact["byteLength"], deadline)
                    with state.transaction() as connection:
                        _authority(payload, state)
                        _pool_quiet(state, scratch, connection=connection)
                        protected, refs = _references(state, scratch["path"])
                        if refs or fact["sha256"] in protected:
                            result["status"] = "protected"
                            break
                        if current != fact:
                            raise _fail("task_scratch_content_changed", "Scratch file differs from its registered terminal inventory")
                        state.put("post_task_cache_file", key, {"operationId": operation, "assessmentDigest": assessment["resultDigest"],
                                  "file": fact, "status": "pending"}, expected_version=0, connection=connection)
                    # The durable per-file intent is committed before unlink.
                    current, identity = _file_fact(storage, fact["path"], fact["byteLength"], deadline)
                    with state.transaction() as connection:
                        _authority(payload, state)
                        _pool_quiet(state, scratch, connection=connection)
                        protected, refs = _references(state, scratch["path"])
                        if refs or fact["sha256"] in protected or current != fact:
                            raise _fail("task_scratch_content_changed", "Scratch ownership or content changed before deletion")
                        if time.monotonic() > deadline:
                            raise _fail("post_task_inventory_limit", "No unlink is allowed after the cleanup deadline")
                        storage.delete(fact["path"], expected_identity=identity)
                        if storage.exists(fact["path"]):
                            raise _fail("task_scratch_deletion_unproven", "The deleted file still has a physical name")
                        state.put("post_task_cache_file", key, {"operationId": operation, "assessmentDigest": assessment["resultDigest"],
                                  "file": fact, "status": "removed", "physicalNameAbsent": True}, connection=connection)
                    result["removedFileCount"] += 1
                    result["reclaimedBytes"] += fact["byteLength"]
        except (JenkinsError, OSError, ValueError) as error:
            result["status"] = "partial" if result["removedFileCount"] else "unknown"
            result["unknown"].append({"reasonCode": getattr(error, "code", type(error).__name__)})
        finally:
            with state.transaction() as connection:
                hold = state.get("cache_maintenance_hold", scratch["preparationKey"], connection=connection)
                if hold and hold["payload"].get("operationId") == operation:
                    state.put("cache_maintenance_hold", hold["key"], {**hold["payload"], "status": "released"}, connection=connection)
        result["resultDigest"] = digest(result)
        state.put("post_task_cache_gc", operation, result)
        return result


def handle(action, payload, state, repo_root):
    phase = payload.get("phase", "assess")
    if action != "post-task-cache" or phase not in {"assess", "gc", "reconcile"}:
        raise _fail("unsupported_post_task_cache_action", "Post-task cache uses assess, gc or reconcile")
    authority, _, _, _ = _authority(payload, state)
    if phase == "gc":
        result = garbage_collect(payload, state, repo_root)
    elif phase == "assess":
        result = assess(payload, state, repo_root)
    else:
        row = state.get("post_task_cache", authority["operationId"]) or state.get("post_task_cache_pending", authority["operationId"])
        result = _verify_record(row["payload"], authority) if row else {"status": "not_assessed", "operationId": authority["operationId"]}
        gc = state.get("post_task_cache_gc", authority["operationId"])
        if gc:
            result = {"assessment": result, "gc": gc["payload"], "status": gc["payload"]["status"]}
    return response("observed", request_id=authority["identity"]["requestId"], operation_id=authority["operationId"],
                    observed_generation=authority["identity"]["generation"], result=result)

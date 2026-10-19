"""Reader freezes and mutation reservations for one reusable Jenkins worktree."""
from __future__ import annotations

from contextlib import contextmanager
import hashlib
from pathlib import Path

from .contracts import JenkinsError, digest, require_digest
from .deployment.driver import _plain

MUTATIONS = frozenset({"prepare-worktree", "apply-patch", "generate", "commit-worktree",
                      "release-worktree", "recover-worktree"})

def _require(value, code, message):
    if not value:
        raise JenkinsError(code, message)


def _pin(row):
    return {"domain": row["domain"], "key": row["key"], "version": row["version"],
            "payloadDigest": digest(row["payload"])}


def _pins_current(state, pins, *, connection=None):
    return bool(pins) and all(
        (row := state.get(pin["domain"], pin["key"], connection=connection)) is not None
        and row["version"] == pin["version"] and digest(row["payload"]) == pin["payloadDigest"]
        for pin in pins)


def reader_retired(state, reference, *, connection=None):
    row = state.get("managed_worktree_reader_retirement", reference, connection=connection)
    return bool(row and row["payload"].get("status") == "retired"
                and row["payload"].get("currentSourceRef") == reference
                and _pins_current(state, row["payload"].get("terminalPins", []), connection=connection))


def active_readers(state, worktree_id, root, *, connection=None):
    values = []
    for row in state.list("current_source_intent", connection=connection):
        value = row["payload"]
        related = (value.get("worktreeId") == worktree_id
                   or (isinstance(value.get("sourceRoot"), str)
                       and Path(value["sourceRoot"]).resolve() == root.resolve()))
        if related and not reader_retired(state, row["key"], connection=connection):
            values.append(row["key"])
    return values


def assert_no_writer(state, worktree_id, *, connection=None):
    row = state.get("managed_worktree_writer", worktree_id, connection=connection)
    _require(not row or row["payload"].get("status") == "released",
             "managed_worktree_writer_active", "Managed worktree has an active or interrupted writer")


@contextmanager
def writer_hold(management):
    """Reserve in the same SQLite authority used by current-source admission."""
    reference = management.payload["worktreeId"]
    row = management.state.get("management_worktree", reference)
    _require(row is not None, "managed_worktree_missing", "Managed target is not registered")
    root = Path(row["payload"]["path"])
    _plain(root)
    binding = {"worktreeId": reference, "operationKey": management.key,
               "inputDigest": management.input_digest, "status": "active"}
    with management.state.transaction() as connection:
        assert_no_writer(management.state, reference, connection=connection)
        _require(not active_readers(management.state, reference, root, connection=connection),
                 "managed_worktree_reader_active", "A registered source reader freezes this worktree")
        old = management.state.get("managed_worktree_writer", reference, connection=connection)
        management.state.put("managed_worktree_writer", reference, binding,
                             expected_version=old["version"] if old else 0, connection=connection)
    # A killed process leaves the durable reservation active. Recovery must prove
    # that original operation/process terminal; another caller cannot adopt it.
    management._worktree_mutation_started = management.payload.get("action") != "prepare-worktree"
    completed = False
    try:
        yield
        completed = True
    finally:
        with management.state.transaction() as connection:
            row = management.state.get("managed_worktree_writer", reference, connection=connection)
            _require(row is not None and row["payload"] == binding,
                     "managed_worktree_writer_changed", "Managed writer identity changed")
            status = "released" if completed or not management._worktree_mutation_started else "interrupted"
            management.state.put("managed_worktree_writer", reference,
                                 {**binding, "status": status},
                                 expected_version=row["version"], connection=connection)



def retire_reader(management):
    from .deployment.drain_observations import native_terminal
    from .deployment.execution_launch_reconciliation import _complete_proof
    from .processes.guards import guard_phases_complete
    reference = require_digest(management.payload.get("currentSourceRef"), "currentSourceRef")
    with management.state.transaction() as connection:
        intent_row = management.state.get("current_source_intent", reference, connection=connection)
        _require(intent_row and digest(intent_row["payload"]) == reference,
                 "managed_reader_missing", "The immutable reader intent is missing")
        intent = intent_row["payload"]
        record = management.state.get("management_worktree", intent.get("worktreeId", ""), connection=connection)
        _require(record and record["payload"].get("sessionId") == management.session
                 and intent.get("worktreeId") == management.payload.get("worktreeId"),
                 "managed_reader_owner_mismatch", "Only this WT's narrow writer can retire its reader")
        identity = intent["identity"]
        request = management.state.get_request(identity["repositoryId"], identity["sessionId"],
                                                identity["requestId"], connection=connection)
        _require(request and request["status"] in {"accepted", "failed", "cancelled"}
                 and all(request["payload"].get(key) == value for key, value in identity.items())
                 and request["payload"].get("currentSourceRef") == reference,
                 "managed_reader_active", "Reader request is not terminal")
        prepare = management.state.get("current_source_preparation", reference, connection=connection)
        _require(prepare and prepare["payload"].get("status") == "prepared",
                 "managed_reader_preparation_missing", "Reader source preparation must be reconciled")
        sealed = prepare["payload"]["result"]["sealedInputRef"]
        flows = [row for row in management.state.list("workflow", connection=connection)
                 if all(row["payload"].get("identity", {}).get(key) == value for key, value in identity.items())
                 and row["payload"].get("sourceDigest") == sealed]
        _require(len(flows) == 1 and flows[0]["payload"].get("status") in {"accepted", "failed", "cancelled", "rejected"},
                 "managed_reader_flow_active", "The exact workflow is unresolved")
        flow = flows[0]
        eid = flow["payload"].get("executionId")
        execution = management.state.get("execution", eid, connection=connection) if eid else None
        _require(execution and execution["payload"].get("status") in {"passed", "failed", "cancelled"}
                 and execution["payload"].get("sourceDigest") == sealed,
                 "managed_reader_execution_active", "Execution has no terminal matching source")
        consumers = [row for row in management.state.list("execution_consumer", connection=connection)
                     if row["payload"].get("executionId") == eid]
        own_consumer = next((row for row in consumers if row["payload"].get("consumerId") == flow["key"]), None)
        binding = management.state.get("execution_binding", digest({
            "consumer": flow["key"], "executionKey": execution["payload"].get("executionKeyDigest")}),
            connection=connection)
        _require(own_consumer and binding and binding["payload"] == {
                     "executionId": eid, "consumerId": flow["key"]}
                 and all(row["payload"].get("status") == "released" for row in consumers
                         if row["key"] != own_consumer["key"]),
                 "managed_reader_consumer_active", "Other execution consumers must be reconciled first")
        launch = management.state.get("execution_launch", eid, connection=connection)
        host = management.state.get("execution_host", eid, connection=connection)
        plan = management.state.get("recipe_plan", flow["payload"].get("recipeRef", ""), connection=connection)
        recipe = plan["payload"].get("recipe", {}) if plan else {}
        phases = execution["payload"].get("phaseRecords", [])
        guard_id = launch["payload"].get("guardNativeJobId") if launch else None
        native = [row for row in management.state.list("native_job", connection=connection)
                  if row["payload"].get("executionId") == eid or row["payload"].get("owner") == eid
                  or (guard_id and row["payload"].get("guardNativeJobId") == guard_id)]
        jobs = {row["key"]: row for row in native}
        guard = jobs.get(guard_id)
        required = recipe.get("requiredPhases", [])
        actual = [phase.get("phase") for phase in phases]
        guard_only_failure = (not phases and execution["payload"]["status"] == "failed"
                              and host and host["payload"].get("status") == "failed")
        _require(native and launch and host and plan and guard and (phases or guard_only_failure) and required
                 and guard["payload"].get("role") == "execution_host_guard"
                 and launch["payload"].get("status") == "exited"
                 and launch["payload"].get("sourceDigest") == sealed
                 and launch["payload"].get("recipeRef") == plan["key"]
                 and execution["payload"].get("recipeRef") == plan["key"]
                 and plan["payload"].get("sourceRef") == sealed
                 and digest(recipe) == plan["key"]
                 and (host["payload"].get("status") in {"terminal", "cancelled"} or guard_only_failure)
                 and host["payload"].get("phaseRecords") == phases
                 and actual == required[:len(actual)]
                 and (execution["payload"]["status"] != "passed" or actual == required)
                 and len({phase.get("nativeJobId") for phase in phases}) == len(phases)
                 and set(jobs) == {guard_id, *[phase.get("nativeJobId") for phase in phases]}
                 and all(native_terminal(row) and row["payload"].get("terminalProof") == row["payload"].get("completeProof")
                         and row["payload"].get("owner") == eid and row["payload"].get("executionId") == eid
                         for row in native),
                 "managed_reader_native_active", "Every declared phase and host Job needs actual terminal/EOF proof")
        resource_binding = None
        if guard_only_failure:
            proof = _complete_proof(guard, eid, role="execution_host_guard")
            abort = execution["payload"].get("abortProof", {})
            _require(guard_phases_complete(management.state, guard, connection=connection)
                     and proof["processExitCode"] == 1 and launch["payload"].get("exitCode") == 1
                     and proof["identity"] == launch["payload"].get("launcherIdentity")
                     and host["payload"].get("hostPid") == proof["identity"]["pid"]
                     and host["payload"].get("hostBirthToken") == proof["identity"]["creationTime"]
                     and abort.get("kind") == "execution-host-aborted"
                     and abort.get("guardNativeJobId") == guard_id
                     and abort.get("proofDigest") == digest(proof)
                     and abort.get("outcome") == "failed"
                     and abort.get("artifactPublicationAllowed") is False,
                     "managed_reader_native_active", "Guard-only failure lacks exact host termination evidence")
            reservation_key = execution["payload"].get("reservationKey")
            reservations = [row for row in management.state.list("resource_reservation", connection=connection)
                            if row["payload"].get("owner") == eid or row["key"] == reservation_key]
            holds = [row for row in management.state.list("pool_writer_hold", connection=connection)
                     if row["payload"].get("owner") == eid
                     or row["payload"].get("reservationKey") == reservation_key]
            _require(reservation_key and len(reservations) == len(holds) == 1
                     and reservations[0]["key"] == reservation_key
                     and reservations[0]["payload"].get("owner") == eid
                     and reservations[0]["payload"].get("status") == "released"
                     and holds[0]["key"] == execution["payload"].get("preparationKey")
                     and holds[0]["payload"].get("owner") == eid
                     and holds[0]["payload"].get("reservationKey") == reservation_key
                     and holds[0]["payload"].get("status") == "released",
                     "managed_reader_resource_active", "Guard-only failure retains unresolved resource ownership")
            resource_binding = {"reservation": _pin(reservations[0]), "writerHold": _pin(holds[0])}
        for phase in phases:
            job = jobs[phase["nativeJobId"]]["payload"]
            command = recipe.get("commands", {}).get(phase["phase"])
            _require(isinstance(command, list) and command
                     and job.get("role") == "phase" and job.get("guardNativeJobId") == guard_id
                     and phase.get("terminalProof") == job["completeProof"]
                     and phase.get("commandDigest") == digest(command)
                     and job["completeProof"]["identity"].get("commandDigest")
                         == hashlib.sha256("\0".join(command).encode()).hexdigest(),
                     "managed_reader_native_active", "Native phase command or receipt changed")
        writer = management.state.check_authorization(management.repo_id, management.session,
                    "implementation", record["payload"]["ownedPaths"], connection=connection)
        _require(writer["snapshotDigest"] == record["payload"].get("writerAuthorizationSnapshot")
                 and writer["principal"] == record["payload"].get("writerPrincipal")
                 and sorted(writer["paths"]) == sorted(management.owned) == sorted(record["payload"]["ownedPaths"]),
                 "managed_reader_owner_mismatch", "Original narrow writer authority changed")
        if own_consumer["payload"].get("status") == "active":
            management.state.put("execution_consumer", own_consumer["key"],
                {**own_consumer["payload"], "status": "released"}, expected_version=own_consumer["version"],
                connection=connection)
            current = execution["payload"]
            remaining = [value for value in current.get("consumers", []) if value != flow["key"]]
            execution = management.state.put("execution", eid,
                {**current, "consumers": remaining, "refs": len(remaining)},
                expected_version=execution["version"], connection=connection)
        else:
            _require(own_consumer["payload"].get("status") == "released",
                     "managed_reader_consumer_active", "The original execution consumer is unresolved")
        consumers = [management.state.get("execution_consumer", row["key"], connection=connection)
                     for row in consumers]
        pins = [_pin(row) for row in (intent_row, prepare, flow, execution, launch, host, plan,
                                     binding, *consumers, *native)]
        value = {"currentSourceRef": reference, "worktreeId": intent["worktreeId"],
                 "status": "retired", "terminalPins": pins, "requestStatus": request["status"],
                 "retiredBy": management.key, "acceptanceSynthesized": False,
                 "worktreeBindingAtRetirement": _pin(record)}
        if guard_only_failure:
            value.update(retirementKind="failed-before-first-phase-guard-terminal-v1",
                         resourceBindingAtRetirement=resource_binding)
        old = management.state.get("managed_worktree_reader_retirement", reference, connection=connection)
        if old:
            _require(old["payload"] == value, "managed_reader_retirement_conflict", "Retirement input changed")
        else:
            management.state.put("managed_worktree_reader_retirement", reference, value,
                                 expected_version=0, connection=connection)
    return {"status": "complete", "worktreeId": intent["worktreeId"], "currentSourceRef": reference,
            "readerRetired": True, "physicalWorktreeRetained": True, "formalAcceptance": False}

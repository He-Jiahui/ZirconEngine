"""Release a formally accepted flow's write claims while retaining its inputs.

This is a source-authority closeout, not execution cancellation, compensation,
acceptance issuance, or a new source request. All State changes share one CAS
transaction; immutable inputs and consumer/artifact references are untouched.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

from ..contracts import JenkinsError, digest, file_digest, identifier, require_digest, response
from ..resources import canonical_build_root, physical_path_under
from ..validation.receipts import HEAVY_STAGES, validate_acceptance_receipt, validate_producer_receipt
from ..workflow.orchestrator import TEMPLATES as WORKFLOW_TEMPLATES
from . import _before_hash, _consumer_key, _overlaps, canonical_path, canonical_paths, compute_source_digest, repository_id


TEMPLATES = frozenset({"comments_only", "module_unit", "cross_module", "failure_repair"})


def _need(condition, message):
    if not condition:
        raise JenkinsError("source_release_unproven", message)


def _row(state, connection, domain, key):
    _need(isinstance(key, str) and bool(key), f"Missing {domain} reference")
    row = state.get(domain, key, connection=connection)
    _need(row is not None and isinstance(row.get("payload"), dict), f"Missing authoritative {domain}")
    return row


def _native(row, execution_id, phase_proof=None):
    job = row["payload"]
    proof = job.get("completeProof")
    _need(job.get("nativeJobId") == row["key"] and job.get("executionId") == execution_id
          and job.get("owner") == execution_id and job.get("status") == "terminal", "Native Job ownership or terminal state differs")
    _need(isinstance(proof, dict) and proof.get("complete") is True
          and proof.get("childrenGone") is True and proof.get("stdoutEof") is True
          and proof.get("stderrEof") is True and proof.get("requested") is True
          and type(proof.get("activeProcesses")) is int and proof["activeProcesses"] == 0
          and type(proof.get("processExitCode")) is int and proof["processExitCode"] == 0,
          "Complete successful native termination is required")
    pid, birth, executable = job.get("pid"), job.get("birthToken"), job.get("executable")
    _need(type(pid) is int and pid > 0 and isinstance(birth, str) and birth.isdecimal()
          and int(birth) > 0 and isinstance(executable, str) and bool(executable), "Native identity is incomplete")
    identity = proof.get("identity")
    _need(isinstance(identity, dict) and type(identity.get("pid")) is int and identity["pid"] == pid
          and identity.get("creationTime") == birth and identity.get("executable") == executable,
          "Native proof does not identify its recorded process")
    require_digest(identity.get("commandDigest"), "native commandDigest")
    _need(digest(job.get("terminalProof")) == digest(proof), "Native terminal proof changed")
    if phase_proof is not None:
        _need(digest(phase_proof) == digest(proof), "Native phase proof changed")
    return proof


def _native_execution(state, connection, flow, source, coverage, driver):
    eid = flow.get("executionId")
    execution = _row(state, connection, "execution", eid)
    value = execution["payload"]
    plan = _row(state, connection, "recipe_plan", flow.get("recipeRef"))
    recipe = plan["payload"].get("recipe")
    _need(isinstance(recipe, dict) and plan["payload"].get("recipeDigest") == plan["key"]
          and digest(recipe) == plan["key"] and plan["payload"].get("sourceRef") == source
          and plan["payload"].get("coverageRef") == coverage, "Execution recipe authority differs")
    _need(value.get("status") == "passed" and value.get("sourceDigest") == source
          and value.get("coverageDigest") == coverage and value.get("driverDigest") == driver
          and value.get("recipeRef") == plan["key"] and value.get("recipeDigest") == plan["key"]
          and digest(value.get("driver")) == digest(plan["payload"].get("driver"))
          and value.get("operationId") == (value.get("driver") or {}).get("runtimeOperationId"),
          "Original passed execution, driver, or runtime differs")
    _need(value.get("buildRoot") == flow.get("buildRoot") and recipe.get("buildRoot") == flow.get("buildRoot"),
          "Original execution compiler root differs")
    _need((value.get("driver") or {}).get("digest") == driver
          and recipe.get("driverDigest") == driver and recipe.get("sourceDigest") == source
          and recipe.get("coverageDigest") == coverage
          and isinstance(value["driver"].get("runtimeOperationId"), str)
          and bool(value["driver"]["runtimeOperationId"]), "Original runtime identity is missing")
    phases = value.get("phaseRecords")
    required = recipe.get("requiredPhases")
    _need(isinstance(required, list) and required == ["compile", "list", "test"]
          and isinstance(phases, list) and [p.get("phase") for p in phases] == required,
          "Managed compile, list, and test phase closure is required")
    host = _row(state, connection, "execution_host", eid)
    launch = _row(state, connection, "execution_launch", eid)
    _need(host["payload"].get("status") == "terminal" and host["payload"].get("executionId") == eid
          and host["payload"].get("operationId") == value["operationId"]
          and digest(host["payload"].get("phaseRecords")) == digest(phases)
          and launch["payload"].get("status") == "exited" and launch["payload"].get("executionId") == eid
          and launch["payload"].get("driverDigest") == driver
          and launch["payload"].get("sourceDigest") == source
          and launch["payload"].get("recipeRef") == plan["key"]
          and launch["payload"].get("operationId") == value["operationId"]
          and type(launch["payload"].get("exitCode")) is int and launch["payload"]["exitCode"] == 0,
          "Original managed host or launch is not terminal")
    guard_id = launch["payload"].get("guardNativeJobId")
    guard = _row(state, connection, "native_job", guard_id)
    _need(guard["payload"].get("role") == "execution_host_guard", "Managed host guard is missing")
    _native(guard, eid)
    _need(host["payload"].get("hostPid") == guard["payload"].get("pid")
          and host["payload"].get("hostBirthToken") == guard["payload"].get("birthToken")
          and digest(launch["payload"].get("launcherIdentity")) == digest(guard["payload"]["completeProof"]["identity"])
          and type(launch["payload"].get("launcherPid")) is int and launch["payload"]["launcherPid"] == guard["payload"]["pid"]
          and launch["payload"].get("launcherBirthToken") == guard["payload"]["birthToken"]
          and launch["payload"].get("launcherExecutable") == guard["payload"]["executable"], "Host guard identity differs")
    native = {}
    for phase in phases:
        command = (recipe.get("commands") or {}).get(phase["phase"])
        _need(isinstance(command, list) and bool(command)
              and all(isinstance(arg, str) and bool(arg) and "\0" not in arg for arg in command),
              "Native phase requires its actual immutable recipe command")
        job = _row(state, connection, "native_job", phase.get("nativeJobId"))
        _need(job["payload"].get("role") == "phase" and job["payload"].get("guardNativeJobId") == guard_id,
              "Native phase lacks its original enclosing Job")
        _native(job, eid, phase.get("terminalProof"))
        _need(phase.get("commandDigest") == digest(command)
              and job["payload"]["completeProof"]["identity"]["commandDigest"] == hashlib.sha256("\0".join(command).encode("utf-8")).hexdigest(),
              "Native command differs from original recipe scope")
        _need(job["key"] not in native, "Duplicate native phase Job")
        native[job["key"]] = job
    known = {*native, guard_id}
    _need({r["key"] for r in state.list("native_job", connection=connection)
           if r["payload"].get("executionId") == eid or r["payload"].get("owner") == eid} == known,
          "Additional or unknown native work is protected")
    _need(digest(value.get("terminalProof")) == digest(phases[-1]["terminalProof"])
          and digest(host["payload"].get("terminalProof")) == digest(phases[-1]["terminalProof"]),
          "Execution terminal proof differs from its final native phase")
    discovery = phases[1].get("discovery")
    _need(isinstance(discovery, dict) and discovery.get("source") == "cargo-test-list"
          and type(discovery.get("testCount")) is int and discovery["testCount"] > 0
          and isinstance(discovery.get("tests"), list) and len(discovery["tests"]) == discovery["testCount"]
          and len(set(discovery["tests"])) == len(discovery["tests"])
          and all(isinstance(name, str) and bool(name) for name in discovery["tests"]),
          "Actual nonempty native test discovery is required")
    required_tests = list(recipe.get("requiredTests") or [])
    if recipe.get("originalFailure"):
        required_tests.append(recipe["originalFailure"])
    _need(all(isinstance(name, str) and bool(name) and any(found == name or found.endswith("::" + name)
              for found in discovery["tests"]) for name in required_tests), "Native discovery does not cover original required tests")
    return value, recipe, native, [execution, plan, host, launch, guard, *native.values()]


def _physical_source(root, sealed, manifest):
    entries = manifest.get("entries")
    _need(isinstance(entries, list) and bool(entries) and not manifest.get("external"),
          "This source closeout requires a complete local sealed closure")
    approved = canonical_build_root(sealed.get("buildRoot"))
    objects = physical_path_under(approved, sealed.get("objectRoot", ""))
    _need(objects in {approved.namespace(), approved.path / "zircon-jenkins"}, "Original source object root differs")
    hashes = {}
    for entry in entries:
        _need(isinstance(entry, dict), "Malformed source manifest entry")
        path = canonical_path(root, entry.get("path"))
        _need(path not in hashes and entry.get("status") in {"present", "deleted"}, "Source closure is ambiguous")
        checksum = entry.get("sha256")
        if entry["status"] == "present":
            require_digest(checksum, "sealed source sha256")
            _need(entry.get("objectDigest") == checksum, "Sealed source object differs")
            obj = physical_path_under(approved, objects / "inputs/objects" / checksum)
            _need(obj.is_file() and file_digest(obj) == checksum, "Original source object is absent or corrupt")
        else:
            _need(checksum is None, "Deleted source entry has bytes")
        _need(_before_hash(root, path) == checksum, "Current physical source differs from accepted after hash")
        hashes[path] = checksum
    return hashes


def release_accepted_claims(state, payload, *, identity, issuer_driver_digest):
    """Public flow/source-release implementation; no source bytes are written."""
    require_digest(issuer_driver_digest, "closeout issuer driver")
    for field in ("repositoryId", "sessionId", "requestId", "attemptId"):
        identifier(identity.get(field), field)
    release_id = identifier(payload.get("releaseRequestId"), "releaseRequestId")
    _need(release_id != identity.get("requestId"), "Closeout requires its own request ID")
    source = require_digest(payload.get("sourceDigest"), "sourceDigest")
    acceptance_digest = require_digest(payload.get("acceptanceReceiptDigest"), "acceptanceReceiptDigest")
    _need(source == identity.get("sourceInputDigest"), "Closeout cannot replace original source identity")
    pins = payload.get("expectedClaims")
    _need(isinstance(pins, list) and bool(pins), "Exact claim CAS pins are required")
    for pin in pins:
        _need(isinstance(pin, dict) and set(pin) == {"key", "version", "payloadDigest"}
              and type(pin.get("version")) is int and pin["version"] > 0 and isinstance(pin.get("key"), str),
              "Malformed claim CAS pin")
        require_digest(pin.get("payloadDigest"), "claim payloadDigest")
    _need(len({p["key"] for p in pins}) == len(pins), "Duplicate claim pin")
    contract = {"identity": identity, "releaseRequestId": release_id, "sourceDigest": source,
                "acceptanceReceiptDigest": acceptance_digest, "buildRoot": payload.get("buildRoot"),
                "expectedClaims": sorted(pins, key=lambda p: p["key"])}
    operation_id = digest({"kind": "source-claim-closeout", "repositoryId": identity.get("repositoryId"),
                           "sessionId": identity.get("sessionId"), "releaseRequestId": release_id})
    key = ":".join(identity[k] for k in ("sessionId", "requestId", "attemptId"))
    with state.transaction() as connection:
        workflow = _row(state, connection, "workflow", key)
        flow = workflow["payload"]
        _need(digest(flow.get("identity")) == digest(identity) and flow.get("status") == "accepted"
              and flow.get("template") in TEMPLATES, "Only the exact accepted source flow can release its claims")
        root = Path(flow.get("repoRoot", "")).absolute()
        repo, owner = identity.get("repositoryId"), identity.get("attemptId")
        _need(repo == repository_id(root) and flow.get("repositoryId") == repo, "Original physical repository identity differs")
        sealed_row = _row(state, connection, "sealed_input", source)
        sealed = sealed_row["payload"]
        manifest = sealed.get("sourceManifest") or sealed.get("manifest")
        _need(sealed.get("status") == "sealed" and sealed.get("sourceDigest") == source
              and sealed.get("repositoryId") == repo and sealed.get("repositoryRoot") == str(root)
              and sealed.get("owner") == owner and sealed.get("sessionId") == identity["sessionId"]
              and sealed.get("buildRoot") == contract["buildRoot"] and isinstance(manifest, dict)
              and flow.get("buildRoot") == contract["buildRoot"] and sealed.get("coverageDigest") == identity["coverageDigest"]
              and digest(flow.get("coverage")) == digest(sealed.get("coverage"))
              and compute_source_digest(manifest) == source and digest(flow.get("sourceManifest")) == digest(manifest),
              "Original source authorship or immutable closure differs")
        scope = canonical_paths(root, [e["path"] for e in manifest["entries"]])
        owned = canonical_paths(root, flow.get("ownedPaths", []))
        _need(canonical_paths(root, sealed.get("ownedPaths", [])) == owned, "Source write ownership differs")
        grants = [state.check_authorization(repo, identity["sessionId"], action, scope, connection=connection)
                  for action in ("implementation", "validation")]
        request = state.get_request(repo, identity["sessionId"], identity["requestId"], connection=connection)
        _need(request is not None and type(request.get("generation")) is int
              and request["generation"] == identity["generation"]
              and all(request["payload"].get(k) == v for k, v in identity.items())
              and all(g["snapshotDigest"] == request["payload"].get("authorizationSnapshot") for g in grants),
              "Original user authorization or source request binding changed")
        consumer = _row(state, connection, "sealed_consumer", _consumer_key(repo, identity["sessionId"], owner, source))
        _need(consumer["payload"].get("status") == "active" and consumer["payload"].get("manifestDigest") == source,
              "Original sealed consumer must be retained, not cancelled")
        acceptance = flow.get("acceptance")
        _need(isinstance(acceptance, dict), "Authoritative acceptance is missing")
        accepted = _row(state, connection, "acceptance", acceptance.get("acceptanceId"))
        _need(digest(accepted["payload"]) == digest(acceptance) and acceptance.get("receiptDigest") == acceptance_digest,
              "Original authoritative acceptance pin differs")
        validate_acceptance_receipt(acceptance)
        driver, coverage = identity["stageImplementationDigest"], identity["coverageDigest"]
        original = {"sourceDigest": source, "coverageDigest": coverage, "driverDigest": driver,
                    "generation": identity["generation"], "beforePatch": flow.get("beforePatch"), "dagDigest": flow.get("dagDigest")}
        required = flow.get("requiredStages")
        stages = list(WORKFLOW_TEMPLATES[flow["template"]])
        _need(isinstance(required, list) and bool(required) and len(set(required)) == len(required)
              and required == [s for s in stages if s not in {"acceptance", "commit", "notify", "gc"}]
              and flow.get("dagDigest") == digest(stages)
              and acceptance.get("stages") == required and digest(acceptance.get("flowIdentity")) == digest(original)
              and acceptance.get("acceptanceId") == digest({"flow": key, "identity": original, "stages": required})
              and flow.get("sourceDigest") == source and flow.get("coverageDigest") == coverage
              and flow.get("driverDigest") == driver and flow.get("generation") == identity["generation"],
              "Accepted DAG, coverage, source, or old driver identity differs")
        prior = state.get("source_claim_closeout", operation_id, connection=connection)
        if prior is not None:
            receipt = prior["payload"]
            _need(receipt.get("inputDigest") == digest(contract)
                  and receipt.get("kind") == "jenkins.source-claim-closeout-receipt" and receipt.get("status") == "complete"
                  and receipt.get("operationId") == operation_id and receipt.get("sourceClaimsReleased") is True
                  and receipt.get("receiptDigest") == digest({k: v for k, v in receipt.items() if k != "receiptDigest"}),
                  "Closeout request ID was reused with different input")
            return response("complete", request_id=release_id, receipt=receipt, reused=True)
        heavy = any(stage in HEAVY_STAGES for stage in required)
        evidence = [workflow, sealed_row, consumer, accepted]
        native = {}
        if heavy:
            execution, recipe, native, rows = _native_execution(state, connection, flow, source, coverage, driver)
            _need(recipe.get("requiredTestStages") == [s for s in required if s != "build" and s in HEAVY_STAGES],
                  "Managed recipe test stage contract differs from accepted DAG")
            _need(sorted(recipe.get("requiredTests") or []) == sorted((flow.get("coverage") or {}).get("requiredTests") or []),
                  "Native required tests differ from authoritative source coverage")
            evidence.extend(rows)
        else:
            _need(flow["template"] == "comments_only" and required == ["patch", "syntax", "format"],
                  "Only the comments-only light DAG can omit compiler proof")
        last = None
        for stage in required:
            ref = flow.get("stageReceipts", {}).get(stage)
            attempt = _row(state, connection, "validation_attempt", ref)
            receipt_row = _row(state, connection, "validation_receipt", ref)
            receipt = attempt["payload"].get("receipt")
            _need(attempt["payload"].get("status") == "passed" and isinstance(receipt, dict)
                  and digest(receipt_row["payload"]) == digest(receipt), "Stage receipt lacks original producer authority")
            expected = {"executionId": ref, "stage": stage, "sourceDigest": source, "coverageDigest": coverage,
                        "driverDigest": driver, "recipeDigest": flow.get("stageRecipeDigests", {}).get(stage)}
            require_digest(expected["recipeDigest"], "immutable stage recipe digest")
            _need(all(attempt["payload"].get(k) == v for k, v in expected.items()), "Stage attempt identity differs")
            validate_producer_receipt(receipt, expected=expected, output_root=receipt.get("outputClosure", {}).get("root"),
                                      required_outputs=flow.get("requiredOutputsByStage", {}).get(stage, flow.get("requiredOutputs", [])))
            if stage in HEAVY_STAGES:
                phase_name = "compile" if stage == "build" else "test"
                expected_attempt = f"{flow['executionId']}:compile" if stage == "build" else f"{flow['executionId']}:test:{stage}"
                _need(ref == expected_attempt, "Required stage does not identify its original execution phase")
                phase = next(p for p in execution["phaseRecords"] if p["phase"] == phase_name)
                job = native[phase["nativeJobId"]]["payload"]
                approved = canonical_build_root(flow["buildRoot"])
                phase_root = physical_path_under(approved, Path(execution.get("phaseOutputRoot", "")) / phase_name, allow_missing=False)
                _need(physical_path_under(approved, attempt["payload"].get("outputRoot", ""), allow_missing=False) == phase_root
                      and physical_path_under(approved, receipt["outputClosure"].get("root", ""), allow_missing=False) == phase_root
                      and receipt.get("requiredOutputGlobs", []) == attempt["payload"].get("requiredOutputGlobs", []),
                      "Producer output root differs from its registered native phase")
                _need(attempt["payload"].get("nativeJobId") == job["nativeJobId"]
                      and attempt["payload"].get("birthToken") == job["birthToken"]
                      and attempt["payload"].get("phase") == phase_name
                      and receipt["terminalProof"].get("nativeJobId") == job["nativeJobId"]
                      and type(receipt["terminalProof"].get("pid")) is int and receipt["terminalProof"]["pid"] == job["pid"]
                      and receipt["terminalProof"].get("birthToken") == job["birthToken"], "Producer receipt native identity differs")
                trusted = job["completeProof"]
                _need(all(digest(receipt["terminalProof"].get(k)) == digest(v) for k, v in trusted.items()),
                      "Producer native proof differs from complete collector proof")
            last = receipt
            evidence.extend([attempt, receipt_row])
        validate_acceptance_receipt(acceptance, producer_receipt=last, output_root=last["outputClosure"]["root"])
        operations = []
        after = {}
        original_before = {}
        _need(flow.get("patchOperationRefs") == sealed.get("patchOperationRefs") and bool(flow.get("patchOperationRefs")),
              "Original source patch journals are required")
        for ref in flow["patchOperationRefs"]:
            op = state.get_operation(ref, connection=connection)
            _need(op is not None and op.get("kind") == "source_patch" and op.get("status") == "complete"
                  and op["payload"].get("owner") == owner and op["payload"].get("repositoryRoot") == str(root),
                  "Source patch journal is incomplete or foreign")
            after.update((op.get("result") or {}).get("afterHashes", {}))
            for path, checksum in op["payload"].get("beforeHashes", {}).items():
                original_before.setdefault(path, checksum)
            operations.append(op)
        _need(sorted(after) == owned, "Source journal and exact owned paths differ")
        for r in connection.execute("SELECT kind,status,payload FROM operations WHERE status NOT IN ('complete','aborted')"):
            q = json.loads(r["payload"])
            _need(q.get("owner") != owner and (q.get("identity") or {}).get("attemptId") != owner,
                  "Unresolved source work still owns this attempt")
        rows = state.list("path_claims", connection=connection)
        claims = [r for r in rows if r["payload"].get("repositoryId") == repo and r["payload"].get("owner") == owner]
        expected = {p["key"]: p for p in pins}
        _need({r["key"] for r in claims} == set(expected) and sorted(r["payload"].get("path") for r in claims) == owned,
              "Only all exact claims of this accepted source may be released")
        sealed_claims = {c["path"]: c for c in sealed.get("claims", [])}
        _need(sorted(sealed_claims) == owned and len(sealed_claims) == len(sealed.get("claims", [])),
              "Original sealed claims do not exactly cover source ownership")
        for row in claims:
            q, pin = row["payload"], expected[row["key"]]
            sealed_claim = sealed_claims.get(q["path"], {})
            _need(type(row["version"]) is int and row["version"] == pin["version"]
                  and row["key"] == f"{repo}:{q['path']}" and q.get("repositoryRoot") == str(root)
                  and digest(q) == pin["payloadDigest"] and type(q.get("epoch")) is int and q["epoch"] > 0
                  and type(sealed_claim.get("epoch")) is int and type(sealed_claim.get("version")) is int
                  and q.get("epoch") == sealed_claim.get("epoch") and row["version"] == sealed_claim.get("version")
                  and q.get("owner") == sealed_claim.get("owner") and q.get("beforeHash") == sealed_claim.get("before_hash")
                  and q.get("status") == "applied" and q.get("beforeHash") == after[q["path"]]
                  and q.get("afterHash") == after[q["path"]] and q["path"] in original_before
                  and q.get("originalBeforeHash") == original_before[q["path"]], "Claim owner, epoch, version, or accepted hash differs")
        for row in rows:
            q = row["payload"]
            _need(q.get("repositoryId") != repo or q.get("owner") == owner
                  or not any(_overlaps(q.get("path", ""), p) for p in owned), "An overlapping foreign claim must be preserved")
        hashes = _physical_source(root, sealed, manifest)
        _need(all(hashes.get(p) == h for p, h in after.items()), "Accepted patch after hashes differ from sealed closure")
        receipt = {"schemaVersion": 1, "kind": "jenkins.source-claim-closeout-receipt", "status": "complete",
                   "operationId": operation_id, "releaseRequestId": release_id, "inputDigest": digest(contract),
                   "identity": identity, "sourceDigest": source, "acceptanceReceiptDigest": acceptance_digest,
                   "acceptedDriverDigest": driver, "closeoutIssuerDriverDigest": issuer_driver_digest,
                   "authorizationSnapshot": grants[0]["snapshotDigest"], "claims": claims, "currentHashes": hashes,
                   "authorityRowsDigest": digest(evidence), "patchOperationsDigest": digest(operations),
                   "sourceClaimsReleased": True, "consumerReferencesRetained": True, "sourceObjectsRetained": True,
                   "artifactReferencesRetained": True, "acceptanceRetained": True, "sourceBytesWritten": False}
        receipt["receiptDigest"] = digest(receipt)
        # Re-read bytes immediately before CAS; source-writing peers cannot claim
        # these paths while this transaction still owns the exact claim rows.
        _need(_physical_source(root, sealed, manifest) == hashes, "Physical source changed before closeout CAS")
        for row in claims:
            state.delete("path_claims", row["key"], expected_version=row["version"], connection=connection)
        state.put("source_claim_closeout", operation_id, receipt, expected_version=0, connection=connection)
        state.event("accepted_source_claims_released", {"operationId": operation_id, "releaseRequestId": release_id,
                    "owner": owner, "repositoryId": repo, "paths": owned, "receiptDigest": receipt["receiptDigest"]}, connection=connection)
    return response("complete", request_id=release_id, receipt=receipt, reused=False)

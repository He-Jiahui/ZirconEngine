"""Coordinator-owned union sealing and explicit per-author union consumption."""
from __future__ import annotations

from collections.abc import Mapping
from pathlib import Path

from ..contracts import JenkinsError, canonical_json, digest, file_digest, require_digest
from ..resources import canonical_build_root, physical_path_under
from ..source import _atomic_write, _consumer_key, canonical_path, compute_source_digest
from ..validation import TrustedReceiptAuthority
from ..validation.receipts import HEAVY_STAGES, validate_acceptance_receipt, validate_producer_receipt

CONSUMPTION_KIND = "jenkins.joint-author-consumption"
CONSUMPTION_DOMAIN = "joint_author_consumption"


def _covered(path, scopes):
    return any(path == scope or path.startswith(scope + "/") for scope in scopes)


def _authors(items):
    return [{"sessionId": item["sessionId"], "requestId": item["requestId"],
             "attemptId": item["attemptId"], "generation": item["generation"],
             "sourceDigest": item["sourceDigest"], "coverageDigest": item["coverageDigest"],
             "paths": item["paths"], **({"identity": item["identity"],
                 "requestPayloadDigest": item["requestPayloadDigest"],
                 "authorizationSnapshot": item["authorizationSnapshot"]} if "identity" in item else {})}
            for item in items]


def merge_joint_inputs(state, items, *, connection=None):
    records = [state.get("sealed_input", item["sourceDigest"], connection=connection)["payload"] for item in items]
    roots = {str(record.get("repositoryRoot") or record.get("repoRoot")) for record in records}
    object_roots = {str(record.get("objectRoot")) for record in records}
    if len(roots) != 1 or len(object_roots) != 1 or "None" in roots | object_roots:
        raise JenkinsError("joint_storage_conflict", "Joint sources must share one proven checkout and object root")
    claimed = []
    for item in items:
        for path in item["paths"]:
            if any(path == prior or path.startswith(prior + "/") or prior.startswith(path + "/")
                   for prior, _ in claimed):
                raise JenkinsError("group_path_conflict", "Joint authors have overlapping source ownership",
                                   details={"path": path})
            claimed.append((path, item["sessionId"]))
    selected, owned_entries, external = {}, {}, {}
    dependencies, required_tests, packages = set(), set(), set()
    for item, record in zip(items, records):
        manifest = record.get("sourceManifest") or record["manifest"]
        required_tests.update(item["coverage"].get("requiredTests", []))
        packages.update(item["coverage"].get("packages", []))
        dependencies.update(manifest.get("declaredDependencies", []))
        seen, covered = set(), set()
        for entry in manifest["entries"]:
            if not isinstance(entry, Mapping):
                raise JenkinsError("joint_manifest_invalid", "Joint input entries must be source records")
            path = canonical_path(next(iter(roots)), entry["path"])
            if path in seen:
                raise JenkinsError("joint_manifest_invalid", "Author manifest has duplicate path identities")
            seen.add(path)
            if _covered(path, item["paths"]):
                owned_entries[path] = dict(entry)
                covered.update(scope for scope in item["paths"] if _covered(path, [scope]))
            selected.setdefault(path, []).append(dict(entry))
        if covered != set(item["paths"]):
            raise JenkinsError("joint_owned_source_missing", "Author input lacks part of its owned source scope")
        for value in manifest.get("external", []):
            if (not isinstance(value, Mapping) or value.get("kind") != "sealed-external-source"
                    or value.get("name") != "zr_vm" or not isinstance(value.get("manifest"), Mapping)
                    or value["manifest"].get("external") or compute_source_digest(value["manifest"]) != value.get("sourceDigest")):
                raise JenkinsError("joint_external_invalid", "External union dependencies require a verified fixed source")
            name = value["name"]
            if name in external and external[name] != value:
                raise JenkinsError("joint_external_conflict", "Authors disagree on the same external dependency")
            external[name] = dict(value)
    entries = []
    for path, candidates in sorted(selected.items()):
        if any(_covered(path, item["paths"]) for item in items):
            if path not in owned_entries:
                raise JenkinsError("joint_owned_source_missing", "Author input lacks its owned source")
            entry = owned_entries[path]
        else:
            if len({digest(candidate) for candidate in candidates}) != 1:
                raise JenkinsError("joint_dependency_conflict", "Authors disagree on an unowned dependency",
                                   details={"path": path})
            entry = candidates[0]
        entries.append(entry)
    entries.sort(key=lambda entry: entry["path"])
    authors = _authors(items)
    # Original input refs distinguish attributed unions even when file bytes
    # coincide; consumer identities remain explicit in the same frozen manifest.
    manifest = {"schemaVersion": 1, "repositoryId": items[0]["repositoryId"],
                "baseHead": items[0]["baseHead"], "preparationKind": "joint-input",
                "entries": entries, "external": [external[name] for name in sorted(external)],
                "declaredDependencies": sorted(dependencies), "authorInputs": authors}
    source = compute_source_digest(manifest)
    coverage = {"template": "cross_module", "sourceDigest": source,
                "ownedPaths": sorted(path for path, _ in claimed), "requiredTests": sorted(required_tests),
                "features": sorted(items[0]["coverage"].get("features", [])),
                "noDefaultFeatures": bool(items[0]["coverage"].get("noDefaultFeatures", False)),
                "requiredStages": ["build", "unit_test", "integration_test"],
                "profile": items[0]["profile"], "authorInputs": authors,
                "validationIdentity": {name: items[0].get(name) for name in
                    ("toolchainDigest", "lockDigest", "compilerIdentity", "wrapperDigest",
                     "environmentPolicyDigest", "resourceProfile", "buildRoot")}}
    if packages:
        coverage["packages"] = sorted(packages)
    coverage["coverageDigest"] = digest(coverage)
    dependency_entries = [entry for entry in entries if not any(_covered(
        canonical_path(next(iter(roots)), entry["path"]), item["paths"]) for item in items)]
    dependency_digest = digest({"entries": dependency_entries, "external": manifest["external"],
                                "declaredDependencies": manifest["declaredDependencies"]})
    payload = {"schemaVersion": 2, "status": "merged", "sourceDigest": source, "sourceManifest": manifest,
               "coverage": coverage, "coverageDigest": coverage["coverageDigest"],
               "repositoryId": items[0]["repositoryId"], "repositoryRoot": next(iter(roots)),
               "objectRoot": next(iter(object_roots)), "buildRoot": items[0]["buildRoot"],
               "ownedPaths": coverage["ownedPaths"], "authors": authors, "unionDependencyDigest": dependency_digest,
               "patchOperationRefs": list(dict.fromkeys(ref for record in records for ref in record.get("patchOperationRefs", [])))}
    old = state.get("joint_input", source, connection=connection)
    if old and old["payload"] != payload:
        raise JenkinsError("joint_input_conflict", "Joint input cannot replace its authored manifest")
    if not old:
        state.put("joint_input", source, payload, expected_version=0, connection=connection)
    return payload


def verify_joint_identity(joint):
    """Check the frozen source, coverage, ownership and dependency attribution."""
    manifest, coverage = joint.get("sourceManifest"), joint.get("coverage")
    if (not isinstance(manifest, Mapping) or compute_source_digest(manifest) != joint.get("sourceDigest")
            or manifest.get("preparationKind") != "joint-input" or manifest.get("repositoryId") != joint.get("repositoryId")
            or manifest.get("authorInputs") != joint.get("authors")):
        raise JenkinsError("joint_manifest_corrupt", "Joint manifest differs from its frozen identity")
    if (not isinstance(coverage, Mapping) or coverage.get("sourceDigest") != joint["sourceDigest"]
            or coverage.get("coverageDigest") != joint.get("coverageDigest")
            or digest({name: value for name, value in coverage.items() if name != "coverageDigest"}) != joint["coverageDigest"]
            or coverage.get("authorInputs") != joint.get("authors") or coverage.get("ownedPaths") != joint.get("ownedPaths")):
        raise JenkinsError("joint_coverage_corrupt", "Union coverage or author attribution differs from its frozen identity")
    authors = joint.get("authors")
    if not isinstance(authors, list) or not authors or not all(isinstance(author, Mapping) for author in authors):
        raise JenkinsError("joint_manifest_corrupt", "Union lacks its original author attribution")
    scope = sorted(path for author in authors for path in author.get("paths", []))
    if scope != joint.get("ownedPaths"):
        raise JenkinsError("joint_manifest_corrupt", "Union source ownership differs from its original authors")
    dependencies = [entry for entry in manifest["entries"] if not _covered(
        canonical_path(joint["repositoryRoot"], entry["path"]), scope)]
    if digest({"entries": dependencies, "external": manifest.get("external", []),
               "declaredDependencies": manifest.get("declaredDependencies", [])}) != joint.get("unionDependencyDigest"):
        raise JenkinsError("joint_manifest_corrupt", "Union dependency attribution differs from its frozen identity")


def verify_joint_objects(joint):
    verify_joint_identity(joint)
    manifest = joint["sourceManifest"]
    approved = canonical_build_root(joint["buildRoot"])
    objects = physical_path_under(approved, joint["objectRoot"], allow_missing=False)
    if objects != physical_path_under(approved, approved.namespace(), allow_missing=False):
        raise JenkinsError("joint_storage_conflict", "Joint objects must use the registered coordinator store")
    for entry in manifest["entries"]:
        canonical_path(joint["repositoryRoot"], entry.get("path", ""))
        if entry.get("status") == "deleted":
            if entry.get("objectDigest") is not None or entry.get("sha256") is not None:
                raise JenkinsError("joint_object_corrupt", "Deleted source cannot carry object bytes")
            continue
        checksum = require_digest(entry.get("objectDigest"), "objectDigest")
        if (entry.get("status") != "present" or not isinstance(entry.get("sha256"), str)
                or entry.get("kind") != "directory" and entry.get("sha256") != checksum):
            raise JenkinsError("joint_object_corrupt", "Joint source object differs from its content identity")
        obj = physical_path_under(approved, objects / "inputs" / "objects" / checksum, allow_missing=False)
        if not obj.is_file() or file_digest(obj) != checksum:
            raise JenkinsError("joint_object_corrupt", "Joint source object is unavailable or corrupt")
    from ..source.external_inputs import verify_objects
    verify_objects(manifest, objects)
    return approved, objects


def seal_joint_input(state, group, *, session_id, owner, identity=None, connection=None):
    record = state.get("joint_input", group.joint_manifest_digest, connection=connection)
    if not record:
        raise JenkinsError("joint_input_missing", "Joint group lacks its frozen manifest")
    joint = record["payload"]
    repo = joint["repositoryId"]
    for author in joint["authors"]:
        state.check_authorization(repo, author["sessionId"], "validation", author["paths"], connection=connection)
    from .groups import _external_scope
    state.check_authorization(repo, session_id, "validation",
                              [*joint["ownedPaths"], *_external_scope(joint["sourceManifest"])], connection=connection)
    approved, objects = verify_joint_objects(joint)
    manifest_path = physical_path_under(approved, objects / "inputs" / (joint["sourceDigest"] + ".json"))
    if manifest_path.exists() and file_digest(manifest_path) != joint["sourceDigest"]:
        raise JenkinsError("joint_manifest_corrupt", "Joint manifest bytes differ from its canonical identity")
    if not manifest_path.exists():
        _atomic_write(manifest_path, canonical_json(joint["sourceManifest"]))
    payload = {**joint, "status": "sealed", "manifest": joint["sourceManifest"], "manifestDigest": joint["sourceDigest"],
               "owner": owner, "sessionId": session_id, "claims": [], "jointGroupId": group.group_id}
    if identity:
        payload["identity"] = dict(identity)
    old = state.get("sealed_input", joint["sourceDigest"], connection=connection)
    if old and old["payload"] != payload:
        raise JenkinsError("joint_seal_conflict", "Joint sealing is immutable")
    if not old:
        state.put("sealed_input", joint["sourceDigest"], payload, expected_version=0, connection=connection)
    key = _consumer_key(repo, session_id, owner, joint["sourceDigest"])
    consumer = {"repositoryId": repo, "sessionId": session_id, "owner": owner,
                "sourceDigest": joint["sourceDigest"], "manifestDigest": joint["sourceDigest"], "status": "active"}
    registered = state.get("sealed_consumer", key, connection=connection)
    if registered and registered["payload"] != consumer:
        raise JenkinsError("sealed_consumer_mismatch", "Joint consumer reference is immutable")
    if not registered:
        state.put("sealed_consumer", key, consumer, expected_version=0, connection=connection)
    return payload


def _native_evidence(state, execution_id, *, connection=None):
    row = state.get("execution", execution_id, connection=connection)
    execution = row["payload"] if row else {}
    phases = execution.get("phaseRecords")
    if (execution.get("status") != "passed" or execution.get("cancelRequested")
            or not isinstance(phases, list) or not phases):
        raise JenkinsError("joint_native_untrusted", "Union requires a passed execution with complete native phase evidence")
    result, seen = [], set()
    for phase in phases:
        native_id = phase.get("nativeJobId")
        row = state.get("native_job", native_id, connection=connection)
        native = row["payload"] if row else {}
        proof = native.get("completeProof")
        if (native_id in seen or native.get("nativeJobId") != native_id or native.get("status") != "terminal"
                or native.get("owner") != execution_id or native.get("executionId") != execution_id
                or type(native.get("pid")) is not int or native["pid"] <= 0 or not native.get("birthToken")
                or not isinstance(proof, dict) or proof.get("complete") is not True
                or type(proof.get("processExitCode")) is not int or proof["processExitCode"] != 0 or proof.get("stdoutEof") is not True
                or proof.get("stderrEof") is not True or proof.get("childrenGone") is not True
                or proof != phase.get("terminalProof")):
            raise JenkinsError("joint_native_untrusted", "Every union phase requires its original complete exit and EOF proof")
        seen.add(native_id)
        result.append({"phase": phase.get("phase"), "nativeJobId": native_id, "pid": native["pid"],
                       "birthToken": native["birthToken"], "proofDigest": digest(proof)})
    for row in state.list("native_job", connection=connection):
        if row["payload"].get("executionId") == execution_id and row["payload"].get("status") != "terminal":
            raise JenkinsError("joint_native_untrusted", "An enclosing union native job remains active")
    return execution, result


def verify_joint_acceptance(state, joint, reference, *, repo_root=None, consumer_identity=None, connection=None):
    verify_joint_identity(joint)
    if not isinstance(reference, Mapping) or not isinstance(reference.get("receiptRef"), str):
        raise JenkinsError("joint_acceptance_untrusted", "Joint acceptance requires a registered collector receipt reference")
    receipt_id = reference["receiptRef"]
    row = state.get("acceptance", receipt_id, connection=connection)
    if not row:
        raise JenkinsError("joint_acceptance_untrusted", "Joint receipt is not registered by the acceptance authority")
    acceptance = validate_acceptance_receipt(row["payload"])
    flow_id = reference.get("flowId") or row["payload"].get("flowId")
    if not flow_id and consumer_identity:
        flow_id = ":".join(consumer_identity[name] for name in ("sessionId", "requestId", "attemptId"))
    workflow = state.get("workflow", flow_id, connection=connection) if isinstance(flow_id, str) else None
    if not workflow or workflow["payload"].get("status") != "accepted":
        raise JenkinsError("joint_flow_unaccepted", "Joint receipt requires its authoritative accepted workflow")
    contract = workflow["payload"]
    if (acceptance.get("sourceDigest") != joint["sourceDigest"]
            or acceptance.get("coverageDigest") != joint["coverageDigest"]
            or contract.get("sourceDigest") != joint["sourceDigest"]
            or contract.get("coverageDigest") != joint["coverageDigest"]
            or consumer_identity and contract.get("identity") != dict(consumer_identity)):
        raise JenkinsError("joint_acceptance_mismatch", "Union receipt must bind the complete merged source, coverage and consumer")
    if consumer_identity and flow_id != ":".join(consumer_identity[name] for name in ("sessionId", "requestId", "attemptId")):
        raise JenkinsError("joint_acceptance_mismatch", "Union workflow must preserve its original request identity")
    authority = TrustedReceiptAuthority(state, repo_root=repo_root)
    if connection is None:
        with state.transaction() as current:
            return verify_joint_acceptance(state, joint, reference, repo_root=repo_root,
                                           consumer_identity=consumer_identity, connection=current)
    trusted = authority._accepted_flow(flow_id, {"flowId": flow_id}, connection)
    if trusted != acceptance or trusted.get("acceptanceId") != receipt_id:
        raise JenkinsError("joint_acceptance_mismatch", "Joint receipt differs from workflow authority")
    required = authority._flow_stages(contract)
    if not set(joint["coverage"].get("requiredStages", ["build", "unit_test", "integration_test"])) <= set(required):
        raise JenkinsError("joint_coverage_incomplete", "Union workflow omits part of its frozen required stage coverage")
    if not set(required) & HEAVY_STAGES:
        raise JenkinsError("joint_native_required", "Joint source acceptance requires complete managed native validation")
    execution, native = _native_evidence(state, contract.get("executionId"), connection=connection)
    from .execution import _execution_identity
    _execution_identity(state.get("execution", contract.get("executionId"), connection=connection), contract.get("executionId"))
    plan = state.get("recipe_plan", execution.get("recipeRef"), connection=connection)
    recipe = plan["payload"].get("recipe") if plan else None
    if (not isinstance(recipe, dict) or plan["payload"].get("recipeDigest") != execution.get("recipeDigest")
            or digest(recipe) != execution.get("recipeDigest")
            or plan["payload"].get("sourceRef") != joint["sourceDigest"]
            or plan["payload"].get("coverageRef") != joint["coverageDigest"]
            or sorted(recipe.get("features", [])) != sorted(joint["coverage"].get("features", []))
            or recipe.get("profile", "dev") != joint["coverage"].get("profile", "dev")
            or not set(joint["coverage"].get("packages", [])) <= set(recipe.get("packages", []))
            or not set(joint["coverage"].get("requiredTests", [])) <= set(recipe.get("requiredTests", []))):
        raise JenkinsError("joint_recipe_mismatch", "Union recipe differs from frozen source, features, profile or required tests")
    frozen = joint["coverage"].get("validationIdentity", {})
    commands = recipe.get("commands")
    if (not isinstance(commands, dict)
            or any(not isinstance(commands.get(phase), list) or not commands[phase]
                   or any(not isinstance(arg, str) for arg in commands[phase]) for phase in ("compile", "list", "test"))
            or any(("--no-default-features" in commands[phase]) != joint["coverage"].get("noDefaultFeatures", False)
                   for phase in ("compile", "list", "test"))
            or any(recipe.get(name) != frozen.get(name) for name in
                   ("toolchainDigest", "lockDigest", "compilerIdentity", "resourceProfile") if frozen.get(name) is not None)
            or frozen.get("environmentPolicyDigest") is not None
                   and recipe.get("environmentPolicyDigest") != frozen["environmentPolicyDigest"]
            or canonical_build_root(recipe.get("buildRoot")).path != canonical_build_root(frozen.get("buildRoot")).path):
        raise JenkinsError("joint_recipe_mismatch", "Union compiler, lock, root or default feature selection differs from its authors")
    environment = recipe.get("environment")
    if frozen.get("environmentPolicyDigest") is not None:
        wrappers = {name: str(environment.get(name, "")) for name in
                    ("RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_RUSTC_WRAPPER")} if isinstance(environment, dict) else None
        if wrappers is None or digest(wrappers) != frozen.get("wrapperDigest"):
            raise JenkinsError("joint_recipe_mismatch", "Union compiler wrappers differ from the frozen author policy")
    phases = execution["phaseRecords"]
    discovery = next((phase.get("discovery") for phase in phases if phase.get("phase") == "list"), None)
    if (not isinstance(discovery, dict) or discovery.get("source") != "cargo-test-list"
            or type(discovery.get("testCount")) is not int or discovery["testCount"] < 1
            or [phase.get("phase") for phase in phases] != recipe.get("phases")
            or any(not any(name == test or name.endswith("::" + test) for name in discovery.get("tests", []))
                   for test in joint["coverage"].get("requiredTests", []))):
        raise JenkinsError("joint_coverage_incomplete", "Native union discovery omits part of the frozen author test coverage")
    if any(phase.get("commandDigest") != digest(commands[phase["phase"]]) for phase in phases):
        raise JenkinsError("joint_native_untrusted", "Original native phase command differs from its immutable union recipe")
    for name in ("sourceDigest", "coverageDigest", "driverDigest"):
        if execution.get(name) != contract.get(name):
            raise JenkinsError("joint_execution_mismatch", "Joint workflow differs from its native execution")
    if str(execution.get("generation")) != str(contract.get("generation")):
        raise JenkinsError("joint_execution_mismatch", "Joint workflow differs from its native execution generation")
    collected, collector_refs = [], []
    for stage in required:
        ref = contract.get("stageReceipts", {}).get(stage)
        attempt = state.get("validation_attempt", ref, connection=connection) if isinstance(ref, str) else None
        registered = state.get("validation_receipt", ref, connection=connection) if isinstance(ref, str) else None
        producer = authority.get(ref, connection=connection) if isinstance(ref, str) else None
        if (not attempt or not registered or producer is None or registered["payload"] != producer
                or attempt["payload"].get("executionId") != ref or attempt["payload"].get("stage") != stage):
            raise JenkinsError("joint_collector_untrusted", "Union stage lacks its original authoritative collector")
        validate_producer_receipt(producer, expected={"executionId": ref, "stage": stage,
            "sourceDigest": joint["sourceDigest"], "coverageDigest": joint["coverageDigest"],
            "driverDigest": contract.get("driverDigest"), "recipeDigest": contract.get("stageRecipeDigests", {}).get(stage)},
            output_root=producer.get("outputClosure", {}).get("root"),
            required_outputs=contract.get("requiredOutputsByStage", {}).get(stage, contract.get("requiredOutputs", [])))
        if stage in HEAVY_STAGES:
            terminal = producer["terminalProof"]
            native_row = state.get("native_job", attempt["payload"].get("nativeJobId"), connection=connection)
            job = native_row["payload"] if native_row else {}
            proof = job.get("completeProof") or {}
            if (terminal.get("nativeJobId") != job.get("nativeJobId")
                    or terminal.get("nativeJobId") not in {item["nativeJobId"] for item in native}
                    or terminal.get("birthToken") != job.get("birthToken")
                    or terminal.get("birthToken") != attempt["payload"].get("birthToken")
                    or terminal.get("pid") != job.get("pid")
                    or any(terminal.get(key) != proof.get(key) for key in
                           ("complete", "processExitCode", "stdoutEof", "stderrEof", "childrenGone"))):
                raise JenkinsError("joint_collector_untrusted", "Union collector differs from its original native phase")
        collected.append(producer)
        collector_refs.append({"stage": stage, "receiptRef": ref, "receiptDigest": digest(producer)})
    validate_acceptance_receipt(acceptance, producer_receipt=collected[-1])
    evidence = acceptance.get("executionEvidence")
    formal = authority._consumer_outputs(flow_id, contract, connection)
    bundle = state.get("artifact_bundle", contract.get("executionId"), connection=connection)
    published = bundle["payload"] if bundle else {}
    if (not isinstance(evidence, dict) or evidence != formal or evidence.get("consumerId") != flow_id
            or evidence.get("executionId") != contract.get("executionId")
            or evidence.get("executionKeyDigest") != execution.get("executionKeyDigest")
            or evidence.get("bundleDigest") != published.get("bundleDigest")
            or published.get("status") != "published"
            or published.get("bundleDigest") != digest({"executionKey": published.get("executionKey"),
                 "identity": published.get("identity"), "files": published.get("files")})):
        raise JenkinsError("joint_formal_evidence_missing", "Union receipt lacks its exact formal publication evidence")
    result = {"receiptRef": receipt_id, "flowId": flow_id, "executionId": contract["executionId"],
              "acceptanceDigest": acceptance["receiptDigest"], "collectors": collector_refs,
              "nativeEvidence": native, "formalEvidence": evidence}
    result["evidenceDigest"] = digest(result)
    return result


def consume_joint_acceptance(groups, group_id, identity, *, expected_version=None):
    from .groups import _external_scope, _session, require_request
    state = groups.state
    record = groups.record(group_id)
    payload = record["payload"]
    author = next((item for item in payload["sessions"] if item["sessionId"] == identity["sessionId"]), None)
    if not author or author.get("identity") != dict(identity):
        raise JenkinsError("group_author_mismatch", "Consumption requires the exact original author request identity")
    sid = identity["sessionId"]
    if sid in payload.get("cancelledConsumers", []):
        raise JenkinsError("group_consumer_cancelled", "Cancelled group authors cannot consume union acceptance")
    if payload["mode"] != "joint" or payload["status"] != "ready" or not payload.get("jointReceiptRef"):
        raise JenkinsError("joint_acceptance_pending", "Union acceptance is still pending", retryable=True)
    key = digest({"groupId": group_id, "authorIdentity": dict(identity)})
    with state.transaction() as connection:
        current = groups.record(group_id, connection=connection)
        require_request(state, identity, author["paths"], connection=connection)
        old = state.get(CONSUMPTION_DOMAIN, key, connection=connection)
        if old:
            if (old["payload"].get("consumptionRef") != key
                    or old["payload"].get("authorIdentity") != dict(identity)
                    or current["payload"].get("consumptionRefs", {}).get(sid) != key
                    or old["payload"].get("receiptDigest") != digest({name: value for name, value in old["payload"].items()
                                                                   if name != "receiptDigest"})):
                raise JenkinsError("joint_consumption_conflict", "Historical author consumption is corrupt")
            return {**groups._projection(current).as_dict(), "status": "consumed", "receiptRef": key,
                    "consumption": old["payload"]}
    joint_row = state.get("joint_input", payload["jointManifestDigest"])
    joint = joint_row["payload"] if joint_row else None
    if not joint:
        raise JenkinsError("joint_input_missing", "Union source attribution is unavailable")
    verify_joint_objects(joint)
    with state.transaction() as connection:
        current = groups.record(group_id, connection=connection)
        groups._version(current, expected_version)
        if current["payload"].get("jointReceiptRef") != payload["jointReceiptRef"] or sid in current["payload"].get("cancelledConsumers", []):
            raise JenkinsError("group_version_conflict", "Group changed before author consumption", retryable=True)
        fresh = _session({"identity": dict(identity)}, state, strict=True, repo_root=groups.repo_root, connection=connection)
        if fresh != author:
            raise JenkinsError("group_author_mismatch", "Original source author binding changed")
        require_request(state, identity, [*author["paths"], *_external_scope(joint["sourceManifest"])], connection=connection)
        verified = verify_joint_acceptance(state, joint, {"receiptRef": payload["jointReceiptRef"], "flowId": payload["jointFlowId"]},
            repo_root=groups.repo_root, consumer_identity=payload.get("jointConsumer"), connection=connection)
        if verified["evidenceDigest"] != payload.get("jointEvidenceDigest"):
            raise JenkinsError("joint_acceptance_mismatch", "Frozen union collector evidence changed before consumption")
        consumption = {"schemaVersion": 1, "kind": CONSUMPTION_KIND, "status": "consumed",
            "acceptanceScope": "joint-union", "standaloneSourceAccepted": False, "consumptionRef": key,
            "groupId": group_id, "authorIdentity": dict(identity), "authorSourceDigest": author["sourceDigest"],
            "authorCoverageDigest": author["coverageDigest"], "authorPaths": author["paths"],
            "authorRequestPayloadDigest": author["requestPayloadDigest"], "unionSourceDigest": joint["sourceDigest"],
            "unionCoverageDigest": joint["coverageDigest"], "unionDependencyDigest": joint["unionDependencyDigest"],
            "authorInputs": joint["authors"], "unionAcceptance": verified}
        consumption["receiptDigest"] = digest(consumption)
        state.put(CONSUMPTION_DOMAIN, key, consumption, expected_version=0, connection=connection)
        updated = {**current["payload"], "acceptedConsumers": sorted([*current["payload"].get("acceptedConsumers", []), sid]),
                   "consumptionRefs": {**current["payload"].get("consumptionRefs", {}), sid: key}}
        saved = state.put(groups.DOMAIN, group_id, updated, expected_version=current["version"], connection=connection)
        return {**groups._projection(saved).as_dict(), "status": "consumed", "receiptRef": key, "consumption": consumption}

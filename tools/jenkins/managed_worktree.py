"""Jenkins preparation and reader lifetime for one reusable managed worktree."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path

from .contracts import JenkinsError, digest, require_digest
from .deployment.driver import _plain
from .source import PathClaim, _before_hash, _git_base_tree, _thaw, canonical_path, seal_candidate
from .source.external_inputs import authorization_paths, verify_objects
from .product_inputs import (COMPILER_CONTROL_PATHS, PROJECT_TEMPLATE_INPUTS, compiler_controls,
                             product_path, require_product_inputs, validate_hash_paths)
from .resources import canonical_build_root

from .managed_worktree_lifetime import (MUTATIONS, _pin, _pins_current, _require,
                                        assert_no_writer, writer_hold)

CAPABILITY = "managed-worktree-source-preparation-v1"


def _target_source(management, reference):
    key = require_digest(reference, "targetSourceOperationKey")
    row = management.state.get("management_operation", key)
    operation = row["payload"] if row else {}
    original, result = operation.get("payload", {}), operation.get("result", {})
    _require(operation.get("phase") == "complete"
             and operation.get("action") in {"create-worktree", "apply-patch", "prepare-worktree"}
             and original.get("repositoryId") == management.repo_id
             and original.get("driverDigest") == management.config["driverDigest"]
             and original.get("runtimeOperationId") == management.config["runtimeOperationId"]
             and result.get("worktreeId") == management.payload["worktreeId"]
             and result.get("operationKey") == key
             and key == digest({name: original.get(name) for name in
                                ("repositoryId", "sessionId", "requestId", "operationId")})
             and operation.get("inputDigest") == digest({k: v for k, v in original.items() if k != "observationOnly"})
             and result.get("status") == "complete"
             and all(result.get(k) == original.get(k) for k in ("sessionId", "requestId", "operationId",
                                                               "driverDigest", "runtimeOperationId"))
             and result.get("source", {}).get("sourceDigest") == digest(
                 {k: v for k, v in result["source"].items() if k != "sourceDigest"}),
             "managed_worktree_preimage_unproven", "Target needs its original completed source operation")
    return row, result["source"]


def _source_membership(management, base, prior):
    """Include staged delete/rename old paths from the immutable WT base."""
    names = management.git(management.repo, "ls-files", "--cached", "--others",
                           "--exclude-standard", "-z").stdout
    values = {canonical_path(management.repo, name.decode("utf-8", "strict"))
              for name in names.split(b"\0") if name}
    values.update(_git_base_tree(management.repo, base))
    values.update(canonical_path(management.repo, name) for name in prior["files"] if product_path(name.casefold()))
    values.update(PROJECT_TEMPLATE_INPUTS)
    values.update(COMPILER_CONTROL_PATHS)
    return {name for name in values if product_path(name)}


def _verify_canonical(management, head, hashes, base, prior):
    _require(management.text(management.repo, "rev-parse", "HEAD") == head,
             "managed_source_changed", "Canonical input HEAD changed during preparation")
    _require(_source_membership(management, base, prior) == set(hashes),
             "managed_source_membership_changed", "The complete canonical source membership changed")
    for path, checksum in hashes.items():
        target = management.repo / path
        _plain(target)
        _require(not target.exists() or target.is_file(),
                 "managed_source_directory", "Source preparation requires explicit ordinary files")
        _require(_before_hash(management.repo, path) == checksum,
                 "managed_source_changed", "Canonical source differs from its declared hash")


def _reader(management, record, hashes, external, *, connection=None):
    paths = sorted(list(hashes) + authorization_paths(external))
    reader = management.state.check_authorization(management.repo_id, management.session,
                                                  "validation", paths, connection=connection)
    writer = management.state.check_authorization(management.repo_id, record["sessionId"],
                                                  "implementation", record["ownedPaths"], connection=connection)
    validation = management.state.check_authorization(management.repo_id, record["sessionId"],
                                                      "validation", record["ownedPaths"], connection=connection)
    _require(reader["actions"] == ["validation"] and sorted(reader["paths"]) == paths
             and reader["principal"] == record.get("writerPrincipal") == writer["principal"]
             and writer["snapshotDigest"] == validation["snapshotDigest"] == record.get("writerAuthorizationSnapshot")
             and writer["actions"] == ["implementation", "validation"]
             and sorted(writer["paths"]) == sorted(record["ownedPaths"])
             and sorted(management.owned) == sorted(record["ownedPaths"])
             and management.session != record["sessionId"],
             "managed_source_authority_changed", "Preparation must retain its separate exact reader and narrow writer")
    return reader



def _external_capture(management, helper, *, inspection=False, payload=None):
    """Use the fixed management capture, including every explicit Gitlink."""
    payload = management.payload if payload is None else payload
    from .current_source import _hashes, _paths
    roots = payload.get("externalRoots")
    values = [None] * len(roots) if inspection and isinstance(roots, list) else payload.get("externalInputs")
    _require(isinstance(roots, list) and isinstance(values, list) and len(roots) == len(values)
             and len(roots) <= 1, "external_scope_invalid", "Declare the actual complete sibling input")
    result, captures, gitlinks = [], [], {}
    for raw_root, value in zip(roots, values):
        expected = management.repo.parent / "zr_vm"
        _plain(expected)
        _require(isinstance(raw_root, str) and Path(raw_root).is_absolute()
                 and Path(raw_root).absolute() == expected and expected.is_dir()
                 and expected.resolve() == expected
                 and (inspection or isinstance(value, dict)
                      and set(value) == {"name", "repositoryRoot", "baseHead", "currentHashes", "untrackedPaths"}
                      and value.get("name") == "zr_vm" and value.get("repositoryRoot") == str(expected))
                 and Path(management.text(expected, "rev-parse", "--show-toplevel")).absolute() == expected,
                 "external_repository_invalid", "External source must be the physical canonical zr_vm")
        if inspection:
            stage = management.git(expected, "ls-files", "--stage", "-z").stdout
            for row in stage.split(b"\0"):
                if row.startswith(b"160000 "):
                    fields, name = row.split(b"\t", 1)
                    relative = name.decode("utf-8", "strict")
                    child = expected / relative
                    _plain(child)
                    gitlinks[relative] = {"indexCommit": fields.decode("ascii").split()[1],
                                          "head": management.text(child, "rev-parse", "HEAD")}
            captured = management.source(expected, payload={**payload, "externalGitlinks": gitlinks})
            hashes = {name.casefold(): checksum for name, checksum in helper.source_files(captured).items()}
        else:
            hashes = _hashes(expected, value.get("currentHashes"))
            captured = management.source(expected, payload=payload)
        _require(not any(path.split("/", 1)[0] in {".git", ".codex", ".jenkins", ".opencode", "target"}
                         for path in hashes), "external_control_forbidden", "External checkout controls are not compiler inputs")
        actual = {name.casefold(): checksum for name, checksum in helper.source_files(captured).items()}
        _require(inspection or captured.get("head") == value.get("baseHead") and actual == hashes,
                 "external_capture_changed", "Complete external source differs from its hash-bound declared capture")
        untracked = []
        def selected(current_root, snapshot, prefix=""):
            cached = {name.decode("utf-8", "strict").casefold() for name in
                      management.git(current_root, "ls-files", "--cached", "-z").stdout.split(b"\0") if name}
            for name, checksum in snapshot["files"].items():
                if isinstance(checksum, dict):
                    selected(current_root / name, checksum["source"], prefix + name.casefold() + "/")
                elif checksum is not None and name.casefold() not in cached:
                    untracked.append(prefix + name.casefold())
        selected(expected, captured)
        declared = sorted(untracked) if inspection else _paths(expected, value.get("untrackedPaths"), "externalUntrackedPaths")
        _require(declared == sorted(untracked), "external_scope_invalid",
                 "External untracked membership must include every actual nested source")
        result.append({"name": "zr_vm", "repositoryRoot": str(expected), "baseHead": captured["head"],
                       "currentHashes": hashes, "untrackedPaths": declared})
        captures.append({"path": str(expected), "source": captured})
    return result, captures, gitlinks


def _seal_external(captures, helper, object_root):
    from .source.external_inputs import KIND
    result = []
    for item in captures:
        root, captured = Path(item["path"]), item["source"]
        claims = [PathClaim(path, "dependency", 0, checksum, 0)
                  for path, checksum in helper.source_files(captured).items()]
        sealed = seal_candidate(root, claims, object_root=object_root, base_head=captured["head"])
        manifest = _thaw(sealed.payload)
        helper.verify_sealed_capture(captured, manifest)
        result.append({"kind": KIND, "name": "zr_vm", "repositoryRoot": str(root),
                       "manifest": manifest, "sourceDigest": sealed.source_digest})
    return result




SOURCE_EVIDENCE_MAX_BYTES = 64 * 1024**2


def _inventory_reference(state, object_root, key, reference, worktree_id, repository_id, driver, runtime):
    key = require_digest(key, "inspectionOperationKey")
    row = state.get("management_operation", key)
    op = row["payload"] if row else {}
    original, result = op.get("payload", {}), op.get("result", {})
    _require(op.get("phase") == "complete" and op.get("action") == "inspect-worktree"
             and result.get("status") == "complete" and result.get("worktreeId") == worktree_id
             and result.get("preparationInputRef") == reference and result.get("operationKey") == key
             and key == digest({name: original.get(name) for name in
                                ("repositoryId", "sessionId", "requestId", "operationId")})
             and original.get("repositoryId") == repository_id
             and original.get("driverDigest") == driver and original.get("runtimeOperationId") == runtime
             and original.get("worktreeId") == worktree_id
             and op.get("inputDigest") == digest({k: v for k, v in original.items() if k != "observationOnly"}),
             "managed_inventory_unproven", "Preparation requires its exact completed Jenkins inventory")
    expected = Path(object_root) / "management/reports" / key / "source-inventory.json"
    _plain(expected)
    _require(isinstance(reference, dict) and set(reference) == {"path", "sha256", "bytes"}
             and reference.get("path") == str(expected) and type(reference.get("bytes")) is int
             and 0 < reference["bytes"] <= SOURCE_EVIDENCE_MAX_BYTES
             and expected.is_file() and expected.stat().st_size == reference["bytes"],
             "managed_inventory_unproven", "Inventory must be the bounded original report")
    raw = expected.read_bytes()
    _require(hashlib.sha256(raw).hexdigest() == require_digest(reference.get("sha256"), "inventorySha256"),
             "managed_inventory_changed", "Inventory report bytes differ from their exact pin")
    value = json.loads(raw)
    _require(isinstance(value, dict) and value.get("inventoryDigest") == result.get("inventoryDigest")
             and digest(value.get("preparationInput")) == value.get("inventoryDigest"),
             "managed_inventory_changed", "Inventory closure digest differs from the completed operation")
    return row, value["preparationInput"]



def inspect_worktree(management, *, preparation_helper):
    """Hash observed inputs; this grants no source acceptance or ownership."""
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
    _require(management.source(target) == prior, "managed_worktree_target_changed",
             "Target must retain its exact previous complete source operation")
    captured = management.source(management.repo)
    membership = _source_membership(management, record["baseHead"], prior)
    observed = {name.casefold(): checksum for name, checksum in captured["files"].items()}
    hashes = {name: observed[name] if name in observed else _before_hash(management.repo, name)
              for name in sorted(membership)}
    controls = {name: hashes[name] for name in sorted(COMPILER_CONTROL_PATHS)}
    compiler_controls(controls, hashes, record["ownedPaths"], managed=True)
    require_product_inputs(hashes, controls)
    before = {}
    for name in hashes:
        destination = target / name
        _plain(destination)
        _require(not destination.exists() or destination.is_file(),
                 "managed_worktree_target_changed", "Inspection target contains an unsupported directory")
        before[name] = _before_hash(target, name)
    external, external_captures, gitlinks = _external_capture(management, preparation_helper, inspection=True)
    _verify_canonical(management, captured["head"], hashes, record["baseHead"], prior)
    _require(management.source(target) == prior, "managed_worktree_target_changed",
             "Target changed during input inspection")
    assert_no_writer(management.state, management.payload["worktreeId"])
    value = {"sourceHead": captured["head"], "currentHashes": hashes, "beforeHashes": before,
             "compilerControlInputs": controls, "externalRoots": management.payload["externalRoots"],
             "externalGitlinks": gitlinks, "externalInputs": external,
             "targetSourceOperationKey": prior_row["key"]}
    report = management.outputs() / "reports" / management.key
    manifest_ref = management.artifact_manifest(target, {"reports": report})
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


def prepare_worktree(management, *, preparation_helper):
    """Materialize read-authorized inputs into Jenkins' WT; canonical is read-only."""
    from .current_source import _hashes
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
    hashes = _hashes(management.repo, inputs.get("currentHashes"))
    controls = compiler_controls(inputs.get("compilerControlInputs"), hashes,
                                  record["ownedPaths"], managed=True)
    _require(controls is not None, "compiler_control_scope_invalid", "Complete product controls are explicit")
    validate_hash_paths(hashes, controls)
    require_product_inputs(hashes, controls)
    head = inputs.get("sourceHead")
    _require(isinstance(head, str) and len(head) in {40, 64}
             and all(c in "0123456789abcdef" for c in head),
             "managed_source_head_missing", "Canonical input HEAD must be immutable")
    external, external_captures, _ = _external_capture(management, preparation_helper, payload=inputs)
    reader = _reader(management, record, hashes, external)
    prior_row, prior = _target_source(management, inputs.get("targetSourceOperationKey"))
    worktree_row = management.state.get("management_worktree", reference)
    _require(prior.get("head") == record["baseHead"], "managed_worktree_base_changed",
             "The previous source receipt must retain this WT base")
    _require(management.source(target) == prior,
             "managed_worktree_target_changed", "Target differs from its previous complete source receipt")
    before = _hashes(management.repo, inputs.get("beforeHashes"))
    _require(set(before) == set(hashes), "managed_source_scope_invalid", "Before/post scopes must be identical")
    for path, checksum in before.items():
        physical = target / path
        _plain(physical)
        _require(len(str(physical)) < 260 and (not physical.exists() or physical.is_file()),
                 "managed_worktree_path_invalid", "Target file violates its physical path budget")
        ancestor = physical.parent
        while ancestor != target:
            _plain(ancestor)
            _require(not ancestor.exists() or ancestor.is_dir(),
                     "managed_worktree_path_invalid", "A target parent is not a directory")
            ancestor = ancestor.parent
        _require(_before_hash(target, path) == checksum,
                 "managed_worktree_target_changed", "Target preimage differs from the declared before hash")
    _verify_canonical(management, head, hashes, record["baseHead"], prior)
    object_root = canonical_build_root(management.build_root()).namespace()
    # Register artifacts before publishing any new CAS object or report.
    report = management.outputs() / "reports" / management.key
    manifest_ref = management.artifact_manifest(target, {"candidate": target, "reports": report,
                                                        "inputObjects": object_root / "inputs"})
    claims = [PathClaim(path, "dependency", 0, checksum, 0, management.repo_id)
              for path, checksum in hashes.items()]
    sealed = seal_candidate(management.repo, claims, object_root=object_root,
                            base_head=record["baseHead"])
    frozen_external = _seal_external(external_captures, preparation_helper, object_root)
    source_manifest = _thaw(sealed.payload)
    source_manifest.update(preparationKind="current-source", external=frozen_external)
    _verify_canonical(management, head, hashes, record["baseHead"], prior)
    _require(management.source(target) == prior,
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
            "sourceKind": "observed-canonical-input", "sourcePreparation": True,
            "implementationOwnershipTransferred": False,
            "provenanceReferences": {"kind": "unasserted-input-metadata",
                                     "references": management.payload.get("provenanceRefs", [])},
            "writerSessionId": record["sessionId"],
            "writerAuthorizationSnapshot": record["writerAuthorizationSnapshot"],
            "repositoryId": management.repo_id, "driverDigest": management.config["driverDigest"],
            "runtimeOperationId": management.config["runtimeOperationId"],
            "artifactManifest": manifest_ref}, expected_version=0, connection=connection)
    # Recheck all preimages and sealed objects before the first target write.
    for path, checksum in hashes.items():
        _plain(target / path)
        _require(_before_hash(target, path) == before[path],
                 "managed_worktree_target_changed", "Target changed after source sealing")
        if checksum is not None:
            obj = object_root / "inputs/objects" / checksum
            _plain(obj)
            _require(obj.is_file() and hashlib.sha256(obj.read_bytes()).hexdigest() == checksum,
                     "managed_source_object_changed", "A sealed source object is absent or corrupt")
    # Every target preimage and every source object has passed before any target
    # write. Each handle rechecks its own preimage immediately before mutation.
    for path, checksum in hashes.items():
        destination = target / path
        _plain(destination)
        _require(_before_hash(target, path) == before[path],
                 "managed_worktree_target_changed", "Target preimage changed before its write")
        if checksum == before[path]:
            continue
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
    _require(all(_before_hash(target, path) == checksum for path, checksum in hashes.items()),
             "managed_worktree_postimage_changed", "Materialized WT differs from its declared input hashes")
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


def preparation(state, reference, worktree_id, *, connection=None):
    reference = require_digest(reference, "worktreePreparationRef")
    row = state.get("managed_worktree_preparation", reference, connection=connection)
    value = row["payload"] if row else {}
    operation = state.get("management_operation", reference, connection=connection)
    op = operation["payload"] if operation else {}
    _require(value.get("status") == "complete" and value.get("worktreeId") == worktree_id
             and op.get("phase") == "complete" and op.get("action") == "prepare-worktree"
             and op.get("inputDigest") == value.get("inputDigest")
             and op.get("result", {}).get("worktreePreparationRef") == reference
             and op.get("result", {}).get("source") == value.get("source")
             and op.get("result", {}).get("sealedCandidateDigest") == digest(value.get("sourceManifest")),
             "managed_source_preparation_unproven", "Worktree lacks its exact completed source preparation")
    original = op.get("payload", {})
    _require(reference == digest({name: original.get(name) for name in
                                  ("repositoryId", "sessionId", "requestId", "operationId")})
             and op.get("inputDigest") == digest({k: v for k, v in original.items() if k != "observationOnly"})
             and all(original.get(key) == value.get(key) for key in
                     ("repositoryId", "driverDigest", "runtimeOperationId", "worktreeId",
                      "inspectionOperationKey", "preparationInputRef"))
             and original.get("sessionId") == value.get("readerSessionId")
             and value.get("sourceManifest", {}).get("baseHead") == value.get("baseHead"),
             "managed_source_preparation_unproven", "Preparation receipt differs from its immutable action input")
    _, inputs = _inventory_reference(state, value["objectRoot"], value["inspectionOperationKey"],
        value["preparationInputRef"], worktree_id, value["repositoryId"], value["driverDigest"],
        value["runtimeOperationId"])
    _require(all(inputs.get(key) == value.get(key) for key in
                 ("sourceHead", "currentHashes", "beforeHashes", "compilerControlInputs", "externalInputs")),
             "managed_source_preparation_unproven", "Preparation differs from its original inventory closure")
    verify_objects(value["sourceManifest"], Path(value["objectRoot"]))
    return row


def reader_binding(state, root, intent, reader, *, connection=None):
    """Only a completed baseline and real narrow writer patches may supply Flow."""
    row = preparation(state, intent["worktreePreparationRef"], intent["worktreeId"], connection=connection)
    value = row["payload"]
    identity = intent["identity"]
    _require(all(value.get(key) == expected for key, expected in {
        "repositoryId": identity["repositoryId"], "driverDigest": identity["stageImplementationDigest"],
        "runtimeOperationId": intent["runtimeOperationId"], "baseHead": intent["baseHead"],
        "readerSessionId": identity["sessionId"], "readerAuthorizationSnapshot": reader["snapshotDigest"],
        "externalInputs": intent["externalInputs"], "compilerControlInputs": intent["compilerControlInputs"],
    }.items()), "managed_source_preparation_mismatch", "Reader changed its prepared input authority")
    hashes, source = dict(value["currentHashes"]), value["source"]
    patch_keys = intent.get("patchOperationKeys", [])
    _require(isinstance(patch_keys, list) and len(patch_keys) == len(set(patch_keys)),
             "managed_source_patch_unproven", "Author patch operation keys must be unique and ordered")
    pins = [_pin(row)]
    for key in patch_keys:
        operation = state.get("management_operation", require_digest(key, "patchOperationKey"), connection=connection)
        op = operation["payload"] if operation else {}
        original, result = op.get("payload", {}), op.get("result", {})
        before, after = original.get("beforeHashes", {}), result.get("afterHashes", {})
        paths = set(before)
        _require(op.get("phase") == "complete" and op.get("action") == "apply-patch"
                 and original.get("worktreeId") == intent["worktreeId"]
                 and original.get("sessionId") == value["writerSessionId"]
                 and original.get("repositoryId") == identity["repositoryId"]
                 and original.get("driverDigest") == identity["stageImplementationDigest"]
                 and original.get("runtimeOperationId") == intent["runtimeOperationId"]
                 and original.get("expectedHead") == intent["baseHead"]
                 and key == digest({name: original.get(name) for name in
                                    ("repositoryId", "sessionId", "requestId", "operationId")})
                 and op.get("inputDigest") == digest({k: v for k, v in original.items() if k != "observationOnly"})
                 and result.get("status") == "complete" and result.get("operationKey") == key
                 and result.get("worktreeId") == intent["worktreeId"] and paths
                 and paths == set(after) and paths <= set(intent["ownedPaths"])
                 and all(hashes.get(path) == checksum for path, checksum in before.items()),
                 "managed_source_patch_unproven", "Source integration requires its original narrow writer operation")
        next_source = result.get("source", {})
        _require(next_source.get("head") == intent["baseHead"]
                 and next_source.get("sourceDigest") == digest({k: v for k, v in next_source.items()
                                                                if k != "sourceDigest"}),
                 "managed_source_patch_unproven", "Writer source receipt is corrupt")
        old_files = {name.casefold(): checksum for name, checksum in source["files"].items()}
        new_files = {name.casefold(): checksum for name, checksum in next_source["files"].items()}
        _require(all(old_files.get(path) == new_files.get(path)
                     for path in (set(old_files) | set(new_files)) - paths)
                 and all(new_files.get(path) == checksum for path, checksum in after.items()),
                 "managed_source_patch_unproven", "Writer operation changed paths outside its exact patch")
        hashes.update(after)
        source = next_source
        pins.append(_pin(operation))
    _require(hashes == intent["currentHashes"],
             "managed_source_patch_unproven", "Flow bytes differ from the baseline plus declared author patches")
    return {"preparationVersion": row["version"], "preparationDigest": digest(value),
            "operationPins": pins, "source": source}


def prepared_external(state, intent):
    row = preparation(state, intent["worktreePreparationRef"], intent["worktreeId"])
    return row["payload"]["sourceManifest"].get("external", [])


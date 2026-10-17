"""Seal index/worktree bytes without staging or consuming the shared index."""
from __future__ import annotations

import os
from pathlib import Path
import re
import subprocess
import hashlib

from .inventory import git, repository_fact, text_git
from .paths import digest, file_fact, physical, relative_source


def _reject_unadopted_legacy_path_claims(authority, repository_id: str, owned_paths: list[str]) -> None:
    """Fail closed until a supported adoption binds the exact historical claim."""
    from .paths import overlaps
    if not owned_paths:
        return
    adoptions = authority.state.list("governance_legacy_adoption")
    for row in authority.state.list("path_claims"):
        claim = row["payload"]
        if claim.get("repositoryId") != repository_id:
            continue
        scopes = claim.get("paths")
        if not isinstance(scopes, list) or any(not isinstance(scope, str) for scope in scopes):
            raise ValueError("legacy_path_claim_scope_unverifiable")
        if not any(overlaps(path, scope) for path in owned_paths for scope in scopes):
            continue
        previous_digest = digest(claim)
        verified = False
        for adoption_row in adoptions:
            adoption = adoption_row["payload"]
            evidence = adoption.get("evidence")
            if (
                adoption_row.get("key") == digest(adoption)
                and adoption.get("domain") == "path_claims"
                and adoption.get("key") == row.get("key")
                and adoption.get("previousDigest") == previous_digest
                and adoption.get("previousVersion") == row.get("version")
                and adoption.get("status") == "verified-adoption"
                and adoption.get("historicalRecordPreserved") is True
                and isinstance(evidence, dict)
                and isinstance(evidence.get("sourceHashes"), dict)
                and bool(evidence["sourceHashes"])
                and isinstance(evidence.get("retainedRefs"), list)
                and bool(evidence["retainedRefs"])
                and isinstance(evidence.get("originalNativeJobs"), list)
                and bool(evidence["originalNativeJobs"])
            ):
                verified = True
                break
        if not verified:
            raise ValueError("legacy_path_claim_requires_verified_adoption")


def write_git(root: Path, args: list[str], *, data: bytes | None = None, index: Path | None = None) -> bytes:
    environment = {**os.environ, "GIT_TERMINAL_PROMPT": "0", "GIT_NO_LAZY_FETCH": "1",
                   "GIT_AUTHOR_NAME": "Jenkins governance", "GIT_AUTHOR_EMAIL": "jenkins-governance@localhost",
                   "GIT_COMMITTER_NAME": "Jenkins governance", "GIT_COMMITTER_EMAIL": "jenkins-governance@localhost"}
    if index:
        environment["GIT_INDEX_FILE"] = str(physical(index, exists=False))
    result = subprocess.run(["git", "-c", "core.fsmonitor=false", "-c", "core.hooksPath=" + os.devnull, "-c", "core.hooksPath=NUL" if os.name == "nt" else "core.hooksPath=/dev/null",
                             "-c", "core.fsmonitor=false", "-c", "core.longpaths=true", "-C", str(physical(root)), *args],
                            input=data, env=environment, capture_output=True, timeout=180)
    if result.returncode:
        failure = ValueError("managed_git_failed:" + str(result.returncode))
        if result.stderr:
            failure.add_note("Git stderr (last 2048 bytes): " + result.stderr[-2048:].decode("utf-8", "replace").strip())
        raise failure
    return result.stdout


def seal(authority, payload: dict, scratch: Path) -> dict:
    authority.require_closed(payload)
    registration = authority.get("repository", payload["repositoryId"])
    source = physical(payload.get("sourceCheckout", registration["canonicalCheckout"]))
    observed = repository_fact(source)
    if observed.get("commonDir") != registration["commonDir"]:
        raise ValueError("source_checkout_belongs_to_another_repository")
    if (observed.get("head") != payload["expectedHead"]
            or observed.get("statusDigest") != payload["expectedStatusDigest"]
            or observed.get("indexDigest") != payload.get("expectedIndexDigest")):
        raise ValueError("source_checkout_changed")
    from .scopes import inspect
    inputs = inspect(source, payload["paths"])
    scopes = inputs["scopes"]
    if not scopes:
        raise ValueError("sealed_paths_required")
    if payload.get("expectedHashes") != inputs["hashes"]:
        raise ValueError("source_hashes_changed")
    head_names = set(git(source, "ls-tree", "-r", "--name-only", "-z", payload["expectedHead"]).decode("utf-8", "surrogateescape").split("\0"))
    paths = sorted(set(inputs["sourceFacts"]) | {name for name in head_names if name
                   and any(name == scope or name.startswith(scope + "/") for scope in scopes)})
    from .paths import overlaps
    for row in authority.state.list("governance_claim"):
        claim = row["payload"]
        if claim["repositoryId"] == payload["repositoryId"] and claim["ownerSessionId"] != payload["ownerSessionId"]:
            if any(overlaps(a, b) for a in paths for b in claim["paths"]):
                raise ValueError("sealed_scope_owned_by_another_session")
    entries = git(source, "ls-files", "--stage", "-z")
    if hashlib.sha256(entries).hexdigest() != payload["expectedIndexDigest"]:
        raise ValueError("staged_input_changed_before_sealing")
    selected, modes = [], {}
    for raw in entries.split(b"\0"):
        if not raw:
            continue
        metadata, filename = raw.split(b"\t", 1)
        mode, oid, stage = metadata.split(b" ")
        if stage != b"0":
            raise ValueError("unmerged_index_requires_review")
        name = filename.decode("utf-8", "surrogateescape")
        if name in paths:
            selected.append(raw + b"\0")
            modes[name] = mode
    before = {}
    for name in paths:
        candidate = physical(source / name, exists=False)
        before[name] = file_fact(candidate) if candidate.exists() else None
    scratch = physical(scratch)
    private = scratch / "seal.index"
    if private.exists():
        raise ValueError("original_seal_index_requires_reconciliation")
    write_git(source, ["read-tree", payload["expectedHead"]], index=private)
    if selected:
        write_git(source, ["update-index", "-z", "--index-info"], data=b"".join(selected), index=private)
    indexed_names = set(modes)
    # A tracked deletion missing from the index must be represented explicitly.
    remove = [name for name in paths if name in head_names and name not in indexed_names]
    if remove:
        write_git(source, ["update-index", "--force-remove", "--", *remove], index=private)
    staged_tree = write_git(source, ["write-tree"], index=private).decode().strip()
    staged_commit = write_git(source, ["commit-tree", staged_tree, "-p", payload["expectedHead"]],
                              data=b"Preserve selected pre-governance index state\n").decode().strip()
    for name, fact in before.items():
        if fact is None:
            write_git(source, ["update-index", "--force-remove", "--", name], index=private)
            continue
        data = (source / name).read_bytes()
        oid = write_git(source, ["hash-object", "-w", "--stdin"], data=data).decode().strip()
        if file_fact(source / name) != fact:
            raise ValueError("source_changed_during_sealing")
        mode = modes.get(name, b"100644")
        if mode == b"160000":
            raise ValueError("gitlink_changes_require_explicit_dependency_registration")
        write_git(source, ["update-index", "--add", "--cacheinfo", mode.decode() + "," + oid + "," + name], index=private)
    tree = write_git(source, ["write-tree"], index=private).decode().strip()
    commit = write_git(source, ["commit-tree", tree, "-p", staged_commit],
                       data=("Preserve governance work owned by " + payload["ownerSessionId"] + "\n").encode()).decode().strip()
    final = repository_fact(source)
    if inspect(source, scopes)["hashes"] != inputs["hashes"]:
        raise ValueError("source_scope_changed_during_sealing")
    if final.get("statusDigest") != observed["statusDigest"] or final.get("head") != observed["head"]:
        raise ValueError("source_changed_during_sealing")
    if git(source, "ls-files", "--stage", "-z") != entries:
        raise ValueError("shared_index_changed_during_sealing")
    branch = "refs/heads/codex/jenkins/" + payload["ownerSessionId"] + "/" + payload["roundId"] + "-" + payload["operationId"]
    if not re.fullmatch(r"refs/heads/codex/jenkins/[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", branch):
        raise ValueError("managed_branch_identity_invalid")
    write_git(source, ["update-ref", branch, commit, "0" * len(commit)])
    result = {"status": "sealed", "repositoryId": payload["repositoryId"], "paths": paths, "scopes": scopes,
              "ownerSessionId": payload["ownerSessionId"], "sourceCheckout": str(source), "branch": branch,
              "indexTree": staged_tree, "indexCommit": staged_commit, "resultTree": tree, "commit": commit,
              "baseHead": observed["head"], "sourceHashes": before, "sharedIndexPreserved": True,
              "roundId": payload["roundId"], "selectedIndexDigest": hashlib.sha256(b"".join(selected)).hexdigest(),
              "sourceStatusDigest": observed["statusDigest"], **authority.runtime}
    authority.put("sealed", payload["operationId"], result, expected_version=0)
    # Only this exact temporary private index belongs to this operation.
    private.unlink()
    return result


def prepare_integration(authority, payload: dict) -> dict:
    authority.require_closed(payload)
    registration = authority.get("repository", payload["repositoryId"])
    sealed = authority.get("sealed", payload["sealedOperationId"])
    if sealed["repositoryId"] != payload["repositoryId"]:
        raise ValueError("integration_repository_mismatch")
    root = physical(registration["canonicalCheckout"])
    primary = "refs/heads/" + registration["primaryBranch"]
    head = text_git(root, "rev-parse", primary)
    if head != payload["expectedPrimaryHead"]:
        raise ValueError("primary_head_changed")
    included = subprocess.run(["git", "-c", "core.fsmonitor=false", "-c", "core.hooksPath=" + os.devnull, "-C", str(root), "merge-base", "--is-ancestor", sealed["commit"], head], capture_output=True)
    if included.returncode == 0:
        value = {"status": "already-integrated", "repositoryId": payload["repositoryId"],
                 "sealedOperationId": payload["sealedOperationId"], "primaryHead": head, "commit": head}
        authority.put("integration", payload["operationId"], value, expected_version=0)
        return value
    # Git 2.31 on this host has only the legacy three-argument merge-tree.
    # Construct the merge with a private index and bounded blob inputs instead.
    tree, diagnostics = _merge_objects(authority, registration, head, sealed["commit"], payload["operationId"])
    if tree is None:
        value = {"status": "conflict", "repositoryId": payload["repositoryId"], "sealedOperationId": payload["sealedOperationId"],
                 "primaryHead": head, "mergeDiagnosticDigest": digest(diagnostics), "conflicts": diagnostics, "sourcePreserved": True}
    else:
        owned = git(root, "diff-tree", "--no-commit-id", "--name-only", "-r", "-z", head, tree).decode("utf-8", "surrogateescape").split("\0")
        owned = [relative_source(name) for name in owned if name]
        from .paths import overlaps
        for row in authority.state.list("governance_claim"):
            claim = row["payload"]
            if claim["repositoryId"] == payload["repositoryId"] and claim["status"] == "claimed":
                if any(overlaps(path, scope) for path in owned for scope in claim["paths"]):
                    raise ValueError("integration_would_change_claimed_source")
        _reject_unadopted_legacy_path_claims(authority, payload["repositoryId"], owned)
        commit = write_git(root, ["commit-tree", tree, "-p", head, "-p", sealed["commit"]],
                           data=b"Integrate unclaimed governance input after validation\n").decode().strip()
        pending = "refs/heads/codex/jenkins/governance/" + payload["operationId"]
        write_git(root, ["update-ref", pending, commit, "0" * len(commit)])
        value = {"status": "validation-required", "repositoryId": payload["repositoryId"],
                 "sealedOperationId": payload["sealedOperationId"], "primaryHead": head,
                 "commit": commit, "resultTree": tree, "pendingBranch": pending, "primaryRef": primary,
                 "validationRecipes": registration["validationRecipes"]}
        value["ownedPaths"] = owned
    authority.put("integration", payload["operationId"], value, expected_version=0)
    return value


def _tree_entries(root, revision):
    result = {}
    for raw in git(root, "ls-tree", "-r", "-z", revision).split(b"\0"):
        if raw:
            metadata, path = raw.split(b"\t", 1)
            mode, kind, oid = metadata.decode().split()
            result[path.decode("utf-8", "surrogateescape")] = (mode, kind, oid)
    return result


def _merge_objects(authority, registration, ours, theirs, operation):
    from .paths import build_namespace
    root = physical(registration["canonicalCheckout"])
    bases = text_git(root, "merge-base", "--all", ours, theirs).splitlines()
    if len(bases) != 1:
        return None, [{"reason": "ambiguous_or_unrelated_merge_base"}]
    base = bases[0]
    original, left, right = (_tree_entries(root, value) for value in (base, ours, theirs))
    scratch = build_namespace(registration["buildNamespace"]) / "governance-merge" / operation
    if "taskId" in authority.runtime:
        from .worker import register_artifacts
        register_artifacts(authority, [{"path": str(scratch), "role": "private-Git-index-and-bounded-blob-inputs",
                                     "retainUntil": "merge is reconciled; retain failed inputs as evidence"}])
    physical(scratch, exists=False).mkdir(parents=True, exist_ok=False)
    index = scratch / "merge.index"
    write_git(root, ["read-tree", ours], index=index)
    conflicts, temporary = [], []
    for number, path in enumerate(sorted(original.keys() | left.keys() | right.keys())):
        before, current, candidate = original.get(path), left.get(path), right.get(path)
        if candidate == before or current == candidate:
            continue
        name = relative_source(path)
        if current == before:
            chosen = candidate
        elif (before and current and candidate and all(item[0] in {"100644", "100755"} for item in (before, current, candidate))
              and (current[0] == candidate[0] or current[0] == before[0] or candidate[0] == before[0])):
            paths = [scratch / (str(number) + suffix) for suffix in (".ours", ".base", ".theirs")]
            for output, entry in zip(paths, (current, before, candidate)):
                data = git(root, "cat-file", "blob", entry[2])
                if len(data) > 100 * 1024**2:
                    raise ValueError("merge_blob_exceeds_bounded_memory_budget")
                output.write_bytes(data)
                temporary.append(file_fact(output))
            merged = subprocess.run(["git", "-c", "core.fsmonitor=false", "-c", "core.hooksPath=" + os.devnull, "merge-file", "-p", *map(str, paths)], capture_output=True, timeout=180,
                                    env={**os.environ, "GIT_TERMINAL_PROMPT": "0"})
            if merged.returncode:
                conflicts.append({"path": name, "reason": "content_conflict_or_binary_merge_unsupported",
                                  "returnCode": merged.returncode, "diagnosticDigest": digest(merged.stderr.hex())})
                continue
            oid = write_git(root, ["hash-object", "-w", "--stdin"], data=merged.stdout).decode().strip()
            mode = candidate[0] if current[0] == before[0] else current[0]
            chosen = (mode, "blob", oid)
        else:
            conflicts.append({"path": name, "reason": "delete_modify_or_source_type_conflict"})
            continue
        if chosen is None:
            write_git(root, ["update-index", "--force-remove", "--", name], index=index)
        else:
            write_git(root, ["update-index", "--add", "--cacheinfo", chosen[0] + "," + chosen[2] + "," + name], index=index)
    if conflicts:
        return None, conflicts
    tree = write_git(root, ["write-tree"], index=index).decode().strip()
    from .guarded_delete import remove_verified
    for fact in temporary + [file_fact(index)]:
        remove_verified(fact)
    return tree, []


def finish_integration(authority, payload: dict) -> dict:
    authority.require_closed(payload)
    integration = authority.get("integration", payload["integrationOperationId"])
    registration = authority.get("repository", integration["repositoryId"])
    proof = authority.get("validation", payload["validationOperationId"])
    if (integration["status"] != "validation-required" or proof.get("status") != "passed"
            or proof.get("resultTree") != integration["resultTree"]
            or proof.get("repositoryId") != integration["repositoryId"]):
        raise ValueError("exact_result_validation_required")
    root = physical(registration["canonicalCheckout"])
    _reject_unadopted_legacy_path_claims(authority, integration["repositoryId"], integration["ownedPaths"])
    if text_git(root, "rev-parse", integration["primaryRef"]) != integration["primaryHead"]:
        raise ValueError("primary_compare_and_swap_failed")
    try:
        checked_out = text_git(root, "symbolic-ref", "HEAD") == integration["primaryRef"]
    except ValueError:
        checked_out = False
    if checked_out:
        _publish_checked_out(authority, root, integration, payload)
    else:
        # Another checkout of main cannot be silently changed behind its shared index.
        observation = repository_fact(root)
        sections = observation.get("worktrees", "").split("\n\n")
        if any("branch " + integration["primaryRef"] in part for part in sections):
            raise ValueError("primary_checked_out_elsewhere_requires_reconciliation")
        write_git(root, ["update-ref", integration["primaryRef"], integration["commit"], integration["primaryHead"]])
    value = {**integration, "status": "integrated", "validationOperationId": payload["validationOperationId"], **authority.runtime}
    authority.put("integration_receipt", payload["operationId"], value, expected_version=0)
    return value


def _publish_checked_out(authority, root, integration, payload):
    """Lock the shared index, transition only owned paths, and CAS the local ref."""
    from .source_write import write_verified
    from .paths import overlaps
    owned = integration["ownedPaths"]
    for row in authority.state.list("governance_claim"):
        claim = row["payload"]
        if claim["repositoryId"] == integration["repositoryId"] and claim["status"] == "claimed":
            if any(overlaps(path, scope) for path in owned for scope in claim["paths"]):
                raise ValueError("new_claim_blocks_primary_integration")
    git_dir = physical(text_git(root, "rev-parse", "--absolute-git-dir"))
    index = git_dir / "index"
    lock = git_dir / "index.lock"
    before_index = file_fact(index)
    sealed = authority.get("sealed", integration["sealedOperationId"])
    before = {path: file_fact(physical(root / path)) if (root / path).exists() else None for path in owned}
    if payload.get("expectedCanonicalHashes") != {path: fact["sha256"] if fact else None for path, fact in before.items()}:
        raise ValueError("canonical_input_hashes_changed")
    entries = git(root, "ls-files", "--stage", "-z")
    selected = [raw + b"\0" for raw in entries.split(b"\0") if raw and raw.split(b"\t", 1)[1].decode("utf-8", "surrogateescape") in sealed["paths"]]
    if str(root) == sealed["sourceCheckout"]:
        if hashlib.sha256(b"".join(selected)).hexdigest() != sealed["selectedIndexDigest"]:
            raise ValueError("selected_staged_input_changed")
        for path in owned:
            if path in sealed["sourceHashes"] and before[path] != sealed["sourceHashes"][path]:
                raise ValueError("selected_worktree_input_changed")
    else:
        changed = repository_fact(root).get("changes", [])
        if any(item["path"] in owned for item in changed):
            raise ValueError("foreign_canonical_change_protected")
    # Source transitions require State-backed proof for every writer cited by this request.
    jobs = payload.get("writerJobIds", [])
    if not jobs:
        raise ValueError("canonical_writer_termination_proof_required")
    for key in jobs:
        row = authority.state.get("native_job", key)
        job = row["payload"] if row else {}
        if (job.get("status") != "terminal" or job.get("completeProof", {}).get("complete") is not True
                or job.get("repositoryId") != integration["repositoryId"]
                or not any(overlaps(scope, root) for scope in job.get("writeScopes", []))):
            raise ValueError("canonical_writer_not_reconciled")
    original_index = index.read_bytes()
    journal = {"status": "preparing-preimages", "preimages": before, "indexBefore": before_index,
               "integration": integration, "retainedIndex": str(lock), "writes": [], **authority.runtime}
    authority.put("source_transition", payload["operationId"], journal, expected_version=0)
    with lock.open("xb") as stream:
        stream.write(original_index)
        stream.flush()
        os.fsync(stream.fileno())
    ref_updated = False
    try:
        if file_fact(index) != before_index:
            raise ValueError("shared_index_changed_before_lock")
        # Save raw CRLF/binary preimages and the complete original shared index
        # in retained Git objects before any source write, including a clean
        # canonical checkout whose on-disk bytes differ from filtered Git blobs.
        preimage_index = git_dir / ("governance-preimage-" + payload["operationId"] + ".index")
        if preimage_index.exists():
            raise ValueError("original_preimage_index_requires_reconciliation")
        journal["preimageIndex"] = str(preimage_index)
        authority.put("source_transition", payload["operationId"], journal)
        with preimage_index.open("xb") as stream:
            stream.write(original_index)
            stream.flush()
            os.fsync(stream.fileno())
        original_staged_tree = write_git(root, ["write-tree"], index=preimage_index).decode().strip()
        for name, fact in before.items():
            if fact is None:
                write_git(root, ["update-index", "--force-remove", "--", name], index=preimage_index)
                continue
            data = (root / name).read_bytes()
            if hashlib.sha256(data).hexdigest() != fact["sha256"] or file_fact(root / name) != fact:
                raise ValueError("source_changed_while_preserving_preimage")
            blob = write_git(root, ["hash-object", "-w", "--stdin"], data=data).decode().strip()
            write_git(root, ["update-index", "--add", "--cacheinfo", "100644," + blob + "," + name], index=preimage_index)
        source_tree = write_git(root, ["write-tree"], index=preimage_index).decode().strip()
        index_blob = write_git(root, ["hash-object", "-w", "--stdin"], data=original_index).decode().strip()
        retained_tree = write_git(root, ["mktree"], data=("100644 blob " + index_blob + "\tshared-index\n"
                        "040000 tree " + source_tree + "\tsource\n"
                        "040000 tree " + original_staged_tree + "\tstaged\n").encode()).decode().strip()
        retained_commit = write_git(root, ["commit-tree", retained_tree, "-p", integration["primaryHead"]],
                                    data=b"Retain exact canonical source and shared-index preimages\n").decode().strip()
        retained_ref = "refs/jenkins/governance/preimages/" + payload["operationId"]
        write_git(root, ["update-ref", retained_ref, retained_commit, "0" * len(retained_commit)])
        journal.update(status="prepared", preimageRef=retained_ref, preimageCommit=retained_commit, indexPreimageBlob=index_blob)
        authority.put("source_transition", payload["operationId"], journal)
        from .guarded_delete import remove_verified
        remove_verified(file_fact(preimage_index))
        for name in owned:
            tree_entry = git(root, "ls-tree", integration["commit"], "--", name).decode().strip()
            if tree_entry:
                mode, _, oid = tree_entry.split("\t", 1)[0].split()
                if mode not in {"100644", "100755"}:
                    raise ValueError("integration_source_type_requires_review")
                content = git(root, "cat-file", "blob", oid)
                write_git(root, ["update-index", "--add", "--cacheinfo", mode + "," + oid + "," + name], index=lock)
            else:
                content = None
                write_git(root, ["update-index", "--force-remove", "--", name], index=lock)
            current = before[name]
            if (hashlib.sha256(content).hexdigest() if content is not None else None) != (current["sha256"] if current else None):
                write_verified(root / name, current, content)
            journal["writes"].append({"path": name, "after": file_fact(root / name) if content is not None else None,
                                     "expectedSha256": hashlib.sha256(content).hexdigest() if content is not None else None})
            authority.put("source_transition", payload["operationId"], journal)
        # Foreign index entries remain byte-for-byte semantically identical in the private replacement.
        foreign_before = [raw for raw in entries.split(b"\0") if raw and raw.split(b"\t", 1)[1].decode("utf-8", "surrogateescape") not in owned]
        env = {**os.environ, "GIT_INDEX_FILE": str(lock), "GIT_OPTIONAL_LOCKS": "0"}
        after = subprocess.run(["git", "-c", "core.fsmonitor=false", "-c", "core.hooksPath=" + os.devnull, "-C", str(root), "ls-files", "--stage", "-z"], env=env, capture_output=True, check=True).stdout
        foreign_after = [raw for raw in after.split(b"\0") if raw and raw.split(b"\t", 1)[1].decode("utf-8", "surrogateescape") not in owned]
        if foreign_before != foreign_after or file_fact(index) != before_index:
            raise ValueError("foreign_index_changed_during_integration")
        journal.update(status="ready-to-publish", replacementIndex=file_fact(lock))
        authority.put("source_transition", payload["operationId"], journal)
        write_git(root, ["update-ref", integration["primaryRef"], integration["commit"], integration["primaryHead"]])
        ref_updated = True
        os.replace(lock, index)
        authority.put("source_transition", payload["operationId"], {**journal, "status": "complete",
            "indexAfter": file_fact(index), "foreignIndexPreserved": True})
    except Exception:
        # Retain the exact original intent, branch, preimages and private index for reconciliation.
        authority.put("source_transition", payload["operationId"], {**journal, "status": "index-pending" if ref_updated else "needs-review"})
        raise


def recover_integration(authority, payload):
    """Reconcile the original publication only when exact retained evidence permits it."""
    authority.require_closed(payload)
    original = authority.state.get_operation(payload["originalOperationId"])
    original_request = original["payload"].get("request", {}) if original else {}
    if (not original or original_request.get("action") != "finish-integration"
            or original["payloadDigest"] != payload["originalIntentDigest"]):
        raise ValueError("original_integration_intent_required")
    transition = authority.get("source_transition", payload["originalOperationId"])
    integration = transition["integration"]
    if (integration["repositoryId"] != payload["repositoryId"]
            or transition.get("status") not in {"index-pending", "complete"}):
        raise ValueError("partial_source_writes_require_review_with_retained_preimages")
    identity = transition.get("processIdentity") or {}
    if not identity.get("pid") or not identity.get("creationTime"):
        raise ValueError("original_publication_process_identity_missing")
    from tools.jenkins.processes.identity import current_identity, process_alive
    alive = process_alive(identity["pid"])
    if alive is None:
        raise ValueError("original_process_death_unproved")
    current = current_identity(identity["pid"]).to_dict() if alive else None
    if current and str(current.get("creationTime")) == str(identity["creationTime"]):
        raise ValueError("original_publication_writer_still_alive")
    registration = authority.get("repository", integration["repositoryId"])
    original_payload = original_request["payload"]
    validation = authority.get("validation", original_payload["validationOperationId"])
    if (validation.get("status") != "passed" or validation.get("resultTree") != integration["resultTree"]
            or validation.get("repositoryId") != integration["repositoryId"]):
        raise ValueError("original_result_validation_changed")
    root = physical(registration["canonicalCheckout"])
    from .paths import overlaps
    for row in authority.state.list("governance_claim"):
        claim = row["payload"]
        if claim["repositoryId"] == integration["repositoryId"] and claim["status"] == "claimed":
            if any(overlaps(path, scope) for path in integration["ownedPaths"] for scope in claim["paths"]):
                raise ValueError("new_claim_blocks_original_index_recovery")
    _reject_unadopted_legacy_path_claims(authority, integration["repositoryId"], integration["ownedPaths"])
    if text_git(root, "rev-parse", integration["primaryRef"]) != integration["commit"]:
        raise ValueError("original_result_reference_changed")
    if text_git(root, "rev-parse", transition["preimageRef"]) != transition["preimageCommit"]:
        raise ValueError("retained_source_preimage_reference_changed")
    writes = transition["writes"]
    if {row["path"] for row in writes} != set(integration["ownedPaths"]):
        raise ValueError("source_transition_missing_exact_write_receipts")
    for row in writes:
        path = physical(root / row["path"], exists=False)
        if (file_fact(path) if path.exists() else None) != row["after"]:
            raise ValueError("source_changed_after_original_publication")
    git_dir = physical(text_git(root, "rev-parse", "--absolute-git-dir"))
    index, lock = git_dir / "index", git_dir / "index.lock"
    expected = transition["replacementIndex"]
    if lock.exists():
        observed = file_fact(index)
        if file_fact(lock) != expected:
            raise ValueError("retained_replacement_index_changed")
        # The original index.lock remains the exclusion barrier; do not remove
        # it or create a competing publication while its evidence is unresolved.
        # Keep the verified existing index identity and deny writers/deletion
        # while publishing its retained bytes. Atomic replacement conflicts with
        # the handle that protects that identity on Windows. Exact raw preimages
        # are already retained, and an interruption before the post-write journal
        # is recorded requires review rather than guessing at a partial write.
        if observed == transition["indexBefore"]:
            from .source_write import write_verified
            data = lock.read_bytes()
            if hashlib.sha256(data).hexdigest() != expected["sha256"] or file_fact(lock) != expected:
                raise ValueError("retained_replacement_changed_during_read")
            write_verified(index, observed, data)
            transition = {**transition, "recoveryIndex": file_fact(index),
                          "recoveryIndexWriter": authority.runtime}
            authority.put("source_transition", payload["originalOperationId"], transition)
        elif observed != transition.get("recoveryIndex"):
            raise ValueError("original_shared_index_or_recovered_publication_changed")
        from .guarded_delete import remove_verified
        remove_verified(expected)
    else:
        observed = file_fact(index)
        publication = transition.get("recoveryIndex") or transition.get("indexAfter") or expected
        if any(observed[key] != publication[key] for key in ("identity", "bytes", "sha256")):
            raise ValueError("missing_index_lock_does_not_prove_publication")
    prior = authority.state.get("governance_integration_recovery", payload["operationId"])
    if prior:
        previous = prior["payload"]
        if (previous.get("originalOperationId") != payload["originalOperationId"]
                or previous.get("originalIntentDigest") != payload["originalIntentDigest"]
                or previous.get("repositoryId") != integration["repositoryId"]
                or previous.get("commit") != integration["commit"]
                or previous.get("indexAfter") != file_fact(index)):
            raise ValueError("original_recovery_receipt_changed")
        return previous
    value = {"status": "reconciled", "originalOperationId": payload["originalOperationId"],
             "originalIntentDigest": payload["originalIntentDigest"], "originalResult": original["result"],
             "repositoryId": integration["repositoryId"], "commit": integration["commit"],
             "indexAfter": file_fact(index), "preimageRef": transition["preimageRef"], **authority.runtime}
    authority.put("source_transition", payload["originalOperationId"], {**transition, "status": "complete",
                  "indexAfter": value["indexAfter"], "recovery": value})
    authority.put("integration_recovery", payload["operationId"], value, expected_version=0)
    if original["status"] != "reconciled":
        authority.state.transition_operation(payload["originalOperationId"], original["status"], "reconciled", value)
    return value


def apply_patch(authority, payload, report):
    """Apply a claimed session's binary-safe handoff to its managed Git branch."""
    import base64
    authority.require_closed(payload)
    claim = authority.get("claim", payload["claimId"])
    if (claim["status"] != "claimed" or claim["repositoryId"] != payload["repositoryId"]
            or claim["ownerSessionId"] != payload["ownerSessionId"]):
        raise ValueError("active_patch_claim_required")
    sealed = authority.get("sealed", payload["sealedOperationId"])
    if sealed["ownerSessionId"] != claim["ownerSessionId"] or sealed["repositoryId"] != claim["repositoryId"]:
        raise ValueError("patch_managed_branch_claim_mismatch")
    registration = authority.get("repository", claim["repositoryId"])
    root = physical(registration["canonicalCheckout"])
    handoff = physical(payload["handoffPath"])
    inbox = root / ".codex/outbox/jenkins-inbox" / claim["ownerSessionId"]
    if not handoff.is_relative_to(inbox) or file_fact(handoff)["sha256"] != payload["handoffSha256"]:
        raise ValueError("patch_handoff_scope_or_hash_changed")
    import json
    patch = json.loads(handoff.read_text(encoding="utf-8"))
    branch = sealed["branch"]
    head = text_git(root, "rev-parse", branch)
    if head != patch["baseHead"] or patch["claimId"] != payload["claimId"]:
        raise ValueError("patch_original_branch_input_changed")
    private = report / "patch.index"
    write_git(root, ["read-tree", head], index=private)
    changed = []
    from .paths import overlaps
    for item in patch["files"]:
        name = relative_source(item["path"])
        if not any(overlaps(name, scope) and (name == scope or name.startswith(scope + "/")) for scope in claim["paths"]):
            raise ValueError("patch_path_not_claimed")
        old = subprocess.run(["git", "-c", "core.fsmonitor=false", "-c", "core.hooksPath=" + os.devnull, "-C", str(root), "show", head + ":" + name], capture_output=True)
        old_hash = hashlib.sha256(old.stdout).hexdigest() if old.returncode == 0 else None
        if old_hash != item["beforeSha256"]:
            raise ValueError("patch_preimage_hash_changed")
        if item.get("contentBase64") is None:
            write_git(root, ["update-index", "--force-remove", "--", name], index=private)
        else:
            data = base64.b64decode(item["contentBase64"], validate=True)
            if hashlib.sha256(data).hexdigest() != item["afterSha256"]:
                raise ValueError("patch_output_hash_mismatch")
            oid = write_git(root, ["hash-object", "-w", "--stdin"], data=data).decode().strip()
            mode = item.get("mode", "100644")
            if mode not in {"100644", "100755"}:
                raise ValueError("patch_mode_requires_review")
            write_git(root, ["update-index", "--add", "--cacheinfo", mode + "," + oid + "," + name], index=private)
        changed.append(name)
    if not changed or len(changed) != len(set(changed)):
        raise ValueError("patch_paths_empty_or_repeated")
    tree = write_git(root, ["write-tree"], index=private).decode().strip()
    commit = write_git(root, ["commit-tree", tree, "-p", head], data=("Claimed session patch " + claim["ownerSessionId"] + "\n").encode()).decode().strip()
    if file_fact(handoff)["sha256"] != payload["handoffSha256"]:
        raise ValueError("patch_handoff_changed_during_admission")
    write_git(root, ["update-ref", branch, commit, head])
    value = {"status": "managed-branch-updated", "repositoryId": claim["repositoryId"], "claimId": claim["claimId"],
             "branch": branch, "previousHead": head, "commit": commit, "resultTree": tree, "paths": changed,
             "handoff": file_fact(handoff), "validation": "pending", **authority.runtime}
    authority.put("patch", payload["operationId"], value, expected_version=0)
    private.unlink()
    return value

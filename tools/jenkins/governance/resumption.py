"""Resume read-only repository triage from hash-bound volume checkpoints."""
from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor, as_completed
import json
from pathlib import Path
import re
import time

from .inventory import repository_fact
from .paths import build_namespace, digest, file_fact, physical


def load_checkpoint(reference, root):
    path = physical(reference["path"])
    if file_fact(path) != reference:
        raise ValueError("original_volume_checkpoint_changed")
    value = json.loads(path.read_text(encoding="utf-8"))
    if file_fact(path) != reference:
        raise ValueError("original_volume_checkpoint_changed_during_read")
    if (value.get("roots") != [str(root)] or value.get("repositories") != []
            or set(value.get("rootCoverage", {})) != {str(root)}
            or not isinstance(value.get("repositoryRoots"), list)
            or value.get("metadataFinishedAt") is None):
        raise ValueError("original_volume_checkpoint_incomplete")
    for candidate in value["repositoryRoots"]:
        path = Path(candidate)
        if not path.is_absolute() or not path.is_relative_to(root):
            raise ValueError("checkpoint_repository_outside_observed_volume")
    return value


def prepare(authority, payload, report_root):
    identifier = payload["resumeOperationId"]
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,127}", identifier):
        raise ValueError("invalid_original_scan_identity")
    original = authority.state.get_operation(identifier)
    if not original or original["kind"] != "host_governance":
        raise ValueError("original_scan_operation_missing")
    intent = original["payload"]
    request = intent["request"]
    if (request["action"] != "scan" or request["taskId"] != authority.runtime["taskId"]
            or request["operationId"] != identifier
            or intent["driverDigest"] != authority.runtime["driverDigest"]):
        raise ValueError("original_scan_authority_mismatch")
    capability = authority.state.get("governance_capability", intent["capsuleDigest"])
    if not capability or capability["payload"].get("status") != "passed":
        raise ValueError("original_scan_implementation_not_verified")
    directory = physical(build_namespace(report_root) / identifier)
    native = physical(directory / "native-scan/native-receipt.json")
    native_reference = file_fact(native)
    from .worker import _terminal
    if not _terminal(json.loads(native.read_text(encoding="utf-8"))):
        raise ValueError("original_scan_native_termination_unproved")
    if file_fact(native) != native_reference:
        raise ValueError("original_scan_native_receipt_changed")
    observed = {}
    stdout = physical(directory / "native-scan/stdout.bin")
    stdout_reference = file_fact(stdout)
    for line in stdout.read_bytes().splitlines():
        try:
            value = json.loads(line)
        except (ValueError, UnicodeDecodeError):
            continue
        if value.get("phase") == "metadata-checkpoint":
            observed[value["root"]] = value["reference"]
    if file_fact(stdout) != stdout_reference:
        raise ValueError("original_scan_checkpoint_log_changed")
    references = []
    for root in (Path("C:/"), Path("D:/"), Path("E:/")):
        expected = directory / "checkpoints" / root.drive.rstrip(":").lower() / "metadata.json"
        reference = observed.get(str(root))
        if not reference or Path(reference["path"]) != expected:
            raise ValueError("original_whole_volume_checkpoint_missing")
        load_checkpoint(reference, root)
        references.append({"root": str(root), "reference": reference})
    return {"originalOperationId": identifier, "originalRequestId": request["requestId"],
            "originalAttemptId": request["attemptId"], "originalCapsuleDigest": intent["capsuleDigest"],
            "originalNativeReceipt": native_reference, "checkpoints": references,
            "originalCheckpointLog": stdout_reference,
            "repositoryObservation": "fresh-status-index-and-Git-identities",
            "sourceContentHashes": "deferred-to-fresh-claim-takeover-integration-or-cleanup-admission"}


def observe(specification, roots, registrations, checkpoint_root, emit):
    from .worker import _json
    original_references = [specification["originalNativeReceipt"], specification["originalCheckpointLog"]]
    original_references += [item["reference"] for item in specification["checkpoints"]]
    def unchanged():
        if any(file_fact(physical(reference["path"])) != reference for reference in original_references):
            raise ValueError("original_resume_evidence_changed")
    unchanged()
    if [item["root"] for item in specification["checkpoints"]] != [str(root) for root in roots]:
        raise ValueError("resume_volume_set_changed")
    reports, repositories, linked_paths = [], {}, {}
    for root, item in zip(roots, specification["checkpoints"]):
        report = load_checkpoint(item["reference"], root)
        report["errors"] = list(report["errors"])
        report["rootCoverage"] = {key: dict(value) for key, value in report["rootCoverage"].items()}
        report["metadataOrigin"] = item["reference"]
        report["sourceContentHashes"] = specification["sourceContentHashes"]
        directory = checkpoint_root / root.drive.rstrip(":").lower()
        directory.mkdir()
        (directory / "repositories").mkdir()
        _json(directory / "metadata.json", report)
        roots_here = {Path(value) for value in report["repositoryRoots"]}
        for repository in roots_here:
            repositories[repository] = (report, directory)
            linked_paths[repository] = []
        for link in report["links"]:
            path = Path(link["path"])
            for parent in path.parents:
                if parent in roots_here:
                    linked_paths[parent].append(path.relative_to(parent).as_posix())
        reports.append(report)
    def inspect(root):
        try:
            value = repository_fact(root, registrations.get(str(root).casefold()), content_hashes=False,
                                    excluded_paths=sorted(linked_paths[root]))
        except (OSError, ValueError) as error:
            value = {"path": str(root), "disposition": "needs-review", "errors": [type(error).__name__]}
        value["observedAt"] = time.time()
        return value
    # Four bounded readers share one retained native Job, including Git children.
    with ThreadPoolExecutor(max_workers=4) as executor:
        pending = {executor.submit(inspect, root): root for root in sorted(repositories, key=lambda value: str(value).casefold())}
        completed = 0
        for future in as_completed(pending):
            root = pending[future]
            fact = future.result()
            report, directory = repositories[root]
            report["repositories"].append(fact)
            _json(directory / "repositories" / (digest(fact["path"]) + ".json"), fact)
            completed += 1
            emit({"phase": "resumed-repository-observation", "completed": completed, "total": len(pending),
                  "repository": str(root), "changes": len(fact.get("changes", []))})
    limitations = ["Volume metadata is reused from the original hash-bound, terminal scan checkpoints.",
        "Git identities, index and working status are observed again; source bytes are hashed at bounded write admission.",
        "Live observations are not a source freeze. Changing or unreadable inputs remain needs-review.",
        "Linked paths are reported without following them; authentication contents are excluded.",
        "Installed binaries, dependency caches and source candidates need provenance before any cleanup.",
        "Logical bytes include hardlinks and do not measure reclaimable space."]
    for report in reports:
        report["repositories"].sort(key=lambda value: value["path"].casefold())
        report.update(schemaVersion=4, finishedAt=time.time(), limitations=limitations,
                      status="partial" if report["errors"] else "complete")
        report["durationSeconds"] = round(report["finishedAt"] - report["startedAt"], 2)
        report["inventoryDigest"] = digest({key: value for key, value in report.items() if key != "inventoryDigest"})
        root = Path(report["roots"][0])
        _json(checkpoint_root / root.drive.rstrip(":").lower() / "complete.json", report)
    unchanged()
    return reports

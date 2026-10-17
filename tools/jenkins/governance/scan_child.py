"""Fixed read-only inventory recipe, executed in a retained native Job."""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor
import json
from pathlib import Path
import threading
import time

from .inventory import scan
from .paths import build_namespace, digest, file_fact, physical


def combine(roots, reports, started):
    started = min([started] + [report["startedAt"] for report in reports])
    result = {"schemaVersion": max(report["schemaVersion"] for report in reports), "roots": [str(root) for root in roots], "startedAt": started,
              "finishedAt": time.time(), "rootCoverage": {}}
    for name in ("repositories", "sourceCandidates", "outputCandidates", "links", "errors"):
        result[name] = [row for report in reports for row in report[name]]
    for name in ("directories", "files", "logicalBytes"):
        result[name] = sum(report[name] for report in reports)
    for report in reports:
        result["rootCoverage"].update(report["rootCoverage"])
    result.update(durationSeconds=round(result["finishedAt"] - started, 2),
                  status="partial" if result["errors"] else "complete",
                  limitations=reports[0]["limitations"],
                  observation="Concurrent per-volume live observations; content mutations require fresh bounded scopes.")
    result["sourceContentHashes"] = reports[0]["sourceContentHashes"]
    result["inventoryDigest"] = digest(result)
    return result



def _managed(args):
    import base64
    from tools.jenkins.resources import canonical_build_root
    from .inventory import MANAGED_ROOTS, scan_managed
    from .worker import _json
    raw_roots = json.loads(base64.b64decode(args.roots_base64, validate=True))
    if raw_roots != MANAGED_ROOTS or args.resume_base64:
        raise ValueError("managed_inventory_requires_fixed_D_E_F_roots")
    destination = build_namespace(args.output)
    index_path = physical(args.managed_index)
    if (destination.exists() or index_path != destination.parent / "managed-index.json"
            or destination.name != "inventory.json"):
        raise ValueError("managed_inventory_control_paths_changed")
    reference = file_fact(index_path)
    if reference["sha256"] != args.managed_index_sha256:
        raise ValueError("managed_inventory_index_changed")
    index = json.loads(index_path.read_text(encoding="utf-8"))
    if (index.get("inventoryPolicy") != "managed-roots-v1"
            or digest({k: v for k, v in index.items() if k != "indexDigest"}) != index.get("indexDigest")):
        raise ValueError("managed_inventory_index_digest_changed")
    admissions = {}
    for raw in raw_roots:
        try:
            approved = canonical_build_root(raw, allow_missing=True)
            admissions[str(Path(raw))] = {"verified": True, "volumeIdentity": approved.volume_identity}
        except (OSError, ValueError, RuntimeError) as error:
            admissions[str(Path(raw))] = {"verified": False, "reason": type(error).__name__}
    result = scan_managed([Path(raw) for raw in raw_roots], index, admissions=admissions,
                          progress=lambda value: print(json.dumps(value), flush=True))
    for raw in raw_roots:
        proof = admissions[str(Path(raw))]
        if not proof["verified"]:
            continue
        try:
            current = canonical_build_root(raw, allow_missing=True)
            if current.volume_identity != proof["volumeIdentity"]:
                raise ValueError("managed_inventory_volume_changed")
            proof["afterVerified"] = True
        except (OSError, ValueError, RuntimeError) as error:
            proof["afterVerified"] = False
            result["errors"].append({"path": raw, "reason": type(error).__name__})
            result["status"] = "partial"
    if file_fact(index_path) != reference:
        raise ValueError("managed_inventory_index_changed")
    result["inventoryDigest"] = digest({k: v for k, v in result.items() if k != "inventoryDigest"})
    _json(destination, result)
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True)
    parser.add_argument("--roots-base64", required=True)
    parser.add_argument("--registrations-base64", default="e30=")
    parser.add_argument("--resume-base64")
    parser.add_argument("--inventory-policy", default="whole-volume-v1", choices=["whole-volume-v1", "managed-roots-v1"])
    parser.add_argument("--managed-index")
    parser.add_argument("--managed-index-sha256")
    args = parser.parse_args()
    if args.inventory_policy == "managed-roots-v1":
        return _managed(args)
    import base64
    roots = [physical(value) for value in json.loads(base64.b64decode(args.roots_base64, validate=True))]
    if roots != [Path("C:/"), Path("D:/"), Path("E:/")]:
        raise ValueError("whole_volume_roots_must_be_C_D_E")
    destination = build_namespace(args.output)
    if destination.exists():
        raise ValueError("inventory_output_already_exists")
    registrations = json.loads(base64.b64decode(args.registrations_base64, validate=True))
    from .worker import _json
    lock = threading.Lock()
    def emit(value):
        with lock:
            print(json.dumps(value, ensure_ascii=True), flush=True)
    checkpoint_root = build_namespace(destination.parent / "checkpoints")
    checkpoint_root.mkdir(exist_ok=False)
    def volume(root):
        directory = checkpoint_root / root.drive.rstrip(":").lower()
        directory.mkdir()
        (directory / "repositories").mkdir()
        def checkpoint(phase, value):
            path = directory / "metadata.json" if phase == "metadata" else directory / "repositories" / (digest(value["path"]) + ".json")
            _json(path, value)
            if phase == "metadata":
                emit({"phase": "metadata-checkpoint", "root": str(root), "reference": file_fact(path),
                      "directories": value["directories"], "files": value["files"],
                      "repositoryCount": len(value["repositoryRoots"]), "rootCoverage": value["rootCoverage"]})
        value = scan([root], progress=emit, registrations=registrations, checkpoint=checkpoint)
        _json(directory / "complete.json", value)
        emit({"phase": "volume-complete", "root": str(root), "status": value["status"],
              "rootCoverage": value["rootCoverage"], "repositories": len(value["repositories"])})
        return value
    started = time.time()
    # One retained native Job owns all three threads and their Git subprocesses.
    # Metadata and each repository are durable before proceeding to another phase.
    if args.resume_base64:
        from .resumption import observe
        specification = json.loads(base64.b64decode(args.resume_base64, validate=True))
        reports = observe(specification, roots, registrations, checkpoint_root, emit)
    else:
        with ThreadPoolExecutor(max_workers=3) as executor:
            reports = list(executor.map(volume, roots))
    result = combine(roots, reports, started)
    if args.resume_base64:
        result["resumedFrom"] = specification
        result["repositoryObservationResumedAt"] = started
        result["sourceContentHashes"] = specification["sourceContentHashes"]
        result["inventoryDigest"] = digest({key: value for key, value in result.items() if key != "inventoryDigest"})
    _json(destination, result)
    print(json.dumps({"status": result["status"], "directories": result["directories"],
        "files": result["files"], "repositories": len(result["repositories"]),
        "errors": len(result["errors"]), "inventoryDigest": result["inventoryDigest"]}), flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

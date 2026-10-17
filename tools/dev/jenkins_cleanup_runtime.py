"""Read-only, hash-pinned runtime binding for a new Jenkins cleanup run.

A timer invocation may use a restarted runtime only with the installed sealed
driver and fresh native owner observations. Later steps retain that exact fence.
This entry never starts a service, changes State, or releases historical holds.
"""
from __future__ import annotations

import argparse
import datetime as dt
import json
from pathlib import Path
import re
import stat
import sys


def _read(path: Path) -> bytes:
    for parent in (path, *path.parents):
        if parent.lstat().st_file_attributes & stat.FILE_ATTRIBUTE_REPARSE_POINT:
            raise ValueError("cleanup_runtime_path_alias")
    return path.read_bytes()


def binding_from_records(repo: Path, expected_driver: str, selected: dict,
                         operation: dict, host: dict) -> dict:
    """Validate deployment agreement before any native identity is trusted."""
    if not re.fullmatch(r"[0-9a-f]{64}", expected_driver):
        raise ValueError("cleanup_driver_digest_invalid")
    if selected.get("driverDigest") != expected_driver:
        raise ValueError("cleanup_driver_changed_reinstall_required")
    sealed = repo / ".jenkins/runtime/drivers" / expected_driver
    for key, path in (("root", sealed), ("steps", sealed / "pipeline/steps.groovy"),
                      ("launcher", sealed / "launcher.py")):
        if Path(selected.get(key, "")).absolute() != path.absolute():
            raise ValueError("cleanup_sealed_paths_changed")
    runtime = selected.get("runtimeOperationId")
    if not isinstance(runtime, str) or not re.fullmatch(r"[A-Za-z0-9_.-]+", runtime):
        raise ValueError("cleanup_runtime_identity_invalid")
    if (operation.get("operationId") != runtime or operation.get("generation") != runtime
            or operation.get("state") != "running" or operation.get("health") is not True
            or host.get("operationId") != runtime or host.get("generation") != runtime
            or host.get("status") != "running" or host.get("hostPid") != operation.get("pid")):
        raise ValueError("cleanup_runtime_records_disagree")
    owners = {"host": operation, "controller": host.get("controllerIdentity"),
              "agent": host.get("agentIdentity")}
    identities = {}
    for role, owner in owners.items():
        if (not isinstance(owner, dict) or not isinstance(owner.get("pid"), int)
                or owner["pid"] <= 0 or not str(owner.get("creationTime", "")).isdigit()
                or not owner.get("executable")
                or not re.fullmatch(r"[0-9a-f]{64}", str(owner.get("commandDigest", "")))):
            raise ValueError("cleanup_native_owner_identity_missing")
        identities[role] = {key: owner[key] for key in
                            ("pid", "creationTime", "executable", "commandDigest")}
    if len({owner["pid"] for owner in identities.values()}) != len(identities):
        raise ValueError("cleanup_native_owner_identity_ambiguous")
    return {"driverDigest": expected_driver, "runtimeOperationId": runtime,
            "sealedRoot": str(sealed), "nativeOwners": identities}


def observe_binding(repo: Path, expected_driver: str) -> dict:
    if repo != Path("E:/Git/ZirconEngine"):
        raise ValueError("cleanup_repository_identity_changed")
    if Path(sys.executable).absolute() != repo / ".jenkins/runtime/python/python.exe":
        raise ValueError("cleanup_requires_managed_job_interpreter")
    paths = {"selected": repo / ".jenkins/state/deployment/driver.json",
             "operation": repo / ".jenkins/state/deployment-operation.json",
             "host": repo / ".jenkins/state/deployment/host.json"}
    raw = {key: _read(path) for key, path in paths.items()}
    records = {key: json.loads(value) for key, value in raw.items()}
    proof = binding_from_records(repo, expected_driver, **records)
    # Resolve imports in the reviewed immutable implementation, independently
    # of the concurrently edited canonical deployment modules.
    sealed = Path(proof["sealedRoot"])
    _read(sealed / "driver-manifest.json")
    sys.path.insert(0, str(sealed))
    from tools.jenkins.deployment import driver
    if Path(driver.__file__).absolute() != sealed / "tools/jenkins/deployment/driver.py":
        raise ValueError("cleanup_driver_import_context_changed")
    driver.verify_driver(sealed, expected_digest=expected_driver)
    from tools.jenkins.deployment.live_observations import observe_live_identity, observe_endpoint_owner
    proof["nativeOwners"] = {role: observe_live_identity(owner)
                            for role, owner in proof["nativeOwners"].items()}
    proof["endpoint"] = observe_endpoint_owner(
        "127.0.0.1", 18080, proof["nativeOwners"]["controller"]["pid"])
    if any(_read(path) != raw[key] for key, path in paths.items()):
        raise ValueError("cleanup_runtime_changed_during_observation")
    proof["verifiedAt"] = dt.datetime.now(dt.timezone.utc).isoformat()
    return proof


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", required=True, type=Path)
    parser.add_argument("--expected-driver-digest", required=True)
    args = parser.parse_args()
    print(json.dumps(observe_binding(args.repo_root.absolute(), args.expected_driver_digest)))


if __name__ == "__main__":
    main()

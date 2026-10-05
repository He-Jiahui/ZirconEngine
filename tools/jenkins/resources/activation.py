"""Versioned deployment-owned policy activation without rewriting historical holds."""
from __future__ import annotations

import hashlib
import time
from pathlib import Path

from ..contracts import JenkinsError, digest
from ..deployment.spec import load_spec
from .capacity import Capacity, ResourceManager
from .paths import canonical_build_root, physical_path_under


def _active_binding(state, connection=None):
    return {domain: [row for row in state.list(domain, connection=connection)
                     if row["payload"].get("status") == "active"]
            for domain in ("resource_reservation", "pool_writer_hold")}


def activate_policy(state, repo_root: Path, payload: dict) -> dict:
    version = payload.get("expectedPolicyVersion")
    expected_sha = payload.get("expectedSpecSha256")
    if type(version) is not int or version < 0 or not isinstance(expected_sha, str) or len(expected_sha) != 64:
        raise JenkinsError("activation_binding_missing", "Exact policy version and deployment spec SHA256 are required")
    if any(key in payload for key in ("policy", "resourcePolicy", "cpuBudget", "memoryBudget", "diskBudget", "allowedBuildRoots", "recipeEstimates")):
        raise JenkinsError("activation_policy_untrusted", "Policy must come from the deployment specification")
    path = Path(repo_root) / ".jenkins/deployment-spec.json"
    raw = path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != expected_sha:
        raise JenkinsError("deployment_spec_changed", "Deployment specification digest differs")
    spec = load_spec(path)
    policy = spec.storage.get("resourcePolicy")
    fields = ("cpuBudget", "memoryBudget", "diskBudget", "maxHeavyWriters", "inventoryMaxAgeSeconds", "diskReserveBytes")
    if not isinstance(policy, dict) or any(type(policy.get(key)) is not int or policy[key] <= 0 for key in fields):
        raise JenkinsError("resource_policy_invalid", "Complete positive integer resource budgets are required")
    estimates = policy.get("recipeEstimates")
    if not isinstance(estimates, dict) or "default" not in estimates:
        raise JenkinsError("resource_policy_invalid", "Trusted recipe estimates require a default profile")
    for name, estimate in estimates.items():
        if (not isinstance(name, str) or not name or not isinstance(estimate, dict)
                or any(type(estimate.get(field)) is not int or not 0 < estimate[field] <= policy[budget]
                       for field, budget in (("cpu", "cpuBudget"), ("memoryBytes", "memoryBudget"),
                                             ("diskBytes", "diskBudget")))):
            raise JenkinsError("resource_policy_invalid", "Recipe estimates must be positive integers within policy budgets")
    root = canonical_build_root(str(spec.build_root))
    if payload.get("buildRoot") is not None and str(payload["buildRoot"]).replace("/", "\\").casefold() != str(root.path).casefold():
        raise JenkinsError("build_root_not_current", "Explicit activation root must match deployment; no fallback")
    namespace = physical_path_under(root, root.namespace())
    old = state.get("resource_policy", "default")
    if (old["version"] if old else 0) != version:
        raise JenkinsError("state_conflict", "Policy version changed", retryable=True)
    active = _active_binding(state)
    namespace.mkdir(parents=True, exist_ok=True)
    physical_path_under(root, namespace, allow_missing=False)
    manager = ResourceManager(state, Capacity(policy["cpuBudget"], policy["memoryBudget"], policy["diskBudget"]))
    inventory = manager.capture_inventory(str(root.path))
    if path.read_bytes() != raw:
        raise JenkinsError("deployment_spec_changed", "Deployment changed during activation")
    if canonical_build_root(str(root.path)).volume_identity != root.volume_identity:
        raise JenkinsError("build_root_alias", "Physical build volume changed during activation")
    physical_path_under(root, namespace, allow_missing=False)
    with state.transaction() as connection:
        if digest(_active_binding(state, connection)) != digest(active):
            raise JenkinsError("activation_holds_changed", "Active holds changed during inventory capture", retryable=True)
        if (inventory.get("complete") is not True or inventory.get("buildRoot") != str(root.path)
                or type(inventory.get("observedAt")) not in (float, int)
                or any(type(inventory.get(key)) is not int or inventory[key] <= 0
                       for key in ("cpu", "memoryBytes", "diskBytes"))
                or not isinstance(inventory.get("generation"), str) or not inventory["generation"]
                or not 0 <= time.time() - inventory["observedAt"] <= policy["inventoryMaxAgeSeconds"]):
            raise JenkinsError("storage_snapshot_stale", "Fresh complete inventory is required", retryable=True)
        current_policy = {**policy, "buildRoot": str(root.path),
                          "allowedBuildRoots": policy.get("allowedBuildRoots", spec.storage["allowedPhysicalRoots"]),
                          "deploymentSpecSha256": expected_sha}
        record = state.put("resource_policy", "default", current_policy, expected_version=version, connection=connection)
        inventory_record = state.put("capacity_inventory", inventory["generation"], inventory, expected_version=0, connection=connection)
        registration = state.put("managed_storage_registration", str(namespace),
                                 {"buildRoot": str(root.path), "namespace": str(namespace),
                                  "volumeIdentity": root.volume_identity, "deploymentSpecSha256": expected_sha,
                                  "policyVersion": record["version"], "inventoryGeneration": inventory["generation"]}, connection=connection)
        result = {"policy": record, "inventory": inventory_record, "registration": registration,
                  "preservedActiveDigest": digest(active),
                  "preservedActiveReservations": len(active["resource_reservation"]),
                  "preservedActiveWriterHolds": len(active["pool_writer_hold"])}
        state.event("resource_policy_activated", {"policyVersion": record["version"], "specSha256": expected_sha,
                    "inventoryGeneration": inventory["generation"], "preservedActiveDigest": digest(active)}, connection=connection)
        return result

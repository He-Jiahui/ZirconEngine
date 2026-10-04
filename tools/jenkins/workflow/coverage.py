"""Deterministic, non-executing coverage and recipe planning."""
from __future__ import annotations
from typing import Any, Iterable, Mapping
from .cargo_adapter import build_adapter_payload, package_for_path, parse_metadata

def discover_selected_test_count(inventory: Any, *, package_ids: Iterable[str] = (), targets: Iterable[str] = ()) -> int:
    """Count tests in supplied discovery evidence; never invokes Cargo."""
    wanted_packages, wanted_targets = set(map(str, package_ids)), set(map(str, targets))
    if isinstance(inventory, Mapping):
        for key in ("tests", "selectedTests", "items"):
            if key in inventory: inventory = inventory[key]; break
        else: return 0
    if not isinstance(inventory, (list, tuple)): return 0
    count = 0
    for item in inventory:
        if not isinstance(item, Mapping): continue
        package = str(item.get("packageId", item.get("package", "")))
        target = str(item.get("target", item.get("targetName", "")))
        if wanted_packages and package not in wanted_packages: continue
        if wanted_targets and target not in wanted_targets: continue
        count += int(item.get("count", 1))
    return count

def build_recipe_registry_payload(metadata: Mapping[str, Any], changed_paths=(), *, features=(), profile=None, test_inventory=None, recipe_id="cargo-selected-tests", source_digest=None, coverage_digest=None, source_manifest_ref=None, toolchain="stable", cargo_path="cargo", wrapper=None, environment=None, preparation_key=None, caller_regression=None):
    metadata = parse_metadata(metadata)
    package_ids = set()
    for path in changed_paths: package_ids.update(package_for_path(metadata, path))
    if changed_paths and not package_ids:
        raise ValueError("unknown impact: changed paths do not map to Cargo packages")
    adapter = build_adapter_payload(metadata, package_ids, profile=profile, features=features)
    discovered = discover_selected_test_count(test_inventory, package_ids=adapter["packageIds"], targets=adapter["targets"]) if test_inventory is not None else 0
    if discovered <= 0: raise ValueError("selected test count must be positive")
    recipe = {"id": recipe_id, "kind": "cargo", "readOnly": True,
              "packages": adapter["packages"], "packageIds": adapter["packageIds"],
              "targets": adapter["targets"], "features": adapter["features"],
              "profile": adapter["profile"], "testCount": discovered,
              "command": [cargo_path, "test", "--workspace", "--no-run"],
              "toolchain": toolchain, "wrapper": wrapper, "environment": dict(environment or {}),
              "sourceDigest": source_digest, "coverageDigest": coverage_digest,
              "sourceManifestRef": source_manifest_ref, "preparationKey": preparation_key,
              "callerRegression": caller_regression or {"templates": ["module_unit", "cross_module", "failure_repair", "comments_only", "cache_maintenance"]}}
    return {"recipes": [recipe], "selected": adapter}

def select_coverage(metadata, changed_paths=(), **kwargs):
    return build_recipe_registry_payload(metadata, changed_paths, **kwargs)

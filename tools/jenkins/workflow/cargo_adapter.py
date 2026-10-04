"""Read-only Cargo metadata adapters used by Jenkins coverage planning."""
from __future__ import annotations
import json
from typing import Any, Iterable, Mapping

def parse_metadata(value):
    if isinstance(value, str): value = json.loads(value)
    if not isinstance(value, Mapping): raise TypeError("Cargo metadata must be an object")
    return dict(value)

def parse_lock(value=None):
    if value is None: return {}
    if isinstance(value, str): value = json.loads(value)
    if not isinstance(value, Mapping): raise TypeError("Cargo lock must be an object")
    return dict(value)

def packages(metadata): return tuple(p for p in metadata.get("packages", ()) if isinstance(p, Mapping) and p.get("name"))
def package_index(metadata): return {str(p["id"]): p for p in packages(metadata) if p.get("id")}

def dependency_graph(metadata):
    ids = package_index(metadata); names = {str(p.get("name")): str(p.get("id")) for p in ids.values()}
    graph = {}
    for p in packages(metadata):
        deps = set()
        for d in p.get("dependencies", ()) or ():
            if isinstance(d, Mapping):
                did = d.get("pkg") or names.get(str(d.get("name")))
                if did in ids: deps.add(str(did))
        graph[str(p["id"])] = deps
    return graph

def reverse_dependencies(metadata):
    out = {pid: set() for pid in package_index(metadata)}
    for owner, deps in dependency_graph(metadata).items():
        for dep in deps: out.setdefault(dep, set()).add(owner)
    return out

def affected_package_ids(metadata, package_ids: Iterable[str] = (), include_reverse=True):
    selected = {str(x) for x in package_ids}
    if include_reverse:
        todo = list(selected); reverse = reverse_dependencies(metadata)
        while todo:
            for child in reverse.get(todo.pop(), ()):
                if child not in selected: selected.add(child); todo.append(child)
    return selected

def targets(package): return tuple(t for t in package.get("targets", ()) or () if isinstance(t, Mapping))
def test_targets(package): return tuple(t for t in targets(package) if "test" in (t.get("kind") or ()))
def selected_test_count(metadata, package_ids=None, *, list_output=None):
    """Count only tests discovered by Cargo ``--list`` output.

    Metadata target count is intentionally never treated as a test count.
    """
    if list_output is None: return 0
    wanted = {str(x) for x in package_ids} if package_ids is not None else None
    if isinstance(list_output, Mapping): list_output = list_output.get("tests", list_output.get("items", ()))
    if not isinstance(list_output, (list, tuple)): return 0
    return sum(int(x.get("count", 1)) for x in list_output if isinstance(x, Mapping)
               and (not wanted or str(x.get("packageId", x.get("package", ""))) in wanted))
def package_features(package): return tuple(sorted(str(k) for k in (package.get("features") or {})))
def select_profile(metadata, requested=None): return requested or "dev"

def package_for_path(metadata, path):
    path = str(path).replace("\\", "/"); out = set()
    for p in packages(metadata):
        for key in ("manifest_path",):
            if path in str(p.get(key, "")).replace("\\", "/"): out.add(str(p["id"]))
        for t in targets(p):
            if path in str(t.get("src_path", "")).replace("\\", "/"): out.add(str(p["id"]))
    return out

def build_adapter_payload(metadata, package_ids, *, profile=None, features=()):
    ids = affected_package_ids(metadata, package_ids); byid = package_index(metadata)
    selected = [byid[i] for i in sorted(ids) if i in byid]
    return {"packages": [str(p["name"]) for p in selected], "packageIds": sorted(ids),
            "targets": sorted(t["name"] for p in selected for t in test_targets(p) if t.get("name")),
            "features": sorted(set(map(str, features))), "profile": select_profile(metadata, profile),
            "testCount": selected_test_count(metadata, ids)}

parse_cargo_metadata = parse_metadata
reverse_dependency_graph = reverse_dependencies
discover_test_count = selected_test_count

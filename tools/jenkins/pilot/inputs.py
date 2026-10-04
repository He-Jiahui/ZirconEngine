"""Fixed-package locked Cargo closure with an explicit public transport.

This planner has no coordinator state or mutable global transport binding.
Cargo's complete dependency graph supplies active compilation packages; all
local manifest edges remain present so the copied workspace resolves unchanged.
"""
from __future__ import annotations

import json
import hashlib
import os
from pathlib import Path
import tomllib

from .contracts import PilotError
from .snapshot import _git, _read_plain, portable_path
from .source_manifest import file_hash
from .storage import require_managed_root

DECLARED_COMMANDS = (("cargo", "check", "--locked", "-p", "zircon_reflect_derive"),)
METADATA_TIMEOUT_SECONDS = 180.0


def live_git_paths(root: Path) -> set[str]:
    return {portable_path(name) for name in _git(root, "ls-files", "--cached", "--others", "--exclude-standard", "-z").decode("utf-8").split("\0") if name}


def guard_input(root: Path, relative: str) -> Path:
    name = portable_path(relative)
    if name != relative:
        raise PilotError("Cargo input path must be canonical")
    # Stable reads check every component, hard links and reparse points.
    if _read_plain(root, name) is None:
        raise PilotError("selected Cargo input is missing")
    return root / name


def input_fingerprint(root: Path, relative: str) -> str | None:
    if portable_path(relative) != relative:
        raise PilotError("Cargo input path must be canonical")
    data = _read_plain(root, relative)
    return hashlib.sha256(data).hexdigest() if data is not None else None


def _source_root(source: Path, path: Path) -> Path:
    path = Path(os.path.abspath(path))
    if path.is_relative_to(source):
        return source
    # Only the declared existing sibling repository is admitted.
    external = source.parent / "zr_vm"
    if path.is_relative_to(external):
        top = Path(_git(external, "rev-parse", "--show-toplevel").decode("utf-8").strip())
        if top != external:
            raise PilotError("external Cargo dependency must own its Git repository")
        return external
    raise PilotError("Cargo path dependency escaped the admitted source roots")


def _path_edges(value, directory: Path):
    if isinstance(value, dict):
        if isinstance(value.get("path"), str):
            yield Path(os.path.abspath(directory / value["path"])) / "Cargo.toml"
        for key, nested in value.items():
            # Target/lib/bin source paths are files, not dependency packages.
            if key in {"dependencies", "dev-dependencies", "build-dependencies", "patch", "replace", "workspace", "target"} or isinstance(nested, dict) and "path" in nested and key not in {"lib", "package"}:
                yield from _dependency_edges(nested, directory)


def _dependency_edges(value, directory: Path):
    if isinstance(value, dict):
        if isinstance(value.get("path"), str):
            yield Path(os.path.abspath(directory / value["path"])) / "Cargo.toml"
        for nested in value.values():
            yield from _dependency_edges(nested, directory)


def manifest_closure(source: Path):
    source = Path(os.path.abspath(source))
    pending, selected, topology = [source / "Cargo.toml"], {}, {}
    while pending:
        manifest = pending.pop()
        root = _source_root(source, manifest)
        relative = manifest.relative_to(root).as_posix()
        key = (root, relative)
        if key in selected:
            continue
        data = _read_plain(root, relative)
        if data is None:
            raise PilotError("Cargo manifest dependency is missing")
        selected[key] = file_hash(root / relative)
        try:
            parsed = tomllib.loads(data.decode("utf-8"))
        except (ValueError, UnicodeError) as error:
            raise PilotError("Cargo manifest is invalid") from error
        pending.extend(_path_edges(parsed, manifest.parent))
        workspace = parsed.get("workspace", {})
        if not isinstance(workspace, dict):
            raise PilotError("Cargo workspace is invalid")
        for member in workspace.get("members", []):
            if not isinstance(member, str) or member.startswith(("/", "\\")) or ".." in Path(member).parts:
                raise PilotError("workspace member path is unsafe")
            matches = list(manifest.parent.glob(member))
            if not matches:
                raise PilotError("workspace member is missing")
            pending.extend(path / "Cargo.toml" for path in matches)
        # A dependency may inherit its containing sibling workspace.
        for parent in manifest.parents:
            if parent == root.parent:
                break
            ancestor = parent / "Cargo.toml"
            if ancestor != manifest and ancestor.is_file():
                raw = _read_plain(root, ancestor.relative_to(root).as_posix())
                if raw and "workspace" in tomllib.loads(raw.decode("utf-8")):
                    pending.append(ancestor)
                    break
    for root, name in selected:
        if root == source:
            topology[name] = selected[root, name]
    for relative in ("Cargo.lock", "rust-toolchain", "rust-toolchain.toml", ".cargo/config", ".cargo/config.toml"):
        if (source / relative).is_file():
            selected[source, relative] = file_hash(guard_input(source, relative))
            topology[relative] = selected[source, relative]
    if "Cargo.lock" not in topology:
        raise PilotError("locked Cargo closure requires the repository lockfile")
    return selected, topology


def select_metadata_inputs(source: Path, metadata: dict, manifests: dict) -> dict:
    packages = metadata.get("packages")
    resolve = metadata.get("resolve")
    if not isinstance(packages, list) or not isinstance(resolve, dict) or not isinstance(resolve.get("nodes"), list):
        raise PilotError("locked Cargo metadata lacks the complete dependency graph")
    package_map, graph = {}, {}
    for package in packages:
        if not isinstance(package, dict) or not isinstance(package.get("id"), str) or package["id"] in package_map:
            raise PilotError("Cargo metadata package identity is invalid")
        package_map[package["id"]] = package
    selected = [p["id"] for p in packages if p.get("name") == "zircon_reflect_derive" and p.get("source") is None]
    if len(selected) != 1:
        raise PilotError("Cargo metadata must resolve exactly the declared local package")
    for node in resolve["nodes"]:
        if not isinstance(node, dict) or node.get("id") not in package_map or node["id"] in graph:
            raise PilotError("Cargo metadata graph node is invalid")
        dependencies = node.get("dependencies")
        if not isinstance(dependencies, list) or any(not isinstance(identity, str) or identity not in package_map for identity in dependencies):
            raise PilotError("Cargo metadata graph edge is invalid")
        graph[node["id"]] = dependencies
    pending, active = selected, set()
    while pending:
        identity = pending.pop()
        if identity in active:
            continue
        if identity not in graph:
            raise PilotError("Cargo metadata omitted an active dependency node")
        active.add(identity)
        pending.extend(graph[identity])
    result = dict(manifests)
    inventories = {}
    for package in packages:
        if package.get("source") is not None:
            continue
        manifest = Path(package.get("manifest_path", ""))
        if not manifest.is_absolute() or ".." in manifest.parts:
            raise PilotError("Cargo metadata manifest path must be absolute without aliases")
        root = _source_root(source, manifest)
        relative = manifest.relative_to(root).as_posix()
        result[root, relative] = file_hash(guard_input(root, relative))
        parsed = tomllib.loads(_read_plain(root, relative).decode("utf-8"))
        if parsed.get("package", {}).get("name") != package.get("name"):
            raise PilotError("Cargo metadata package name differs from the sealed manifest")
        if package["id"] in active:
            if root not in inventories:
                inventories[root] = live_git_paths(root)
            inventory = inventories[root]
            prefix = manifest.parent.relative_to(root).as_posix()
            # Whole active package input includes build scripts, include files,
            # data and generated checked-in inputs, not merely Rust extensions.
            for name in inventory:
                if prefix == "." or name.startswith(prefix + "/"):
                    # Git inventories retain tracked deletions. Seal their
                    # absence explicitly rather than dropping base provenance.
                    result[root, name] = input_fingerprint(root, name)
    return result


def plan_inputs(source: Path, target: Path, commands=DECLARED_COMMANDS, *, transport, metadata_cache=None):
    if commands != DECLARED_COMMANDS:
        raise PilotError("closure planner accepts only the fixed declared Cargo command")
    source = Path(os.path.abspath(source))
    target = require_managed_root(target, allow_root=False)
    manifests, topology = manifest_closure(source)
    inventories = {root: live_git_paths(root) for root in {owner for owner, _name in manifests}}
    key = (tuple(sorted((str(root), name, identity) for (root, name), identity in manifests.items())),
           tuple(sorted((str(root), tuple(sorted(names))) for root, names in inventories.items())))
    cache = metadata_cache if metadata_cache is not None else {}
    if key not in cache:
        # Tool binding is supplied by the independent executor, not ambient PATH.
        with metadata_execution(source, target, input_binding={
                str(root / name): identity for (root, name), identity in manifests.items()}) as binding:
            try:
                result = transport(binding.command, source, binding.environment)
            except BaseException as error:
                observation = getattr(error, "native_observation", None)
                if observation is not None:
                    binding.complete(observation)
                raise
            binding.complete(result.native_observation)
        if result.returncode != 0:
            raise PilotError("locked Cargo metadata failed: " + result.stderr[-8192:])
        try:
            cache[key] = json.loads(result.stdout)
        except (ValueError, TypeError) as error:
            raise PilotError("locked Cargo metadata is invalid JSON") from error
    if not isinstance(cache[key], dict):
        raise PilotError("locked Cargo metadata must be an object")
    selected = select_metadata_inputs(source, cache[key], manifests)
    for (root, name), identity in selected.items():
        if input_fingerprint(root, name) != identity:
            raise PilotError("Cargo closure changed during planning")
    if any(live_git_paths(root) != paths for root, paths in inventories.items()):
        raise PilotError("Cargo source topology changed during planning")
    primary = {name: identity for (root, name), identity in selected.items() if root == source}
    external = {root: {name: identity for (owner, name), identity in selected.items() if owner == root}
                for root in inventories if root != source}
    return primary, external, topology


def metadata_execution(source: Path, target: Path, *, input_binding: dict[str, str]):
    from .managed_cargo import metadata_execution as bind_metadata
    # Ancestor configs would redirect source/cache/tool execution outside the
    # admitted manifest. The direct executor deliberately accepts none.
    for parent in source.parents:
        if any((parent / ".cargo" / name).exists() for name in ("config", "config.toml")):
            raise PilotError("ambient ancestor Cargo configuration is forbidden")
    return bind_metadata(source, target, input_binding=input_binding)

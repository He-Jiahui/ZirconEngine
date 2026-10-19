"""One explicitly hashed sibling dependency, sealed and prepared by Jenkins."""
from __future__ import annotations

from pathlib import Path, PurePosixPath
from typing import Mapping

from ..contracts import JenkinsError, digest, file_digest, require_digest

KIND = "sealed-external-source"


def normalize_current(repository_root: Path, values) -> list[dict]:
    if not isinstance(values, list) or len(values) > 1:
        raise JenkinsError("external_scope_invalid", "External inputs must be an explicit optional zr_vm entry")
    from ..current_source import _hashes, _paths, _verify_current
    from ..deployment.driver import _plain
    result = []
    for value in values:
        if (not isinstance(value, dict) or set(value) != {"name", "repositoryRoot", "baseHead", "currentHashes", "untrackedPaths"}
                or value.get("name") != "zr_vm"):
            raise JenkinsError("external_scope_invalid", "Only the fixed zr_vm dependency metadata is supported")
        expected = repository_root.absolute().parent / "zr_vm"
        supplied = Path(value.get("repositoryRoot", ""))
        _plain(expected)
        if (not supplied.is_absolute() or supplied.absolute() != expected or not expected.is_dir()
                or expected.resolve() != expected):
            raise JenkinsError("external_repository_invalid", "zr_vm must be the physical canonical sibling checkout")
        base = value.get("baseHead")
        if not isinstance(base, str) or len(base) not in {40, 64} or any(ch not in "0123456789abcdef" for ch in base):
            raise JenkinsError("external_base_invalid", "External dependency requires an immutable Git base")
        hashes = _hashes(expected, value.get("currentHashes"))
        if any(path.split("/", 1)[0] in {".git", ".codex", ".jenkins", ".opencode", "target"} for path in hashes):
            raise JenkinsError("external_control_forbidden", "External inputs cannot include control or compiler output files")
        untracked = _paths(expected, value.get("untrackedPaths"), "untrackedPaths")
        if not set(untracked) <= set(hashes):
            raise JenkinsError("external_scope_invalid", "External untracked paths must belong to their exact closure")
        entry = {"name": "zr_vm", "repositoryRoot": str(expected), "baseHead": base,
                 "currentHashes": hashes, "untrackedPaths": untracked}
        _verify_current(expected, entry)
        result.append(entry)
    return result


def authorization_paths(values: list[dict]) -> list[str]:
    return ["external/zr_vm/" + path for value in values for path in value["currentHashes"]]


def verify_current(values: list[dict]) -> None:
    from ..current_source import _verify_current
    for value in values:
        _verify_current(Path(value["repositoryRoot"]), value)


def seal(values: list[dict], objects: Path) -> list[dict]:
    from . import PathClaim, seal_candidate, _thaw
    result = []
    verify_current(values)
    for value in values:
        claims = [PathClaim(path, "dependency", 0, checksum, 0) for path, checksum in value["currentHashes"].items()]
        manifest = seal_candidate(Path(value["repositoryRoot"]), claims, object_root=objects, base_head=value["baseHead"])
        result.append({"kind": KIND, "name": "zr_vm", "repositoryRoot": value["repositoryRoot"],
                       "manifest": _thaw(manifest.payload), "sourceDigest": manifest.source_digest})
    verify_current(values)
    return result


def sources(manifest: Mapping) -> list[dict]:
    values = manifest.get("external", [])
    if manifest.get("preparationKind") not in {"current-source", "joint-input"} or not values:
        return []
    if not isinstance(values, (list, tuple)) or len(values) != 1:
        raise JenkinsError("external_manifest_invalid", "Current-source external manifest is invalid")
    value = values[0]
    if (not isinstance(value, Mapping) or value.get("kind") != KIND or value.get("name") != "zr_vm"
            or not isinstance(value.get("manifest"), Mapping) or value["manifest"].get("external")
            or digest(value["manifest"]) != value.get("sourceDigest")):
        raise JenkinsError("external_manifest_invalid", "External dependency manifest differs from its exact source identity")
    return [value]


def flattened(manifest: Mapping) -> dict:
    values = sources(manifest)
    if not values:
        return dict(manifest)
    entries = [{**entry, "path": "source/" + entry["path"]} for entry in manifest["entries"]]
    for value in values:
        entries.extend({**entry, "path": value["name"] + "/" + entry["path"]} for entry in value["manifest"]["entries"])
    return {**manifest, "entries": sorted(entries, key=lambda entry: entry["path"])}


def verify_objects(manifest: Mapping, objects: Path) -> None:
    from . import canonical_path
    store = objects / "inputs/objects"
    for value in sources(manifest):
        for entry in value["manifest"]["entries"]:
            path = str(entry.get("path", ""))
            if (not path or "\\" in path or ":" in path or path.startswith("/")
                    or any(part in {"", ".", ".."} for part in path.split("/"))):
                raise JenkinsError("external_manifest_invalid", "External file path is not canonical")
            if entry.get("status") == "deleted":
                if entry.get("sha256") is not None:
                    raise JenkinsError("external_manifest_invalid", "Deleted external input has a content hash")
                continue
            checksum = require_digest(entry.get("sha256"), "externalHash")
            if entry.get("status") != "present" or entry.get("objectDigest") != checksum:
                raise JenkinsError("external_manifest_invalid", "External input object differs from its content hash")
            obj = store / checksum
            canonical_path(store, checksum)
            if not obj.is_file() or file_digest(obj) != checksum:
                raise JenkinsError("external_object_corrupt", "External dependency CAS object is missing or corrupt")


def materialize(manifest: Mapping, objects: Path, source: Path) -> None:
    from . import materialize_manifest
    values = sources(manifest)
    if not values:
        return
    if source.name != "source":
        raise JenkinsError("external_layout_invalid", "External dependencies require an isolated source bundle")
    allowed = {"source", *[value["name"] for value in values]}
    if any(path.name not in allowed for path in source.parent.iterdir()):
        raise JenkinsError("external_layout_invalid", "Source bundle contains an unregistered sibling")
    verify_objects(manifest, objects)
    for value in values:
        materialize_manifest(value["manifest"], objects, source.parent / value["name"])


def verify_prepared(manifest: Mapping, source: Path, approved) -> None:
    from ..resources import physical_path_under
    values = sources(manifest)
    if not values:
        return
    if source.name != "source":
        raise JenkinsError("external_layout_invalid", "Prepared external dependencies require the isolated source layout")
    for value in values:
        root = physical_path_under(approved, source.parent / value["name"], allow_missing=False)
        present = [entry for entry in value["manifest"]["entries"] if entry["status"] == "present"]
        # Source claims use case-insensitive canonical paths. Windows preserves
        # directory spelling when materializing a differently cased sibling.
        expected = {entry["path"].casefold(): entry["sha256"] for entry in present}
        if len(expected) != len(present):
            raise JenkinsError("external_prepared_mismatch", "Sealed sibling contains colliding canonical paths")
        actual = {}
        for path in root.rglob("*"):
            physical_path_under(approved, path, allow_missing=False)
            if path.is_file():
                relative = path.relative_to(root).as_posix().casefold()
                if relative in actual:
                    raise JenkinsError("external_prepared_mismatch", "Prepared sibling contains colliding canonical paths")
                actual[relative] = file_digest(path)
        if actual != expected:
            raise JenkinsError("external_prepared_mismatch", "Prepared sibling dependency differs from its complete sealed closure")

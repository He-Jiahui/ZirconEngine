"""Explicit sibling repositories in the schema 2 sealed-input protocol.

Every selected sibling file has a payload, including unchanged Git files.
Reconstruction therefore needs neither sibling checkout bytes nor Git objects.
"""
from __future__ import annotations

import hashlib
import os
import stat
import zipfile
from pathlib import Path

from .native.paths import WORKER_SUBDIRECTORIES

from .contracts import PilotError, sha256_identity
from .storage import ManagedStorage, require_managed_root


def _mount(value: str) -> str:
    from .snapshot import portable_path, _admit_source
    name = portable_path(value)
    _admit_source(name)
    if name != value or "/" in name or name.casefold() in {*WORKER_SUBDIRECTORIES, "snapshots", "receipts", "sealed"}:
        raise PilotError("external mount must be an unreserved canonical sibling name")
    return name


def _plain_root(root: Path) -> None:
    for path in (root, *root.parents):
        info = path.lstat()
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise PilotError("external repository cannot cross links or reparse points")
    if not root.is_dir():
        raise PilotError("external repository root is not a directory")


def prepare(repo_root: Path, inputs: dict[Path, dict[str, str | None]]) -> list[tuple]:
    from .snapshot import _git, _GIT_OBJECT, portable_path
    if len(inputs) > 1000:
        raise PilotError("external input exceeds the sibling limit")
    result, keys = [], {repo_root.name.casefold()}
    for supplied_root, selected in inputs.items():
        if ".." in Path(supplied_root).parts:
            raise PilotError("external root cannot contain traversal aliases")
        root = Path(os.path.abspath(supplied_root))
        _plain_root(root)
        if root.parent != repo_root.parent:
            raise PilotError("external input must be a direct repository sibling")
        mount = _mount(root.name)
        if mount.casefold() in keys:
            raise PilotError("external mount collides with another source")
        keys.add(mount.casefold())
        if not isinstance(selected, dict) or not selected or len(selected) > 100000:
            raise PilotError("external input requires a bounded explicit selection")
        for name, expected in selected.items():
            if not isinstance(name, str) or portable_path(name) != name:
                raise PilotError("external selection path is not canonical")
            if expected is not None:
                sha256_identity(expected, "external planner sha256")
        base = _git(root, "rev-parse", "HEAD").decode("ascii").strip()
        if not _GIT_OBJECT.fullmatch(base):
            raise PilotError("external source requires a fixed Git commit")
        if Path(_git(root, "rev-parse", "--show-toplevel").decode("utf-8").strip()).resolve() != root:
            raise PilotError("external input root must own its Git repository")
        result.append((root, mount, base, dict(selected)))
    return sorted(result, key=lambda item: item[1])


def capture(sources: list[tuple]) -> tuple[list[dict], dict[str, bytes]]:
    from .snapshot import _git, _base_tree, _blobs, _read_plain, _validate_entries, MAX_BUNDLE_BYTES
    siblings, payloads, payload_bytes = [], {}, 0
    for root, mount, base, selected in sources:
        _plain_root(root)
        if _git(root, "rev-parse", "HEAD").decode("ascii").strip() != base:
            raise PilotError("external Git base changed during sealing")
        tree = _base_tree(root, base)
        originals = _blobs(root, [tree[name][1] for name in selected if name in tree and tree[name][0] in {"100644", "100755"}])
        entries = []
        for name, expected in sorted(selected.items()):
            original = tree.get(name)
            if original is not None and original[0] not in {"100644", "100755"}:
                raise PilotError("external source contains a symlink or submodule")
            data = _read_plain(root, name)
            identity = hashlib.sha256(data).hexdigest() if data is not None else None
            if identity != expected or data is None and original is None:
                raise PilotError("external source differs from the planner input digest")
            state = "new" if original is None else "deleted" if data is None else "base" if data == originals[original[1]] else "modified"
            if data is not None:
                if identity not in payloads:
                    payload_bytes += len(data)
                payloads[identity] = data
                if payload_bytes > MAX_BUNDLE_BYTES:
                    raise PilotError("external source exceeds the small-pilot transfer limit")
            entries.append({"path": name, "status": state, "mode": original[0] if original else "100644",
                            "baseBlob": original[1] if original else None, "size": len(data) if data is not None else 0,
                            "sha256": identity, "payload": identity})
        _validate_entries(entries, ["."], full_payload=True)
        if _git(root, "rev-parse", "HEAD").decode("ascii").strip() != base:
            raise PilotError("external Git base changed during sealing")
        siblings.append({"mount": mount, "baseCommit": base, "entries": entries})
    return siblings, payloads


def validate(siblings: object) -> None:
    from .snapshot import _GIT_OBJECT, _validate_entries
    if not isinstance(siblings, list) or not siblings or len(siblings) > 1000:
        raise PilotError("external manifest must declare bounded sibling inputs")
    keys, names = set(), []
    for sibling in siblings:
        if not isinstance(sibling, dict) or set(sibling) != {"mount", "baseCommit", "entries"}:
            raise PilotError("external manifest fields are invalid")
        if not isinstance(sibling["mount"], str):
            raise PilotError("external mount is invalid")
        mount = _mount(sibling["mount"])
        if mount.casefold() in keys:
            raise PilotError("external mounts collide")
        keys.add(mount.casefold())
        names.append(mount)
        if not isinstance(sibling["baseCommit"], str) or not _GIT_OBJECT.fullmatch(sibling["baseCommit"]):
            raise PilotError("external manifest has no fixed Git base")
        _validate_entries(sibling["entries"], ["."], full_payload=True)
    if names != sorted(names):
        raise PilotError("external mounts are not canonically ordered")


def preflight_destination(siblings: list[dict], destination: Path) -> None:
    parent = require_managed_root(destination.parent, allow_root=False)
    if parent.exists() and any(parent.iterdir()):
        raise PilotError("external materialization requires a fresh sealed parent directory")
    for sibling in siblings:
        if sibling["mount"].casefold() == destination.name.casefold():
            raise PilotError("external mount collides with the primary destination")
        require_managed_root(parent / sibling["mount"], allow_root=False)


def materialize_external(sealed, destination: Path) -> None:
    with zipfile.ZipFile(sealed.bundle) as archive:
        for sibling in sealed.manifest["external"]:
            storage = ManagedStorage(destination.parent / sibling["mount"])
            with storage.backend() as backend:
                for entry in sibling["entries"]:
                    if entry["status"] != "deleted":
                        backend.write_bytes(entry["path"], archive.read(f"payloads/{entry['payload']}"))


def verify_external(siblings: list[dict], destination: Path) -> None:
    from .snapshot import _verify_entries
    parent = require_managed_root(destination.parent, allow_root=False)
    expected = {destination.name, *(sibling["mount"] for sibling in siblings)}
    if {item.name for item in parent.iterdir()} != expected:
        raise PilotError("sealed parent contains unexpected or missing sibling inputs")
    for sibling in siblings:
        _verify_entries(sibling["entries"], parent / sibling["mount"])

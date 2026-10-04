"""Independent, canonical content identities for sealed compiler inputs."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import stat

from .contracts import PilotError, sha256_identity


def file_hash(path: Path, *, allow_hardlinks: bool = False) -> str:
    path = Path(path)
    for ancestor in (path, *path.parents):
        info = ancestor.lstat()
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise PilotError("source identity cannot cross filesystem links")
    before = path.lstat()
    if not stat.S_ISREG(before.st_mode) or not allow_hardlinks and before.st_nlink != 1:
        raise PilotError("source identity requires a plain unaliased file")
    with path.open("rb") as stream:
        opened = os.fstat(stream.fileno())
        if (opened.st_dev, opened.st_ino) != (before.st_dev, before.st_ino):
            raise PilotError("source changed before identity read")
        identity = hashlib.file_digest(stream, "sha256").hexdigest()
    after = path.lstat()
    attributes = lambda value: (value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns, value.st_ctime_ns)
    if attributes(before) != attributes(after):
        raise PilotError("source changed during identity read")
    return identity


def tree_manifest(root: Path, *, exclude: frozenset[str] = frozenset()) -> dict[str, str]:
    from .snapshot import portable_path
    root = Path(root)
    for ancestor in (root, *root.parents):
        try:
            info = ancestor.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise PilotError("source tree root cannot cross filesystem links")
    if not root.exists():
        return {}
    result, components = {}, {}
    for parent, directories, files in os.walk(root, followlinks=False):
        directories[:] = [name for name in directories if (Path(parent) / name).relative_to(root).as_posix() not in exclude]
        for name in [*directories, *files]:
            path = Path(parent) / name
            relative = path.relative_to(root).as_posix()
            if relative in exclude:
                continue
            portable_path(relative)
            info = path.lstat()
            if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
                raise PilotError("source tree contains filesystem links")
            prior = components.setdefault(relative.casefold(), relative)
            if prior != relative:
                raise PilotError("source tree has case insensitive collisions")
            if name in files:
                result[relative] = file_hash(path)
    return result


def manifest_digest(manifest: dict[str, str]) -> str:
    from .snapshot import portable_path
    keys = set()
    for path, identity in manifest.items():
        if portable_path(path) != path or path.casefold() in keys:
            raise PilotError("source manifest has noncanonical or colliding paths")
        keys.add(path.casefold())
        sha256_identity(identity, "source sha256")
    payload = [{"path": path, "sha256": manifest[path]} for path in sorted(manifest, key=str.casefold)]
    return hashlib.sha256(json.dumps(payload, sort_keys=True, separators=(",", ":")).encode()).hexdigest()

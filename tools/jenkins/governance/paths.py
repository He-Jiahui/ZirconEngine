"""Physical identities shared by inventory, admission and reclamation."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import stat


REPARSE = getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0x400)


def encoded(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, ensure_ascii=False,
                      separators=(",", ":"), allow_nan=False).encode("utf-8")


def digest(value: object) -> str:
    return hashlib.sha256(encoded(value)).hexdigest()


def linked(info: os.stat_result) -> bool:
    return stat.S_ISLNK(info.st_mode) or bool(getattr(info, "st_file_attributes", 0) & REPARSE)


def physical(value: str | Path, *, exists: bool = True) -> Path:
    path = Path(value)
    if not path.is_absolute() or ".." in path.parts:
        raise ValueError("absolute_literal_path_required")
    for item in (path, *path.parents):
        try:
            if linked(item.lstat()):
                raise ValueError("path_alias_rejected")
        except FileNotFoundError:
            pass
    if os.path.normcase(str(path.resolve())) != os.path.normcase(str(path)):
        raise ValueError("path_alias_rejected")
    if exists and not path.exists():
        raise ValueError("physical_path_missing")
    return path


def file_identity(info: os.stat_result) -> list[int]:
    return [info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns, info.st_ctime_ns, info.st_nlink]


def _opened_identity(info: os.stat_result) -> list[int]:
    # Windows pathname stat and CRT fstat can expose different ctime semantics.
    # Device/file ID, mtime, size and link count have the same handle/path meaning.
    return [info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns, info.st_nlink]


def file_fact(path: Path) -> dict:
    physical(path)
    before = path.lstat()
    if not stat.S_ISREG(before.st_mode):
        raise ValueError("ordinary_file_required")
    checksum = hashlib.sha256()
    with path.open("rb") as stream:
        opened = os.fstat(stream.fileno())
        if _opened_identity(opened) != _opened_identity(before):
            raise ValueError("file_replaced_during_open")
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            checksum.update(chunk)
        if _opened_identity(os.fstat(stream.fileno())) != _opened_identity(before):
            raise ValueError("file_changed_during_read")
    if file_identity(path.lstat()) != file_identity(before):
        raise ValueError("file_changed_during_read")
    return {"path": str(path), "identity": file_identity(before),
            "bytes": before.st_size, "sha256": checksum.hexdigest()}


def build_namespace(value: str | Path) -> Path:
    path = physical(value, exists=False)
    allowed = [Path(f"{drive}:/cargo-targets") for drive in "DEF"]
    if not any(path.is_relative_to(root) and path != root for root in allowed):
        raise ValueError("compiler_namespace_rejected")
    return path


def relative_source(value: str) -> str:
    path = Path(value)
    normalized = value.replace("\\", "/")
    if (not value or path.is_absolute() or ":" in value or "\x00" in value
            or any(part in {"", ".", ".."} for part in normalized.split("/"))):
        raise ValueError("relative_source_path_required")
    if normalized.split("/")[0].casefold() in {".git", ".jenkins"}:
        raise ValueError("protected_control_path")
    return normalized


def overlaps(left: str | Path, right: str | Path) -> bool:
    a, b = str(left).replace("\\", "/").casefold().rstrip("/"), str(right).replace("\\", "/").casefold().rstrip("/")
    return a == b or a.startswith(b + "/") or b.startswith(a + "/")

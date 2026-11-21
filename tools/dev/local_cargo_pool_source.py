"""Bind an approved Cargo compiler pool to one physical source checkout.

Call this after the local entry validates and creates its output directories,
and before it starts Cargo. This is not a source-content seal. Cargo still owns
freshness within one checkout and its native locks. Do not share a bound pool
with tools that do not enforce this binding.
"""
from __future__ import annotations

import json
import os
from pathlib import Path
import stat


RECORD_NAME = ".zircon-cargo-pool-source-v1.json"
MAX_RECORD_BYTES = 8192


def _directory_fact(path: Path) -> dict[str, str | int]:
    if not path.is_absolute() or ".." in path.parts or path.resolve(strict=True) != path:
        raise ValueError(f"Cargo pool source binding requires a physical path: {path}")
    info = path.lstat()
    if (not stat.S_ISDIR(info.st_mode) or info.st_ino == 0
            or getattr(info, "st_file_attributes", 0) & stat.FILE_ATTRIBUTE_REPARSE_POINT):
        raise ValueError(f"Cannot identify physical Cargo pool directory: {path}")
    return {"path": os.path.normcase(str(path)), "device": info.st_dev, "fileId": info.st_ino}


def _check_record(marker: Path, expected: bytes) -> bool:
    try:
        info = marker.lstat()
    except FileNotFoundError:
        return False
    if (not stat.S_ISREG(info.st_mode) or info.st_nlink != 1
            or getattr(info, "st_file_attributes", 0) & stat.FILE_ATTRIBUTE_REPARSE_POINT
            or not 0 < info.st_size <= MAX_RECORD_BYTES):
        raise ValueError(f"Incomplete or invalid Cargo pool source record; preserve it: {marker}")
    with marker.open("rb") as stream:
        opened = os.fstat(stream.fileno())
        if not os.path.samestat(info, opened) or opened.st_nlink != 1:
            raise ValueError(f"Cargo pool source record changed during admission: {marker}")
        actual = stream.read(MAX_RECORD_BYTES + 1)
    if actual != expected:
        raise ValueError(
            "Cargo compiler pool source or output directories do not match; "
            f"preserve the pool and select a compatible one: {marker}"
        )
    return True


def bind_compiler_pool(repo: Path, target: Path, build: Path) -> None:
    """Admit same-checkout reuse or claim compiler roots that are still empty.

    The record covers both compiler roots; changing just the target name does
    not separate a sibling build directory. Never adopt a populated unbound
    pool, overwrite its record, clean compiler output, or change Cargo home.
    """
    if build.name != "build" or build.parent != target.parent or target == build:
        raise ValueError("Cargo pool binding requires distinct sibling target and build directories")
    record = {
        "schema": "zircon_cargo_pool_source_v1",
        "source": _directory_fact(repo),
        "target": _directory_fact(target),
        "build": _directory_fact(build),
    }
    expected = (json.dumps(record, sort_keys=True, separators=(",", ":")) + "\n").encode("utf-8")
    if len(expected) > MAX_RECORD_BYTES:
        raise ValueError("Cargo pool source record is too large")
    marker = build.parent / RECORD_NAME
    if _check_record(marker, expected):
        return

    for directory in (target, build):
        if next(directory.iterdir(), None) is not None:
            # A cooperating caller can publish the record and start Cargo while
            # this caller is checking emptiness. Recheck before rejecting.
            if _check_record(marker, expected):
                return
            raise ValueError(f"Unbound Cargo compiler output is not empty; preserve it: {directory}")

    try:
        stream = marker.open("xb")
    except FileExistsError:
        if not _check_record(marker, expected):
            raise ValueError(f"Cargo pool source record disappeared during admission: {marker}")
        return
    # Exclusive creation lets only one cooperating checkout claim a fresh pool.
    # Any reader meeting an interrupted or partial write fails closed.
    with stream:
        stream.write(expected)
        stream.flush()
        os.fsync(stream.fileno())

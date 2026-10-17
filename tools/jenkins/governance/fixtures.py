"""Account for and reclaim only this native test recipe's exact generated fixtures."""
from __future__ import annotations

import json
import os
from pathlib import Path
import stat

from .paths import encoded, file_fact, file_identity, linked, physical


def capture(root):
    root = physical(root)
    pending, files, directories = [root], [], []
    while pending:
        directory = pending.pop()
        information = directory.lstat()
        if linked(information):
            raise ValueError("fixture_alias_requires_preservation")
        identity = file_identity(information)
        # Windows may report a directory's allocation length as zero while its
        # handles are open, then 4096 after close. It is not a content identity.
        identity[2] = 0
        directories.append({"path": str(directory), "identity": identity})
        with os.scandir(directory) as iterator:
            for entry in iterator:
                info = entry.stat(follow_symlinks=False)
                if linked(info):
                    raise ValueError("fixture_alias_requires_preservation")
                if stat.S_ISDIR(info.st_mode):
                    pending.append(Path(entry.path))
                elif stat.S_ISREG(info.st_mode):
                    files.append(file_fact(Path(entry.path)))
    return {"root": str(root), "files": sorted(files, key=lambda row: row["path"]),
            "directories": sorted(directories, key=lambda row: row["path"])}


def remove(authority, report):
    from .worker import _json
    from .guarded_delete import remove_empty, remove_verified
    root = physical(report / "fixtures")
    recorded = json.loads((report / "fixture-inventory.json").read_text(encoding="utf-8"))
    if recorded != capture(root):
        raise ValueError("generated_test_fixture_changed_after_native_termination")
    deleted = []
    for fact in recorded["files"]:
        remove_verified(fact, remove=False)
    for fact in recorded["files"]:
        deleted.append(remove_verified(fact, owned_readonly_fixture=True))
    directories = []
    for row in sorted(recorded["directories"], key=lambda row: len(Path(row["path"]).parts), reverse=True):
        directory = physical(row["path"])
        current = directory.lstat()
        if [current.st_dev, current.st_ino] != row["identity"][:2]:
            raise ValueError("owned_fixture_directory_replaced")
        if remove_empty(directory, expected_directory_identity=row["identity"][:2]):
            directories.append(str(directory))
    value = {"status": "reclaimed-owned-test-fixtures", "files": deleted, "directories": directories,
             "logicalDeletedBytes": sum(row["logicalBytes"] for row in deleted),
             "deletedAllocationBytes": sum(row["allocationBytes"] for row in deleted), "actualFreedBytes": None,
             "nativeOwnership": authority.runtime}
    _json(report / "fixture-cleanup.json", value)
    return {key: value[key] for key in ("status", "logicalDeletedBytes", "deletedAllocationBytes", "actualFreedBytes")}

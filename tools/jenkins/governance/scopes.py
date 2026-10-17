"""Hash literal claim scopes without following links or opening authentication files."""
from __future__ import annotations

import os
from pathlib import Path

from .inventory import authentication_path
from .paths import digest, file_fact, physical, relative_source


def inspect(root: Path, selected: list[str], *, maximum_files=100000):
    root = physical(root)
    scopes = sorted({relative_source(value) for value in selected})
    hashes, facts = {}, {}
    for scope in scopes:
        path = physical(root / scope, exists=False)
        if authentication_path(path):
            raise ValueError("authentication_source_requires_review_without_reading_content")
        if not path.exists():
            hashes[scope], facts[scope] = None, None
        elif path.is_file():
            fact = file_fact(path)
            hashes[scope], facts[scope] = fact["sha256"], fact
        elif path.is_dir():
            pending, members, directories = [path], {}, []
            while pending:
                directory = physical(pending.pop())
                identity = directory.stat()
                directories.append(directory.relative_to(root).as_posix())
                with os.scandir(directory) as entries:
                    for entry in entries:
                        candidate = physical(entry.path)
                        name = relative_source(candidate.relative_to(root).as_posix())
                        if candidate.name.casefold() in {".git", ".jenkins"}:
                            raise ValueError("nested_source_authority_requires_separate_registration")
                        if authentication_path(candidate):
                            raise ValueError("authentication_source_requires_review_without_reading_content")
                        if candidate.is_dir():
                            pending.append(candidate)
                        else:
                            fact = file_fact(candidate)
                            members[name], facts[name] = fact["sha256"], fact
                            if len(facts) > maximum_files:
                                raise ValueError("claim_scope_file_budget_exceeded")
                after = directory.stat()
                if (identity.st_dev, identity.st_ino, identity.st_mtime_ns) != (after.st_dev, after.st_ino, after.st_mtime_ns):
                    raise ValueError("claim_directory_changed_during_observation")
            hashes[scope] = digest({"kind": "directory", "files": members, "directories": sorted(directories)})
        else:
            raise ValueError("claim_source_type_requires_review")
    return {"scopes": scopes, "hashes": hashes, "sourceFacts": facts}


def preview(authority, payload, report):
    registration = authority.get("repository", payload["repositoryId"])
    root = physical(payload.get("sourceCheckout", registration["canonicalCheckout"]))
    from .inventory import repository_fact
    observation = repository_fact(root)
    if observation.get("commonDir") != registration["commonDir"]:
        raise ValueError("claim_checkout_repository_mismatch")
    value = {"repositoryId": payload["repositoryId"], "sourceCheckout": str(root),
             "expectedHead": observation.get("head"), "expectedStatusDigest": observation.get("statusDigest"),
             "expectedIndexDigest": observation.get("indexDigest"), **inspect(root, payload["paths"])}
    from .worker import _json
    _json(report / "scope-preview.json", value)
    return {"status": "observed", "scopePreview": file_fact(report / "scope-preview.json"),
            "expectedHashes": value["hashes"], "expectedIndexDigest": value["expectedIndexDigest"],
            "fileCount": len(value["sourceFacts"])}

from __future__ import annotations

from collections.abc import Sequence


MAX_FAILURE_SNAPSHOT_CHANGES = 64


def failure_snapshot_drift(
    expected: Sequence[tuple[str, str]],
    current: Sequence[tuple[str, str]],
    *,
    limit: int = MAX_FAILURE_SNAPSHOT_CHANGES,
) -> dict[str, object]:
    """Project an immutable manifest mismatch into bounded diagnostics."""
    if limit < 0:
        raise ValueError("failure snapshot drift limit cannot be negative")

    expected_by_path = dict(expected)
    current_by_path = dict(current)
    changes: list[dict[str, str | None]] = []
    added_count = 0
    removed_count = 0
    modified_count = 0

    paths = expected_by_path.keys() | current_by_path.keys()
    for path in sorted(paths, key=lambda value: (value.casefold(), value)):
        expected_hash = expected_by_path.get(path)
        current_hash = current_by_path.get(path)
        if expected_hash is None:
            kind = "added"
            added_count += 1
        elif current_hash is None:
            kind = "removed"
            removed_count += 1
        elif expected_hash != current_hash:
            kind = "modified"
            modified_count += 1
        else:
            continue
        if len(changes) < limit:
            changes.append(
                {
                    "path": path,
                    "kind": kind,
                    "expectedHash": expected_hash,
                    "currentHash": current_hash,
                }
            )

    change_count = added_count + removed_count + modified_count
    return {
        "expectedArtifactCount": len(expected_by_path),
        "currentArtifactCount": len(current_by_path),
        "addedCount": added_count,
        "removedCount": removed_count,
        "modifiedCount": modified_count,
        "changeCount": change_count,
        "changes": changes,
        "truncated": change_count > len(changes),
    }

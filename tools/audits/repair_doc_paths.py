"""Repair current front-matter paths; retire declarations with no current owner.

Only unambiguous same-owner suffix moves and file-to-module moves are rewritten.
Historical prose/evidence is never rewritten to claim current implementation.
Original declarations remain available at the frozen engineering-review revision.
"""
from __future__ import annotations

import argparse
from collections import Counter, defaultdict
from pathlib import Path, PurePosixPath
import subprocess

if __package__:
    from .check_conventions import audit_document_paths, FRONT_MATTER_LIST_FIELDS, _test_reference_paths
else:
    from check_conventions import audit_document_paths, FRONT_MATTER_LIST_FIELDS, _test_reference_paths


def replacement(path: str, files: set[str], suffixes: dict) -> str | None:
    if path.endswith(".rs") and path[:-3] + "/mod.rs" in files:
        return path[:-3] + "/mod.rs"
    parts = PurePosixPath(path).parts
    if len(parts) < 3 or ".." in parts:
        return None
    for size in range(len(parts) - 1, 1, -1):
        matches = suffixes.get((parts[0], tuple(parts[-size:])), set())
        if len(matches) == 1:
            return next(iter(matches))
    return None


def repair_text(source: str, invalid: dict, files: set[str], suffixes: dict) -> tuple[str, Counter]:
    lines = source.splitlines(keepends=True)
    if not lines or lines[0].strip() != "---":
        return source, Counter()
    output = [lines[0]]
    field = None
    counts = Counter()
    for index, line in enumerate(lines[1:], 1):
        if line.strip() == "---":
            output.extend(lines[index:])
            break
        if line and not line[0].isspace():
            key, separator, _ = line.partition(":")
            field = key if separator and key in FRONT_MATTER_LIST_FIELDS else None
        value = line.strip()[2:].strip().strip("'\"").strip("`") if line.strip().startswith("- ") else ""
        paths = _test_reference_paths(value) if field == "tests" else [value]
        bad = next((path for path in paths if (field, path) in invalid), None)
        if bad is not None:
            updated = replacement(bad, files, suffixes)
            if updated:
                output.append(line.replace(bad, updated, 1))
                counts["repaired"] += 1
            else:
                counts["retired"] += 1
        else:
            output.append(line)
    return "".join(output), counts


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    files = set(subprocess.check_output(["git", "ls-files", "-z"], cwd=root).decode().split("\0"))
    files = {path for path in files if path and (root / path).exists()}
    files.update(parent.as_posix() for path in list(files) for parent in PurePosixPath(path).parents)
    suffixes = defaultdict(set)
    for path in files:
        parts = PurePosixPath(path).parts
        for size in range(2, len(parts)):
            suffixes[(parts[0], tuple(parts[-size:]))].add(path)
    documents = defaultdict(dict)
    for violation in audit_document_paths(root)["violations"]:
        documents[violation["document"]][(violation["field"], violation["path"])] = violation["reason"]
    totals = Counter()
    for name, invalid in sorted(documents.items()):
        path = root / name
        raw = path.read_bytes()
        source = raw.decode("utf-8-sig")
        updated, counts = repair_text(source, invalid, files, suffixes)
        totals.update(counts)
        if args.apply and source != updated:
            path.write_bytes((b"\xef\xbb\xbf" if raw.startswith(b"\xef\xbb\xbf") else b"") + updated.encode("utf-8"))
    print({"documents": len(documents), **totals, "applied": args.apply})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

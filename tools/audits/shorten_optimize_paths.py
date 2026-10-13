"""Shorten docs/plans/optimize filenames that exceed the managed worktree budget.

A Jenkins managed worktree is created under
``.jenkins/worktrees/<session>-<name>``, which adds ~66 characters to every
repository-relative path.  Windows' legacy MAX_PATH then rejects entries whose
absolute length passes ~259, so a checkout half-applies and is reported as an
opaque interruption.  Every over-budget path in this repository lives under
``docs/plans/optimize/`` and carries a long boilerplate suffix.

This tool removes those suffixes, rewrites the affected references, and can
report the result without writing.  It never touches file contents other than
the recorded reference strings.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

# A managed worktree root is ``<repo>/.jenkins/worktrees/<session>-<name>``.
# With a 40-character reference that root is 79 units, and the measured safe
# absolute limit is 259, so the relative path may not exceed 179.  A caller may
# still pass a shorter reference, but staying inside this budget keeps every
# supported session name usable.
RELATIVE_LIMIT = 179
SUFFIXES = (
    "-product-integration-current-source-review",
    "-product-current-working-tree-review",
    "-product-integration-review",
    "-current-source-review",
    "-current-working-tree-review",
    "-review",
)
REFERENCE_SUFFIXES = (".md", ".json", ".jsonl", ".txt", ".toml", ".py", ".ps1", ".rs")


def tracked(root: Path) -> list[str]:
    completed = subprocess.run(["git", "-C", str(root), "ls-files", "-z"],
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    if completed.returncode:
        raise SystemExit(completed.stderr.decode("utf-8", "replace"))
    return [item.decode("utf-8", "replace") for item in completed.stdout.split(b"\0") if item]


def git_lines(root: Path, *arguments: str) -> list[str]:
    completed = subprocess.run(["git", "-C", str(root), *arguments],
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    if completed.returncode:
        raise SystemExit(completed.stderr.decode("utf-8", "replace"))
    return [item.decode("utf-8", "replace") for item in completed.stdout.splitlines() if item]


def shortened(relative: str) -> str:
    directory, _, name = relative.rpartition("/")
    stem = name[:-3] if name.endswith(".md") else name
    for suffix in SUFFIXES:
        if stem.endswith(suffix):
            stem = stem[: -len(suffix)]
            break
    # Some entries carry no boilerplate suffix at all.  Trim whole trailing
    # words until the path fits, so the result stays readable and stable
    # rather than cutting mid-token.
    while len(f"{directory}/{stem}.md") > RELATIVE_LIMIT and "-" in stem:
        stem = stem.rsplit("-", 1)[0]
    return f"{directory}/{stem}.md" if directory else f"{stem}.md"


def plan(root: Path) -> tuple[dict[str, str], list[str]]:
    paths = tracked(root)
    over = [path for path in paths if len(path) > RELATIVE_LIMIT]
    known = set(paths)
    mapping, blocked = {}, []
    for path in over:
        target = shortened(path)
        if target == path:
            blocked.append(f"{path}: no suffix rule applies")
        elif target in known:
            blocked.append(f"{path}: target already exists ({target})")
        elif target in mapping.values():
            blocked.append(f"{path}: duplicate target ({target})")
        else:
            mapping[path] = target
    return mapping, blocked


def references(root: Path, mapping: dict[str, str]) -> dict[str, int]:
    """Count, per file, how many renamed basenames it mentions.

    The candidate set is resolved by Git rather than by reading every tracked
    file, because a plain walk over this repository's 50k files is far slower
    than the same search in Git's own index.
    """
    names = sorted({path.rsplit("/", 1)[-1] for path in mapping})
    if not names:
        return {}
    handle, name = tempfile.mkstemp(prefix="shorten-paths-", suffix=".txt")
    pattern_file = Path(name)
    try:
        # mkstemp leaves its descriptor open; on Windows an open handle blocks
        # the later unlink, so it is closed before Git reads the file.
        with os.fdopen(handle, "w", encoding="utf-8") as stream:
            stream.write("\n".join(names) + "\n")
        # -F keeps every basename literal; the renames contain no regex syntax
        # that a pattern file would otherwise have to escape.
        found = git_lines(root, "grep", "-l", "-F", "-f", str(pattern_file))
    finally:
        pattern_file.unlink(missing_ok=True)
    hits: dict[str, int] = {}
    for relative in found:
        candidate = root / relative
        try:
            text = candidate.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        count = sum(1 for name in names if name in text)
        if count:
            hits[relative] = count
    return hits


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--apply", action="store_true")
    arguments = parser.parse_args(argv)

    root = Path(arguments.repo_root).resolve()
    mapping, blocked = plan(root)
    hits = references(root, mapping)

    report = {"overBudget": len(mapping) + len(blocked), "renamable": len(mapping),
              "blocked": blocked, "referenceFiles": len(hits),
              "newDeepest": max((len(target) for target in mapping.values()), default=0)}
    print(json.dumps(report, ensure_ascii=False, indent=1))
    if blocked or not arguments.apply:
        return 1 if blocked else 0

    names = {path.rsplit("/", 1)[-1]: target.rsplit("/", 1)[-1] for path, target in mapping.items()}
    for relative in sorted(hits, key=len, reverse=True):
        candidate = root / relative
        text = candidate.read_text(encoding="utf-8")
        for old, new in names.items():
            text = text.replace(old, new)
        candidate.write_text(text, encoding="utf-8")

    for path, target in sorted(mapping.items(), key=lambda item: -len(item[0])):
        completed = subprocess.run(["git", "-C", str(root), "mv", path, target],
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
        if completed.returncode:
            raise SystemExit(completed.stderr.decode("utf-8", "replace"))
    print(json.dumps({"renamed": len(mapping), "references": len(hits)}, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())

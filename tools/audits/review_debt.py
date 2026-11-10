"""Read the frozen review baseline directly from Git; never copy a source tree.

CI needs full history for these debt gates. Only violations present at this
revision are grandfathered; new references and test declarations still fail.
"""
from __future__ import annotations

from pathlib import Path
import subprocess

# Trunk aa73011a1 already contains the inline-test modules and long paths that
# accumulated after 4f1ec912. Later declarations still fail against this tree.
BASELINE = "aa73011a10ecf21069e1ef461a59766ce8f1f37c"


def baseline_sources(root: Path, paths: list[str]) -> tuple[set[str], dict[str, str]]:
    listing = subprocess.run(
        ["git", "ls-tree", "-rz", "--name-only", BASELINE], cwd=root,
        capture_output=True, check=True,
    ).stdout
    files = set(listing.decode("utf-8").split("\0"))
    selected = sorted(set(paths) & files)
    result = subprocess.run(
        ["git", "cat-file", "--batch"], cwd=root,
        input="".join(f"{BASELINE}:{path}\n" for path in selected).encode(),
        capture_output=True, check=True,
    ).stdout
    sources = {}
    offset = 0
    for path in selected:
        end = result.index(b"\n", offset)
        size = int(result[offset:end].split()[-1])
        offset = end + 1
        sources[path] = result[offset:offset + size].decode("utf-8-sig")
        offset += size + 1
    # Directories are valid document references too.
    files.update(parent.as_posix() for path in list(files) for parent in Path(path).parents)
    return files, sources

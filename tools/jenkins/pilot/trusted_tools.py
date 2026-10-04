"""Read-only Git binding independent of retired coordinator tooling."""
from __future__ import annotations

import os
from pathlib import Path
import shutil

from .contracts import PilotError
from .source_manifest import file_hash

_BOUND_GIT: dict[str, str] | None = None


def git_identity() -> dict[str, str]:
    global _BOUND_GIT
    if os.name != "nt":
        raise PilotError("the fixed Git source transport requires native Windows")
    if _BOUND_GIT is not None:
        if file_hash(Path(_BOUND_GIT["path"]), allow_hardlinks=True) != _BOUND_GIT["sha256"]:
            raise PilotError("the bound Git executable changed")
        return dict(_BOUND_GIT)
    # Discover the installed executable once and bind its absolute path/hash.
    # Subsequent PATH changes cannot redirect source operations.
    program_files = Path(os.environ.get("ProgramFiles", r"C:\Program Files"))
    discovered = shutil.which("git")
    candidates = (program_files / "Git/cmd/git.exe", program_files / "Git/bin/git.exe",
                  *((Path(discovered),) if discovered else ()))
    path = next((candidate for candidate in candidates if candidate.is_file()), None)
    if path is None:
        raise PilotError("the source transport requires installed system Git")
    path = Path(os.path.abspath(path))
    # Package managers commonly install executables with hard links. These
    # binaries are pinned by content; source input files retain nlink==1.
    _BOUND_GIT = {"path": str(path), "sha256": file_hash(path, allow_hardlinks=True)}
    return dict(_BOUND_GIT)


def git_command(root: Path, *arguments: str):
    identity = git_identity()
    return [identity["path"], "--no-replace-objects", "--no-optional-locks", "-C", str(root), *arguments], identity

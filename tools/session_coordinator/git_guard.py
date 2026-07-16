from __future__ import annotations

import os
import subprocess
from pathlib import Path

from .models import CoordinatorError


_MARKER = "# zircon-session-coordinator-managed-commit-guard"
_BACKUP_SUFFIX = ".zircon-user"
_HOOK_NAMES = ("pre-commit", "prepare-commit-msg")


def remove_commit_guard(repo_root: str | Path) -> tuple[str, ...]:
    """Remove legacy coordinator commit hooks without touching user hooks."""
    root = Path(repo_root).resolve()
    hooks_dir = _hooks_dir(root)
    if not hooks_dir.is_dir():
        return ()

    removed: list[str] = []
    for name in _HOOK_NAMES:
        hook = hooks_dir / name
        if not hook.is_file():
            continue
        content = hook.read_text(encoding="utf-8", errors="replace")
        if _MARKER not in content:
            continue
        hook.unlink()
        backup = hooks_dir / f"{name}{_BACKUP_SUFFIX}"
        if backup.is_file():
            os.replace(backup, hook)
        removed.append(name)
    return tuple(removed)


def _hooks_dir(repo_root: Path) -> Path:
    result = subprocess.run(
        ["git", "rev-parse", "--git-path", "hooks"],
        cwd=repo_root,
        check=False,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode != 0 or not result.stdout.strip():
        raise CoordinatorError("git_commit_guard_path", "Cannot resolve the local Git hooks directory")
    candidate = Path(result.stdout.strip())
    return candidate if candidate.is_absolute() else (repo_root / candidate).resolve()

from __future__ import annotations

import subprocess
from pathlib import Path


def init_repo(path: Path) -> Path:
    path.mkdir(parents=True, exist_ok=True)
    subprocess.run(["git", "init", "-q"], cwd=path, check=True)
    subprocess.run(["git", "config", "user.email", "coordinator-tests@example.invalid"], cwd=path, check=True)
    subprocess.run(["git", "config", "user.name", "Coordinator Tests"], cwd=path, check=True)
    subprocess.run(["git", "branch", "-M", "main"], cwd=path, check=True)
    (path / "README.md").write_text("baseline\n", encoding="utf-8")
    subprocess.run(["git", "add", "README.md"], cwd=path, check=True)
    subprocess.run(["git", "commit", "-q", "-m", "test: baseline"], cwd=path, check=True)
    return path

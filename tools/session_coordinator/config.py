from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True, slots=True)
class CoordinatorConfig:
    repo_root: Path
    state_root: Path
    host: str = "127.0.0.1"
    port: int = 0
    session_ttl_seconds: int = 600
    lease_ttl_seconds: int = 300
    lease_grace_seconds: int = 120
    watch_interval_seconds: float = 30.0

    @classmethod
    def for_repo(
        cls,
        repo_root: str | Path,
        *,
        state_root: str | Path | None = None,
        host: str = "127.0.0.1",
        port: int = 0,
        watch_interval_seconds: float = 30.0,
    ) -> "CoordinatorConfig":
        resolved_repo = Path(repo_root).resolve()
        resolved_state = (
            Path(state_root).resolve()
            if state_root is not None
            else resolved_repo / ".codex" / "state" / "session-coordinator"
        )
        return cls(
            repo_root=resolved_repo,
            state_root=resolved_state,
            host=host,
            port=port,
            watch_interval_seconds=watch_interval_seconds,
        )

    @property
    def database_path(self) -> Path:
        return self.state_root / "coordinator.sqlite3"

    @property
    def runtime_path(self) -> Path:
        return self.state_root / "runtime.json"

    @property
    def lock_path(self) -> Path:
        return self.state_root / "coordinator.lock"

    @property
    def object_root(self) -> Path:
        return self.state_root / "objects"

    @property
    def patch_artifact_root(self) -> Path:
        return self.state_root / "patch-artifacts"

    @property
    def enabled_target_roots(self) -> tuple[Path, ...]:
        roots: list[Path] = []
        for drive in ("D:\\", "E:\\", "F:\\"):
            drive_path = Path(drive)
            if drive_path.exists():
                roots.append(drive_path / "targets" / "zircon-engine")
        return tuple(roots)

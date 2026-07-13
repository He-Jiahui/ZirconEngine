from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path


DEFAULT_COORDINATOR_PORT = 65189


@dataclass(frozen=True, slots=True)
class CoordinatorConfig:
    repo_root: Path
    state_root: Path
    host: str = "127.0.0.1"
    port: int = DEFAULT_COORDINATOR_PORT
    session_ttl_seconds: int = 600
    lease_ttl_seconds: int = 300
    lease_grace_seconds: int = 120
    watch_interval_seconds: float = 30.0
    maintenance_interval_seconds: float = 900.0
    codex_home: Path | None = None
    codex_spool_base: Path | None = None
    codex_membership_interval_seconds: float = 30.0
    codex_full_interval_seconds: float = 900.0

    @classmethod
    def for_repo(
        cls,
        repo_root: str | Path,
        *,
        state_root: str | Path | None = None,
        host: str = "127.0.0.1",
        port: int = DEFAULT_COORDINATOR_PORT,
        watch_interval_seconds: float = 30.0,
        maintenance_interval_seconds: float = 900.0,
        codex_home: str | Path | None = None,
        codex_spool_base: str | Path | None = None,
        codex_membership_interval_seconds: float = 30.0,
        codex_full_interval_seconds: float = 900.0,
    ) -> "CoordinatorConfig":
        resolved_repo = Path(repo_root).resolve()
        resolved_state = (
            Path(state_root).resolve()
            if state_root is not None
            else resolved_repo / ".codex" / "state" / "session-coordinator"
        )
        isolated = state_root is not None
        resolved_codex_home = (
            Path(codex_home).resolve()
            if codex_home is not None
            else (
                resolved_state / "codex-source"
                if isolated
                else Path(os.environ.get("CODEX_HOME", Path.home() / ".codex")).resolve()
            )
        )
        local_app_data = os.environ.get("LOCALAPPDATA")
        resolved_spool = (
            Path(codex_spool_base).resolve()
            if codex_spool_base is not None
            else (
                resolved_state / "codex-hook"
                if isolated or not local_app_data
                else Path(local_app_data).resolve()
                / "Zircon Session Coordinator"
                / "codex-hook"
            )
        )
        return cls(
            repo_root=resolved_repo,
            state_root=resolved_state,
            host=host,
            port=port,
            watch_interval_seconds=watch_interval_seconds,
            maintenance_interval_seconds=maintenance_interval_seconds,
            codex_home=resolved_codex_home,
            codex_spool_base=resolved_spool,
            codex_membership_interval_seconds=codex_membership_interval_seconds,
            codex_full_interval_seconds=codex_full_interval_seconds,
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
    def workflow_artifact_root(self) -> Path:
        return self.state_root / "workflow-artifacts"

    @property
    def control_web_dist_root(self) -> Path:
        return self.repo_root / "tools" / "session_coordinator" / "web" / "dist"

    @property
    def enabled_target_roots(self) -> tuple[Path, ...]:
        roots: list[Path] = []
        for drive in ("D:\\", "E:\\", "F:\\"):
            drive_path = Path(drive)
            if drive_path.exists():
                roots.extend(
                    drive_path / name
                    for name in ("cargo-targets", "targets", "ZirconBuilds")
                )
        return tuple(roots)

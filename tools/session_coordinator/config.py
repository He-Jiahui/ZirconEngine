from __future__ import annotations

import os
import hashlib
from dataclasses import dataclass
from pathlib import Path


# The shared ZirconEngine coordinator is a single local service. Keep its
# loopback endpoint stable so the browser, tray and Codex Hook have one URL.
# Isolated test coordinators explicitly request port 0.
DEFAULT_COORDINATOR_PORT = 6518


def _normalize_windows_extended_path(value: str | Path) -> str | Path:
    raw = os.fspath(value)
    if raw.startswith("\\\\?\\UNC\\"):
        return "\\\\" + raw[8:]
    if raw.startswith("\\\\?\\"):
        return raw[4:]
    return value


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
    isolated_state: bool = False

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
        resolved_repo = Path(_normalize_windows_extended_path(repo_root)).resolve()
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
            isolated_state=isolated,
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
    def cargo_run_log_root(self) -> Path:
        return self.state_root / "cargo-runs"

    @property
    def offline_command_queue_root(self) -> Path:
        """Durable local handoff for safe CLI work while the daemon is unavailable."""
        return self.state_root / "offline-command-queue"

    @property
    def repository_key(self) -> str:
        """Use the same normalized repository identity as the Windows launcher."""
        identity = str(self.repo_root).replace("/", "\\").rstrip("\\").casefold()
        return hashlib.sha256(identity.encode("utf-8")).hexdigest()

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

    @property
    def unmanaged_artifact_sweep_enabled(self) -> bool:
        """Avoid touching host D/E/F artifacts from isolated test coordinators."""
        return not self.isolated_state

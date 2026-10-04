"""Windows tray adapter for the formal repository-local Jenkins deployment."""
from __future__ import annotations

import json
import os
import time
import subprocess
from dataclasses import dataclass
from pathlib import Path
from urllib.parse import urlparse

from tools.jenkins.deployment.lifecycle import java_major
from tools.jenkins.deployment.manager import DeploymentManager
from tools.jenkins.deployment.paths import DeploymentPaths, resolve_paths
from tools.jenkins.deployment.spec import DeploymentSpec, JenkinsError, load_spec
from .formal_native import identity, terminate_owned


@dataclass(frozen=True)
class FormalTrayConfig:
    spec: DeploymentSpec
    paths: DeploymentPaths
    java: Path
    war: Path

    @property
    def state_file(self) -> Path: return self.paths.state / "tray" / "formal-controller.json"

    @property
    def url(self) -> str:
        c = self.spec.controller
        return f"http://{c.get('listenAddress', '127.0.0.1')}:{int(c['httpPort'])}/"


def load_formal_config(spec_path: str | Path, *, java: str | Path | None = None, war: str | Path | None = None) -> FormalTrayConfig:
    spec = load_spec(spec_path)
    paths = resolve_paths(spec)
    java_path = Path(java) if java else paths.runtime / "jdk" / "bin" / "java.exe"
    war_path = Path(war) if war else paths.runtime / "jenkins.war"
    if not java_path.is_file(): raise JenkinsError("java_missing", "formal Java runtime is missing", details={"path": str(java_path)})
    if not war_path.is_file(): raise JenkinsError("war_missing", "formal Jenkins WAR is missing", details={"path": str(war_path)})
    if java_major(java_path) < 21: raise JenkinsError("java_version", "formal deployment requires Java 21")
    return FormalTrayConfig(spec, paths, java_path.resolve(), war_path.resolve())


def _save(config: FormalTrayConfig, payload: dict) -> None:
    config.paths.state.mkdir(parents=True, exist_ok=True)
    config.state_file.parent.mkdir(parents=True, exist_ok=True)
    temporary = config.state_file.with_suffix(".tmp")
    temporary.write_text(json.dumps(payload, sort_keys=True), encoding="utf-8")
    os.replace(temporary, config.state_file)


def status(config: FormalTrayConfig) -> dict:
    try: probe = DeploymentManager(config.spec, config.paths, config.java, config.war).health()
    except Exception: probe = {"ready": False, "controller": False, "agent": False, "plugins": False}
    operation = None
    journal = config.paths.state / "deployment-operation.json"
    if journal.exists():
        try: operation = json.loads(journal.read_text(encoding="utf8"))
        except (OSError, ValueError): operation = None
    owner = False
    if operation and operation.get("component") == "lifecycle-host" and operation.get("state") in {"running", "starting", "stopping"}:
        try:
            observed = identity(int(operation.get("hostPid")))
            owner = bool(observed and observed.get("birth") == operation.get("creationTime") and observed.get("executable") == operation.get("executable"))
        except Exception: owner = False
    state = "ready" if probe.get("ready") else "stopped" if not probe.get("controller") else "degraded"
    owner_lost = bool(operation and operation.get("component") == "lifecycle-host"
                      and operation.get("state") in {"running", "starting", "stopping", "unknown-owner"}
                      and not owner and not probe.get("controller"))
    if owner_lost:
        state = "degraded"
    return {"state": state, "url": config.url, "formal": True, "controller": probe,
        "operation": operation, "ownerKnown": owner, "ownerLost": owner_lost,
        "message": "Jenkins 异常退出；启动将先核验恢复条件。" if owner_lost else None,
        "canStart": state in {"stopped", "degraded"} and not owner and not probe.get("controller"),
        "canStop": state in {"ready", "degraded"} and owner}


def start(config: FormalTrayConfig) -> dict:
    return DeploymentManager(config.spec, config.paths, config.java, config.war).start()


def stop(config: FormalTrayConfig) -> dict:
    return DeploymentManager(config.spec, config.paths, config.java, config.war).stop()

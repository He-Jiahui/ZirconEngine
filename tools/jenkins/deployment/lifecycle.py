from __future__ import annotations

import os
import socket
import subprocess
import time
from pathlib import Path
from typing import Iterable

from .paths import DeploymentPaths
from .spec import JenkinsError
from tools.jenkins.processes.identity import identity_from_popen, current_identity, ProcessIdentity, process_alive
from tools.jenkins.processes.job import NativeJob, ManagedProcess


def java_major(java: Path) -> int:
    try:
        result = subprocess.run([str(java), "-version"], capture_output=True, text=True, timeout=10, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
    except (OSError, subprocess.TimeoutExpired) as exc: raise JenkinsError("java_unavailable", "configured Java executable could not be queried") from exc
    text = result.stderr + result.stdout
    import re
    match = re.search(r'version\s+"(\d+)', text)
    if not match: raise JenkinsError("java_version_unknown", "configured Java did not report a version")
    return int(match.group(1))


def launch_command(paths: DeploymentPaths, java: Path, war: Path, port: int) -> tuple[list[str], dict[str, str]]:
    if java_major(java) < 21: raise JenkinsError("java_version", "Jenkins requires Java 21 or newer")
    log_root = getattr(paths, "logs", paths.tmp / "logs")
    for directory in (paths.cache / "war", paths.cache / "plugins", paths.tmp / "extracted", log_root / "native"):
        directory.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, JENKINS_HOME=str(paths.home), TEMP=str(paths.tmp), TMP=str(paths.tmp))
    args = [str(java), f"-Djava.io.tmpdir={paths.tmp}", f"-Dhudson.PluginManager.workDir={paths.cache / 'plugins'}", "-jar", str(war), f"--httpPort={port}", "--httpListenAddress=127.0.0.1", f"--webroot={paths.cache / 'war'}", f"--pluginroot={paths.cache / 'plugins'}", f"--extractedFilesFolder={paths.tmp / 'extracted'}"]
    return args, env


def check_port_available(address: str, port: int) -> None:
    """Check that the controller endpoint can be bound before spawning Java.

    This is deliberately a preflight check rather than a reservation.  The
    socket is closed immediately, so a concurrent owner can still win the
    race; the controller's own bind remains authoritative.  It does catch
    Windows excluded ranges and an existing listener early, before creating a
    lifecycle generation whose termination evidence would otherwise be harder
    to collect.
    """
    if not 1 <= int(port) <= 65535:
        raise JenkinsError("controller_port_invalid", f"controller port is outside 1..65535: {port}")
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as probe:
            # Do not set SO_REUSEADDR: the probe must model a normal Jenkins
            # bind and must not hide an occupied or excluded endpoint.
            probe.bind((address, int(port)))
    except (OSError, socket.gaierror) as exc:
        raise JenkinsError(
            "controller_port_unavailable",
            f"controller endpoint {address}:{port} cannot be bound",
            details={"address": address, "port": int(port), "osError": getattr(exc, "winerror", None) or str(exc)},
        ) from exc


def spawn_owned(paths: DeploymentPaths, args: Iterable[str], env: dict[str, str], *, component: str = "process") -> NativeJob:
    paths.logs.mkdir(parents=True, exist_ok=True); paths.state.mkdir(parents=True, exist_ok=True)
    working_directory = paths.agent if component == "agent" else paths.root
    working_directory.mkdir(parents=True, exist_ok=True)
    return NativeJob.launch(list(args), cwd=working_directory, env=env,
                            log_dir=paths.logs / "native" / component)


def wait_for_health(url: str, timeout: float = 300.0, *, headers: dict[str, str] | None = None,
                    process: object | None = None) -> bool:
    import urllib.request
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if process is not None:
            try:
                if process.poll() is not None:
                    return False
            except (AttributeError, OSError):
                # Health remains authoritative when an injected observer does
                # not expose a Popen-like poll method.
                pass
        try:
            request = urllib.request.Request(url.rstrip("/") + "/api/json", headers=headers or {})
            with urllib.request.urlopen(request, timeout=2) as response:
                if response.status == 200:
                    return True
        except Exception:
            time.sleep(.25)
    return False

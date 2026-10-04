"""Current-user Windows login startup for the independent Jenkins tray.

The registry policy itself lives in ``install-jenkins-tray-startup.ps1``.  This
module is the small, structured Python bridge used by the tray UI and tests;
it never starts Jenkins and never touches the retired coordinator entries.
"""
from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path
from typing import Any

from .config import TrayConfig, TrayError

_ACTIONS = frozenset({"Install", "Update", "Query", "Remove"})
_TIMEOUT_SECONDS = 30


def _value_name(config: TrayConfig) -> str:
    import hashlib
    material = (str(config.repo_root).casefold() + "\0" + str(config.pilot_root).casefold()).encode("utf-8")
    return "ZirconJenkinsTray-" + hashlib.sha256(material).hexdigest()[:16]


def _powershell() -> str:
    preferred = Path(r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe")
    return str(preferred) if preferred.is_file() else "powershell.exe"


def _script(config: TrayConfig) -> Path:
    script = config.repo_root / "tools" / "jenkins" / "install-jenkins-tray-startup.ps1"
    if not script.is_file():
        raise TrayError("Jenkins 托盘启动项脚本不存在")
    return script


def _run(config: TrayConfig, action: str, *, dry_run: bool = False) -> dict[str, Any]:
    if action not in _ACTIONS:
        raise TrayError("不支持的托盘启动项操作")
    args = [
        _powershell(), "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
        "-WindowStyle", "Hidden", "-File", str(_script(config)),
        "-Action", action, "-ConfigFile", str(config.config_path),
    ]
    if dry_run:
        args.append("-DryRun")
    flags = getattr(subprocess, "CREATE_NO_WINDOW", 0)
    try:
        result = subprocess.run(
            args, capture_output=True, text=True, encoding="utf-8",
            errors="replace", timeout=_TIMEOUT_SECONDS, check=False,
            creationflags=flags,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise TrayError("托盘启动项操作未能完成") from error
    output = (result.stdout or "").strip().splitlines()
    payload: dict[str, Any] | None = None
    for line in reversed(output):
        try:
            value = json.loads(line)
        except ValueError:
            continue
        if isinstance(value, dict):
            payload = value
            break
    if (result.returncode != 0 or payload is None
            or payload.get("action") != action
            or payload.get("key") != _value_name(config)):
        raise TrayError(f"Tray startup script failed (exit {result.returncode})")
    return payload


def query(config: TrayConfig) -> dict[str, Any]:
    return _run(config, "Query")


def install(config: TrayConfig, *, dry_run: bool = False) -> dict[str, Any]:
    return _run(config, "Install", dry_run=dry_run)


def update(config: TrayConfig, *, dry_run: bool = False) -> dict[str, Any]:
    return _run(config, "Update", dry_run=dry_run)


def remove(config: TrayConfig, *, dry_run: bool = False) -> dict[str, Any]:
    return _run(config, "Remove", dry_run=dry_run)

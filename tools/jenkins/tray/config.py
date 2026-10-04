"""Explicit, physically bound configuration; observation never creates state."""
from __future__ import annotations

import hashlib
import json
import os
import re
import stat
from dataclasses import dataclass
from pathlib import Path
from typing import Mapping

from tools.jenkins.pilot.storage import physical_identity, require_managed_root
from tools.jenkins.pilot.native.paths import WorkerStorage


class TrayError(ValueError):
    """A safe operational error that contains no authentication material."""


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def absolute_plain_path(value: object, *, directory: bool = False) -> Path:
    if not isinstance(value, (str, Path)) or not str(value) or any(c in str(value) for c in '\0\r\n"'):
        raise TrayError("路径必须是明确的绝对路径")
    path = Path(value)
    if not path.is_absolute() or any(p in {".", ".."} for p in str(value).replace("/", "\\").split("\\")):
        raise TrayError("路径不能使用相对路径或路径别名")
    for candidate in (*reversed(path.parents), path):
        try:
            info = candidate.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise TrayError("路径不能跨越链接或重解析点")
        if stat.S_ISREG(info.st_mode) and info.st_nlink != 1:
            raise TrayError("配置和可执行文件不能使用硬链接")
    if path.exists() and path.is_dir() != directory:
        raise TrayError("路径类型不符合配置")
    if os.path.normcase(str(path.resolve())) != os.path.normcase(str(path)):
        raise TrayError("路径不能使用别名")
    return path


@dataclass(frozen=True)
class TrayConfig:
    repo_root: Path
    pilot_root: Path
    python_executable: Path
    pythonw_executable: Path
    vsdevcmd_path: Path
    config_path: Path
    state_dir: Path
    root_identity: Mapping[str, object]
    python_sha256: str
    pythonw_sha256: str

    @property
    def launch_path(self) -> Path:
        return self.repo_root / "tools/jenkins_tray/launch.py"

    @property
    def logs_dir(self) -> Path:
        return require_managed_root(self.pilot_root / "logs/tray", allow_root=False)


def load_config(path: Path | str) -> TrayConfig:
    path = absolute_plain_path(path)
    try:
        payload = json.loads(path.read_bytes())
    except (OSError, ValueError):
        raise TrayError("托盘配置缺失或损坏") from None
    if not isinstance(payload, dict) or payload.get("schemaVersion") != 1:
        raise TrayError("托盘配置版本不受支持")
    try:
        repo = absolute_plain_path(payload["repoRoot"], directory=True)
        root = require_managed_root(payload["pilotRoot"], allow_root=False)
        state = absolute_plain_path(payload["stateDir"], directory=True)
        permitted = repo / ".codex/state/jenkins-tray"
        if not path.is_relative_to(permitted) or not state.is_relative_to(permitted) or state == permitted:
            raise TrayError("托盘配置和状态必须位于本仓库的 Jenkins 托盘状态目录")
        config = TrayConfig(repo, root, absolute_plain_path(payload["pythonExecutable"]),
                            absolute_plain_path(payload["pythonwExecutable"]),
                            absolute_plain_path(payload["vsDevCmdPath"]), path, state,
                            payload["rootIdentity"], payload["pythonSha256"], payload["pythonwSha256"])
    except (KeyError, TypeError):
        raise TrayError("托盘配置字段不完整") from None
    if not isinstance(config.root_identity, dict) or any(not re.fullmatch(r"[0-9a-f]{64}", v or "")
                                                      for v in (config.python_sha256, config.pythonw_sha256)):
        raise TrayError("托盘配置缺少原生目录身份或执行器摘要")
    if config.python_executable.name.casefold() != "python.exe" or config.pythonw_executable.name.casefold() != "pythonw.exe":
        raise TrayError("必须显式绑定 python.exe 与 pythonw.exe")
    if config.python_executable.parent != config.pythonw_executable.parent:
        raise TrayError("后台执行器与托盘必须使用同一 Python 安装")
    if not repo.is_dir() or not (repo / "tools/jenkins_pilot").is_dir():
        raise TrayError("Jenkins 仓库目录不存在")
    return config


def validate_runtime(config: TrayConfig) -> dict:
    if physical_identity(config.pilot_root) != config.root_identity:
        raise TrayError("Jenkins 物理目录身份已变化，已拒绝操作")
    with WorkerStorage(config.pilot_root, create=False) as backend:
        manifest = json.loads(backend.read_bytes("pilot-manifest.json", max_bytes=1024 * 1024))
    if manifest.get("prepared") is not True or manifest.get("repoRoot") != str(config.repo_root):
        raise TrayError("Jenkins 实例与配置仓库不匹配")
    if manifest.get("pythonExecutable") != str(config.python_executable) or manifest.get("pythonSha256") != config.python_sha256:
        raise TrayError("后台 Python 与既有封存驱动不匹配")
    for executable, expected in ((config.python_executable, config.python_sha256),
                                 (config.pythonw_executable, config.pythonw_sha256)):
        absolute_plain_path(executable)
        if file_sha256(executable) != expected:
            raise TrayError("Python 执行器字节已变化")
    return manifest


def configure(*, repo_root: Path, pilot_root: Path, config_path: Path, vsdevcmd_path: Path,
              python_executable: Path | None = None) -> TrayConfig:
    """Write a new profile only after current root and sealed Python checks."""
    repo = absolute_plain_path(repo_root, directory=True)
    root = require_managed_root(pilot_root, allow_root=False)
    target = absolute_plain_path(config_path)
    permitted = repo / ".codex/state/jenkins-tray"
    if not target.is_relative_to(permitted) or target == permitted:
        raise TrayError("配置路径必须位于仓库 Jenkins 托盘状态目录")
    if target.exists():
        raise TrayError("配置已存在；先检查现有配置，不能覆盖其身份")
    with WorkerStorage(root, create=False) as backend:
        manifest = json.loads(backend.read_bytes("pilot-manifest.json", max_bytes=1024 * 1024))
        owner = json.loads(backend.read_bytes("storage-owner.json", max_bytes=65536))
    if manifest.get("repoRoot") != str(repo) or owner.get("rootIdentity") != physical_identity(root):
        raise TrayError("实例没有匹配的仓库和原生目录身份")
    executable = absolute_plain_path(python_executable or manifest["pythonExecutable"])
    gui = absolute_plain_path(executable.with_name("pythonw.exe"))
    vsdev = absolute_plain_path(vsdevcmd_path)
    if not vsdev.is_file():
        raise TrayError("MSVC 环境入口不存在")
    digest, gui_digest = file_sha256(executable), file_sha256(gui)
    if digest != manifest.get("pythonSha256") or str(executable) != manifest.get("pythonExecutable"):
        raise TrayError("Python 必须与已封存驱动相同")
    profile = target.parent / target.stem
    value = {"schemaVersion": 1, "repoRoot": str(repo), "pilotRoot": str(root),
             "pythonExecutable": str(executable), "pythonwExecutable": str(gui),
             "vsDevCmdPath": str(vsdev), "stateDir": str(profile), "rootIdentity": physical_identity(root),
             "pythonSha256": digest, "pythonwSha256": gui_digest}
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open("xb") as stream:
        stream.write(json.dumps(value, ensure_ascii=True, sort_keys=True).encode("utf-8"))
        stream.flush()
        os.fsync(stream.fileno())
    profile.mkdir(exist_ok=True)
    return load_config(target)

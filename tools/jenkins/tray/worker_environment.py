"""Load x64 MSVC only for lifecycle workers, with approved scratch storage."""
from __future__ import annotations

import os
import subprocess
import tempfile
import secrets
from pathlib import Path

from tools.jenkins.pilot.storage import require_managed_root
from .config import TrayConfig, TrayError, absolute_plain_path


def load_msvc_environment(config: TrayConfig) -> dict[str, str]:
    path = absolute_plain_path(config.vsdevcmd_path)
    if not path.is_file() or any(c in str(path) for c in '%!&|<>^'):
        raise TrayError("MSVC 入口路径不可安全调用")
    scratch = require_managed_root(config.pilot_root / "tmp/tray", allow_root=False)
    scratch.mkdir(parents=True, exist_ok=True)
    environment = dict(os.environ, TEMP=str(scratch), TMP=str(scratch),
                       PYTHONDONTWRITEBYTECODE="1", PYTHONUTF8="1", VSCMD_SKIP_SENDTELEMETRY="1")
    system_root = Path(os.environ.get("SystemRoot", r"C:\Windows"))
    cmd = system_root / "System32/cmd.exe"
    script = scratch / ("msvc-" + secrets.token_hex(12) + ".cmd")
    script.write_bytes((f'@echo off\r\nchcp 65001 >nul\r\ncall "{path}" -no_logo -arch=x64 -host_arch=x64 >nul\r\n'
                        'if errorlevel 1 exit /b 1\r\nset\r\n').encode("utf-8"))
    command_line = f'"{cmd}" /d /q /s /c ""{script}""'
    try:
        # A VS helper may inherit standard handles beyond the cmd lifetime.
        # An approved-root file avoids waiting forever for inherited pipe EOF.
        with tempfile.TemporaryFile(dir=scratch, mode="w+b") as output:
            result = subprocess.run(command_line,
                                    cwd=config.repo_root, env=environment, stdin=subprocess.DEVNULL,
                                    stdout=output, stderr=subprocess.DEVNULL, timeout=180,
                                    creationflags=subprocess.CREATE_NO_WINDOW)
            output.seek(0)
            raw = output.read(4 * 1024 * 1024 + 1)
    except (OSError, subprocess.TimeoutExpired):
        raise TrayError("加载 MSVC 环境失败或超时") from None
    finally:
        script.unlink(missing_ok=True)
    if result.returncode:
        raise TrayError("MSVC 环境初始化未成功")
    if len(raw) > 4 * 1024 * 1024:
        raise TrayError("MSVC 环境快照超过限制")
    # The wrapper selects UTF-8; environment content never enters logs.
    for line in raw.decode("utf-8").splitlines():
        key, separator, value = line.partition("=")
        if separator and key and not key.startswith("="):
            environment[key] = value
    if environment.get("VSCMD_ARG_TGT_ARCH", "").casefold() != "x64" or not environment.get("VSCMD_VER"):
        raise TrayError("MSVC 环境没有确认为 x64")
    environment.update(TEMP=str(scratch), TMP=str(scratch), PYTHONDONTWRITEBYTECODE="1", PYTHONUTF8="1",
                       PYTHONPATH=str(config.repo_root))
    return environment

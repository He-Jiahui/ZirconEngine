"""Small durable tray records, separate from immutable Jenkins receipts."""
from __future__ import annotations

import json
import os
import secrets
import time
from pathlib import Path

from .config import TrayError, absolute_plain_path


def read_json(path: Path) -> dict | None:
    absolute_plain_path(path)
    try:
        raw = path.read_bytes()
    except FileNotFoundError:
        return None
    if len(raw) > 1024 * 1024:
        raise TrayError("托盘状态超过读取限制")
    try:
        value = json.loads(raw)
    except ValueError:
        raise TrayError("托盘状态损坏；保留原文件以便核对") from None
    if not isinstance(value, dict):
        raise TrayError("托盘状态不是有效对象")
    return value


def write_json(path: Path, value: dict) -> None:
    absolute_plain_path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name("." + path.name + "." + secrets.token_hex(8) + ".tmp")
    try:
        with temporary.open("xb") as stream:
            stream.write(json.dumps(value, ensure_ascii=True, sort_keys=True, allow_nan=False).encode("utf-8"))
            stream.flush()
            os.fsync(stream.fileno())
        deadline = time.monotonic() + 2.0
        while True:
            try:
                os.replace(temporary, path)
                break
            except PermissionError as error:
                # Windows readers which did not request FILE_SHARE_DELETE can
                # briefly block publication. Retry only the atomic rename;
                # never retry serialization or any lifecycle side effect.
                if os.name != "nt" or getattr(error, "winerror", None) not in {5, 32, 33} or time.monotonic() >= deadline:
                    raise
                time.sleep(0.02)
    finally:
        temporary.unlink(missing_ok=True)


def safe_error(error: BaseException) -> str:
    """Known contract errors are safe; arbitrary exception text may contain secrets."""
    from tools.jenkins.pilot.contracts import PilotError
    from tools.jenkins.pilot.bootstrap import BootstrapError
    if isinstance(error, (TrayError, PilotError, BootstrapError)):
        return str(error)
    return f"操作失败（{type(error).__name__}）；请查看 Jenkins 和托盘日志"

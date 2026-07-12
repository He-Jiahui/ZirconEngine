from __future__ import annotations

import json
import os
import urllib.error
import urllib.request
from collections.abc import Callable
from datetime import UTC, datetime
from pathlib import Path
from typing import BinaryIO, TextIO

from ..migrations import LATEST_SCHEMA_VERSION
from ..processes import process_creation_time, process_is_alive
from ..supervision.repository_identity import repository_identity
from ..supervision.runtime_descriptor import RUNTIME_DESCRIPTOR_VERSION
from .spool import CodexHookEvent, CodexTrigger, CodexTriggerSpool


MAX_HOOK_STDIN_BYTES = 64 * 1024
HOOK_SIGNAL_TIMEOUT_SECONDS = 0.25
_SESSION_SOURCES = frozenset(("startup", "resume", "clear", "compact"))
_PERMISSION_MODES = frozenset(
    ("default", "acceptEdits", "plan", "dontAsk", "bypassPermissions")
)


def run_hook(
    configured_event: str,
    stdin: BinaryIO,
    stdout: TextIO,
    *,
    repo_root: str | Path,
    spool_base: str | Path | None = None,
    signaler: Callable[[Path, str], bool] | None = None,
    clock: Callable[[], str] | None = None,
) -> int:
    """Reduce one Codex Hook input without ever blocking the Codex lifecycle."""

    stop_output = configured_event == "Stop"
    try:
        event = CodexHookEvent.from_codex_name(configured_event)
        if event is None:
            return 0
        raw = stdin.read(MAX_HOOK_STDIN_BYTES + 1)
        if len(raw) > MAX_HOOK_STDIN_BYTES:
            return 0
        payload = json.loads(raw.decode("utf-8"))
        if not isinstance(payload, dict) or payload.get("hook_event_name") != configured_event:
            return 0
        resolved_repo = Path(repo_root).resolve()
        trigger = _reduce(payload, event, resolved_repo, clock or _utc_now)
        if trigger is None:
            return 0
        identity = repository_identity(resolved_repo)
        spool = CodexTriggerSpool(
            Path(spool_base).resolve() if spool_base is not None else _default_spool_base(),
            identity.key,
        )
        spool.enqueue(trigger)
        try:
            (signaler or signal_coordinator)(resolved_repo, identity.key)
        except Exception:
            pass
    except Exception:
        pass
    finally:
        if stop_output:
            stdout.write('{"continue":true}\n')
            stdout.flush()
    return 0


def _reduce(
    payload: dict[str, object],
    event: CodexHookEvent,
    repo_root: Path,
    clock: Callable[[], str],
) -> CodexTrigger | None:
    session_id = _safe_id(payload.get("session_id"))
    cwd_text = payload.get("cwd")
    if session_id is None or not isinstance(cwd_text, str) or any(
        ord(character) < 32 for character in cwd_text
    ):
        return None
    try:
        cwd = Path(cwd_text).resolve(strict=False)
    except (OSError, ValueError):
        return None
    if not _inside(cwd, repo_root):
        return None
    turn_id = _safe_id(payload.get("turn_id"))
    source: str | None = None
    agent_id: str | None = None
    agent_type: str | None = None
    if event is CodexHookEvent.SESSION_START:
        source_value = payload.get("source")
        if not isinstance(source_value, str) or source_value not in _SESSION_SOURCES:
            return None
        source = source_value
    else:
        if turn_id is None:
            return None
    if event in (CodexHookEvent.SUBAGENT_START, CodexHookEvent.SUBAGENT_STOP):
        agent_id = _safe_id(payload.get("agent_id"))
        agent_type = _safe_metadata(payload.get("agent_type"))
        if agent_id is None or agent_type is None:
            return None
    permission = payload.get("permission_mode")
    permission_mode = permission if isinstance(permission, str) and permission in _PERMISSION_MODES else None
    return CodexTrigger(
        event=event,
        session_id=session_id,
        cwd=str(cwd),
        created_at=clock(),
        turn_id=turn_id,
        source=source,
        model=_safe_metadata(payload.get("model")),
        permission_mode=permission_mode,
        agent_id=agent_id,
        agent_type=agent_type,
    )


def signal_coordinator(repo_root: Path, repository_key: str) -> bool:
    runtime_path = repo_root / ".codex" / "state" / "session-coordinator" / "runtime.json"
    lock_path = repo_root / ".codex" / "state" / "session-coordinator" / "coordinator.lock"
    try:
        if runtime_path.stat().st_size > 64 * 1024 or lock_path.stat().st_size > 4096:
            return False
        runtime = json.loads(runtime_path.read_text(encoding="utf-8"))
        lock = json.loads(lock_path.read_text(encoding="utf-8"))
        pid = int(runtime["pid"])
        port = int(runtime["port"])
        if (
            runtime.get("descriptor_version") != RUNTIME_DESCRIPTOR_VERSION
            or runtime.get("host") != "127.0.0.1"
            or runtime.get("repository_key") != repository_key
            or repository_identity(runtime["repo_root"]).key != repository_key
            or int(runtime.get("schema_version", -1)) != LATEST_SCHEMA_VERSION
            or 1 not in runtime.get("control_api_versions", [])
            or int(lock.get("pid", -1)) != pid
            or not (0 < port <= 65535)
            or not process_is_alive(pid)
            or str(runtime.get("process_creation_time")) != process_creation_time(pid)
        ):
            return False
        token = runtime.get("token")
        if not isinstance(token, str) or not token:
            return False
        data = json.dumps(
            {"repositoryKey": repository_key, "schemaVersion": 1},
            separators=(",", ":"),
        ).encode("utf-8")
        request = urllib.request.Request(
            f"http://127.0.0.1:{port}/control/v1/codex-sync/wake",
            data=data,
            headers={
                "Authorization": f"Bearer {token}",
                "Content-Type": "application/json",
            },
            method="POST",
        )
        with urllib.request.urlopen(
            request, timeout=HOOK_SIGNAL_TIMEOUT_SECONDS
        ) as response:
            return response.status == 202
    except (
        OSError,
        ValueError,
        KeyError,
        TypeError,
        json.JSONDecodeError,
        urllib.error.URLError,
    ):
        return False


def _default_spool_base() -> Path:
    local_app_data = os.environ.get("LOCALAPPDATA")
    if not local_app_data:
        raise RuntimeError("LOCALAPPDATA is unavailable")
    return (Path(local_app_data) / "Zircon Session Coordinator" / "codex-hook").resolve()


def _inside(child: Path, parent: Path) -> bool:
    try:
        return os.path.commonpath(
            (os.path.normcase(str(child)), os.path.normcase(str(parent)))
        ) == os.path.normcase(str(parent))
    except ValueError:
        return False


def _safe_id(value: object) -> str | None:
    if not isinstance(value, str) or not value or len(value) > 160:
        return None
    return value if all(character.isalnum() or character in "._:-" for character in value) else None


def _safe_metadata(value: object) -> str | None:
    if not isinstance(value, str) or not value or len(value) > 256:
        return None
    allowed = all(character.isalnum() or character in " ._:-" for character in value)
    return value if allowed and value[0].isalnum() else None


def _utc_now() -> str:
    return datetime.now(tz=UTC).isoformat()

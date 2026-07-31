from __future__ import annotations

import json
import re
import sys
from datetime import UTC, datetime
from pathlib import Path
from typing import Any


_ARTIFACT_COMMAND = re.compile(
    r"(?<![A-Za-z0-9_.-])(?:&\s*)?cargo(?:\.exe)?"
    r"(?:\s+\+[A-Za-z0-9_.-]+)?\s+"
    r"(?P<subcommand>build|check|test|run|bench|clippy|doc|clean)\b",
    re.IGNORECASE,
)
_ARTIFACT_DIRECTORY_CREATION_COMMAND = re.compile(
    r"(?<![A-Za-z0-9_.-])(?P<subcommand>mkdir|md|new-item|ni)\b",
    re.IGNORECASE,
)
_MANAGED_ARTIFACT_ROOT = re.compile(
    r"(?<![A-Za-z0-9_.-])[DEF]:[\\/](?:cargo-targets|targets|zirconbuilds)(?:[\\/]|$)",
    re.IGNORECASE,
)
_CARGO_DENIAL_REASON = (
    "ZirconEngine 构建必须通过协调器：请使用 validate-matrix.ps1、"
    "受控里程碑验证，或 zircon-session cargo 租约命令。"
)
_ARTIFACT_DIRECTORY_DENIAL_REASON = (
    "ZirconEngine 工件目录必须先由协调器登记：请通过受管 Cargo 租约、"
    "validation-copy 或 workflow artifact 创建输出；不要直接在 D/E/F 工件根目录创建目录。"
)


class GuardDecision:
    def __init__(
        self,
        allowed: bool,
        subcommand: str | None = None,
        denial_reason: str | None = None,
    ):
        self.allowed = allowed
        self.subcommand = subcommand
        self.denial_reason = denial_reason


def evaluate_pre_tool_use(payload: object, repo_root: str | Path) -> GuardDecision:
    """Return a local Hook decision without retaining command or transcript text."""
    if not isinstance(payload, dict):
        return GuardDecision(True)
    if payload.get("hook_event_name") != "PreToolUse" or payload.get("tool_name") != "Bash":
        return GuardDecision(True)
    cwd_text = payload.get("cwd")
    tool_input = payload.get("tool_input")
    if not isinstance(cwd_text, str) or not isinstance(tool_input, dict):
        return GuardDecision(True)
    command = tool_input.get("command")
    if not isinstance(command, str):
        return GuardDecision(True)
    root = Path(repo_root).resolve()
    try:
        cwd = Path(cwd_text).resolve(strict=False)
    except (OSError, ValueError):
        return GuardDecision(True)
    if not _inside(cwd, root):
        return GuardDecision(True)
    match = _ARTIFACT_COMMAND.search(command)
    if match is not None:
        subcommand = match.group("subcommand").casefold()
        _append_denial(root, payload, subcommand, "unmanaged_cargo_artifact_command")
        return GuardDecision(False, subcommand, _CARGO_DENIAL_REASON)
    artifact_directory = _ARTIFACT_DIRECTORY_CREATION_COMMAND.search(command)
    if artifact_directory is not None and _MANAGED_ARTIFACT_ROOT.search(command) is not None:
        subcommand = artifact_directory.group("subcommand").casefold()
        _append_denial(root, payload, subcommand, "unmanaged_artifact_directory")
        return GuardDecision(False, subcommand, _ARTIFACT_DIRECTORY_DENIAL_REASON)
    return GuardDecision(True)


def main() -> int:
    try:
        payload = json.load(sys.stdin)
        root = Path(__file__).resolve().parents[2]
        decision = evaluate_pre_tool_use(payload, root)
        if decision.allowed:
            return 0
        json.dump(
            {
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "deny",
                    "permissionDecisionReason": decision.denial_reason,
                }
            },
            sys.stdout,
            ensure_ascii=False,
        )
        sys.stdout.write("\n")
    except Exception:
        # The guard is intentionally fail-open so a malformed Hook payload never blocks Codex.
        return 0
    return 0


def _append_denial(
    repo_root: Path,
    payload: dict[str, Any],
    subcommand: str,
    reason: str,
) -> None:
    try:
        cwd = Path(str(payload["cwd"])).resolve(strict=False)
        relative_cwd = str(cwd.relative_to(repo_root)).replace("\\", "/") or "."
        record = {
            "timestamp": datetime.now(tz=UTC).isoformat(),
            "sessionId": str(payload.get("session_id") or "")[:160],
            "cwd": relative_cwd,
            "subcommand": subcommand,
            "reason": reason,
        }
        destination = repo_root / ".codex/state/session-coordinator/logs/blocked-workflow.jsonl"
        destination.parent.mkdir(parents=True, exist_ok=True)
        with destination.open("a", encoding="utf-8", newline="\n") as stream:
            stream.write(json.dumps(record, ensure_ascii=False, sort_keys=True) + "\n")
    except (OSError, ValueError, TypeError):
        pass


def _inside(child: Path, parent: Path) -> bool:
    try:
        return child.is_relative_to(parent)
    except ValueError:
        return False


if __name__ == "__main__":
    raise SystemExit(main())

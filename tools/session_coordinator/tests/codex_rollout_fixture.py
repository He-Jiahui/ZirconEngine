from __future__ import annotations

import json
from pathlib import Path


def write_rollout(
    root: Path,
    *,
    thread_id: str,
    cwd: Path,
    archived: bool = False,
    lifecycle: tuple[str, ...] = (),
    secret_marker: str = "fixture-secret-must-not-project",
    originator: str = "Codex Desktop",
    cli_version: str = "0.test",
    thread_source: str = "user",
) -> Path:
    owner = (
        root / "archived_sessions"
        if archived
        else root / "sessions" / "2026" / "07" / "13"
    )
    owner.mkdir(parents=True, exist_ok=True)
    path = owner / f"rollout-2026-07-13T00-00-00-{thread_id}.jsonl"
    records: list[dict[str, object]] = [
        {
            "timestamp": "2026-07-13T00:00:00Z",
            "type": "session_meta",
            "payload": {
                "session_id": thread_id,
                "cwd": str(cwd),
                "originator": originator,
                "cli_version": cli_version,
                "thread_source": thread_source,
                "base_instructions": {"text": secret_marker * 64},
                "prompt": secret_marker,
                "webhook": f"https://example.invalid/?key={secret_marker}",
            },
        }
    ]
    for index, event_type in enumerate(lifecycle, start=1):
        records.append(
            {
                "timestamp": f"2026-07-13T00:00:{index:02d}Z",
                "type": "event_msg",
                "payload": {
                    "type": event_type,
                    "turn_id": f"turn-{index}",
                    "message": secret_marker,
                },
            }
        )
    path.write_text(
        "".join(json.dumps(record, separators=(",", ":")) + "\n" for record in records),
        encoding="utf-8",
    )
    return path

from __future__ import annotations

import json
from collections.abc import Mapping, Sequence
from typing import Any


MAX_CONTROL_EVENT_PAYLOAD_BYTES = 16 * 1024
BASELINE_DEGRADED_PATH_SAMPLE_LIMIT = 50

CONTROL_EVENT_PAYLOAD_COLUMNS = f"""
CASE
    WHEN LENGTH(CAST(payload_json AS BLOB)) <= {MAX_CONTROL_EVENT_PAYLOAD_BYTES}
    THEN payload_json
    ELSE NULL
END AS control_payload_json,
LENGTH(CAST(payload_json AS BLOB)) AS control_payload_bytes
"""


def oversized_event_payload(original_bytes: int, *, reason: str) -> dict[str, object]:
    return {
        "truncated": True,
        "originalBytes": original_bytes,
        "reason": reason,
    }


def encode_oversized_event_payload(original_bytes: int, *, reason: str) -> str:
    return json.dumps(
        oversized_event_payload(original_bytes, reason=reason),
        sort_keys=True,
    )


def project_control_event_payload(row: Mapping[str, Any]) -> dict[str, object]:
    original_bytes = int(row["control_payload_bytes"] or 0)
    raw_payload = row["control_payload_json"]
    if raw_payload is None:
        return oversized_event_payload(
            original_bytes,
            reason="event_payload_exceeds_control_limit",
        )
    try:
        payload = json.loads(raw_payload)
    except (TypeError, ValueError):
        return oversized_event_payload(
            original_bytes,
            reason="event_payload_is_invalid_json",
        )
    if isinstance(payload, dict):
        return payload
    return {"value": payload}


def baseline_degraded_payload(epoch_id: int, paths: Sequence[str]) -> dict[str, object]:
    path_count = len(paths)
    sample = list(paths[:BASELINE_DEGRADED_PATH_SAMPLE_LIMIT])
    while True:
        payload: dict[str, object] = {
            "epoch_id": epoch_id,
            "path_count": path_count,
            "path_sample": sample,
            "omitted_path_count": path_count - len(sample),
        }
        encoded = json.dumps(payload, sort_keys=True).encode("utf-8")
        if len(encoded) <= MAX_CONTROL_EVENT_PAYLOAD_BYTES or not sample:
            return payload
        sample.pop()

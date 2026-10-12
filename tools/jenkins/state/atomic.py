"""Durable pointer replacement with bounded Windows sharing-conflict retries."""
from __future__ import annotations

import json
import os
from pathlib import Path
import time
import uuid


def replace_file(source: Path, destination: Path, *, before_replace=None,
                 timeout_seconds: float = 2.0) -> None:
    deadline = time.monotonic() + timeout_seconds
    while True:
        if before_replace is not None:
            before_replace()
        try:
            os.replace(source, destination)
            return
        except OSError as error:
            # Readers and virus scanners can briefly deny DELETE sharing on
            # Windows. Retry the same replacement; never remove the old file,
            # change permissions, or fall back to a non-atomic write.
            if (os.name != "nt" or getattr(error, "winerror", None) not in {5, 32, 33}
                    or time.monotonic() >= deadline):
                raise
            time.sleep(min(.025, max(0.0, deadline - time.monotonic())))


def atomic_json(path: Path, value: dict) -> None:
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    pending = path.with_name(path.name + ".pending-" + uuid.uuid4().hex)
    with pending.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, sort_keys=True, indent=2)
        stream.flush()
        os.fsync(stream.fileno())
    # A failed replacement retains the exact attempted record for recovery.
    replace_file(pending, path)

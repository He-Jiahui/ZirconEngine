from __future__ import annotations

import json
import threading
from contextlib import contextmanager
from dataclasses import dataclass
from typing import Iterator

from ..database import Database
from ..event_payloads import (
    CONTROL_EVENT_PAYLOAD_COLUMNS,
    project_control_event_payload,
)
from ..models import CoordinatorError


@dataclass(frozen=True, slots=True)
class ControlEvent:
    event_id: int
    event_type: str
    payload: dict[str, object]
    created_at: str


@dataclass(frozen=True, slots=True)
class EventReplay:
    events: tuple[ControlEvent, ...]
    resync_required: bool


class EventStreamService:
    def __init__(
        self,
        database: Database,
        *,
        max_clients: int = 8,
        replay_capacity: int = 4096,
    ):
        self.database = database
        self.max_clients = max_clients
        self.replay_capacity = replay_capacity
        self._lock = threading.Lock()
        self._client_count = 0
        self._closing = threading.Event()

    def close(self) -> None:
        self._closing.set()

    def wait_for_close(self, timeout_seconds: float) -> bool:
        return self._closing.wait(timeout_seconds)

    def read_after(self, cursor: int, *, limit: int = 256) -> EventReplay:
        with self.database.connect() as connection:
            bounds = connection.execute(
                "SELECT MIN(event_id), MAX(event_id) FROM events"
            ).fetchone()
            earliest = int(bounds[0]) if bounds[0] is not None else 0
            latest = int(bounds[1]) if bounds[1] is not None else 0
            replay_earliest = max(earliest, latest - self.replay_capacity + 1)
            resync_required = cursor > latest or (
                replay_earliest > 0 and cursor < replay_earliest - 1
            )
            if resync_required:
                return EventReplay((), True)
            rows = connection.execute(
                f"""
                SELECT event_id, event_type, created_at,
                       {CONTROL_EVENT_PAYLOAD_COLUMNS}
                FROM events WHERE event_id > ? ORDER BY event_id LIMIT ?
                """,
                (cursor, limit),
            ).fetchall()
        return EventReplay(
            tuple(
                ControlEvent(
                    event_id=int(row["event_id"]),
                    event_type=row["event_type"],
                    payload=project_control_event_payload(row),
                    created_at=row["created_at"],
                )
                for row in rows
            ),
            False,
        )

    @contextmanager
    def client_slot(self) -> Iterator[None]:
        with self._lock:
            if self._client_count >= self.max_clients:
                raise CoordinatorError("sse_capacity", "SSE client capacity is exhausted")
            self._client_count += 1
        try:
            yield
        finally:
            with self._lock:
                self._client_count -= 1

    @staticmethod
    def encode(event: ControlEvent) -> str:
        data = json.dumps(
            {"type": event.event_type, "payload": event.payload, "createdAt": event.created_at},
            sort_keys=True,
        )
        return f"id: {event.event_id}\nevent: coordinator\ndata: {data}\n\n"

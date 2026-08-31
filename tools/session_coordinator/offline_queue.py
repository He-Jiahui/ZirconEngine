from __future__ import annotations

import json
import os
import re
import time
import uuid
from collections.abc import Callable, Mapping
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from .client import CoordinatorClientError
from .processes import process_creation_time


OFFLINE_COMMAND_SCHEMA_VERSION = 1
MAX_OFFLINE_COMMAND_BYTES = 8 * 1024
MAX_PENDING_OFFLINE_COMMANDS = 128

# These commands converge on the same coordinator state when a local replay is
# retried after process loss.  Operations that allocate a resource, launch a
# process, change lifecycle/commit state, or require an ordered transition are
# deliberately excluded.
SAFE_OFFLINE_COMMANDS = frozenset(
    {
        "session.register",
        "session.heartbeat",
        "lease.heartbeat",
    }
)

_REPOSITORY_KEY = re.compile(r"^[a-f0-9]{64}$")
_COMMAND_NAME = re.compile(r"^[a-z][a-z0-9_.]{0,127}$")
_QUEUE_ID = re.compile(r"^[a-f0-9]{32}$")


@dataclass(frozen=True, slots=True)
class OfflineCommand:
    path: Path
    queue_id: str
    repository_key: str
    command: str
    arguments: dict[str, Any]
    created_at: str


@dataclass(frozen=True, slots=True)
class OfflineQueueSnapshot:
    pending: int
    failed: int
    quarantined: int

    def to_dict(self) -> dict[str, int]:
        return {
            "pending": self.pending,
            "failed": self.failed,
            "quarantined": self.quarantined,
        }


@dataclass(frozen=True, slots=True)
class OfflineReplayResult:
    acknowledged: int = 0
    retained: int = 0
    failed: int = 0
    quarantined: int = 0

    def to_dict(self) -> dict[str, int]:
        return {
            "acknowledged": self.acknowledged,
            "retained": self.retained,
            "failed": self.failed,
            "quarantined": self.quarantined,
        }


class OfflineCommandSpool:
    """Atomic, repository-bound local handoff for idempotent CLI commands.

    The spool intentionally never starts processes or mutates coordinator
    storage itself.  A command leaves ``pending`` only after a live daemon has
    returned a normal response for that exact command envelope.
    """

    def __init__(
        self,
        root: str | Path,
        *,
        repository_key: str,
        max_pending: int = MAX_PENDING_OFFLINE_COMMANDS,
    ) -> None:
        if _REPOSITORY_KEY.fullmatch(repository_key) is None:
            raise ValueError("repository key must be a lowercase SHA-256 digest")
        if not 0 < max_pending <= MAX_PENDING_OFFLINE_COMMANDS:
            raise ValueError("offline command queue capacity is invalid")
        self.root = Path(root).resolve()
        self.pending_root = self.root / "pending"
        self.failed_root = self.root / "failed"
        self.quarantine_root = self.root / "quarantine"
        self.enqueue_lock_path = self.root / "enqueue.lock"
        self.replay_lock_path = self.root / "replay.lock"
        self.repository_key = repository_key
        self.max_pending = max_pending

    def enqueue(self, command: str, arguments: Mapping[str, Any]) -> OfflineCommand:
        if command not in SAFE_OFFLINE_COMMANDS:
            raise ValueError("command is not safe for offline replay")
        if not self._try_acquire_lock(self.enqueue_lock_path):
            raise ValueError("offline command queue is busy")
        try:
            if len(self.validated_pending()) >= self.max_pending:
                raise ValueError("offline command queue is full")
            queue_id = uuid.uuid4().hex
            created_at = datetime.now(timezone.utc).isoformat()
            payload = {
                "schemaVersion": OFFLINE_COMMAND_SCHEMA_VERSION,
                "queueId": queue_id,
                "repositoryKey": self.repository_key,
                "command": command,
                "arguments": dict(arguments),
                "createdAt": created_at,
            }
            encoded = self._encode(payload)
            self.pending_root.mkdir(parents=True, exist_ok=True)
            destination = self.pending_root / f"{time.time_ns():020d}-{queue_id}.json"
            temporary = self.pending_root / f".tmp-{queue_id}"
            try:
                with temporary.open("xb") as handle:
                    handle.write(encoded)
                    handle.flush()
                    os.fsync(handle.fileno())
                os.replace(temporary, destination)
            finally:
                temporary.unlink(missing_ok=True)
            return self._command_from_payload(destination, payload)
        finally:
            self._release_lock(self.enqueue_lock_path)

    def validated_pending(self) -> tuple[OfflineCommand, ...]:
        items, _quarantined = self._validated_pending()
        return items

    def _validated_pending(self) -> tuple[tuple[OfflineCommand, ...], int]:
        items: list[OfflineCommand] = []
        quarantined = 0
        for path in self._ordered(self.pending_root):
            try:
                if path.stat().st_size > MAX_OFFLINE_COMMAND_BYTES:
                    raise ValueError("offline command is oversized")
                payload = json.loads(path.read_text(encoding="utf-8"))
                items.append(self._command_from_payload(path, payload))
            except (OSError, UnicodeError, TypeError, ValueError, json.JSONDecodeError):
                self._move(path, self.quarantine_root)
                quarantined += 1
        return tuple(items), quarantined

    def snapshot(self) -> OfflineQueueSnapshot:
        return OfflineQueueSnapshot(
            pending=len(self.validated_pending()),
            failed=len(self._ordered(self.failed_root)),
            quarantined=len(self._ordered(self.quarantine_root)),
        )

    def replay(
        self, execute: Callable[[str, dict[str, Any]], object]
    ) -> OfflineReplayResult:
        if not self._try_acquire_lock(self.replay_lock_path):
            return OfflineReplayResult(retained=len(self.validated_pending()))
        try:
            return self._replay_locked(execute)
        finally:
            self._release_lock(self.replay_lock_path)

    def _replay_locked(
        self, execute: Callable[[str, dict[str, Any]], object]
    ) -> OfflineReplayResult:
        acknowledged = retained = failed = 0
        pending, quarantined = self._validated_pending()
        for index, item in enumerate(pending):
            try:
                execute(item.command, item.arguments)
            except CoordinatorClientError as error:
                if error.code == "offline":
                    return OfflineReplayResult(
                        acknowledged=acknowledged,
                        retained=len(pending) - index,
                        failed=failed,
                        quarantined=quarantined,
                    )
                self._move(item.path, self.failed_root)
                failed += 1
                return OfflineReplayResult(
                    acknowledged=acknowledged,
                    retained=len(pending) - index - 1,
                    failed=failed,
                    quarantined=quarantined,
                )
            else:
                item.path.unlink(missing_ok=True)
                acknowledged += 1
        return OfflineReplayResult(
            acknowledged=acknowledged,
            retained=retained,
            failed=failed,
            quarantined=quarantined,
        )

    def _command_from_payload(self, path: Path, payload: object) -> OfflineCommand:
        if not isinstance(payload, dict) or set(payload) != {
            "schemaVersion",
            "queueId",
            "repositoryKey",
            "command",
            "arguments",
            "createdAt",
        }:
            raise ValueError("offline command schema is not exact")
        if payload["schemaVersion"] != OFFLINE_COMMAND_SCHEMA_VERSION:
            raise ValueError("offline command schema version is unsupported")
        queue_id = payload["queueId"]
        repository_key = payload["repositoryKey"]
        command = payload["command"]
        arguments = payload["arguments"]
        created_at = payload["createdAt"]
        if not isinstance(queue_id, str) or _QUEUE_ID.fullmatch(queue_id) is None:
            raise ValueError("offline command queue id is invalid")
        if repository_key != self.repository_key:
            raise ValueError("offline command belongs to another repository")
        if not isinstance(command, str) or _COMMAND_NAME.fullmatch(command) is None:
            raise ValueError("offline command name is invalid")
        if command not in SAFE_OFFLINE_COMMANDS:
            raise ValueError("offline command is not safe for replay")
        if not isinstance(arguments, dict):
            raise ValueError("offline command arguments must be an object")
        if not isinstance(created_at, str):
            raise ValueError("offline command timestamp is invalid")
        try:
            timestamp = datetime.fromisoformat(created_at.replace("Z", "+00:00"))
        except ValueError as error:
            raise ValueError("offline command timestamp is invalid") from error
        if timestamp.tzinfo is None:
            raise ValueError("offline command timestamp must include a timezone")
        # Encoding again rejects NaN, unsupported values, and overlarge payloads.
        normalized = json.loads(self._encode(payload).decode("utf-8"))
        return OfflineCommand(
            path=path,
            queue_id=queue_id,
            repository_key=repository_key,
            command=command,
            arguments=normalized["arguments"],
            created_at=created_at,
        )

    @staticmethod
    def _encode(payload: Mapping[str, object]) -> bytes:
        encoded = json.dumps(
            payload,
            allow_nan=False,
            separators=(",", ":"),
            sort_keys=True,
        ).encode("utf-8")
        if len(encoded) > MAX_OFFLINE_COMMAND_BYTES:
            raise ValueError("offline command exceeds the local queue size limit")
        return encoded

    @staticmethod
    def _ordered(root: Path) -> tuple[Path, ...]:
        if not root.exists():
            return ()
        try:
            return tuple(sorted((path for path in root.glob("*.json") if path.is_file()), key=lambda path: path.name))
        except OSError:
            return ()

    def _move(self, path: Path, destination_root: Path) -> None:
        if path.parent != self.pending_root or path.suffix != ".json":
            raise ValueError("offline command path escaped the pending queue")
        destination_root.mkdir(parents=True, exist_ok=True)
        os.replace(path, destination_root / path.name)

    def _try_acquire_lock(self, lock_path: Path) -> bool:
        """Publish a complete lock atomically; callers never wait for queue ownership."""
        self.root.mkdir(parents=True, exist_ok=True)
        for _attempt in range(2):
            temporary: Path | None = None
            try:
                temporary = self.root / f".tmp-lock-{uuid.uuid4().hex}"
                with temporary.open("xb") as handle:
                    handle.write(
                        json.dumps(
                            self._lock_descriptor(), separators=(",", ":")
                        ).encode("utf-8")
                    )
                    handle.flush()
                    os.fsync(handle.fileno())
                os.link(temporary, lock_path)
                return True
            except FileExistsError:
                if not self._lock_owner_is_dead(lock_path):
                    return False
                self._release_lock(lock_path)
            finally:
                if temporary is not None:
                    temporary.unlink(missing_ok=True)
        return False

    @staticmethod
    def _lock_descriptor() -> dict[str, object]:
        return {
            "pid": os.getpid(),
            "processCreationTime": OfflineCommandSpool._process_creation_identity(os.getpid()),
            "createdAt": datetime.now(timezone.utc).isoformat(),
        }

    @staticmethod
    def _process_creation_identity(pid: int) -> str | None:
        if os.name != "nt":
            return None
        try:
            identity = process_creation_time(pid)
        except OSError:
            return None
        return f"{int(identity):016x}"

    def _lock_owner_is_dead(self, lock_path: Path) -> bool:
        try:
            payload = json.loads(lock_path.read_text(encoding="utf-8"))
            pid = int(payload["pid"])
            if pid <= 0:
                return True
        except (OSError, ValueError, KeyError, TypeError, json.JSONDecodeError):
            return True
        if os.name != "nt":
            try:
                os.kill(pid, 0)
            except ProcessLookupError:
                return True
            except OSError:
                return False
            return False
        live_identity = self._process_creation_identity(pid)
        if live_identity is None:
            return True
        return payload.get("processCreationTime") != live_identity

    @staticmethod
    def _release_lock(lock_path: Path) -> None:
        """Windows can keep a just-read hard link open momentarily after close."""
        for attempt in range(20):
            try:
                lock_path.unlink(missing_ok=True)
                return
            except PermissionError:
                if attempt == 19:
                    raise
                time.sleep(0.01)

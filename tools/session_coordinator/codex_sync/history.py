from __future__ import annotations

import hashlib
import json
import os
import re
import uuid
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from ..database import Database


MAX_ROLLOUT_FILES = 10_000
MAX_SESSION_META_BYTES = 1024 * 1024
MAX_INCREMENTAL_BYTES = 8 * 1024 * 1024
MAX_SOURCE_BYTES_PER_PASS = 512 * 1024
MAX_EVENT_LINE_BYTES = 1024 * 1024
MAX_PENDING_CALLS = 1_024
PREFIX_HASH_BYTES = 4 * 1024
MAX_RENDERED_RECORDS = 500
MAX_OUTPUT_TEXT = 64 * 1024
_SAFE_ID = re.compile(r"^[A-Za-z0-9._:-]+$")
_EXIT_CODE = re.compile(r"\bExit code:\s*(-?\d+)\b", re.IGNORECASE)
_TERMINAL_TASKS = {
    "task_complete": "succeeded",
    "task_completed": "succeeded",
    "turn_complete": "succeeded",
    "turn_completed": "succeeded",
    "turn_aborted": "aborted",
    "turn_cancelled": "aborted",
}


@dataclass(frozen=True)
class _EvidenceRecord:
    source_id: str
    thread_id: str
    rollout_name: str
    event_key_hash: str
    kind: str
    outcome: str
    exit_code: int | None
    event_at: str


@dataclass(frozen=True)
class _SourceScan:
    records: tuple[_EvidenceRecord, ...]
    pending_calls: dict[str, tuple[str, str]]
    next_offset: int
    bytes_read: int
    reached_eof: bool


class CodexHistoricalEvidenceCollector:
    """Collect bounded, prompt-free execution outcomes from repository rollouts."""

    def __init__(
        self,
        database: Database,
        *,
        codex_home: str | Path,
        repo_root: str | Path,
        max_files: int = MAX_ROLLOUT_FILES,
    ) -> None:
        self.database = database
        self.codex_home = Path(codex_home).resolve()
        self.repo_root = Path(repo_root).resolve()
        self.max_files = max_files

    def collect_month(
        self,
        generated_at: datetime,
        *,
        byte_budget: int = MAX_INCREMENTAL_BYTES,
    ) -> Path:
        generated_at = generated_at.astimezone(UTC)
        self.advance_month(generated_at, byte_budget=byte_budget)
        month_root = self._month_root(generated_at)
        return self._write_month_history(month_root, generated_at)

    def advance_month(
        self,
        generated_at: datetime,
        *,
        byte_budget: int = MAX_INCREMENTAL_BYTES,
    ) -> None:
        generated_at = generated_at.astimezone(UTC)
        if byte_budget < 0:
            raise ValueError("byte_budget must not be negative")
        month_root = self._month_root(generated_at)
        remaining = byte_budget
        paths = self._prioritized_rollouts(self._rollouts(month_root))
        while remaining > 0:
            advanced = False
            for path in paths:
                if remaining <= 0:
                    break
                consumed = self._collect_source(
                    path,
                    generated_at,
                    byte_budget=min(remaining, MAX_SOURCE_BYTES_PER_PASS),
                )
                if consumed > 0:
                    advanced = True
                    remaining = max(0, remaining - consumed)
            if not advanced:
                break

    def render_month_history(self, generated_at: datetime) -> Path:
        generated_at = generated_at.astimezone(UTC)
        return self._write_month_history(self._month_root(generated_at), generated_at)

    def recent_records(self, since: str, *, limit: int = 50):
        with self.database.connect() as connection:
            return connection.execute(
                """SELECT thread_id, rollout_name, event_key_hash, kind, outcome,
                          exit_code, event_at
                   FROM codex_evidence_records
                   WHERE event_at >= ?
                   ORDER BY event_at DESC, evidence_id DESC
                   LIMIT ?""",
                (since, limit),
            ).fetchall()

    def _rollouts(self, month_root: Path) -> tuple[Path, ...]:
        candidates: list[Path] = []
        if month_root.exists():
            try:
                candidates = [
                    path for path in month_root.rglob("rollout-*.jsonl") if path.is_file()
                ]
            except OSError:
                candidates = []
        candidates.sort(key=lambda path: os.path.normcase(str(path)))
        candidates = candidates[: self.max_files]

        # Codex moves completed rollout files to archived_sessions.  A file with
        # a persisted incomplete cursor must remain discoverable after that move,
        # but the collector must never enumerate arbitrary archive contents.
        candidates.extend(self._incomplete_archived_rollouts(month_root))
        unique: dict[str, Path] = {}
        for path in candidates:
            try:
                resolved = path.resolve(strict=True)
            except OSError:
                continue
            unique.setdefault(os.path.normcase(str(resolved)), resolved)
        return tuple(unique.values())

    def _incomplete_archived_rollouts(self, month_root: Path) -> tuple[Path, ...]:
        month_prefix = f"rollout-{month_root.parent.name}-{month_root.name}-"
        with self.database.connect() as connection:
            rows = connection.execute(
                """SELECT evidence.rollout_name, sessions.rollout_path
                   FROM codex_evidence_sources AS evidence
                   JOIN codex_sessions AS sessions ON sessions.thread_id=evidence.thread_id
                   WHERE evidence.scan_complete=0
                     AND sessions.source_location='archived'
                     AND evidence.rollout_name LIKE ?
                     AND sessions.rollout_path IS NOT NULL
                   ORDER BY evidence.indexed_at ASC, evidence.source_id ASC""",
                (f"{month_prefix}%",),
            ).fetchall()

        candidates: list[Path] = []
        for row in rows:
            rollout_name = str(row["rollout_name"])
            try:
                resolved = Path(str(row["rollout_path"])).resolve(strict=True)
            except (OSError, ValueError):
                continue
            if (
                resolved.name != rollout_name
                or not resolved.match("rollout-*.jsonl")
                or not self._inside(resolved, self.codex_home)
                or not resolved.is_file()
            ):
                continue
            candidates.append(resolved)
        return tuple(candidates)

    def _month_root(self, generated_at: datetime) -> Path:
        return (
            self.codex_home
            / "sessions"
            / generated_at.strftime("%Y")
            / generated_at.strftime("%m")
        )

    def _prioritized_rollouts(self, paths: tuple[Path, ...]) -> tuple[Path, ...]:
        with self.database.connect() as connection:
            states = {
                row["source_id"]: (
                    row["indexed_at"],
                    bool(row["scan_complete"]),
                    int(row["source_mtime_ns"]),
                    int(row["source_size"]),
                )
                for row in connection.execute(
                    """SELECT source_id, indexed_at, scan_complete, source_mtime_ns,
                              source_size
                       FROM codex_evidence_sources"""
                ).fetchall()
            }

        def priority(path: Path) -> tuple[int, str, str]:
            try:
                source_id = self._hash(str(path.resolve(strict=True)))
            except OSError:
                return (3, "", os.path.normcase(str(path)))
            state = states.get(source_id)
            if state is None:
                return (1, "", os.path.normcase(str(path)))
            indexed_at, scan_complete, source_mtime_ns, source_size = state
            try:
                stat = path.stat()
            except OSError:
                return (3, "", os.path.normcase(str(path)))
            if stat.st_mtime_ns != source_mtime_ns or stat.st_size != source_size:
                return (0, str(indexed_at), os.path.normcase(str(path)))
            return (
                3 if scan_complete else 2,
                str(indexed_at),
                os.path.normcase(str(path)),
            )

        return tuple(sorted(paths, key=priority))

    def _collect_source(
        self,
        path: Path,
        generated_at: datetime,
        *,
        byte_budget: int,
    ) -> int:
        try:
            resolved = path.resolve(strict=True)
            if not self._inside(resolved, self.codex_home):
                return 0
            stat = resolved.stat()
            thread_id = self._thread_id(resolved)
            prefix_hash = self._prefix_hash(resolved)
        except (OSError, UnicodeError, json.JSONDecodeError, ValueError, TypeError):
            return 0
        if thread_id is None:
            return 0

        source_id = self._hash(str(resolved))
        source_aliases: tuple[str, ...] = ()
        with self.database.connect() as connection:
            previous = connection.execute(
                """SELECT source_mtime_ns, source_size, scan_offset, prefix_hash,
                          pending_calls_json, scan_complete, scan_revision
                   FROM codex_evidence_sources
                   WHERE source_id=?""",
                (source_id,),
            ).fetchone()
            if self._inside(resolved, self.codex_home / "archived_sessions"):
                migrated = connection.execute(
                    """SELECT source_id, source_mtime_ns, source_size, scan_offset,
                              prefix_hash, pending_calls_json, scan_complete, scan_revision
                       FROM codex_evidence_sources
                       WHERE thread_id=?
                         AND rollout_name=?
                         AND source_size=?
                         AND prefix_hash=?
                       ORDER BY scan_offset DESC, scan_complete DESC, source_id ASC
                    """,
                    (thread_id, resolved.name, stat.st_size, prefix_hash),
                ).fetchall()
                if migrated:
                    # Preserve the pre-archive identity so its evidence cursor
                    # and records continue to describe the same rollout bytes.
                    prearchive_source_id = self._prearchive_source_id(resolved)
                    preferred = next(
                        (
                            row
                            for row in migrated
                            if str(row["source_id"]) == prearchive_source_id
                        ),
                        migrated[0],
                    )
                    source_id = str(preferred["source_id"])
                    previous = preferred
                    source_aliases = tuple(
                        str(row["source_id"])
                        for row in migrated
                        if str(row["source_id"]) != source_id
                    )

        self._consolidate_archived_source_aliases(source_id, source_aliases)

        offset = 0
        revision = 1
        pending_calls: dict[str, tuple[str, str]] = {}
        if previous is not None:
            offset = int(previous["scan_offset"])
            revision = int(previous["scan_revision"])
            pending_calls = self._decode_pending_calls(previous["pending_calls_json"])
            prefix_changed = bool(previous["prefix_hash"]) and (
                previous["prefix_hash"] != prefix_hash
            )
            if stat.st_size < offset or prefix_changed:
                offset = 0
                revision += 1
                pending_calls = {}
            elif (
                int(previous["source_size"]) == stat.st_size
                and bool(previous["scan_complete"])
            ):
                return 0

        scan = self._stream_records(
            resolved,
            source_id=source_id,
            thread_id=thread_id,
            rollout_name=resolved.name,
            source_revision=revision,
            start_offset=offset,
            byte_budget=byte_budget,
            pending_calls=pending_calls,
        )
        try:
            current_stat = resolved.stat()
            current_prefix_hash = self._prefix_hash(resolved)
        except OSError:
            return scan.bytes_read
        if (
            current_stat.st_size != stat.st_size
            or current_stat.st_mtime_ns != stat.st_mtime_ns
            or current_prefix_hash != prefix_hash
        ):
            return scan.bytes_read

        indexed_at = generated_at.isoformat()
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO codex_evidence_sources(
                       source_id, thread_id, rollout_name, source_mtime_ns,
                       source_size, indexed_at, scan_offset, prefix_hash,
                       pending_calls_json, scan_complete, scan_revision
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                   ON CONFLICT(source_id) DO UPDATE SET
                       thread_id=excluded.thread_id,
                       rollout_name=excluded.rollout_name,
                       source_mtime_ns=excluded.source_mtime_ns,
                       source_size=excluded.source_size,
                       indexed_at=excluded.indexed_at,
                       scan_offset=excluded.scan_offset,
                       prefix_hash=excluded.prefix_hash,
                       pending_calls_json=excluded.pending_calls_json,
                       scan_complete=excluded.scan_complete,
                       scan_revision=excluded.scan_revision""",
                (
                    source_id,
                    thread_id,
                    resolved.name,
                    stat.st_mtime_ns,
                    stat.st_size,
                    indexed_at,
                    scan.next_offset,
                    prefix_hash,
                    self._encode_pending_calls(scan.pending_calls),
                    int(scan.reached_eof and scan.next_offset == stat.st_size),
                    revision,
                ),
            )
            connection.executemany(
                """INSERT OR IGNORE INTO codex_evidence_records(
                       source_id, thread_id, rollout_name, event_key_hash, kind,
                       outcome, exit_code, event_at, recorded_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    (
                        record.source_id,
                        record.thread_id,
                        record.rollout_name,
                        record.event_key_hash,
                        record.kind,
                        record.outcome,
                        record.exit_code,
                        record.event_at,
                        indexed_at,
                    )
                    for record in scan.records
                ),
            )
        return scan.bytes_read

    def _consolidate_archived_source_aliases(
        self, source_id: str, source_aliases: tuple[str, ...]
    ) -> None:
        if not source_aliases:
            return
        with self.database.transaction() as connection:
            for alias_source_id in source_aliases:
                connection.execute(
                    """INSERT OR IGNORE INTO codex_evidence_records(
                           source_id, thread_id, rollout_name, event_key_hash, kind,
                           outcome, exit_code, event_at, recorded_at
                       ) SELECT ?, thread_id, rollout_name, event_key_hash, kind,
                                outcome, exit_code, event_at, recorded_at
                         FROM codex_evidence_records
                         WHERE source_id=?""",
                    (source_id, alias_source_id),
                )
                connection.execute(
                    "DELETE FROM codex_evidence_records WHERE source_id=?",
                    (alias_source_id,),
                )
                connection.execute(
                    "DELETE FROM codex_evidence_sources WHERE source_id=?",
                    (alias_source_id,),
                )

    def _prearchive_source_id(self, archived_path: Path) -> str | None:
        """Return the original daily-rollout identity when an archive name encodes it."""
        match = re.match(
            r"^rollout-(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})T",
            archived_path.name,
        )
        if match is None:
            return None
        original = (
            self.codex_home
            / "sessions"
            / match.group("year")
            / match.group("month")
            / match.group("day")
            / archived_path.name
        )
        return self._hash(str(original.resolve()))

    def _thread_id(self, path: Path) -> str | None:
        with path.open("rb") as handle:
            raw = handle.readline(MAX_SESSION_META_BYTES + 1)
        if len(raw) > MAX_SESSION_META_BYTES or not raw.endswith(b"\n"):
            return None
        record = json.loads(raw.decode("utf-8"))
        if not isinstance(record, dict) or record.get("type") != "session_meta":
            return None
        payload = record.get("payload")
        if not isinstance(payload, dict):
            return None
        thread_id = payload.get("session_id") or payload.get("id")
        cwd = payload.get("cwd")
        if not self._safe_id(thread_id) or not isinstance(cwd, str) or not cwd:
            return None
        if not self._inside(Path(cwd).resolve(strict=False), self.repo_root):
            return None
        return thread_id

    def _stream_records(
        self,
        path: Path,
        *,
        source_id: str,
        thread_id: str,
        rollout_name: str,
        source_revision: int,
        start_offset: int,
        byte_budget: int,
        pending_calls: dict[str, tuple[str, str]],
    ) -> _SourceScan:
        records: list[_EvidenceRecord] = []
        calls = dict(pending_calls)
        bytes_read = 0
        offset = start_offset
        with path.open("rb") as handle:
            handle.seek(start_offset)
            while True:
                before = handle.tell()
                if bytes_read >= byte_budget and before > start_offset:
                    return _SourceScan(
                        tuple(records), calls, offset, bytes_read, reached_eof=False
                    )
                raw_line = handle.readline(MAX_EVENT_LINE_BYTES + 1)
                after = handle.tell()
                consumed = after - before
                if not raw_line:
                    return _SourceScan(
                        tuple(records), calls, offset, bytes_read, reached_eof=True
                    )
                bytes_read += consumed
                if len(raw_line) > MAX_EVENT_LINE_BYTES:
                    while not raw_line.endswith(b"\n"):
                        raw_line = handle.readline(MAX_EVENT_LINE_BYTES + 1)
                        bytes_read += len(raw_line)
                        if not raw_line:
                            return _SourceScan(
                                tuple(records), calls, handle.tell(), bytes_read, reached_eof=True
                            )
                    offset = handle.tell()
                    continue
                if not raw_line.endswith(b"\n"):
                    return _SourceScan(
                        tuple(records), calls, before, bytes_read, reached_eof=False
                    )
                offset = after
                self._classify_line(
                    raw_line.rstrip(b"\r\n"),
                    source_id=source_id,
                    thread_id=thread_id,
                    rollout_name=rollout_name,
                    source_revision=source_revision,
                    calls=calls,
                    records=records,
                )

    def _classify_line(
        self,
        raw_line: bytes,
        *,
        source_id: str,
        thread_id: str,
        rollout_name: str,
        source_revision: int,
        calls: dict[str, tuple[str, str]],
        records: list[_EvidenceRecord],
    ) -> None:
        try:
            item = json.loads(raw_line.decode("utf-8"))
        except (UnicodeError, json.JSONDecodeError):
            return
        if not isinstance(item, dict):
            return
        timestamp = self._timestamp(item.get("timestamp"))
        if timestamp is None:
            return
        payload = item.get("payload")
        if not isinstance(payload, dict):
            return
        if item.get("type") == "response_item":
            response_type = payload.get("type")
            call_id = payload.get("call_id")
            if response_type == "custom_tool_call" and self._safe_id(call_id):
                kind = self._classify_call(payload.get("name"), payload.get("input"))
                if kind is not None and (
                    call_id in calls or len(calls) < MAX_PENDING_CALLS
                ):
                    calls[call_id] = (kind, timestamp)
            elif response_type == "custom_tool_call_output" and self._safe_id(call_id):
                pending = calls.pop(call_id, None)
                if pending is not None:
                    kind, call_timestamp = pending
                    exit_code = self._exit_code(payload.get("output"))
                    records.append(
                        self._record(
                            source_id=source_id,
                            thread_id=thread_id,
                            rollout_name=rollout_name,
                            key_material=f"call|{call_id}|{kind}|{call_timestamp}",
                            kind=kind,
                            outcome=self._outcome(exit_code),
                            exit_code=exit_code,
                            event_at=timestamp,
                            source_revision=source_revision,
                        )
                    )
        elif item.get("type") == "event_msg":
            event_type = payload.get("type")
            outcome = _TERMINAL_TASKS.get(event_type)
            if outcome is not None:
                turn_id = payload.get("turn_id")
                safe_turn = turn_id if self._safe_id(turn_id) else "none"
                records.append(
                    self._record(
                        source_id=source_id,
                        thread_id=thread_id,
                        rollout_name=rollout_name,
                        key_material=f"task|{event_type}|{safe_turn}|{timestamp}",
                        kind="task",
                        outcome=outcome,
                        exit_code=None,
                        event_at=timestamp,
                        source_revision=source_revision,
                    )
                )

    def _write_month_history(self, month_root: Path, generated_at: datetime) -> Path:
        month_start = generated_at.replace(
            day=1, hour=0, minute=0, second=0, microsecond=0
        )
        month_end = self._next_month(generated_at)
        target = month_root / f"zircon-engine-evidence-history-{generated_at:%Y-%m}.md"
        with self.database.connect() as connection:
            totals = connection.execute(
                """SELECT kind, outcome, COUNT(*) AS total
                   FROM codex_evidence_records
                   WHERE event_at >= ? AND event_at < ?
                   GROUP BY kind, outcome ORDER BY kind, outcome""",
                (month_start.isoformat(), month_end.isoformat()),
            ).fetchall()
            records = connection.execute(
                """SELECT thread_id, rollout_name, event_key_hash, kind, outcome,
                          exit_code, event_at
                   FROM codex_evidence_records
                   WHERE event_at >= ? AND event_at < ?
                   ORDER BY event_at DESC, evidence_id DESC LIMIT ?""",
                (
                    month_start.isoformat(),
                    month_end.isoformat(),
                    MAX_RENDERED_RECORDS,
                ),
            ).fetchall()
        if not totals and target.exists():
            return target
        lines = [
            "# ZirconEngine Session Evidence（历史索引）",
            "",
            f"- 生成时间：`{generated_at.isoformat()}`",
            "- 边界：仅保存脱敏后的执行类别、结果、时间、会话、rollout 文件名与哈希事件键。",
            "- 不保存提示词、命令参数、日志正文、CWD、绝对路径、令牌或 webhook。",
            "",
            "## 汇总",
            "",
            "| 类型 | 结果 | 数量 |",
            "| --- | --- | --- |",
        ]
        lines.extend(
            f"| {row['kind']} | {row['outcome']} | {row['total']} |" for row in totals
        )
        if not totals:
            lines.append("| — | — | 0 |")
        lines.extend(
            [
                "",
                "## 最近索引记录",
                "",
                "| 时间 | Thread | Rollout | 类型 | 结果 | Exit | 事件键 |",
                "| --- | --- | --- | --- | --- | --- | --- |",
            ]
        )
        lines.extend(
            "| {event_at} | {thread_id} | {rollout_name} | {kind} | {outcome} | {exit_code} | {event_key} |".format(
                event_at=self._cell(row["event_at"]),
                thread_id=self._cell(row["thread_id"]),
                rollout_name=self._cell(row["rollout_name"]),
                kind=self._cell(row["kind"]),
                outcome=self._cell(row["outcome"]),
                exit_code=self._cell(row["exit_code"] if row["exit_code"] is not None else "—"),
                event_key=self._cell(str(row["event_key_hash"])[:12]),
            )
            for row in records
        )
        if not records:
            lines.append("| — | — | — | — | — | — | — |")
        month_root.mkdir(parents=True, exist_ok=True)
        temporary = target.with_name(f".{target.name}.{uuid.uuid4().hex}.tmp")
        temporary.write_text("\n".join(lines) + "\n", encoding="utf-8")
        os.replace(temporary, target)
        return target

    @staticmethod
    def _record(
        *,
        source_id: str,
        thread_id: str,
        rollout_name: str,
        key_material: str,
        kind: str,
        outcome: str,
        exit_code: int | None,
        event_at: str,
        source_revision: int = 1,
    ) -> _EvidenceRecord:
        revision_prefix = "" if source_revision == 1 else f"revision:{source_revision}|"
        return _EvidenceRecord(
            source_id=source_id,
            thread_id=thread_id,
            rollout_name=rollout_name,
            event_key_hash=CodexHistoricalEvidenceCollector._hash(
                f"{source_id}|{revision_prefix}{key_material}"
            ),
            kind=kind,
            outcome=outcome,
            exit_code=exit_code,
            event_at=event_at,
        )

    @staticmethod
    def _classify_call(name: object, input_value: object) -> str | None:
        if not isinstance(name, str) or not isinstance(input_value, str):
            return None
        if name not in {"exec", "shell_command"}:
            return None
        command = input_value.casefold()
        if "git commit" in command or "milestone commit" in command:
            return "commit"
        if "failure return" in command or "failure-return" in command:
            return "failure"
        if "maintenance.cleanup" in command or "cargo release" in command:
            return "cleanup"
        if "cargo " in command or "validate-matrix" in command:
            return "validation"
        return None

    @staticmethod
    def _exit_code(output: object) -> int | None:
        text = CodexHistoricalEvidenceCollector._bounded_text(output)
        match = _EXIT_CODE.search(text)
        return int(match.group(1)) if match is not None else None

    @staticmethod
    def _prefix_hash(path: Path) -> str:
        with path.open("rb") as handle:
            return CodexHistoricalEvidenceCollector._hash(handle.read(PREFIX_HASH_BYTES).hex())

    @staticmethod
    def _decode_pending_calls(value: object) -> dict[str, tuple[str, str]]:
        try:
            parsed = json.loads(str(value or "{}"))
        except json.JSONDecodeError:
            return {}
        if not isinstance(parsed, dict):
            return {}
        calls: dict[str, tuple[str, str]] = {}
        for call_id, metadata in parsed.items():
            if len(calls) >= MAX_PENDING_CALLS or not CodexHistoricalEvidenceCollector._safe_id(call_id):
                continue
            if not isinstance(metadata, dict):
                continue
            kind = metadata.get("kind")
            timestamp = CodexHistoricalEvidenceCollector._timestamp(metadata.get("timestamp"))
            if kind in {"validation", "commit", "failure", "cleanup"} and timestamp is not None:
                calls[call_id] = (kind, timestamp)
        return calls

    @staticmethod
    def _encode_pending_calls(calls: dict[str, tuple[str, str]]) -> str:
        return json.dumps(
            {
                call_id: {"kind": kind, "timestamp": timestamp}
                for call_id, (kind, timestamp) in sorted(calls.items())
            },
            separators=(",", ":"),
            sort_keys=True,
        )

    @staticmethod
    def _bounded_text(value: object, remaining: int = MAX_OUTPUT_TEXT) -> str:
        if remaining <= 0:
            return ""
        if isinstance(value, str):
            return value[:remaining]
        if isinstance(value, list):
            parts: list[str] = []
            for item in value:
                part = CodexHistoricalEvidenceCollector._bounded_text(item, remaining)
                parts.append(part)
                remaining -= len(part)
                if remaining <= 0:
                    break
            return " ".join(parts)
        if isinstance(value, dict):
            parts = []
            for item in value.values():
                part = CodexHistoricalEvidenceCollector._bounded_text(item, remaining)
                parts.append(part)
                remaining -= len(part)
                if remaining <= 0:
                    break
            return " ".join(parts)
        return ""

    @staticmethod
    def _outcome(exit_code: int | None) -> str:
        if exit_code is None:
            return "unknown"
        return "succeeded" if exit_code == 0 else "failed"

    @staticmethod
    def _safe_id(value: object) -> bool:
        return isinstance(value, str) and bool(value) and len(value) <= 160 and bool(_SAFE_ID.fullmatch(value))

    @staticmethod
    def _timestamp(value: object) -> str | None:
        if not isinstance(value, str) or len(value) > 64:
            return None
        try:
            parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
        except ValueError:
            return None
        if parsed.tzinfo is None:
            parsed = parsed.replace(tzinfo=UTC)
        return parsed.astimezone(UTC).isoformat()

    @staticmethod
    def _next_month(value: datetime) -> datetime:
        if value.month == 12:
            return value.replace(year=value.year + 1, month=1, day=1, hour=0, minute=0, second=0, microsecond=0)
        return value.replace(month=value.month + 1, day=1, hour=0, minute=0, second=0, microsecond=0)

    @staticmethod
    def _inside(child: Path, parent: Path) -> bool:
        try:
            return os.path.commonpath((os.path.normcase(str(child)), os.path.normcase(str(parent)))) == os.path.normcase(str(parent))
        except ValueError:
            return False

    @staticmethod
    def _hash(value: str) -> str:
        return hashlib.sha256(value.encode("utf-8")).hexdigest()

    @staticmethod
    def _cell(value: object) -> str:
        return str(value).replace("|", "\\|").replace("\n", " ").replace("\r", " ")

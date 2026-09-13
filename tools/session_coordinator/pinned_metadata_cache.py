from __future__ import annotations

import copy
import hashlib
import json
import os
import pickle
import re
import threading
import time
from collections import OrderedDict
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Mapping, cast
from urllib.parse import unquote, urlsplit, urlunsplit

from .cargo_command_policy import inline_cargo_config_key
from .models import CoordinatorError


PINNED_METADATA_CACHE_POLICY_VERSION = 1
_DEFAULT_MAX_ENTRIES = 16
_DEFAULT_MAX_BYTES = 16 * 1024 * 1024
_MAX_SCANNED_ENTRIES = 262_144
_MAX_TOPOLOGY_CONTENT_BYTES = 16 * 1024 * 1024
_CONTENT_NAMES = frozenset(
    {"Cargo.toml", "Cargo.lock", "rust-toolchain", "rust-toolchain.toml"}
)
_CONFIG_RELATIVE_PATHS = frozenset({".cargo/config", ".cargo/config.toml"})
_PATH_FIELDS = frozenset(
    {"manifest_path", "src_path", "workspace_root", "target_directory"}
)
_PACKAGE_ID_FIELDS = frozenset({"id", "pkg", "root"})
_PACKAGE_ID_LIST_FIELDS = frozenset(
    {"dependencies", "workspace_default_members", "workspace_members"}
)
_PATH_PACKAGE_ID = re.compile(r"path\+(file://[^\s)]+)")


@dataclass(frozen=True, slots=True)
class PinnedMetadataCacheStats:
    hits: int
    misses: int
    coalesced: int
    bypasses: int
    entries: int
    bytes: int
    executions: int
    execution_seconds: float


@dataclass(frozen=True, slots=True)
class PinnedMetadataCacheObservation:
    outcome: str
    execution_seconds: float

    def as_payload(self) -> dict[str, str | bool | float]:
        return {
            "outcome": self.outcome,
            "hit": self.outcome == "hit",
            "miss": self.outcome == "miss",
            "coalesced": self.outcome == "coalesced",
            "executionSeconds": self.execution_seconds,
        }


@dataclass(frozen=True, slots=True)
class _RelativePath:
    relative: str


@dataclass(frozen=True, slots=True)
class _RelativePackageId:
    prefix: str
    relative: str
    query: str
    fragment: str
    suffix: str


@dataclass(slots=True)
class _CacheEntry:
    metadata: object
    byte_count: int


@dataclass(slots=True)
class _Flight:
    event: threading.Event
    metadata: object | None = None
    error: BaseException | None = None


class PinnedMetadataCache:
    """Bounded process-local LRU with one metadata execution per cache key."""

    def __init__(
        self,
        *,
        max_entries: int = _DEFAULT_MAX_ENTRIES,
        max_bytes: int = _DEFAULT_MAX_BYTES,
    ) -> None:
        if max_entries <= 0 or max_bytes <= 0:
            raise ValueError("Pinned metadata cache bounds must be positive")
        self._max_entries = max_entries
        self._max_bytes = max_bytes
        self._entries: OrderedDict[str, _CacheEntry] = OrderedDict()
        self._inflight: dict[str, _Flight] = {}
        self._byte_count = 0
        self._hits = 0
        self._misses = 0
        self._coalesced = 0
        self._bypasses = 0
        self._executions = 0
        self._execution_seconds = 0.0
        self._lock = threading.Lock()

    def get_or_execute(
        self,
        key: str | None,
        view_root: str | Path,
        executor: Callable[[], Mapping[str, object]],
    ) -> Mapping[str, object]:
        """Return rebased metadata or bypass storage without a bounded key."""
        metadata, _observation = self.get_or_execute_observed(
            key, view_root, executor
        )
        return metadata

    def get_or_execute_observed(
        self,
        key: str | None,
        view_root: str | Path,
        executor: Callable[[], Mapping[str, object]],
    ) -> tuple[Mapping[str, object], PinnedMetadataCacheObservation]:
        """Return metadata together with the outcome of this cache request."""
        resolved_view_root = Path(view_root).resolve()
        if key is None:
            started = time.perf_counter()
            try:
                result = executor()
            finally:
                execution_seconds = time.perf_counter() - started
                with self._lock:
                    self._bypasses += 1
                    self._record_execution_locked(execution_seconds)
            return result, PinnedMetadataCacheObservation(
                "bypass", execution_seconds
            )

        leader = False
        cached: object | None = None
        flight: _Flight | None = None
        with self._lock:
            entry = self._entries.get(key)
            if entry is not None:
                self._entries.move_to_end(key)
                self._hits += 1
                cached = entry.metadata
            else:
                flight = self._inflight.get(key)
                if flight is None:
                    flight = _Flight(threading.Event())
                    self._inflight[key] = flight
                    self._misses += 1
                    leader = True
                else:
                    self._coalesced += 1

        if cached is not None:
            return (
                cast(
                    Mapping[str, object],
                    _restore_metadata(cached, resolved_view_root),
                ),
                PinnedMetadataCacheObservation("hit", 0.0),
            )

        if not leader:
            assert flight is not None
            flight.event.wait()
            if flight.error is not None:
                raise flight.error
            return (
                cast(
                    Mapping[str, object],
                    _restore_metadata(flight.metadata, resolved_view_root),
                ),
                PinnedMetadataCacheObservation("coalesced", 0.0),
            )

        assert flight is not None

        started = time.perf_counter()
        try:
            result = executor()
            execution_seconds = time.perf_counter() - started
            canonical = (
                _canonicalize_metadata(result, resolved_view_root)
                if isinstance(result, Mapping)
                else copy.deepcopy(result)
            )
            try:
                byte_count = len(pickle.dumps(canonical, protocol=5)) + len(
                    key.encode("utf-8")
                )
            except (pickle.PickleError, TypeError, AttributeError):
                byte_count = self._max_bytes + 1
        except BaseException as error:
            execution_seconds = time.perf_counter() - started
            with self._lock:
                self._record_execution_locked(execution_seconds)
                flight.error = error
                self._inflight.pop(key, None)
                flight.event.set()
            raise

        with self._lock:
            self._record_execution_locked(execution_seconds)
            flight.metadata = canonical
            if isinstance(result, Mapping) and byte_count <= self._max_bytes:
                previous = self._entries.pop(key, None)
                if previous is not None:
                    self._byte_count -= previous.byte_count
                self._entries[key] = _CacheEntry(canonical, byte_count)
                self._byte_count += byte_count
                self._evict_locked()
            self._inflight.pop(key, None)
            flight.event.set()
        return (
            cast(
                Mapping[str, object],
                _restore_metadata(canonical, resolved_view_root),
            ),
            PinnedMetadataCacheObservation("miss", execution_seconds),
        )

    def stats(self) -> PinnedMetadataCacheStats:
        with self._lock:
            return PinnedMetadataCacheStats(
                hits=self._hits,
                misses=self._misses,
                coalesced=self._coalesced,
                bypasses=self._bypasses,
                entries=len(self._entries),
                bytes=self._byte_count,
                executions=self._executions,
                execution_seconds=self._execution_seconds,
            )

    def _record_execution_locked(self, execution_seconds: float) -> None:
        self._executions += 1
        self._execution_seconds += execution_seconds

    def _evict_locked(self) -> None:
        while (
            len(self._entries) > self._max_entries
            or self._byte_count > self._max_bytes
        ):
            _key, evicted = self._entries.popitem(last=False)
            self._byte_count -= evicted.byte_count


def build_pinned_metadata_cache_key(
    view_root: str | Path,
    source_root: str | Path,
    command: tuple[str, ...],
    *,
    external_identities: tuple[str, ...] = (),
    executor_namespace: str = "production",
    tool_identity: str = "",
) -> str | None:
    """Hash bounded Cargo topology and stable metadata execution semantics."""
    resolved_view_root = Path(view_root).resolve()
    resolved_source_root = Path(source_root).resolve()
    try:
        source_relative = resolved_source_root.relative_to(
            resolved_view_root
        ).as_posix()
    except ValueError:
        return None

    explicit_configs = _explicit_config_paths(
        command, resolved_view_root, resolved_source_root
    )
    topology: list[tuple[str, str, str]] = []
    content_bytes = 0
    visited_count = 0
    try:
        repository_roots = _bounded_repository_roots(resolved_view_root)
        if repository_roots is None:
            return None
        visited_count = len(repository_roots)
        for repository_root in repository_roots:
            repository_name = repository_root.relative_to(
                resolved_view_root
            ).as_posix()
            scan = _bounded_repository_files(
                repository_root, _MAX_SCANNED_ENTRIES - visited_count
            )
            if scan is None:
                return None
            repository_files, repository_visited = scan
            visited_count += repository_visited
            for path in repository_files:
                relative = path.relative_to(resolved_view_root).as_posix()
                repository_relative = path.relative_to(repository_root).as_posix()
                content_relevant = (
                    path.name in _CONTENT_NAMES
                    or repository_relative in _CONFIG_RELATIVE_PATHS
                    or (
                        path.parent.name.casefold() == ".cargo"
                        and path.name.casefold() in {"config", "config.toml"}
                    )
                    or path.resolve() in explicit_configs
                )
                if content_relevant:
                    content = path.read_bytes()
                    content_bytes += len(content)
                    if content_bytes > _MAX_TOPOLOGY_CONTENT_BYTES:
                        return None
                    topology.append(
                        (relative, "content", hashlib.sha256(content).hexdigest())
                    )
                else:
                    # Cargo target discovery and manifest references depend on
                    # the repository path shape, not production file contents.
                    topology.append((relative, "path-shape", ""))
            topology.append((repository_name, "repository", ""))
    except OSError:
        return None

    payload = {
        "policy": PINNED_METADATA_CACHE_POLICY_VERSION,
        "sourceRoot": source_relative,
        "command": _stable_command(command, resolved_view_root),
        "topology": sorted(topology, key=lambda item: item[0].casefold()),
        "external": tuple(sorted(external_identities)),
        "executor": executor_namespace,
        "tool": tool_identity,
    }
    encoded = json.dumps(
        payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def _bounded_repository_roots(view_root: Path) -> tuple[Path, ...] | None:
    roots: list[Path] = []
    try:
        with os.scandir(view_root) as entries:
            for entry in entries:
                metadata = entry.stat(follow_symlinks=False)
                if entry.is_symlink() or (
                    getattr(metadata, "st_file_attributes", 0) & 0x400
                ):
                    return None
                if (
                    entry.is_dir(follow_symlinks=False)
                    and entry.name not in {"metadata-target", "metadata-cargo-home"}
                ):
                    roots.append(Path(entry.path))
    except OSError:
        return None
    if len(roots) > _MAX_SCANNED_ENTRIES:
        return None
    return tuple(sorted(roots, key=lambda path: path.name.casefold()))


def _bounded_repository_files(
    repository_root: Path, remaining_entries: int
) -> tuple[tuple[Path, ...], int] | None:
    if remaining_entries < 0:
        return None
    pending = [repository_root]
    files: list[Path] = []
    visited = 0
    try:
        while pending:
            directory = pending.pop()
            with os.scandir(directory) as entries:
                for entry in entries:
                    visited += 1
                    if visited > remaining_entries:
                        return None
                    metadata = entry.stat(follow_symlinks=False)
                    if entry.is_symlink() or (
                        getattr(metadata, "st_file_attributes", 0) & 0x400
                    ):
                        return None
                    path = Path(entry.path)
                    if entry.is_dir(follow_symlinks=False):
                        pending.append(path)
                    elif entry.is_file(follow_symlinks=False):
                        files.append(path)
    except OSError:
        return None
    return tuple(sorted(files, key=lambda path: str(path).casefold())), visited


def _explicit_config_paths(
    command: tuple[str, ...], view_root: Path, source_root: Path
) -> frozenset[Path]:
    paths: set[Path] = set()
    index = 0
    while index < len(command):
        part = command[index]
        value: str | None = None
        if part == "--config" and index + 1 < len(command):
            value = command[index + 1]
            index += 1
        elif part.startswith("--config="):
            value = part.partition("=")[2]
        if value and not _is_inline_cargo_config(value):
            candidate = Path(value)
            if not candidate.is_absolute():
                candidate = source_root / candidate
            try:
                resolved = candidate.resolve()
                if resolved.is_relative_to(view_root):
                    paths.add(resolved)
            except OSError:
                pass
        index += 1
    return frozenset(paths)


def _is_inline_cargo_config(value: str) -> bool:
    if "=" not in value:
        return False
    try:
        return inline_cargo_config_key(value) is not None
    except CoordinatorError:
        # A relative filename may itself contain '='. Ticket admission rejects
        # malformed inline TOML before a planner view is created.
        return False


def _stable_command(command: tuple[str, ...], view_root: Path) -> tuple[str, ...]:
    stable: list[str] = []
    for part in command:
        whole = Path(part)
        if whole.is_absolute():
            normalized = _view_relative_command_path(whole, view_root)
            stable.append(normalized if normalized is not None else part)
            continue
        option, separator, value = part.partition("=")
        candidate_value = value if separator else part
        candidate = Path(candidate_value)
        if candidate.is_absolute():
            normalized = _view_relative_command_path(candidate, view_root)
            if normalized is None:
                stable.append(part)
            else:
                stable.append(f"{option}={normalized}" if separator else normalized)
        else:
            stable.append(part)
    return tuple(stable)


def _view_relative_command_path(path: Path, view_root: Path) -> str | None:
    try:
        relative = path.resolve().relative_to(view_root).as_posix()
    except (OSError, ValueError):
        return None
    return f"$VIEW_ROOT/{relative}"


def _canonicalize_metadata(value: object, view_root: Path, field: str = "") -> object:
    if isinstance(value, Mapping):
        return {
            key: _canonicalize_metadata(item, view_root, str(key))
            for key, item in value.items()
        }
    if isinstance(value, list):
        if field in _PACKAGE_ID_LIST_FIELDS:
            return [
                _canonicalize_package_id(item, view_root)
                if isinstance(item, str)
                else _canonicalize_metadata(item, view_root)
                for item in value
            ]
        return [_canonicalize_metadata(item, view_root) for item in value]
    if isinstance(value, tuple):
        return tuple(_canonicalize_metadata(item, view_root) for item in value)
    if isinstance(value, str):
        if field in _PATH_FIELDS:
            relative = _relative_path(value, view_root)
            return _RelativePath(relative) if relative is not None else value
        if field in _PACKAGE_ID_FIELDS:
            return _canonicalize_package_id(value, view_root)
        if Path(value).is_absolute():
            relative = _relative_path(value, view_root)
            return _RelativePath(relative) if relative is not None else value
    return copy.deepcopy(value)


def _canonicalize_package_id(value: str, view_root: Path) -> object:
    match = _PATH_PACKAGE_ID.search(value)
    if match is None:
        return value
    parsed = urlsplit(match.group(1))
    filesystem_path = _file_uri_path(parsed.netloc, parsed.path)
    try:
        relative = filesystem_path.resolve().relative_to(view_root).as_posix()
    except (OSError, ValueError):
        return value
    return _RelativePackageId(
        prefix=value[: match.start(1)],
        relative=relative,
        query=parsed.query,
        fragment=parsed.fragment,
        suffix=value[match.end(1) :],
    )


def _file_uri_path(netloc: str, raw_path: str) -> Path:
    decoded = unquote(raw_path)
    if re.match(r"^/[A-Za-z]:/", decoded):
        decoded = decoded[1:]
    if netloc and netloc.casefold() != "localhost":
        decoded = f"//{netloc}{decoded}"
    return Path(decoded)


def _relative_path(value: str, view_root: Path) -> str | None:
    try:
        return Path(value).resolve().relative_to(view_root).as_posix()
    except (OSError, ValueError):
        return None


def _restore_metadata(value: object, view_root: Path) -> object:
    if isinstance(value, _RelativePath):
        return str(view_root.joinpath(*Path(value.relative).parts))
    if isinstance(value, _RelativePackageId):
        uri = view_root.joinpath(*Path(value.relative).parts).as_uri()
        parsed = urlsplit(uri)
        uri = urlunsplit(
            (parsed.scheme, parsed.netloc, parsed.path, value.query, value.fragment)
        )
        return f"{value.prefix}{uri}{value.suffix}"
    if isinstance(value, Mapping):
        return {key: _restore_metadata(item, view_root) for key, item in value.items()}
    if isinstance(value, list):
        return [_restore_metadata(item, view_root) for item in value]
    if isinstance(value, tuple):
        return tuple(_restore_metadata(item, view_root) for item in value)
    return copy.deepcopy(value)

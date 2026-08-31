from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
from collections import deque
from concurrent.futures import Future, ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Iterator, Sequence, TypeVar

if __package__ in {None, ""}:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools.runtime_domain_dependency_audit import _rust_code_view, _rust_use_paths


SCHEMA_VERSION = 2
READ_WORKERS = 8
READ_IN_FLIGHT = 16
DEFAULT_OWNER_ROOTS = ("zircon_runtime/src/core/resource",)
EXCLUDED_TOP_LEVEL_ROOTS = frozenset({".codex", "dev", "target"})
RESOURCE_PATH_PREFIXES = (
    ("crate", "core", "resource"),
    ("zircon_runtime", "core", "resource"),
)
RESOURCE_LITERAL_PATH = re.compile(
    r"(?<![A-Za-z0-9_])(?:crate|zircon_runtime)\s*::\s*core\s*::\s*resource\b"
)


class ResourceConsumerManifestError(RuntimeError):
    pass


class ManifestStabilityError(ResourceConsumerManifestError):
    def __init__(self, reason: str, changed_paths: Sequence[str] = ()) -> None:
        self.reason = reason
        self.changed_paths = list(changed_paths)
        suffix = (
            f": {', '.join(self.changed_paths)}" if self.changed_paths else ""
        )
        super().__init__(f"resource consumer manifest is unstable ({reason}){suffix}")


@dataclass(frozen=True, slots=True)
class ResourceConsumerSnapshot:
    report: dict[str, object]
    candidates: tuple[Path, ...]
    candidate_fingerprints: tuple[dict[str, object], ...]
    owner_roots: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class ResourceConsumerCapture:
    candidates: tuple[Path, ...]
    candidate_fingerprints: tuple[dict[str, object], ...]
    consumers: tuple[dict[str, object], ...]
    owner_roots: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class ResourceReferenceInventory:
    rust_candidates: tuple[Path, ...]
    textual_candidates: tuple[Path, ...]


InputT = TypeVar("InputT")
OutputT = TypeVar("OutputT")


def _canonical_json_bytes(value: object) -> bytes:
    return json.dumps(
        value,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")


def _sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def _manifest_sha256(value: object) -> str:
    return _sha256_bytes(_canonical_json_bytes(value))


def _run_git(
    repo_root: Path,
    *arguments: str,
    allowed_returncodes: Sequence[int] = (0,),
) -> bytes:
    completed = subprocess.run(
        ["git", "-C", str(repo_root), *arguments],
        check=False,
        capture_output=True,
    )
    if completed.returncode not in allowed_returncodes:
        message = completed.stderr.decode("utf-8", errors="replace").strip()
        raise ResourceConsumerManifestError(
            f"git {' '.join(arguments)} failed with {completed.returncode}: {message}"
        )
    return completed.stdout


def _safe_relative_path(raw_path: str) -> Path:
    normalized = raw_path.replace("\\", "/").rstrip("/")
    path = Path(normalized)
    if not normalized or path.is_absolute() or ".." in path.parts:
        raise ResourceConsumerManifestError(
            f"git returned an unsafe repository-relative path: {raw_path!r}"
        )
    return path


def _path_key(path: Path) -> tuple[str, str]:
    display_path = path.as_posix()
    return display_path.casefold(), display_path


def _current_git_paths(repo_root: Path, output: bytes) -> tuple[Path, ...]:
    paths: set[Path] = set()
    for raw in output.split(b"\0"):
        if not raw:
            continue
        path = _safe_relative_path(raw.decode("utf-8"))
        if (repo_root / path).is_file():
            paths.add(path)
    return tuple(sorted(paths, key=_path_key))


def _git_resource_reference_inventory(
    repo_root: Path,
    *,
    textual_roots: Sequence[str] = (),
    textual_suffixes: frozenset[str] = frozenset(),
    textual_tokens: Sequence[bytes] = (),
) -> ResourceReferenceInventory:
    normalized_textual_roots = _normalized_owner_roots(textual_roots)
    normalized_textual_suffixes = frozenset(
        suffix.casefold() for suffix in textual_suffixes
    )
    pathspecs = ["*.rs"]
    pathspecs.extend(
        f"{root}/*{suffix}"
        for root in normalized_textual_roots
        for suffix in sorted(normalized_textual_suffixes)
        if suffix != ".rs"
    )

    with ThreadPoolExecutor(max_workers=2) as executor:
        tracked_future = executor.submit(
            _run_git,
            repo_root,
            "grep",
            "-z",
            "-l",
            "-I",
            "-e",
            "resource",
            "--",
            *pathspecs,
            allowed_returncodes=(0, 1),
        )
        untracked_future = executor.submit(
            _run_git,
            repo_root,
            "ls-files",
            "--others",
            "--exclude-standard",
            "-z",
            "--",
            *pathspecs,
        )
        tracked = _current_git_paths(repo_root, tracked_future.result())
        untracked = _current_git_paths(repo_root, untracked_future.result())

    def inside_textual_root(path: Path) -> bool:
        display_path = path.as_posix().casefold()
        return any(
            display_path == root.casefold()
            or display_path.startswith(root.casefold() + "/")
            for root in normalized_textual_roots
        )

    def textual_path(path: Path) -> bool:
        return (
            path.suffix.casefold() in normalized_textual_suffixes
            and inside_textual_root(path)
        )

    untracked_set = set(untracked)
    read_candidates = tuple(
        sorted(
            untracked_set.union(path for path in tracked if textual_path(path)),
            key=_path_key,
        )
    )

    def read_source(path: Path) -> tuple[Path, bytes] | None:
        try:
            source_bytes = (repo_root / path).read_bytes()
        except FileNotFoundError:
            return None
        except OSError as error:
            raise ResourceConsumerManifestError(
                f"failed to read candidate {path.as_posix()}: {error}"
            ) from error
        return path, source_bytes

    rust_candidates = {path for path in tracked if path.suffix.casefold() == ".rs"}
    textual_candidates: set[Path] = set()
    for result in _bounded_ordered_map(read_source, read_candidates):
        if result is None:
            continue
        path, source_bytes = result
        if (
            path in untracked_set
            and path.suffix.casefold() == ".rs"
            and _may_reference_resource(source_bytes)
        ):
            rust_candidates.add(path)
        if textual_path(path) and any(
            token in source_bytes for token in textual_tokens
        ):
            textual_candidates.add(path)

    return ResourceReferenceInventory(
        rust_candidates=tuple(sorted(rust_candidates, key=_path_key)),
        textual_candidates=tuple(sorted(textual_candidates, key=_path_key)),
    )


def _git_resource_token_candidates(repo_root: Path) -> tuple[Path, ...]:
    return _git_resource_reference_inventory(repo_root).rust_candidates


def _normalized_owner_roots(owner_roots: Sequence[str]) -> tuple[str, ...]:
    normalized: set[str] = set()
    for raw_root in owner_roots:
        root = _safe_relative_path(raw_root).as_posix().rstrip("/")
        normalized.add(root)
    return tuple(sorted(normalized, key=lambda value: (value.casefold(), value)))


def _excluded_repository_path(path: Path) -> bool:
    parts = path.parts
    if not parts:
        return True
    if parts[0].casefold() in EXCLUDED_TOP_LEVEL_ROOTS:
        return True
    return any(part.casefold() == "target" for part in parts)


def _inside_owner_root(path: Path, owner_roots: Sequence[str]) -> bool:
    display_path = path.as_posix().casefold()
    return any(
        display_path == root.casefold()
        or display_path.startswith(root.casefold() + "/")
        for root in owner_roots
    )


def _may_reference_resource(source_bytes: bytes) -> bool:
    return (
        b"resource" in source_bytes
        and b"core" in source_bytes
        and (b"crate" in source_bytes or b"zircon_runtime" in source_bytes)
    )


def _bounded_ordered_map(
    operation: Callable[[InputT], OutputT], values: Sequence[InputT]
) -> Iterator[OutputT]:
    if not values:
        return
    iterator = iter(values)
    pending: deque[Future[OutputT]] = deque()
    with ThreadPoolExecutor(
        max_workers=READ_WORKERS,
        thread_name_prefix="zr-resource-manifest-read",
    ) as executor:
        for _ in range(min(READ_IN_FLIGHT, len(values))):
            pending.append(executor.submit(operation, next(iterator)))
        while pending:
            result = pending.popleft().result()
            try:
                value = next(iterator)
            except StopIteration:
                pass
            else:
                pending.append(executor.submit(operation, value))
            yield result


def _read_candidate_source(
    repo_root: Path, relative_path: Path
) -> tuple[Path, bytes, str]:
    absolute_path = repo_root / relative_path
    try:
        source_bytes = absolute_path.read_bytes()
        source = source_bytes.decode("utf-8")
    except FileNotFoundError as error:
        raise ManifestStabilityError(
            "source_content_changed", [relative_path.as_posix()]
        ) from error
    except (OSError, UnicodeError) as error:
        raise ResourceConsumerManifestError(
            f"failed to read UTF-8 Rust source {relative_path.as_posix()}: {error}"
        ) from error
    return relative_path, source_bytes, source


def _scan_candidates(
    repo_root: Path,
    candidates: Sequence[Path],
    *,
    collect_consumers: bool = True,
) -> tuple[list[dict[str, object]], list[dict[str, object]]]:
    candidate_fingerprints: list[dict[str, object]] = []
    consumers: list[dict[str, object]] = []

    def read_source(path: Path) -> tuple[Path, bytes, str]:
        return _read_candidate_source(repo_root, path)

    for relative_path, source_bytes, source in _bounded_ordered_map(
        read_source, candidates
    ):
        if not _may_reference_resource(source_bytes):
            continue
        source_sha256 = _sha256_bytes(source_bytes)
        display_path = relative_path.as_posix()
        candidate_fingerprints.append(
            {
                "bytes": len(source_bytes),
                "path": display_path,
                "sha256": source_sha256,
            }
        )

        if not collect_consumers:
            continue

        code_view = _rust_code_view(source)
        literal_match = RESOURCE_LITERAL_PATH.search(code_view) is not None
        structured_match = any(
            tuple(path[: len(prefix)]) == prefix
            for path, _alias, _line in _rust_use_paths(code_view)
            for prefix in RESOURCE_PATH_PREFIXES
        )
        if literal_match or structured_match:
            consumers.append(
                {
                    "bytes": len(source_bytes),
                    "literal": literal_match,
                    "path": display_path,
                    "sha256": source_sha256,
                    "structured": structured_match,
                }
            )

    return candidate_fingerprints, consumers


def _semantic_candidate_paths(
    candidate_fingerprints: Sequence[dict[str, object]],
) -> tuple[Path, ...]:
    return tuple(Path(str(fingerprint["path"])) for fingerprint in candidate_fingerprints)


def _changed_fingerprint_paths(
    before: Sequence[dict[str, object]], after: Sequence[dict[str, object]]
) -> list[str]:
    after_by_path = {str(fingerprint["path"]): fingerprint for fingerprint in after}
    return [
        str(fingerprint["path"])
        for fingerprint in before
        if after_by_path.get(str(fingerprint["path"])) != fingerprint
    ]


def _root_counts(consumers: Sequence[dict[str, object]]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for consumer in consumers:
        root = str(consumer["path"]).split("/", 1)[0]
        counts[root] = counts.get(root, 0) + 1
    return dict(sorted(counts.items(), key=lambda item: (item[0].casefold(), item[0])))


def _resource_consumer_candidates(
    repo_root: Path,
    owner_roots: Sequence[str],
    rust_candidates: Sequence[Path] | None = None,
) -> tuple[Path, ...]:
    source_candidates = (
        _git_resource_token_candidates(repo_root)
        if rust_candidates is None
        else rust_candidates
    )
    return tuple(
        path
        for path in source_candidates
        if not _excluded_repository_path(path)
        and not _inside_owner_root(path, owner_roots)
    )


def _candidate_set_changed_paths(
    before: Sequence[Path], after: Sequence[Path]
) -> list[str]:
    return sorted(
        {path.as_posix() for path in set(before).symmetric_difference(after)},
        key=lambda path: (path.casefold(), path),
    )


def capture_resource_consumer_snapshot(
    repo_root: Path,
    *,
    owner_roots: Sequence[str] = DEFAULT_OWNER_ROOTS,
    rust_candidates: Sequence[Path] | None = None,
) -> ResourceConsumerCapture:
    repo_root = repo_root.resolve()
    normalized_owner_roots = _normalized_owner_roots(owner_roots)
    rust_candidates = _resource_consumer_candidates(
        repo_root, normalized_owner_roots, rust_candidates
    )
    candidate_fingerprints, consumers = _scan_candidates(repo_root, rust_candidates)
    return ResourceConsumerCapture(
        candidates=_semantic_candidate_paths(candidate_fingerprints),
        candidate_fingerprints=tuple(candidate_fingerprints),
        consumers=tuple(consumers),
        owner_roots=normalized_owner_roots,
    )


def finalize_resource_consumer_snapshot(
    repo_root: Path,
    capture: ResourceConsumerCapture,
    *,
    rust_candidates: Sequence[Path] | None = None,
) -> ResourceConsumerSnapshot:
    repo_root = repo_root.resolve()
    final_rust_candidates = _resource_consumer_candidates(
        repo_root, capture.owner_roots, rust_candidates
    )
    final_fingerprints, _ = _scan_candidates(
        repo_root, final_rust_candidates, collect_consumers=False
    )
    final_candidates = _semantic_candidate_paths(final_fingerprints)
    if capture.candidates != final_candidates:
        raise ManifestStabilityError(
            "candidate_set_changed",
            _candidate_set_changed_paths(capture.candidates, final_candidates),
        )
    changed_paths = _changed_fingerprint_paths(
        capture.candidate_fingerprints, final_fingerprints
    )
    if changed_paths:
        raise ManifestStabilityError("source_content_changed", changed_paths)
    literal_count = sum(bool(consumer["literal"]) for consumer in capture.consumers)
    structured_count = sum(
        bool(consumer["structured"]) for consumer in capture.consumers
    )
    both_count = sum(
        bool(consumer["literal"]) and bool(consumer["structured"])
        for consumer in capture.consumers
    )
    report = {
        "candidate_count": len(capture.candidate_fingerprints),
        "candidate_manifest_sha256": _manifest_sha256(
            capture.candidate_fingerprints
        ),
        "consumer_count": len(capture.consumers),
        "consumer_manifest_sha256": _manifest_sha256(capture.consumers),
        "consumer_root_counts": _root_counts(capture.consumers),
        "consumers": list(capture.consumers),
        "excluded_owner_roots": list(capture.owner_roots),
        "match_counts": {
            "both": both_count,
            "literal": literal_count,
            "literal_only": literal_count - both_count,
            "structured": structured_count,
            "structured_only": structured_count - both_count,
        },
        "schema_version": SCHEMA_VERSION,
        "stability": {
            "candidate_set": True,
            "source_content": True,
        },
    }
    return ResourceConsumerSnapshot(
        report=report,
        candidates=capture.candidates,
        candidate_fingerprints=capture.candidate_fingerprints,
        owner_roots=capture.owner_roots,
    )


def build_resource_consumer_snapshot(
    repo_root: Path,
    *,
    owner_roots: Sequence[str] = DEFAULT_OWNER_ROOTS,
) -> ResourceConsumerSnapshot:
    capture = capture_resource_consumer_snapshot(
        repo_root,
        owner_roots=owner_roots,
    )
    return finalize_resource_consumer_snapshot(
        repo_root,
        capture,
    )


def build_resource_consumer_manifest(
    repo_root: Path,
    *,
    owner_roots: Sequence[str] = DEFAULT_OWNER_ROOTS,
) -> dict[str, object]:
    return build_resource_consumer_snapshot(
        repo_root,
        owner_roots=owner_roots,
    ).report


def revalidate_resource_consumer_snapshot(
    repo_root: Path,
    snapshot: ResourceConsumerSnapshot,
    *,
    rust_candidates: Sequence[Path] | None = None,
) -> None:
    repo_root = repo_root.resolve()
    rust_candidates = _resource_consumer_candidates(
        repo_root, snapshot.owner_roots, rust_candidates
    )
    candidate_fingerprints, _ = _scan_candidates(
        repo_root, rust_candidates, collect_consumers=False
    )
    candidates = _semantic_candidate_paths(candidate_fingerprints)
    if candidates != snapshot.candidates:
        raise ManifestStabilityError(
            "candidate_set_changed",
            _candidate_set_changed_paths(snapshot.candidates, candidates),
        )
    changed_paths = _changed_fingerprint_paths(
        snapshot.candidate_fingerprints, candidate_fingerprints
    )
    if changed_paths:
        raise ManifestStabilityError("source_content_changed", changed_paths)


def write_resource_consumer_manifest(
    report: dict[str, object], output_path: Path
) -> None:
    output_path = output_path.resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    payload = json.dumps(
        report,
        ensure_ascii=False,
        indent=2,
        sort_keys=True,
    ).encode("utf-8") + b"\n"
    descriptor, temporary_name = tempfile.mkstemp(
        dir=output_path.parent,
        prefix=f".{output_path.name}.",
        suffix=".tmp",
    )
    temporary_path = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(payload)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary_path, output_path)
    finally:
        if temporary_path.exists():
            temporary_path.unlink()


def _parse_arguments(arguments: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build a stable Frameworks01 Resource consumer union manifest."
    )
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--owner-root", action="append", default=[])
    return parser.parse_args(arguments)


def main(arguments: Sequence[str] | None = None) -> int:
    parsed = _parse_arguments(sys.argv[1:] if arguments is None else arguments)
    owner_roots = parsed.owner_root or list(DEFAULT_OWNER_ROOTS)
    try:
        report = build_resource_consumer_manifest(
            parsed.repo_root,
            owner_roots=owner_roots,
        )
        write_resource_consumer_manifest(report, parsed.output)
    except ResourceConsumerManifestError as error:
        print(str(error), file=sys.stderr)
        return 2
    print(
        json.dumps(
            {
                "candidate_count": report["candidate_count"],
                "consumer_count": report["consumer_count"],
                "consumer_manifest_sha256": report["consumer_manifest_sha256"],
                "output": str(parsed.output.resolve()),
            },
            ensure_ascii=False,
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Validate and replay a Zircon PBR viewer RenderDoc capture."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
import tempfile
import threading
from dataclasses import asdict, dataclass
from pathlib import Path

_DEFAULT_RENDERDOCCMD = Path(r"D:\Tools\renderdoc\renderdoccmd.exe")
_DEFAULT_TIMEOUT_SECONDS = 120
_MAX_TIMEOUT_SECONDS = 600
_HASH_CHUNK_BYTES = 1024 * 1024
_REPLAY_OUTPUT_TAIL_BYTES = 16 * 1024


@dataclass(frozen=True)
class RenderDocReplayEvidence:
    capture_path: Path
    capture_size_bytes: int
    capture_sha256: str
    renderdoccmd: Path
    replay_uses_verified_snapshot: bool
    replay_returncode: int


@dataclass(frozen=True)
class _CaptureIdentity:
    size_bytes: int
    sha256: str


@dataclass(frozen=True)
class _ReplayProcessResult:
    returncode: int
    stdout_tail: str
    stderr_tail: str


class _BoundedByteTail:
    def __init__(self, byte_limit: int) -> None:
        self._byte_limit = byte_limit
        self._data = bytearray()

    def append(self, data: bytes) -> None:
        if len(data) >= self._byte_limit:
            self._data = bytearray(data[-self._byte_limit :])
            return
        overflow = len(self._data) + len(data) - self._byte_limit
        if overflow > 0:
            del self._data[:overflow]
        self._data.extend(data)

    def text(self) -> str:
        return self._data.decode("utf-8", errors="replace")


def validate_renderdoc_replay(
    capture_path: Path,
    *,
    executable: Path = _DEFAULT_RENDERDOCCMD,
    timeout_seconds: int = _DEFAULT_TIMEOUT_SECONDS,
) -> RenderDocReplayEvidence:
    """Replay one regular `.rdc` capture and retain its immutable identity."""

    capture_path = _validated_capture_path(capture_path)
    if not 1 <= timeout_seconds <= _MAX_TIMEOUT_SECONDS:
        raise ValueError(
            "RenderDoc replay timeout must be between "
            f"1 and {_MAX_TIMEOUT_SECONDS} seconds: {timeout_seconds}"
        )
    source_command = [
        str(executable),
        "replay",
        "--loops",
        "1",
        str(capture_path),
    ]
    identity_before = _capture_identity(capture_path)
    snapshot_path, snapshot_identity_before = _create_verified_snapshot(
        capture_path, identity_before
    )
    command = [*source_command[:-1], str(snapshot_path)]
    primary_error: BaseException | None = None
    try:
        timeout_error = None
        try:
            completed = _run_replay_process(command, timeout_seconds)
        except FileNotFoundError as error:
            raise RuntimeError(
                "RenderDoc command is unavailable: "
                f"{_identity_details(capture_path, identity_before, command)}"
            ) from error
        except subprocess.TimeoutExpired as error:
            completed = None
            timeout_error = error
        except OSError as error:
            raise RuntimeError(
                "RenderDoc command is unavailable: "
                f"{_identity_details(capture_path, identity_before, command)}"
            ) from error
        snapshot_identity_after = _snapshot_identity_after_replay(
            snapshot_path, command, snapshot_identity_before
        )
        _require_unchanged_snapshot(
            snapshot_path,
            command,
            snapshot_identity_before,
            snapshot_identity_after,
        )
        identity_after = _capture_identity_after_replay(
            capture_path, command, identity_before
        )
        _require_unchanged_capture(capture_path, command, identity_before, identity_after)
        if timeout_error is not None:
            raise RuntimeError(
                "RenderDoc replay timed out: "
                f"timeout_seconds={timeout_seconds} "
                f"{_identity_details(capture_path, identity_before, command)}"
            ) from timeout_error
        assert completed is not None
        if completed.returncode:
            raise RuntimeError(
                "RenderDoc replay failed: "
                f"returncode={completed.returncode} "
                f"{_identity_details(capture_path, identity_before, command)} "
                f"stdout_tail={completed.stdout_tail!r} "
                f"stderr_tail={completed.stderr_tail!r}"
            )
        return RenderDocReplayEvidence(
            capture_path=capture_path,
            capture_size_bytes=identity_before.size_bytes,
            capture_sha256=identity_before.sha256,
            renderdoccmd=executable,
            replay_uses_verified_snapshot=True,
            replay_returncode=completed.returncode,
        )
    except BaseException as error:
        primary_error = error
        raise
    finally:
        try:
            _remove_snapshot(snapshot_path)
        except RuntimeError as error:
            if primary_error is None:
                raise
            raise RuntimeError(
                f"{error}; the prior replay failure is chained as the cause"
            ) from primary_error


def _validated_capture_path(capture_path: Path) -> Path:
    try:
        resolved_path = capture_path.resolve(strict=True)
    except OSError as error:
        raise RuntimeError(f"RenderDoc capture is unavailable: {capture_path}") from error
    if resolved_path.suffix != ".rdc":
        raise RuntimeError(
            "RenderDoc capture must use a lowercase .rdc extension: "
            f"{resolved_path}"
        )
    if not resolved_path.is_file():
        raise RuntimeError(f"RenderDoc capture is not a regular file: {resolved_path}")
    if resolved_path.stat().st_size <= 0:
        raise RuntimeError(f"RenderDoc capture is empty: {resolved_path}")
    return resolved_path


def _sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as capture_file:
        while chunk := capture_file.read(_HASH_CHUNK_BYTES):
            digest.update(chunk)
    return digest.hexdigest()


def _capture_identity(path: Path) -> _CaptureIdentity:
    size_before_hash = path.stat().st_size
    identity = _CaptureIdentity(
        size_bytes=size_before_hash,
        sha256=_sha256_file(path),
    )
    if path.stat().st_size != size_before_hash:
        raise RuntimeError(f"RenderDoc capture changed while hashing: {path}")
    return identity


def _create_verified_snapshot(
    capture_path: Path, identity_before: _CaptureIdentity
) -> tuple[Path, _CaptureIdentity]:
    descriptor, snapshot_name = tempfile.mkstemp(
        prefix="zircon_shader_pbr_renderdoc_replay_",
        suffix=".rdc",
        dir=capture_path.parent,
    )
    snapshot_path = Path(snapshot_name)
    try:
        with os.fdopen(descriptor, "wb") as snapshot_file, capture_path.open("rb") as source_file:
            while chunk := source_file.read(_HASH_CHUNK_BYTES):
                snapshot_file.write(chunk)
        snapshot_identity = _capture_identity(snapshot_path)
        identity_after_copy = _capture_identity(capture_path)
        if snapshot_identity != identity_before or identity_after_copy != identity_before:
            raise RuntimeError(
                "RenderDoc capture changed while snapshotting: "
                f"capture={capture_path} capture_size_bytes={identity_before.size_bytes} "
                f"sha256={identity_before.sha256} "
                f"snapshot_size_bytes={snapshot_identity.size_bytes} "
                f"snapshot_sha256={snapshot_identity.sha256}"
            )
        return snapshot_path, snapshot_identity
    except Exception as error:
        try:
            _remove_snapshot(snapshot_path)
        except RuntimeError as cleanup_error:
            raise RuntimeError(
                f"{cleanup_error}; the snapshotting failure is chained as the cause"
            ) from error
        raise


def _run_replay_process(
    command: list[str], timeout_seconds: int
) -> _ReplayProcessResult:
    process = subprocess.Popen(
        command,
        stderr=subprocess.PIPE,
        stdout=subprocess.PIPE,
    )
    assert process.stdout is not None
    assert process.stderr is not None
    stdout_tail = _BoundedByteTail(_REPLAY_OUTPUT_TAIL_BYTES)
    stderr_tail = _BoundedByteTail(_REPLAY_OUTPUT_TAIL_BYTES)
    readers = [
        threading.Thread(target=_read_output_tail, args=(process.stdout, stdout_tail)),
        threading.Thread(target=_read_output_tail, args=(process.stderr, stderr_tail)),
    ]
    for reader in readers:
        reader.start()
    try:
        returncode = process.wait(timeout=timeout_seconds)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait()
        raise
    finally:
        for reader in readers:
            reader.join()
    return _ReplayProcessResult(
        returncode=returncode,
        stdout_tail=stdout_tail.text(),
        stderr_tail=stderr_tail.text(),
    )


def _read_output_tail(stream, tail: _BoundedByteTail) -> None:
    try:
        while chunk := stream.read(_HASH_CHUNK_BYTES):
            tail.append(chunk)
    finally:
        stream.close()


def _remove_snapshot(snapshot_path: Path) -> None:
    try:
        snapshot_path.unlink(missing_ok=True)
    except OSError as error:
        raise RuntimeError(
            f"RenderDoc replay snapshot cleanup failed: snapshot={snapshot_path}"
        ) from error


def _capture_identity_after_replay(
    capture_path: Path,
    command: list[str],
    identity_before: _CaptureIdentity,
) -> _CaptureIdentity:
    try:
        return _capture_identity(capture_path)
    except (OSError, RuntimeError) as error:
        raise RuntimeError(
            "RenderDoc capture changed during replay: "
            f"{_identity_details(capture_path, identity_before, command)} "
            f"after_state=unavailable_or_unstable"
        ) from error


def _snapshot_identity_after_replay(
    snapshot_path: Path,
    command: list[str],
    identity_before: _CaptureIdentity,
) -> _CaptureIdentity:
    try:
        return _capture_identity(snapshot_path)
    except (OSError, RuntimeError) as error:
        raise RuntimeError(
            "RenderDoc replay snapshot changed during replay: "
            f"snapshot={snapshot_path} snapshot_size_bytes={identity_before.size_bytes} "
            f"snapshot_sha256={identity_before.sha256} command={command!r} "
            "after_state=unavailable_or_unstable"
        ) from error


def _require_unchanged_snapshot(
    snapshot_path: Path,
    command: list[str],
    identity_before: _CaptureIdentity,
    identity_after: _CaptureIdentity,
) -> None:
    if identity_before != identity_after:
        raise RuntimeError(
            "RenderDoc replay snapshot changed during replay: "
            f"snapshot={snapshot_path} snapshot_size_bytes={identity_before.size_bytes} "
            f"snapshot_sha256={identity_before.sha256} command={command!r} "
            f"after_snapshot_size_bytes={identity_after.size_bytes} "
            f"after_snapshot_sha256={identity_after.sha256}"
        )


def _require_unchanged_capture(
    capture_path: Path,
    command: list[str],
    identity_before: _CaptureIdentity,
    identity_after: _CaptureIdentity,
) -> None:
    if identity_before != identity_after:
        raise RuntimeError(
            "RenderDoc capture changed during replay: "
            f"{_identity_details(capture_path, identity_before, command)} "
            f"after_capture_size_bytes={identity_after.size_bytes} "
            f"after_sha256={identity_after.sha256}"
        )


def _identity_details(
    capture_path: Path,
    identity: _CaptureIdentity,
    command: list[str],
) -> str:
    return (
        f"capture={capture_path} capture_size_bytes={identity.size_bytes} "
        f"sha256={identity.sha256} command={command!r}"
    )


def _parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Replay a Zircon PBR RenderDoc capture and print its immutable evidence."
    )
    parser.add_argument("capture", type=Path, help="Lowercase .rdc capture file")
    parser.add_argument(
        "--renderdoccmd",
        type=Path,
        default=_DEFAULT_RENDERDOCCMD,
        help=f"RenderDoc command path (default: {_DEFAULT_RENDERDOCCMD})",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=int,
        default=_DEFAULT_TIMEOUT_SECONDS,
        help=f"Replay timeout in seconds, 1..{_MAX_TIMEOUT_SECONDS}",
    )
    return parser.parse_args()


def main() -> int:
    arguments = _parse_arguments()
    try:
        evidence = validate_renderdoc_replay(
            arguments.capture,
            executable=arguments.renderdoccmd,
            timeout_seconds=arguments.timeout_seconds,
        )
    except (OSError, RuntimeError, ValueError) as error:
        print(f"PBR RenderDoc replay validation failed: {error}", file=sys.stderr)
        return 1
    output = asdict(evidence)
    output["capture_path"] = str(evidence.capture_path)
    output["renderdoccmd"] = str(evidence.renderdoccmd)
    print(json.dumps(output, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

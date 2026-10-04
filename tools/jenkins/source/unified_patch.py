"""Strict, read-only application of a Git unified diff to memory.

The parser intentionally does not invoke Git or perform a three-way merge.  A
caller supplies the complete before-hash scope; every file is read and checked
before any result is returned.
"""

from __future__ import annotations

import hashlib
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Mapping

from ..contracts import JenkinsError
from . import canonical_path


@dataclass(frozen=True)
class Hunk:
    old_start: int
    old_count: int
    new_start: int
    new_count: int
    # (kind, logical line, has-newline-in-diff-stream)
    lines: tuple[tuple[str, str, bool], ...]


@dataclass(frozen=True)
class PatchFile:
    path: str
    old_path: str | None
    new_path: str | None
    hunks: tuple[Hunk, ...]
    old_exists: bool
    new_exists: bool


@dataclass(frozen=True)
class UnifiedPatch:
    files: tuple[PatchFile, ...]

    @property
    def paths(self) -> tuple[str, ...]:
        return tuple(item.path for item in self.files)


_HUNK = re.compile(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@(?: .*)?$")
_HEADER = re.compile(r"^(---|\+\+\+) (.+?)(?:\t.*)?$")


def _error(code: str, message: str, **details: object) -> JenkinsError:
    return JenkinsError(code, message, details=details or None)


def _logical_lines(text: str) -> list[tuple[str, bool]]:
    result: list[tuple[str, bool]] = []
    for value in text.splitlines(keepends=True):
        if value.endswith("\r\n"):
            result.append((value[:-2], True))
        elif value.endswith(("\n", "\r")):
            result.append((value[:-1], True))
        else:
            result.append((value, False))
    return result


def _header_path(value: str) -> str | None:
    value = value.split("\t", 1)[0]
    if value == "/dev/null":
        return None
    if value.startswith("a/") or value.startswith("b/"):
        value = value[2:]
    return value


def _validate_patch_path(value: str | None) -> None:
    if value is None:
        return
    raw = value.replace("\\", "/")
    parts = raw.split("/")
    reserved = {"con", "prn", "aux", "nul"} | {f"com{i}" for i in range(1, 10)} | {f"lpt{i}" for i in range(1, 10)}
    if (not raw or raw.startswith("/") or (len(raw) >= 2 and raw[1] == ":") or
            any(part in {"", ".", ".."} or ":" in part or any(ch in part for ch in '*?"<>|') or
                part.rstrip(" .") != part or part.casefold().split(".", 1)[0] in reserved for part in parts)):
        raise _error("invalid_source_path", "patch path is not a safe repository-relative path", path=value)


def _normalized_path(value: str | None) -> str | None:
    if value is None:
        return None
    _validate_patch_path(value)
    return value.replace("\\", "/").casefold()


def _git_header_paths(line: str) -> tuple[str | None, str | None]:
    value = line[len("diff --git "):]
    match = re.fullmatch(r"(?:a/)?(.+) (?:b/)?(.+)", value)
    if not match:
        raise _error("patch_malformed", "malformed git diff header")
    old, new = match.group(1), match.group(2)
    return (None if old == "/dev/null" else _normalized_path(old),
            None if new == "/dev/null" else _normalized_path(new))


def parse_unified_patch(text: str) -> UnifiedPatch:
    if not isinstance(text, str) or "\x00" in text:
        raise _error("patch_invalid", "unified patch must be text without NUL bytes")
    raw = text.splitlines(keepends=True)
    files: list[PatchFile] = []
    index = 0
    while index < len(raw):
        line = raw[index].rstrip("\r\n")
        if not line:
            index += 1
            continue
        if line.startswith("Binary files ") or line.startswith("GIT binary patch"):
            raise _error("patch_binary", "binary patches are not supported")
        if line.startswith("similarity index") or line.startswith("rename from") or line.startswith("rename to") or line.startswith("copy from") or line.startswith("copy to"):
            raise _error("patch_rename", "renames and copies are not supported")
        if not line.startswith("diff --git "):
            raise _error("patch_malformed", "expected a git diff header", line=line)
        git_old, git_new = _git_header_paths(line)
        index += 1
        # Metadata is allowed only for ordinary 100644 creation/deletion.
        while index < len(raw) and not raw[index].rstrip("\r\n").startswith("--- "):
            metadata = raw[index].rstrip("\r\n")
            if metadata.startswith(("old mode ", "new mode ", "new file mode ", "deleted file mode ")):
                if not (re.fullmatch(r"(?:new file mode|deleted file mode) 100644", metadata)):
                    raise _error("patch_mode", "file mode changes are not supported")
            elif metadata.startswith("index "):
                if not re.fullmatch(r"index [0-9a-fA-F]+\.\.[0-9a-fA-F]+(?: 100644)?", metadata):
                    raise _error("patch_mode", "malformed or unsupported index mode")
            elif metadata:
                raise _error("patch_malformed", "unsupported diff metadata", line=metadata)
            index += 1
        if index >= len(raw):
            raise _error("patch_truncated", "diff has no file headers")
        match_old = _HEADER.match(raw[index].rstrip("\r\n"))
        if not match_old or match_old.group(1) != "---":
            raise _error("patch_malformed", "missing old file header")
        old_path = _normalized_path(_header_path(match_old.group(2))); index += 1
        if index >= len(raw):
            raise _error("patch_truncated", "diff has no new file header")
        match_new = _HEADER.match(raw[index].rstrip("\r\n"))
        if not match_new or match_new.group(1) != "+++":
            raise _error("patch_malformed", "missing new file header")
        new_path = _normalized_path(_header_path(match_new.group(2))); index += 1
        if old_path is None and new_path is None:
            raise _error("patch_malformed", "both file headers cannot be /dev/null")
        if old_path is not None and new_path is not None and old_path != new_path:
            raise _error("patch_rename", "old and new paths differ; renames are not supported")
        if git_old is not None and old_path is not None and git_old != old_path:
            raise _error("patch_malformed", "git and file headers disagree")
        if git_new is not None and new_path is not None and git_new != new_path:
            raise _error("patch_malformed", "git and file headers disagree")
        canonical = new_path or old_path
        if canonical is None:
            raise _error("patch_malformed", "file path is missing")
        hunks: list[Hunk] = []
        while index < len(raw):
            current = raw[index].rstrip("\r\n")
            if current.startswith("diff --git "):
                break
            hm = _HUNK.match(current)
            if not hm:
                if current.startswith("\\ No newline at end of file"):
                    raise _error("patch_malformed", "newline marker has no preceding hunk line")
                if current:
                    raise _error("patch_malformed", "unexpected content outside hunk", line=current)
                index += 1; continue
            old_start = int(hm.group(1)); old_count = int(hm.group(2) or "1")
            new_start = int(hm.group(3)); new_count = int(hm.group(4) or "1")
            if old_start < 0 or new_start < 0 or old_count < 0 or new_count < 0:
                raise _error("patch_malformed", "hunk offsets and counts must be non-negative")
            entries: list[tuple[str, str, bool]] = []; old_seen = new_seen = 0; index += 1
            while index < len(raw):
                source = raw[index]
                value = source.rstrip("\r\n")
                if value.startswith("@@ ") or value.startswith("diff --git "):
                    break
                if value == "\\ No newline at end of file":
                    if not entries:
                        raise _error("patch_malformed", "newline marker has no preceding line")
                    kind, body, had_stream_newline = entries[-1]
                    if not had_stream_newline:
                        raise _error("patch_malformed", "duplicate newline marker")
                    entries[-1] = (kind, body, False); index += 1; continue
                if not value or value[0] not in " +-":
                    raise _error("patch_malformed", "hunk line lacks a prefix")
                kind = value[0]; entries.append((kind, value[1:], bool(source.endswith(("\n", "\r")))))
                if kind in " -": old_seen += 1
                if kind in " +": new_seen += 1
                index += 1
            if old_seen != old_count or new_seen != new_count:
                raise _error("patch_truncated", "hunk line counts do not match", expected=(old_count, new_count), observed=(old_seen, new_seen))
            hunks.append(Hunk(old_start, old_count, new_start, new_count, tuple(entries)))
        if not hunks:
            raise _error("patch_malformed", "file has no hunks")
        path = new_path or old_path
        files.append(PatchFile(path, old_path, new_path, tuple(hunks), old_path is not None, new_path is not None))
    if not files:
        raise _error("patch_empty", "unified patch is empty")
    seen: set[str] = set()
    for item in files:
        key = item.path
        if key in seen:
            raise _error("patch_path_collision", "patch contains duplicate case-insensitive paths", path=item.path)
        seen.add(key)
    return UnifiedPatch(tuple(files))


def _apply_file(root: Path, item: PatchFile, expected: str | None) -> str | None:
    path = canonical_path(root, item.path)
    target = root.joinpath(*path.split("/"))
    existing = target.read_bytes() if target.is_file() else None
    actual = None if existing is None else hashlib.sha256(existing).hexdigest()
    if actual != expected:
        raise _error("before_hash_mismatch", "source differs from supplied before hash", path=path, expected=expected, observed=actual)
    if not item.old_exists:
        if expected is not None: raise _error("before_hash_mismatch", "new file already exists", path=path)
        old_lines: list[tuple[str, bool]] = []
    else:
        if existing is None: raise _error("patch_source_missing", "patch source file is missing", path=path)
        try: old_lines = _logical_lines(existing.decode("utf-8"))
        except UnicodeDecodeError as exc: raise _error("patch_binary", "only UTF-8 text files are supported", path=path) from exc
    cursor = 0; result: list[tuple[str, bool]] = []
    previous_old_start = -1
    for hunk in item.hunks:
        if hunk.old_count == 0:
            start = hunk.old_start
        else:
            start = hunk.old_start - 1
        if hunk.old_start < 0 or hunk.new_start < 0 or start < previous_old_start:
            raise _error("patch_context_mismatch", "hunk positions overlap or are invalid", path=path)
        if start < cursor or start > len(old_lines): raise _error("patch_context_mismatch", "hunk position is invalid", path=path)
        expected_new_start = len(result) + (start - cursor) + (0 if hunk.new_count == 0 else 1)
        if hunk.new_start != expected_new_start:
            raise _error("patch_context_mismatch", "new hunk position is invalid", path=path,
                         expected=expected_new_start, observed=hunk.new_start)
        result.extend(old_lines[cursor:start]); cursor = start
        previous_old_start = start
        for kind, body, marker_newline in hunk.lines:
            if kind == " ":
                if cursor >= len(old_lines) or old_lines[cursor][0] != body: raise _error("patch_context_mismatch", "context line differs", path=path)
                if not marker_newline and old_lines[cursor][1]: raise _error("patch_context_mismatch", "newline marker differs from source", path=path)
                result.append(old_lines[cursor]); cursor += 1
            elif kind == "-":
                if cursor >= len(old_lines) or old_lines[cursor][0] != body: raise _error("patch_context_mismatch", "deletion line differs", path=path)
                if not marker_newline and old_lines[cursor][1]: raise _error("patch_context_mismatch", "newline marker differs from source", path=path)
                cursor += 1
            else:
                ended = marker_newline
                result.append((body, ended))
    result.extend(old_lines[cursor:])
    if not item.new_exists:
        if cursor != len(old_lines):
            raise _error("patch_context_mismatch", "deletion patch does not consume the complete file", path=path)
        return None
    if any(not ended for _, ended in result[:-1]):
        raise _error("patch_malformed", "a no-newline marker may only terminate the resulting file", path=path)
    style = "\r\n" if existing is not None and b"\r\n" in existing else "\n"
    return "".join(body + (style if ended else "") for body, ended in result)


def materialize_patch(repository_root: str | Path, patch: UnifiedPatch | str,
                      before_hashes: Mapping[str, str | None]) -> dict[str, str | None]:
    parsed = parse_unified_patch(patch) if isinstance(patch, str) else patch
    if not isinstance(parsed, UnifiedPatch): raise _error("patch_invalid", "parsed patch type is invalid")
    root = Path(repository_root).absolute()
    expected: dict[str, str | None] = {}
    for key, value in before_hashes.items():
        canonical = canonical_path(root, key)
        if canonical in expected:
            raise _error("patch_path_collision", "before-hash scope contains duplicate case-insensitive paths", path=key)
        expected[canonical] = value
    patch_paths = {canonical_path(root, item.path) for item in parsed.files}
    if set(expected) != patch_paths: raise _error("patch_scope_mismatch", "before-hash scope differs from patch paths")
    return {canonical_path(root, item.path): _apply_file(root, item, expected[canonical_path(root, item.path)]) for item in parsed.files}


__all__ = ["Hunk", "PatchFile", "UnifiedPatch", "parse_unified_patch", "materialize_patch"]

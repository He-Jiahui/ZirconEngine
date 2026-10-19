#!/usr/bin/env python3
"""Static self-check for a ZirconEngine change before Jenkins handoff.

`patch` checks a unified diff (from a file, stdin, or read-only Git queries);
`scan` checks whole files for the content rules. The guard reads files only.
It is not a build, a test run, or Jenkins acceptance; rule meanings live in
references/defect-classes.md.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tomllib
from dataclasses import dataclass, field
from pathlib import Path, PurePosixPath

TOOL = "zircon-change-guard"
VERSION = 2
ERROR, WARN, INFO = "error", "warn", "info"

MARKER_ID_PATTERN = r"CR-[A-Za-z0-9_]+(?:-[A-Za-z0-9_]+)*-\d{4}"
MARKER_LINE = re.compile(r"(?:^|\s)(?://+!?|#)\s*BUG:")
MARKER_ID = re.compile(rf"BUG:\s*\[({MARKER_ID_PATTERN})\]")
MARKER_WELL_FORMED = re.compile(rf"BUG:\s*\[{MARKER_ID_PATTERN}\]\s+\S")
ALLOW = re.compile(r"guard-allow\(([a-z0-9.-]+)\):\s*(\S.{7,})")

SLASH_COMMENT_EXT = {
    ".rs", ".ts", ".tsx", ".js", ".mjs", ".cjs", ".groovy", ".wgsl", ".c", ".cc",
    ".cpp", ".h", ".hpp", ".cs", ".java", ".kt", ".jsonc", ".slint",
}
HASH_COMMENT_EXT = {".py", ".ps1", ".psm1", ".toml", ".yml", ".yaml", ".sh", ".ini", ".cfg"}
DOC_EXT = {".md", ".txt", ".rst"}
SCAN_EXT = SLASH_COMMENT_EXT | HASH_COMMENT_EXT | {".json"}
SKIP_DIRS = {".git", "target", "node_modules", "__pycache__", ".venv"}

TEST_FILE = re.compile(
    r"(^test_.*\.py$|_tests?\.(py|rs)$|^tests?\.rs$|^conftest\.py$|\.Tests\.ps1$"
    r"|\.(spec|test)\.[cm]?[jt]sx?$|_test_support\.rs$)"
)
TEST_DIRS = {"tests", "test", "fixtures", "testdata", "test_support"}

CONTROL_PLANE = (
    ".github/workflows/", ".jenkins/", "tools/jenkins/", ".codex/hooks/",
    ".codex/hooks.json", ".codex/config.toml", "AGENTS.md",
)
LOCKFILES = {"Cargo.lock", "package-lock.json", "pnpm-lock.yaml", "yarn.lock"}
BACKUP_NAME = re.compile(
    r"(?i)(\.(bak|orig|rej|swp|swo|tmp|old)$|~$|( - copy| copy( \d+)?|\.copy| \(\d+\))(\.\w+)?$)"
)
BACKUP_DIRS = {"backup", "backups", "bak", "scratch"}

UNSAFE_USE = re.compile(r"\bunsafe\s*\{|\bunsafe\s+impl\b")
PUB_UNSAFE_FN = re.compile(r"^\s*pub(?:\([^)]*\))?\s+(?:const\s+)?(?:async\s+)?unsafe\s+(?:extern\s+(?:\"[^\"]*\"\s+)?)?fn\b")
PUB_FN = re.compile(r"^\s*pub\s+(?:const\s+)?(?:async\s+)?(?:extern\s+(?:\"[^\"]*\"\s+)?)?fn\s+(?:r#)?(\w+)")
FN_POINTER_TYPE = re.compile(r"\b(?:unsafe\s+)?(?:extern\s+\"[^\"]*\"\s+)?(?:fn|Fn|FnMut|FnOnce)\s*\(")
SAFETY_NOTE = re.compile(r"(?i)\bsafety\b\s*:")
SAFETY_DOC = re.compile(r"(?i)^\s*(///|//!|/\*\*|\*).*#\s*safety\b")
RAW_POINTER = re.compile(r"\*\s*(?:const|mut)\b")

DELETE_CALL = {
    ".py": re.compile(r"\bshutil\.rmtree\s*\("),
    ".rs": re.compile(r"\bremove_dir_all\s*\("),
    ".ps1": re.compile(r"(?i)\b(?:Remove-Item|rm|rmdir|del|ri)\b[^#\n]*-Recurse\b"),
    ".psm1": re.compile(r"(?i)\b(?:Remove-Item|rm|rmdir|del|ri)\b[^#\n]*-Recurse\b"),
}
CONTAINMENT = re.compile(
    r"is_relative_to\s*\(|\.relative_to\s*\(|commonpath\s*\(|starts_with\s*\(|strip_prefix\s*\("
    r"|\.StartsWith\s*\(|GetRelativePath\s*\(|\w*(?:within|inside|contained|containment|managed_root)\w*\s*\("
    r"|(?i:Assert-\w*(?:Inside|Managed|Within|Contained))"
)

STORAGE_FORBIDDEN = re.compile(
    r"""(?i)(?:[\\/]\s*["']?|(?:join|from|Path)\(\s*["']|-ChildPath\s+["']?)zirconbuilds\b|\b[def]:[\\/]+targets\b"""
)
CARGO_OUTPUT_KEY = re.compile(r"(?i)\bCARGO_(?:TARGET_DIR|BUILD_TARGET_DIR|BUILD_BUILD_DIR)\b|--target-dir\b|-TargetDir\b|\btarget[-_]dir[\"']?\s*[:=]")
QUOTED = re.compile(r"""(["'])(.*?)\1""")
UNQUOTED_VALUE = re.compile(r"""^\s*=?\s*([^\s"'$`{(\[]+)""")
LITERAL_LOCATION = re.compile(
    r"(?i)^(?:[a-z]:|/|~[\\/]|\.{1,2}[\\/]|(?:\.[\\/])?target(?:[\\/]|$))"
)
REJECTION_CONTEXT = re.compile(r"(?i)forbid|reject|deny|denied|disallow|legacy|blocked|invalid|not allowed|must not")

TEST_FN = re.compile(r"^\s*#\[(?:\w+::)*test\b|^\s*(?:async\s+)?def\s+test_\w*\s*\(")
TEST_DISABLE = re.compile(r"^\s*#\[ignore\b|@(?:unittest\.)?skip\w*\b|@pytest\.mark\.(?:skip|xfail)\b|\bit\.skip\(|\bdescribe\.skip\(")
CATCH_UNWIND = re.compile(r"\bcatch_unwind\s*\(")
SERDE_SAFE = re.compile(r"#\[serde\([^\]]*\btry_from\s*=")
DERIVE_DESERIALIZE = re.compile(r"#\[derive\([^\]]*\bDeserialize\b")
ITEM_NAME = re.compile(r"^\s*pub(?:\([^)]*\))?\s+(?:struct|enum)\s+(\w+)")

STRING_LITERAL = re.compile(r'"(?:[^"\\\n]|\\.)*"')
SOURCE_STRING = re.compile(r"""(?:"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')""")
CONFLICT = re.compile(r"^(<{7}|={7}|>{7})(?:\s|$)")
HUNK = re.compile(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@")
RUST_RAW = re.compile(r'(?:br|cr|r)(#*)"')
RUST_QUOTED = re.compile(r'(?:b|c)?"')
RUST_CHAR = re.compile(r"(?:b)?'(?:\\(?:u\{[0-9A-Fa-f_]+\}|x[0-9A-Fa-f]{2}|.)|[^'\\\n])'")
RUST_WORD = re.compile(r"(?:r#)?[A-Za-z_]\w*")

UNSUPPRESSIBLE_PREFIXES = ("scope.", "input.")
UNSUPPRESSIBLE = {"diff.conflict-marker", "marker.removed-without-fix"}


@dataclass
class Finding:
    severity: str
    rule: str
    path: str | None
    line: int | None
    message: str
    allowed_by: str | None = None

    @property
    def suppressible(self) -> bool:
        return self.rule not in UNSUPPRESSIBLE and not self.rule.startswith(UNSUPPRESSIBLE_PREFIXES)

    def as_dict(self) -> dict:
        data = {"severity": self.severity, "rule": self.rule, "path": self.path, "line": self.line, "message": self.message}
        if self.allowed_by is not None:
            data["allowedBy"] = self.allowed_by
        return data


@dataclass(eq=False)
class FileChange:
    old_path: str | None
    new_path: str | None
    added: dict[int, str] = field(default_factory=dict)
    removed: list[tuple[int, str]] = field(default_factory=list)
    context: dict[int, str] = field(default_factory=dict)
    binary: bool = False
    lines: list[str] | None = None
    created: bool = False
    deleted: bool = False
    saw_old_header: bool = False
    saw_new_header: bool = False
    last_old_end: int = 0
    last_new_end: int = 0
    rust_lines: list[str] | None = None
    comment_lines: list[str] | None = None

    @property
    def path(self) -> str:
        return self.new_path or self.old_path or ""

    @property
    def suffix(self) -> str:
        return PurePosixPath(self.path).suffix.lower()

    def line(self, number: int) -> str | None:
        if self.lines is not None:
            return self.lines[number - 1] if 1 <= number <= len(self.lines) else None
        return self.added.get(number, self.context.get(number))

    def code_line(self, number: int) -> str:
        if self.suffix != ".rs" or self.lines is None:
            return code_part(self.suffix, self.line(number) or "")
        if self.rust_lines is None:
            self.rust_lines = split_lines(mask_rust("\n".join(self.lines)))
        return self.rust_lines[number - 1] if 1 <= number <= len(self.rust_lines) else ""


@dataclass
class Grants:
    prefix: str
    entries: dict[str, str]

    @classmethod
    def load(cls, config: Path, profile: str | None = None, root: Path | None = None) -> "Grants":
        data = tomllib.loads(config.read_text(encoding="utf-8"))
        profile = profile if profile is not None else data.get("default_permissions", "zircon-strict")
        if not isinstance(profile, str) or not profile or profile != profile.strip():
            raise ValueError(f"invalid permission selection {profile!r}")
        prefix = (root or config.parent.parent).resolve().as_posix().rstrip("/")
        if profile == ":danger-full-access":
            return cls(prefix, {"": "write"})
        permissions = data.get("permissions", {})
        selected = permissions.get(profile, {}) if isinstance(permissions, dict) else {}
        raw = selected.get("filesystem") if isinstance(selected, dict) else None
        if not isinstance(raw, dict) or not raw:
            raise ValueError(f"missing filesystem grants for permission profile {profile!r}")
        fold = str.casefold if os.name == "nt" else lambda value: value
        entries = {}
        for key, mode in raw.items():
            if not isinstance(mode, str):
                raise ValueError(f"invalid permission mode for {key!r}")
            if key.startswith(":"):
                continue
            absolute = key.replace("\\", "/").rstrip("/")
            if fold(absolute) == fold(prefix):
                entries[""] = mode
            elif fold(absolute).startswith(fold(prefix) + "/"):
                relative = normalize_rel(absolute[len(prefix) + 1:])
                entries[fold(relative)] = mode
        return cls(prefix, entries)

    def mode(self, rel: str) -> str:
        target = normalize_rel(rel)
        if os.name == "nt":
            target = target.casefold()
        best: tuple[str, str] | None = None
        for key, mode in self.entries.items():
            if (not key or target == key or target.startswith(key + "/")) and (best is None or len(key) > len(best[0])):
                best = (key, mode)
        return best[1] if best else "none"


@dataclass
class Context:
    root: Path | None
    mode: str
    changes: list[FileChange]
    grants: Grants | None = None
    declared_paths: list[str] | None = None
    abort_profiles: list[str] = field(default_factory=list)
    text_cache: dict[Path, str | None] = field(default_factory=dict)
    path_index: dict[Path, "PathIndex"] = field(default_factory=dict)
    by_path: dict[str, FileChange] = field(default_factory=dict)
    manifest_cache: dict[Path, dict | None] = field(default_factory=dict)
    contributor: bool = False

    def __post_init__(self) -> None:
        if self.root is not None:
            self.root = self.root.resolve()
        for change in self.changes:
            change.old_path = normalize_rel(change.old_path) if change.old_path is not None else None
            change.new_path = normalize_rel(change.new_path) if change.new_path is not None else None
            if change.path == ".":
                raise ValueError("a file change cannot name the repository directory")
            if change.path in self.by_path:
                raise ValueError(f"duplicate file change: {change.path}")
            self.by_path.setdefault(change.path, change)

    def read(self, path: Path) -> str | None:
        path = normalized(path)
        if self.root is not None and not path.is_relative_to(self.root):
            raise ValueError(f"dependency leaves the repository: {path}")
        if path not in self.text_cache:
            if self.root is not None:
                physical_path(self.root, path.relative_to(self.root).as_posix())
            try:
                self.text_cache[path] = path.read_text(encoding="utf-8")
            except FileNotFoundError:
                self.text_cache[path] = None
        return self.text_cache[path]

    def changed(self, rel: str) -> FileChange | None:
        return self.by_path.get(rel)

    def manifest(self, directory: Path) -> dict | None:
        if directory not in self.manifest_cache:
            text = self.read(directory / "Cargo.toml")
            self.manifest_cache[directory] = tomllib.loads(text) if text is not None else None
        return self.manifest_cache[directory]


def is_test_path(rel: str) -> bool:
    parts = PurePosixPath(rel).parts
    return any(part in TEST_DIRS for part in parts[:-1]) or bool(TEST_FILE.search(parts[-1] if parts else ""))


def is_comment_or_blank(suffix: str, text: str) -> bool:
    stripped = text.strip()
    if not stripped or suffix in DOC_EXT:
        return True
    if suffix in SLASH_COMMENT_EXT:
        return stripped.startswith(("//", "/*", "*/")) or stripped == "*" or stripped.startswith("* ")
    if suffix in HASH_COMMENT_EXT:
        return stripped.startswith("#") and not stripped.startswith("#[")
    return False


def code_part(suffix: str, text: str) -> str:
    masked = STRING_LITERAL.sub('""', text)
    if suffix in SLASH_COMMENT_EXT:
        index = masked.find("//")
        return masked if index < 0 else masked[:index]
    if suffix in HASH_COMMENT_EXT:
        index = masked.find("#")
        while index >= 0 and masked[index:index + 2] == "#[":
            index = masked.find("#", index + 1)
        return masked if index < 0 else masked[:index]
    return masked


def rust_lex(text: str):
    """Yield (kind, value, start, end), retaining offsets and skipping comments."""
    position = 0
    while position < len(text):
        start = position
        if text[position].isspace():
            position += 1
            continue
        if text.startswith("//", position):
            end = text.find("\n", position)
            position = len(text) if end < 0 else end
            continue
        if text.startswith("/*", position):
            depth, position = 1, position + 2
            while position < len(text) and depth:
                if text.startswith("/*", position):
                    depth, position = depth + 1, position + 2
                elif text.startswith("*/", position):
                    depth, position = depth - 1, position + 2
                else:
                    position += 1
            continue
        raw = RUST_RAW.match(text, position)
        if raw:
            content = raw.end()
            end = text.find('"' + raw.group(1), content)
            position = len(text) if end < 0 else end + 1 + len(raw.group(1))
            yield "string", text[content:end if end >= 0 else len(text)], start, position
            continue
        quoted = RUST_QUOTED.match(text, position)
        if quoted:
            position = quoted.end()
            content = position
            while position < len(text):
                if text[position] == "\\":
                    position += 2
                elif text[position] == '"':
                    break
                else:
                    position += 1
            value = text[content:position].replace('\\"', '"').replace("\\\\", "\\")
            position = min(len(text), position + 1)
            yield "string", value, start, position
            continue
        char = RUST_CHAR.match(text, position)
        if char:
            position = char.end()
            yield "string", "", start, position
            continue
        word = RUST_WORD.match(text, position)
        if word:
            position = word.end()
            yield "word", word.group().removeprefix("r#"), start, position
        else:
            position += 1
            yield "punct", text[start:position], start, position


def mask_rust(text: str, keep_strings: bool = False) -> str:
    result = ["\n" if char == "\n" else " " for char in text]
    for kind, _, start, end in rust_lex(text):
        if kind != "string" or keep_strings:
            result[start:end] = text[start:end]
    return "".join(result)


def strip_rust_comments(text: str) -> str:
    return mask_rust(text, keep_strings=True)


def unquote_git_path(token: str) -> str:
    if len(token) >= 2 and token[0] == token[-1] == '"':
        try:
            return token[1:-1].encode("latin-1", "backslashreplace").decode("unicode_escape").encode("latin-1").decode("utf-8")
        except (UnicodeError, ValueError):
            return token[1:-1]
    return token


def diff_path(token: str) -> str | None:
    token = unquote_git_path(token.split("\t", 1)[0].strip())
    if token == "/dev/null":
        return None
    return normalize_rel(token[2:] if token[:2] in ("a/", "b/") else token)


def split_git_header(rest: str) -> tuple[str | None, str | None]:
    if rest.startswith('"'):
        match = re.match(r'("(?:[^"\\]|\\.)*")\s+(.+)$', rest)
        return (diff_path(match.group(1)), diff_path(match.group(2))) if match else (None, None)
    for found in re.finditer(r" b/", rest):
        left, right = rest[: found.start()], rest[found.start() + 1:]
        if left[2:] == right[2:]:
            return diff_path(left), diff_path(right)
    left, _, right = rest.partition(" b/")
    return (diff_path(left), diff_path("b/" + right)) if right else (None, None)


def parse_unified_diff(text: str) -> list[FileChange]:
    changes: list[FileChange] = []
    current: FileChange | None = None
    old_left = new_left = old_no = new_no = 0
    for raw in split_lines(text):
        if current is not None and (old_left > 0 or new_left > 0):
            tag, body = raw[:1], raw[1:]
            if tag == " " and old_left > 0 and new_left > 0:
                current.context[new_no] = body
                old_no, new_no, old_left, new_left = old_no + 1, new_no + 1, old_left - 1, new_left - 1
            elif tag == "+" and new_left > 0:
                current.added[new_no] = body
                new_no, new_left = new_no + 1, new_left - 1
            elif tag == "-" and old_left > 0:
                current.removed.append((old_no, body))
                old_no, old_left = old_no + 1, old_left - 1
            elif raw != "\\ No newline at end of file":
                raise ValueError(f"malformed hunk in {current.path!r} near {raw[:60]!r}")
            continue
        if raw.startswith("diff --git "):
            current = FileChange(*split_git_header(raw[len("diff --git "):]))
            if current.old_path is None or current.new_path is None:
                raise ValueError("malformed Git file header")
            changes.append(current)
        elif raw.startswith("--- ") and (current is None or current.saw_old_header or current.added or current.removed):
            current = FileChange(diff_path(raw[4:]), None, saw_old_header=True)
            changes.append(current)
        elif raw.startswith("--- ") and current is not None:
            current.old_path, current.saw_old_header = diff_path(raw[4:]), True
        elif raw.startswith("+++ ") and current is not None:
            current.new_path = diff_path(raw[4:])
            current.saw_new_header = True
        elif raw.startswith("@@") and current is not None:
            match = HUNK.match(raw)
            if not match:
                raise ValueError(f"malformed hunk header {raw[:60]!r}")
            if not current.saw_old_header or not current.saw_new_header:
                raise ValueError("hunk requires both --- and +++ file headers")
            old_no, old_left = int(match.group(1)), int(match.group(2) or 1)
            new_no, new_left = int(match.group(3)), int(match.group(4) or 1)
            if (old_left and not old_no) or (new_left and not new_no) or not (old_left or new_left):
                raise ValueError("invalid hunk line range")
            old_start, new_start = old_no + (old_left == 0), new_no + (new_left == 0)
            if old_start < current.last_old_end or new_start < current.last_new_end:
                raise ValueError("overlapping or out-of-order hunks")
            current.last_old_end, current.last_new_end = old_start + old_left, new_start + new_left
        elif current is not None and raw.startswith("new file mode"):
            current.created = True
        elif current is not None and raw.startswith("deleted file mode"):
            current.deleted = True
        elif current is not None and raw.startswith("rename from "):
            current.old_path = normalize_rel(unquote_git_path(raw[len("rename from "):]))
        elif current is not None and raw.startswith("rename to "):
            current.new_path = normalize_rel(unquote_git_path(raw[len("rename to "):]))
        elif current is not None and (raw.startswith("Binary files ") or raw == "GIT binary patch"):
            current.binary = True
    if old_left > 0 or new_left > 0:
        raise ValueError("patch ends inside a hunk")
    for change in changes:
        change.created |= change.saw_old_header and change.old_path is None
        change.deleted |= change.saw_new_header and change.new_path is None
        if change.created:
            change.old_path = None
        if change.deleted:
            change.new_path = None
    return [change for change in changes if change.path]


def split_lines(text: str) -> list[str]:
    lines = text.replace("\r\n", "\n").split("\n")
    return lines[:-1] if lines and lines[-1] == "" else lines


def attach_worktree(context: Context) -> list[Finding]:
    notes = []
    for change in context.changes:
        if context.root is None:
            continue
        physical_path(context.root, change.path)
        if change.new_path is None:
            if (context.root / change.path).exists():
                notes.append(Finding(ERROR, "input.worktree-mismatch", change.path, None,
                                     "deleted file is still present; use the final patched checkout"))
            continue
        if change.binary:
            if not (context.root / change.path).is_file():
                notes.append(Finding(ERROR, "input.worktree-mismatch", change.path, None,
                                     "binary file is missing from the final patched checkout"))
            continue
        text = context.read(context.root / change.new_path)
        if text is None:
            notes.append(Finding(ERROR, "input.worktree-mismatch", change.path, None,
                                 "file is missing or unreadable; use the final patched checkout"))
            continue
        lines = split_lines(text)
        expected = {**change.context, **change.added}
        matches = all(1 <= number <= len(lines) and lines[number - 1] == body for number, body in expected.items())
        if change.created:
            matches = matches and lines == [change.added[number] for number in sorted(change.added)]
        if matches:
            change.lines = lines
        else:
            notes.append(Finding(ERROR, "input.worktree-mismatch", change.path, None,
                                 "working tree differs from the patch; check the exact final patched checkout"))
    return notes


def normalize_rel(value: str) -> str:
    if not value or "\x00" in value or "\n" in value or "\r" in value:
        raise ValueError(f"invalid repository path: {value!r}")
    rel = PurePosixPath(value.replace("\\", "/"))
    if rel.is_absolute() or ":" in str(rel) or ".." in rel.parts:
        raise ValueError(f"path must be repository-relative without '..' or ':': {value!r}")
    if os.name == "nt" and any(part.endswith((" ", ".")) for part in rel.parts):
        raise ValueError(f"ambiguous Windows repository path: {value!r}")
    if os.name == "nt" and any(re.fullmatch(r"(?i)(?:CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(?:\..*)?", part) for part in rel.parts):
        raise ValueError(f"Windows device name is not a repository file: {value!r}")
    return str(rel).strip("/") or "."


def physical_path(root: Path, rel: str) -> Path:
    """Do not follow aliases when examining a patch or scan input."""
    path = root / normalize_rel(rel)
    current = path
    while current != root:
        if current.is_symlink() or (hasattr(current, "is_junction") and current.is_junction()):
            raise ValueError(f"repository input uses a path alias: {rel!r}")
        current = current.parent
    if not path.resolve().is_relative_to(root):
        raise ValueError(f"repository input escapes the checkout: {rel!r}")
    return path


def under(rel: str, parent: str) -> bool:
    if os.name == "nt":
        rel, parent = rel.casefold(), parent.casefold()
    return parent in ("", ".") or rel == parent or rel.startswith(parent.rstrip("/") + "/")


# --- scope -----------------------------------------------------------------

def rule_scope(context: Context) -> list[Finding]:
    findings = []
    if context.mode != "patch":
        return findings
    for change in context.changes:
        for rel in {path for path in (change.old_path, change.new_path) if path}:
            if context.declared_paths is not None and not any(under(rel, parent) for parent in context.declared_paths):
                findings.append(Finding(ERROR, "scope.unlisted-path", rel, None,
                                        "path is outside the declared --path allowlist"))
            if context.grants is not None and context.grants.mode(rel) != "write":
                findings.append(Finding(ERROR, "scope.write-grant", rel, None,
                                        f"no write grant in the active permission profile (mode: {context.grants.mode(rel)})"))
            parts = PurePosixPath(rel).parts
            cargo_target = any(
                part == "target" and (index == 0 or (
                    context.root is not None and context.manifest(context.root.joinpath(*parts[:index])) is not None
                )) for index, part in enumerate(parts[:-1])
            )
            if change.new_path == rel and (BACKUP_NAME.search(parts[-1]) or BACKUP_DIRS & set(parts[:-1]) or cargo_target):
                findings.append(Finding(ERROR, "scope.backup-artifact", rel, None,
                                        "backup, scratch or build-output file must not enter a handoff"))
            if rel.startswith(".opencode/skills/"):
                findings.append(Finding(WARN, "scope.generated-adapter", rel, None,
                                        "generated adapter: edit .codex/skills and run the synchronizer"))
            if parts[-1] in LOCKFILES:
                findings.append(Finding(WARN, "scope.lockfile", rel, None,
                                        "lockfile change: record the dependency reason in the manifest"))
            if rel.startswith(CONTROL_PLANE) or parts[-1] == "AGENTS.md":
                findings.append(Finding(WARN, "scope.control-plane", rel, None,
                                        "control-plane file: needs explicit user authorization recorded in the manifest"))
    return findings


def rule_conflict_markers(context: Context) -> list[Finding]:
    return [
        Finding(ERROR, "diff.conflict-marker", change.path, number, "unresolved merge conflict marker")
        for change in context.changes
        for number, text in sorted(change.added.items())
        if CONFLICT.match(text) and not (change.suffix == ".rst" and re.fullmatch(r"=+", text))
    ]


# --- markers ---------------------------------------------------------------

def marker_ids(lines) -> list[str]:
    return [match.group(1) for text in lines if MARKER_LINE.search(text) for match in [MARKER_ID.search(text)] if match]


def marker_sets(changes: list[FileChange]) -> tuple[dict[str, FileChange], set[str]]:
    code = [change for change in changes if change.suffix not in DOC_EXT]
    removed = {mid: change for change in code for mid in marker_ids(text for _, text in change.removed)}
    added = {mid for change in code for mid in marker_ids(change.added.values())}
    return removed, added


def rule_markers(context: Context) -> list[Finding]:
    findings = []
    for change in context.changes:
        if change.suffix in DOC_EXT:
            continue
        for number, text in sorted(change.added.items()):
            if MARKER_LINE.search(text) and not MARKER_WELL_FORMED.search(text):
                findings.append(Finding(ERROR, "marker.format", change.path, number,
                                        "use `BUG: [CR-<AREA>-<NNNN>] <defect>` in a code comment"))
    if context.mode != "patch":
        return findings
    removed, added = marker_sets(context.changes)
    closed = sorted(set(removed) - added)
    code_changes = [
        change for change in context.changes
        if any(not is_comment_or_blank(change.suffix, text) for text in change.added.values())
        or any(not is_comment_or_blank(change.suffix, text) for _, text in change.removed)
    ]
    test_touched = any(
        any(not is_comment_or_blank(change.suffix, text) for text in change.added.values())
        and (is_test_path(change.path) or any(TEST_FN.match(text) for text in change.added.values()))
        for change in context.changes
    )
    for mid in closed:
        owner = removed[mid]
        if not code_changes:
            findings.append(Finding(ERROR, "marker.removed-without-fix", owner.path, None,
                                    f"{mid} removed by a comment-only patch; land the fix or keep the marker"))
            continue
        if all(owner is not change for change in code_changes):
            findings.append(Finding(WARN, "marker.fix-elsewhere", owner.path, None,
                                    f"{mid} removed but this file has no code change; name the fixing file in the manifest"))
        if not test_touched:
            findings.append(Finding(WARN, "marker.closed-without-test", owner.path, None,
                                    f"{mid} closed without an added or changed test; add a regression test"))
    return findings


# --- Rust modules ----------------------------------------------------------

def crate_dir(context: Context, path: Path) -> Path | None:
    for current in path.parents:
        manifest = context.manifest(current)
        if manifest is not None and isinstance(manifest.get("package"), dict):
            return current
        if current == context.root:
            return None
    return None


def crate_roots(context: Context, crate: Path) -> set[Path]:
    manifest = context.manifest(crate) or {}
    package = manifest.get("package", {})
    roots = set()

    def add(value: str):
        target = normalized(crate / value)
        if target.is_file():
            roots.add(target)

    lib = manifest.get("lib")
    if isinstance(lib, dict):
        add(lib.get("path", "src/lib.rs"))
    elif package.get("autolib", True):
        add("src/lib.rs")
    build = package.get("build", "build.rs")
    if isinstance(build, str):
        add(build)
    manual = any(name in manifest for name in ("lib", "bin", "test", "bench", "example"))
    automatic = not (str(package.get("edition", "2015")) == "2015" and manual)
    for kind, directory, flag in (
        ("bin", "src/bin", "autobins"), ("test", "tests", "autotests"),
        ("bench", "benches", "autobenches"), ("example", "examples", "autoexamples"),
    ):
        targets = manifest.get(kind, [])
        for target in targets:
            if "path" in target:
                add(target["path"])
            elif "name" in target:
                name = target["name"]
                if kind == "bin" and name == package.get("name"):
                    add("src/main.rs")
                add(f"{directory}/{name}.rs")
                add(f"{directory}/{name}/main.rs")
        if package.get(flag, automatic):
            if kind == "bin":
                add("src/main.rs")
            roots.update(path for path in (crate / directory).glob("*.rs") if path.is_file())
            roots.update(path for path in (crate / directory).glob("*/main.rs") if path.is_file())
    return roots


def normalized(path: Path) -> Path:
    return Path(os.path.normpath(path))


@dataclass
class PathIndex:
    targets: set[Path] = field(default_factory=set)
    data_literals: set[str] = field(default_factory=set)


def rust_module_edges(context: Context, path: Path, base: Path):
    """Follow literal modules/includes; leave macro expansion and cfg to rustc."""
    tokens = list(rust_lex(context.read(path) or ""))
    values = [value for _, value, _, _ in tokens]
    stack = [(base, path.parent)]
    pending_paths: list[str] = []
    conditional_path = False
    position = 0
    while position < len(tokens):
        value = values[position]
        directory, attr_directory = stack[-1]
        if value == "#" and values[position + 1:position + 2] == ["["]:
            end, depth = position + 2, 1
            while end < len(tokens) and depth:
                depth += {"[": 1, "]": -1}.get(values[end], 0)
                end += 1
            attr = tokens[position + 2:end - 1]
            for index in range(len(attr) - 2):
                if attr[index][1] == "path" and attr[index + 1][1] == "=" and attr[index + 2][0] == "string":
                    pending_paths.append(attr[index + 2][1])
                    conditional_path |= any(token[1] == "cfg_attr" for token in attr)
            position = end
            continue
        if value == "mod" and tokens[position][0] == "word" and position + 2 < len(tokens) and tokens[position + 1][0] == "word":
            name, terminator = values[position + 1:position + 3]
            if terminator == ";":
                candidates = [(normalized(attr_directory / item), True) for item in pending_paths]
                if not pending_paths or conditional_path:
                    candidates += [(directory / f"{name}.rs", False), (directory / name / "mod.rs", False)]
                for target, custom in candidates:
                    if target.is_file():
                        child_base = target.parent if custom or target.name == "mod.rs" else target.parent / target.stem
                        yield target, child_base
                pending_paths, conditional_path = [], False
                position += 3
                continue
            if terminator == "{":
                child = normalized(attr_directory / pending_paths[0]) if pending_paths else directory / name
                stack.append((child, child))
                pending_paths, conditional_path = [], False
                position += 3
                continue
        if value == "include" and values[position + 1:position + 3] == ["!", "("]:
            if position + 3 < len(tokens) and tokens[position + 3][0] == "string":
                target = normalized(path.parent / values[position + 3])
                if target.is_file():
                    yield target, target.parent
        # Macro definitions and invocations are token trees, not module edges.
        if value == "!" or (value == "macro_rules" and values[position + 1:position + 2] == ["!"]):
            end = position + 1
            while end < len(tokens) and values[end] not in ("(", "[", "{", ";"):
                end += 1
            if end < len(tokens) and values[end] in ("(", "[", "{"):
                opening = values[end]
                closing = {"(": ")", "[": "]", "{": "}"}[opening]
                depth, end = 1, end + 1
                while end < len(tokens) and depth:
                    depth += (values[end] == opening) - (values[end] == closing)
                    end += 1
                position = end
                pending_paths, conditional_path = [], False
                continue
        if value == "{":
            stack.append(stack[-1])
            pending_paths, conditional_path = [], False
        elif value == "}" and len(stack) > 1:
            stack.pop()
            pending_paths, conditional_path = [], False
        elif value == ";":
            pending_paths, conditional_path = [], False
        position += 1


def crate_path_index(context: Context, crate: Path) -> PathIndex:
    if crate not in context.path_index:
        index = PathIndex()
        pending = [(path, path.parent) for path in crate_roots(context, crate)]
        visited = set()
        while pending:
            path, base = pending.pop()
            key = (path, base)
            if key in visited:
                continue
            visited.add(key)
            text = context.read(path)
            if text is None:
                continue
            index.targets.add(path)
            index.data_literals.update(value for kind, value, _, _ in rust_lex(text) if kind == "string" and value.endswith(".rs"))
            pending.extend(rust_module_edges(context, path, base))
        context.path_index[crate] = index
    return context.path_index[crate]


def referenced_as_data(context: Context, path: Path) -> bool:
    crate = crate_dir(context, path)
    if crate is None:
        return False
    rel = path.relative_to(crate).as_posix()
    suffixes = {rel, rel.split("/", 1)[-1]}
    return any(literal.replace("\\", "/").endswith(suffix) for literal in crate_path_index(context, crate).data_literals
               for suffix in suffixes if "/" in suffix)


def module_declared(context: Context, path: Path) -> bool:
    crate = crate_dir(context, path)
    return crate is not None and normalized(path) in crate_path_index(context, crate).targets


def rule_unreachable_module(context: Context) -> list[Finding]:
    findings = []
    if context.root is None:
        return findings
    for change in context.changes:
        if change.suffix != ".rs" and PurePosixPath(change.old_path or "").suffix != ".rs":
            continue
        path = normalized(context.root / change.path)
        if change.new_path is not None and (change.created or change.old_path != change.new_path or context.mode == "scan") and not module_declared(context, path):
            if referenced_as_data(context, path):
                findings.append(Finding(WARN, "rust.uncompiled-source", change.path, None,
                                        "file is never compiled; it is only named by a path string (fixture data?)"))
            else:
                findings.append(Finding(ERROR, "rust.unreachable-module", change.path, None,
                                        "file is not a crate root and no `mod` or #[path] declaration reaches it"))
        findings.extend(removed_module_declarations(context, change, path))
    return findings


def removed_module_declarations(context: Context, change: FileChange, declaring: Path) -> list[Finding]:
    findings = []
    if change.old_path is None or not (change.removed or change.old_path != change.new_path):
        return findings
    old_path = normalized(context.root / change.old_path)
    old_lines = [text for number, text in enumerate(change.lines or [], start=1) if number not in change.added]
    for number, text in sorted(change.removed):
        old_lines.insert(number - 1, text)
    # Reconstruct only this file in memory; never make a source backup.
    before = Context(context.root, "scan", [])
    before.text_cache = dict(context.text_cache)
    before.text_cache[old_path] = "\n".join(old_lines)
    crate = crate_dir(context, old_path)
    flat = old_path.name == "mod.rs" or (crate is not None and old_path in crate_roots(context, crate))
    if crate is not None:
        rel = old_path.relative_to(crate).as_posix()
        flat |= rel in ("src/lib.rs", "src/main.rs", "build.rs")
        manifest = context.manifest(crate) or {}
        targets = [manifest.get("lib", {})] + [target for kind in ("bin", "test", "bench", "example") for target in manifest.get(kind, [])]
        flat |= any(target.get("path") == rel for target in targets)
    old_base = old_path.parent if flat else old_path.parent / old_path.stem
    old_children = {child for child, _ in rust_module_edges(before, old_path, old_base)}
    new_flat = declaring.name == "mod.rs" or (crate is not None and declaring in crate_roots(context, crate))
    new_base = declaring.parent if new_flat else declaring.parent / declaring.stem
    new_children = {child for child, _ in rust_module_edges(context, declaring, new_base)} if change.new_path else set()
    for child in sorted(old_children - new_children):
        if child.is_file() and not module_declared(context, child):
            number = change.removed[0][0] if change.removed else None
            findings.append(Finding(ERROR, "rust.unreachable-module", change.path, number,
                                    f"removed module edge orphans {child.relative_to(context.root).as_posix()}"))
    return findings


# --- Rust safety -----------------------------------------------------------

def has_safety_note(change: FileChange, number: int) -> bool:
    match = SAFETY_NOTE.search(change.line(number) or "")
    if match and in_comment(change, number, match):
        return True
    for index in range(number - 1, max(0, number - 16), -1):
        text = change.line(index)
        if text is None:
            return False
        stripped = text.strip()
        match = SAFETY_NOTE.search(text)
        if match and in_comment(change, index, match):
            return True
        if is_comment_or_blank(".rs", stripped) or stripped.startswith("#["):
            continue
        if stripped.endswith((";", "{", "}")):
            return False
    return False


def has_safety_doc(change: FileChange, number: int) -> bool:
    for index in range(number - 1, max(0, number - 60), -1):
        text = change.line(index)
        if text is None:
            return False
        stripped = text.strip()
        if SAFETY_DOC.search(text):
            return True
        if not (stripped.startswith(("///", "#[", "//", "/*", "*")) or not stripped):
            return False
    return False


def rule_unsafe(context: Context) -> list[Finding]:
    findings = []
    for change in context.changes:
        if change.suffix != ".rs":
            continue
        for number, text in sorted(change.added.items()):
            code = change.code_line(number)
            if UNSAFE_USE.search(code) and not has_safety_note(change, number):
                findings.append(Finding(ERROR, "rust.unsafe-without-safety", change.path, number,
                                        "unsafe block or impl needs a `// SAFETY:` comment stating the upheld invariant"))
            if PUB_UNSAFE_FN.match(code) and not has_safety_doc(change, number):
                findings.append(Finding(WARN, "rust.unsafe-fn-safety-doc", change.path, number,
                                        "public unsafe fn needs a `/// # Safety` section listing caller obligations"))
    return findings


def fn_parameters(change: FileChange, number: int, name_end: int) -> tuple[str, int] | None:
    """Return parameters and their last line; None for bodiless declarations."""
    pieces = []
    for index in range(number, number + 40):
        text = change.line(index)
        if text is None:
            break
        pieces.append(change.code_line(index))
    joined = "\n".join(pieces)[name_end:]
    position = 0
    while position < len(joined) and joined[position].isspace():
        position += 1
    if position < len(joined) and joined[position] == "<":
        depth = 0
        while position < len(joined):
            char = joined[position]
            if char == "<":
                depth += 1
            elif char == ">" and joined[position - 1] != "-":
                depth -= 1
                if depth == 0:
                    position += 1
                    break
            position += 1
    start = joined.find("(", position)
    if start < 0:
        return None
    depth = 0
    for index in range(start, len(joined)):
        depth += {"(": 1, ")": -1}.get(joined[index], 0)
        if depth == 0:
            body = re.search(r"[{;]", joined[index:])
            return (joined[start + 1:index], number + joined[:index + 1].count("\n")) if body is None or body.group() == "{" else None
    return None


def without_fn_pointer_types(parameters: str) -> str:
    """Blank `fn(..)`/`Fn(..)` argument lists so pointer-typed callback signatures do not count."""
    result, position = [], 0
    for match in FN_POINTER_TYPE.finditer(parameters):
        if match.start() < position:
            continue
        depth, index = 0, match.end() - 1
        while index < len(parameters):
            depth += {"(": 1, ")": -1}.get(parameters[index], 0)
            if depth == 0:
                break
            index += 1
        result.append(parameters[position:match.start()])
        position = index + 1
        returned = re.match(r"\s*->\s*", parameters[position:])
        if returned:
            position += returned.end()
            depth = 0
            while position < len(parameters):
                char = parameters[position]
                if char == "<":
                    depth += 1
                elif char == ">":
                    if not depth:
                        break
                    depth -= 1
                elif char == "," and not depth:
                    break
                position += 1
    result.append(parameters[position:])
    return "".join(result)


def rule_safe_fn_raw_pointer(context: Context) -> list[Finding]:
    findings = []
    for change in context.changes:
        if change.suffix != ".rs" or is_test_path(change.path):
            continue
        candidates = enumerate(change.lines, start=1) if change.lines is not None else sorted(change.added.items())
        for number, text in candidates:
            match = PUB_FN.match(change.code_line(number))
            if not match:
                continue
            parsed = fn_parameters(change, number, match.end(1))
            if parsed is None:
                continue
            parameters, end = parsed
            changed_lines = [line for line in change.added if number <= line <= end]
            if changed_lines and RAW_POINTER.search(without_fn_pointer_types(parameters)):
                findings.append(Finding(ERROR, "rust.safe-fn-raw-pointer", change.path, number,
                                        f"safe `pub fn {match.group(1)}` takes a raw pointer; mark it unsafe with `# Safety` or take a reference/handle"))
    return findings


def rule_catch_unwind(context: Context) -> list[Finding]:
    if not context.abort_profiles:
        return []
    profiles = ", ".join(context.abort_profiles)
    return [
        Finding(WARN, "rust.catch-unwind-under-abort", change.path, number,
                f"profile(s) {profiles} use panic = \"abort\"; catch_unwind cannot contain panics there")
        for change in context.changes
        if change.suffix == ".rs" and not is_test_path(change.path)
        for number, text in sorted(change.added.items())
        if CATCH_UNWIND.search(change.code_line(number))
    ]


def rule_serde_bypass(context: Context) -> list[Finding]:
    findings = []
    for change in context.changes:
        if change.suffix != ".rs" or is_test_path(change.path) or change.lines is None:
            continue
        body = "\n".join(change.lines)
        for number, text in sorted(change.added.items()):
            if not DERIVE_DESERIALIZE.search(code_part(".rs", text)):
                continue
            attributes, name = [], None
            for index in range(number, number + 12):
                line = change.line(index) or ""
                item = ITEM_NAME.match(line)
                if item:
                    name = item.group(1)
                    break
                attributes.append(line)
            if name and not SERDE_SAFE.search("\n".join(attributes)) and validating_constructor(body, name):
                findings.append(Finding(WARN, "rust.serde-bypass", change.path, number,
                                        f"derived Deserialize skips the validating constructor of {name}; use `#[serde(try_from = ...)]`"))
    return findings


def validating_constructor(body: str, name: str) -> bool:
    constructor = rf"\bfn\s+(?:new|try_new|parse|from_str|validated|checked)\s*\([^)]*\)\s*->\s*(?:Result|Option)<\s*(?:Self|{name})\b"
    return any(
        re.search(constructor, block.group(1))
        for block in re.finditer(rf"(?m)^impl(?:<[^>]*>)?\s+{name}\b[^{{;]*\{{(.*?)^\}}", body, re.S)
    )


# --- filesystem and storage -----------------------------------------------

def rule_unbounded_delete(context: Context) -> list[Finding]:
    findings = []
    for change in context.changes:
        pattern = DELETE_CALL.get(change.suffix)
        if pattern is None or is_test_path(change.path):
            continue
        for number, text in sorted(change.added.items()):
            if not pattern.search(code_part(change.suffix, text)):
                continue
            window = [change.line(index) or "" for index in range(max(1, number - 25), number + 1)]
            if not any(CONTAINMENT.search(code_part(change.suffix, line)) for line in window):
                findings.append(Finding(ERROR, "fs.unbounded-delete", change.path, number,
                                        "recursive delete without a visible root-containment check in the preceding 25 lines"))
    return findings


def rule_storage(context: Context) -> list[Finding]:
    findings = []
    for change in context.changes:
        if change.suffix in DOC_EXT | {".lock"} or is_test_path(change.path):
            continue
        for number, text in sorted(change.added.items()):
            if is_comment_or_blank(change.suffix, text):
                continue
            key = output_key(text)
            if not key and REJECTION_CONTEXT.search(text):
                continue
            if STORAGE_FORBIDDEN.search(text):
                findings.append(Finding(ERROR, "storage.output-root", change.path, number,
                                        "build output must use the approved drive-root cargo-targets directories"))
                continue
            if key and forbidden_output_location(text[key.end():]):
                findings.append(Finding(ERROR, "storage.output-root", change.path, number,
                                        "Cargo output redirected to a literal location outside D:/E:/F:\\cargo-targets"))
    return findings


def output_key(text: str) -> re.Match | None:
    strings = list(SOURCE_STRING.finditer(text))
    for match in CARGO_OUTPUT_KEY.finditer(text):
        quoted = next((item for item in strings if item.start() <= match.start() < item.end()), None)
        if quoted is not None:
            value = quoted.group()[1:-1]
            exact_key = re.fullmatch(r"(?i)CARGO_(?:TARGET_DIR|BUILD_TARGET_DIR|BUILD_BUILD_DIR)|target[-_]dir|--target-dir|-TargetDir", value)
            command = re.match(r"(?i)^\s*cargo(?:\.exe)?\s", value)
            if not exact_key and not command:
                continue
        return match
    return None


def forbidden_output_location(rest: str) -> bool:
    values = [(match.group(2), True) for match in QUOTED.finditer(rest)]
    unquoted = UNQUOTED_VALUE.match(rest.lstrip(" \t:='\"](),"))
    if unquoted:
        values.append((unquoted.group(1), False))
    for value, quoted in values:
        value = value.strip()
        if value and not value.startswith(("$", "%", "{")):
            if LITERAL_LOCATION.match(value) or (quoted and re.match(r"^[\w.-]+(?:[\\/]|$)", value)) or "/" in value or "\\" in value:
                normalized_value = value.replace("\\", "/")
                approved = re.fullmatch(r"(?i)[def]:/+cargo-targets(?:/[^/:]+)*/*", normalized_value)
                if not approved or any(part in (".", "..") for part in normalized_value.split("/")):
                    return True
    return False


def rule_test_weakened(context: Context) -> list[Finding]:
    findings = []
    if context.mode != "patch":
        return findings
    added = sum(1 for change in context.changes for text in change.added.values() if TEST_FN.match(text))
    removed = sum(1 for change in context.changes for _, text in change.removed if TEST_FN.match(text))
    if removed > added:
        findings.append(Finding(WARN, "test.weakened", None, None,
                                f"patch removes {removed} test functions and adds {added}; justify in the manifest"))
    for change in context.changes:
        for number, text in sorted(change.added.items()):
            if TEST_DISABLE.search(text):
                findings.append(Finding(WARN, "test.weakened", change.path, number,
                                        "test disabled or skipped; record why and the follow-up marker"))
    return findings


RULES = (
    rule_scope, rule_conflict_markers, rule_markers, rule_unreachable_module, rule_unsafe,
    rule_safe_fn_raw_pointer, rule_catch_unwind, rule_serde_bypass, rule_unbounded_delete,
    rule_storage, rule_test_weakened,
)


def in_comment(change: FileChange, number: int, match: re.Match) -> bool:
    text = change.line(number) or ""
    if change.suffix in SLASH_COMMENT_EXT:
        if change.lines is not None:
            if change.comment_lines is None:
                change.comment_lines = split_lines(strip_rust_comments("\n".join(change.lines)))
            line = change.comment_lines[number - 1]
        else:
            line = strip_rust_comments(text)
        return not line[match.start():match.end()].strip()
    if change.suffix in HASH_COMMENT_EXT:
        masked = STRING_LITERAL.sub(lambda item: " " * len(item.group()), text)
        index = masked.find("#")
        while index >= 0 and masked[index:index + 2] == "#[":
            index = masked.find("#", index + 1)
        return index >= 0 and match.start() > index
    return change.suffix in DOC_EXT


def apply_allows(context: Context, findings: list[Finding]) -> None:
    for finding in findings:
        change = context.changed(finding.path) if finding.path else None
        if change is None or not finding.suppressible:
            continue
        numbers = range(max(1, finding.line - 3), finding.line + 1) if finding.line else range(1, 41)
        for number in numbers:
            match = ALLOW.search(change.line(number) or "")
            if match and match.group(1) == finding.rule and len(match.group(2).strip()) >= 8 and in_comment(change, number, match):
                finding.allowed_by = match.group(2).strip()
                break


def abort_profiles(root: Path | None) -> list[str]:
    if root is None:
        return []
    try:
        text = (root / "Cargo.toml").read_text(encoding="utf-8")
    except OSError:
        return []
    profiles, current = [], None
    for line in text.splitlines():
        header = re.match(r"^\s*\[profile\.([\w-]+)\]", line)
        if header:
            current = header.group(1)
        elif line.strip().startswith("["):
            current = None
        elif current and re.match(r'^\s*panic\s*=\s*"abort"', line):
            profiles.append(current)
    return profiles


def evaluate(context: Context) -> list[Finding]:
    findings = attach_worktree(context) if context.mode == "patch" else []
    if findings:
        return findings + rule_scope(context)
    context.abort_profiles = abort_profiles(context.root)
    for rule in RULES:
        findings.extend(rule(context))
    apply_allows(context, findings)
    return findings


def git(root: Path, *args: str) -> str:
    if shutil.which("git") is None:
        raise ValueError("git is not available; pass --patch instead")
    result = subprocess.run(
        ["git", "--literal-pathspecs", "-C", str(root), "-c", "core.quotepath=false", *args],
        capture_output=True, text=True, encoding="utf-8", errors="replace",
        env={**os.environ, "GIT_OPTIONAL_LOCKS": "0", "GIT_TERMINAL_PROMPT": "0"},
    )
    if result.returncode != 0:
        raise ValueError(f"git {args[0]} failed: {result.stderr.strip()}")
    return result.stdout


def synthesize_addition(rel: str, text: str) -> str:
    rel = normalize_rel(rel)
    lines = split_lines(text)
    old, new = json.dumps("a/" + rel, ensure_ascii=False), json.dumps("b/" + rel, ensure_ascii=False)
    header = f"diff --git {old} {new}\nnew file mode 100644\n--- /dev/null\n+++ {new}\n"
    if not lines:
        return header
    body = "".join(f"+{line}\n" for line in lines)
    tail = "" if text.endswith("\n") else "\\ No newline at end of file\n"
    return f"{header}@@ -0,0 +1,{len(lines)} @@\n{body}{tail}"


def git_patch(root: Path, paths: list[str], base: str) -> tuple[str, str]:
    commit = git(root, "rev-parse", "--verify", "--end-of-options", f"{base}^{{commit}}").strip()
    patch = git(root, "diff", "--no-color", "--no-ext-diff", "--no-textconv", "--find-renames",
                "--src-prefix=a/", "--dst-prefix=b/", "--unified=3", commit, "--", *paths)
    untracked = git(root, "ls-files", "--others", "--exclude-standard", "-z", "--", *paths)
    ignored = set(filter(None, git(root, "ls-files", "--others", "--ignored", "--exclude-standard", "-z", "--", *paths).split("\0")))
    explicit = {path for path in paths if physical_path(root, path).is_file()}
    hidden = ignored - explicit
    if hidden:
        raise ValueError("ignored files inside the selected scope require exact --path entries: " + ", ".join(sorted(hidden)[:5]))
    for rel in sorted(set(filter(None, untracked.split("\0"))) | (ignored & explicit)):
        try:
            text = physical_path(root, rel).read_text(encoding="utf-8")
        except UnicodeDecodeError:
            patch += f"diff --git a/{rel} b/{rel}\nnew file mode 100644\nBinary files /dev/null and b/{rel} differ\n"
            continue
        patch += synthesize_addition(rel, text)
    return patch, commit


def scan_changes(root: Path, paths: list[str]) -> list[FileChange]:
    changes = []
    for rel in paths:
        target = physical_path(root, rel)
        if not target.exists():
            raise ValueError(f"scan path does not exist: {rel}")
        files = [target] if target.is_file() else sorted(
            path for path in target.rglob("*")
            if path.is_file() and path.suffix.lower() in SCAN_EXT and not SKIP_DIRS & set(path.relative_to(root).parts[:-1])
        )
        for path in files:
            physical_path(root, path.relative_to(root).as_posix())
            lines = split_lines(path.read_text(encoding="utf-8"))
            changes.append(FileChange(None, path.relative_to(root).as_posix(),
                                      added=dict(enumerate(lines, start=1)), lines=lines, created=True))
    return changes


def build_receipt(mode: str, findings: list[Finding], context: Context, digest: str | None, base: str | None) -> dict:
    active = [finding for finding in findings if finding.allowed_by is None]
    errors = sum(1 for finding in active if finding.severity == ERROR)
    removed, markers_added = marker_sets(context.changes)
    markers_removed = set(removed) if mode == "patch" else set()
    if mode != "patch":
        markers_added = set()
    return {
        "tool": TOOL,
        "version": VERSION,
        "mode": mode,
        "status": "failed" if errors else "passed",
        "input": {
            "patchDigest": digest, "base": base,
            "files": sorted(change.path for change in context.changes),
            "declaredPaths": context.declared_paths,
            "writeGrantCheck": context.grants is not None,
            "permissionRoot": context.grants.prefix if context.grants is not None else None,
            "contributor": context.contributor,
        },
        "summary": {
            "errors": errors,
            "warnings": sum(1 for finding in active if finding.severity == WARN),
            "suppressed": sum(1 for finding in findings if finding.allowed_by is not None),
        },
        "markers": {
            "closed": sorted(markers_removed - markers_added),
            "opened": sorted(markers_added - markers_removed),
            "moved": sorted(markers_added & markers_removed),
        },
        "findings": [finding.as_dict() for finding in active],
        "suppressed": [finding.as_dict() for finding in findings if finding.allowed_by is not None],
    }


def print_text(receipt: dict, limit: int) -> None:
    summary = receipt["summary"]
    print(f"{TOOL} {receipt['mode']}: {receipt['status']} "
          f"({summary['errors']} errors, {summary['warnings']} warnings, {summary['suppressed']} suppressed)")
    if receipt["input"]["patchDigest"]:
        print(f"patch sha256:{receipt['input']['patchDigest']}")
    markers = receipt["markers"]
    for key in ("closed", "opened"):
        if markers[key]:
            print(f"markers {key}: {', '.join(markers[key])}")
    counts: dict[str, int] = {}
    for finding in receipt["findings"]:
        counts[finding["rule"]] = counts.get(finding["rule"], 0) + 1
    if len(receipt["findings"]) > limit:
        print("by rule: " + ", ".join(f"{rule}={count}" for rule, count in sorted(counts.items())))
    for finding in receipt["findings"][:limit]:
        where = finding["path"] or "<patch>"
        if finding["line"]:
            where += f":{finding['line']}"
        print(f"{finding['severity']:5} {finding['rule']}  {where}  {finding['message']}")
    if len(receipt["findings"]) > limit:
        print(f"... {len(receipt['findings']) - limit} more; use --json for all findings")


def parse_args(argv: list[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--config", type=Path, help="permission config (default: <repo>/.codex/config.toml)")
    parser.add_argument("--profile", help="permission selection (default: config default_permissions, or legacy zircon-strict)")
    parser.add_argument("--json", action="store_true", help="print the receipt as JSON")
    parser.add_argument("--limit", type=int, default=60, help="maximum findings printed in text mode")
    commands = parser.add_subparsers(dest="command", required=True)
    patch = commands.add_parser("patch", help="check a unified diff")
    source = patch.add_mutually_exclusive_group(required=True)
    source.add_argument("--patch", help="unified diff file, or - for stdin")
    source.add_argument("--git", action="store_true", help="diff --path entries against --base with read-only git queries")
    patch.add_argument("--path", action="append", default=None, help="declared allowlist entry (repeatable)")
    patch.add_argument("--base", default="HEAD")
    patch.add_argument("--contributor", action="store_true",
                       help="check a contributor PR without agent write grants or a mandatory --path list")
    scan = commands.add_parser("scan", help="check whole files with the content rules")
    scan.add_argument("paths", nargs="+")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        root = args.repo_root.absolute()
        if root != root.resolve():
            raise ValueError("repository root must be the physical checkout, not a path alias")
        if args.command == "scan":
            paths = [normalize_rel(path) for path in args.paths]
            context = Context(root, "scan", scan_changes(root, paths))
            digest = base = None
        else:
            declared = [normalize_rel(path) for path in args.path] if args.path else None
            if args.contributor and args.config:
                raise ValueError("--contributor cannot be combined with --config")
            if not args.contributor and not declared:
                raise ValueError("agent handoff requires at least one --path; use --contributor for contributor PRs")
            grants = None if args.contributor else Grants.load(args.config or root / ".codex" / "config.toml", args.profile, root)
            if args.git:
                if not declared:
                    raise ValueError("--git requires at least one --path")
                text, base = git_patch(root, declared, args.base)
            else:
                if args.patch == "-":
                    raw = sys.stdin.buffer.read() if hasattr(sys.stdin, "buffer") else sys.stdin.read().encode("utf-8")
                else:
                    raw = Path(args.patch).read_bytes()
                text = raw.decode("utf-8")
                base = None
            if args.git:
                raw = text.encode("utf-8")
            digest = hashlib.sha256(raw).hexdigest()
            context = Context(root, "patch", parse_unified_diff(text), grants, declared, contributor=args.contributor)
            if not context.changes:
                raise ValueError("patch contains no file changes")
        findings = evaluate(context)
    except (OSError, ValueError) as error:
        print(f"{TOOL}: {error}", file=sys.stderr)
        return 2
    receipt = build_receipt(args.command, findings, context, digest, base)
    if args.json:
        print(json.dumps(receipt, indent=2, ensure_ascii=False))
    else:
        print_text(receipt, args.limit)
    return 1 if receipt["status"] == "failed" else 0


if __name__ == "__main__":
    sys.exit(main())

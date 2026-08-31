"""Rust allow-attribute inventory and exemption policy enforcement."""

from __future__ import annotations

from bisect import bisect_right
from collections import Counter
from pathlib import Path
import re
import subprocess
import tomllib


RUST_ALLOW_ATTRIBUTE_PATTERN = re.compile(r"#\s*!?\s*\[\s*allow\s*\(")
RUST_EXEMPTION_ENFORCED_MEMBERS = ("zircon_app", "zircon_runtime_interface")


def audit_rust_exemptions(
    repo_root: Path,
    *,
    rule_report: dict[str, object],
    convention_rules_path: Path,
    rule_id_pattern: str,
) -> dict[str, object]:
    repo_root = repo_root.resolve()
    exemption_marker_pattern = re.compile(
        rf"^\s*//\s*EXEMPT\((?P<rule_id>{rule_id_pattern})\):(?P<reason>.*)$"
    )
    known_rule_ids = set(rule_report["rule_ids"])
    must_rule_ids = set(rule_report["must_rule_ids"])
    member_roots, workspace_violations = _workspace_member_roots(repo_root)
    member_by_root = {member_root: member for member, member_root in member_roots}
    enforced_members = sorted(RUST_EXEMPTION_ENFORCED_MEMBERS)
    enforced_member_set = set(enforced_members)
    allow_counts: Counter[str] = Counter()
    exemption_counts: Counter[str] = Counter()
    violations = list(workspace_violations)
    available_members = {member for member, _ in member_roots}
    for missing_member in sorted(enforced_member_set - available_members):
        violations.append(
            {
                "member": missing_member,
                "path": "Cargo.toml",
                "line": 0,
                "reason": "missing enforced workspace member",
            }
        )

    for catalog_violation in rule_report["violations"]:
        violations.append(
            {
                "member": "<catalog>",
                "path": convention_rules_path.as_posix(),
                "line": catalog_violation["line"],
                "reason": (
                    "invalid exemption rule catalog: "
                    f"{catalog_violation['rule_id']} ({catalog_violation['reason']})"
                ),
            }
        )
    if not must_rule_ids and not rule_report["violations"]:
        violations.append(
            {
                "member": "<catalog>",
                "path": convention_rules_path.as_posix(),
                "line": 0,
                "reason": "missing exemption rule catalog",
            }
        )

    allow_candidate_files, source_inventory = _workspace_allow_candidate_sources(
        repo_root, member_roots
    )
    if source_inventory.startswith("git-error:"):
        violations.append(
            {
                "member": "<workspace>",
                "path": ".git",
                "line": 0,
                "reason": source_inventory.removeprefix("git-error:"),
            }
        )

    scoped_allow_attribute_count = 0
    for source in sorted(allow_candidate_files):
        member = _owning_workspace_member(source, member_by_root)
        relative_path = source.relative_to(repo_root).as_posix()
        try:
            contents = source.read_text(encoding="utf-8-sig")
        except (OSError, UnicodeDecodeError) as error:
            violations.append(
                {
                    "member": member,
                    "path": relative_path,
                    "line": 0,
                    "reason": f"unreadable Rust source: {type(error).__name__}",
                }
            )
            continue
        lexical_view, line_comments, line_starts = _rust_lexical_view(contents)
        for attribute in RUST_ALLOW_ATTRIBUTE_PATTERN.finditer(lexical_view):
            line_number = bisect_right(line_starts, attribute.start())
            allow_counts[member] += 1
            marker_line = line_comments.get(line_number - 1, "")
            marker = exemption_marker_pattern.match(marker_line)
            marker_rule_id = marker.group("rule_id") if marker else None
            marker_reason = marker.group("reason").strip() if marker else ""
            if marker_rule_id in must_rule_ids and marker_reason:
                exemption_counts[marker_rule_id] += 1

            if member not in enforced_member_set:
                continue
            scoped_allow_attribute_count += 1
            reason: str | None = None
            if not marker_line.strip().startswith("// EXEMPT("):
                reason = "missing exemption marker"
            elif marker is None:
                reason = "malformed exemption marker"
            elif marker_rule_id not in known_rule_ids:
                reason = "unknown exemption rule id"
            elif marker_rule_id not in must_rule_ids:
                reason = "non-MUST exemption rule id"
            elif not marker_reason:
                reason = "empty exemption reason"
            if reason is not None:
                violations.append(
                    {
                        "member": member,
                        "path": relative_path,
                        "line": line_number,
                        "reason": reason,
                    }
                )

    violations.sort(
        key=lambda item: (str(item["path"]), int(item["line"]), str(item["reason"]))
    )
    allow_attribute_count = sum(allow_counts.values())
    return {
        "schema_version": 1,
        "workspace_member_count": len(member_roots),
        "allow_candidate_file_count": len(allow_candidate_files),
        "source_inventory": source_inventory,
        "enforced_members": enforced_members,
        "allow_attribute_count": allow_attribute_count,
        "scoped_allow_attribute_count": scoped_allow_attribute_count,
        "unscoped_allow_attribute_count": (
            allow_attribute_count - scoped_allow_attribute_count
        ),
        "valid_exemption_count": sum(exemption_counts.values()),
        "allow_counts_by_member": dict(sorted(allow_counts.items())),
        "valid_exemption_counts_by_rule": dict(sorted(exemption_counts.items())),
        "violation_count": len(violations),
        "violations": violations,
    }


def _rust_lexical_view(source: str) -> tuple[str, dict[int, str], list[int]]:
    view = list(source)
    line_starts = [0, *(index + 1 for index, char in enumerate(source) if char == "\n")]
    line_comments: dict[int, str] = {}
    index = 0
    while index < len(source):
        if source.startswith("//", index):
            end = source.find("\n", index)
            if end < 0:
                end = len(source)
            line_number = bisect_right(line_starts, index)
            line_start = line_starts[line_number - 1]
            if not source[line_start:index].strip():
                line_comments[line_number] = source[index:end]
            _mask_rust_lexeme(view, index, end)
            index = end
            continue
        if source.startswith("/*", index):
            end = _nested_block_comment_end(source, index)
            _mask_rust_lexeme(view, index, end)
            index = end
            continue
        raw_string_end = _raw_string_end(source, index)
        if raw_string_end is not None:
            _mask_rust_lexeme(view, index, raw_string_end)
            index = raw_string_end
            continue
        if source[index] == '"':
            end = _quoted_string_end(source, index)
            _mask_rust_lexeme(view, index, end)
            index = end
            continue
        if source[index] == "'":
            end = _character_literal_end(source, index)
            if end is not None:
                _mask_rust_lexeme(view, index, end)
                index = end
                continue
        index += 1
    return "".join(view), line_comments, line_starts


def _mask_rust_lexeme(view: list[str], start: int, end: int) -> None:
    for index in range(start, end):
        if view[index] not in {"\r", "\n"}:
            view[index] = " "


def _nested_block_comment_end(source: str, start: int) -> int:
    depth = 1
    index = start + 2
    while index < len(source) and depth:
        if source.startswith("/*", index):
            depth += 1
            index += 2
        elif source.startswith("*/", index):
            depth -= 1
            index += 2
        else:
            index += 1
    return index


def _raw_string_end(source: str, start: int) -> int | None:
    if start and (source[start - 1].isalnum() or source[start - 1] == "_"):
        return None
    if source.startswith(("br", "cr"), start):
        delimiter_start = start + 2
    elif source.startswith("r", start):
        delimiter_start = start + 1
    else:
        return None
    quote = delimiter_start
    while quote < len(source) and source[quote] == "#":
        quote += 1
    if quote >= len(source) or source[quote] != '"':
        return None
    hashes = source[delimiter_start:quote]
    terminator = '"' + hashes
    end = source.find(terminator, quote + 1)
    return len(source) if end < 0 else end + len(terminator)


def _quoted_string_end(source: str, start: int) -> int:
    index = start + 1
    while index < len(source):
        if source[index] == "\\":
            index += 2
        elif source[index] == '"':
            return index + 1
        else:
            index += 1
    return len(source)


def _character_literal_end(source: str, start: int) -> int | None:
    value = start + 1
    if value >= len(source) or source[value] in {"\r", "\n", "'"}:
        return None
    if source[value] == "\\":
        value += 1
        if value >= len(source):
            return None
        if source[value] == "u" and value + 1 < len(source) and source[value + 1] == "{":
            value = source.find("}", value + 2)
            if value < 0:
                return None
        elif source[value] == "x":
            value += 2
    if value + 1 < len(source) and source[value + 1] == "'":
        return value + 2
    return None


def _workspace_member_roots(
    repo_root: Path,
) -> tuple[list[tuple[str, Path]], list[dict[str, object]]]:
    manifest = repo_root / "Cargo.toml"
    violations: list[dict[str, object]] = []
    try:
        with manifest.open("rb") as source:
            members = tomllib.load(source).get("workspace", {}).get("members")
    except (OSError, tomllib.TOMLDecodeError) as error:
        return [], [
            {
                "member": "<workspace>",
                "path": "Cargo.toml",
                "line": 0,
                "reason": f"unreadable workspace manifest: {type(error).__name__}",
            }
        ]
    if not isinstance(members, list):
        return [], [
            {
                "member": "<workspace>",
                "path": "Cargo.toml",
                "line": 0,
                "reason": "missing workspace members",
            }
        ]

    member_roots: list[tuple[str, Path]] = []
    seen_member_roots: set[Path] = set()
    for raw_member in members:
        if not isinstance(raw_member, str) or any(token in raw_member for token in "*?["):
            violations.append(
                {
                    "member": str(raw_member),
                    "path": "Cargo.toml",
                    "line": 0,
                    "reason": "workspace member must be a literal path",
                }
            )
            continue
        declared_member = Path(raw_member.replace("\\", "/")).as_posix().rstrip("/")
        member_root = (repo_root / declared_member).resolve()
        try:
            member = member_root.relative_to(repo_root).as_posix()
        except ValueError:
            member_root_is_valid = False
        else:
            member_root_is_valid = member_root.is_dir()
        if not member_root_is_valid:
            violations.append(
                {
                    "member": declared_member,
                    "path": declared_member,
                    "line": 0,
                    "reason": "missing or escaped workspace member",
                }
            )
            continue
        if member_root in seen_member_roots:
            violations.append(
                {
                    "member": member,
                    "path": declared_member,
                    "line": 0,
                    "reason": "duplicate canonical workspace member",
                }
            )
            continue
        seen_member_roots.add(member_root)
        member_roots.append((member, member_root))
    return member_roots, violations


def _workspace_allow_candidate_sources(
    repo_root: Path, member_roots: list[tuple[str, Path]]
) -> tuple[set[Path], str]:
    if not member_roots:
        return set(), "none"
    member_root_set = {root for _, root in member_roots}
    pathspecs = tuple(
        pathspec
        for member, _ in member_roots
        for pathspec in (
            f":(glob){member}/*.rs",
            f":(glob){member}/**/*.rs",
        )
    )
    try:
        completed = subprocess.run(
            (
                "git",
                "grep",
                "--untracked",
                "--exclude-standard",
                "-l",
                "-z",
                "-F",
                "allow",
                "--",
                *pathspecs,
            ),
            cwd=repo_root,
            check=False,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="surrogateescape",
        )
    except OSError as error:
        if (repo_root / ".git").exists():
            return set(), f"git-error:unable to inventory Rust exemptions: {type(error).__name__}"
        completed = None
    if completed is not None and completed.returncode in {0, 1}:
        sources = {
            candidate
            for raw_path in completed.stdout.split("\0")
            if raw_path
            for candidate in [(repo_root / raw_path).resolve()]
            if candidate.is_file()
            and (
                candidate in member_root_set
                or any(
                    parent in member_root_set for parent in candidate.parents
                )
            )
        }
        return sources, "git-grep"
    if completed is not None and (repo_root / ".git").exists():
        return set(), f"git-error:git grep failed with exit {completed.returncode}"

    sources: set[Path] = set()
    for _, member_root in member_roots:
        sources.update(_member_rust_sources(member_root))
    return sources, "cargo-roots-fallback"


def _member_rust_sources(member_root: Path) -> set[Path]:
    sources: set[Path] = set()
    for directory_name in ("src", "tests", "examples", "benches"):
        source_root = member_root / directory_name
        if source_root.is_dir():
            sources.update(source_root.rglob("*.rs"))

    default_build_script = member_root / "build.rs"
    if default_build_script.is_file():
        sources.add(default_build_script)

    manifest = member_root / "Cargo.toml"
    if not manifest.is_file():
        return sources
    try:
        with manifest.open("rb") as source:
            manifest_data = tomllib.load(source)
    except (OSError, tomllib.TOMLDecodeError):
        return sources

    explicit_paths: list[str] = []
    package = manifest_data.get("package")
    if isinstance(package, dict) and isinstance(package.get("build"), str):
        explicit_paths.append(package["build"])
    library = manifest_data.get("lib")
    if isinstance(library, dict) and isinstance(library.get("path"), str):
        explicit_paths.append(library["path"])
    for target_kind in ("bin", "example", "test", "bench"):
        targets = manifest_data.get(target_kind, [])
        if not isinstance(targets, list):
            continue
        explicit_paths.extend(
            target["path"]
            for target in targets
            if isinstance(target, dict) and isinstance(target.get("path"), str)
        )
    for explicit_path in explicit_paths:
        candidate = (member_root / explicit_path).resolve()
        try:
            candidate.relative_to(member_root)
        except ValueError:
            continue
        if candidate.is_file() and candidate.suffix == ".rs":
            sources.add(candidate)
    return sources


def _owning_workspace_member(
    source: Path, member_by_root: dict[Path, str]
) -> str:
    member = member_by_root.get(source)
    if member is not None:
        return member
    for parent in source.parents:
        member = member_by_root.get(parent)
        if member is not None:
            return member
    raise ValueError(f"source has no owning workspace member: {source}")

#!/usr/bin/env python3
"""Validate and build the ZirconEngine Wiki site.

The Markdown tree and navigation.yaml are the source of truth.  This module
keeps the validation usable without MkDocs, while the build command delegates
HTML generation to the pinned MkDocs Material toolchain.

Frontmatter may describe source or plan files that are intentionally delivered
in a separate code commit.  Those missing metadata targets are reported as
warnings by default; ``--strict-metadata`` turns them into hard errors.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
from typing import Any, Iterable
from urllib.parse import unquote

try:
    import yaml
except ImportError:  # pragma: no cover - exercised by the user-facing error path.
    yaml = None  # type: ignore[assignment]


REPO_ROOT = Path(__file__).resolve().parents[1]
DOCS_ROOT_NAME = Path("docs") / "wiki"
NAVIGATION_NAME = "navigation.yaml"
REQUIRED_FRONTMATTER = (
    "related_code",
    "implementation_files",
    "plan_sources",
    "tests",
    "doc_type",
)
LINK_PATTERN = re.compile(r"\[[^\]]*\]\((?:<)?([^)>\s]+)(?:>)?\)")
FENCE_PATTERN = re.compile(r"^\s*```")
H1_PATTERN = re.compile(r"^#\s+\S")
CONTROL_PATTERN = re.compile(r"[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]")


class WikiValidationError(ValueError):
    """Raised when the source tree cannot produce a complete Wiki site."""

    def __init__(self, errors: Iterable[str]) -> None:
        self.errors = tuple(errors)
        super().__init__("\n".join(self.errors))


@dataclass(frozen=True)
class WikiPage:
    slug: str
    path: str
    title: str


@dataclass(frozen=True)
class WikiSection:
    identifier: str
    title: str
    pages: tuple[WikiPage, ...]


@dataclass(frozen=True)
class WikiNavigation:
    version: int
    title: str
    locale: str
    sections: tuple[WikiSection, ...]

    @property
    def pages(self) -> tuple[WikiPage, ...]:
        return tuple(page for section in self.sections for page in section.pages)


def _require_yaml() -> Any:
    if yaml is None:
        raise RuntimeError(
            "PyYAML is required. Install the pinned documentation dependencies "
            "with `python -m pip install -r requirements-docs.txt`."
        )
    return yaml


def _repo_root(value: Path | str | None) -> Path:
    root = (Path(value) if value is not None else REPO_ROOT).resolve()
    if not root.is_dir():
        raise WikiValidationError((f"repository root does not exist: {root}",))
    return root


def _docs_root(repo_root: Path) -> Path:
    root = repo_root / DOCS_ROOT_NAME
    if not root.is_dir():
        raise WikiValidationError((f"Wiki source directory does not exist: {root}",))
    return root


def _safe_relative_path(raw: str, *, base: Path, label: str) -> Path:
    normalized = raw.replace("\\", "/")
    relative = PurePosixPath(normalized)
    if relative.is_absolute() or ".." in relative.parts:
        raise WikiValidationError((f"{label} escapes its allowed root: {raw}",))
    candidate = (base / Path(*relative.parts)).resolve()
    try:
        candidate.relative_to(base.resolve())
    except ValueError as error:
        raise WikiValidationError((f"{label} escapes its allowed root: {raw}",)) from error
    return candidate


def load_navigation(repo_root: Path | str | None = None) -> WikiNavigation:
    """Load and structurally validate the checked-in navigation manifest."""

    root = _repo_root(repo_root)
    docs_root = _docs_root(root)
    navigation_path = docs_root / NAVIGATION_NAME
    if not navigation_path.is_file():
        raise WikiValidationError((f"navigation manifest does not exist: {navigation_path}",))

    parser = _require_yaml()
    raw = parser.safe_load(navigation_path.read_text(encoding="utf-8-sig"))
    errors: list[str] = []
    if not isinstance(raw, dict):
        raise WikiValidationError(("navigation manifest must contain a YAML mapping",))
    if raw.get("version") != 1:
        errors.append("navigation.yaml must declare version: 1")
    title = raw.get("title")
    locale = raw.get("locale")
    if not isinstance(title, str) or not title.strip():
        errors.append("navigation.yaml title must be a non-empty string")
        title = "ZirconEngine 文档"
    if not isinstance(locale, str) or not locale.strip():
        errors.append("navigation.yaml locale must be a non-empty string")
        locale = "zh-CN"

    sections_raw = raw.get("sections")
    if not isinstance(sections_raw, list) or not sections_raw:
        errors.append("navigation.yaml sections must be a non-empty list")
        sections_raw = []

    sections: list[WikiSection] = []
    seen_section_ids: set[str] = set()
    seen_slugs: set[str] = set()
    seen_paths: set[str] = set()
    for section_index, section_raw in enumerate(sections_raw):
        location = f"sections[{section_index}]"
        if not isinstance(section_raw, dict):
            errors.append(f"{location} must be a mapping")
            continue
        identifier = section_raw.get("id")
        section_title = section_raw.get("title")
        if not isinstance(identifier, str) or not re.fullmatch(r"[a-z0-9-]+", identifier):
            errors.append(f"{location}.id must contain only lowercase letters, digits, and hyphens")
            identifier = f"section-{section_index}"
        if identifier in seen_section_ids:
            errors.append(f"duplicate section id: {identifier}")
        seen_section_ids.add(identifier)
        if not isinstance(section_title, str) or not section_title.strip():
            errors.append(f"{location}.title must be a non-empty string")
            section_title = identifier
        pages_raw = section_raw.get("pages")
        if not isinstance(pages_raw, list) or not pages_raw:
            errors.append(f"{location}.pages must be a non-empty list")
            pages_raw = []

        pages: list[WikiPage] = []
        for page_index, page_raw in enumerate(pages_raw):
            page_location = f"{location}.pages[{page_index}]"
            if not isinstance(page_raw, dict):
                errors.append(f"{page_location} must be a mapping")
                continue
            slug = page_raw.get("slug")
            path = page_raw.get("path")
            page_title = page_raw.get("title")
            if not isinstance(slug, str) or not re.fullmatch(r"[a-z0-9][a-z0-9-]*", slug):
                errors.append(f"{page_location}.slug is invalid: {slug!r}")
                slug = f"page-{section_index}-{page_index}"
            if slug in seen_slugs:
                errors.append(f"duplicate page slug: {slug}")
            seen_slugs.add(slug)
            if not isinstance(path, str) or not path.endswith(".md"):
                errors.append(f"{page_location}.path must be a Markdown path")
                path = f"invalid/{section_index}-{page_index}.md"
            if path in seen_paths:
                errors.append(f"duplicate navigation path: {path}")
            seen_paths.add(path)
            if not isinstance(page_title, str) or not page_title.strip():
                errors.append(f"{page_location}.title must be a non-empty string")
                page_title = path
            if isinstance(path, str):
                try:
                    if not _safe_relative_path(path, base=docs_root, label=page_location).is_file():
                        errors.append(f"{page_location}.path does not exist: {path}")
                except WikiValidationError as error:
                    errors.extend(error.errors)
            pages.append(WikiPage(slug=slug, path=path, title=page_title))
        sections.append(WikiSection(identifier, section_title, tuple(pages)))

    if errors:
        raise WikiValidationError(errors)
    return WikiNavigation(int(raw["version"]), title, locale, tuple(sections))


def mkdocs_navigation(navigation: WikiNavigation) -> list[dict[str, list[dict[str, str]]]]:
    """Convert the repository navigation shape to MkDocs' page tree."""

    return [
        {section.title: [{page.title: page.path} for page in section.pages]}
        for section in navigation.sections
    ]


def _frontmatter(text: str, path: Path) -> tuple[list[str], dict[str, Any]]:
    lines = text.splitlines()
    if not lines or lines[0] != "---":
        raise WikiValidationError((f"{path}: frontmatter must start on the first line",))
    closing = next((index for index in range(1, len(lines)) if lines[index] == "---"), None)
    if closing is None:
        raise WikiValidationError((f"{path}: frontmatter is not terminated",))
    parser = _require_yaml()
    parsed = parser.safe_load("\n".join(lines[1:closing]))
    if not isinstance(parsed, dict):
        raise WikiValidationError((f"{path}: frontmatter must be a YAML mapping",))
    keys = [line.split(":", 1)[0].strip() for line in lines[1:closing] if ":" in line]
    return keys, parsed


def _iter_path_values(value: Any) -> Iterable[str]:
    if isinstance(value, str):
        yield value
    elif isinstance(value, (list, tuple)):
        for item in value:
            yield from _iter_path_values(item)


def _validate_declared_paths(
    frontmatter: dict[str, Any],
    path: Path,
    repo_root: Path,
    warnings: list[str],
    *,
    strict_metadata: bool,
) -> list[str]:
    errors: list[str] = []

    def report_missing(field: str, declared: str, candidate: Path) -> None:
        message = (
            f"{path}:{field} references a path not present in this checkout: "
            f"{declared}"
        )
        if strict_metadata:
            errors.append(message)
        elif message not in warnings:
            warnings.append(message)

    for field in ("related_code", "implementation_files", "tests"):
        for declared in _iter_path_values(frontmatter.get(field)):
            if declared.startswith("user:") or re.match(r"^[a-z][a-z0-9+.-]*://", declared):
                continue
            if any(marker in declared for marker in ("*", "?", "[", "]", "{", "}")):
                continue
            normalized = declared.split("::", 1)[0].replace("\\", "/")
            if normalized.startswith("./"):
                normalized = normalized[2:]
            try:
                candidate = _safe_relative_path(normalized, base=repo_root, label=f"{path}:{field}")
            except WikiValidationError as error:
                errors.extend(error.errors)
                continue
            if not candidate.exists():
                report_missing(field, declared, candidate)
    for declared in _iter_path_values(frontmatter.get("plan_sources")):
        if declared.startswith("user:") or re.match(r"^[a-z][a-z0-9+.-]*://", declared):
            continue
        if any(marker in declared for marker in ("*", "?", "[", "]", "{", "}")):
            continue
        normalized = declared.split("::", 1)[0].replace("\\", "/")
        try:
            candidate = _safe_relative_path(normalized, base=repo_root, label=f"{path}:plan_sources")
        except WikiValidationError as error:
            errors.extend(error.errors)
            continue
        if not candidate.exists():
            report_missing("plan_sources", declared, candidate)
    return errors


def _validate_markdown(
    path: Path,
    repo_root: Path,
    warnings: list[str],
    *,
    strict_metadata: bool,
) -> list[str]:
    text = path.read_text(encoding="utf-8-sig")
    errors: list[str] = []
    try:
        keys, frontmatter = _frontmatter(text, path)
    except WikiValidationError as error:
        return list(error.errors)
    positions = {key: index for index, key in enumerate(keys)}
    for index, field in enumerate(REQUIRED_FRONTMATTER):
        if field not in positions:
            errors.append(f"{path}: missing frontmatter field {field}")
        elif index and positions[field] <= positions[REQUIRED_FRONTMATTER[index - 1]]:
            errors.append(f"{path}: frontmatter fields are out of order")
    errors.extend(
        _validate_declared_paths(
            frontmatter,
            path,
            repo_root,
            warnings,
            strict_metadata=strict_metadata,
        )
    )

    in_fence = False
    fence_count = 0
    h1_count = 0
    for line in text.splitlines():
        if FENCE_PATTERN.match(line):
            in_fence = not in_fence
            fence_count += 1
            continue
        if in_fence:
            continue
        if line.rstrip() != line:
            errors.append(f"{path}: trailing whitespace")
        if CONTROL_PATTERN.search(line):
            errors.append(f"{path}: control character found")
        if H1_PATTERN.match(line):
            h1_count += 1
        for match in LINK_PATTERN.finditer(line):
            target = unquote(match.group(1)).split("#", 1)[0].split("?", 1)[0]
            if not target or target.startswith(("http://", "https://", "mailto:", "#", "/")):
                continue
            candidate = (path.parent / target).resolve()
            try:
                candidate.relative_to(repo_root.resolve())
            except ValueError:
                errors.append(f"{path}: link escapes repository: {target}")
                continue
            if not candidate.exists():
                errors.append(f"{path}: broken link target: {target}")
    if fence_count % 2:
        errors.append(f"{path}: unbalanced Markdown code fence")
    if h1_count != 1:
        errors.append(f"{path}: expected exactly one H1, found {h1_count}")
    return errors


def validate(
    repo_root: Path | str | None = None,
    *,
    strict_metadata: bool = False,
) -> dict[str, Any]:
    """Validate navigation, Markdown metadata, links, and coverage."""

    root = _repo_root(repo_root)
    docs_root = _docs_root(root)
    errors: list[str] = []
    warnings: list[str] = []
    try:
        navigation = load_navigation(root)
    except WikiValidationError as error:
        navigation = None
        errors.extend(error.errors)

    markdown_files = tuple(sorted(docs_root.rglob("*.md")))
    for markdown_path in markdown_files:
        errors.extend(
            _validate_markdown(
                markdown_path,
                root,
                warnings,
                strict_metadata=strict_metadata,
            )
        )

    navigation_paths: set[str] = set()
    if navigation is not None:
        navigation_paths = {page.path for page in navigation.pages}
        markdown_paths = {
            markdown_path.relative_to(docs_root).as_posix() for markdown_path in markdown_files
        }
        for missing in sorted(markdown_paths - navigation_paths):
            errors.append(f"navigation.yaml omits Markdown page: {missing}")
        for extra in sorted(navigation_paths - markdown_paths):
            errors.append(f"navigation.yaml references non-existent Markdown page: {extra}")

    report = {
        "repository": str(root),
        "navigation": str(docs_root / NAVIGATION_NAME),
        "markdown_pages": len(markdown_files),
        "navigation_pages": len(navigation_paths),
        "error_count": len(errors),
        "errors": errors,
        "warning_count": len(warnings),
        "warnings": warnings,
    }
    if errors:
        raise WikiValidationError(errors)
    return report


def _safe_output_path(repo_root: Path, value: str) -> Path:
    raw = Path(value)
    output = (raw if raw.is_absolute() else repo_root / raw).resolve()
    try:
        output.relative_to(repo_root.resolve())
    except ValueError as error:
        raise WikiValidationError((f"build output must stay inside the repository: {value}",)) from error
    if output == repo_root.resolve():
        raise WikiValidationError(("build output cannot be the repository root",))
    return output


def build(
    repo_root: Path | str | None = None,
    output: str = "site",
    *,
    strict_metadata: bool = False,
) -> dict[str, Any]:
    root = _repo_root(repo_root)
    report = validate(root, strict_metadata=strict_metadata)
    output_path = _safe_output_path(root, output)
    environment = os.environ.copy()
    environment["WIKI_SITE_DIR"] = output_path.relative_to(root).as_posix()
    command = [
        sys.executable,
        "-m",
        "mkdocs",
        "build",
        "--config-file",
        "mkdocs.yml",
        "--strict",
        "--clean",
    ]
    try:
        completed = subprocess.run(command, cwd=root, env=environment, check=False)
    except OSError as error:
        raise RuntimeError(
            "MkDocs is unavailable. Install the pinned documentation dependencies "
            "with `python -m pip install -r requirements-docs.txt`."
        ) from error
    if completed.returncode != 0:
        raise RuntimeError(f"MkDocs build failed with exit code {completed.returncode}")
    if not (output_path / "index.html").is_file():
        raise RuntimeError(f"MkDocs build did not produce {output_path / 'index.html'}")
    return {**report, "output": str(output_path)}


def serve(
    repo_root: Path | str | None = None,
    dev_addr: str = "127.0.0.1:8000",
    *,
    strict_metadata: bool = False,
) -> None:
    root = _repo_root(repo_root)
    validate(root, strict_metadata=strict_metadata)
    command = [
        sys.executable,
        "-m",
        "mkdocs",
        "serve",
        "--config-file",
        "mkdocs.yml",
        "--dev-addr",
        dev_addr,
    ]
    try:
        completed = subprocess.run(command, cwd=root, check=False)
    except OSError as error:
        raise RuntimeError(
            "MkDocs is unavailable. Install the pinned documentation dependencies "
            "with `python -m pip install -r requirements-docs.txt`."
        ) from error
    except KeyboardInterrupt:
        # Ctrl+C is the normal way to stop a local preview server.
        return
    raise SystemExit(completed.returncode)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=REPO_ROOT)
    subparsers = parser.add_subparsers(dest="command", required=True)
    validate_parser = subparsers.add_parser("validate", help="validate navigation and Markdown")
    validate_parser.add_argument("--json", action="store_true", help="emit JSON")
    validate_parser.add_argument(
        "--strict-metadata",
        action="store_true",
        help="fail when frontmatter source, test, or plan paths are absent",
    )
    build_parser = subparsers.add_parser("build", help="validate and build the static site")
    build_parser.add_argument("--output", default="site", help="output directory inside the repository")
    build_parser.add_argument("--json", action="store_true", help="emit JSON after building")
    build_parser.add_argument(
        "--strict-metadata",
        action="store_true",
        help="fail when frontmatter source, test, or plan paths are absent",
    )
    serve_parser = subparsers.add_parser("serve", help="validate and start the local preview server")
    serve_parser.add_argument("--dev-addr", default="127.0.0.1:8000")
    serve_parser.add_argument(
        "--strict-metadata",
        action="store_true",
        help="fail when frontmatter source, test, or plan paths are absent",
    )
    args = parser.parse_args(argv)
    try:
        if args.command == "validate":
            report = validate(args.repo_root, strict_metadata=args.strict_metadata)
            if args.json:
                print(json.dumps(report, ensure_ascii=False, indent=2))
            else:
                print(
                    "Wiki validation passed: "
                    f"{report['markdown_pages']} Markdown pages, "
                    f"{report['navigation_pages']} navigation entries, "
                    f"{report['warning_count']} metadata warnings."
                )
            return 0
        if args.command == "build":
            report = build(
                args.repo_root,
                args.output,
                strict_metadata=args.strict_metadata,
            )
            if args.json:
                print(json.dumps(report, ensure_ascii=False, indent=2))
            else:
                print(f"Wiki site built at {report['output']}")
            return 0
        serve(args.repo_root, args.dev_addr, strict_metadata=args.strict_metadata)
        return 0
    except WikiValidationError as error:
        for message in error.errors:
            print(f"ERROR: {message}", file=sys.stderr)
        return 1
    except RuntimeError as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())

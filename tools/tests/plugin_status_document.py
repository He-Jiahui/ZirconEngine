from __future__ import annotations

import re
from functools import lru_cache
from pathlib import Path


MARKDOWN_LINK_PATTERN = re.compile(r"\[[^\]]*\]\(([^)\s]+\.md)\)")
NUMBERED_ARCHIVE_DIRECTORY_PATTERN = re.compile(r"^\d{2}$")
DATED_ARCHIVE_FILE_PATTERN = re.compile(r"^\d{4}-\d{2}-\d{2}-.+\.md$")
RESOLVED_ARCHIVE_BLOCK_PATTERN = re.compile(
    r"\n?<!-- resolved plan output archive: .*? -->\n.*?"
    r"<!-- end resolved plan output archive -->\n?",
    re.DOTALL,
)


class StatusDocumentPath(type(Path())):
    """Test-only path that resolves numbered plan-output archive links in place."""

    def read_text(self, encoding=None, errors=None, newline=None):
        if self.suffix.lower() != ".md":
            return super().read_text(
                encoding=encoding,
                errors=errors,
                newline=newline,
            )
        resolved = Path(self).resolve()
        document_stat = resolved.stat()
        text = read_plain_text_cached(
            str(resolved),
            encoding,
            errors,
            newline,
            document_stat.st_mtime_ns,
            document_stat.st_size,
        )
        archive_fingerprints = numbered_output_archive_fingerprints(
            resolved,
            text,
        )
        return read_expanded_text_cached(
            str(resolved),
            encoding,
            errors,
            newline,
            document_stat.st_mtime_ns,
            document_stat.st_size,
            archive_fingerprints,
        )


@lru_cache(maxsize=256)
def read_plain_text_cached(
    path: str,
    encoding: str | None,
    errors: str | None,
    newline: str | None,
    _mtime_ns: int,
    _size: int,
) -> str:
    return Path(path).read_text(
        encoding=encoding,
        errors=errors,
        newline=newline,
    )


@lru_cache(maxsize=128)
def read_expanded_text_cached(
    path: str,
    encoding: str | None,
    errors: str | None,
    newline: str | None,
    mtime_ns: int,
    size: int,
    _archive_fingerprints: tuple[tuple[str, int, int], ...],
) -> str:
    document_path = Path(path)
    text = read_plain_text_cached(
        path,
        encoding,
        errors,
        newline,
        mtime_ns,
        size,
    )
    return expand_numbered_output_archives(
        document_path,
        text,
        encoding or "utf-8",
    )


def expand_numbered_output_archives(
    document_path: Path,
    text: str,
    encoding: str,
) -> str:
    def replace_link(match: re.Match[str]) -> str:
        target = match.group(1)
        archive_path = (document_path.parent / target).resolve()
        if not is_numbered_output_archive(archive_path) or not archive_path.is_file():
            return match.group(0)
        archive_text = Path(archive_path).read_text(encoding=encoding)
        return (
            f"{match.group(0)}\n\n"
            f"<!-- resolved plan output archive: {target} -->\n"
            f"{archive_text}\n"
            "<!-- end resolved plan output archive -->"
        )

    return MARKDOWN_LINK_PATTERN.sub(replace_link, text)


def numbered_output_archive_fingerprints(
    document_path: Path,
    text: str,
) -> tuple[tuple[str, int, int], ...]:
    fingerprints: list[tuple[str, int, int]] = []
    for match in MARKDOWN_LINK_PATTERN.finditer(text):
        archive_path = (document_path.parent / match.group(1)).resolve()
        if not is_numbered_output_archive(archive_path) or not archive_path.is_file():
            continue
        archive_stat = archive_path.stat()
        fingerprints.append(
            (str(archive_path), archive_stat.st_mtime_ns, archive_stat.st_size)
        )
    return tuple(fingerprints)


def is_numbered_output_archive(path: Path) -> bool:
    return bool(
        NUMBERED_ARCHIVE_DIRECTORY_PATTERN.fullmatch(path.parent.name)
        and DATED_ARCHIVE_FILE_PATTERN.fullmatch(path.name)
    )


def strip_resolved_output_archives(text: str) -> str:
    """Remove injected historical evidence for current-wording assertions."""
    return RESOLVED_ARCHIVE_BLOCK_PATTERN.sub("", text)

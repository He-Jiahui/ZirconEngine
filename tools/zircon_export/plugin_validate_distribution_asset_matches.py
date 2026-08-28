"""Filesystem matching for plugin distribution asset globs."""

from __future__ import annotations

from pathlib import Path


def _portable_distribution_asset_glob(pattern: str) -> str:
    pattern_path = Path(pattern)
    if pattern_path.name == "**":
        # Python 3.12 yields only the directory for a terminal `**`, while
        # newer versions also yield its contents. The manifest contract is recursive.
        return str(pattern_path / "*")
    return pattern


def plugin_validate_distribution_asset_matches(
    *,
    pattern: str,
    plugin_root: Path,
    resolved_plugin_root: Path,
    item_label: str,
    diagnostics: list[str],
) -> list[tuple[Path, Path]]:
    try:
        matches = sorted(
            path
            for path in plugin_root.glob(_portable_distribution_asset_glob(pattern))
            if path.is_file()
        )
    except (OSError, ValueError, NotImplementedError) as error:
        diagnostics.append(f"{item_label} could not be matched: {error}")
        return []
    if not matches:
        diagnostics.append(f"{item_label} matched no plugin asset files")
        return []

    contained_matches: list[tuple[Path, Path]] = []
    for source_path in matches:
        resolved_source = source_path.resolve()
        try:
            relative_source = resolved_source.relative_to(resolved_plugin_root)
        except ValueError:
            diagnostics.append(
                f"{item_label} matched asset outside plugin root: {resolved_source}"
            )
            continue
        contained_matches.append((source_path, relative_source))
    return contained_matches

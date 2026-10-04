"""PlatformBundle template-resolution path semantics diagnostics."""

from __future__ import annotations

from os.path import normcase
from pathlib import Path
from typing import Any


def template_resolution_path_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    containment_diagnostics: list[str] = []
    uniqueness_diagnostics: list[str] = []
    template_root = resolution.get("template_root")
    resolved_root: Path | None = None
    if isinstance(template_root, str) and template_root.strip():
        try:
            resolved_root = Path(template_root).expanduser().resolve()
        except OSError as error:
            containment_diagnostics.append(
                f"{label}.template_root could not be resolved: {error}"
            )

    seen: dict[str, tuple[str, int]] = {}
    for field in ("candidates", "skipped_candidates"):
        entries = resolution.get(field)
        if not isinstance(entries, list):
            continue
        for index, entry in enumerate(entries):
            if not isinstance(entry, dict):
                continue
            template_dir = entry.get("template_dir")
            if not isinstance(template_dir, str) or not template_dir.strip():
                continue
            try:
                resolved_template_dir = Path(template_dir).expanduser().resolve()
            except OSError as error:
                if resolved_root is not None:
                    containment_diagnostics.append(
                        f"{label} {field}[{index}].template_dir "
                        f"could not be resolved: {error}"
                    )
                continue

            if resolved_root is not None:
                try:
                    relative_template_dir = resolved_template_dir.relative_to(
                        resolved_root
                    )
                except ValueError:
                    containment_diagnostics.append(
                        f"{label} {field}[{index}].template_dir "
                        "must be inside template_root"
                    )
                else:
                    if len(relative_template_dir.parts) != 1:
                        containment_diagnostics.append(
                            f"{label} {field}[{index}].template_dir "
                            "must be a direct child of template_root"
                        )

            key = normcase(str(resolved_template_dir))
            if key in seen:
                seen_field, seen_index = seen[key]
                uniqueness_diagnostics.append(
                    f"{label} {field}[{index}].template_dir duplicates "
                    f"{seen_field}[{seen_index}].template_dir"
                )
                continue
            seen[key] = (field, index)

    return containment_diagnostics + uniqueness_diagnostics


def template_resolution_path_containment_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    template_root = resolution.get("template_root")
    if not isinstance(template_root, str) or not template_root.strip():
        return []
    try:
        resolved_root = Path(template_root).expanduser().resolve()
    except OSError as error:
        return [f"{label}.template_root could not be resolved: {error}"]

    diagnostics: list[str] = []
    diagnostics.extend(
        template_resolution_entries_inside_root_diagnostics(
            label,
            resolved_root,
            resolution,
            "candidates",
        )
    )
    diagnostics.extend(
        template_resolution_entries_inside_root_diagnostics(
            label,
            resolved_root,
            resolution,
            "skipped_candidates",
        )
    )
    return diagnostics


def template_resolution_template_dir_uniqueness_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    seen: dict[str, tuple[str, int]] = {}
    for field in ("candidates", "skipped_candidates"):
        entries = resolution.get(field)
        if not isinstance(entries, list):
            continue
        for index, entry in enumerate(entries):
            if not isinstance(entry, dict):
                continue
            template_dir = entry.get("template_dir")
            if not isinstance(template_dir, str) or not template_dir.strip():
                continue
            try:
                key = normcase(str(Path(template_dir).expanduser().resolve()))
            except OSError:
                continue
            if key in seen:
                seen_field, seen_index = seen[key]
                diagnostics.append(
                    f"{label} {field}[{index}].template_dir duplicates "
                    f"{seen_field}[{seen_index}].template_dir"
                )
                continue
            seen[key] = (field, index)
    return diagnostics


def template_resolution_entries_inside_root_diagnostics(
    label: str,
    resolved_root: Path,
    resolution: dict[str, Any],
    field: str,
) -> list[str]:
    entries = resolution.get(field)
    if not isinstance(entries, list):
        return []
    diagnostics: list[str] = []
    for index, entry in enumerate(entries):
        if not isinstance(entry, dict):
            continue
        template_dir = entry.get("template_dir")
        if not isinstance(template_dir, str) or not template_dir.strip():
            continue
        try:
            resolved_template_dir = Path(template_dir).expanduser().resolve()
            relative_template_dir = resolved_template_dir.relative_to(resolved_root)
        except ValueError:
            diagnostics.append(
                f"{label} {field}[{index}].template_dir must be inside template_root"
            )
        except OSError as error:
            diagnostics.append(
                f"{label} {field}[{index}].template_dir could not be resolved: {error}"
            )
        else:
            if len(relative_template_dir.parts) != 1:
                diagnostics.append(
                    f"{label} {field}[{index}].template_dir "
                    "must be a direct child of template_root"
                )
    return diagnostics

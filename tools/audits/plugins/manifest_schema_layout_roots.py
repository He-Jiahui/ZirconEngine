from __future__ import annotations

from typing import Any


LAYOUT_ROOT_FIELDS = ("asset_roots", "content_roots")


def collect_layout_root_schema_violations(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    for field in LAYOUT_ROOT_FIELDS:
        roots = manifest.get(field)
        if roots is None:
            continue
        if not isinstance(roots, list):
            violations.append(f"{display_path}: {field} must be an array")
            continue
        seen: dict[str, int] = {}
        for index, root in enumerate(roots):
            field_label = f"{field}[{index}]"
            if not is_non_empty_trimmed_string(root):
                violations.append(
                    f"{display_path}: {field_label} "
                    "must be a non-empty trimmed string"
                )
                continue
            previous_index = seen.get(root)
            if previous_index is not None:
                violations.append(
                    f"{display_path}: {field_label} {root} "
                    f"duplicates {field}[{previous_index}]"
                )
            else:
                seen[root] = index
            collect_layout_root_path_violations(
                display_path, field_label, root, violations
            )


def collect_layout_root_path_violations(
    display_path: str,
    field_label: str,
    root: str,
    violations: list[str],
) -> None:
    if root.startswith("/") or root.startswith("\\"):
        violations.append(f"{display_path}: {field_label} {root} must be relative")
    if layout_root_has_drive_separator(root):
        violations.append(
            f"{display_path}: {field_label} {root} "
            "must not contain a drive separator"
        )
    if "\\" in root:
        violations.append(
            f"{display_path}: {field_label} {root} must use forward slashes"
        )
    if any(segment in {"", ".", ".."} for segment in root.split("/")):
        violations.append(
            f"{display_path}: {field_label} {root} "
            "must not contain empty, current, or parent path segments"
        )


def layout_root_has_drive_separator(root: str) -> bool:
    return len(root) >= 2 and root[1] == ":" and root[0].isascii() and root[0].isalpha()


def is_non_empty_trimmed_string(value: object) -> bool:
    return isinstance(value, str) and value.strip() == value and bool(value)

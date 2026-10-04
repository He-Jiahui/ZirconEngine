from __future__ import annotations

from typing import Any


LAYOUT_COORDINATE_FIELDS = (
    "package_prefix",
    "package_company",
    "package_name",
)
LAYOUT_COORDINATE_COMPLETENESS_MESSAGE = (
    "package coordinates must declare package_prefix, package_company, "
    "and package_name together or leave all empty"
)


def collect_layout_coordinate_schema_violations(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    values = {field: manifest.get(field, "") for field in LAYOUT_COORDINATE_FIELDS}
    declares_any = any(layout_coordinate_declared(value) for value in values.values())
    declares_all = all(isinstance(value, str) and bool(value) for value in values.values())
    if declares_any and not declares_all:
        violations.append(
            f"{display_path}: {LAYOUT_COORDINATE_COMPLETENESS_MESSAGE}"
        )
    if not declares_any:
        return
    collect_layout_coordinate_prefix_violation(
        display_path,
        "package_prefix",
        values["package_prefix"],
        violations,
    )
    collect_layout_coordinate_segment_violation(
        display_path,
        "package_company",
        values["package_company"],
        violations,
    )
    collect_layout_coordinate_segment_violation(
        display_path,
        "package_name",
        values["package_name"],
        violations,
    )


def layout_coordinate_declared(value: object) -> bool:
    if isinstance(value, str):
        return bool(value)
    return value is not None


def collect_layout_coordinate_prefix_violation(
    display_path: str,
    label: str,
    value: object,
    violations: list[str],
) -> None:
    if not (
        isinstance(value, str)
        and value.strip()
        and value.strip() == value
        and all(layout_coordinate_lowercase_token(segment) for segment in value.split("."))
    ):
        violations.append(
            f"{display_path}: {label} {layout_coordinate_display(value)} "
            "must contain only non-empty lowercase coordinate segments"
        )


def collect_layout_coordinate_segment_violation(
    display_path: str,
    label: str,
    value: object,
    violations: list[str],
) -> None:
    if not (
        isinstance(value, str)
        and value.strip()
        and value.strip() == value
        and layout_coordinate_lowercase_token(value)
    ):
        violations.append(
            f"{display_path}: {label} {layout_coordinate_display(value)} "
            "must be a non-empty lowercase coordinate segment"
        )


def layout_coordinate_lowercase_token(value: str) -> bool:
    return bool(value) and all(
        char.isascii() and (char.islower() or char.isdigit() or char == "_")
        for char in value
    )


def layout_coordinate_display(value: object) -> str:
    return value if isinstance(value, str) else str(value)

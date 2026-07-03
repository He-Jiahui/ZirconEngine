"""Validate report CompileHost linked runtime crate schema diagnostics."""

from __future__ import annotations

from typing import Any

from .export_template_manifest import is_safe_relative_path, normalize_relative_path
from .pipeline_report_schema_primitives import validate_string_schema_diagnostics
from .pipeline_report_validate_identifier_schema import (
    validate_non_empty_trimmed_string_schema_diagnostics,
    validate_project_plugin_package_id_schema_diagnostics,
    validate_project_runtime_crate_name_schema_diagnostics,
)

VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_FIELDS = (
    "crate_name",
    "path",
    "provider_package_id",
    "registration_kind",
)
VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_STRING_FIELDS = (
    "crate_name",
    "path",
    "provider_package_id",
    "registration_kind",
)
VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_RELATIVE_PATH_FIELDS = ("path",)
VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_NAME_FIELDS = ("crate_name",)
VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_PROVIDER_ID_FIELDS = (
    "provider_package_id",
)
VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_REGISTRATION_KINDS = {
    "runtime_plugin",
}


def validate_linked_runtime_crate_schema_diagnostics(
    linked_runtime_crates: list[Any],
    *,
    label: str = "validate report plan_summary.library_embed_compile_host.linked_runtime_crates",
) -> list[str]:
    diagnostics: list[str] = []
    known_linked_crate_fields = set(VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_FIELDS)
    seen_crate_names: dict[str, int] = {}
    for index, crate in enumerate(linked_runtime_crates):
        if not isinstance(crate, dict):
            continue
        diagnostics.extend(
            f"{label}[{index}] unknown field {field}"
            for field in sorted(crate)
            if field not in known_linked_crate_fields
        )
        for field in VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_STRING_FIELDS:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"{label}[{index}].{field}",
                    crate.get(field),
                )
            )
            value = crate.get(field)
            if isinstance(value, str):
                diagnostics.extend(
                    validate_non_empty_trimmed_string_schema_diagnostics(
                        f"{label}[{index}].{field}",
                        value,
                    )
                )
        for field in VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_RELATIVE_PATH_FIELDS:
            value = crate.get(field)
            if (
                isinstance(value, str)
                and value.strip()
                and value.strip() == value
                and not linked_crate_path_is_safe(value)
            ):
                diagnostics.append(
                    f"{label}[{index}].{field} must be a safe relative path"
                )
        for field in VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_NAME_FIELDS:
            value = crate.get(field)
            if isinstance(value, str):
                crate_name_diagnostics = (
                    validate_project_runtime_crate_name_schema_diagnostics(
                        f"{label}[{index}].{field}",
                        value,
                    )
                )
                diagnostics.extend(crate_name_diagnostics)
                if crate_name_diagnostics:
                    continue
                previous_index = seen_crate_names.get(value)
                if previous_index is None:
                    seen_crate_names[value] = index
                    continue
                diagnostics.append(
                    f"{label}[{index}].{field} duplicates entry {previous_index}"
                )
        for field in VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_PROVIDER_ID_FIELDS:
            value = crate.get(field)
            if isinstance(value, str):
                diagnostics.extend(
                    validate_project_plugin_package_id_schema_diagnostics(
                        f"{label}[{index}].{field}",
                        value,
                    )
                )
        registration_kind = crate.get("registration_kind")
        if (
            isinstance(registration_kind, str)
            and registration_kind.strip()
            and registration_kind.strip() == registration_kind
            and registration_kind
            not in VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_REGISTRATION_KINDS
        ):
            diagnostics.append(
                f"{label}[{index}].registration_kind must be runtime_plugin"
            )
        diagnostics.extend(
            linked_runtime_crate_identity_diagnostics(
                crate,
                label=label,
                index=index,
            )
        )
    return diagnostics


def linked_runtime_crate_identity_diagnostics(
    crate: dict[str, Any],
    *,
    label: str,
    index: int,
) -> list[str]:
    diagnostics: list[str] = []
    provider_package_id = crate.get("provider_package_id")
    if not project_plugin_id_is_schema_clean(provider_package_id):
        return diagnostics

    crate_name = crate.get("crate_name")
    if project_runtime_crate_name_is_schema_clean(crate_name):
        expected_crate_name = expected_runtime_crate_name_for_provider_package_id(
            provider_package_id,
        )
        if crate_name != expected_crate_name:
            diagnostics.append(
                f"{label}[{index}].crate_name must match "
                f"provider_package_id {provider_package_id} as "
                f"{expected_crate_name}"
            )

    path = crate.get("path")
    if linked_crate_path_is_schema_clean(path) and not (
        linked_crate_path_matches_provider_package_id(
            provider_package_id,
            path,
        )
    ):
        diagnostics.append(
            f"{label}[{index}].path must match "
            f"provider_package_id {provider_package_id}"
        )
    return diagnostics


def linked_runtime_crates_cover_expected_plugins_diagnostics(
    expected_runtime_plugins: object,
    linked_runtime_crates: object,
    *,
    label: str,
    field_separator: str = ".",
) -> list[str]:
    if not isinstance(expected_runtime_plugins, list) or not isinstance(
        linked_runtime_crates,
        list,
    ):
        return []

    provider_ids = {
        provider_id
        for provider_id in linked_runtime_crate_provider_ids(linked_runtime_crates)
        if project_plugin_id_is_schema_clean(provider_id)
    }
    diagnostics: list[str] = []
    for index, plugin_id in enumerate(expected_runtime_plugins):
        if not project_plugin_id_is_schema_clean(plugin_id):
            continue
        if plugin_id not in provider_ids:
            diagnostics.append(
                f"{label}{field_separator}linked_runtime_crates must include "
                f"provider_package_id {plugin_id} for "
                f"expected_runtime_plugins[{index}]"
            )
    return diagnostics


def linked_runtime_crates_only_expected_plugins_diagnostics(
    expected_runtime_plugins: object,
    linked_runtime_crates: object,
    *,
    label: str,
    field_separator: str = ".",
) -> list[str]:
    if not isinstance(expected_runtime_plugins, list) or not isinstance(
        linked_runtime_crates,
        list,
    ):
        return []

    expected_plugin_ids = {
        plugin_id
        for plugin_id in expected_runtime_plugins
        if project_plugin_id_is_schema_clean(plugin_id)
    }
    diagnostics: list[str] = []
    for index, crate in enumerate(linked_runtime_crates):
        if not isinstance(crate, dict):
            continue
        provider_package_id = crate.get("provider_package_id")
        if not project_plugin_id_is_schema_clean(provider_package_id):
            continue
        if provider_package_id not in expected_plugin_ids:
            diagnostics.append(
                f"{label}{field_separator}linked_runtime_crates[{index}]"
                ".provider_package_id must be listed in "
                "expected_runtime_plugins"
            )
    return diagnostics


def linked_runtime_crate_provider_ids(linked_runtime_crates: list[Any]) -> list[str]:
    provider_ids: list[str] = []
    for crate in linked_runtime_crates:
        if not isinstance(crate, dict):
            continue
        provider_package_id = crate.get("provider_package_id")
        if isinstance(provider_package_id, str):
            provider_ids.append(provider_package_id)
    return provider_ids


def project_plugin_id_is_schema_clean(value: object) -> bool:
    return isinstance(
        value,
        str,
    ) and not validate_project_plugin_package_id_schema_diagnostics(
        "plugin id",
        value,
    )


def project_runtime_crate_name_is_schema_clean(value: object) -> bool:
    return isinstance(
        value,
        str,
    ) and not validate_project_runtime_crate_name_schema_diagnostics(
        "runtime crate name",
        value,
    )


def expected_runtime_crate_name_for_provider_package_id(provider_package_id: str) -> str:
    return f"zircon_plugin_{provider_package_id}_runtime"


def linked_crate_path_is_schema_clean(value: object) -> bool:
    return isinstance(value, str) and linked_crate_path_is_safe(value)


def linked_crate_path_matches_provider_package_id(
    provider_package_id: str,
    path: str,
) -> bool:
    return provider_package_id in linked_crate_path_provider_identity_candidates(path)


def linked_crate_path_provider_identity_candidates(path: str) -> set[str]:
    path_tokens: list[list[str]] = []
    for component in normalize_relative_path(path).split("/"):
        if component in {"zircon_plugins", "features", "runtime"}:
            continue
        for token in identifier_tokens(component):
            path_tokens.append(token_identity_forms(token))
    if not path_tokens:
        return set()

    candidates: set[tuple[str, ...]] = {()}
    for token_forms in path_tokens:
        candidates = {
            (*candidate, token_form)
            for candidate in candidates
            for token_form in token_forms
        }
    return {"_".join(candidate) for candidate in candidates}


def identifier_tokens(value: str) -> list[str]:
    normalized = value.lower().replace("-", "_").replace(".", "_")
    return [token for token in normalized.split("_") if token]


def token_identity_forms(token: str) -> list[str]:
    if token.endswith("s") and len(token) > 3:
        return [token, token[:-1]]
    return [token]


def linked_crate_path_is_safe(value: str) -> bool:
    return (
        bool(value.strip())
        and value.strip() == value
        and is_safe_relative_path(normalize_relative_path(value))
    )

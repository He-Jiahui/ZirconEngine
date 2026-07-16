from __future__ import annotations

from typing import Any

from .manifest_schema import (
    INIT_LEVEL_VALUES,
    MODULE_KIND_VALUES,
    REQUIRED_MODULE_FIELDS,
    SUPPORTED_TARGET_VALUES,
    collect_allowed_string_array_values,
    collect_allowed_string_value,
    collect_required_field_violation,
    is_non_empty_trimmed_string,
)


MODULE_FIELDS = frozenset(
    (
        "name",
        "description",
        "kind",
        "crate_name",
        "init_level",
        "module_dependencies",
        "target_modes",
        "capabilities",
        "system_sets",
        "system_anchors",
        "event_consumers",
    )
)
MODULE_SYSTEM_FIELDS = ("system_sets", "system_anchors")


def collect_module_schema_violations(
    display_path: str,
    field_label: str,
    module: object,
    violations: list[str],
    *,
    table_label: str | None = None,
    namespace_id: str | None = None,
    supported_targets: set[str] | None = None,
    seen_names: dict[str, str] | None = None,
    row_identity: str | None = None,
) -> None:
    if not isinstance(module, dict):
        violations.append(
            f"{display_path}: {table_label or field_label} must be a table"
        )
        return

    collect_module_known_field_violations(
        display_path,
        field_label,
        module,
        violations,
    )
    for field in REQUIRED_MODULE_FIELDS:
        collect_required_field_violation(
            display_path,
            f"{field_label}.{field}",
            module,
            violations,
            field_name=field,
        )
    module_name = module_trimmed_string(module, "name")
    module_kind = module_trimmed_string(module, "kind")
    crate_name = module_trimmed_string(module, "crate_name")
    target_modes = module_string_array(module, "target_modes")
    capabilities = module_string_array(module, "capabilities")

    collect_allowed_string_value(
        display_path,
        f"{field_label}.kind",
        module,
        "kind",
        MODULE_KIND_VALUES,
        violations,
    )
    if module_kind not in MODULE_KIND_VALUES:
        module_kind = None
    collect_allowed_string_array_values(
        display_path,
        f"{field_label}.target_modes",
        module,
        "target_modes",
        SUPPORTED_TARGET_VALUES,
        violations,
    )

    if module_name is not None:
        collect_module_dot_namespace_violations(
            display_path,
            f"{field_label}.name",
            module_name,
            violations,
        )
        if namespace_id:
            expected_prefix = f"{namespace_id}."
            if not module_name.startswith(expected_prefix):
                violations.append(
                    f"{display_path}: {field_label}.name {module_name} "
                    f"should stay under namespace {expected_prefix}"
                )
        collect_module_duplicate_name_violation(
            display_path,
            f"{field_label}.name",
            module_name,
            seen_names,
            row_identity or field_label,
            violations,
        )
    if module_name is not None and module_kind is not None:
        collect_module_name_kind_violation(
            display_path,
            field_label,
            module_name,
            module_kind,
            violations,
        )
    if crate_name is not None:
        collect_module_crate_name_violations(
            display_path,
            f"{field_label}.crate_name",
            crate_name,
            violations,
        )
    if target_modes is not None:
        collect_module_target_mode_violations(
            display_path,
            field_label,
            target_modes,
            module_kind,
            supported_targets or set(),
            violations,
        )
    if module_kind is not None and capabilities is not None:
        collect_module_capability_violations(
            display_path,
            f"{field_label}.capabilities",
            capabilities,
            module_kind,
            violations,
        )
    collect_module_descriptor_projection_violations(
        display_path,
        field_label,
        module,
        violations,
    )
    collect_module_system_contract_violations(
        display_path,
        field_label,
        module,
        module_kind,
        namespace_id,
        violations,
    )
    collect_module_event_consumer_violations(
        display_path,
        field_label,
        module,
        module_kind,
        namespace_id,
        violations,
    )


def collect_module_event_consumer_violations(
    display_path: str,
    field_label: str,
    module: dict[str, Any],
    module_kind: str | None,
    namespace_id: str | None,
    violations: list[str],
) -> None:
    consumers = module.get("event_consumers")
    if consumers is None:
        return
    if module_kind != "editor":
        violations.append(
            f"{display_path}: {field_label}.event_consumers may only be declared by editor modules"
        )
    if not isinstance(consumers, list) or not consumers:
        violations.append(
            f"{display_path}: {field_label}.event_consumers must be a non-empty array"
        )
        return
    seen_ids: set[str] = set()
    known_fields = {"consumer_id", "event_id", "payload_schema", "required_capability"}
    for index, consumer in enumerate(consumers):
        label = f"{field_label}.event_consumers[{index}]"
        if not isinstance(consumer, dict):
            violations.append(f"{display_path}: {label} must be a table")
            continue
        for field in sorted(consumer):
            if field not in known_fields:
                violations.append(f"{display_path}: {label}.{field} is not a known event consumer field")
        values: dict[str, str] = {}
        for field in ("consumer_id", "event_id", "payload_schema", "required_capability"):
            value = consumer.get(field)
            if not is_non_empty_trimmed_string(value):
                violations.append(f"{display_path}: {label}.{field} must be a non-empty trimmed string")
            else:
                values[field] = value
        consumer_id = values.get("consumer_id")
        if consumer_id is not None:
            if consumer_id in seen_ids:
                violations.append(f"{display_path}: {label}.consumer_id {consumer_id} is duplicated")
            seen_ids.add(consumer_id)
            if namespace_id and not consumer_id.startswith(f"{namespace_id}."):
                violations.append(
                    f"{display_path}: {label}.consumer_id {consumer_id} should stay under namespace {namespace_id}."
                )
        event_id = values.get("event_id")
        if event_id is not None and namespace_id and not event_id.startswith(f"{namespace_id}."):
            violations.append(
                f"{display_path}: {label}.event_id {event_id} should stay under namespace {namespace_id}."
            )
        capability = values.get("required_capability")
        if capability is not None and not capability.startswith("editor."):
            violations.append(
                f"{display_path}: {label}.required_capability {capability} should start with editor."
            )


def collect_module_known_field_violations(
    display_path: str,
    field_label: str,
    module: dict[str, Any],
    violations: list[str],
) -> None:
    for field in sorted(module):
        if field not in MODULE_FIELDS:
            violations.append(
                f"{display_path}: {field_label}.{field} "
                "is not a known module field"
        )


def collect_module_descriptor_projection_violations(
    display_path: str,
    field_label: str,
    module: dict[str, Any],
    violations: list[str],
) -> None:
    if "description" in module and not is_non_empty_trimmed_string(
        module.get("description")
    ):
        violations.append(
            f"{display_path}: {field_label}.description must be a non-empty trimmed string when declared"
        )
    if "init_level" in module:
        collect_allowed_string_value(
            display_path,
            f"{field_label}.init_level",
            module,
            "init_level",
            INIT_LEVEL_VALUES,
            violations,
        )
    if "module_dependencies" in module:
        collect_module_dependency_violations(
            display_path,
            field_label,
            module["module_dependencies"],
            violations,
        )


def collect_module_dependency_violations(
    display_path: str,
    field_label: str,
    dependencies: object,
    violations: list[str],
) -> None:
    label = f"{field_label}.module_dependencies"
    if not isinstance(dependencies, list) or not dependencies:
        violations.append(f"{display_path}: {label} must be a non-empty table array")
        return
    seen: dict[str, int] = {}
    for dependency_index, dependency in enumerate(dependencies):
        dependency_label = f"{label}[{dependency_index}]"
        if not isinstance(dependency, dict):
            violations.append(f"{display_path}: {dependency_label} must be a table")
            continue
        for field in sorted(dependency):
            if field != "module_name":
                violations.append(
                    f"{display_path}: {dependency_label}.{field} "
                    "is not a known module dependency field"
                )
        module_name = dependency.get("module_name")
        if not is_non_empty_trimmed_string(module_name):
            violations.append(
                f"{display_path}: {dependency_label}.module_name must be a non-empty trimmed string"
            )
            continue
        collect_module_dot_namespace_violations(
            display_path,
            f"{dependency_label}.module_name",
            module_name,
            violations,
        )
        previous_index = seen.get(module_name)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {dependency_label}.module_name {module_name} "
                f"duplicates module_dependencies[{previous_index}]"
            )
            continue
        seen[module_name] = dependency_index


def collect_module_duplicate_name_violation(
    display_path: str,
    field_label: str,
    module_name: str,
    seen_names: dict[str, str] | None,
    row_identity: str,
    violations: list[str],
) -> None:
    if seen_names is None:
        return
    previous = seen_names.get(module_name)
    if previous is not None:
        violations.append(
            f"{display_path}: {field_label} {module_name} "
            f"duplicates module name {previous}"
        )
        return
    seen_names[module_name] = row_identity


def collect_module_dot_namespace_violations(
    display_path: str,
    label: str,
    value: str,
    violations: list[str],
) -> None:
    segments = value.split(".")
    if len(segments) < 2:
        violations.append(
            f"{display_path}: {label} {value} "
            "should use package.module dot namespace form"
        )
    if any(not segment for segment in segments):
        violations.append(
            f"{display_path}: {label} {value} "
            "should not contain empty namespace segments"
        )
    if not all(
        char.isascii() and (char.islower() or char.isdigit() or char in {"_", "."})
        for char in value
    ):
        violations.append(
            f"{display_path}: {label} {value} should contain only lowercase ASCII "
            "letters, digits, underscores, and dots"
        )


def collect_module_name_kind_violation(
    display_path: str,
    field_label: str,
    module_name: str,
    module_kind: str,
    violations: list[str],
) -> None:
    expected_suffix = {"runtime": ".runtime", "editor": ".editor"}.get(module_kind)
    if expected_suffix is not None and not module_name.endswith(expected_suffix):
        violations.append(
            f"{display_path}: {field_label}.name {module_name} with kind "
            f"{module_kind} should end with {expected_suffix}"
        )


def collect_module_crate_name_violations(
    display_path: str,
    label: str,
    crate_name: str,
    violations: list[str],
) -> None:
    if not crate_name.startswith("zircon_plugin_"):
        violations.append(
            f"{display_path}: {label} {crate_name} should use the zircon_plugin_ prefix"
        )
    if not all(
        char.isascii() and (char.islower() or char.isdigit() or char == "_")
        for char in crate_name
    ):
        violations.append(
            f"{display_path}: {label} {crate_name} should use lowercase ASCII "
            "letters, digits, or underscores"
        )
    if crate_name.endswith("_") or "__" in crate_name:
        violations.append(
            f"{display_path}: {label} {crate_name} should not end with an underscore "
            "or contain repeated underscores"
        )


def collect_module_target_mode_violations(
    display_path: str,
    field_label: str,
    target_modes: list[str],
    module_kind: str | None,
    supported_targets: set[str],
    violations: list[str],
) -> None:
    allowed_targets = set(SUPPORTED_TARGET_VALUES)
    for target_index, target_mode in enumerate(target_modes):
        if target_mode not in allowed_targets:
            continue
        item_label = f"{field_label}.target_modes[{target_index}]"
        if supported_targets and target_mode not in supported_targets:
            violations.append(
                f"{display_path}: {item_label} {target_mode} "
                "should be covered by package supported_targets"
            )
        if module_kind == "editor" and target_mode != "editor_host":
            violations.append(
                f"{display_path}: {field_label} is an editor module and should only "
                f"target editor_host, got {target_mode}"
            )


def collect_module_capability_violations(
    display_path: str,
    label: str,
    capabilities: list[str],
    module_kind: str,
    violations: list[str],
) -> None:
    expected_prefix = {"runtime": "runtime.", "editor": "editor."}.get(module_kind)
    if expected_prefix is None:
        return
    for capability_index, capability in enumerate(capabilities):
        if not capability.startswith(expected_prefix):
            violations.append(
                f"{display_path}: {label}[{capability_index}] {capability} "
                f"should start with {expected_prefix}"
            )


def collect_module_system_contract_violations(
    display_path: str,
    field_label: str,
    module: dict[str, Any],
    module_kind: str | None,
    namespace_id: str | None,
    violations: list[str],
) -> None:
    for field in MODULE_SYSTEM_FIELDS:
        collect_module_system_field_violations(
            display_path,
            field_label,
            module,
            field,
            module_kind,
            namespace_id,
            violations,
        )


def collect_module_system_field_violations(
    display_path: str,
    field_label: str,
    module: dict[str, Any],
    field_name: str,
    module_kind: str | None,
    namespace_id: str | None,
    violations: list[str],
) -> None:
    if field_name not in module:
        return
    label = f"{field_label}.{field_name}"
    if module_kind is not None and module_kind != "runtime":
        violations.append(
            f"{display_path}: {label} may only be declared by runtime modules"
        )
    values = module_optional_string_array(
        display_path,
        label,
        module,
        field_name,
        violations,
    )
    if values is None:
        return
    seen: dict[str, int] = {}
    for item_index, item in enumerate(values):
        item_label = f"{label}[{item_index}]"
        collect_module_dot_namespace_violations(
            display_path,
            item_label,
            item,
            violations,
        )
        if namespace_id:
            expected_prefix = f"{namespace_id}."
            if not item.startswith(expected_prefix):
                violations.append(
                    f"{display_path}: {item_label} {item} "
                    f"should stay under namespace {expected_prefix}"
                )
        previous_index = seen.get(item)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {item_label} {item} "
                f"duplicates {field_name}[{previous_index}]"
            )
            continue
        seen[item] = item_index


def module_trimmed_string(
    module: dict[str, Any],
    field_name: str,
) -> str | None:
    value = module.get(field_name)
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        return None
    return value


def module_string_array(
    module: dict[str, Any],
    field_name: str,
) -> list[str] | None:
    value = module.get(field_name)
    if not isinstance(value, list) or not value:
        return None
    return [
        item
        for item in value
        if isinstance(item, str) and item.strip() and item.strip() == item
    ]


def module_optional_string_array(
    display_path: str,
    field_label: str,
    module: dict[str, Any],
    field_name: str,
    violations: list[str],
) -> list[str] | None:
    value = module[field_name]
    if not isinstance(value, list) or not value:
        violations.append(
            f"{display_path}: {field_label} must be a non-empty string array when declared"
        )
        return None
    values: list[str] = []
    for item_index, item in enumerate(value):
        if not isinstance(item, str) or not item.strip() or item.strip() != item:
            violations.append(
                f"{display_path}: {field_label}[{item_index}] "
                "must be a non-empty trimmed string"
            )
            continue
        values.append(item)
    return values


def module_supported_targets(manifest: dict[str, Any]) -> set[str]:
    supported_targets = manifest.get("supported_targets")
    if not isinstance(supported_targets, list):
        return set()
    return {
        target
        for target in supported_targets
        if isinstance(target, str) and target.strip() and target.strip() == target
    }

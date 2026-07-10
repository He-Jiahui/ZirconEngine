from __future__ import annotations

from typing import Any


def collect_native_crate_name_collisions(
    loaded_manifests: list[tuple[str, dict[str, Any]]],
) -> list[str]:
    """Return ambiguous dist-module ownership while allowing explicit dual entry kinds."""
    collisions: list[str] = []
    for display_path, manifest in loaded_manifests:
        distribution = manifest.get("distribution")
        if not isinstance(distribution, dict):
            continue
        forms = distribution.get("forms")
        if not isinstance(forms, list) or "dist" not in forms:
            continue
        modules = manifest.get("modules")
        if not isinstance(modules, list):
            continue

        owners: dict[str, tuple[int, str | None]] = {}
        for module_index, module in enumerate(modules):
            if not isinstance(module, dict):
                continue
            crate_name = module.get("crate_name")
            if not isinstance(crate_name, str) or not crate_name.strip():
                continue
            kind = module.get("kind")
            if not isinstance(kind, str) or not kind.strip():
                kind = None
            previous = owners.get(crate_name)
            if previous is None:
                owners[crate_name] = (module_index, kind)
                continue

            previous_index, previous_kind = previous
            if kind is None or previous_kind is None:
                collisions.append(
                    f"{display_path}: modules[{module_index}].crate_name {crate_name} "
                    f"reuses modules[{previous_index}] without an explicit distinct kind"
                )
            elif kind == previous_kind:
                collisions.append(
                    f"{display_path}: modules[{module_index}].crate_name {crate_name} "
                    f"duplicates {kind} ownership from modules[{previous_index}]"
                )
    return collisions

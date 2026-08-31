"""Plugin package models for zircon_build."""

from __future__ import annotations

import dataclasses
import functools
from pathlib import Path

try:
    from .zircon_build_plugin_manifest_contract import (
        PLUGIN_DISTRIBUTION_FORM_DIST,
        PLUGIN_DISTRIBUTION_FORM_EMBED,
    )
except ImportError:  # pragma: no cover - exercised when run as a script.
    from zircon_build_plugin_manifest_contract import (
        PLUGIN_DISTRIBUTION_FORM_DIST,
        PLUGIN_DISTRIBUTION_FORM_EMBED,
    )


@dataclasses.dataclass(frozen=True)
class CargoPackage:
    name: str
    member: str
    manifest_path: Path
    crate_types: tuple[str, ...]

    @property
    def is_native_dynamic(self) -> bool:
        return "cdylib" in self.crate_types


@dataclasses.dataclass(frozen=True)
class PluginPackage:
    plugin_id: str
    display_name: str
    manifest_path: Path
    package_root: Path
    asset_roots: tuple[Path, ...]
    default_packaging: tuple[str, ...]
    distribution_forms: tuple[str, ...]
    dist_crate_name: str | None
    module_crate_names: tuple[str, ...]
    shader_geometry_source_ids: tuple[str, ...]
    shader_geometry_source_descriptors: tuple[dict[str, object], ...]
    shader_shading_model_ids: tuple[str, ...]
    shader_shading_model_descriptors: tuple[dict[str, object], ...]
    crates: tuple[CargoPackage, ...]
    shader_modules: tuple[dict[str, object], ...] = ()

    @functools.cached_property
    def native_dynamic_crates(self) -> tuple[CargoPackage, ...]:
        if PLUGIN_DISTRIBUTION_FORM_DIST not in self.distribution_forms:
            return ()
        if self.dist_crate_name:
            return tuple(
                crate for crate in self.crates if crate.name == self.dist_crate_name
            )
        return tuple(crate for crate in self.crates if crate.is_native_dynamic)

    @functools.cached_property
    def rlib_static_crates(self) -> tuple[CargoPackage, ...]:
        if PLUGIN_DISTRIBUTION_FORM_EMBED not in self.distribution_forms:
            return ()
        if self.dist_crate_name and len(self.crates) > 1:
            return tuple(
                crate for crate in self.crates if crate.name != self.dist_crate_name
            )
        return self.crates

    @functools.cached_property
    def carriers(self) -> tuple[str, ...]:
        carriers: list[str] = []
        if self.native_dynamic_crates:
            carriers.append("native_dynamic")
        if self.rlib_static_crates:
            carriers.append("rlib_static")
        return tuple(carriers)

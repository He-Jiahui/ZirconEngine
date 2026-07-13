"""Shared build configuration data; target policy belongs in child owners."""

from __future__ import annotations

import dataclasses
from pathlib import Path

try:
    from .zircon_build_plugin_packages import PluginPackage
except ImportError:  # pragma: no cover - direct script import path.
    from zircon_build_plugin_packages import PluginPackage


TARGET_FEATURES = ("target-client", "target-server", "target-editor-host")
ENGINE_DIR_NAME = "ZirconEngine"


@dataclasses.dataclass(frozen=True)
class BuildConfig:
    repo_root: Path
    out_root: Path
    cargo: str
    mode: str
    targets: tuple[str, ...]
    runtime_features: tuple[str, ...]
    plugins: tuple[PluginPackage, ...]
    plugin_carrier: str
    locked: bool
    jobs: str | None
    dry_run: bool
    prewarm_shaders: bool
    validate_wgpu_shaders: bool
    validate_wgpu_pipelines: bool
    shader_quality_tiers: tuple[str, ...]
    shader_geometry_sources: tuple[str, ...]
    shader_asset_roots: tuple[Path, ...]
    shader_geometry_source_ids: tuple[str, ...]
    shader_shading_model_ids: tuple[str, ...]
    shader_permutation_registries: tuple[Path, ...]
    shader_resource_registry: Path | None
    font_sdf_manifest: Path | None

    @property
    def engine_root(self) -> Path:
        return self.out_root / ENGINE_DIR_NAME

    @property
    def targets_root(self) -> Path:
        return self.out_root / "targets"

    @property
    def profile_dir(self) -> str:
        if self.mode == "release":
            return "release"
        if self.mode == "profiling":
            return "profiling"
        return "debug"

    @property
    def runtime_feature_arg(self) -> str:
        return " ".join(self.runtime_features)

    @property
    def runtime_preview_feature_arg(self) -> str:
        return self.feature_arg_for_target("target-client")

    def feature_arg_for_target(self, target_feature: str) -> str:
        features = [target_feature]
        features.extend(
            feature
            for feature in self.runtime_features
            if feature not in TARGET_FEATURES and feature not in features
        )
        return " ".join(features)

    @property
    def shader_prewarm_cache_root(self) -> Path:
        return self.engine_root / "cache" / "shader_variants"

    @property
    def shader_prewarm_report_path(self) -> Path:
        return self.engine_root / "cache" / "shader_variants_report.json"

    @property
    def shader_prewarm_resource_registry_path(self) -> Path:
        return self.engine_root / "cache" / "shader_resource_records.json"

    @property
    def shader_prewarm_permutation_registry_path(self) -> Path:
        return self.engine_root / "cache" / "shader_permutation_registry.json"

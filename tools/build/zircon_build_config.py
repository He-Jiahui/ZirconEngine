"""Shared build configuration data; target policy belongs in child owners."""

from __future__ import annotations

import dataclasses
from pathlib import Path

from .zircon_build_plugin_packages import PluginPackage
from .zircon_build_product_profile import ProductBuildProfile, build_set_id_for


TARGET_FEATURES = ("target-client", "target-server", "target-editor-host")
ENGINE_DIR_NAME = "ZirconEngine"  # Published engine tree root.


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
    # Product profiles are deliberately optional so existing development
    # invocations retain their exact feature/profile behavior.
    product_profile: ProductBuildProfile | None = None
    target_triple: str | None = None
    clean_output: bool = False
    asset_scope: str = "all"
    project_pack_root: Path | None = None
    editor_asset_root: Path | None = None
    cargo_profile_override: str | None = None

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
        if self.mode in {"shipping", "shipping-symbols"}:
            return self.mode
        return "debug"

    @property
    def is_shipping(self) -> bool:
        return self.mode in {"shipping", "shipping-symbols"} or self.product_profile is not None

    @property
    def cargo_profile_name(self) -> str:
        if self.cargo_profile_override:
            return self.cargo_profile_override
        return self.product_profile.cargo_profile if self.product_profile else self.profile_dir

    @property
    def effective_target_triple(self) -> str | None:
        if self.product_profile is not None:
            return self.product_profile.target_triple
        return self.target_triple

    @property
    def app_feature_arg(self) -> str:
        if self.product_profile is not None:
            return " ".join(self.product_profile.app_features)
        return self.runtime_preview_feature_arg

    @property
    def product_profile_name(self) -> str | None:
        return self.product_profile.name if self.product_profile else None

    @property
    def build_set_id(self) -> str | None:
        if self.product_profile is None:
            return None
        # Product profiles own the complete feature closure.  Re-hashing the
        # same profile features as invocation "overrides" would give the
        # build runner a different identity from CompileHost/export for an
        # otherwise identical product.  Only an explicit profile override is
        # an additional input to the Build Set identity.
        feature_overrides: list[str] = []
        if self.cargo_profile_override:
            feature_overrides.append(f"cargo-profile:{self.cargo_profile_override}")
        return build_set_id_for(
            self.product_profile,
            selected_plugins=[package.plugin_id for package in self.plugins],
            feature_overrides=feature_overrides,
        )

    @property
    def symbols_root(self) -> Path:
        profile_name = self.product_profile_name or self.mode
        return self.out_root / "symbols" / profile_name

    @property
    def reports_root(self) -> Path:
        profile_name = self.product_profile_name or self.mode
        return self.out_root / "reports" / profile_name

    @property
    def size_report_path(self) -> Path:
        return self.reports_root / "product_size_report.json"

    @property
    def archive_path(self) -> Path:
        profile_name = self.product_profile_name or self.mode
        return self.out_root / "archives" / f"{profile_name}.zip"

    @property
    def runtime_feature_arg(self) -> str:
        return " ".join(self.runtime_features)

    @property
    def runtime_preview_feature_arg(self) -> str:
        return self.feature_arg_for_target("target-client")

    def feature_arg_for_target(self, target_feature: str) -> str:
        if self.product_profile is not None:
            # Product profiles carry an explicit app feature closure. Do not
            # prepend target-client/target-editor-host: those development
            # aliases intentionally expand the full project plugin catalog.
            return self.app_feature_arg
        features = [target_feature]
        feature_set = {target_feature}
        for feature in self.runtime_features:
            if feature not in TARGET_FEATURES and feature not in feature_set:
                features.append(feature)
                feature_set.add(feature)
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

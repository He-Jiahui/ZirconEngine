from pathlib import Path


class FakePrewarmConfig:
    cargo = "cargo"
    dry_run = False
    engine_root = Path("stage") / "ZirconEngine"
    jobs = "1"
    locked = True
    mode = "debug"
    plugins: tuple[object, ...] = ()
    repo_root = Path(".")
    shader_geometry_source_ids: tuple[str, ...] = ()
    shader_geometry_sources = ("static",)
    shader_asset_roots: tuple[Path, ...] = ()
    shader_permutation_registries: tuple[Path, ...] = ()
    shader_quality_tiers = ("medium",)
    shader_resource_registry = None
    shader_shading_model_ids: tuple[str, ...] = ()
    targets_root = Path("target") / "prewarm-summary-test"
    validate_wgpu_shaders = False
    validate_wgpu_pipelines = False

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

    def feature_arg_for_target(self, target: str) -> str:
        self._target = target
        return "target-server"


class FakePluginPackage:
    def __init__(
        self,
        asset_roots: tuple[Path, ...] = (),
        shader_geometry_source_ids: tuple[str, ...] = (),
        shader_geometry_source_descriptors: tuple[dict[str, object], ...] = (),
        shader_shading_model_ids: tuple[str, ...] = (),
        shader_shading_model_descriptors: tuple[dict[str, object], ...] = (),
        shader_modules: tuple[dict[str, object], ...] = (),
    ):
        self.asset_roots = asset_roots
        self.shader_geometry_source_ids = shader_geometry_source_ids
        self.shader_geometry_source_descriptors = shader_geometry_source_descriptors
        self.shader_shading_model_ids = shader_shading_model_ids
        self.shader_shading_model_descriptors = shader_shading_model_descriptors
        self.shader_modules = shader_modules

import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]


class HybridGiM4ContractTests(unittest.TestCase):
    def test_runtime_manifest_enables_the_graphics_contract_it_imports(self) -> None:
        manifest = (
            ROOT / "zircon_plugins/hybrid_gi/runtime/Cargo.toml"
        ).read_text(encoding="utf-8")

        self.assertIn(
            'zircon_runtime = { workspace = true, features = ["graphics"] }',
            manifest,
        )

    def test_project_fixture_uses_explicit_asset_roots_and_project_writers(self) -> None:
        fixture_root = (
            ROOT
            / "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/hybrid_gi_scene_prepare_material_fixtures.rs"
        )
        fixture = fixture_root.read_text(encoding="utf-8")
        project_documents = fixture_root.with_name(
            "hybrid_gi_scene_prepare_material_fixtures"
        ) / "project_documents.rs"
        fixture_contract = fixture + project_documents.read_text(encoding="utf-8")
        manifest = (
            ROOT / "zircon_plugins/hybrid_gi/runtime/Cargo.toml"
        ).read_text(encoding="utf-8")

        self.assertIn(
            '#[path = "hybrid_gi_scene_prepare_material_fixtures/project_documents.rs"]',
            fixture,
        )
        self.assertIn("ensure_layout(&manifest.asset_roots)", fixture)
        self.assertIn("manifest.primary_asset_root()", fixture)
        self.assertIn(
            "to_project_toml_string(|reference| persist_fixture_reference(asset_root, reference))",
            fixture_contract,
        )
        self.assertIn("fn persist_fixture_reference(", fixture_contract)
        self.assertNotIn(".assets_root()", fixture)
        self.assertNotIn(".ensure_layout().unwrap()", fixture)
        self.assertNotIn("RelPath::project_assets()", fixture_contract)
        self.assertNotIn(".to_toml_string().unwrap()", fixture_contract)
        self.assertIn("zircon_runtime_interface = { workspace = true }", manifest)

    def test_public_extract_exposes_serializable_mode_and_profile(self) -> None:
        source = (ROOT / "zircon_runtime/src/core/framework/render/scene_extract.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("pub enum RenderHybridGiMode", source)
        self.assertIn("BakedStaticDynamic", source)
        self.assertIn("pub enum RenderHybridGiProfile", source)
        self.assertIn("IndoorStatic", source)
        self.assertIn("pub mode: RenderHybridGiMode", source)
        self.assertIn("pub profile: RenderHybridGiProfile", source)
        self.assertIn("Serialize, Deserialize", source)

    def test_runtime_provider_consumes_plan11_baked_contract_read_only(self) -> None:
        source = (
            ROOT
            / "zircon_runtime/src/graphics/hybrid_gi_runtime_provider/prepare_input.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("LightmapConsumeContract", source)
        self.assertIn("baked_lighting", source)
        self.assertIn("has_baked_probe_grid", source)

    def test_scene_representation_has_explicit_surface_participation(self) -> None:
        source = (
            ROOT
            / "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/participation.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("enum HybridGiSurfaceParticipation", source)
        self.assertIn("BakedStatic", source)
        self.assertIn("DynamicReceiver", source)
        self.assertIn("HybridReceiver", source)
        self.assertIn("Disabled", source)

    def test_participation_tracks_baked_generation_and_epoch(self) -> None:
        source = (
            ROOT
            / "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/participation.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("light_set_generation", source)
        self.assertIn("participation_epoch", source)
        self.assertIn("synchronize", source)

    def test_source_ledger_forbids_baked_and_full_dynamic_double_ownership(self) -> None:
        source = (
            ROOT
            / "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/source_ledger.rs"
        ).read_text(encoding="utf-8")
        prepared = (
            ROOT / "zircon_runtime/src/core/framework/render/prepared_runtime_sidebands.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("HybridGiSourceLedger", source)
        self.assertIn("HYBRID_GI_SOURCE_BAKED_BASELINE", source)
        self.assertIn("HYBRID_GI_SOURCE_DYNAMIC_DELTA", source)
        self.assertIn("valid_source_mask", source)
        self.assertIn("RenderHybridGiCompositePolicy", prepared)
        self.assertIn("accepts_hybrid_gi_output", prepared)

    def test_baked_mode_filters_static_lights_and_shader_applies_dynamic_weight(self) -> None:
        representation = (
            ROOT
            / "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/representation.rs"
        ).read_text(encoding="utf-8")
        shader = (
            ROOT
            / "zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl"
        ).read_text(encoding="utf-8")

        self.assertIn("light.mobility == Mobility::Dynamic", representation)
        self.assertIn("hybrid_gi_source_ledger", shader)
        self.assertIn("dynamic_source_weight", shader)
        self.assertIn("source_ledger_valid", shader)

    def test_temporal_invalidation_covers_scene_revisions_and_per_probe_ownership(self) -> None:
        participation = (
            ROOT
            / "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/participation.rs"
        ).read_text(encoding="utf-8")
        prepared = (
            ROOT / "zircon_runtime/src/core/framework/render/prepared_runtime_sidebands.rs"
        ).read_text(encoding="utf-8")
        encoder = (
            ROOT
            / "zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/encode.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("transform_revision", participation)
        self.assertIn("material_revision", participation)
        self.assertIn("light_invalidation_signatures", participation)
        self.assertIn("pub stable_instance_key: u64", prepared)
        self.assertIn("pub source_mask: u32", prepared)
        self.assertIn("probe_temporal_signature", encoder)
        self.assertIn("policy.participation_epoch()", encoder)
        self.assertIn("baked_light_set_generation", encoder)

    def test_four_profiles_resolve_real_budgets_and_structured_baked_fallback(self) -> None:
        extract = (ROOT / "zircon_runtime/src/core/framework/render/scene_extract.rs").read_text(
            encoding="utf-8"
        )
        representation = (
            ROOT
            / "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/representation.rs"
        ).read_text(encoding="utf-8")

        for profile in ("FullyDynamic", "IndoorStatic", "OpenWorld", "Cinematic"):
            self.assertIn(f"RenderHybridGiProfile::{profile}", extract)
        self.assertIn("RenderHybridGiResolvedSettings", extract)
        self.assertIn("BakedLightingUnavailable", extract)
        self.assertIn("resolved_settings(true)", representation)
        self.assertIn("effective_mode", representation)

    def test_resolved_settings_flow_from_provider_stats_into_editor_diagnostics(self) -> None:
        provider_stats = (
            ROOT
            / "zircon_runtime/src/graphics/hybrid_gi_runtime_provider/runtime_stats.rs"
        ).read_text(encoding="utf-8")
        render_stats = (
            ROOT / "zircon_runtime/src/core/framework/render/backend_types.rs"
        ).read_text(encoding="utf-8")
        updater = (
            ROOT
            / "zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/hybrid_gi_stats.rs"
        ).read_text(encoding="utf-8")
        editor = (
            ROOT
            / "zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("resolved_settings: Option<RenderHybridGiResolvedSettings>", provider_stats)
        self.assertIn("last_hybrid_gi_resolved_settings", render_stats)
        self.assertIn(
            "last_hybrid_gi_resolved_settings = hybrid_gi_stats.resolved_settings()",
            updater,
        )
        self.assertIn("last_hybrid_gi_resolved_settings = None", updater)
        self.assertIn("Hybrid GI effective: profile=", editor)
        self.assertIn("Hybrid GI budgets: trace=", editor)
        self.assertIn("Hybrid GI fallback:", editor)

    def test_resolve_accepts_composed_graph_scene_velocity_ownership(self) -> None:
        source = (
            ROOT / "zircon_plugins/hybrid_gi/runtime/src/render_pass_executors.rs"
        ).read_text(encoding="utf-8")
        resolve_resources = source.split(
            "const RESOLVE_RESOURCES: &[ExpectedResource] = &[", 1
        )[1].split("const HISTORY_RESOURCES", 1)[0]

        self.assertIn(
            "ExpectedResource::any_of(\n        SCENE_VELOCITY_RESOURCE,\n"
            "        READ_ONLY_TEXTURE_INPUT_KINDS,",
            resolve_resources,
        )
        self.assertIn(
            "hybrid_gi_resolve_accepts_external_or_transient_scene_velocity",
            source,
        )

    def test_hybrid_gi_capability_preserves_the_scene_velocity_producer(self) -> None:
        flagship = (
            ROOT
            / "zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/apply_flagship_profile_features.rs"
        ).read_text(encoding="utf-8")
        tests = (
            ROOT
            / "zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/compile_options_for_profile.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "with_feature_enabled(BuiltinRenderFeature::Temporal)", flagship
        )
        self.assertIn(
            "hybrid_gi_keeps_the_scene_velocity_producer_without_enabling_taa",
            tests,
        )


if __name__ == "__main__":
    unittest.main()

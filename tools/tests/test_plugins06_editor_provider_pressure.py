import unittest
from pathlib import Path

from tools.plugins06_editor_provider_pressure import run


ROOT = Path(__file__).resolve().parents[2]
CARGO_TOML = ROOT / "zircon_app/Cargo.toml"
ENTRY_MOD = ROOT / "zircon_app/src/entry/mod.rs"
PROVIDERS = ROOT / "zircon_app/src/entry/first_party_editor_plugins.rs"
BATCH_VALIDATOR = ROOT / "tools/zircon-validation-plugins06-editor-provider-batch.ps1"


class Plugins06EditorProviderPressureTests(unittest.TestCase):
    def test_provider_composition_removes_neural_only_fallback(self) -> None:
        composition = run()["provider_composition"]

        self.assertEqual(composition["baseline_neural_only_registration_count"], 0)
        self.assertEqual(composition["candidate_neural_only_registration_count"], 1)
        self.assertEqual(composition["candidate_navigation_only_registration_count"], 1)
        self.assertEqual(composition["candidate_empty_fallback_branch_count"], 0)
        self.assertEqual(composition["target_editor_host_provider_count"], 2)

    def test_validation_workload_covers_both_real_provider_paths(self) -> None:
        workload = run()["validation_workload"]

        self.assertEqual(workload["manifest_projection_count_per_provider"], 21_504)
        self.assertEqual(workload["manifest_projection_count_all_providers"], 43_008)
        self.assertEqual(workload["registration_count_per_projection"], 1)

    def test_cargo_features_compose_through_provider_neutral_catalog(self) -> None:
        source = CARGO_TOML.read_text(encoding="utf-8")

        self.assertIn("first-party-editor-catalog = [", source)
        for feature in (
            "first-party-navigation-editor-plugin",
            "first-party-neural-editor-plugin",
        ):
            body = source.split(f"{feature} = [", 1)[1].split("]", 1)[0]
            self.assertIn('"first-party-editor-catalog"', body)
        target = source.split("target-editor-host = [", 1)[1].split("]", 1)[0]
        self.assertIn('"first-party-navigation-editor-plugin"', target)
        self.assertIn('"first-party-neural-editor-plugin"', target)

    def test_entry_source_uses_shared_catalog_without_empty_fallback(self) -> None:
        source = PROVIDERS.read_text(encoding="utf-8")
        entry_mod = ENTRY_MOD.read_text(encoding="utf-8")

        self.assertIn('#[cfg(feature = "first-party-editor-catalog")]', source)
        self.assertIn(
            "zircon_first_party_editor_catalog::first_party_editor_plugin_registrations_for_manifest",
            source,
        )
        self.assertNotIn("Vec::new()", source)
        self.assertIn("mod first_party_editor_plugins", entry_mod)

    def test_release_contract_keeps_two_provider_tests_and_budget(self) -> None:
        source = PROVIDERS.read_text(encoding="utf-8")

        self.assertIn("app_composition_projects_selected_navigation_editor_provider", source)
        self.assertIn("app_composition_projects_selected_neural_editor_provider", source)
        self.assertIn("const SAMPLE_COUNT: usize = 21", source)
        self.assertIn("const ITERATIONS: usize = 1_024", source)
        self.assertIn("const MAX_P95_US: u128 = 250_000", source)
        self.assertIn("PERF-MVP-PLUGINS06", source)

    def test_batch_validator_runs_two_no_default_feature_configurations(self) -> None:
        source = BATCH_VALIDATOR.read_text(encoding="utf-8")

        self.assertEqual(source.count("[pscustomobject]@{"), 2)
        self.assertEqual(source.count('"--no-default-features"'), 2)
        self.assertIn('"first-party-neural-editor-plugin"', source)
        self.assertIn('"first-party-navigation-editor-plugin"', source)
        self.assertIn('"--exact"', source)
        self.assertIn("PLUGINS06_BATCH_PASS", source)

    def test_acceptance_is_explicit_and_timing_pending(self) -> None:
        acceptance = run()["acceptance"]

        self.assertEqual(acceptance["provider_p95_maximum_microseconds"], 250_000)
        self.assertEqual(acceptance["percentile_method"], "nearest_rank")
        self.assertTrue(acceptance["navigation_release_timing_pending"])
        self.assertTrue(acceptance["neural_release_timing_pending"])

    def test_model_is_bound_to_current_and_head_sources(self) -> None:
        binding = run()["source_binding"]

        self.assertEqual(
            binding["git_revision"],
            "050d8e6c36cd1bf4f3ab0d8fc4df0864c1c29a3f",
        )
        self.assertEqual(len(binding["source_sha256"]), 5)
        self.assertEqual(len(binding["source_manifest_sha256"]), 64)

    def test_rejects_non_positive_workloads(self) -> None:
        with self.assertRaises(ValueError):
            run(provider_configurations=0)
        with self.assertRaises(ValueError):
            run(iterations_per_sample=0)


if __name__ == "__main__":
    unittest.main()

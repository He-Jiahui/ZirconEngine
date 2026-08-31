import unittest
from pathlib import Path

from tools.plugins01_03_09_pressure import run


ROOT = Path(__file__).resolve().parents[2]
NATIVE = ROOT / "zircon_plugins/plugin_sdk/src/native.rs"
NATIVE_TESTS = ROOT / "zircon_plugins/plugin_sdk/src/native/tests.rs"
WINDOW_PLUGIN = ROOT / "zircon_plugins/native_window_hosting/editor/src/plugin.rs"
WINDOW_TESTS = ROOT / "zircon_plugins/native_window_hosting/editor/src/tests.rs"
PARTICLE_SERVICE = ROOT / "zircon_plugins/particles/runtime/src/service.rs"
PARTICLE_TESTS = ROOT / "zircon_plugins/particles/runtime/src/tests/snapshot.rs"
BATCH_VALIDATOR = ROOT / "tools/zircon-validation-plugins01-03-09-batch.ps1"


class Plugins010309PressureTests(unittest.TestCase):
    def test_sealed_native_static_keeps_zero_cost_contract(self) -> None:
        native = run()["sealed_native_static"]

        self.assertEqual(native["baseline_blanket_sync_impl_count"], 1)
        self.assertEqual(native["candidate_blanket_sync_impl_count"], 0)
        self.assertEqual(native["candidate_audited_carrier_type_count"], 5)
        self.assertEqual(native["candidate_layout_overhead_bytes"], 0)
        self.assertEqual(native["candidate_runtime_allocation_count"], 0)

    def test_phantom_authoring_work_is_eliminated(self) -> None:
        authoring = run()["phantom_authoring"]

        self.assertEqual(authoring["baseline_contribution_count"], 8_000)
        self.assertEqual(authoring["candidate_contribution_count"], 0)
        self.assertEqual(
            authoring["baseline_missing_template_resolution_count"], 1_000
        )
        self.assertEqual(authoring["candidate_missing_template_resolution_count"], 0)

    def test_particle_snapshot_eliminates_large_payload_clones(self) -> None:
        snapshot = run()["particle_snapshot"]

        self.assertEqual(
            snapshot["baseline_large_payload_element_clone_count"], 557_056
        )
        self.assertEqual(snapshot["candidate_large_payload_element_clone_count"], 0)
        self.assertEqual(snapshot["candidate_shared_handle_clone_count"], 256)
        self.assertEqual(snapshot["diagnostic_retention_limit"], 256)
        self.assertEqual(snapshot["diagnostic_page_limit"], 64)

    def test_native_source_restricts_sync_to_audited_carriers(self) -> None:
        source = NATIVE.read_text(encoding="utf-8")
        tests = NATIVE_TESTS.read_text(encoding="utf-8")

        self.assertIn("unsafe trait NativePluginStaticValue", source)
        self.assertIn("unsafe impl<T: NativePluginStaticValue> Sync", source)
        self.assertNotIn("unsafe impl<T> Sync for NativePluginStatic<T>", source)
        self.assertIn("PERF-MVP-PLUGINS01-SEALED-NATIVE-STATIC", tests)

    def test_window_source_keeps_empty_authoring_registration(self) -> None:
        source = WINDOW_PLUGIN.read_text(encoding="utf-8")
        tests = WINDOW_TESTS.read_text(encoding="utf-8")
        registration = source.split("fn register_editor_extensions", 1)[1].split(
            "pub fn editor_plugin_descriptor", 1
        )[0]

        self.assertIn("Ok(())", registration)
        self.assertNotIn("registry.", registration)
        self.assertIn("PERF-MVP-PLUGINS03-NO-PHANTOM-AUTHORING", tests)

    def test_particle_source_keeps_shared_bounded_payloads(self) -> None:
        source = PARTICLE_SERVICE.read_text(encoding="utf-8")
        tests = PARTICLE_TESTS.read_text(encoding="utf-8")

        self.assertIn("pub sprites: Arc<[crate::ParticleSpriteSnapshot]>", source)
        self.assertIn("pub diagnostics: Arc<[ParticleRuntimeDiagnostic]>", source)
        self.assertIn("const MAX_RUNTIME_DIAGNOSTICS: usize = 256", source)
        self.assertIn("const MAX_RUNTIME_DIAGNOSTIC_PAGE: usize = 64", source)
        self.assertIn("PARTICLE_SNAPSHOT_SHARE_BENCH_V1", tests)
        self.assertIn("optimized_p95.saturating_mul(4) <= legacy_p95", tests)

    def test_batch_validator_covers_three_packages_and_four_commands(self) -> None:
        source = BATCH_VALIDATOR.read_text(encoding="utf-8")

        self.assertEqual(source.count("[pscustomobject]@{"), 4)
        for package in (
            "zircon_plugin_sdk",
            "zircon_plugin_native_window_hosting_editor",
            "zircon_plugin_particles_runtime",
        ):
            self.assertIn(package, source)
        self.assertIn('"--locked"', source)
        self.assertIn('"--release"', source)
        self.assertIn('"--jobs"', source)
        self.assertIn("PLUGINS01_03_09_BATCH_PASS", source)

    def test_acceptance_is_explicit_and_particle_timing_pending(self) -> None:
        acceptance = run()["acceptance"]

        self.assertTrue(acceptance["native_static_zero_layout_overhead_required"])
        self.assertEqual(acceptance["phantom_contributions_must_equal"], 0)
        self.assertEqual(
            acceptance["particle_snapshot_p95_maximum_legacy_ratio"], 0.25
        )
        self.assertTrue(acceptance["particle_release_timing_pending"])

    def test_model_is_bound_to_current_and_head_sources(self) -> None:
        binding = run()["source_binding"]

        self.assertEqual(
            binding["git_revision"],
            "ca3ac3cc6ad218d04a5cd469447cea2452441321",
        )
        self.assertEqual(len(binding["source_sha256"]), 10)
        self.assertEqual(len(binding["source_manifest_sha256"]), 64)

    def test_rejects_non_positive_workloads(self) -> None:
        with self.assertRaises(ValueError):
            run(registrations=0)
        with self.assertRaises(ValueError):
            run(snapshot_iterations=0)


if __name__ == "__main__":
    unittest.main()

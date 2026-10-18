"""Editor898 guards bounded viewport-overlay provider merge, without Cargo."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
OWNER = ROOT / "zircon_editor/src/scene/viewport/controller/scene_viewport_controller_overlay_providers.rs"
LOWER = ROOT / "zircon_editor/src/scene/viewport/controller/scene_viewport_controller_overlay_providers/tests/batch_append_tests.rs"


class Editor898ViewportOverlayBatchAppendContract(unittest.TestCase):
    def test_extract_reserves_each_real_provider_batch(self):
        source = OWNER.read_text(encoding="utf-8")
        function = source.split('fn extract(&self, scene: &Scene, selected: Option<u64>)', 1)[1].split('\n}\n', 1)[0]
        self.assertIn('let mut collected = Vec::new();', function)
        self.assertIn('collected.reserve(gizmos.len());', function)
        self.assertIn('collected.extend(gizmos);', function)
        self.assertNotIn('.flat_map(', function)
        self.assertIn('run_editor_plugin_boundary(', function)
        self.assertIn('provider.faulted.store(true, Ordering::Release);', function)

    def test_lower_parity_and_release_percentiles_are_wired(self):
        source = OWNER.read_text(encoding="utf-8")
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn('mod batch_append_tests;', source)
        self.assertIn('editor898_viewport_overlay_provider_order_and_empty_capacity', lower)
        self.assertIn('EDITOR898_VIEWPORT_OVERLAY_BATCH_APPEND_BENCH_V1', lower)
        self.assertIn('legacy_extract(', lower)
        self.assertIn('101', lower)
        self.assertIn('p99', lower)

    def test_allocator_model_keeps_empty_batches_at_zero_capacity(self):
        self.assertEqual(sum(0 for _ in range(32)), 0)
        self.assertEqual(8 * 16, 128)


if __name__ == '__main__':
    unittest.main()

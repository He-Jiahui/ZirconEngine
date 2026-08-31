from pathlib import Path
import unittest

from tools.ui_draw_buffer_reuse_pressure import run


ROOT = Path(__file__).resolve().parents[2]
RENDER_PASS = ROOT / (
    "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/render_pass.rs"
)
PROFILE_MANIFEST = ROOT / "tools/profile-capture-manifest.ps1"
PROFILE_GATE = ROOT / "tools/ui-profile-counter-evidence.ps1"


class UiDrawBufferReusePerformanceContractTests(unittest.TestCase):
    def source(self) -> str:
        return RENDER_PASS.read_text(encoding="utf-8")

    def test_unversioned_damage_reuses_capacity_without_aliasing_uploads(self) -> None:
        source = self.source()
        resolve = source.split("impl WgpuUiDrawBufferCache", 1)[1].split(
            "pub(super) enum TargetLoad", 1
        )[0]

        self.assertIn("WgpuUiDrawBuffers::upload", resolve)
        self.assertIn("self.buffers.as_ref()", resolve)
        self.assertIn("self.key = cache_key", resolve)
        self.assertIn("self.buffers = Some(buffers.clone())", resolve)
        self.assertNotIn("cache_key?", resolve)

    def test_matching_projection_generation_skips_buffer_uploads(self) -> None:
        source = self.source()
        resolve = source.split("impl WgpuUiDrawBufferCache", 1)[1].split(
            "pub(super) enum TargetLoad", 1
        )[0]

        cache_hit = resolve.split("if self.key == Some(key)", 1)[1].split(
            "let (buffers, stats)", 1
        )[0]
        self.assertIn("buffers: buffers.clone()", cache_hit)
        self.assertIn("stats: WgpuUiDrawBufferStats::default()", cache_hit)

    def test_capacity_growth_is_bounded_and_profiled(self) -> None:
        source = self.source()
        manifest = PROFILE_MANIFEST.read_text(encoding="utf-8")
        gate = PROFILE_GATE.read_text(encoding="utf-8")

        self.assertIn("checked_next_power_of_two()", source)
        self.assertIn("UiVertexBufferUploadAction::ReuseExisting", source)
        self.assertIn("UiVertexBufferUploadAction::RetainExisting", source)
        self.assertIn(
            "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/render_pass.rs",
            manifest,
        )
        self.assertIn("gpu_vertex_buffer_creates", gate)

    def test_pressure_model_keeps_damage_upload_claim_conservative(self) -> None:
        result = run(
            present_count=4096,
            solid_bytes=32_768,
            solid_instance_bytes=4_096,
            image_bytes=8_192,
        )

        self.assertEqual(
            result["retired_per_present_allocation"]["vertex_buffer_creates"],
            12_288,
        )
        self.assertEqual(
            result["persistent_unversioned_damage"]["vertex_buffer_creates"], 3
        )
        self.assertEqual(result["delta"]["damage_avoided_buffer_creates"], 12_285)
        self.assertEqual(
            result["delta"]["damage_buffer_create_reduction_ratio"], 4096.0
        )
        self.assertEqual(result["delta"]["damage_avoided_upload_bytes"], 0)
        self.assertEqual(
            result["persistent_unversioned_damage"]["vertex_upload_bytes"],
            184_549_376,
        )
        self.assertEqual(
            result["versioned_projection_reuse"]["vertex_upload_bytes"], 45_056
        )
        self.assertEqual(
            result["delta"]["versioned_avoided_upload_bytes"], 184_504_320
        )


if __name__ == "__main__":
    unittest.main()

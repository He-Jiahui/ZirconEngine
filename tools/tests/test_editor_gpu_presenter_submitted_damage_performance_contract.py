from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
GPU_PRESENT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/present.rs"
)


def function_body(source: str, signature: str, next_signature: str) -> str:
    return source.split(signature, 1)[1].split(next_signature, 1)[0]


class EditorGpuPresenterSubmittedDamagePerformanceContract(unittest.TestCase):
    def test_diagnostics_follow_built_stream_damage_not_requested_damage(self) -> None:
        source = GPU_PRESENT.read_text(encoding="utf-8")
        body = function_body(
            source,
            "fn present(",
            "fn present_during_native_resize",
        )

        build = body.index("build_chrome_command_stream_with_residency")
        submitted_damage = body.index("let submitted_damage = stream.damage().cloned()")
        move_stream = body.index("ui_surface_draw_list_from_owned_stream_with_residency")
        diagnostics = body.index("submitted_damage.as_ref()")

        self.assertLess(build, submitted_damage)
        self.assertLess(submitted_damage, move_stream)
        self.assertLess(move_stream, diagnostics)
        self.assertIn("let region_present = submitted_damage.is_some();", body)
        self.assertNotIn("let region_present = damage.is_some()", body)


if __name__ == "__main__":
    unittest.main()

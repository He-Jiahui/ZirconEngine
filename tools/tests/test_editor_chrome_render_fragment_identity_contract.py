from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
RECORDED_MODEL = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording/model.rs"
)
RECORDING_STATE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording/state.rs"
)
FRAME_SCOPE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_frame/"
    "recording_frame/source_scope.rs"
)
STREAM_MODEL = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/"
    "stream/model.rs"
)


class EditorChromeRenderFragmentIdentityContractTests(unittest.TestCase):
    def test_recorded_source_uses_a_compact_fragment_ordinal(self) -> None:
        source = RECORDED_MODEL.read_text(encoding="utf-8")

        self.assertIn("fragment_index: u16", source)
        self.assertNotIn("fragment_role: String", source)

    def test_final_recording_assigns_checked_per_source_fragment_ordinals(self) -> None:
        source = RECORDING_STATE.read_text(encoding="utf-8")

        self.assertIn(
            "HashMap<(HostRenderSourceKey, UiRenderFrameCommandRef), u32>", source
        )
        self.assertIn(".entry((surface_key, command_ref))", source)
        self.assertIn("u16::try_from(*next_fragment_index)", source)

    def test_recording_scope_keeps_surface_and_command_identity_atomic(self) -> None:
        state = RECORDING_STATE.read_text(encoding="utf-8")
        scope = FRAME_SCOPE.read_text(encoding="utf-8")

        self.assertIn("current_source_surface: Option<HostRenderSourceKey>", state)
        self.assertIn("current_source_command: Option<UiRenderFrameCommandRef>", state)
        self.assertIn("with_render_source_frame", scope)
        self.assertIn("with_render_source_command", scope)

    def test_stream_resolution_returns_command_and_fragment_identity(self) -> None:
        source = STREAM_MODEL.read_text(encoding="utf-8")
        resolver = source.split("fn resolve_command_source", 1)[1].split(
            "fn image_resource", 1
        )[0]

        self.assertIn("UiRenderFrameCommandRef, u16", resolver)
        self.assertIn("source.fragment_index", resolver)

    def test_runtime_command_resolution_is_borrowed_indexed_and_fail_closed(self) -> None:
        source = STREAM_MODEL.read_text(encoding="utf-8")
        resolver = source.split("fn resolve_runtime_command_source", 1)[1].split(
            "fn image_resource", 1
        )[0]

        self.assertIn("&UiRenderCommand", resolver)
        self.assertIn(".render_extract", resolver)
        self.assertIn(".command_by_ref(source.command_ref)?", resolver)
        self.assertNotIn(".iter()", resolver)
        self.assertNotIn(".position(", resolver)


if __name__ == "__main__":
    unittest.main()

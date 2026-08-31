from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE_TABLE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_frame/"
    "recording/source_table.rs"
)
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
TEMPLATE_COMMANDS = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_nodes/commands.rs"
)
HOST_COMMAND_MODEL = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "render_commands/command/model.rs"
)
PANE_TEMPLATE_PAINT = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/"
    "docks/pane/template_nodes.rs"
)
EXTRACTION = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/"
    "extraction/entry.rs"
)
EXTRACTION_COMMAND = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/"
    "extraction/command.rs"
)
CHROME_COMMAND = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/command.rs"
)
STREAM_MODEL = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/"
    "stream/model.rs"
)


class EditorChromeRenderSourceTableContractTests(unittest.TestCase):
    def test_source_table_uses_compact_keys_and_arc_identity_deduplication(self) -> None:
        source = SOURCE_TABLE.read_text(encoding="utf-8")

        self.assertIn("struct HostRenderSourceKey(", source)
        self.assertIn("host_contract) u32", source)
        self.assertIn("Vec<Arc<UiSurfaceFrame>>", source)
        self.assertIn("HashMap<usize, HostRenderSourceKey>", source)
        self.assertIn("Arc::as_ptr(frame) as usize", source)
        self.assertIn("Arc::ptr_eq", source)
        self.assertIn("u32::try_from(self.frames.len())", source)
        registration = source.split("fn register(", 1)[1].split("fn resolve(", 1)[0]
        self.assertNotIn(".position(", registration)

    def test_recorded_commands_hold_a_key_and_relative_command_ref_only(self) -> None:
        source = RECORDED_MODEL.read_text(encoding="utf-8")

        self.assertIn("struct HostRenderCommandSource", source)
        self.assertIn("surface_key: HostRenderSourceKey", source)
        self.assertIn("command_ref: UiRenderFrameCommandRef", source)
        self.assertIn("pub source: Option<HostRenderCommandSource>", source)
        self.assertNotIn("Arc<UiSurfaceFrame>", source)

    def test_recording_scopes_pair_pane_frame_and_command_identity(self) -> None:
        state = RECORDING_STATE.read_text(encoding="utf-8")
        scope = FRAME_SCOPE.read_text(encoding="utf-8")

        self.assertIn("current_source_surface: Option<HostRenderSourceKey>", state)
        self.assertIn("current_source_command: Option<UiRenderFrameCommandRef>", state)
        self.assertIn(".current_source_surface", state)
        self.assertIn(".zip(self.current_source_command)", state)
        self.assertIn("with_render_source_frame", scope)
        self.assertIn("with_render_source_command", scope)

    def test_template_rows_tag_only_the_commands_they_emit(self) -> None:
        source = TEMPLATE_COMMANDS.read_text(encoding="utf-8")
        host_command = HOST_COMMAND_MODEL.read_text(encoding="utf-8")

        self.assertIn("let command_start = commands.len();", source)
        self.assertIn("commands[command_start..]", source)
        self.assertIn("node.surface_render_command_ref", source)
        self.assertIn("source_render_command_ref:", host_command)
        self.assertIn("Option<UiRenderFrameCommandRef>", host_command)
        self.assertNotIn("Arc<UiSurfaceFrame>", host_command)

    def test_assets_pane_binds_the_source_frame_before_drawing_rows(self) -> None:
        source = PANE_TEMPLATE_PAINT.read_text(encoding="utf-8")

        self.assertIn("pane.assets_activity.render_source_frame.as_ref()", source)
        self.assertIn("frame.with_render_source_frame", source)

    def test_extraction_moves_the_table_into_the_chrome_stream(self) -> None:
        extraction = EXTRACTION.read_text(encoding="utf-8")
        extraction_command = EXTRACTION_COMMAND.read_text(encoding="utf-8")
        chrome_command = CHROME_COMMAND.read_text(encoding="utf-8")
        stream = STREAM_MODEL.read_text(encoding="utf-8")

        self.assertIn("recorded_frame.render_sources", extraction)
        self.assertIn("source: command.source", extraction_command)
        self.assertIn("source: Option<HostRenderCommandSource>", chrome_command)
        self.assertNotIn("Arc<UiSurfaceFrame>", chrome_command)
        self.assertIn("render_sources: HostRenderSourceTable", stream)
        self.assertIn("resolve_command_source", stream)


if __name__ == "__main__":
    unittest.main()

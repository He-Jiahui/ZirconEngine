from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs"
)


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated function: {signature}")


class StreamingHotReloadDiagnosticsPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_restore_error_display_streams_diagnostics(self) -> None:
        display = function_body(
            self.source,
            "impl std::fmt::Display for NativePluginHotReloadError",
        )
        self.assertIn("write_joined_hot_reload_diagnostics", display)
        self.assertNotIn("diagnostics.join", display)

    def test_rollback_diagnostic_uses_one_sized_buffer(self) -> None:
        rollback = function_body(self.source, "pub(super) fn rollback_diagnostic(&self)")
        rollback_error = function_body(self.source, "pub(super) fn rollback_error(")
        self.assertIn("String::with_capacity", rollback)
        self.assertIn("write_joined_hot_reload_diagnostics", rollback)
        self.assertNotIn(".join(", rollback)
        self.assertIn("error.push_str", rollback_error)
        self.assertNotIn("format!", rollback_error)

    def test_rust_regression_preserves_hot_reload_diagnostics(self) -> None:
        self.assertIn(
            "streaming_hot_reload_diagnostics_preserve_contract",
            self.source,
        )
        self.assertIn(
            "plugin physics hot reload failed while restoring runtime state: status 17; first; second",
            self.source,
        )


if __name__ == "__main__":
    unittest.main()

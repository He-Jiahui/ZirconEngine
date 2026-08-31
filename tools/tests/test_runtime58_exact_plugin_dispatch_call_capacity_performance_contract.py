from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
RUNTIME_BEHAVIOR = (
    ROOT
    / "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/runtime_behavior.rs"
)


class Runtime58ExactPluginDispatchCallCapacityPerformanceContractTests(
    unittest.TestCase
):
    def dispatch_body(self) -> str:
        source = RUNTIME_BEHAVIOR.read_text(encoding="utf-8")
        return source.split(
            "pub(super) fn dispatch_runtime_plugin_command_result", 1
        )[1].split("pub fn save_runtime_plugin_state", 1)[0]

    def test_dispatch_calls_use_the_exact_snapshot_count(self) -> None:
        body = self.dispatch_body()
        normalized = " ".join(body.split())

        self.assertIn(
            "collect::<NativePluginRuntimeBehaviorResult<Vec<_>>>()?", normalized
        )
        self.assertIn(
            "let mut calls = Vec::with_capacity(snapshots.len());", normalized
        )
        self.assertNotIn("let mut calls = Vec::new();", body)

    def test_dispatch_still_emits_one_call_and_diagnostics_per_snapshot(self) -> None:
        body = self.dispatch_body()
        normalized = " ".join(body.split())

        self.assertIn("for (plugin_id, snapshot) in snapshots", normalized)
        self.assertIn(
            "diagnostics.extend(report_diagnostics(&plugin_id, command_name, &report));",
            normalized,
        )
        self.assertIn(
            "calls.push(NativePluginRuntimeBehaviorCall { plugin_id, report });",
            normalized,
        )

    def test_play_mode_enter_and_exit_share_the_dispatch_path(self) -> None:
        source = RUNTIME_BEHAVIOR.read_text(encoding="utf-8")

        self.assertEqual(
            source.count("dispatch_runtime_plugin_command_result("), 4
        )
        self.assertIn("NATIVE_RUNTIME_PLAY_MODE_ENTER_COMMAND", source)
        self.assertIn("NATIVE_RUNTIME_PLAY_MODE_EXIT_COMMAND", source)


if __name__ == "__main__":
    unittest.main()

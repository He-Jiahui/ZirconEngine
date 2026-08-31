import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
COUNTER_EVIDENCE_PATH = REPO_ROOT / "tools" / "ui-profile-counter-evidence.ps1"
COUNTER_CATALOG_PATH = (
    REPO_ROOT / "zircon_editor" / "src" / "ui" / "retained_host" / "ui_perf" / "counter_catalog.rs"
)
UI_PERF_PATH = REPO_ROOT / "zircon_editor" / "src" / "ui" / "retained_host" / "ui_perf.rs"
APPLY_PRESENTATION_PATH = (
    REPO_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "retained_host"
    / "ui"
    / "apply_presentation.rs"
)
HOST_STATE_PATH = (
    REPO_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "retained_host"
    / "host_contract"
    / "globals"
    / "state.rs"
)
VIEWPORT_CHROME_PATH = HOST_STATE_PATH.parent / "state" / "viewport_chrome.rs"

STRUCTURE_COUNTERS = {
    "ShellPresentationBuildCount": "shell_presentation_build_count",
    "HostSceneBuildCount": "host_scene_build_count",
    "PaneProjectionBuildCount": "pane_projection_build_count",
    "PresentationStructureGenerationChangeCount": (
        "presentation_structure_generation_change_count"
    ),
}


def powershell_function(source: str, name: str) -> str:
    match = re.search(rf"function\s+{re.escape(name)}\s*\{{", source)
    if match is None:
        raise AssertionError(f"missing PowerShell function: {name}")
    depth = 1
    index = match.end()
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError(f"unterminated PowerShell function: {name}")
    return source[match.end() : index - 1]


class EditorWindowResizeProfileStructureGatePerformanceContractTests(unittest.TestCase):
    def test_resize_gate_rejects_any_full_presentation_rebuild(self):
        source = COUNTER_EVIDENCE_PATH.read_text(encoding="utf-8")
        gate = powershell_function(source, "Test-ZirconWindowResizeCounterGate")

        self.assertIn('"ui.window_resize.presentation_rebuild_count"', gate)
        self.assertIn(
            '$presentationRebuildCount = $counterTotals["ui.window_resize.presentation_rebuild_count"]',
            gate,
        )
        self.assertIn("$presentationRebuildCount -eq 0", gate)

    def test_resize_gate_rejects_each_full_structure_build_stage(self):
        gate = powershell_function(
            COUNTER_EVIDENCE_PATH.read_text(encoding="utf-8"),
            "Test-ZirconWindowResizeCounterGate",
        )

        for counter_name in STRUCTURE_COUNTERS.values():
            qualified_name = f"ui.window_resize.{counter_name}"
            self.assertIn(f'"{qualified_name}"', gate)
            self.assertRegex(
                gate,
                rf"\$\w+\s*=\s*\$counterTotals\[\"{re.escape(qualified_name)}\"\]",
            )
        self.assertGreaterEqual(gate.count("-eq 0"), 11)

    def test_structure_counters_are_mapped_and_emitted_at_their_authorities(self):
        catalog = COUNTER_CATALOG_PATH.read_text(encoding="utf-8")
        ui_perf = UI_PERF_PATH.read_text(encoding="utf-8")
        apply_presentation = APPLY_PRESENTATION_PATH.read_text(encoding="utf-8")
        host_state = HOST_STATE_PATH.read_text(encoding="utf-8")
        viewport_chrome = VIEWPORT_CHROME_PATH.read_text(encoding="utf-8")

        for variant, counter_name in STRUCTURE_COUNTERS.items():
            self.assertIn(f"{variant},", catalog)
            self.assertRegex(
                ui_perf,
                rf"UiPerfCounter::{variant}\s*=>\s*\{{?\s*"
                rf"concat!\(\s*\$prefix,\s*\"\.{counter_name}\"",
            )

        for variant in (
            "ShellPresentationBuildCount",
            "HostSceneBuildCount",
            "PaneProjectionBuildCount",
        ):
            self.assertIn(
                f"record_current_ui_perf_counter(UiPerfCounter::{variant}, 1.0);",
                apply_presentation,
            )

        self.assertIn("fn advance_structure_generation(&mut self)", host_state)
        self.assertIn(
            "UiPerfCounter::PresentationStructureGenerationChangeCount",
            host_state,
        )
        combined_state = host_state + viewport_chrome
        self.assertEqual(combined_state.count("self.advance_structure_generation();"), 6)
        self.assertEqual(
            combined_state.count("self.presentation_structure_generation ="),
            1,
        )


if __name__ == "__main__":
    unittest.main()

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def source(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


def rust_item(text: str, marker: str) -> str:
    start = text.index(marker)
    brace = text.index("{", start)
    depth = 0
    for index in range(brace, len(text)):
        if text[index] == "{":
            depth += 1
        elif text[index] == "}":
            depth -= 1
            if depth == 0:
                return text[start : index + 1]
    raise AssertionError(f"unterminated Rust item: {marker}")


class FloatingWindowProjectionSpatialScaleInstrumentationContract(unittest.TestCase):
    def test_bundle_records_build_and_source_cardinality_before_allocation(self):
        text = source("zircon_editor/src/ui/retained_host/floating_window_projection.rs")
        build = rust_item(
            text,
            "pub(crate) fn build_floating_window_projection_bundle_from_windows_with_shared_source",
        )

        for counter in [
            "ui.floating_projection.bundle_build_count",
            "ui.floating_projection.native_host_row_count",
            "ui.floating_projection.window_row_count",
        ]:
            self.assertIn(counter, build)
        self.assertLess(
            build.index("ui.floating_projection.bundle_build_count"),
            build.index("let mut native_hosts_by_window_id"),
        )

    def test_recompute_records_native_bounds_sync_candidates_before_the_loop(self):
        text = source(
            "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/"
            "floating_projection.rs"
        )
        build = rust_item(
            text, "pub(super) fn build_recompute_floating_window_projection_bundle"
        )

        counter = "ui.floating_projection.bounds_sync_candidate_count"
        self.assertIn(counter, build)
        self.assertLess(build.index(counter), build.index("for (window_index, window)"))

    def test_pointer_patch_records_all_stable_topology_candidate_work(self):
        text = source("zircon_editor/src/ui/retained_host/shell_pointer/drag_surface.rs")
        patch = rust_item(text, "pub(super) fn patch_drag_surface")

        for counter in [
            "ui.shell_drag.geometry_resolve_count",
            "ui.shell_drag.floating_frame_candidate_count",
            "ui.shell_drag.node_candidate_count",
        ]:
            self.assertIn(counter, patch)
        self.assertIn("floating_windows.len().saturating_mul(5)", patch)
        self.assertLess(
            patch.index("ui.shell_drag.node_candidate_count"),
            patch.index("let base_nodes_missing"),
        )

    def test_pointer_patch_distinguishes_topology_miss_from_geometry_reuse(self):
        text = source("zircon_editor/src/ui/retained_host/shell_pointer/drag_surface.rs")
        patch = rust_item(text, "pub(super) fn patch_drag_surface")

        topology = "ui.shell_drag.topology_miss_count"
        reuse = "ui.shell_drag.geometry_reuse_count"
        self.assertIn(topology, patch)
        self.assertIn(reuse, patch)
        self.assertLess(patch.index(topology), patch.index("return None;"))
        self.assertIn("if !changed", patch)
        self.assertGreater(patch.index(reuse), patch.index("if !changed"))


if __name__ == "__main__":
    unittest.main()

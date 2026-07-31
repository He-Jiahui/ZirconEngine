from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def function_body(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class EditorUiAssetWorkspaceWatcherPerformanceContractTests(unittest.TestCase):
    def test_path_classification_does_not_allocate_a_matching_roots_vector(self) -> None:
        source = (
            ROOT
            / "zircon_editor/src/ui/host/asset_editor_sessions/watcher/path_identity.rs"
        ).read_text(encoding="utf-8")
        body = function_body(
            source,
            "pub(super) fn asset_id_for_watched_path(",
            "\n}",
        )

        self.assertNotIn("collect::<Vec<_>>()", body)
        self.assertIn("matching_roots.next()?", body)
        self.assertIn("matching_roots.next().is_some()", body)

    def test_poll_refresh_queues_the_reported_change_batch_without_sync_io(self) -> None:
        source = (
            ROOT
            / "zircon_editor/src/ui/host/asset_editor_sessions/watcher/host.rs"
        ).read_text(encoding="utf-8")
        body = function_body(
            source,
            "    pub fn poll_ui_asset_workspace_watcher(",
            "\n    }\n}",
        )

        self.assertIn(".enqueue(report.changed_asset_ids.iter().cloned());", body)
        self.assertIn("self.start_next_ui_asset_refresh()?;", body)
        self.assertNotIn("refresh_ui_asset_workspace_for_changes", body)
        self.assertNotIn("report.changed_asset_ids.clone()", body)


if __name__ == "__main__":
    unittest.main()

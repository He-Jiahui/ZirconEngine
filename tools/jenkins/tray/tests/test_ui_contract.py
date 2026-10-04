"""Contract tests for tray behaviour that can run without Explorer or Jenkins."""
from __future__ import annotations

import sys
import tempfile
import types
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).parents[3]))

from tools.jenkins.tray.app import TrayApp, TRAY_MESSAGE
from tools.jenkins.tray import native_windows as nw


class Config:
    def __init__(self, root: Path):
        self.repo_root = root
        self.pilot_root = root
        self.state_dir = root / "state"


class UiContractTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(dir="D:\\cargo-targets") if Path("D:\\").exists() else tempfile.TemporaryDirectory()
        self.app = TrayApp(Config(Path(self.tmp.name)))

    def tearDown(self):
        self.tmp.cleanup()

    def test_exit_does_not_submit_stop_worker(self):
        self.app.hwnd = 1
        with patch.object(self.app, "_work") as work, patch("tools.jenkins.tray.app.nw.user32.DestroyWindow"):
            self.app._command(1008)
        work.assert_not_called()
        self.assertTrue(self.app.closed)

    def test_stop_preview_no_cancels_without_mutation(self):
        self.app.stop_preview = {"activeBuilds": [{"job": "demo", "number": 7}], "rootIdentity": {"root": "owned"}}
        with patch("tools.jenkins.tray.app.nw.message_box", return_value=0), patch.object(self.app, "_mutate") as mutate:
            self.app._confirm_stop(self.app.stop_preview)
        mutate.assert_not_called()

    def test_duplicate_mutation_click_is_ignored(self):
        self.app.operation = True
        with patch.object(self.app, "_work") as work:
            self.app._mutate("start")
        work.assert_not_called()
        self.assertTrue(self.app.operation)

    def test_background_status_result_is_projected(self):
        fake = types.SimpleNamespace(run_worker=lambda config, action, **kwargs: {"schemaVersion": 1, "state": "ready", "url": "http://127.0.0.1:5555"}, read_operation=lambda config: None)
        with patch.dict(sys.modules, {"tools.jenkins.tray.operations": fake}):
            self.app._refresh()
        self.app.events.put(("status", {"schemaVersion": 1, "state": "ready", "url": "http://127.0.0.1:5555"}))
        with patch.object(self.app, "_paint"):
            self.app._consume_events()
        self.assertEqual(self.app.status["state"], "ready")

    def test_v4_callback_uses_loword_for_double_click(self):
        with patch("tools.jenkins.tray.app.nw.IS_WINDOWS", False), patch.object(self.app, "_open") as opened:
            # High word contains the icon id in Shell_NotifyIcon v4; low word
            # is the mouse event that the window must dispatch.
            self.app._window_proc(0, TRAY_MESSAGE, 0, (1 << 16) | nw.WM_LBUTTONDBLCLK)
        opened.assert_called_once_with()


if __name__ == "__main__":
    unittest.main()

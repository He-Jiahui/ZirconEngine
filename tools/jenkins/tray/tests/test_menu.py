import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parents[3]))

from tools.jenkins.tray.menu import MenuState, label_for, menu_items


class MenuTests(unittest.TestCase):
  def test_status_labels_and_commands_are_chinese(self):
    self.assertEqual(label_for({"state": "busy"}), "运行任务")
    items = menu_items(MenuState({"state": "ready", "url": "http://127.0.0.1:5555", "canStop": True}))
    labels = [label for _, label, _ in items]
    self.assertIn("打开 Jenkins", labels)
    self.assertIn("停止 Jenkins", labels)


  def test_stale_status_disables_open_and_stop(self):
    state = MenuState({"state": "degraded", "message": "identity mismatch"})
    self.assertFalse(state.enabled("open"))
    self.assertFalse(state.enabled("stop"))

  def test_control_plane_service_can_open_while_builds_are_paused(self):
    state = MenuState({"state": "degraded", "serviceReady": True,
                       "url": "http://127.0.0.1:18080/"})
    self.assertTrue(state.enabled("open"))
    self.assertIn("构建暂停", __import__("tools.jenkins.tray.menu", fromlist=["label_for"]).label_for(state.status))


  def test_running_operation_serializes_mutations(self):
    state = MenuState({"state": "ready", "canStart": True, "canStop": True}, operation=True)
    self.assertFalse(state.enabled("start"))
    self.assertFalse(state.enabled("stop"))


if __name__ == "__main__":
    unittest.main()

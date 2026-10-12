from __future__ import annotations
import unittest
from pathlib import Path
from tools.jenkins.tray.formal_app import FormalTray
from tools.jenkins.tray.menu import COMMANDS

class FormalCommandTests(unittest.TestCase):
    def test_boot_recovery_error_explains_the_operator_action(self):
        from tools.jenkins.contracts import JenkinsError
        tray = FormalTray.__new__(FormalTray)
        tray.config = type('C', (), {'url': 'http://127.0.0.1:53748/'})()
        result = tray._operation_error(JenkinsError('deployment_termination_unproven', 'unproven'))
        self.assertEqual('deployment_termination_unproven', result['reasonCode'])
        self.assertIn('终止证据不足', result['message'])
        self.assertNotIn('重启 Windows', result['message'])

    def test_degraded_owned_controller_can_stop(self):
        tray = FormalTray.__new__(FormalTray)
        tray.status = {'state': 'degraded', 'ownerKnown': True, 'canStop': True}
        tray.last_command = None; tray._telemetry = lambda: None
        tray._mutate = lambda function: setattr(tray, 'stopped_with', function)
        tray.command(COMMANDS['stop'])
        self.assertIsNotNone(tray.stopped_with)

    def test_restart_requires_known_owner(self):
        tray = FormalTray.__new__(FormalTray); tray.status={'state':'ready','ownerKnown':False,'canStop':False}; tray.last_command=None; tray._telemetry=lambda:None
        tray.command(COMMANDS['restart'])
        self.assertIsNone(tray.last_command)

    def test_command_telemetry_records_id(self):
        tray = FormalTray.__new__(FormalTray); tray.status={'state':'ready','ownerKnown':True,'canStop':True}; tray.last_command=None; tray._telemetry=lambda:None; tray._restart=lambda:None
        tray.command(COMMANDS['restart'])
        self.assertEqual(tray.last_command['commandId'], COMMANDS['restart'])

    def test_telemetry_uses_transactional_state_pointer(self):
        from unittest.mock import patch
        import tools.jenkins.tray.formal_app as app
        tray = FormalTray.__new__(FormalTray)
        tray.config = type('C', (), {'paths': type('P', (), {'state': Path('E:/cargo-targets/zircon-ui-test/state')})(),
                                     'url': 'http://127.0.0.1:53748/'})()
        tray.status = {'state': 'ready', 'serviceReady': True}
        tray.hwnd = 7; tray.icon_added = False; tray.last_command = None
        with patch.object(app, 'atomic_json') as write, \
             patch('tools.jenkins.tray.formal_native.identity', return_value=None):
            tray._telemetry()
        write.assert_called_once()
        self.assertEqual(write.call_args.args[0], tray.config.paths.state / 'tray' / 'ui.json')

    def test_icon_retry_stops_after_six_gui_attempts(self):
        from unittest.mock import patch
        import tools.jenkins.tray.formal_app as app
        tray = FormalTray.__new__(FormalTray)
        tray.hwnd = 7; tray.icon = None; tray._reset_icon_retry()
        with patch.object(app.nw, 'add_icon', return_value=False) as add, \
             patch.object(tray, '_telemetry'):
            for _ in range(app.ICON_RETRY_LIMIT + 2):
                tray._attempt_icon_add()
        self.assertEqual(add.call_count, app.ICON_RETRY_LIMIT)
        self.assertFalse(tray.icon_added)

    def test_popup_creation_and_append_failures_are_typed(self):
        from unittest.mock import patch
        import ctypes
        import tools.jenkins.tray.native_windows as nw

        class FakeUser32:
            def CreatePopupMenu(self): return 0

        with patch.object(nw, 'IS_WINDOWS', True), patch.object(nw, 'user32', FakeUser32()):
            ctypes.set_last_error(5)
            with self.assertRaises(OSError):
                nw.popup_menu(7, [], 0, 0)

        class FakeAppendUser32:
            def CreatePopupMenu(self): return 1
            def AppendMenuW(self, *_args): return 0
            def DestroyMenu(self, _menu): self.destroyed = True; return 1

        fake = FakeAppendUser32()
        with patch.object(nw, 'IS_WINDOWS', True), patch.object(nw, 'user32', fake):
            ctypes.set_last_error(5)
            with self.assertRaises(OSError):
                nw.popup_menu(7, [(1, 'one', True)], 0, 0)
        self.assertTrue(fake.destroyed)

    def test_tray_callback_decodes_v4_event_from_low_word(self):
        from unittest.mock import patch
        import tools.jenkins.tray.formal_app as app
        tray = FormalTray.__new__(FormalTray)
        tray.status = {'state': 'ready', 'ownerKnown': True, 'canStop': True}
        tray.config = type('C', (), {'url': 'http://127.0.0.1:53748/'})()
        tray.closed = False
        tray.command = lambda command: setattr(tray, 'selected', command)
        with patch.object(app.nw, 'popup_menu', return_value=COMMANDS['status']) as popup, \
             patch.object(app.nw, 'cursor_position', return_value=(10, 20)):
            callback = None
            # Capture the callback passed to the native window factory.
            with patch.object(app.nw, 'create_hidden_window', side_effect=lambda c, t, cb: (1, cb)), \
                 patch.object(app.nw, 'status_icon', return_value=None), \
                 patch.object(app.nw, 'add_icon', return_value=True), \
                 patch.object(app.nw, 'remove_icon'), \
                 patch.object(app.nw, 'update_icon'), \
                 patch.object(app.nw.user32, 'DestroyWindow'):
                tray._single_instance = lambda: True
                tray._poll = lambda: None
                tray._telemetry = lambda: None
                with patch.object(app.nw, 'IS_WINDOWS', True), patch.object(tray, '_poll'):
                    # Avoid entering the loop; invoke the callback through a
                    # captured factory directly instead.
                    callback = (lambda hwnd, message, wparam, lparam: None)
            def factory(_class, _title, cb):
                nonlocal callback
                callback = cb
                return 1, cb
            with patch.object(app.nw, 'create_hidden_window', side_effect=factory), \
                 patch.object(app.nw, 'status_icon', return_value=None), \
                 patch.object(app.nw, 'add_icon', return_value=True), \
                 patch.object(app.nw, 'remove_icon'), \
                 patch.object(app.nw, 'update_icon'), \
                 patch.object(app.nw, 'IS_WINDOWS', True), \
                 patch.object(app.nw.user32, 'DestroyWindow'), \
                 patch.object(tray, '_poll'):
                # The run loop is bounded by setting closed after callback.
                tray.closed = True
                tray.run()
            callback(1, app.TRAY_MESSAGE, 0, (1 << 16) | app.nw.WM_RBUTTONUP)
            popup.assert_called_once()

if __name__ == '__main__': unittest.main()

from __future__ import annotations
import unittest
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

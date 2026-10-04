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
        self.assertIn('Windows', result['message'])

    def test_restart_requires_known_owner(self):
        tray = FormalTray.__new__(FormalTray); tray.status={'state':'ready','ownerKnown':False,'canStop':False}; tray.last_command=None; tray._telemetry=lambda:None
        tray.command(COMMANDS['restart'])
        self.assertIsNone(tray.last_command)

    def test_command_telemetry_records_id(self):
        tray = FormalTray.__new__(FormalTray); tray.status={'state':'ready','ownerKnown':True,'canStop':True}; tray.last_command=None; tray._telemetry=lambda:None; tray._restart=lambda:None
        tray.command(COMMANDS['restart'])
        self.assertEqual(tray.last_command['commandId'], COMMANDS['restart'])

if __name__ == '__main__': unittest.main()

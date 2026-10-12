"""The persistent owner may detach; managed native children remain contained."""
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment import manager


class SupervisorLifetimeTests(unittest.TestCase):
    def test_windows_supervisor_requests_explicit_breakaway_without_changing_native_jobs(self):
        flags = SimpleNamespace(CREATE_NEW_PROCESS_GROUP=0x200, CREATE_NO_WINDOW=0x8000000,
                                CREATE_BREAKAWAY_FROM_JOB=0x1000000)
        with patch.object(manager, 'os', SimpleNamespace(name='nt')), patch.object(manager, 'subprocess', flags):
            result = manager._supervisor_creation_flags()
        self.assertEqual(0x9000200, result)

    def test_missing_windows_breakaway_support_is_an_error(self):
        for value in (None, 0):
            flags = SimpleNamespace(CREATE_NO_WINDOW=0x8000000, CREATE_BREAKAWAY_FROM_JOB=value)
            with self.subTest(value=value), patch.object(manager, 'os', SimpleNamespace(name='nt')), \
                    patch.object(manager, 'subprocess', flags), self.assertRaises(JenkinsError) as error:
                manager._supervisor_creation_flags()
            self.assertEqual('supervisor_breakaway_unavailable', error.exception.code)


if __name__ == '__main__':
    unittest.main()

"""Exact PID reuse may be proved through restricted query-only handles."""
import ctypes
from ctypes import wintypes
import os
import unittest
from unittest.mock import Mock, patch

from tools.jenkins.pilot.native import process_identity as native
from tools.jenkins.pilot.governance_recovery import _require_dead


class RestrictedProcessIdentityTests(unittest.TestCase):
    @unittest.skipUnless(os.name == "nt", "requires actual Windows query-only handle")
    def test_real_query_only_handle_reads_birth_but_cannot_wait(self):
        api = native._kernel()
        handle = api.OpenProcess(0x1000, False, os.getpid())
        self.assertTrue(handle)
        try:
            self.assertEqual(native._birth(api, handle), native.process_creation_time(os.getpid()))
            with self.assertRaises(OSError):
                native._alive(api, handle)
        finally:
            api.CloseHandle(handle)

    def api(self, *, fallback_handle=42, birth=200, fallback_error=5):
        api = Mock()
        errors = []
        def open_process(access, inherit, pid):
            self.assertFalse(inherit)
            self.assertEqual(pid, 123)
            if access == 0x1000 | 0x100000:
                errors.append(5)
                return None
            self.assertEqual(access, 0x1000)
            errors.append(fallback_error)
            return fallback_handle
        def times(handle, created, exited, kernel, user):
            self.assertEqual(handle, fallback_handle)
            value = ctypes.cast(created, ctypes.POINTER(wintypes.FILETIME)).contents
            value.dwLowDateTime = birth
            value.dwHighDateTime = 0
            return True
        api.OpenProcess.side_effect = open_process
        api.GetProcessTimes.side_effect = times
        api.WaitForSingleObject.return_value = 0xFFFFFFFF
        api.CloseHandle.return_value = True
        return api, errors

    def probe(self, api, errors):
        from contextlib import ExitStack
        stack = ExitStack()
        stack.enter_context(patch.object(native, "_kernel", return_value=api))
        stack.enter_context(patch.object(native.ctypes, "get_last_error", side_effect=lambda: errors[-1], create=True))
        return stack

    def test_query_only_different_birth_proves_reuse_without_waiting(self):
        api, errors = self.api()
        with self.probe(api, errors):
            self.assertFalse(native.process_matches_creation_time(123, "100"))
            proof = _require_dead(123, "100")
            native.wait_for_process_exit(123, "100")
            self.assertEqual(native.process_creation_time(123), "200")
        self.assertEqual(proof["proof"], "native-pid-reused")
        self.assertEqual(proof["observedCreationTime"], "200")
        api.WaitForSingleObject.assert_not_called()
        self.assertEqual(api.CloseHandle.call_count, 4)

    def test_query_only_same_birth_cannot_prove_liveness_or_death(self):
        for operation in (lambda: native.process_matches_creation_time(123, "200"),
                          lambda: _require_dead(123, "200"),
                          lambda: native.wait_for_process_exit(123, "200"),
                          lambda: native.process_is_alive(123)):
            with self.subTest(operation=operation):
                api, errors = self.api()
                with self.probe(api, errors), self.assertRaises(OSError):
                    operation()
                api.WaitForSingleObject.assert_called_once()
                api.CloseHandle.assert_called_once_with(42)

    def test_both_denied_and_unknown_query_errors_remain_uncertain(self):
        for error in (5, 6, 299):
            api, errors = self.api(fallback_handle=None, fallback_error=error)
            with self.probe(api, errors), self.assertRaises(OSError):
                _require_dead(123, "100")
            api.WaitForSingleObject.assert_not_called()
            api.CloseHandle.assert_not_called()

    def test_pid_disappearing_before_query_fallback_is_exact_absence(self):
        api, errors = self.api(fallback_handle=None, fallback_error=87)
        with self.probe(api, errors):
            self.assertEqual(_require_dead(123, "100")["proof"], "native-pid-absent")
        api.CloseHandle.assert_not_called()

    def test_birth_query_failure_still_closes_retained_handle(self):
        api, errors = self.api()
        api.GetProcessTimes.side_effect = None
        api.GetProcessTimes.return_value = False
        with self.probe(api, errors), self.assertRaises(OSError):
            _require_dead(123, "100")
        api.CloseHandle.assert_called_once_with(42)
        api.WaitForSingleObject.assert_not_called()

    def test_other_initial_open_errors_do_not_trigger_less_privileged_retry(self):
        api = Mock()
        api.OpenProcess.return_value = None
        with patch.object(native, "_kernel", return_value=api), \
             patch.object(native.ctypes, "get_last_error", return_value=6, create=True), \
             self.assertRaises(OSError):
            _require_dead(123, "100")
        api.OpenProcess.assert_called_once_with(0x1000 | 0x100000, False, 123)


if __name__ == "__main__":
    unittest.main()

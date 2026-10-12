"""Pure native ABI/evidence controls; never invoke a real boot query."""
import ctypes
import unittest
from unittest.mock import Mock, patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.processes import boot


def sample(**changes):
    return {"status": 0, "returnedLength": 48, "bootFileTime": 134358454705000000,
            "currentFileTime": 134360000000000000, **changes}


class NativeBootObservationTests(unittest.TestCase):
    def test_stable_repeated_native_boot_ignores_legacy_wmi_success(self):
        with patch.object(boot, "_query_time_of_day", side_effect=[sample(), sample(currentFileTime=134360000000000001)]) as query, \
             patch.object(boot.subprocess, "run", side_effect=AssertionError("WMI must not run")):
            self.assertEqual(boot.windows_boot_filetime(), 134358454705000000)
            self.assertEqual(query.call_count, 2)

    def test_unknown_status_length_types_dates_and_changed_boot_fail_closed(self):
        for bad in (sample(status=5), sample(status=-1), sample(status=True), sample(returnedLength=47),
                    sample(returnedLength=49), sample(returnedLength=False), sample(bootFileTime=0),
                    sample(bootFileTime="100"), sample(bootFileTime=-1), sample(currentFileTime=1),
                    sample(currentFileTime=2**63), {}, {**sample(), "extra": True}):
            with self.subTest(bad=bad), patch.object(boot, "_query_time_of_day", side_effect=[sample(), bad]), self.assertRaises(JenkinsError):
                boot.windows_boot_filetime()
        with patch.object(boot, "_query_time_of_day", side_effect=[sample(), sample(bootFileTime=134358454705000001)]), self.assertRaises(JenkinsError):
            boot.windows_boot_filetime()

    def test_native_buffer_matches_actual_typed_nt_contract(self):
        def query(info_class, buffer, size, returned):
            self.assertEqual((info_class, size), (3, 48))
            value = ctypes.cast(buffer, ctypes.POINTER(boot._TimeOfDay)).contents
            value.BootTime = 134358454705000000; value.CurrentTime = 134360000000000000
            ctypes.cast(returned, ctypes.POINTER(ctypes.c_uint32)).contents.value = 48
            return 0
        native_query = Mock(side_effect=query)
        with patch.object(boot.os, "name", "nt"), patch.object(boot.ctypes, "WinDLL", return_value=Mock(NtQuerySystemInformation=native_query)) as load:
            self.assertEqual(boot.windows_boot_filetime(), 134358454705000000)
        load.assert_called_with("ntdll.dll")
        self.assertIs(native_query.restype, ctypes.c_int32)
        self.assertEqual(native_query.argtypes, [ctypes.c_int32, ctypes.c_void_p, ctypes.c_uint32, ctypes.POINTER(ctypes.c_uint32)])
        self.assertEqual(native_query.call_count, 2)

    def test_unknown_layout_platform_or_loader_never_falls_back_to_wmi(self):
        with patch.object(boot.os, "name", "nt"), patch.object(boot.ctypes, "sizeof", return_value=40), \
             patch.object(boot.ctypes, "WinDLL") as load, patch.object(boot.subprocess, "run") as legacy:
            with self.assertRaises(JenkinsError): boot.windows_boot_filetime()
            load.assert_not_called(); legacy.assert_not_called()
        with patch.object(boot.os, "name", "nt"), patch.object(boot.ctypes, "WinDLL", side_effect=OSError("unavailable")), \
             patch.object(boot.subprocess, "run") as legacy:
            with self.assertRaises(JenkinsError): boot.windows_boot_filetime()
            legacy.assert_not_called()
        with patch.object(boot.os, "name", "posix"), patch.object(boot.ctypes, "WinDLL") as load, self.assertRaises(JenkinsError):
            boot.windows_boot_filetime()
        load.assert_not_called()

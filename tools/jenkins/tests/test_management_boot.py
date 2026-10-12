"""Targeted tests for the bounded Windows Event Log management boot stamp."""
from __future__ import annotations

import ctypes
from ctypes import wintypes
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError
import tools.jenkins.deployment.management_boot as management_boot


_EVENT_XML = """<Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event">
  <System>
    <Provider Name="Microsoft-Windows-Kernel-General" Guid="{A68CA8B7-004F-D7B6-A698-07E2DE0F1F5D}" />
    <EventID>12</EventID>
    <Channel>System</Channel>
    <EventRecordID>125749</EventRecordID>
    <TimeCreated SystemTime="2026-10-04T03:47:45.7941411Z" />
  </System>
  <EventData>
    <Data Name="StartTime">2026-10-04T03:47:45.5000000Z</Data>
    <Data Name="BootMode">0</Data>
  </EventData>
</Event>"""


class _FakeFunction:
    def __init__(self, callback):
        self.callback = callback
        self.calls = []
        self.argtypes = None
        self.restype = None

    def __call__(self, *args):
        self.calls.append(args)
        return self.callback(*args)


class _FakeApi:
    def __init__(self, *, xml=_EVENT_XML, query_result=123, next_error=None,
                 render_error=None):
        self.xml = xml
        self.query_result = query_result
        self.next_error = next_error
        self.render_error = render_error
        self.closed = []
        self.EvtQuery = _FakeFunction(self._query)
        self.EvtNext = _FakeFunction(self._next)
        self.EvtRender = _FakeFunction(self._render)
        self.EvtClose = _FakeFunction(self._close)

    def _query(self, *_args):
        if not self.query_result:
            ctypes.set_last_error(5)
        return self.query_result

    def _next(self, _query, _count, events, _timeout, _flags, returned):
        if self.next_error is not None:
            ctypes.set_last_error(self.next_error)
            ctypes.cast(returned, ctypes.POINTER(wintypes.DWORD))[0] = 0
            return 0
        ctypes.cast(events, ctypes.POINTER(wintypes.HANDLE))[0] = 456
        ctypes.cast(returned, ctypes.POINTER(wintypes.DWORD))[0] = 1
        return 1

    def _render(self, _context, _event, _flags, _size, buffer, used, _value_count):
        if self.render_error is not None:
            ctypes.set_last_error(self.render_error)
            return 0
        required = len(self.xml.encode("utf-16-le")) + 2
        if buffer is None:
            ctypes.cast(used, ctypes.POINTER(wintypes.DWORD))[0] = required
            ctypes.set_last_error(122)
            return 0
        buffer.value = self.xml
        ctypes.cast(used, ctypes.POINTER(wintypes.DWORD))[0] = required
        return 1

    def _close(self, handle):
        self.closed.append(handle)
        return 1


class ManagementBootStampTests(unittest.TestCase):
    def _observe(self, api, *, platform="nt"):
        with patch.object(management_boot.os, "name", platform), \
             patch.object(management_boot.ctypes, "WinDLL", return_value=api):
            return management_boot.observe_management_boot_stamp()

    def test_native_event_is_returned_with_exact_string_shape(self):
        api = _FakeApi()
        self.assertEqual(
            {
                "provider": "Microsoft-Windows-Kernel-General",
                "eventId": "12",
                "recordId": "125749",
                "timeCreated": "2026-10-04T03:47:45.7941411Z",
                "startTime": "2026-10-04T03:47:45.5000000Z",
                "bootMode": "0",
                "providerGuid": "{a68ca8b7-004f-d7b6-a698-07e2de0f1f5d}",
                "channel": "System",
            },
            self._observe(api),
        )
        self.assertEqual(2, len(api.closed))
        self.assertEqual(
            (None, "System", management_boot._QUERY, 0x201),
            api.EvtQuery.calls[0],
        )
        self.assertEqual(1, api.EvtNext.calls[0][1])
        self.assertEqual(1, api.EvtRender.calls[0][2])
        self.assertEqual(1, api.EvtRender.calls[1][2])

    def test_non_windows_is_closed_error_without_loading_wevtapi(self):
        with patch.object(management_boot.os, "name", "posix"), \
             patch.object(management_boot.ctypes, "WinDLL") as load:
            with self.assertRaises(JenkinsError) as caught:
                management_boot.observe_management_boot_stamp()
        self.assertEqual("management_boot_unavailable", caught.exception.code)
        load.assert_not_called()

    def test_query_denial_and_empty_result_are_unavailable_and_closed(self):
        for api in (_FakeApi(query_result=0), _FakeApi(next_error=259)):
            with self.subTest(api=api):
                with self.assertRaises(JenkinsError) as caught:
                    self._observe(api)
                self.assertEqual("management_boot_unavailable", caught.exception.code)

    def test_render_denial_is_unavailable_and_closes_both_handles(self):
        api = _FakeApi(render_error=5)
        with self.assertRaises(JenkinsError) as caught:
            self._observe(api)
        self.assertEqual("management_boot_unavailable", caught.exception.code)
        self.assertEqual(2, len(api.closed))

    def test_invalid_provider_guid_and_timestamp_are_rejected(self):
        cases = (
            (_EVENT_XML.replace("Kernel-General", "Kernel-Wrong"), "provider"),
            (_EVENT_XML.replace("A68CA8B7", "00000000"), "guid"),
            (_EVENT_XML.replace("Channel>System", "Channel>Other"), "channel"),
            (_EVENT_XML.replace("EventID>12", "EventID>11"), "event id"),
            (_EVENT_XML.replace("EventRecordID>125749", "EventRecordID>0"), "record"),
            (_EVENT_XML.replace("03:47:45.7941411Z", "03:47:45"), "time"),
            (_EVENT_XML.replace("03:47:45.5000000Z", "2999-01-01T00:00:00.0000000Z"), "future"),
        )
        for xml, label in cases:
            with self.subTest(label=label):
                with self.assertRaises(JenkinsError) as caught:
                    self._observe(_FakeApi(xml=xml))
                self.assertEqual("management_boot_invalid", caught.exception.code)

    def test_render_size_limit_is_closed_error(self):
        api = _FakeApi()
        with patch.object(management_boot, "_MAX_RENDER_BYTES", 1):
            with self.assertRaises(JenkinsError) as caught:
                self._observe(api)
        self.assertEqual("management_boot_unavailable", caught.exception.code)
        self.assertEqual(2, len(api.closed))

    def test_api_exception_is_closed_error(self):
        api = _FakeApi()
        api.EvtQuery = _FakeFunction(lambda *_args: (_ for _ in ()).throw(OSError("denied")))
        with self.assertRaises(JenkinsError) as caught:
            self._observe(api)
        self.assertEqual("management_boot_unavailable", caught.exception.code)


if __name__ == "__main__":
    unittest.main()

"""Bounded native Windows Event Log observation for management recovery.

The deployment recovery path needs a same-boot identity for the management
controller.  This module deliberately reads one well-defined System event and
does not write state, inspect WMI, or alter the historical build recovery
boundary.
"""
from __future__ import annotations

import ctypes
from ctypes import wintypes
from datetime import datetime, timezone
import os
import xml.etree.ElementTree as ET

from ..contracts import JenkinsError


_QUERY = "*[System[(EventID=12) and Provider[@Name='Microsoft-Windows-Kernel-General']]]"
_QUERY_FLAGS = 0x201  # EvtQueryChannelPath | EvtQueryReverseDirection
_RENDER_EVENT_XML = 1  # EvtRenderEventXml
_ERROR_ACCESS_DENIED = 5
_ERROR_INSUFFICIENT_BUFFER = 122
_ERROR_NO_MORE_ITEMS = 259
_MAX_RENDER_BYTES = 16 * 1024 * 1024
_PROVIDER = "Microsoft-Windows-Kernel-General"
_PROVIDER_GUID = "{a68ca8b7-004f-d7b6-a698-07e2de0f1f5d}"
_CHANNEL = "System"
_EVENT_ID = "12"
_UTC = timezone.utc
_NS = {"e": "http://schemas.microsoft.com/win/2004/08/events/event"}
_FIELDS = (
    "provider",
    "eventId",
    "recordId",
    "timeCreated",
    "startTime",
    "bootMode",
    "providerGuid",
    "channel",
)


def _unavailable(stage: str, error: int | None = None) -> JenkinsError:
    details = {"stage": stage}
    if error is not None:
        details["winError"] = int(error)
    return JenkinsError(
        "management_boot_unavailable",
        f"Windows Event Log boot stamp is unavailable ({stage})",
        retryable=True,
        details=details,
    )


def _invalid(message: str) -> JenkinsError:
    return JenkinsError("management_boot_invalid", message, retryable=False)


def _last_error() -> int:
    try:
        return int(ctypes.get_last_error())
    except (AttributeError, OSError, ValueError):
        return 0


def _configure_api(api: object) -> None:
    """Set the native signatures before calling any wevtapi entry point."""
    try:
        api.EvtQuery.argtypes = [
            wintypes.HANDLE,
            wintypes.LPCWSTR,
            wintypes.LPCWSTR,
            wintypes.DWORD,
        ]
        api.EvtQuery.restype = wintypes.HANDLE
        api.EvtNext.argtypes = [
            wintypes.HANDLE,
            wintypes.DWORD,
            ctypes.POINTER(wintypes.HANDLE),
            wintypes.DWORD,
            wintypes.DWORD,
            ctypes.POINTER(wintypes.DWORD),
        ]
        api.EvtNext.restype = wintypes.BOOL
        api.EvtRender.argtypes = [
            wintypes.HANDLE,
            wintypes.HANDLE,
            wintypes.DWORD,
            wintypes.DWORD,
            ctypes.c_void_p,
            ctypes.POINTER(wintypes.DWORD),
            ctypes.POINTER(wintypes.DWORD),
        ]
        api.EvtRender.restype = wintypes.BOOL
        api.EvtClose.argtypes = [wintypes.HANDLE]
        api.EvtClose.restype = wintypes.BOOL
    except Exception as error:
        raise _unavailable("api-signature") from error


def _close(api: object, handle: object) -> None:
    if not handle:
        return
    try:
        api.EvtClose(handle)
    except Exception:
        # Cleanup must not mask the query/render error.  EvtClose is still
        # attempted for every non-null handle returned by wevtapi.
        pass


def _parse_utc(value: object, name: str) -> str:
    if not isinstance(value, str) or not value:
        raise _invalid(f"event {name} is missing")
    raw = value.strip()
    if raw != value:
        raise _invalid(f"event {name} contains surrounding whitespace")
    # Event XML uses RFC 3339's trailing Z.  Accept an explicit zero offset as
    # well, while retaining the original precision and spelling in the result.
    candidate = raw[:-1] + "+00:00" if raw.endswith(("Z", "z")) else raw
    try:
        parsed = datetime.fromisoformat(candidate)
    except (TypeError, ValueError) as error:
        raise _invalid(f"event {name} is not an ISO-8601 timestamp") from error
    if parsed.tzinfo is None or parsed.utcoffset() != _UTC.utcoffset(None):
        raise _invalid(f"event {name} is not UTC")
    if parsed > datetime.now(_UTC):
        raise _invalid(f"event {name} is in the future")
    return raw


def _text(parent: ET.Element | None, path: str, name: str) -> str:
    if parent is None:
        raise _invalid("event System section is missing")
    value = parent.findtext(path, namespaces=_NS)
    if value is None or not isinstance(value, str) or not value:
        raise _invalid(f"event {name} is missing")
    return value


def _extract_stamp(xml: str) -> dict[str, str]:
    try:
        node = ET.fromstring(xml)
    except (ET.ParseError, TypeError, ValueError) as error:
        raise _invalid("EvtRender returned invalid event XML") from error

    system = node.find("e:System", _NS)
    if system is None:
        raise _invalid("event System section is missing")
    provider_node = system.find("e:Provider", _NS)
    if provider_node is None:
        raise _invalid("event Provider section is missing")
    provider = provider_node.attrib.get("Name")
    provider_guid = provider_node.attrib.get("Guid")
    event_id = _text(system, "e:EventID", "eventId")
    channel = _text(system, "e:Channel", "channel")
    record_id = _text(system, "e:EventRecordID", "recordId")
    time_created_node = system.find("e:TimeCreated", _NS)
    time_created = time_created_node.attrib.get("SystemTime") if time_created_node is not None else None
    data = {
        child.attrib.get("Name"): child.text
        for child in node.findall("e:EventData/e:Data", _NS)
        if child.attrib.get("Name")
    }
    start_time = data.get("StartTime")
    boot_mode = data.get("BootMode")

    if provider != _PROVIDER:
        raise _invalid("event provider does not match Kernel-General")
    if not isinstance(provider_guid, str) or provider_guid.casefold() != _PROVIDER_GUID:
        raise _invalid("event provider GUID does not match Kernel-General")
    if channel != _CHANNEL or event_id != _EVENT_ID:
        raise _invalid("event channel or event ID does not match boot event")
    if (not record_id or any(char < "0" or char > "9" for char in record_id)
            or int(record_id) <= 0):
        raise _invalid("event record ID is not a positive integer")
    if (not isinstance(boot_mode, str) or not boot_mode
            or any(char < "0" or char > "9" for char in boot_mode)):
        raise _invalid("event BootMode is missing or invalid")

    stamp = {
        "provider": provider,
        "eventId": event_id,
        "recordId": record_id,
        "timeCreated": _parse_utc(time_created, "timeCreated"),
        "startTime": _parse_utc(start_time, "startTime"),
        "bootMode": boot_mode,
        # The provider GUID is an identity, so expose its canonical case while
        # keeping timestamp strings byte-for-byte as rendered by Windows.
        "providerGuid": provider_guid.casefold(),
        "channel": channel,
    }
    if tuple(stamp) != _FIELDS or any(type(value) is not str for value in stamp.values()):
        raise _invalid("event stamp shape is invalid")
    return stamp


def observe_management_boot_stamp() -> dict[str, str]:
    """Read and validate the newest Kernel-General Event ID 12 boot stamp.

    The function is intentionally bounded to the native Windows Event Log
    API.  Every query, render, denied-access, no-event, and non-Windows failure
    is reported as a closed :class:`JenkinsError`; no fallback to WMI or the
    historical ``windows_boot_filetime`` observation is permitted.
    """
    if os.name != "nt":
        raise _unavailable("non-windows")

    try:
        api = ctypes.WinDLL("wevtapi", use_last_error=True)
        _configure_api(api)
    except JenkinsError:
        raise
    except Exception as error:
        raise _unavailable("api-load") from error

    query = None
    event = wintypes.HANDLE()
    try:
        try:
            query = api.EvtQuery(None, "System", _QUERY, _QUERY_FLAGS)
        except Exception as error:
            raise _unavailable("query-call") from error
        if not query:
            error = _last_error()
            if error == _ERROR_ACCESS_DENIED:
                raise _unavailable("query-denied", error)
            raise _unavailable("query", error)

        returned = wintypes.DWORD()
        try:
            next_ok = api.EvtNext(query, 1, ctypes.byref(event), 0, 0, ctypes.byref(returned))
        except Exception as error:
            raise _unavailable("next-call") from error
        if not next_ok or returned.value != 1 or not event:
            error = _last_error()
            if error == _ERROR_NO_MORE_ITEMS or returned.value == 0:
                raise _unavailable("no-event", error or _ERROR_NO_MORE_ITEMS)
            if error == _ERROR_ACCESS_DENIED:
                raise _unavailable("next-denied", error)
            raise _unavailable("next", error)

        used = wintypes.DWORD()
        rendered_count = wintypes.DWORD()
        try:
            first_ok = api.EvtRender(
                None,
                event,
                _RENDER_EVENT_XML,
                0,
                None,
                ctypes.byref(used),
                ctypes.byref(rendered_count),
            )
        except Exception as error:
            raise _unavailable("render-size-call") from error
        error = _last_error()
        if first_ok or error != _ERROR_INSUFFICIENT_BUFFER or used.value <= 0:
            if error == _ERROR_ACCESS_DENIED:
                raise _unavailable("render-size-denied", error)
            raise _unavailable("render-size", error)

        if used.value <= 0 or used.value > _MAX_RENDER_BYTES:
            raise _unavailable('render-size-limit', used.value)
        wchar_bytes = ctypes.sizeof(ctypes.c_wchar)
        if wchar_bytes <= 0:
            raise _unavailable('render-buffer')
        try:
            buffer = ctypes.create_unicode_buffer(used.value // wchar_bytes + 1)
        except (MemoryError, ValueError, OverflowError) as error:
            raise _unavailable('render-buffer') from error
        try:
            second_ok = api.EvtRender(
                None,
                event,
                _RENDER_EVENT_XML,
                ctypes.sizeof(buffer),
                buffer,
                ctypes.byref(used),
                ctypes.byref(rendered_count),
            )
        except Exception as error:
            raise _unavailable("render-call") from error
        if not second_ok:
            error = _last_error()
            if error == _ERROR_ACCESS_DENIED:
                raise _unavailable("render-denied", error)
            raise _unavailable("render", error)
        try:
            xml = buffer.value
        except (AttributeError, ValueError) as error:
            raise _unavailable("render-buffer") from error
        return _extract_stamp(xml)
    finally:
        _close(api, event)
        _close(api, query)


__all__ = ["observe_management_boot_stamp"]

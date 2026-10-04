"""Small ctypes-only Win32 surface used by the Jenkins tray.

The module intentionally has no dependency on the retired coordinator tray.
It is importable on non-Windows hosts so menu and status tests can run there.
"""
from __future__ import annotations

import ctypes
import os
from urllib.parse import urlsplit
from ctypes import wintypes

IS_WINDOWS = os.name == "nt"

if IS_WINDOWS:
    user32 = ctypes.WinDLL("user32", use_last_error=True)
    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    shell32 = ctypes.WinDLL("shell32", use_last_error=True)
    kernel32.GetModuleHandleW.argtypes = [wintypes.LPCWSTR]; kernel32.GetModuleHandleW.restype = wintypes.HMODULE
    user32.CreateWindowExW.restype = wintypes.HWND
    user32.CreateWindowExW.argtypes = [wintypes.DWORD,wintypes.LPCWSTR,wintypes.LPCWSTR,wintypes.DWORD,ctypes.c_int,ctypes.c_int,ctypes.c_int,ctypes.c_int,wintypes.HWND,wintypes.HMENU,wintypes.HINSTANCE,ctypes.c_void_p]
    user32.RegisterWindowMessageW.argtypes = [wintypes.LPCWSTR]; user32.RegisterWindowMessageW.restype = wintypes.UINT
    user32.PostMessageW.argtypes = [wintypes.HWND,wintypes.UINT,wintypes.WPARAM,wintypes.LPARAM]; user32.PostMessageW.restype = wintypes.BOOL
    user32.GetMessageW.argtypes = [ctypes.POINTER(wintypes.MSG),wintypes.HWND,wintypes.UINT,wintypes.UINT]; user32.GetMessageW.restype = wintypes.BOOL
    user32.RegisterClassW.argtypes = [ctypes.c_void_p]; user32.RegisterClassW.restype = wintypes.ATOM
    user32.DefWindowProcW.restype = wintypes.LRESULT if hasattr(wintypes, "LRESULT") else ctypes.c_ssize_t
    user32.DefWindowProcW.argtypes = [wintypes.HWND, wintypes.UINT, wintypes.WPARAM, wintypes.LPARAM]
    user32.TranslateMessage.argtypes = [ctypes.POINTER(wintypes.MSG)]
    user32.DispatchMessageW.argtypes = [ctypes.POINTER(wintypes.MSG)]
    user32.DispatchMessageW.restype = ctypes.c_ssize_t
    user32.GetCursorPos.argtypes = [ctypes.c_void_p]
    user32.DestroyWindow.argtypes = [wintypes.HWND]; user32.DestroyWindow.restype = wintypes.BOOL
    user32.PostQuitMessage.argtypes = [ctypes.c_int]
    user32.LoadIconW.argtypes = [wintypes.HINSTANCE, wintypes.LPCWSTR]; user32.LoadIconW.restype = wintypes.HICON
    user32.MessageBoxW.argtypes = [wintypes.HWND,wintypes.LPCWSTR,wintypes.LPCWSTR,wintypes.UINT]; user32.MessageBoxW.restype = ctypes.c_int
    user32.CreatePopupMenu.restype = wintypes.HMENU
    user32.AppendMenuW.argtypes = [wintypes.HMENU,wintypes.UINT,ctypes.c_size_t,wintypes.LPCWSTR]; user32.AppendMenuW.restype = wintypes.BOOL
    user32.TrackPopupMenu.argtypes = [wintypes.HMENU,wintypes.UINT,ctypes.c_int,ctypes.c_int,ctypes.c_int,wintypes.HWND,ctypes.c_void_p]; user32.TrackPopupMenu.restype = wintypes.BOOL
    user32.DestroyMenu.argtypes = [wintypes.HMENU]; user32.DestroyMenu.restype = wintypes.BOOL
    user32.SetForegroundWindow.argtypes = [wintypes.HWND]; user32.SetForegroundWindow.restype = wintypes.BOOL
    shell32.Shell_NotifyIconW.argtypes = [wintypes.DWORD, ctypes.c_void_p]; shell32.Shell_NotifyIconW.restype = wintypes.BOOL
    shell32.ShellExecuteW.restype = wintypes.HINSTANCE
    shell32.ShellExecuteW.argtypes = [wintypes.HWND, wintypes.LPCWSTR, wintypes.LPCWSTR,
                                     wintypes.LPCWSTR, wintypes.LPCWSTR, ctypes.c_int]
    kernel32.CreateMutexW.argtypes = [ctypes.c_void_p, wintypes.BOOL, wintypes.LPCWSTR]
    kernel32.CreateMutexW.restype = wintypes.HANDLE
    kernel32.CloseHandle.argtypes = [wintypes.HANDLE]
    user32.CreateIcon.argtypes = [wintypes.HINSTANCE, ctypes.c_int, ctypes.c_int, wintypes.BYTE,
                                 wintypes.BYTE, ctypes.c_void_p, ctypes.c_void_p]
    user32.CreateIcon.restype = wintypes.HICON
    user32.DestroyIcon.argtypes = [wintypes.HICON]
else:  # pragma: no cover
    user32 = kernel32 = shell32 = None

WM_APP = 0x8000
WM_COMMAND = 0x0111
WM_DESTROY = 0x0002
WM_LBUTTONDBLCLK = 0x0203
WM_RBUTTONUP = 0x0205
WM_CONTEXTMENU = 0x007b
WM_USER = 0x0400
NIM_ADD, NIM_MODIFY, NIM_DELETE, NIM_SETVERSION = 0, 1, 2, 4
NIF_MESSAGE, NIF_ICON, NIF_TIP, NIF_INFO = 1, 2, 4, 0x10
NOTIFYICON_VERSION_4 = 4
IDI_APPLICATION = 32512
MB_OK, MB_YESNO, MB_ICONINFORMATION, MB_ICONWARNING, MB_ICONERROR = 0, 4, 0x40, 0x30, 0x10
IDYES = 6
SW_SHOWNORMAL = 1


if IS_WINDOWS:
    class POINT(ctypes.Structure):
        _fields_ = [("x", wintypes.LONG), ("y", wintypes.LONG)]

    class NOTIFYICONDATAW(ctypes.Structure):
        _fields_ = [
            ("cbSize", wintypes.DWORD), ("hWnd", wintypes.HWND), ("uID", wintypes.UINT),
            ("uFlags", wintypes.UINT), ("uCallbackMessage", wintypes.UINT),
            ("hIcon", wintypes.HICON), ("szTip", wintypes.WCHAR * 128),
            ("dwState", wintypes.DWORD), ("dwStateMask", wintypes.DWORD),
            ("szInfo", wintypes.WCHAR * 256), ("uVersionOrTimeout", wintypes.UINT),
            ("szInfoTitle", wintypes.WCHAR * 64), ("dwInfoFlags", wintypes.DWORD),
            ("guidItem", ctypes.c_byte * 16), ("hBalloonIcon", wintypes.HICON),
        ]

    WNDPROC = ctypes.WINFUNCTYPE(ctypes.c_ssize_t, wintypes.HWND, wintypes.UINT, wintypes.WPARAM, wintypes.LPARAM)


def message_box(hwnd, text: str, title: str, flags: int = MB_OK) -> int:
    if not IS_WINDOWS:
        return IDYES if flags & MB_YESNO else 1
    return int(user32.MessageBoxW(hwnd, text, title, flags))


def open_url(url: str) -> bool:
    try:
        parsed = urlsplit(url)
        valid = (parsed.scheme == "http" and parsed.hostname == "127.0.0.1" and parsed.port
                 and not parsed.username and not parsed.password and not parsed.query
                 and not parsed.fragment and parsed.path in ("", "/"))
    except ValueError:
        valid = False
    if not valid:
        return False
    if not IS_WINDOWS:
        return False
    return int(shell32.ShellExecuteW(None, "open", url, None, None, SW_SHOWNORMAL) or 0) > 32


def popup_menu(hwnd, items: list[tuple[int, str, bool]], x: int, y: int) -> int:
    if not IS_WINDOWS:
        return 0
    menu = user32.CreatePopupMenu()
    try:
        for command, label, enabled in items:
            state = 0 if enabled else 1  # MF_GRAYED
            user32.AppendMenuW(menu, 0x0000 | state, command, label)
        user32.SetForegroundWindow(hwnd)
        selected = user32.TrackPopupMenu(menu, 0x0100 | 0x0002, x, y, 0, hwnd, None)
        user32.PostMessageW(hwnd, WM_NULL if 'WM_NULL' in globals() else 0, 0, 0)
        return int(selected)
    finally:
        user32.DestroyMenu(menu)


def cursor_position() -> tuple[int, int]:
    if not IS_WINDOWS:
        return 0, 0
    point = POINT()
    user32.GetCursorPos(ctypes.byref(point))
    return int(point.x), int(point.y)


def create_hidden_window(class_name: str, title: str, callback):
    """Create a top-level hidden window and return (hwnd, callback_ref)."""
    if not IS_WINDOWS:
        return 0, callback
    instance = kernel32.GetModuleHandleW(None)
    class_def = type("WNDCLASSW", (ctypes.Structure,), {"_fields_": [
        ("style", wintypes.UINT), ("lpfnWndProc", WNDPROC), ("cbClsExtra", ctypes.c_int),
        ("cbWndExtra", ctypes.c_int), ("hInstance", wintypes.HINSTANCE),
        ("hIcon", wintypes.HICON), ("hCursor", wintypes.HCURSOR),
        ("hbrBackground", wintypes.HBRUSH), ("lpszMenuName", wintypes.LPCWSTR),
        ("lpszClassName", wintypes.LPCWSTR),]})
    proc = WNDPROC(callback)
    icon_id = ctypes.cast(ctypes.c_void_p(IDI_APPLICATION), wintypes.LPCWSTR)
    wc = class_def(0, proc, 0, 0, instance, user32.LoadIconW(None, icon_id), None, None, None, class_name)
    if not user32.RegisterClassW(ctypes.byref(wc)):
        raise ctypes.WinError(ctypes.get_last_error())
    hwnd = user32.CreateWindowExW(0, class_name, title, 0, 0, 0, 0, 0, None, None, instance, None)
    if not hwnd:
        raise ctypes.WinError(ctypes.get_last_error())
    return hwnd, proc


def add_icon(hwnd, icon_id: int, callback_message: int, tooltip: str, icon=None) -> bool:
    if not IS_WINDOWS:
        return True
    data = NOTIFYICONDATAW()
    data.cbSize = ctypes.sizeof(data); data.hWnd = hwnd; data.uID = icon_id
    data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP; data.uCallbackMessage = callback_message
    resource = ctypes.cast(ctypes.c_void_p(IDI_APPLICATION), wintypes.LPCWSTR)
    data.hIcon = icon or user32.LoadIconW(None, resource); data.szTip = tooltip[:127]
    data.uVersionOrTimeout = NOTIFYICON_VERSION_4
    return bool(shell32.Shell_NotifyIconW(NIM_ADD, ctypes.byref(data)) and shell32.Shell_NotifyIconW(NIM_SETVERSION, ctypes.byref(data)))


def update_icon(hwnd, icon_id: int, tooltip: str, icon=None) -> bool:
    data = NOTIFYICONDATAW()
    data.cbSize = ctypes.sizeof(data)
    data.hWnd, data.uID = hwnd, icon_id
    data.uFlags, data.szTip = NIF_TIP, tooltip[:127]
    if icon:
        data.uFlags |= NIF_ICON
        data.hIcon = icon
    return bool(shell32.Shell_NotifyIconW(NIM_MODIFY, ctypes.byref(data)))


def status_icon(state: str):
    """A small native icon with both J glyph and state color."""
    colors = {"ready": (45, 160, 85), "busy": (45, 105, 210), "stopped": (120, 125, 135),
              "starting": (220, 160, 35), "stopping": (220, 160, 35)}
    red, green, blue = colors.get(state, (205, 65, 65))
    size = 16
    pixels = bytearray(size * size * 4)
    mask = bytearray(size * size // 8)
    for y in range(size):
        for x in range(size):
            index = y * size + x
            if (x - 7.5) ** 2 + (y - 7.5) ** 2 > 55:
                mask[index // 8] |= 1 << (7 - index % 8)
                continue
            glyph = ((y in (4, 5) and 6 <= x <= 11) or (9 <= x <= 10 and 5 <= y <= 10)
                     or (y in (10, 11) and 5 <= x <= 9) or (x in (4, 5) and y in (8, 9)))
            pixels[index * 4:index * 4 + 4] = bytes((255, 255, 255, 255) if glyph else (blue, green, red, 255))
    and_bits = (ctypes.c_ubyte * len(mask)).from_buffer_copy(mask)
    xor_bits = (ctypes.c_ubyte * len(pixels)).from_buffer_copy(pixels)
    icon = user32.CreateIcon(kernel32.GetModuleHandleW(None), size, size, 1, 32, and_bits, xor_bits)
    if not icon:
        raise ctypes.WinError(ctypes.get_last_error())
    return icon


def remove_icon(hwnd, icon_id: int) -> None:
    if not IS_WINDOWS: return
    data = NOTIFYICONDATAW(); data.cbSize = ctypes.sizeof(data); data.hWnd = hwnd; data.uID = icon_id
    shell32.Shell_NotifyIconW(NIM_DELETE, ctypes.byref(data))


def run_message_loop() -> None:
    if not IS_WINDOWS: return
    msg = wintypes.MSG()
    while True:
        result = user32.GetMessageW(ctypes.byref(msg), None, 0, 0)
        if result == -1:
            raise ctypes.WinError(ctypes.get_last_error())
        if result == 0:
            return
        user32.TranslateMessage(ctypes.byref(msg)); user32.DispatchMessageW(ctypes.byref(msg))

"""Capture one X11 drawable without changing focus or capturing other windows."""

from __future__ import annotations

import ctypes
import ctypes.util
from pathlib import Path

from PIL import Image


class XImage(ctypes.Structure):
    _fields_ = [
        ("width", ctypes.c_int), ("height", ctypes.c_int),
        ("xoffset", ctypes.c_int), ("format", ctypes.c_int),
        ("data", ctypes.c_void_p), ("byte_order", ctypes.c_int),
        ("bitmap_unit", ctypes.c_int), ("bitmap_bit_order", ctypes.c_int),
        ("bitmap_pad", ctypes.c_int), ("depth", ctypes.c_int),
        ("bytes_per_line", ctypes.c_int), ("bits_per_pixel", ctypes.c_int),
        ("red_mask", ctypes.c_ulong), ("green_mask", ctypes.c_ulong),
        ("blue_mask", ctypes.c_ulong),
    ]


def capture_window(window: str, output: Path, display_name: str) -> None:
    library = ctypes.util.find_library("X11")
    if library is None:
        raise RuntimeError("X11 capture requires libX11")
    x11 = ctypes.CDLL(library)
    x11.XOpenDisplay.argtypes = [ctypes.c_char_p]
    x11.XOpenDisplay.restype = ctypes.c_void_p
    x11.XGetGeometry.argtypes = [ctypes.c_void_p, ctypes.c_ulong,
        ctypes.POINTER(ctypes.c_ulong), ctypes.POINTER(ctypes.c_int),
        ctypes.POINTER(ctypes.c_int), *([ctypes.POINTER(ctypes.c_uint)] * 4)]
    x11.XGetImage.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_int,
        ctypes.c_int, ctypes.c_uint, ctypes.c_uint, ctypes.c_ulong, ctypes.c_int]
    x11.XGetImage.restype = ctypes.POINTER(XImage)
    x11.XDestroyImage.argtypes = [ctypes.POINTER(XImage)]
    x11.XCloseDisplay.argtypes = [ctypes.c_void_p]
    display = x11.XOpenDisplay(display_name.encode())
    if not display:
        raise RuntimeError(f"cannot open X11 display {display_name}")
    native = None
    try:
        root, x, y = ctypes.c_ulong(), ctypes.c_int(), ctypes.c_int()
        width, height, border, depth = [ctypes.c_uint() for _ in range(4)]
        if not x11.XGetGeometry(display, int(window, 0), ctypes.byref(root),
                ctypes.byref(x), ctypes.byref(y), ctypes.byref(width),
                ctypes.byref(height), ctypes.byref(border), ctypes.byref(depth)):
            raise RuntimeError("cannot query the owned X11 window")
        native = x11.XGetImage(display, int(window, 0), 0, 0, width.value,
                              height.value, ctypes.c_ulong(-1).value, 2)
        if not native:
            raise RuntimeError("cannot capture the owned X11 window")
        frame = native.contents
        if (frame.bits_per_pixel, frame.byte_order, frame.red_mask,
                frame.green_mask, frame.blue_mask) != (32, 0, 0xFF0000, 0xFF00, 0xFF):
            raise RuntimeError("unsupported X11 pixel format")
        pixels = ctypes.string_at(frame.data, frame.bytes_per_line * frame.height)
        Image.frombytes("RGB", (frame.width, frame.height), pixels, "raw",
                        "BGRX", frame.bytes_per_line, 1).save(output)
    finally:
        if native:
            x11.XDestroyImage(native)
        x11.XCloseDisplay(display)

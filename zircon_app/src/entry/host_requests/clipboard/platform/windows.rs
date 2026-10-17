//! 目标窗口线程上执行 Win32 Unicode 剪贴板事务。
//! 打开、锁定和全局内存拥有权由局部 guard 收束；只有完成移交后剪贴板才拥有写入内存。

use std::{mem::size_of, ptr};

use windows_sys::Win32::Foundation::{
    GetLastError, GlobalFree, ERROR_ACCESS_DENIED, HGLOBAL, HWND,
};
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
};
use windows_sys::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};
use windows_sys::Win32::System::Ole::CF_UNICODETEXT;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::Window;
use zircon_runtime_interface::ui::dispatch::UiClipboardTransferFailure;
use zircon_runtime_interface::ZR_RUNTIME_CLIPBOARD_TEXT_MAX_ENCODED_BYTES_V1;

/// 仅在有目标 Win32 窗口时读 Unicode 文本；将平台内存限制为 ABI 可接收的字节预算。
pub(super) fn read_text(window: Option<&dyn Window>) -> Result<String, UiClipboardTransferFailure> {
    let _clipboard = ClipboardGuard::open(clipboard_owner(window)?)?;
    // SAFETY: the clipboard is open on this event-loop thread and the returned handle remains
    // owned by the system for the lifetime of the guard.
    let handle = unsafe { GetClipboardData(u32::from(CF_UNICODETEXT)) };
    if handle.is_null() {
        return Err(UiClipboardTransferFailure::ContentUnavailable);
    }
    // SAFETY: GetClipboardData(CF_UNICODETEXT) returns a movable global-memory handle.
    let byte_len = unsafe { GlobalSize(handle) };
    if byte_len < size_of::<u16>() {
        return Err(UiClipboardTransferFailure::ContentUnavailable);
    }
    // SAFETY: the clipboard guard keeps the handle valid while LockedGlobal exposes its bytes.
    let locked = unsafe { LockedGlobal::lock(handle)? };
    let available_units = byte_len / size_of::<u16>();
    let inspected_units =
        available_units.min(ZR_RUNTIME_CLIPBOARD_TEXT_MAX_ENCODED_BYTES_V1.saturating_add(1));
    // SAFETY: GlobalSize bounds the locked allocation and inspected_units never exceeds it.
    let units =
        unsafe { std::slice::from_raw_parts(locked.pointer.cast::<u16>(), inspected_units) };
    let Some(terminator) = units.iter().position(|unit| *unit == 0) else {
        return Err(
            if available_units > ZR_RUNTIME_CLIPBOARD_TEXT_MAX_ENCODED_BYTES_V1 {
                UiClipboardTransferFailure::PayloadTooLarge
            } else {
                UiClipboardTransferFailure::ContentUnavailable
            },
        );
    };
    let text = String::from_utf16(&units[..terminator])
        .map_err(|_| UiClipboardTransferFailure::ContentUnavailable)?;
    if text.len() > ZR_RUNTIME_CLIPBOARD_TEXT_MAX_ENCODED_BYTES_V1 {
        return Err(UiClipboardTransferFailure::PayloadTooLarge);
    }
    Ok(text)
}

/// 以当前窗口 HWND 打开剪贴板，成功移交全局内存后才报告完成。
pub(super) fn write_text(
    window: Option<&dyn Window>,
    text: &str,
) -> Result<(), UiClipboardTransferFailure> {
    if text.len() > ZR_RUNTIME_CLIPBOARD_TEXT_MAX_ENCODED_BYTES_V1 {
        return Err(UiClipboardTransferFailure::PayloadTooLarge);
    }
    let mut units = text.encode_utf16().collect::<Vec<_>>();
    units.push(0);
    let byte_len = units
        .len()
        .checked_mul(size_of::<u16>())
        .ok_or(UiClipboardTransferFailure::PayloadTooLarge)?;
    let _clipboard = ClipboardGuard::open(clipboard_owner(window)?)?;
    let mut memory = prepare_before_empty(
        || {
            // SAFETY: byte_len is checked above and GMEM_MOVEABLE is required by SetClipboardData.
            let handle = unsafe { GlobalAlloc(GMEM_MOVEABLE, byte_len) };
            if handle.is_null() {
                return Err(UiClipboardTransferFailure::Unavailable);
            }
            let memory = OwnedGlobal::new(handle);
            // SAFETY: handle was allocated above and remains owned by memory until transfer succeeds.
            let destination = unsafe { GlobalLock(handle) }.cast::<u16>();
            if destination.is_null() {
                return Err(UiClipboardTransferFailure::Unavailable);
            }
            // SAFETY: destination spans byte_len bytes and units contains exactly byte_len bytes.
            unsafe { ptr::copy_nonoverlapping(units.as_ptr(), destination, units.len()) };
            // SAFETY: destination came from GlobalLock for this handle.
            unsafe { GlobalUnlock(handle) };
            Ok(memory)
        },
        || {
            // SAFETY: the clipboard is open and EmptyClipboard has no pointer preconditions.
            if unsafe { EmptyClipboard() } == 0 {
                Err(last_clipboard_failure())
            } else {
                Ok(())
            }
        },
    )?;
    publish_owned_global(
        &mut memory,
        |memory| {
            // BUG: [CR-APP-ENTRY-0012] SetClipboardData can still fail after EmptyClipboard has destroyed the old formats; Win32 has no atomic rollback here.
            // SAFETY: the clipboard is open, emptied, and handle is a GMEM_MOVEABLE allocation.
            if unsafe { SetClipboardData(u32::from(CF_UNICODETEXT), memory.handle) }.is_null() {
                Err(last_clipboard_failure())
            } else {
                Ok(())
            }
        },
        OwnedGlobal::release_to_clipboard,
    )
}

fn prepare_before_empty<T>(
    prepare: impl FnOnce() -> Result<T, UiClipboardTransferFailure>,
    empty: impl FnOnce() -> Result<(), UiClipboardTransferFailure>,
) -> Result<T, UiClipboardTransferFailure> {
    let prepared = prepare()?;
    empty()?;
    Ok(prepared)
}

fn publish_owned_global<T>(
    memory: &mut T,
    publish: impl FnOnce(&mut T) -> Result<(), UiClipboardTransferFailure>,
    transfer: impl FnOnce(&mut T),
) -> Result<(), UiClipboardTransferFailure> {
    publish(memory)?;
    transfer(memory);
    Ok(())
}

struct ClipboardGuard;

impl ClipboardGuard {
    fn open(owner: HWND) -> Result<Self, UiClipboardTransferFailure> {
        // SAFETY: owner is the live winit Win32 window for the target runtime viewport.
        if unsafe { OpenClipboard(owner) } == 0 {
            return Err(last_clipboard_failure());
        }
        Ok(Self)
    }
}

fn clipboard_owner(window: Option<&dyn Window>) -> Result<HWND, UiClipboardTransferFailure> {
    let window = window.ok_or(UiClipboardTransferFailure::HostDisconnected)?;
    let raw = window
        .window_handle()
        .map_err(|_| UiClipboardTransferFailure::HostDisconnected)?
        .as_raw();
    let RawWindowHandle::Win32(window) = raw else {
        return Err(UiClipboardTransferFailure::Unsupported);
    };
    Ok(window.hwnd.get() as usize as HWND)
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        // SAFETY: the guard exists only after OpenClipboard succeeds.
        unsafe { CloseClipboard() };
    }
}

struct LockedGlobal {
    handle: HGLOBAL,
    pointer: *mut core::ffi::c_void,
}

impl LockedGlobal {
    unsafe fn lock(handle: HGLOBAL) -> Result<Self, UiClipboardTransferFailure> {
        // SAFETY: the caller supplies a live global-memory handle from GetClipboardData.
        let pointer = unsafe { GlobalLock(handle) };
        if pointer.is_null() {
            return Err(UiClipboardTransferFailure::ContentUnavailable);
        }
        Ok(Self { handle, pointer })
    }
}

impl Drop for LockedGlobal {
    fn drop(&mut self) {
        // SAFETY: pointer was returned by GlobalLock for this handle.
        unsafe { GlobalUnlock(self.handle) };
    }
}

struct OwnedGlobal {
    handle: HGLOBAL,
}

impl OwnedGlobal {
    const fn new(handle: HGLOBAL) -> Self {
        Self { handle }
    }

    fn release_to_clipboard(&mut self) {
        self.handle = ptr::null_mut();
    }
}

impl Drop for OwnedGlobal {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            // SAFETY: non-null handles remain owned here only when SetClipboardData did not accept them.
            unsafe { GlobalFree(self.handle) };
        }
    }
}

fn last_clipboard_failure() -> UiClipboardTransferFailure {
    // SAFETY: GetLastError has no preconditions and is read immediately after a failed Win32 call.
    match unsafe { GetLastError() } {
        ERROR_ACCESS_DENIED => UiClipboardTransferFailure::PermissionDenied,
        _ => UiClipboardTransferFailure::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::{prepare_before_empty, publish_owned_global};
    use std::cell::RefCell;
    use zircon_runtime_interface::ui::dispatch::UiClipboardTransferFailure;

    #[test]
    fn clipboard_is_not_emptied_until_global_payload_is_prepared() {
        let calls = RefCell::new(Vec::new());
        let result = prepare_before_empty(
            || {
                calls.borrow_mut().push("allocate");
                calls.borrow_mut().push("lock");
                calls.borrow_mut().push("copy");
                Ok(())
            },
            || {
                calls.borrow_mut().push("empty");
                Ok(())
            },
        );
        assert_eq!(result, Ok(()));
        assert_eq!(*calls.borrow(), ["allocate", "lock", "copy", "empty"]);

        calls.borrow_mut().clear();
        let result: Result<(), UiClipboardTransferFailure> = prepare_before_empty(
            || {
                calls.borrow_mut().push("allocate");
                Err(UiClipboardTransferFailure::Unavailable)
            },
            || {
                calls.borrow_mut().push("empty");
                Ok(())
            },
        );
        assert_eq!(result, Err(UiClipboardTransferFailure::Unavailable));
        assert_eq!(*calls.borrow(), ["allocate"]);

        calls.borrow_mut().clear();
        let result: Result<(), UiClipboardTransferFailure> = prepare_before_empty(
            || {
                calls.borrow_mut().push("allocate");
                calls.borrow_mut().push("lock");
                Err(UiClipboardTransferFailure::Unavailable)
            },
            || {
                calls.borrow_mut().push("empty");
                Ok(())
            },
        );
        assert_eq!(result, Err(UiClipboardTransferFailure::Unavailable));
        assert_eq!(*calls.borrow(), ["allocate", "lock"]);
    }

    #[test]
    fn global_memory_ownership_transfers_only_after_publish_succeeds() {
        #[derive(Default)]
        struct Ownership(bool);

        let mut ownership = Ownership(true);
        let failed = publish_owned_global(
            &mut ownership,
            |_| Err(UiClipboardTransferFailure::Unavailable),
            |ownership| ownership.0 = false,
        );
        assert_eq!(failed, Err(UiClipboardTransferFailure::Unavailable));
        assert!(ownership.0);

        let succeeded =
            publish_owned_global(&mut ownership, |_| Ok(()), |ownership| ownership.0 = false);
        assert_eq!(succeeded, Ok(()));
        assert!(!ownership.0);
    }
}

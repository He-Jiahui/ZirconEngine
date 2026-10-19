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

/// 未能安全快照任何旧格式时不清空；发布失败逐项恢复，恢复本身仍可能失败。
pub(super) fn write_text(
    window: Option<&dyn Window>,
    text: &str,
) -> Result<(), UiClipboardTransferFailure> {
    if text.len() > ZR_RUNTIME_CLIPBOARD_TEXT_MAX_ENCODED_BYTES_V1 {
        return Err(UiClipboardTransferFailure::PayloadTooLarge);
    }
    let mut units = text.encode_utf16().collect::<Vec<_>>();
    units.push(0);
    write_clipboard_transaction(&mut NativeClipboard, clipboard_owner(window)?, &units)
}

// Bounded logical payload, not a bound on GDI/OS allocator overhead. A single
// GetClipboardData delayed-render call can still occupy Win32's 30-second wait;
// the deadline stops subsequent work and never pretends to cancel that call.
const SNAPSHOT_MAX_FORMATS: usize = 128;
const SNAPSHOT_MAX_BYTES: usize = 16 * 1024 * 1024;
const SNAPSHOT_DEADLINE: std::time::Duration = std::time::Duration::from_secs(2);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SnapshotKind {
    Global,
    Bitmap,
    Palette,
    EnhancedMetafile,
    MetafilePicture,
}

fn snapshot_kind(format: u32) -> Result<SnapshotKind, UiClipboardTransferFailure> {
    use windows_sys::Win32::System::Ole::*;
    match u16::try_from(format).ok() {
        Some(
            CF_TEXT | CF_OEMTEXT | CF_UNICODETEXT | CF_LOCALE | CF_DIB | CF_DIBV5 | CF_HDROP
            | CF_DSPTEXT | CF_SYLK | CF_DIF | CF_TIFF | CF_RIFF | CF_WAVE,
        ) => Ok(SnapshotKind::Global),
        Some(CF_BITMAP | CF_DSPBITMAP) => Ok(SnapshotKind::Bitmap),
        Some(CF_PALETTE) => Ok(SnapshotKind::Palette),
        Some(CF_ENHMETAFILE | CF_DSPENHMETAFILE) => Ok(SnapshotKind::EnhancedMetafile),
        Some(CF_METAFILEPICT | CF_DSPMETAFILEPICT) => Ok(SnapshotKind::MetafilePicture),
        // Registered/private/owner-display formats may carry owner-dependent
        // state; a successful byte copy would not prove an independent backup.
        _ => Err(UiClipboardTransferFailure::Unsupported),
    }
}

struct OwnedSnapshot {
    format: u32,
    handle: windows_sys::Win32::Foundation::HANDLE,
    kind: SnapshotKind,
    metafile: windows_sys::Win32::Graphics::Gdi::HMETAFILE,
}

impl OwnedSnapshot {
    fn release(&mut self) {
        self.handle = ptr::null_mut();
        self.metafile = ptr::null_mut();
    }
}

impl Drop for OwnedSnapshot {
    fn drop(&mut self) {
        if self.handle.is_null() && self.metafile.is_null() {
            return;
        }
        use windows_sys::Win32::Graphics::Gdi::*;
        // SAFETY: only independently cloned or exclusively allocated handles
        // enter this owner; successful SetClipboardData clears the handle.
        unsafe {
            if !self.metafile.is_null() {
                DeleteMetaFile(self.metafile);
            }
            if self.handle.is_null() {
                return;
            }
            match self.kind {
                SnapshotKind::Global => {
                    GlobalFree(self.handle);
                }
                SnapshotKind::Bitmap | SnapshotKind::Palette => {
                    DeleteObject(self.handle);
                }
                SnapshotKind::EnhancedMetafile => {
                    DeleteEnhMetaFile(self.handle);
                }
                SnapshotKind::MetafilePicture => {
                    GlobalFree(self.handle);
                }
            }
        }
    }
}

fn prepare_unicode(units: &[u16]) -> Result<OwnedGlobal, UiClipboardTransferFailure> {
    let byte_len = units
        .len()
        .checked_mul(size_of::<u16>())
        .ok_or(UiClipboardTransferFailure::PayloadTooLarge)?;
    // SAFETY: the checked allocation is owned locally until a successful publish.
    let handle = unsafe { GlobalAlloc(GMEM_MOVEABLE, byte_len) };
    if handle.is_null() {
        return Err(UiClipboardTransferFailure::Unavailable);
    }
    let memory = OwnedGlobal::new(handle);
    // SAFETY: handle is a live exclusively owned HGLOBAL allocation.
    let locked = unsafe { LockedGlobal::lock(handle)? };
    // SAFETY: the allocation contains exactly enough space for units.
    unsafe {
        ptr::copy_nonoverlapping(units.as_ptr(), locked.pointer.cast::<u16>(), units.len());
    }
    drop(locked);
    Ok(memory)
}

// Local Win32 call seam: successful native memory/GDI cloning and its typed
// destructors remain shared by production and the injected tests below.
trait ClipboardApi {
    fn open(&mut self, owner: HWND) -> Result<(), UiClipboardTransferFailure>;
    fn close(&mut self);
    fn formats(&mut self) -> Result<Vec<u32>, UiClipboardTransferFailure>;
    fn get(
        &mut self,
        format: u32,
    ) -> Result<windows_sys::Win32::Foundation::HANDLE, UiClipboardTransferFailure>;
    fn clone_data(
        &mut self,
        format: u32,
        kind: SnapshotKind,
        source: windows_sys::Win32::Foundation::HANDLE,
        total: &mut usize,
    ) -> Result<OwnedSnapshot, UiClipboardTransferFailure> {
        // SAFETY: the transaction obtained this live format handle from get().
        unsafe { clone_snapshot(format, kind, source, total) }
    }
    fn empty(&mut self) -> Result<(), UiClipboardTransferFailure>;
    fn set(
        &mut self,
        format: u32,
        handle: windows_sys::Win32::Foundation::HANDLE,
    ) -> Result<(), UiClipboardTransferFailure>;
}

struct NativeClipboard;
impl ClipboardApi for NativeClipboard {
    fn open(&mut self, owner: HWND) -> Result<(), UiClipboardTransferFailure> {
        // SAFETY: the caller supplies the current target window's live HWND.
        if unsafe { OpenClipboard(owner) } == 0 {
            Err(last_clipboard_failure())
        } else {
            Ok(())
        }
    }
    fn close(&mut self) {
        // SAFETY: the transaction guard calls this only after a successful open.
        unsafe {
            CloseClipboard();
        }
    }
    fn formats(&mut self) -> Result<Vec<u32>, UiClipboardTransferFailure> {
        let mut formats = Vec::new();
        let mut previous = 0;
        loop {
            // SAFETY: clipboard is open; clearing last error distinguishes end
            // of enumeration from failure, as required by EnumClipboardFormats.
            unsafe {
                windows_sys::Win32::Foundation::SetLastError(0);
            }
            // SAFETY: the clipboard stays open on this thread; previous is zero or a format returned by this enumeration.
            let next =
                unsafe { windows_sys::Win32::System::DataExchange::EnumClipboardFormats(previous) };
            if next == 0 {
                // SAFETY: GetLastError has no preconditions and immediately follows the zero enumeration result.
                return if unsafe { GetLastError() } == 0 {
                    Ok(formats)
                } else {
                    Err(last_clipboard_failure())
                };
            }
            if formats.len() == SNAPSHOT_MAX_FORMATS || formats.contains(&next) {
                return Err(UiClipboardTransferFailure::PayloadTooLarge);
            }
            formats.push(next);
            previous = next;
        }
    }
    fn get(
        &mut self,
        format: u32,
    ) -> Result<windows_sys::Win32::Foundation::HANDLE, UiClipboardTransferFailure> {
        // SAFETY: clipboard stays open; borrowed handle is cloned before Empty.
        let handle = unsafe { GetClipboardData(format) };
        if handle.is_null() {
            Err(last_clipboard_failure())
        } else {
            Ok(handle)
        }
    }
    fn empty(&mut self) -> Result<(), UiClipboardTransferFailure> {
        // SAFETY: clipboard is open and the complete snapshot has been prepared.
        if unsafe { EmptyClipboard() } == 0 {
            Err(last_clipboard_failure())
        } else {
            Ok(())
        }
    }
    fn set(
        &mut self,
        format: u32,
        handle: windows_sys::Win32::Foundation::HANDLE,
    ) -> Result<(), UiClipboardTransferFailure> {
        // SAFETY: each format has its matching owned handle, unlocked, and the
        // transaction owns the clipboard. Capture the error before any rollback.
        if unsafe { SetClipboardData(format, handle) }.is_null() {
            Err(last_clipboard_failure())
        } else {
            Ok(())
        }
    }
}

struct ClipboardSession<'a, A: ClipboardApi>(&'a mut A);
impl<A: ClipboardApi> ClipboardSession<'_, A> {
    fn api(&mut self) -> &mut A {
        self.0
    }
}
impl<A: ClipboardApi> Drop for ClipboardSession<'_, A> {
    fn drop(&mut self) {
        self.0.close();
    }
}

fn check_snapshot_deadline(deadline: std::time::Instant) -> Result<(), UiClipboardTransferFailure> {
    if std::time::Instant::now() >= deadline {
        Err(UiClipboardTransferFailure::Timeout)
    } else {
        Ok(())
    }
}

fn snapshot_clipboard(
    api: &mut impl ClipboardApi,
) -> Result<Vec<OwnedSnapshot>, UiClipboardTransferFailure> {
    let deadline = std::time::Instant::now() + SNAPSHOT_DEADLINE;
    let formats = api.formats()?;
    if formats.len() > SNAPSHOT_MAX_FORMATS {
        return Err(UiClipboardTransferFailure::PayloadTooLarge);
    }
    // Classify every format before requesting any delayed-render data.
    for (index, &format) in formats.iter().enumerate() {
        snapshot_kind(format)?;
        if formats[..index].contains(&format) {
            return Err(UiClipboardTransferFailure::ContentUnavailable);
        }
    }
    let mut total_bytes = 0usize;
    let mut snapshots = Vec::new();
    for &format in &formats {
        check_snapshot_deadline(deadline)?;
        let kind = snapshot_kind(format)?;
        let handle = api.get(format)?;
        check_snapshot_deadline(deadline)?;
        let snapshot = api.clone_data(format, kind, handle, &mut total_bytes)?;
        snapshots.push(snapshot);
        check_snapshot_deadline(deadline)?;
    }
    // Delayed rendering may add formats; never empty an incompletely captured set.
    if api.formats()? != formats {
        return Err(UiClipboardTransferFailure::ContentUnavailable);
    }
    check_snapshot_deadline(deadline)?;
    Ok(snapshots)
}

fn charge_snapshot(total: &mut usize, size: usize) -> Result<(), UiClipboardTransferFailure> {
    if size == 0 {
        return Err(UiClipboardTransferFailure::ContentUnavailable);
    }
    *total = total
        .checked_add(size)
        .filter(|n| *n <= SNAPSHOT_MAX_BYTES)
        .ok_or(UiClipboardTransferFailure::PayloadTooLarge)?;
    Ok(())
}

unsafe fn clone_snapshot(
    format: u32,
    kind: SnapshotKind,
    source: windows_sys::Win32::Foundation::HANDLE,
    total: &mut usize,
) -> Result<OwnedSnapshot, UiClipboardTransferFailure> {
    use windows_sys::Win32::Graphics::Gdi::*;
    use windows_sys::Win32::System::DataExchange::METAFILEPICT;
    use windows_sys::Win32::System::Ole::{OleDuplicateData, CF_BITMAP, CF_PALETTE};
    let mut metafile = ptr::null_mut();
    // SAFETY: source is a live borrowed handle matching the classified format.
    let handle = unsafe {
        match kind {
            SnapshotKind::Global => {
                charge_snapshot(total, GlobalSize(source))?;
                OleDuplicateData(source, format as u16, GMEM_MOVEABLE)
            }
            SnapshotKind::Bitmap => {
                let mut bitmap = BITMAP::default();
                if GetObjectW(
                    source,
                    size_of::<BITMAP>() as i32,
                    (&mut bitmap as *mut BITMAP).cast(),
                ) != size_of::<BITMAP>() as i32
                    || bitmap.bmWidth <= 0
                    || bitmap.bmHeight == 0
                    || bitmap.bmWidthBytes <= 0
                    || bitmap.bmPlanes == 0
                    || bitmap.bmBitsPixel == 0
                {
                    return Err(UiClipboardTransferFailure::ContentUnavailable);
                }
                let bytes = (bitmap.bmWidthBytes as usize)
                    .checked_mul(bitmap.bmHeight.unsigned_abs() as usize)
                    .and_then(|n| n.checked_mul(bitmap.bmPlanes as usize))
                    .ok_or(UiClipboardTransferFailure::PayloadTooLarge)?;
                charge_snapshot(total, bytes)?;
                OleDuplicateData(source, CF_BITMAP, GMEM_MOVEABLE)
            }
            SnapshotKind::Palette => {
                let entries = GetPaletteEntries(source, 0, 0, ptr::null_mut());
                charge_snapshot(
                    total,
                    (entries as usize)
                        .checked_mul(size_of::<PALETTEENTRY>())
                        .ok_or(UiClipboardTransferFailure::PayloadTooLarge)?,
                )?;
                OleDuplicateData(source, CF_PALETTE, GMEM_MOVEABLE)
            }
            SnapshotKind::EnhancedMetafile => {
                charge_snapshot(
                    total,
                    GetEnhMetaFileBits(source, 0, ptr::null_mut()) as usize,
                )?;
                CopyEnhMetaFileW(source, ptr::null())
            }
            SnapshotKind::MetafilePicture => {
                if GlobalSize(source) < size_of::<METAFILEPICT>() {
                    return Err(UiClipboardTransferFailure::ContentUnavailable);
                }
                let locked = LockedGlobal::lock(source)?;
                let picture = *locked.pointer.cast::<METAFILEPICT>();
                charge_snapshot(total, size_of::<METAFILEPICT>())?;
                charge_snapshot(
                    total,
                    GetMetaFileBitsEx(picture.hMF, 0, ptr::null_mut()) as usize,
                )?;
                let nested = CopyMetaFileW(picture.hMF, ptr::null());
                if nested.is_null() {
                    return Err(UiClipboardTransferFailure::Unavailable);
                }
                let outer = GlobalAlloc(GMEM_MOVEABLE, size_of::<METAFILEPICT>());
                if outer.is_null() {
                    DeleteMetaFile(nested);
                    return Err(UiClipboardTransferFailure::Unavailable);
                }
                let owned = OwnedGlobal::new(outer);
                let destination = match LockedGlobal::lock(outer) {
                    Ok(locked) => locked,
                    Err(error) => {
                        DeleteMetaFile(nested);
                        return Err(error);
                    }
                };
                ptr::write(
                    destination.pointer.cast::<METAFILEPICT>(),
                    METAFILEPICT {
                        hMF: nested,
                        ..picture
                    },
                );
                drop(destination);
                let mut owned = owned;
                owned.release_to_clipboard(); // transferred locally to the typed owner below, not Win32
                metafile = nested;
                outer
            }
        }
    };
    if handle.is_null() {
        return Err(UiClipboardTransferFailure::Unavailable);
    }
    Ok(OwnedSnapshot {
        format,
        handle,
        kind,
        metafile,
    })
}

fn write_clipboard_transaction(
    api: &mut impl ClipboardApi,
    owner: HWND,
    units: &[u16],
) -> Result<(), UiClipboardTransferFailure> {
    api.open(owner)?;
    let mut session = ClipboardSession(api);
    let api = session.api();
    let mut snapshots = Vec::new();
    let mut memory = prepare_before_empty(
        || prepare_unicode(units),
        || {
            snapshots = snapshot_clipboard(api)?;
            api.empty()
        },
    )?;
    let result = publish_owned_global(
        &mut memory,
        |memory| api.set(u32::from(CF_UNICODETEXT), memory.handle),
        OwnedGlobal::release_to_clipboard,
    );
    if let Err(original_error) = result {
        // Best effort, not atomic rollback: retain the initial publish error,
        // continue after a failed restore, and release only accepted handles.
        for snapshot in &mut snapshots {
            if api.set(snapshot.format, snapshot.handle).is_ok() {
                snapshot.release();
            }
        }
        return Err(original_error);
    }
    Ok(())
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
    use super::*;
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

    struct MockClipboard {
        items: Vec<OwnedSnapshot>,
        calls: Vec<String>,
        fail_set: Vec<usize>,
        set_count: usize,
        empty_fails: bool,
        get_fails: bool,
        enum_fails: bool,
        extra_formats: Vec<u32>,
        clone_fail_at: Option<usize>,
        clone_count: usize,
    }

    impl MockClipboard {
        fn new(items: Vec<OwnedSnapshot>) -> Self {
            Self {
                items,
                calls: Vec::new(),
                fail_set: Vec::new(),
                set_count: 0,
                empty_fails: false,
                get_fails: false,
                enum_fails: false,
                extra_formats: Vec::new(),
                clone_fail_at: None,
                clone_count: 0,
            }
        }
        fn run(&mut self) -> Result<(), UiClipboardTransferFailure> {
            write_clipboard_transaction(self, ptr::null_mut(), &[110, 101, 119, 0])
        }
    }

    impl ClipboardApi for MockClipboard {
        fn open(&mut self, _: HWND) -> Result<(), UiClipboardTransferFailure> {
            self.calls.push("open".into());
            Ok(())
        }
        fn close(&mut self) {
            self.calls.push("close".into());
        }
        fn formats(&mut self) -> Result<Vec<u32>, UiClipboardTransferFailure> {
            self.calls.push("enum".into());
            if self.enum_fails {
                return Err(UiClipboardTransferFailure::Unavailable);
            }
            Ok(self
                .items
                .iter()
                .map(|item| item.format)
                .chain(self.extra_formats.iter().copied())
                .collect())
        }
        fn get(
            &mut self,
            format: u32,
        ) -> Result<windows_sys::Win32::Foundation::HANDLE, UiClipboardTransferFailure> {
            self.calls.push(format!("get:{format}"));
            if self.get_fails {
                return Err(UiClipboardTransferFailure::ContentUnavailable);
            }
            self.items
                .iter()
                .find(|item| item.format == format)
                .map(|item| item.handle)
                .ok_or(UiClipboardTransferFailure::ContentUnavailable)
        }
        fn clone_data(
            &mut self,
            format: u32,
            kind: SnapshotKind,
            source: windows_sys::Win32::Foundation::HANDLE,
            total: &mut usize,
        ) -> Result<OwnedSnapshot, UiClipboardTransferFailure> {
            self.clone_count += 1;
            self.calls.push(format!("clone:{format}"));
            if self.clone_fail_at == Some(self.clone_count) {
                return Err(UiClipboardTransferFailure::Unavailable);
            }
            // SAFETY: mock owns real live typed objects, not fabricated handles.
            unsafe { clone_snapshot(format, kind, source, total) }
        }
        fn empty(&mut self) -> Result<(), UiClipboardTransferFailure> {
            self.calls.push("empty".into());
            if self.empty_fails {
                return Err(UiClipboardTransferFailure::Unavailable);
            }
            // The real typed owners destroy original native objects just as
            // EmptyClipboard would; no user's OS clipboard is opened here.
            self.items.clear();
            Ok(())
        }
        fn set(
            &mut self,
            format: u32,
            handle: windows_sys::Win32::Foundation::HANDLE,
        ) -> Result<(), UiClipboardTransferFailure> {
            self.set_count += 1;
            self.calls.push(format!("set:{format}"));
            if self.fail_set.contains(&self.set_count) {
                return Err(if self.set_count == 1 {
                    UiClipboardTransferFailure::PermissionDenied
                } else {
                    UiClipboardTransferFailure::Unavailable
                });
            }
            assert!(!handle.is_null());
            assert!(!self.items.iter().any(|item| item.format == format));
            let kind = snapshot_kind(format)?;
            let metafile = if kind == SnapshotKind::MetafilePicture {
                // SAFETY: successful native cloning supplied a live owned wrapper;
                // record its nested owner now, never by reading it during Drop.
                let locked = unsafe { LockedGlobal::lock(handle) }.unwrap();
                // SAFETY: this live wrapper has the known METAFILEPICT layout.
                unsafe {
                    (*locked
                        .pointer
                        .cast::<windows_sys::Win32::System::DataExchange::METAFILEPICT>())
                    .hMF
                }
            } else {
                ptr::null_mut()
            };
            self.items.push(OwnedSnapshot {
                format,
                handle,
                kind,
                metafile,
            });
            Ok(())
        }
    }

    fn native_global(format: u32, bytes: &[u8]) -> OwnedSnapshot {
        // SAFETY: test owns the real native allocation; mock clipboard receives
        // ownership only through the same successful production transfer path.
        unsafe {
            let handle = GlobalAlloc(GMEM_MOVEABLE, bytes.len());
            assert!(!handle.is_null());
            let locked = LockedGlobal::lock(handle).unwrap();
            ptr::write_bytes(locked.pointer.cast::<u8>(), 0, GlobalSize(handle));
            ptr::copy_nonoverlapping(bytes.as_ptr(), locked.pointer.cast::<u8>(), bytes.len());
            drop(locked);
            OwnedSnapshot {
                format,
                handle,
                kind: SnapshotKind::Global,
                metafile: ptr::null_mut(),
            }
        }
    }

    fn native_bytes(item: &OwnedSnapshot) -> Vec<u8> {
        use windows_sys::Win32::Graphics::Gdi::*;
        // SAFETY: test inspects a live typed owner with the corresponding API.
        unsafe {
            match item.kind {
                SnapshotKind::Global => {
                    let locked = LockedGlobal::lock(item.handle).unwrap();
                    std::slice::from_raw_parts(locked.pointer.cast::<u8>(), GlobalSize(item.handle))
                        .to_vec()
                }
                SnapshotKind::Bitmap => {
                    let mut bitmap = BITMAP::default();
                    assert_eq!(
                        GetObjectW(
                            item.handle,
                            size_of::<BITMAP>() as i32,
                            (&mut bitmap as *mut BITMAP).cast()
                        ),
                        size_of::<BITMAP>() as i32
                    );
                    let mut bytes = vec![
                        0;
                        bitmap.bmWidthBytes as usize
                            * bitmap.bmHeight.unsigned_abs() as usize
                    ];
                    assert_eq!(
                        GetBitmapBits(item.handle, bytes.len() as i32, bytes.as_mut_ptr().cast()),
                        bytes.len() as i32
                    );
                    bytes
                }
                SnapshotKind::Palette => {
                    let count = GetPaletteEntries(item.handle, 0, 0, ptr::null_mut());
                    let mut entries = vec![PALETTEENTRY::default(); count as usize];
                    assert_eq!(
                        GetPaletteEntries(item.handle, 0, count, entries.as_mut_ptr()),
                        count
                    );
                    entries
                        .iter()
                        .flat_map(|e| [e.peRed, e.peGreen, e.peBlue, e.peFlags])
                        .collect()
                }
                SnapshotKind::EnhancedMetafile => {
                    let size = GetEnhMetaFileBits(item.handle, 0, ptr::null_mut());
                    assert!(size > 0);
                    let mut bytes = vec![0; size as usize];
                    assert_eq!(
                        GetEnhMetaFileBits(item.handle, size, bytes.as_mut_ptr()),
                        size
                    );
                    bytes
                }
                SnapshotKind::MetafilePicture => {
                    let locked = LockedGlobal::lock(item.handle).unwrap();
                    let picture = &*locked
                        .pointer
                        .cast::<windows_sys::Win32::System::DataExchange::METAFILEPICT>(
                    );
                    let size = GetMetaFileBitsEx(picture.hMF, 0, ptr::null_mut());
                    assert!(size > 0);
                    let mut bytes = Vec::new();
                    for value in [picture.mm, picture.xExt, picture.yExt] {
                        bytes.extend(value.to_le_bytes());
                    }
                    bytes.resize(12 + size as usize, 0);
                    assert_eq!(
                        GetMetaFileBitsEx(picture.hMF, size, bytes[12..].as_mut_ptr().cast()),
                        size
                    );
                    bytes
                }
            }
        }
    }

    fn mixed_native_objects() -> Vec<OwnedSnapshot> {
        use windows_sys::Win32::Graphics::Gdi::*;
        use windows_sys::Win32::System::Ole::*;
        let mut items = vec![
            native_global(u32::from(CF_UNICODETEXT), &[111, 0, 108, 0, 100, 0, 0, 0]),
            native_global(u32::from(CF_TEXT), b"old\0"),
            native_global(u32::from(CF_LOCALE), &0x409u32.to_le_bytes()),
        ];
        // Real DROPFILES (20-byte header), Unicode file-list, unchanged offsets.
        let mut dropfiles = vec![0u8; 20];
        dropfiles[..4].copy_from_slice(&20u32.to_le_bytes());
        dropfiles[16..20].copy_from_slice(&1u32.to_le_bytes());
        for unit in "E:\\original.txt".encode_utf16().chain([0, 0]) {
            dropfiles.extend(unit.to_le_bytes());
        }
        items.push(native_global(u32::from(CF_HDROP), &dropfiles));
        // SAFETY: all objects are created in memory and immediately owned here.
        unsafe {
            let bitmap = CreateBitmap(8, 2, 1, 1, [0x55u8, 0, 0xaau8, 0].as_ptr().cast());
            assert!(!bitmap.is_null());
            items.push(OwnedSnapshot {
                format: u32::from(CF_BITMAP),
                handle: bitmap,
                kind: SnapshotKind::Bitmap,
                metafile: ptr::null_mut(),
            });
            let palette = LOGPALETTE {
                palVersion: 0x300,
                palNumEntries: 1,
                palPalEntry: [PALETTEENTRY {
                    peRed: 17,
                    peGreen: 63,
                    peBlue: 201,
                    peFlags: 0,
                }],
            };
            let handle = CreatePalette(&palette);
            assert!(!handle.is_null());
            items.push(OwnedSnapshot {
                format: u32::from(CF_PALETTE),
                handle,
                kind: SnapshotKind::Palette,
                metafile: ptr::null_mut(),
            });
            let dc = CreateEnhMetaFileW(ptr::null_mut(), ptr::null(), ptr::null(), ptr::null());
            assert!(!dc.is_null());
            assert_ne!(Rectangle(dc, 0, 0, 8, 4), 0);
            let handle = CloseEnhMetaFile(dc);
            assert!(!handle.is_null());
            items.push(OwnedSnapshot {
                format: u32::from(CF_ENHMETAFILE),
                handle,
                kind: SnapshotKind::EnhancedMetafile,
                metafile: ptr::null_mut(),
            });
            let dc = CreateMetaFileW(ptr::null());
            assert!(!dc.is_null());
            assert_ne!(Rectangle(dc, 0, 0, 8, 4), 0);
            let nested = CloseMetaFile(dc);
            assert!(!nested.is_null());
            let picture = windows_sys::Win32::System::DataExchange::METAFILEPICT {
                mm: 8,
                xExt: 80,
                yExt: 40,
                hMF: nested,
            };
            let mut outer = native_global(
                u32::from(CF_METAFILEPICT),
                &vec![0; size_of::<windows_sys::Win32::System::DataExchange::METAFILEPICT>()],
            );
            let locked = LockedGlobal::lock(outer.handle).unwrap();
            ptr::write(
                locked
                    .pointer
                    .cast::<windows_sys::Win32::System::DataExchange::METAFILEPICT>(),
                picture,
            );
            drop(locked);
            outer.kind = SnapshotKind::MetafilePicture;
            outer.metafile = nested;
            items.push(outer);
        }
        items
    }

    #[test]
    fn publish_failure_restores_every_snapshotted_format_before_reporting_failure() {
        let items = mixed_native_objects();
        let expected = items
            .iter()
            .map(|item| (item.format, native_bytes(item)))
            .collect::<Vec<_>>();
        let mut api = MockClipboard::new(items);
        api.fail_set = vec![1];
        assert_eq!(api.run(), Err(UiClipboardTransferFailure::PermissionDenied));
        assert_eq!(
            api.items
                .iter()
                .map(|item| (item.format, native_bytes(item)))
                .collect::<Vec<_>>(),
            expected
        );
        let mut calls = vec!["open".to_string(), "enum".into()];
        calls.extend(
            expected
                .iter()
                .flat_map(|(format, _)| [format!("get:{format}"), format!("clone:{format}")]),
        );
        calls.extend(["enum".into(), "empty".into(), "set:13".into()]);
        calls.extend(expected.iter().map(|(format, _)| format!("set:{format}")));
        calls.push("close".into());
        assert_eq!(api.calls, calls);
    }

    #[test]
    fn unsupported_or_uncopyable_format_preserves_clipboard_without_emptying() {
        for unsupported in [128, 512, 768, 0xc001, 0xffff, 18] {
            let mut api = MockClipboard::new(vec![native_global(13, &[111, 0, 0, 0])]);
            let handle = api.items[0].handle;
            let bytes = native_bytes(&api.items[0]);
            api.extra_formats.push(unsupported);
            assert_eq!(api.run(), Err(UiClipboardTransferFailure::Unsupported));
            assert_eq!(api.items[0].handle, handle);
            assert_eq!(native_bytes(&api.items[0]), bytes);
            assert_eq!(api.calls, ["open", "enum", "close"]);
        }
        for enumeration_fails in [false, true] {
            let mut api = MockClipboard::new(vec![native_global(13, &[111, 0, 0, 0])]);
            let handle = api.items[0].handle;
            api.enum_fails = enumeration_fails;
            api.get_fails = !enumeration_fails;
            assert!(api.run().is_err());
            assert_eq!(api.items[0].handle, handle);
            assert!(!api
                .calls
                .iter()
                .any(|call| call == "empty" || call.starts_with("set:")));
            assert_eq!(api.calls.last().map(String::as_str), Some("close"));
        }
        let mut api = MockClipboard::new(vec![
            native_global(13, &[111, 0, 0, 0]),
            native_global(1, b"old\0"),
        ]);
        let original_handles = api.items.iter().map(|item| item.handle).collect::<Vec<_>>();
        api.clone_fail_at = Some(2);
        assert_eq!(api.run(), Err(UiClipboardTransferFailure::Unavailable));
        assert_eq!(
            api.items.iter().map(|item| item.handle).collect::<Vec<_>>(),
            original_handles
        );
        assert!(!api
            .calls
            .iter()
            .any(|call| call == "empty" || call.starts_with("set:")));
        assert_eq!(api.calls.last().map(String::as_str), Some("close"));
        let mut api = MockClipboard::new(vec![native_global(13, &vec![0; SNAPSHOT_MAX_BYTES + 1])]);
        assert_eq!(api.run(), Err(UiClipboardTransferFailure::PayloadTooLarge));
        assert!(!api
            .calls
            .iter()
            .any(|call| call == "empty" || call.starts_with("set:")));
        let mut api = MockClipboard::new(vec![native_global(13, &[111, 0, 0, 0])]);
        api.extra_formats = vec![13; SNAPSHOT_MAX_FORMATS];
        assert_eq!(api.run(), Err(UiClipboardTransferFailure::PayloadTooLarge));
        assert_eq!(api.calls, ["open", "enum", "close"]);
        let mut total = SNAPSHOT_MAX_BYTES;
        assert_eq!(
            charge_snapshot(&mut total, 1),
            Err(UiClipboardTransferFailure::PayloadTooLarge)
        );
        let mut total = usize::MAX;
        assert_eq!(
            charge_snapshot(&mut total, 1),
            Err(UiClipboardTransferFailure::PayloadTooLarge)
        );
        assert_eq!(
            check_snapshot_deadline(std::time::Instant::now()),
            Err(UiClipboardTransferFailure::Timeout)
        );
    }

    #[test]
    fn successful_publish_releases_snapshots_and_transfers_only_the_new_payload() {
        let mut api = MockClipboard::new(mixed_native_objects());
        assert_eq!(api.run(), Ok(()));
        assert_eq!(api.items.len(), 1);
        assert_eq!(api.items[0].format, 13);
        assert_eq!(
            &native_bytes(&api.items[0])[..8],
            &[110, 0, 101, 0, 119, 0, 0, 0]
        );
        assert_eq!(api.set_count, 1);
        assert_eq!(api.calls.last().map(String::as_str), Some("close"));
    }

    #[test]
    fn restore_failure_attempts_remaining_formats_and_never_reports_write_success() {
        let mut api = MockClipboard::new(vec![
            native_global(13, &[111, 0, 0, 0]),
            native_global(1, b"old\0"),
        ]);
        api.fail_set = vec![1, 2];
        assert_eq!(api.run(), Err(UiClipboardTransferFailure::PermissionDenied));
        assert_eq!(api.set_count, 3);
        assert_eq!(api.items.len(), 1);
        assert_eq!(api.items[0].format, 1);
        assert_eq!(&native_bytes(&api.items[0])[..4], b"old\0");
        assert_eq!(
            &api.calls[api.calls.len() - 4..],
            ["set:13", "set:13", "set:1", "close"]
        );
    }

    #[test]
    fn typed_gdi_snapshots_preserve_native_payload_and_own_independent_handles() {
        let items = mixed_native_objects();
        let mut total = 0;
        for item in &items {
            // SAFETY: native fixture owns a valid matching format handle.
            let copied =
                unsafe { clone_snapshot(item.format, item.kind, item.handle, &mut total) }.unwrap();
            assert_ne!(copied.handle, item.handle);
            assert_eq!(native_bytes(&copied), native_bytes(item));
            drop(copied);
            assert!(
                !native_bytes(item).is_empty(),
                "dropping the copy must not destroy its borrowed source"
            );
        }
    }

    #[test]
    fn empty_failure_keeps_original_formats_and_drops_only_prepared_copies() {
        let mut api = MockClipboard::new(mixed_native_objects());
        let expected = api
            .items
            .iter()
            .map(|item| (item.handle, native_bytes(item)))
            .collect::<Vec<_>>();
        api.empty_fails = true;
        assert_eq!(api.run(), Err(UiClipboardTransferFailure::Unavailable));
        assert_eq!(api.set_count, 0);
        assert_eq!(
            api.items
                .iter()
                .map(|item| (item.handle, native_bytes(item)))
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(&api.calls[api.calls.len() - 2..], ["empty", "close"]);
    }

    #[test]
    fn wmf_nested_owner_survives_transfer_and_is_freed_without_rereading_wrapper() {
        use windows_sys::Win32::Graphics::Gdi::GetMetaFileBitsEx;
        use windows_sys::Win32::System::DataExchange::METAFILEPICT;
        let mut originals = mixed_native_objects();
        let owner = originals.pop().unwrap();
        assert_eq!(owner.kind, SnapshotKind::MetafilePicture);
        let nested = owner.metafile;
        // SAFETY: this unpublished fixture exclusively owns a live WMF and wrapper.
        unsafe {
            assert!(GetMetaFileBitsEx(nested, 0, ptr::null_mut()) > 0);
            let locked = LockedGlobal::lock(owner.handle).unwrap();
            (*locked.pointer.cast::<METAFILEPICT>()).hMF = ptr::null_mut();
        }
        // Drop must use its explicit nested owner, not the now-empty wrapper field.
        drop(owner);
        // SAFETY: GDI reports zero for a deleted handle; no wrapper is dereferenced.
        assert_eq!(unsafe { GetMetaFileBitsEx(nested, 0, ptr::null_mut()) }, 0);

        let mut originals = mixed_native_objects();
        let owner = originals.pop().unwrap();
        let expected = native_bytes(&owner);
        let mut api = MockClipboard::new(vec![owner]);
        api.fail_set = vec![1];
        assert_eq!(api.run(), Err(UiClipboardTransferFailure::PermissionDenied));
        assert_eq!(api.items.len(), 1);
        let received = &api.items[0];
        assert_eq!(received.kind, SnapshotKind::MetafilePicture);
        assert!(!received.metafile.is_null());
        assert_eq!(native_bytes(received), expected);
        let transferred_nested = received.metafile;
        // The transaction's local snapshot owner has already dropped; successful
        // restore transferred both handles and must not have freed this WMF.
        // SAFETY: the mock clipboard now exclusively owns the successful transfer.
        assert!(unsafe { GetMetaFileBitsEx(transferred_nested, 0, ptr::null_mut()) } > 0);
        drop(api);
        // SAFETY: after the receiving owner drops, GDI must reject its deleted handle.
        assert_eq!(
            unsafe { GetMetaFileBitsEx(transferred_nested, 0, ptr::null_mut()) },
            0
        );
    }
}

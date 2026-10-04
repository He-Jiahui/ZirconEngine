use winit::event::Ime;
use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::Window;

use super::super::RuntimeEntryApp;
use super::composition_v2::ImeCompositionClauseAvailability;
use super::composition_v2::{
    ImeCompositionProducerError, NativeImeCompositionInput, RuntimeImeCompositionContextAllocator,
    RuntimeImeCompositionProducer,
};
use zircon_runtime_interface::ui::dispatch::{
    UiImePreeditClause, UiImePreeditClauseKind, UiTextByteRange,
};
use zircon_runtime_interface::{ZrRuntimeImeCandidateRectV2, ZrRuntimeImeCompositionNegotiation};

// IMM reports composition bytes and one attribute byte per UTF-16 code unit. Keep both the
// native allocation and the V2 clause envelope bounded before copying platform-owned memory.
const MAX_NATIVE_IME_COMPOSITION_BYTES: usize = 64 * 1024;
const MAX_NATIVE_IME_CLAUSES: usize = 256;

/// The platform callback is reduced to an owned event before the producer is borrowed.  This
/// keeps IMM memory and the Winit event lifetime out of the V2 envelope while the synchronous
/// runtime dispatch is in progress.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum NativeImeWindowInput {
    Preedit {
        text: String,
        cursor: Option<UiTextByteRange>,
        clauses: Vec<UiImePreeditClause>,
        clauses_available: bool,
    },
    Commit {
        text: String,
    },
    Cancel,
}

impl NativeImeWindowInput {
    fn as_composition_input(&self, begin: bool) -> NativeImeCompositionInput<'_> {
        match self {
            Self::Preedit {
                text,
                cursor,
                clauses,
                clauses_available,
            } => NativeImeCompositionInput::Preedit {
                text,
                cursor: *cursor,
                clauses: if *clauses_available {
                    ImeCompositionClauseAvailability::Available(clauses)
                } else {
                    ImeCompositionClauseAvailability::Unavailable
                },
                begin,
            },
            Self::Commit { text } => NativeImeCompositionInput::Commit { text },
            Self::Cancel => NativeImeCompositionInput::Cancel,
        }
    }
}

/// The current Winit Windows backend already receives the real `WM_IME_*` callback and reads
/// `GCS_COMPSTR`, `GCS_COMPATTR`, and `GCS_CURSORPOS` before emitting `WindowEvent::Ime`.  The app
/// consumes that callback synchronously here, preserving the native clause data while suppressing
/// the duplicate V1 preedit/commit stream.  This is an IMM adapter, not a capability-only marker.
#[cfg(windows)]
fn native_window_input(window: &dyn Window, ime: &Ime) -> Option<NativeImeWindowInput> {
    let hwnd = window_handle(window)?;
    match ime {
        Ime::Preedit(winit_text, winit_cursor) => {
            let fallback_cursor = winit_cursor.map(|(start, end)| {
                UiTextByteRange::new(
                    u32::try_from(start).unwrap_or(u32::MAX),
                    u32::try_from(end).unwrap_or(u32::MAX),
                )
            });
            let data = unsafe { read_imm_preedit(hwnd, winit_text, fallback_cursor) };
            Some(NativeImeWindowInput::Preedit {
                text: data.text,
                cursor: data.cursor,
                clauses: data.clauses,
                clauses_available: data.clauses_available,
            })
        }
        Ime::Commit(text) => Some(NativeImeWindowInput::Commit { text: text.clone() }),
        Ime::Disabled => Some(NativeImeWindowInput::Cancel),
        Ime::Enabled | Ime::DeleteSurrounding { .. } => None,
        _ => None,
    }
}

#[cfg(not(windows))]
fn native_window_input(_window: &dyn Window, _ime: &Ime) -> Option<NativeImeWindowInput> {
    None
}

#[cfg(windows)]
fn window_handle(window: &dyn Window) -> Option<*mut core::ffi::c_void> {
    let raw = window.window_handle().ok()?.as_raw();
    let RawWindowHandle::Win32(handle) = raw else {
        return None;
    };
    Some(handle.hwnd.get() as isize as *mut core::ffi::c_void)
}

#[cfg(windows)]
struct ImmPreedit {
    text: String,
    cursor: Option<UiTextByteRange>,
    clauses: Vec<UiImePreeditClause>,
    clauses_available: bool,
}

#[cfg(windows)]
unsafe fn read_imm_preedit(
    hwnd: *mut core::ffi::c_void,
    winit_text: &str,
    fallback_cursor: Option<UiTextByteRange>,
) -> ImmPreedit {
    let himc = ImmGetContext(hwnd);
    if himc.is_null() {
        return ImmPreedit {
            text: winit_text.to_owned(),
            cursor: fallback_cursor,
            clauses: Vec::new(),
            clauses_available: false,
        };
    }

    let text = read_imm_utf16(himc, GCS_COMPSTR)
        .filter(|value| !value.is_empty() || winit_text.is_empty())
        .unwrap_or_else(|| winit_text.to_owned());
    let cursor = read_imm_cursor(himc, &text).or(fallback_cursor);
    let (clauses, clauses_available) = match read_imm_bytes(himc, GCS_COMPATTR) {
        Some(attributes) if !attributes.is_empty() => {
            match clauses_from_attributes(&text, &attributes) {
                Some(clauses) => (clauses, true),
                None => (Vec::new(), false),
            }
        }
        Some(_) if text.is_empty() => (Vec::new(), true),
        Some(_) => (Vec::new(), false),
        None => (Vec::new(), false),
    };
    ImmReleaseContext(hwnd, himc);

    ImmPreedit {
        text,
        cursor,
        clauses,
        clauses_available,
    }
}

#[cfg(windows)]
unsafe fn read_imm_cursor(himc: *mut core::ffi::c_void, text: &str) -> Option<UiTextByteRange> {
    let units = ImmGetCompositionStringW(himc, GCS_CURSORPOS, core::ptr::null_mut(), 0);
    if units < 0 {
        return None;
    }
    let offset = utf16_units_to_utf8_bytes(text, usize::try_from(units).ok()?)?;
    Some(UiTextByteRange::new(offset, offset))
}

fn utf16_units_to_utf8_bytes(text: &str, units: usize) -> Option<u32> {
    let mut consumed = 0_usize;
    let mut bytes = 0_usize;
    for character in text.chars() {
        if consumed == units {
            return u32::try_from(bytes).ok();
        }
        consumed = consumed.checked_add(character.len_utf16())?;
        bytes = bytes.checked_add(character.len_utf8())?;
        if consumed > units {
            return None;
        }
    }
    (consumed == units)
        .then(|| u32::try_from(bytes).ok())
        .flatten()
}

#[cfg(windows)]
unsafe fn read_imm_utf16(himc: *mut core::ffi::c_void, mode: u32) -> Option<String> {
    let bytes = read_imm_bytes(himc, mode)?;
    if bytes.is_empty() || bytes.len() % core::mem::size_of::<u16>() != 0 {
        return bytes.is_empty().then(String::new);
    }
    // `read_imm_bytes` owns a byte-aligned Vec<u8>; casting its pointer to u16 would
    // make alignment an undocumented platform assumption. IMM returns UTF-16LE on
    // Windows, so decode each pair explicitly before handing it to String::from_utf16.
    let units = bytes
        .chunks_exact(core::mem::size_of::<u16>())
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect::<Vec<_>>();
    String::from_utf16(&units).ok()
}

#[cfg(windows)]
unsafe fn read_imm_bytes(himc: *mut core::ffi::c_void, mode: u32) -> Option<Vec<u8>> {
    let size = ImmGetCompositionStringW(himc, mode, core::ptr::null_mut(), 0);
    if size < 0 {
        return None;
    }
    let size = usize::try_from(size).ok()?;
    if size > MAX_NATIVE_IME_COMPOSITION_BYTES {
        return None;
    }
    if size == 0 {
        return Some(Vec::new());
    }
    let mut bytes = vec![0_u8; size];
    let copied = ImmGetCompositionStringW(
        himc,
        mode,
        bytes.as_mut_ptr().cast(),
        u32::try_from(size).ok()?,
    );
    if copied < 0 || usize::try_from(copied).ok()? != size {
        return None;
    }
    Some(bytes)
}

fn clauses_from_attributes(text: &str, attributes: &[u8]) -> Option<Vec<UiImePreeditClause>> {
    if attributes.len() > MAX_NATIVE_IME_COMPOSITION_BYTES {
        return None;
    }
    let expected_units = text.encode_utf16().count();
    if expected_units != attributes.len() {
        return None;
    }
    if text.is_empty() {
        return Some(Vec::new());
    }

    // IMM attributes are indexed by UTF-16 code unit, while the V2 DTO carries
    // UTF-8 byte ranges. Keep surrogate pairs in one clause and reject a pair
    // whose two code units disagree instead of emitting a split UTF-8 range.
    let mut clauses = Vec::new();
    let mut unit_offset = 0_usize;
    let mut byte_offset = 0_usize;
    let mut clause_start = 0_usize;
    let mut clause_kind = None;
    for character in text.chars() {
        let units = character.len_utf16();
        let end_units = unit_offset.checked_add(units)?;
        let first_attribute = attributes.get(unit_offset).copied()?;
        if attributes
            .get(unit_offset..end_units)?
            .iter()
            .any(|attribute| *attribute != first_attribute)
        {
            return None;
        }
        let kind = imm_attribute_kind(first_attribute);
        if clause_kind.is_some_and(|previous| previous != kind) {
            if clauses.len() >= MAX_NATIVE_IME_CLAUSES {
                return None;
            }
            clauses.push(UiImePreeditClause::new(
                UiTextByteRange::new(
                    u32::try_from(clause_start).ok()?,
                    u32::try_from(byte_offset).ok()?,
                ),
                clause_kind?,
            ));
            clause_start = byte_offset;
        }
        clause_kind = Some(kind);
        unit_offset = end_units;
        byte_offset = byte_offset.checked_add(character.len_utf8())?;
    }
    if clauses.len() >= MAX_NATIVE_IME_CLAUSES {
        return None;
    }
    clauses.push(UiImePreeditClause::new(
        UiTextByteRange::new(
            u32::try_from(clause_start).ok()?,
            u32::try_from(byte_offset).ok()?,
        ),
        clause_kind?,
    ));
    Some(clauses)
}

const fn imm_attribute_kind(attribute: u8) -> UiImePreeditClauseKind {
    match attribute {
        1 => UiImePreeditClauseKind::TargetConverted,
        2 => UiImePreeditClauseKind::Converted,
        3 => UiImePreeditClauseKind::TargetNotConverted,
        _ => UiImePreeditClauseKind::Input,
    }
}

#[cfg(windows)]
const GCS_COMPSTR: u32 = 0x0008;
#[cfg(windows)]
const GCS_COMPATTR: u32 = 0x0010;
#[cfg(windows)]
const GCS_CURSORPOS: u32 = 0x0080;

#[cfg(windows)]
#[link(name = "imm32")]
unsafe extern "system" {
    fn ImmGetContext(hwnd: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn ImmReleaseContext(hwnd: *mut core::ffi::c_void, himc: *mut core::ffi::c_void) -> i32;
    fn ImmGetCompositionStringW(
        himc: *mut core::ffi::c_void,
        index: u32,
        data: *mut core::ffi::c_void,
        length: u32,
    ) -> i32;
}

#[cfg(windows)]
pub(super) const fn native_ime_provider_available() -> bool {
    // The Winit Windows backend's normal WM_IME callback and this IMM reader are compiled as one
    // adapter.  No TSF capability is advertised; a host that requests V2 on another platform is
    // rejected before window/session mutation.
    true
}

#[cfg(not(windows))]
pub(super) const fn native_ime_provider_available() -> bool {
    false
}

impl RuntimeEntryApp {
    /// Installs the host-owned window lifetime only after the host has a real negotiated peer
    /// capability. No node id or implicit default token is accepted here.
    pub(in crate::entry::runtime_entry_app) fn set_native_ime_composition_context(
        &mut self,
        negotiation: ZrRuntimeImeCompositionNegotiation,
        window_generation: u64,
        candidate_rect: ZrRuntimeImeCandidateRectV2,
    ) -> Result<(), ImeCompositionProducerError> {
        let allocator = RuntimeImeCompositionContextAllocator::new(window_generation)?;
        self.ime_composition_producer =
            Some(RuntimeImeCompositionProducer::new_with_candidate_rect(
                negotiation,
                allocator,
                candidate_rect,
            ));
        Ok(())
    }

    pub(in crate::entry::runtime_entry_app) fn handle_native_ime_composition(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        input: NativeImeCompositionInput<'_>,
    ) -> bool {
        let Some(producer) = self.ime_composition_producer.as_mut() else {
            // Native attributes without an installed host/window context are rejected. Winit's
            // regular text/cursor callback continues to use the existing V1 path.
            return false;
        };
        let event = match producer.native_event(self.viewport, input) {
            Ok(event) => event,
            Err(_) => return false,
        };
        self.dispatch_runtime_event(event_loop, event.event())
    }

    /// Converts a real Winit Windows IMM callback into the negotiated V2 producer.  Returning
    /// `Some(false)` deliberately suppresses the legacy event if dispatch fails; emitting V1 after
    /// a failed V2 dispatch would duplicate or reorder one composition.
    pub(in crate::entry::runtime_entry_app) fn handle_native_ime_window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        ime: &Ime,
    ) -> Option<bool> {
        if !self.native_ime_composition_requested {
            return None;
        }
        if self.ime_composition_producer.is_none() {
            // A V2 opt-in is an explicit source choice. If a composition arrives
            // before the host has installed its checked caret context, do not
            // silently fall back to V1 (which would create a second source); fail
            // the request before dispatching any unnegotiated composition event.
            if matches!(ime, Ime::Preedit(..) | Ime::Commit(..)) {
                self.report_fatal_failure(
                    "runtime_ime",
                    "native_app_session_v2_composition",
                    "native V2 composition arrived before its live window/caret producer",
                    "publish a focused candidate rectangle before enabling native IME V2",
                );
                event_loop.exit();
                return Some(false);
            }
            return None;
        }
        let composition_event = matches!(ime, Ime::Preedit(..) | Ime::Commit(..) | Ime::Disabled);
        let Some(window) = self.window.as_deref() else {
            if composition_event {
                self.report_fatal_failure(
                    "runtime_ime",
                    "native_app_session_v2_composition",
                    "native V2 composition callback had no live primary window",
                    "retain the focused native window until IME composition has been cancelled",
                );
                event_loop.exit();
                return Some(false);
            }
            return None;
        };
        let Some(input) = native_window_input(window, ime) else {
            if composition_event {
                self.report_fatal_failure(
                    "runtime_ime",
                    "native_app_session_v2_composition",
                    "native V2 composition callback could not resolve the live Win32 IMM context",
                    "use the supported Windows Winit/IMM window adapter before requesting IME V2",
                );
                event_loop.exit();
                return Some(false);
            }
            return None;
        };
        if matches!(&input, NativeImeWindowInput::Cancel) && !self.native_ime_composition_started {
            // Winit emits Disabled after a result commit as part of the normal
            // WM_IME_ENDCOMPOSITION sequence. That closes source arbitration but
            // must not manufacture a second V2 cancellation for an already
            // committed composition.
            return Some(true);
        }
        let begin = matches!(&input, NativeImeWindowInput::Preedit { .. })
            && !self.native_ime_composition_started;
        let composition_input = input.as_composition_input(begin);
        let dispatched = self.handle_native_ime_composition(event_loop, composition_input);
        if dispatched {
            match input {
                NativeImeWindowInput::Preedit { .. } => self.native_ime_composition_started = true,
                NativeImeWindowInput::Commit { .. } | NativeImeWindowInput::Cancel => {
                    self.native_ime_composition_started = false;
                }
            }
        }
        Some(dispatched)
    }

    pub(in crate::entry::runtime_entry_app) fn handle_native_ime_focus_loss(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) -> bool {
        let viewport = self.viewport;
        let Some(producer) = self.ime_composition_producer.as_mut() else {
            return true;
        };
        let event = match producer.focus_changed_event(viewport) {
            Ok(event) => event,
            Err(_) => return false,
        };
        self.dispatch_runtime_event(event_loop, event.event())
    }

    /// Retires a live native composition before surface/window ownership is dropped. Teardown has
    /// no `ActiveEventLoop`, so the cancellation is sent synchronously through the retained
    /// session while the producer and its runtime library are still alive.
    pub(in crate::entry::runtime_entry_app) fn retire_native_ime_composition(&mut self) {
        if !self.native_ime_composition_started {
            return;
        }
        let viewport = self.viewport;
        let event = self
            .ime_composition_producer
            .as_mut()
            .and_then(|producer| producer.focus_changed_event(viewport).ok());
        let Some(event) = event else {
            self.report_fatal_failure(
                "runtime_ime",
                "native_app_session_v2_teardown",
                "native IME producer could not allocate a teardown cancellation",
                "retire the focused native IME composition before destroying its window",
            );
            self.native_ime_composition_started = false;
            return;
        };
        if let Err(error) = self.session.handle_event(event.event()) {
            self.report_fatal_failure(
                "runtime_ime",
                "native_app_session_v2_teardown",
                format!("native IME teardown cancellation failed: {error}"),
                "keep the runtime session alive until native IME cancellation is delivered",
            );
        }
        self.native_ime_composition_started = false;
    }
}

#[cfg(test)]
#[path = "tests/native.rs"]
mod tests;

use std::slice;

use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeEventV1, ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1,
    ZrRuntimeSessionConfigV1, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1, ZrStatus, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::{LoadedRuntime, RuntimeLibraryError};

pub(crate) struct RuntimeSession {
    runtime: LoadedRuntime,
    handle: ZrRuntimeSessionHandle,
}

impl RuntimeSession {
    pub(crate) fn create(runtime: LoadedRuntime) -> Result<Self, RuntimeLibraryError> {
        Self::create_with_profile(runtime, b"runtime")
    }

    pub(crate) fn create_with_profile(
        runtime: LoadedRuntime,
        profile: &'static [u8],
    ) -> Result<Self, RuntimeLibraryError> {
        let create_session = runtime
            .api()
            .create_session
            .ok_or_else(|| RuntimeLibraryError::new("runtime API missing create_session"))?;
        let mut handle = ZrRuntimeSessionHandle::invalid();
        let status = unsafe {
            create_session(
                ZrRuntimeSessionConfigV1 {
                    abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
                    profile: ZrByteSlice::from_static(profile),
                    project_manifest: ZrByteSlice::empty(),
                },
                &mut handle,
            )
        };
        ensure_status(status, "create runtime session")?;
        if !handle.is_valid() {
            return Err(RuntimeLibraryError::new(
                "runtime returned an invalid session handle",
            ));
        }
        Ok(Self { runtime, handle })
    }

    pub(crate) fn handle(&self) -> ZrRuntimeSessionHandle {
        self.handle
    }

    pub(crate) fn handle_event(&self, event: ZrRuntimeEventV1) -> Result<(), RuntimeLibraryError> {
        let handle_event = self
            .runtime
            .api()
            .handle_event
            .ok_or_else(|| RuntimeLibraryError::new("runtime API missing handle_event"))?;
        let status = unsafe { handle_event(self.handle, event) };
        ensure_status(status, "send runtime event")
    }

    pub(crate) fn capture_frame(
        &self,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<RuntimeFrame, RuntimeLibraryError> {
        let capture_frame = self
            .runtime
            .api()
            .capture_frame
            .ok_or_else(|| RuntimeLibraryError::new("runtime API missing capture_frame"))?;
        let mut frame = ZrRuntimeFrameV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
        let status = unsafe {
            capture_frame(
                self.handle,
                ZrRuntimeFrameRequestV1::new(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport, size),
                &mut frame,
            )
        };
        ensure_status(status, "capture runtime frame")?;
        Ok(RuntimeFrame { frame })
    }
}

#[cfg(feature = "target-editor-host")]
impl zircon_editor::EditorRuntimeClient for RuntimeSession {
    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        self.handle()
    }

    fn handle_event(&self, event: ZrRuntimeEventV1) -> Result<(), String> {
        RuntimeSession::handle_event(self, event).map_err(|error| error.to_string())
    }

    fn capture_frame(
        &self,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<ZrRuntimeFrameV1, String> {
        let capture_frame = self
            .runtime
            .api()
            .capture_frame
            .ok_or_else(|| "runtime API missing capture_frame".to_string())?;
        let mut frame = ZrRuntimeFrameV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
        let status = unsafe {
            capture_frame(
                self.handle,
                ZrRuntimeFrameRequestV1::new(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport, size),
                &mut frame,
            )
        };
        ensure_status(status, "capture runtime frame").map_err(|error| error.to_string())?;
        Ok(frame)
    }
}

impl Drop for RuntimeSession {
    fn drop(&mut self) {
        if let Some(destroy_session) = self.runtime.api().destroy_session {
            let _ = unsafe { destroy_session(self.handle) };
        }
    }
}

pub(crate) struct RuntimeFrame {
    frame: ZrRuntimeFrameV1,
}

impl RuntimeFrame {
    pub(crate) fn width(&self) -> u32 {
        self.frame.width
    }

    pub(crate) fn height(&self) -> u32 {
        self.frame.height
    }

    pub(crate) fn rgba(&self) -> &[u8] {
        let rgba = self.frame.rgba;
        if rgba.data.is_null() || rgba.len == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(rgba.data.cast_const(), rgba.len) }
        }
    }
}

impl Drop for RuntimeFrame {
    fn drop(&mut self) {
        if let Some(free) = self.frame.rgba.free {
            let buffer = self.frame.rgba;
            let _ = unsafe { free(buffer) };
            self.frame.rgba = ZrOwnedByteBuffer::empty();
        }
    }
}

fn ensure_status(status: ZrStatus, operation: &'static str) -> Result<(), RuntimeLibraryError> {
    if status.is_ok() {
        return Ok(());
    }
    let diagnostics = unsafe { status.diagnostics.as_slice() };
    let diagnostics = String::from_utf8_lossy(diagnostics);
    let code = match status.status_code() {
        ZrStatusCode::Ok => "ok",
        ZrStatusCode::Error => "error",
        ZrStatusCode::UnsupportedVersion => "unsupported-version",
        ZrStatusCode::InvalidArgument => "invalid-argument",
        ZrStatusCode::NotFound => "not-found",
        ZrStatusCode::CapabilityDenied => "capability-denied",
        ZrStatusCode::Panic => "panic",
    };
    Err(RuntimeLibraryError::new(format!(
        "failed to {operation}: {code}: {diagnostics}"
    )))
}

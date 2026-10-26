//! A bounded diagnostic hold owned by the actual normal GPU presenter.
use super::super::data::HostPresentationGenerationCursor;
use super::normal_present::NormalSceneImageSelection;
use crate::ui::retained_host::host_contract::normal_capture_redraw_trace as capture_trace;
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use zircon_runtime::rhi::RenderNativeSurfaceTarget;

#[derive(Deserialize)]
struct Request {
    nonce: String,
    #[serde(rename = "caseSha256")]
    case_sha256: String,
    pid: u32,
    #[serde(rename = "requestedAtUnixMs")]
    requested_at_unix_ms: u64,
    #[serde(rename = "deadlineMs")]
    deadline_ms: u64,
    #[serde(rename = "physicalSize")]
    physical_size: [u32; 2],
    #[serde(default)]
    release: bool,
    #[serde(default, rename = "gpuReadback")]
    gpu_readback: bool,
    #[serde(default)]
    hwnd: Option<u64>,
}

struct PendingReadback {
    nonce: String,
    case_sha256: String,
    hwnd: u64,
    cursor: HostPresentationGenerationCursor,
    size: (u32, u32),
    deadline: Instant,
    presented_frame_count: Option<u64>,
    scene_image: Option<NormalSceneImageSelection>,
    scene_unavailable_reason: Option<String>,
}
struct Held {
    nonce: String,
    cursor: HostPresentationGenerationCursor,
    size: (u32, u32),
    deadline: Instant,
    evidence: Value,
}
#[derive(Default)]
pub(in crate::ui::retained_host::host_contract) struct NormalCaptureController {
    native_hwnd: Option<u64>,
    held: Option<Held>,
    consumed_nonce: Option<String>,
    readback_pending: Option<PendingReadback>,
    attempted_readback_nonce: Option<String>,
}
fn scene_readback_matches(
    selection: Option<&NormalSceneImageSelection>,
    result: &zircon_runtime::rhi::UiSurfaceFrameReadback,
) -> bool {
    use zircon_runtime::rhi::{
        DiagnosticReadbackKind, DiagnosticReadbackTerminal, UiSurfaceImageAlphaMode,
    };
    let (selection, request, image) = match (
        selection,
        result.request.image.as_ref(),
        result.image.as_ref(),
    ) {
        (None, None, None) => return true,
        (Some(selection), Some(request), Some(image)) => (selection, request, image),
        _ => return false,
    };
    let expected = u64::from(selection.request.physical_size.0)
        .checked_mul(u64::from(selection.request.physical_size.1))
        .and_then(|pixels| pixels.checked_mul(4));
    let raw_valid = image.color8.as_ref().is_some_and(|raw| {
        expected == Some(raw.len() as u64)
            && image.opaque_pixels == Some(raw.chunks_exact(4).all(|pixel| pixel[3] == 255))
            && (image.alpha_mode != Some(UiSurfaceImageAlphaMode::Opaque)
                || image.opaque_pixels == Some(true))
    });
    request == &selection.request
        && image.request == selection.request
        && image.command_frame == Some(selection.command_frame)
        && image.command_clip == selection.command_clip
        && image.failure.is_none()
        && image.pixel_format.is_some()
        && image.alpha_mode.is_some()
        && raw_valid
        && request
            .expected_pixel_format
            .is_none_or(|format| image.pixel_format == Some(format))
        && result.submission.is_some()
        && result.receipt.is_some_and(|window| {
            window.kind() == DiagnosticReadbackKind::Texture
                && window.terminal() == DiagnosticReadbackTerminal::Succeeded
                && window.frame_key().map(|key| key.submission()) == result.submission
                && image.receipt.is_some_and(|receipt| {
                    receipt.kind() == DiagnosticReadbackKind::Texture
                        && receipt.terminal() == DiagnosticReadbackTerminal::Succeeded
                        && receipt.request() != window.request()
                        && receipt.frame_key() == window.frame_key()
                        && expected.is_some_and(|expected| receipt.byte_len() >= expected)
                })
        })
}
fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis() as u64)
        .unwrap_or(0)
}
fn directory() -> Option<std::path::PathBuf> {
    if !super::profile_capture_enabled() {
        return None;
    }
    let path = super::environment::profile_export_dir()
        .ok()??
        .canonicalize()
        .ok()?;
    approved_owned_directory(&path).then_some(path)
}

// The already-resolved path keeps its physical location and verbatim prefix.
// Accept only drive roots; UNC and nested lookalike roots remain inadmissible.
fn approved_owned_directory(path: &std::path::Path) -> bool {
    #[cfg(windows)]
    {
        use std::path::{Component, Prefix};
        let mut components = path.components();
        let drive = match components.next() {
            Some(Component::Prefix(prefix)) => match prefix.kind() {
                Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) => drive.to_ascii_uppercase(),
                _ => return false,
            },
            _ => return false,
        };
        if !matches!(drive, b'D' | b'E' | b'F') || components.next() != Some(Component::RootDir) {
            return false;
        }
        for expected in ["cargo-targets", "zircon-local"] {
            match components.next() {
                Some(Component::Normal(value))
                    if value
                        .to_str()
                        .is_some_and(|name| name.eq_ignore_ascii_case(expected)) => {}
                _ => return false,
            }
        }
        components.all(|component| matches!(component, Component::Normal(_)))
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        false
    }
}

fn request() -> Option<Request> {
    let path = directory()?.join("capture-request.json");
    if path.metadata().ok()?.len() > 16 * 1024 {
        return None;
    }
    let value: Request = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
    let now = unix_ms();
    (value.pid == std::process::id()
        && !value.nonce.is_empty()
        && value.nonce.len() <= 96
        && value
            .nonce
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        && value.case_sha256.len() == 64
        && value.case_sha256.bytes().all(|c| c.is_ascii_hexdigit())
        && (50..=10_000).contains(&value.deadline_ms)
        && value.requested_at_unix_ms <= now
        && now - value.requested_at_unix_ms <= value.deadline_ms)
        .then_some(value)
}
fn publish(value: &Value) {
    let Some(root) = directory() else {
        return;
    };
    let path = root.join("capture-hold-ack.json");
    if let Ok(bytes) = std::fs::read(&path) {
        if serde_json::from_slice::<Value>(&bytes)
            .ok()
            .and_then(|old| old["pid"].as_u64())
            != Some(u64::from(std::process::id()))
        {
            return;
        }
    }
    if let Ok(bytes) = serde_json::to_vec_pretty(value) {
        let _ = std::fs::write(path, bytes);
    }
}
impl NormalCaptureController {
    pub(in crate::ui::retained_host::host_contract) fn for_native_target(
        target: Option<RenderNativeSurfaceTarget>,
    ) -> Self {
        Self {
            native_hwnd: match target {
                Some(RenderNativeSurfaceTarget::Win32 { hwnd, .. }) if hwnd > 0 => Some(hwnd),
                _ => None,
            },
            ..Default::default()
        }
    }

    fn owns_request(&self, value: &Request) -> bool {
        self.native_hwnd
            .is_some_and(|hwnd| value.hwnd == Some(hwnd))
    }

    fn live_request(&self) -> Option<Request> {
        request().filter(|value| self.owns_request(value))
    }

    fn publication_matches_request(&self, request: &Request, value: &Value) -> bool {
        self.owns_request(request)
            && value["pid"].as_u64() == Some(u64::from(request.pid))
            && value["nonce"].as_str() == Some(request.nonce.as_str())
            && value["caseSha256"].as_str() == Some(request.case_sha256.as_str())
    }

    fn publish(&self, value: &Value) {
        // An obsolete hold or cancelled readback must not overwrite another
        // window's ACK after the process-wide request moves to that window.
        if self
            .live_request()
            .is_some_and(|request| self.publication_matches_request(&request, value))
        {
            publish(value);
        }
    }

    // Profiling is an explicit normal-window diagnostic mode. Its bounded poll
    // observes a new request while the regular UI event loop is otherwise idle.
    pub(in crate::ui::retained_host::host_contract) fn request_poll_deadline(
        &self,
    ) -> Option<Instant> {
        directory().map(|_| Instant::now() + Duration::from_millis(100))
    }

    pub(in crate::ui::retained_host::host_contract) fn capture_present_requested(
        &self,
        size: (u32, u32),
    ) -> bool {
        self.live_request()
            .is_some_and(|value| self.request_requires_present(&value, size))
    }

    fn request_requires_present(&self, value: &Request, size: (u32, u32)) -> bool {
        // Consumed nonces still need one normal present to acknowledge release while idle.
        // An OS WM_PAINT alone has no pending HostRedrawRequest and never enters before_present.
        if value.release
            && self.owns_request(value)
            && value.physical_size == [size.0, size.1]
            && self.held.as_ref().is_some_and(|held| {
                held.size == size
                    && held.nonce == value.nonce
                    && self.publication_matches_request(value, &held.evidence)
            })
        {
            return true;
        }
        value.gpu_readback
            && !value.release
            && self.owns_request(value)
            && value.physical_size == [size.0, size.1]
            && self.readback_pending.is_none()
            && self.attempted_readback_nonce.as_deref() != Some(value.nonce.as_str())
            && self.consumed_nonce.as_deref() != Some(value.nonce.as_str())
            && self
                .held
                .as_ref()
                .is_none_or(|held| held.nonce != value.nonce)
    }

    pub(in crate::ui::retained_host::host_contract) fn prepare_frame_readback(
        &mut self,
        cursor: HostPresentationGenerationCursor,
        draw_list: &zircon_runtime::rhi::UiSurfaceDrawList,
        scene_image: Result<Option<NormalSceneImageSelection>, String>,
    ) -> Option<zircon_runtime::rhi::UiSurfaceFrameReadbackRequest> {
        let request = self.live_request()?;
        let mut readback =
            self.prepare_frame_readback_for_request(cursor, draw_list.surface_size, request)?;
        readback.expected_draw_generation = draw_list.generation();
        let pending = self.readback_pending.as_mut()?;
        match scene_image {
            Ok(Some(scene)) if scene.matches_draw_list(draw_list) => {
                readback.image = Some(scene.request.clone());
                pending.scene_image = Some(scene);
            }
            Ok(Some(_)) => {
                pending.scene_unavailable_reason =
                    Some("scene selection differs from the actual RHI draw list".into())
            }
            Ok(None) => {
                pending.scene_unavailable_reason =
                    Some("no authored Scene image was submitted".into())
            }
            Err(reason) => pending.scene_unavailable_reason = Some(reason),
        }
        Some(readback)
    }

    fn prepare_frame_readback_for_request(
        &mut self,
        cursor: HostPresentationGenerationCursor,
        size: (u32, u32),
        request: Request,
    ) -> Option<zircon_runtime::rhi::UiSurfaceFrameReadbackRequest> {
        if !self.owns_request(&request)
            || !request.gpu_readback
            || request.release
            || self.readback_pending.is_some()
            || self.attempted_readback_nonce.as_deref() == Some(request.nonce.as_str())
            || self.consumed_nonce.as_deref() == Some(request.nonce.as_str())
            || request.physical_size != [size.0, size.1]
        {
            return None;
        }
        let hwnd = self.native_hwnd?;
        capture_trace::record("capture_prepare_readback", || {
            format!(
                "hwnd={hwnd} nonce={} cursor={cursor:?} size={size:?}",
                request.nonce
            )
        });
        self.attempted_readback_nonce = Some(request.nonce.clone());
        self.readback_pending = Some(PendingReadback {
            nonce: request.nonce.clone(),
            case_sha256: request.case_sha256,
            hwnd,
            cursor,
            size,
            deadline: Instant::now() + Duration::from_millis(request.deadline_ms),
            presented_frame_count: None,
            scene_image: None,
            scene_unavailable_reason: None,
        });
        Some(zircon_runtime::rhi::UiSurfaceFrameReadbackRequest {
            nonce: request.nonce,
            physical_size: size,
            expected_draw_generation: None,
            deadline_ms: request.deadline_ms,
            image: None,
        })
    }
    pub(in crate::ui::retained_host::host_contract) fn fail_frame_readback(
        &mut self,
        reason: &str,
    ) {
        self.invalidate(reason);
        if let Some(pending) = self.readback_pending.take() {
            self.publish(
                &json!({"schema":"dev.zircon.editor.normal-capture-hold","version":1,"pid":std::process::id(),"nonce":pending.nonce,"caseSha256":pending.case_sha256,"status":reason,"pixelFrameVerified":false}),
            );
        }
    }
    pub(in crate::ui::retained_host::host_contract) fn readback_current(
        &self,
        cursor: HostPresentationGenerationCursor,
        size: (u32, u32),
    ) -> bool {
        self.readback_pending.as_ref().is_some_and(|pending| {
            pending.cursor == cursor
                && pending.size == size
                && Instant::now() < pending.deadline
                && self.live_request().is_some_and(|live| {
                    !live.release
                        && live.nonce == pending.nonce
                        && live.case_sha256 == pending.case_sha256
                        && live.hwnd == Some(pending.hwnd)
                        && live.physical_size == [pending.size.0, pending.size.1]
                })
        })
    }
    pub(in crate::ui::retained_host::host_contract) fn readback_wake_deadline(
        &self,
    ) -> Option<Instant> {
        self.readback_pending.as_ref().map(|pending| {
            pending
                .deadline
                .min(Instant::now() + Duration::from_millis(8))
        })
    }
    pub(in crate::ui::retained_host::host_contract) fn accept_frame_readback(
        &mut self,
        mut result: zircon_runtime::rhi::UiSurfaceFrameReadback,
        cursor: HostPresentationGenerationCursor,
        size: (u32, u32),
    ) -> Option<super::normal_frame_readback::NormalReadbackExport> {
        let current = self.readback_current(cursor, size);
        let pending = self.readback_pending.take()?;
        let held = self.held.as_ref().filter(|held| {
            held.nonce == pending.nonce
                && held.cursor == cursor
                && held.size == size
                && Instant::now() < held.deadline
                && held.evidence["caseSha256"].as_str() == Some(pending.case_sha256.as_str())
                && held.evidence["presentedFrameCount"].as_u64() == pending.presented_frame_count
        });
        let actual_hwnd = match result.native_target {
            Some(zircon_runtime::rhi::RenderNativeSurfaceTarget::Win32 { hwnd, .. }) => Some(hwnd),
            _ => None,
        };
        if !current
            || result.request.nonce != pending.nonce
            || result.request.physical_size != pending.size
            || pending.cursor != cursor
            || pending.size != size
            || Instant::now() >= pending.deadline
            || actual_hwnd != Some(pending.hwnd)
            || pending.presented_frame_count.is_none()
            || held.is_none()
        {
            result.failure=Some("normal readback nonce, HWND, held presentation or current generation is not proven".into());
        }
        if result.failure.is_none()
            && !scene_readback_matches(pending.scene_image.as_ref(), &result)
        {
            result.failure = Some("scene readback does not match its source-owned command and the same successful normal submission".into());
        }
        let mut metadata = json!({"schema":"dev.zircon.editor.normal-frame-readback","version":1,"authority":"normal-actual-gpu-packet-and-readback-receipt","pid":std::process::id(),"hwnd":actual_hwnd,
            "nonce":pending.nonce,"caseSha256":pending.case_sha256,"physicalSize":[pending.size.0,pending.size.1],"presentationCursor":format!("{:?}",pending.cursor),
            "presentedFrameCount":pending.presented_frame_count,"surfaceEpoch":result.surface_epoch,"nativePresentIndex":result.present_index,
            "submission":result.submission,"receipt":result.receipt,"surfaceAlphaMode":result.surface_alpha_mode,"opaquePixels":result.opaque_pixels,
            "expiresAtUnixMs":held.map(|held|held.evidence["expiresAtUnixMs"].clone()),"gpuReadbackCompleted":result.failure.is_none()&&result.color8.is_some(),
            "failure":result.failure,"pixelFrameVerified":false});
        if let Some(scene) = pending.scene_image.as_ref() {
            let mut evidence = scene.evidence.clone();
            let image = result.image.as_ref();
            evidence["receipt"] = json!(image.and_then(|image| image.receipt));
            evidence["alphaMode"] =
                json!(image
                    .and_then(|image| image.alpha_mode)
                    .map(|mode| match mode {
                        zircon_runtime::rhi::UiSurfaceImageAlphaMode::Opaque => "opaque",
                        zircon_runtime::rhi::UiSurfaceImageAlphaMode::Premultiplied =>
                            "premultiplied",
                    }));
            evidence["opaquePixels"] = json!(image.and_then(|image| image.opaque_pixels));
            evidence["readbackCompleted"] = json!(
                result.failure.is_none()
                    && image.is_some_and(|image| image.failure.is_none() && image.color8.is_some())
            );
            evidence["failure"] = json!(result
                .failure
                .as_ref()
                .or_else(|| image.and_then(|image| image.failure.as_ref())));
            metadata["sceneImage"] = evidence;
        } else if let Some(reason) = pending.scene_unavailable_reason.as_ref() {
            metadata["sceneImageUnavailableReason"] = json!(reason);
        }
        if result.failure.is_some() {
            self.invalidate("normal-readback-invalidated");
        }
        Some(super::normal_frame_readback::NormalReadbackExport {
            directory: directory()?,
            metadata,
            result,
        })
    }

    pub(in crate::ui::retained_host::host_contract) fn invalidate(&mut self, reason: &str) {
        if let Some(held) = self.held.take() {
            let mut value = held.evidence;
            value["status"] = json!(reason);
            self.publish(&value);
        }
    }
    pub(in crate::ui::retained_host::host_contract) fn before_present(
        &mut self,
        cursor: HostPresentationGenerationCursor,
        size: (u32, u32),
    ) -> bool {
        let live_request = self.live_request();
        self.before_present_for_request(cursor, size, live_request.as_ref())
    }

    fn before_present_for_request(
        &mut self,
        cursor: HostPresentationGenerationCursor,
        size: (u32, u32),
        live_request: Option<&Request>,
    ) -> bool {
        let live_request = live_request.filter(|request| self.owns_request(request));
        let Some(held) = self.held.as_ref() else {
            return false;
        };
        let release = live_request
            .filter(|request| {
                request.release
                    && request.physical_size == [held.size.0, held.size.1]
                    && held.size == size
                    && self.publication_matches_request(request, &held.evidence)
            })
            .map(|request| request.nonce.as_str());
        let reason = if release == Some(held.nonce.as_str()) {
            Some("released")
        } else if live_request.is_some_and(|request| request.nonce != held.nonce) {
            Some("capture-request-replaced")
        } else if Instant::now() >= held.deadline {
            Some("deadline")
        } else if cursor != held.cursor || size != held.size {
            Some("presentation-generation-or-extent-changed")
        } else {
            None
        };
        if let Some(reason) = reason {
            let mut value = held.evidence.clone();
            value["status"] = json!(reason);
            self.publish(&value);
            self.held = None;
            return false;
        }
        true
    }
    pub(in crate::ui::retained_host::host_contract) fn after_successful_present(
        &mut self,
        cursor: HostPresentationGenerationCursor,
        size: (u32, u32),
        profile: &mut Value,
    ) {
        let Some(request) = self.live_request() else {
            return;
        };
        if request.release
            || self.consumed_nonce.as_deref() == Some(request.nonce.as_str())
            || (request.gpu_readback && !self.readback_current(cursor, size))
            || request.physical_size != [size.0, size.1]
            || profile["commandStreamFullRebuild"] != true
        {
            return;
        }
        let Some(frame) = profile["presentedFrameCount"]
            .as_u64()
            .filter(|frame| *frame > 0)
        else {
            return;
        };
        if let Some(pending) = self
            .readback_pending
            .as_mut()
            .filter(|pending| pending.nonce == request.nonce)
        {
            pending.presented_frame_count = Some(frame);
        }
        let now = Instant::now();
        let evidence = json!({"schema":"dev.zircon.editor.normal-capture-hold","version":1,
            "status":"held","nonce":request.nonce,"caseSha256":request.case_sha256,"pid":std::process::id(),
            "presentedFrameCount":frame,"physicalSize":[size.0,size.1],
            "presentationCursor":format!("{cursor:?}"),"expiresAtUnixMs":unix_ms()+request.deadline_ms,
            "acquiredAtUnixMs":unix_ms(),
            "authority":"normal-successful-gpu-present","pixelFrameVerified":false});
        capture_trace::record("capture_held", || {
            format!(
                "hwnd={:?} nonce={} frame={frame} cursor={cursor:?} size={size:?}",
                self.native_hwnd, request.nonce
            )
        });
        profile["normalWindowEvidence"]["captureHold"] = evidence.clone();
        self.consumed_nonce = Some(request.nonce.clone());
        self.held = Some(Held {
            nonce: request.nonce,
            cursor,
            size,
            deadline: now + Duration::from_millis(request.deadline_ms),
            evidence: evidence.clone(),
        });
        self.publish(&evidence);
    }
}
#[cfg(test)]
#[path = "tests/normal_capture_hold.rs"]
mod tests;

#[cfg(all(test, windows))]
#[path = "tests/owned_directory.rs"]
mod owned_directory_tests;

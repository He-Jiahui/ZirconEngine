//! A bounded diagnostic hold owned by the actual normal GPU presenter.
use crate::ui::retained_host::host_contract::normal_capture_redraw_trace as capture_trace;
use super::super::data::HostPresentationGenerationCursor;
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
        size: (u32, u32),
    ) -> Option<zircon_runtime::rhi::UiSurfaceFrameReadbackRequest> {
        let request = self.live_request()?;
        self.prepare_frame_readback_for_request(cursor, size, request)
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
            format!("hwnd={hwnd} nonce={} cursor={cursor:?} size={size:?}", request.nonce)
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
        });
        Some(zircon_runtime::rhi::UiSurfaceFrameReadbackRequest {
            nonce: request.nonce,
            physical_size: size,
            expected_draw_generation: None,
            deadline_ms: request.deadline_ms,
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
        let metadata = json!({"schema":"dev.zircon.editor.normal-frame-readback","version":1,"authority":"normal-actual-gpu-packet-and-readback-receipt","pid":std::process::id(),"hwnd":actual_hwnd,
            "nonce":pending.nonce,"caseSha256":pending.case_sha256,"physicalSize":[pending.size.0,pending.size.1],"presentationCursor":format!("{:?}",pending.cursor),
            "presentedFrameCount":pending.presented_frame_count,"surfaceEpoch":result.surface_epoch,"nativePresentIndex":result.present_index,
            "submission":result.submission,"receipt":result.receipt,"surfaceAlphaMode":result.surface_alpha_mode,"opaquePixels":result.opaque_pixels,
            "expiresAtUnixMs":held.map(|held|held.evidence["expiresAtUnixMs"].clone()),"gpuReadbackCompleted":result.failure.is_none()&&result.color8.is_some(),
            "failure":result.failure,"pixelFrameVerified":false});
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
        let release = live_request
            .filter(|request| request.release)
            .map(|request| request.nonce.as_str());
        let Some(held) = self.held.as_ref() else {
            return false;
        };
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
            format!("hwnd={:?} nonce={} frame={frame} cursor={cursor:?} size={size:?}", self.native_hwnd, request.nonce)
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
mod tests {
    use super::*;

    fn controller(hwnd: u64) -> NormalCaptureController {
        NormalCaptureController::for_native_target(Some(RenderNativeSurfaceTarget::Win32 {
            hwnd,
            hinstance: None,
        }))
    }

    fn held(deadline: Instant) -> NormalCaptureController {
        NormalCaptureController {
            native_hwnd: Some(9),
            held: Some(Held {
                nonce: "test".into(),
                cursor: HostPresentationGenerationCursor::default(),
                size: (100, 100),
                deadline,
                evidence: json!({"pid":std::process::id()}),
            }),
            consumed_nonce: Some("test".into()),
            ..Default::default()
        }
    }
    fn capture_request() -> Request {
        Request {
            nonce: "fresh".into(),
            case_sha256: "a".repeat(64),
            pid: std::process::id(),
            requested_at_unix_ms: unix_ms(),
            deadline_ms: 1000,
            physical_size: [100, 100],
            release: false,
            gpu_readback: true,
            hwnd: Some(9),
        }
    }

    #[test]
    fn idle_capture_demand_requires_a_new_nonce_and_matching_actual_extent() {
        let mut controller = controller(9);
        let mut value = capture_request();
        assert!(controller.request_requires_present(&value, (100, 100)));
        assert!(!controller.request_requires_present(&value, (101, 100)));
        value.hwnd = None;
        assert!(!controller.request_requires_present(&value, (100, 100)));
        value.hwnd = Some(9);
        value.release = true;
        assert!(!controller.request_requires_present(&value, (100, 100)));
        value.release = false;
        controller.attempted_readback_nonce = Some(value.nonce.clone());
        assert!(!controller.request_requires_present(&value, (100, 100)));
        controller.attempted_readback_nonce = None;
        controller.consumed_nonce = Some(value.nonce.clone());
        assert!(!controller.request_requires_present(&value, (100, 100)));
    }

    #[test]
    fn an_existing_hold_and_pending_readback_do_not_schedule_another_capture_present() {
        let mut controller = held(Instant::now() + Duration::from_secs(1));
        let mut value = capture_request();
        value.nonce = "test".into();
        assert!(!controller.request_requires_present(&value, (100, 100)));
        value.nonce = "fresh".into();
        assert!(controller.request_requires_present(&value, (100, 100)));
        controller.readback_pending = Some(PendingReadback {
            nonce: value.nonce.clone(),
            case_sha256: value.case_sha256.clone(),
            hwnd: 9,
            cursor: HostPresentationGenerationCursor::default(),
            size: (100, 100),
            deadline: Instant::now() + Duration::from_secs(1),
            presented_frame_count: None,
        });
        assert!(!controller.request_requires_present(&value, (100, 100)));
    }

    #[test]
    fn capture_requests_are_routed_before_a_foreign_presenter_consumes_the_nonce() {
        let cursor = HostPresentationGenerationCursor::default();
        let mut target = controller(9);
        let mut foreign = controller(10);
        let request = capture_request();
        assert!(target.request_requires_present(&request, (100, 100)));
        assert!(!foreign.request_requires_present(&request, (100, 100)));
        assert!(foreign
            .prepare_frame_readback_for_request(cursor, (100, 100), request)
            .is_none());
        assert!(foreign.attempted_readback_nonce.is_none());
        assert!(foreign.readback_pending.is_none());
        assert!(foreign.consumed_nonce.is_none());

        let prepared = target
            .prepare_frame_readback_for_request(cursor, (100, 100), capture_request())
            .expect("the requested native owner should prepare the actual readback");
        assert_eq!(prepared.nonce, "fresh");
        assert_eq!(target.readback_pending.as_ref().unwrap().hwnd, 9);
        assert_eq!(target.attempted_readback_nonce.as_deref(), Some("fresh"));

        // A foreign request must not consume the nonce: this presenter can still
        // prepare it when a request of the same PID and extent targets its HWND.
        let mut request = capture_request();
        request.hwnd = Some(10);
        assert!(foreign.request_requires_present(&request, (100, 100)));
        assert!(foreign
            .prepare_frame_readback_for_request(cursor, (100, 100), request)
            .is_some());
        assert_eq!(foreign.readback_pending.as_ref().unwrap().hwnd, 10);
    }

    #[test]
    fn unbound_and_zero_native_targets_cannot_consume_capture_requests() {
        let cursor = HostPresentationGenerationCursor::default();
        for mut controller in [NormalCaptureController::default(), controller(0)] {
            assert!(!controller.request_requires_present(&capture_request(), (100, 100)));
            assert!(controller
                .prepare_frame_readback_for_request(cursor, (100, 100), capture_request())
                .is_none());
            assert!(controller.attempted_readback_nonce.is_none());
            assert!(controller.readback_pending.is_none());
        }
    }

    #[test]
    fn foreign_replacement_and_release_leave_the_existing_owner_hold_intact() {
        let cursor = HostPresentationGenerationCursor::default();
        let mut owner = held(Instant::now() + Duration::from_secs(1));
        let mut foreign = capture_request();
        foreign.hwnd = Some(10);
        assert!(owner.before_present_for_request(cursor, (100, 100), Some(&foreign)));
        assert_eq!(owner.held.as_ref().unwrap().nonce, "test");
        foreign.release = true;
        foreign.nonce = "test".into();
        assert!(owner.before_present_for_request(cursor, (100, 100), Some(&foreign)));
        assert_eq!(owner.held.as_ref().unwrap().nonce, "test");

        let fresh = capture_request();
        assert!(!owner.before_present_for_request(cursor, (100, 100), Some(&fresh)));
        assert!(owner.held.is_none());
        assert!(owner.request_requires_present(&fresh, (100, 100)));

        let mut owner = held(Instant::now() + Duration::from_secs(1));
        let mut release = capture_request();
        release.nonce = "test".into();
        release.release = true;
        assert!(!owner.before_present_for_request(cursor, (100, 100), Some(&release)));
        assert!(owner.held.is_none());
    }

    #[test]
    fn obsolete_or_foreign_acknowledgements_cannot_replace_the_live_request_ack() {
        let owner = controller(9);
        let foreign = controller(10);
        let request = capture_request();
        let evidence = json!({
            "pid": request.pid,
            "nonce": request.nonce.as_str(),
            "caseSha256": request.case_sha256.as_str(),
        });
        assert!(owner.publication_matches_request(&request, &evidence));
        assert!(!foreign.publication_matches_request(&request, &evidence));

        let mut obsolete = evidence.clone();
        obsolete["nonce"] = json!("old-request");
        assert!(!owner.publication_matches_request(&request, &obsolete));
        obsolete = evidence.clone();
        obsolete["caseSha256"] = json!("b".repeat(64));
        assert!(!owner.publication_matches_request(&request, &obsolete));
        obsolete = evidence.clone();
        obsolete["pid"] = json!(u64::from(request.pid) + 1);
        assert!(!owner.publication_matches_request(&request, &obsolete));

        let mut next = capture_request();
        next.hwnd = Some(10);
        assert!(!owner.publication_matches_request(&next, &evidence));
        assert!(foreign.publication_matches_request(&next, &evidence));
    }

    #[test]
    fn generation_and_extent_changes_release_the_real_present_gate() {
        let cursor = HostPresentationGenerationCursor::default();
        let mut controller = held(Instant::now() + Duration::from_secs(1));
        assert!(controller.before_present(cursor, (100, 100)));
        assert!(!controller.before_present(
            HostPresentationGenerationCursor::new(1, 0, 0, 0, 0, 0),
            (100, 100)
        ));
        assert!(controller.held.is_none());
        let mut controller = held(Instant::now() + Duration::from_secs(1));
        assert!(!controller.before_present(cursor, (101, 100)));
    }
    #[test]
    fn expired_hold_does_not_delay_an_input_driven_present() {
        let mut controller = held(Instant::now() - Duration::from_millis(1));
        assert!(!controller.before_present(HostPresentationGenerationCursor::default(), (100, 100)));
        assert_eq!(controller.consumed_nonce.as_deref(), Some("test"));
    }
}

#[cfg(all(test, windows))]
mod owned_directory_tests {
    use super::approved_owned_directory;
    use std::path::Path;

    #[test]
    fn actual_windows_canonical_drive_root_keeps_verbatim_admission() {
        let canonical = [
            "D:/cargo-targets/zircon-local",
            "E:/cargo-targets/zircon-local",
            "F:/cargo-targets/zircon-local",
        ]
        .into_iter()
        .find_map(|root| Path::new(root).canonicalize().ok())
        .expect("Windows validation requires an actual approved physical output root");
        assert!(approved_owned_directory(&canonical), "{canonical:?}");
        assert!(approved_owned_directory(
            &canonical.join("penpot-workbench")
        ));
    }

    #[test]
    fn rejects_unc_other_drives_and_nested_lookalike_roots() {
        for path in [
            r"\\server\share\cargo-targets\zircon-local",
            r"\\?\UNC\server\share\cargo-targets\zircon-local",
            r"C:\cargo-targets\zircon-local",
            r"D:\nested\cargo-targets\zircon-local",
            r"D:\cargo-targets\zircon-local-other",
            r"D:\cargo-targets\zircon-local\..\other",
        ] {
            assert!(!approved_owned_directory(Path::new(path)), "{path}");
        }
        assert!(approved_owned_directory(Path::new(
            r"D:\cargo-targets\zircon-local"
        )));
        assert!(approved_owned_directory(Path::new(
            r"\\?\D:\cargo-targets\zircon-local"
        )));
    }
}

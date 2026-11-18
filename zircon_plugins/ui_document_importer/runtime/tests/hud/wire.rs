use std::path::Path;

use zircon_plugin_ui_document_importer_runtime::plugin_registration;
use zircon_runtime::{
    dynamic_api::{create_linked_runtime_session, zircon_runtime_get_api_v8},
    plugin::RuntimePluginRegistrationReport,
};
use zircon_runtime_interface::{
    reflect::ReflectedValue, ui::layout::UiFrame, validate_runtime_api_v8_shape,
    validate_runtime_frame_rgba_shape, ZrOwnedResultV2, ZrRuntimeAccessibilityTreeRequestV1,
    ZrRuntimeApiV8, ZrRuntimeEventV1, ZrRuntimeFrameDemandV1, ZrRuntimeFrameRequestV1,
    ZrRuntimeFrameV2, ZrRuntimeSessionHandle, ZrRuntimeViewportSizeV1, ZrStatus, ZrStatusCode,
    ZIRCON_RUNTIME_ABI_VERSION_V1 as V1, ZIRCON_RUNTIME_ABI_VERSION_V2 as V2,
    ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1 as VIEWPORT, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
    ZR_RUNTIME_BUTTON_STATE_RELEASED_V1, ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
    ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1,
};

use super::project::{Hud, Project, IMAGE_LABEL, LABEL};

pub(super) const WIDTH: u32 = 640;
pub(super) const HEIGHT: u32 = 360;
const SIZE: ZrRuntimeViewportSizeV1 = ZrRuntimeViewportSizeV1::new(WIDTH, HEIGHT);
const FRAME_BYTES: usize = WIDTH as usize * HEIGHT as usize * 4;

pub(super) struct Session {
    api: &'static ZrRuntimeApiV8,
    handle: ZrRuntimeSessionHandle,
    project: Project,
    tag: &'static str,
}

impl Session {
    pub fn open(
        tag: &'static str,
        project: Project,
        fixture: RuntimePluginRegistrationReport,
    ) -> Self {
        // SAFETY: the built-in export supports a null (default) host and returns a static table.
        let api = unsafe { zircon_runtime_get_api_v8(std::ptr::null()).as_ref() }
            .expect("exported V8 function table");
        validate_runtime_api_v8_shape(api).expect("V8 table shape");
        let importer = plugin_registration();
        for report in [&importer, &fixture] {
            assert!(
                report.diagnostics.is_empty(),
                "{tag} linked registration: {:?}",
                report.diagnostics
            );
        }
        let handle = create_linked_runtime_session(
            tag.as_bytes(),
            Some(project.root()),
            vec![importer, fixture],
        )
        .unwrap_or_else(|e| {
            panic!(
                "{tag} linked project bootstrap at {:?}: {e}",
                project.root()
            )
        });
        Self {
            api,
            handle,
            project,
            tag,
        }
    }

    pub fn tag(&self) -> &str {
        self.tag
    }

    pub fn project_root(&self) -> &Path {
        self.project.root()
    }

    pub fn resize(&self) {
        self.event(
            "viewport resize",
            ZrRuntimeEventV1::viewport_resized(V1, VIEWPORT, SIZE),
        );
    }

    fn event(&self, edge: &str, event: ZrRuntimeEventV1) -> ZrStatusCode {
        let call = self.api.handle_event.expect("V8 handle_event slot");
        // SAFETY: this published handle remains live and event constructors borrow no payload.
        let status = unsafe { call(self.handle, event) };
        self.ok(edge, status);
        status.status_code()
    }

    pub fn primary_click(&self, [x, y]: [f32; 2]) -> [ZrStatusCode; 3] {
        // Do not tick between edges or inject directly into InputManager.
        let moved = self.event(
            "pointer move",
            ZrRuntimeEventV1::pointer_moved(V1, VIEWPORT, x, y),
        );
        let button = |state| {
            ZrRuntimeEventV1::mouse_button(
                V1,
                VIEWPORT,
                ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
                state,
                x,
                y,
            )
        };
        let down = self.event("primary down", button(ZR_RUNTIME_BUTTON_STATE_PRESSED_V1));
        let up = self.event("primary up", button(ZR_RUNTIME_BUTTON_STATE_RELEASED_V1));
        [moved, down, up]
    }

    pub fn tick(&self) {
        let tick = self.api.tick_frame.expect("V8 tick_frame slot");
        let mut demand = ZrRuntimeFrameDemandV1::idle();
        // SAFETY: demand is a unique output on the caller stack and the session is live.
        self.ok("owner tick", unsafe { tick(self.handle, &mut demand) });
    }

    pub fn a11y(&self, hud: Hud) -> A11y {
        let capture = self
            .api
            .capture_accessibility_tree
            .expect("V8 accessibility slot");
        let mut output = ZrOwnedResultV2::empty();
        // SAFETY: request is a by-value V1 DTO; the fresh output is unique and live.
        let status = unsafe {
            capture(
                self.handle,
                ZrRuntimeAccessibilityTreeRequestV1::new(V1, VIEWPORT, SIZE, 0),
                &mut output,
            )
        };
        let lease = Allocation {
            session: self,
            output,
        };
        self.ok("accessibility capture", status);
        A11y::parse(&lease.bytes(16 * 1024), hud)
    }

    pub fn frame(&self) -> Frame {
        let capture = self.api.capture_frame.expect("V8 framebuffer capture slot");
        let mut output = ZrRuntimeFrameV2::empty(V2);
        // SAFETY: request is a by-value V1 DTO; the fresh V2 output is unique and live.
        let status = unsafe {
            capture(
                self.handle,
                ZrRuntimeFrameRequestV1::new(V1, VIEWPORT, SIZE),
                &mut output,
            )
        };
        let lease = Allocation {
            session: self,
            output: output.rgba,
        };
        self.ok("GPU framebuffer capture", status);
        assert_eq!(
            (output.abi_version, output.width, output.height),
            (V2, WIDTH, HEIGHT)
        );
        validate_runtime_frame_rgba_shape(output.width, output.height, lease.output.len)
            .expect("actual ABI RGBA shape");
        Frame {
            generation: output.generation,
            rgba: lease.bytes(FRAME_BYTES),
        }
    }

    fn ok(&self, step: &str, status: ZrStatus) {
        assert!(
            status.is_ok(),
            "{} {step}: {:?}: {}",
            self.tag,
            status.status_code(),
            diagnostics(status)
        );
    }
}

fn diagnostics(status: ZrStatus) -> String {
    // SAFETY: this is the local export's diagnostic borrow, copied before the next API call.
    let bytes = unsafe {
        status
            .diagnostics
            .checked_slice(ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1)
    };
    bytes
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .unwrap_or_else(|e| format!("{e:?}"))
}

impl Drop for Session {
    fn drop(&mut self) {
        let destroy = self.api.destroy_session.expect("V8 destroy_session slot");
        // SAFETY: release all output guards first, then close the published session once.
        let status = unsafe { destroy(self.handle) };
        if !status.is_ok() {
            self.project.preserve();
            if std::thread::panicking() {
                eprintln!("hud-public {} teardown: {}", self.tag, diagnostics(status));
            } else {
                self.ok("session teardown", status);
            }
        }
    }
}

struct Allocation<'a> {
    session: &'a Session,
    output: ZrOwnedResultV2,
}

impl Allocation<'_> {
    fn bytes(&self, bound: usize) -> Vec<u8> {
        let len = usize::try_from(self.output.len).expect("bounded host length");
        assert!(
            self.output.allocation.is_valid()
                && !self.output.data.is_null()
                && (1..=bound).contains(&len),
            "invalid runtime-owned allocation shape: {:?}",
            self.output
        );
        // SAFETY: an acknowledged local ABI allocation owns this range until this guard drops.
        unsafe { std::slice::from_raw_parts(self.output.data, len) }.to_vec()
    }
}

impl Drop for Allocation<'_> {
    fn drop(&mut self) {
        if self.output.allocation.is_valid() {
            let release = self
                .session
                .api
                .release_allocation
                .expect("V8 release slot");
            // SAFETY: drop returns this allocation exactly once to its originating live session.
            let status = unsafe { release(self.session.handle, self.output.allocation) };
            if !status.is_ok() && std::thread::panicking() {
                eprintln!("hud-public release during unwind: {}", diagnostics(status));
            } else {
                self.session.ok("release result", status);
            }
        }
    }
}

pub(super) struct Frame {
    pub generation: u64,
    pub rgba: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub(super) struct A11y {
    pub caption: Option<(u64, UiFrame)>,
    image: Option<(u64, UiFrame)>,
}

impl A11y {
    fn parse(bytes: &[u8], hud: Hud) -> Self {
        // Reuse the standard JSON Value FromStr implementation via the public interface
        // variant's argument type. The named fields below are checked at their DTO positions,
        // not found by substring search. No serde_json dependency or wire rewriting is needed.
        let text = std::str::from_utf8(bytes).expect("a11y JSON is UTF-8");
        let ReflectedValue::Json(value) = text
            .parse()
            .map(ReflectedValue::Json)
            .expect("a11y wire JSON object")
        else {
            unreachable!()
        };
        assert!(
            value.get("focused").is_some_and(|v| v.is_null()),
            "no HUD focus: {value}"
        );
        let roots = value["roots"].as_array().expect("DTO roots");
        let nodes = value["nodes"].as_array().expect("DTO nodes");
        assert_eq!(roots.len(), 1);
        let root_id = roots[0]
            .as_u64()
            .filter(|id| *id != 0)
            .expect("nonzero root ID");
        let root = nodes
            .iter()
            .find(|n| n["node_id"].as_u64() == Some(root_id))
            .expect("root DTO node");
        if hud == Hud::None {
            assert_eq!(
                value["tree_id"].as_str(),
                Some("zircon-runtime-dynamic-preview")
            );
            assert_eq!(nodes.len(), 1);
            assert_eq!(root["name"].as_str(), Some("Zircon Runtime Preview"));
            return Self {
                caption: None,
                image: None,
            }; // not a menu-pixel oracle
        }
        assert_eq!(value["tree_id"].as_str(), Some("zircon-runtime-project-ui"));
        let node = |name, role| {
            let mut hits = nodes.iter().filter(|n| n["name"].as_str() == Some(name));
            let n = hits
                .next()
                .unwrap_or_else(|| panic!("missing authored {role} {name:?}: {value}"));
            assert!(hits.next().is_none(), "non-unique authored label {name:?}");
            assert_eq!(n["role"].as_str(), Some(role));
            assert_eq!(n["state"]["hidden"].as_bool(), Some(false));
            assert_eq!(n["state"]["focused"].as_bool(), Some(false));
            assert!(n["actions"].as_array().expect("DTO actions").is_empty());
            let id = n["node_id"]
                .as_u64()
                .filter(|id| *id != 0)
                .expect("authored global node ID");
            let coord = |field| {
                n["bounds"]
                    .get(field)
                    .and_then(|v| v.as_f64())
                    .expect("DTO bounds coordinate") as f32
            };
            let bounds = UiFrame::new(coord("x"), coord("y"), coord("width"), coord("height"));
            assert!(
                bounds.x >= 0.0
                    && bounds.y >= 0.0
                    && bounds.width > 0.0
                    && bounds.height > 0.0
                    && bounds.x + bounds.width <= WIDTH as f32
                    && bounds.y + bounds.height <= HEIGHT as f32,
                "out-of-viewport a11y bounds {bounds:?}"
            );
            (id, bounds)
        };
        let caption = node(LABEL, "text");
        let image = if hud == Hud::TextAndImage {
            let image = node(IMAGE_LABEL, "image");
            let children = root["children"].as_array().expect("HUD overlay children");
            assert!(children.iter().any(|id| id.as_u64() == Some(caption.0)));
            assert!(children.iter().any(|id| id.as_u64() == Some(image.0)));
            assert_eq!(image.1, UiFrame::new(24.0, 24.0, 64.0, 32.0));
            assert_eq!(caption.1, UiFrame::new(120.0, 24.0, 250.0, 40.0));
            Some(image)
        } else {
            assert_eq!(caption.0, root_id);
            None
        };
        Self {
            caption: Some(caption),
            image,
        }
    }
}

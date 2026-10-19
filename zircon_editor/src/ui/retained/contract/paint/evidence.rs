use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Mutex, OnceLock};

use serde_json::{json, Value};
use zircon_runtime_interface::runtime_build_set::ZrRuntimeDigestV1;

use super::super::paint_theme::active_host_paint_token_input;
use super::render_commands::{HostPaintCommand, HostPaintCommandKind, PaintNodeIdentity};

#[derive(Clone)]
struct MediaFileReceipt {
    path: PathBuf,
    sha256: String,
}

#[derive(Default)]
struct StyleValues {
    background: BTreeSet<String>,
    foreground: BTreeSet<String>,
    border: BTreeSet<String>,
    border_width: BTreeSet<String>,
    radius: BTreeSet<String>,
    opacity: BTreeSet<String>,
    saw_background: bool,
    saw_foreground: bool,
    saw_border: bool,
    saw_shape: bool,
    shadow_scan_complete: bool,
    shadow_incomplete: bool,
    shadow_layers: Vec<Value>,
    identity: Option<Rc<PaintNodeIdentity>>,
}

#[derive(Default)]
struct Capture {
    repo: Option<PathBuf>,
    dpi: f32,
    allow_absolute_media: bool,
    design_token_input: Option<Value>,
    styles: BTreeMap<(String, String, String), StyleValues>,
    media: BTreeMap<(String, String, String, String), Value>,
    issues: Vec<String>,
    native_hierarchy_draws: Vec<Value>,
}

thread_local! {
    static CAPTURE_ACTIVE: Cell<bool> = const { Cell::new(false) };
    static CAPTURE: RefCell<Option<Capture>> = const { RefCell::new(None) };
    static ACTIVE_OWNER: RefCell<Option<Rc<PaintNodeIdentity>>> = const { RefCell::new(None) };
}

static MEDIA_FILES: OnceLock<Mutex<HashMap<String, Option<MediaFileReceipt>>>> = OnceLock::new();

pub(super) fn capture_active() -> bool {
    CAPTURE_ACTIVE.get()
}

pub(crate) struct PaintEvidenceScope(PhantomData<Rc<()>>);

impl PaintEvidenceScope {
    /// The normal product loads its admitted bundle/project files outside the
    /// repository. Preserve those actual resolver receipts without changing the
    /// stricter source-relative catalog capture contract.
    pub(crate) fn begin_for_normal_present(repo: &Path) -> Result<Self, String> {
        let scope = Self::begin(repo, 1.0)?;
        CAPTURE.with_borrow_mut(|capture| {
            if let Some(capture) = capture {
                capture.allow_absolute_media = true;
                capture.design_token_input = active_host_paint_token_input();
            }
        });
        Ok(scope)
    }

    pub(crate) fn begin(repo: &Path, dpi: f32) -> Result<Self, String> {
        let repo = repo.canonicalize().map_err(|error| error.to_string())?;
        if !dpi.is_finite() || dpi <= 0.0 {
            return Err("paint evidence needs a finite positive DPI".into());
        }
        CAPTURE.with_borrow_mut(|capture| {
            if capture.is_some() {
                return Err("paint evidence capture is already active".into());
            }
            *capture = Some(Capture {
                repo: Some(repo),
                dpi,
                ..Capture::default()
            });
            CAPTURE_ACTIVE.set(true);
            Ok(Self(PhantomData))
        })
    }

    pub(crate) fn finish(self) -> Result<Value, String> {
        let mut capture = CAPTURE.with_borrow_mut(|capture| {
            capture.take().ok_or("paint evidence capture is not active")
        })?;
        let styles = capture
            .styles
            .into_iter()
            .map(|((source_path, source_node_id, instance_path), values)| {
                let identity = values.identity.unwrap_or_else(|| {
                    Rc::new(PaintNodeIdentity {
                        node_id: String::new(),
                        parent_node_id: None,
                        source_path: source_path.clone(),
                        source_node_id: source_node_id.clone(),
                        instance_path: instance_path.clone(),
                        parent_source_path: None,
                        parent_source_node_id: None,
                        parent_instance_path: None,
                        control_id: None,
                    })
                });
                let (background, background_complete) =
                    unique_color(values.background, values.shadow_scan_complete);
                let (foreground, foreground_complete) =
                    unique_color(values.foreground, values.shadow_scan_complete);
                let (border, border_complete) =
                    unique_color(values.border, values.shadow_scan_complete);
                let (border_width, width_complete) =
                    unique_number(values.border_width, "0", values.shadow_scan_complete);
                let (border_radius, radius_complete) =
                    unique_number(values.radius, "0", values.shadow_scan_complete);
                let (opacity, opacity_complete) =
                    unique_number(values.opacity, "1", values.shadow_scan_complete);
                let box_shadow_complete = values.shadow_scan_complete && !values.shadow_incomplete;
                let style_complete = values.saw_shape
                    && background_complete
                    && foreground_complete
                    && border_complete
                    && width_complete
                    && radius_complete
                    && opacity_complete;
                let inventory_complete = style_complete && box_shadow_complete;
                let properties = json!({
                    "foregroundColor": foreground,
                    "backgroundColor": background,
                    "borderColor": border,
                    "borderWidth": border_width,
                    "borderRadius": border_radius,
                    "opacity": opacity,
                });
                let style_inventory = json!({
                    "version": 1,
                    "complete": inventory_complete,
                    "properties": {
                        "foregroundColor": properties["foregroundColor"],
                        "backgroundColor": properties["backgroundColor"],
                        "borderColor": properties["borderColor"],
                        "borderWidth": properties["borderWidth"],
                        "borderRadius": properties["borderRadius"],
                        "opacity": properties["opacity"],
                        "boxShadow": {
                            "complete": box_shadow_complete,
                            "layers": values.shadow_layers,
                        },
                    },
                });
                json!({
                    "nodeId": identity.node_id,
                    "sourcePath": source_path,
                    "sourceNodeId": source_node_id,
                    "instancePath": instance_path,
                    "parentNodeId": identity.parent_node_id,
                    "controlId": identity.control_id,
                    "parentSourcePath": identity.parent_source_path,
                    "parentSourceNodeId": identity.parent_source_node_id,
                    "parentInstancePath": identity.parent_instance_path,
                    "effectiveStyle": {"complete": style_complete, "properties": properties},
                    "styleInventory": style_inventory,
                })
            })
            .collect::<Vec<_>>();

        let runtime_asset_fingerprints = capture
            .media
            .values()
            .filter_map(|resource| {
                Some((
                    resource.get("path")?.as_str()?.to_owned(),
                    resource.get("sha256")?.as_str()?.to_owned(),
                ))
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|(path, sha)| json!([path, sha]))
            .collect::<Vec<_>>();
        let mut resources = capture.media.into_values().collect::<Vec<_>>();
        if let Some(repo) = capture.repo.as_deref() {
            verify_current_media_files(
                repo,
                &mut resources,
                &mut capture.issues,
                capture.allow_absolute_media,
            );
        }
        let asset_complete = capture.issues.is_empty()
            && resources.iter().all(|resource| {
                resource["loaded"].as_bool() == Some(true)
                    && resource["currentFileMatches"].as_bool() == Some(true)
                    && resource["sourcePath"].as_str().is_some()
                    && resource["sourceNodeId"].as_str().is_some()
                    && resource["instancePath"].as_str().is_some()
            });
        Ok(json!({
            "schema": "dev.zircon.editor.paint-evidence",
            "version": 1,
            "coordinateSpace": "physical-pixels",
            "designTokenInput": capture.design_token_input,
            "styles": styles,
            "nativeHierarchyDraws": capture.native_hierarchy_draws,
            "assetAudit": {"complete": asset_complete, "resources": resources},
            "runtimeAssetFingerprints": runtime_asset_fingerprints,
            "issues": capture.issues,
        }))
    }
}

impl Drop for PaintEvidenceScope {
    fn drop(&mut self) {
        CAPTURE_ACTIVE.set(false);
        CAPTURE.with_borrow_mut(|capture| *capture = None);
        ACTIVE_OWNER.with_borrow_mut(|owner| *owner = None);
    }
}

pub(super) struct PaintNodeOwnerScope(Option<Rc<PaintNodeIdentity>>);

impl PaintNodeOwnerScope {
    pub(super) fn enter(owner: Option<&Rc<PaintNodeIdentity>>) -> Self {
        if !capture_active() {
            return Self(None);
        }
        let previous =
            ACTIVE_OWNER.with_borrow_mut(|active| std::mem::replace(active, owner.cloned()));
        Self(previous)
    }
}

impl Drop for PaintNodeOwnerScope {
    fn drop(&mut self) {
        ACTIVE_OWNER.with_borrow_mut(|active| *active = self.0.take());
    }
}

pub(in crate::ui::retained_host::host_contract) fn active_owner() -> Option<Rc<PaintNodeIdentity>> {
    ACTIVE_OWNER.with_borrow(|owner| owner.clone())
}

pub(super) fn record_identity_issue(issue: String) {
    CAPTURE.with_borrow_mut(|capture| {
        if let Some(capture) = capture {
            capture.issues.push(issue);
        }
    });
}

pub(super) fn record_owner_commands_complete(owner: &Rc<PaintNodeIdentity>) {
    CAPTURE.with_borrow_mut(|capture| {
        let Some(capture) = capture else { return };
        let values = capture.styles.entry(identity_key(owner)).or_default();
        values.identity = Some(owner.clone());
        values.shadow_scan_complete = true;
    });
}

pub(super) fn record_box_shadow_command(command: &HostPaintCommand) {
    if !capture_active() {
        return;
    }
    let Some(geometry) = command.box_shadow else {
        return;
    };
    let Some(owner) = command.owner.as_ref() else {
        record_identity_issue("box shadow command has no authored owner".into());
        return;
    };
    CAPTURE.with_borrow_mut(|capture| {
        let Some(capture) = capture else { return };
        let key = identity_key(owner);
        let dpi = capture.dpi;
        let values = capture.styles.entry(key).or_default();
        values.identity = Some(owner.clone());
        let Some(color) = command.background_color else {
            values.shadow_incomplete = true;
            capture
                .issues
                .push("box shadow command has no final fill color".into());
            return;
        };
        let numbers = [
            geometry.offset_x,
            geometry.offset_y,
            geometry.blur_radius,
            geometry.spread_radius,
            geometry.radius,
            command.opacity,
        ];
        if numbers.iter().any(|value| !value.is_finite()) || dpi <= 0.0 {
            values.shadow_incomplete = true;
            capture
                .issues
                .push("box shadow command has non-finite resolved values".into());
            return;
        }
        // `draw_quad_command` multiplies source alpha by the command opacity
        // before blending. Record that same effective alpha in the CSS-style
        // shadow receipt; keeping only the palette alpha would overstate
        // shadows whose node or material opacity is below one.
        let alpha = (f32::from(color[3]) / 255.0) * command.opacity.clamp(0.0, 1.0);
        values.shadow_layers.push(json!({
            "offsetX": geometry.offset_x / dpi,
            "offsetY": geometry.offset_y / dpi,
            "blurRadius": geometry.blur_radius / dpi,
            "spreadRadius": geometry.spread_radius / dpi,
            "radius": geometry.radius / dpi,
            "color": format!("rgba({},{},{},1)", color[0], color[1], color[2]),
            "opacity": alpha,
            "inset": geometry.inset,
        }));
    });
}

pub(in crate::ui::retained_host::host_contract) fn record_native_hierarchy_row(
    node: &super::super::data::SceneNodeData,
    index: usize,
    row: &super::super::data::FrameRect,
    clip: &super::super::data::FrameRect,
    commands: &[super::super::paint_frame::HostRecordedPaintCommand],
) {
    use super::super::paint_frame::HostRecordedPaintKind;
    CAPTURE.with_borrow_mut(|capture| {
        let Some(capture)=capture else { return; };
        if !capture.allow_absolute_media || commands.is_empty() { return; }
        let commands=commands.iter().map(|command| {
            let paint=match &command.kind {
                HostRecordedPaintKind::Quad{color,corner_radius}=>json!({"kind":"quad","color":color,"cornerRadius":corner_radius}),
                HostRecordedPaintKind::Border{color,width,corner_radius}=>json!({"kind":"border","color":color,"width":width,"cornerRadius":corner_radius}),
                HostRecordedPaintKind::Text{text,color,font_size,line_height,font_request,..}=>json!({"kind":"text","text":text,"color":color,"fontSize":font_size,"lineHeight":line_height,"fontFamily":font_request.as_ref().map(|request|request.family.as_str()),"fontWeight":font_request.as_ref().map(|request|request.weight)}),
                HostRecordedPaintKind::Image{resource_key,resource_generation,width,height,rgba,..}=>json!({"kind":"image","resourceKey":resource_key,"generation":resource_generation,"width":width,"height":height,"hasUploadBytes":rgba.is_some()}),
            };
            json!({"bounds":{"x":command.frame.x,"y":command.frame.y,"width":command.frame.width,"height":command.frame.height},
                "clip":command.clip_frame.as_ref().map(|rect|json!({"x":rect.x,"y":rect.y,"width":rect.width,"height":rect.height})),"paint":paint})
        }).collect::<Vec<_>>();
        capture.native_hierarchy_draws.push(json!({"dataIdentity":{"nodeId":node.id.as_str(),"index":index},
            "controlId":"HierarchyListPanel","modelName":node.name.as_str(),"depth":node.depth,"selected":node.selected,
            "bounds":{"x":row.x,"y":row.y,"width":row.width,"height":row.height},
            "clip":{"x":clip.x,"y":clip.y,"width":clip.width,"height":clip.height},
            "authority":"actual-native-hierarchy-row-draw","commands":commands}));
    });
}

pub(super) fn record_drawn_command(command: &HostPaintCommand) {
    let Some(owner) = command.owner.as_ref() else {
        return;
    };
    if command.box_shadow.is_some() {
        return;
    }
    CAPTURE.with_borrow_mut(|capture| {
        let Some(capture) = capture else { return };
        let key = identity_key(owner);
        let dpi = capture.dpi;
        let values = capture.styles.entry(key).or_default();
        values.identity = Some(owner.clone());
        values.saw_shape = true;
        values.opacity.insert(number(command.opacity));

        if let Some(color) = command.background_color {
            values.saw_background = true;
            values.background.insert(css_color(color));
        }
        if let Some(color) = command.foreground_color {
            values.saw_foreground = true;
            values.foreground.insert(css_color(color));
        }
        let image_is_raster =
            matches!(command.kind, HostPaintCommandKind::Image) && command.image_pixels.is_some();
        if !image_is_raster {
            if command.border_width > 0.0 {
                if let Some(color) = command.border_color {
                    values.saw_border = true;
                    values.border.insert(css_color(color));
                }
            }
            if command.border_width > 0.0 {
                values
                    .border_width
                    .insert(number(command.border_width / dpi));
            }
            if command.corner_radius > 0.0 {
                values.radius.insert(number(command.corner_radius / dpi));
            }
        }
    });
}

pub(super) fn register_media_source(resource_key: &str, path: &Path) {
    let key = MEDIA_FILES.get_or_init(|| Mutex::new(HashMap::new()));
    let receipt = (|| {
        let canonical = path.canonicalize().ok()?;
        let bytes = std::fs::read(&canonical).ok()?;
        let sha256 = ZrRuntimeDigestV1::sha256(&bytes).as_str().to_owned();
        Some(MediaFileReceipt {
            path: canonical,
            sha256,
        })
    })();
    if let Ok(mut files) = key.lock() {
        match (files.get(resource_key), receipt) {
            (None, receipt) => {
                files.insert(resource_key.to_owned(), receipt);
            }
            (Some(Some(old)), Some(new)) if old.path == new.path && old.sha256 == new.sha256 => {}
            _ => {
                files.insert(resource_key.to_owned(), None);
            }
        }
    }
}

pub(super) fn record_media_use(resource_key: &str) {
    if !capture_active() {
        return;
    }
    let owner = active_owner();
    CAPTURE.with_borrow_mut(|capture| {
        let Some(capture) = capture else { return };
        let Some(owner) = owner.as_ref() else {
            capture.issues.push(format!(
                "media resource {resource_key} has no authored owner"
            ));
            return;
        };
        let registry = MEDIA_FILES.get_or_init(|| Mutex::new(HashMap::new()));
        let file = registry
            .lock()
            .ok()
            .and_then(|files| files.get(resource_key).cloned());
        let Some(Some(file)) = file else {
            capture.issues.push(format!(
                "media resource {resource_key} has no unambiguous source file receipt"
            ));
            return;
        };
        let Some(repo) = capture.repo.as_ref() else {
            return;
        };
        let path = match file.path.strip_prefix(repo) {
            Ok(relative) => relative.to_string_lossy().replace('\\', "/"),
            Err(_) if capture.allow_absolute_media => {
                file.path.to_string_lossy().replace('\\', "/")
            }
            Err(_) => {
                capture.issues.push(format!(
                    "media resource {resource_key} is outside the captured repository"
                ));
                return;
            }
        };
        let (source_path, node_id, instance_path) = identity_key(owner);
        let key = (
            source_path.clone(),
            node_id.clone(),
            instance_path.clone(),
            path.clone(),
        );
        capture.media.insert(
            key,
            json!({
                "kind": "image",
                "resourceKey": resource_key,
                "path": path,
                "sha256": file.sha256,
                "loaded": true,
                "sourcePath": source_path,
                "sourceNodeId": node_id,
                "instancePath": instance_path,
            }),
        );
    });
}

pub(super) fn record_media_not_ready(resource_key: Option<&str>) {
    CAPTURE.with_borrow_mut(|capture| {
        if let Some(capture) = capture {
            capture.issues.push(match resource_key {
                Some(resource_key) if !resource_key.is_empty() => format!(
                    "image resource {resource_key} was painted without ready file-backed pixels"
                ),
                _ => "image command was painted without a ready resource identity".into(),
            });
        }
    });
}

fn verify_current_media_files(
    repo: &Path,
    resources: &mut [Value],
    issues: &mut Vec<String>,
    allow_absolute: bool,
) {
    for resource in resources {
        let path = resource.get("path").and_then(Value::as_str);
        let sha256 = resource.get("sha256").and_then(Value::as_str);
        let matches = path
            .zip(sha256)
            .filter(|(path, sha256)| {
                !path.is_empty()
                    && (allow_absolute || !Path::new(path).is_absolute())
                    && !Path::new(path)
                        .components()
                        .any(|component| matches!(component, std::path::Component::ParentDir))
                    && sha256.len() == 64
            })
            .and_then(|(path, expected)| {
                let current = repo.join(path).canonicalize().ok()?;
                if !allow_absolute && !current.starts_with(repo) {
                    return None;
                }
                let bytes = std::fs::read(current).ok()?;
                Some(ZrRuntimeDigestV1::sha256(&bytes).as_str() == expected)
            })
            .unwrap_or(false);
        resource["currentFileMatches"] = json!(matches);
        if !matches {
            resource["loaded"] = json!(false);
            issues.push(format!(
                "media resource {} changed or became unavailable during capture",
                resource["path"].as_str().unwrap_or("<unknown>")
            ));
        }
    }
}

pub(in crate::ui::retained_host::host_contract) fn owner_json(
    owner: Option<&PaintNodeIdentity>,
) -> Value {
    owner.map_or(Value::Null, |owner| {
        json!({
            "nodeId": owner.node_id,
            "sourcePath": owner.source_path,
            "sourceNodeId": owner.source_node_id,
            "instancePath": owner.instance_path,
            "controlId": owner.control_id,
            "parentNodeId": owner.parent_node_id,
            "parentSourcePath": owner.parent_source_path,
            "parentSourceNodeId": owner.parent_source_node_id,
            "parentInstancePath": owner.parent_instance_path,
        })
    })
}

fn identity_key(owner: &PaintNodeIdentity) -> (String, String, String) {
    (
        owner.source_path.clone(),
        owner.source_node_id.clone(),
        owner.instance_path.clone(),
    )
}

fn unique_color(values: BTreeSet<String>, absence_proven: bool) -> (Value, bool) {
    match values.len() {
        0 => (Value::Null, absence_proven),
        1 => (
            Value::String(values.into_iter().next().unwrap_or_default()),
            true,
        ),
        _ => (Value::Null, false),
    }
}

fn unique_number(values: BTreeSet<String>, absent: &str, absence_proven: bool) -> (Value, bool) {
    match values.len() {
        0 if absence_proven => (serde_json::from_str(absent).unwrap_or(Value::Null), true),
        0 => (Value::Null, false),
        1 => (
            serde_json::from_str(&values.into_iter().next().unwrap_or_default())
                .unwrap_or(Value::Null),
            true,
        ),
        _ => (Value::Null, false),
    }
}

fn number(value: f32) -> String {
    let mut value = format!("{value:.6}");
    while value.contains('.') && value.ends_with('0') {
        value.pop();
    }
    if value.ends_with('.') {
        value.pop();
    }
    value
}

fn css_color(color: [u8; 4]) -> String {
    let alpha = f32::from(color[3]) / 255.0;
    format!(
        "rgba({},{},{},{})",
        color[0],
        color[1],
        color[2],
        number(alpha)
    )
}

#[cfg(test)]
#[path = "tests/evidence.rs"]
mod tests;

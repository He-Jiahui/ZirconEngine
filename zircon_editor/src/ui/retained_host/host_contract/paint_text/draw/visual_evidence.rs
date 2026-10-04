use std::cell::RefCell;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde_json::{json, Value};
use zircon_runtime::core::framework::text::{TextGlyphBitmapFormat, TextGlyphRasterReceipt};
use zircon_runtime::ui::surface::UiTextGlyphArtifactRasterFace;
use zircon_runtime_interface::runtime_build_set::ZrRuntimeDigestV1;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::data::FrameRect;
use super::super::super::paint_geometry::PixelRect;
use super::super::font::{font_face_for_paint_style, font_request_for_face};
use super::super::layout_policy::HostTextLayoutPolicy;
use super::layout::{centered_line_y, PaintTextLayout, RuntimeTextGlyph};
use crate::ui::retained_host::host_contract::paint_template_nodes::{active_owner, owner_json};

#[derive(Default)]
struct Capture {
    runs: Vec<Value>,
    faces: BTreeMap<String, Value>,
    active_run: Option<usize>,
    repo: PathBuf,
    dpi: f32,
}

thread_local! {
    static CAPTURE: RefCell<Option<Capture>> = const { RefCell::new(None) };
}

pub(crate) struct TextPaintEvidenceScope(PhantomData<Rc<()>>);

impl TextPaintEvidenceScope {
    pub(crate) fn begin() -> Result<Self, String> {
        let repo = std::env::current_dir().map_err(|error| error.to_string())?;
        Self::begin_for_capture(&repo, 1.0)
    }

    pub(crate) fn begin_for_capture(repo: &Path, dpi: f32) -> Result<Self, String> {
        let repo = repo.canonicalize().map_err(|error| error.to_string())?;
        if !dpi.is_finite() || dpi <= 0.0 {
            return Err("text evidence needs a finite positive DPI".into());
        }
        CAPTURE.with_borrow_mut(|capture| {
            if capture.is_some() {
                return Err("text paint evidence capture is already active".into());
            }
            *capture = Some(Capture {
                runs: Vec::new(),
                faces: BTreeMap::new(),
                active_run: None,
                repo,
                dpi,
            });
            Ok(Self(PhantomData))
        })
    }

    pub(crate) fn finish(self) -> Result<Value, String> {
        CAPTURE.with_borrow_mut(|capture| {
            let captured = capture
                .take()
                .ok_or("text paint evidence capture is not active")?;
            let nodes = native_text_nodes(&captured);
            let runtime_asset_fingerprints = used_font_fingerprints(&nodes);
            let font_loaded = !nodes.is_empty()
                && nodes.iter().all(|node| {
                    node["fonts"].as_array().is_some_and(|faces| {
                        !faces.is_empty()
                            && faces.iter().all(|face| {
                                face["resourcePath"]
                                    .as_str()
                                    .is_some_and(|path| !path.is_empty())
                                    && face["sha256"].as_str().is_some_and(|sha| sha.len() == 64)
                                    && face["glyphCount"].as_u64().unwrap_or(0) > 0
                                    && face["currentFileMatches"].as_bool() == Some(true)
                            })
                    })
                });
            Ok(json!({
                "schema": "dev.zircon.editor.text-paint-evidence", "version": 1,
                "coordinateSpace": "physical-pixels",
                "faces": captured.faces, "runs": captured.runs,
                "nodes": nodes,
                "fontAudit": {"loaded": font_loaded},
                "runtimeAssetFingerprints": runtime_asset_fingerprints,
            }))
        })
    }
}

impl Drop for TextPaintEvidenceScope {
    fn drop(&mut self) {
        CAPTURE.with_borrow_mut(|capture| *capture = None);
    }
}

pub(super) struct TextRunScope(Option<usize>);

impl Drop for TextRunScope {
    fn drop(&mut self) {
        CAPTURE.with_borrow_mut(|capture| {
            if let Some(capture) = capture {
                capture.active_run = self.0;
            }
        });
    }
}

pub(super) fn begin_run(
    source: &str,
    rect: &FrameRect,
    clip: &PixelRect,
    font_size: f32,
    line_height: f32,
    style: UiTextRunPaintStyle,
    layout: &PaintTextLayout,
    layout_policy: HostTextLayoutPolicy,
) -> TextRunScope {
    CAPTURE.with_borrow_mut(|capture| {
        let Some(capture) = capture else { return TextRunScope(None) };
        let request = font_request_for_face(font_face_for_paint_style(style));
        let previous = capture.active_run.replace(capture.runs.len());
        let errors = if layout.glyphs.is_empty() && !source.trim().is_empty() {
            vec!["nonempty source text produced no positioned glyphs"]
        } else { Vec::new() };
        capture.runs.push(json!({
            "sourceText": source, "displayText": layout.display_text,
            "owner": owner_json(active_owner().as_ref()),
            "bounds": {"x": rect.x, "y": rect.y, "width": rect.width, "height": rect.height},
            "clip": bounds_json([i64::from(clip.x0), i64::from(clip.y0), i64::from(clip.x1), i64::from(clip.y1)]),
            "requestedFamily": request.family, "requestedWeight": request.weight,
            "fontSize": font_size, "lineHeight": line_height,
            "fontFamily": request.family, "fontWeight": request.weight,
            "fontStyle": if style.emphasis { "italic" } else { "normal" },
            "measuredLines": measured_line_records(rect, line_height, layout, layout_policy),
            "positionedGlyphCount": layout.glyphs.len(), "glyphs": [], "errors": errors,
        }));
        TextRunScope(previous)
    })
}

pub(super) fn failure(message: &str) {
    CAPTURE.with_borrow_mut(|capture| {
        let Some(capture) = capture else { return };
        if let Some(index) = capture.active_run {
            append_run_value(capture, index, "errors", json!(message));
        }
    });
}

fn measured_line_records(
    rect: &FrameRect,
    line_height: f32,
    layout: &PaintTextLayout,
    layout_policy: HostTextLayoutPolicy,
) -> Vec<Value> {
    layout
        .measured_lines
        .iter()
        .map(|line| {
            let x = rect.x + line.frame_x;
            let y = match layout_policy {
                HostTextLayoutPolicy::SingleLineEllipsis => {
                    centered_line_y(rect.y, rect.height, line_height) + line.frame_y
                }
                HostTextLayoutPolicy::WordWrap => rect.y + line.frame_y,
            };
            json!({
                "text": line.text,
                "frame": {
                    "x": f64::from(x), "y": f64::from(y),
                    "width": f64::from(line.frame_width),
                    "height": f64::from(line.frame_height),
                }
            })
        })
        .collect()
}

pub(super) fn glyph(
    face: &UiTextGlyphArtifactRasterFace,
    glyph: &RuntimeTextGlyph,
    receipt: &TextGlyphRasterReceipt,
    origin: [i32; 2],
    clip: &PixelRect,
    color_alpha: u8,
) {
    CAPTURE.with_borrow_mut(|capture| {
        let Some(capture) = capture else { return };
        let Some(index) = capture.active_run else {
            return;
        };
        let face_key = format!(
            "{:02x?}:{}:{}:{:?}:{:?}",
            face.source_identity(),
            face.collection_index(),
            face.font_generation(),
            face.font_instance(),
            face.variations().map(|coords| &coords.0)
        );
        let repo = capture.repo.clone();
        capture.faces.entry(face_key.clone()).or_insert_with(|| {
            let receipt = face.font_receipt();
            let resource_path = receipt
                .resource_path
                .as_deref()
                .and_then(|path| path.canonicalize().ok())
                .and_then(|path| path.strip_prefix(&repo).ok().map(Path::to_path_buf))
                .map(|path| path.to_string_lossy().replace('\\', "/"));
            let resource_sha256 = receipt.resource_sha256.map(|sha| hex(&sha));
            let current_file_matches = receipt
                .resource_path
                .as_deref()
                .and_then(|path| std::fs::read(path).ok())
                .and_then(|bytes| Some(ZrRuntimeDigestV1::sha256(&bytes).as_str().to_owned()))
                .zip(resource_sha256.as_ref())
                .is_some_and(|(actual, expected)| &actual == expected);
            json!({
                "sourceSha256": ZrRuntimeDigestV1::sha256(face.bytes().as_ref()).as_str(),
                "collectionIndex": face.collection_index(), "fontGeneration": face.font_generation(),
                "sourceIdentity": face.source_identity(),
                "fontFace": format!("{:?}", face.font_face()),
                "fontInstance": face.font_instance().map(|handle| format!("{handle:?}")),
                "variations": face.variations().map(|coords| &coords.0),
                "familyName": receipt.family_name,
                "postScriptName": receipt.postscript_name,
                "faceIndex": receipt.face_index,
                "resourcePath": resource_path,
                "sha256": resource_sha256,
                "rasterSha256": hex(&receipt.raster_sha256),
                "currentFileMatches": current_file_matches,
            })
        });
        let (ink, visible) = ink_bounds(
            &receipt.bitmap,
            receipt.size,
            receipt.format,
            origin,
            clip,
            color_alpha,
        );
        if glyph.glyph_id == 0 {
            append_run_value(capture, index, "errors", json!("missing glyph (.notdef)"));
        }
        append_run_value(
            capture,
            index,
            "glyphs",
            json!({
                "glyphId": glyph.glyph_id, "face": face_key, "physicalPpem": glyph.physical_ppem,
                "originX": glyph.origin_x, "baselineY": glyph.baseline_y,
                "inkBounds": ink.map(bounds_json), "visibleInkBounds": visible.map(bounds_json),
                "clipped": ink != visible,
            }),
        );
    });
}

fn append_run_value(capture: &mut Capture, index: usize, field: &str, value: Value) {
    if let Some(values) = capture
        .runs
        .get_mut(index)
        .and_then(|run| run.get_mut(field))
        .and_then(Value::as_array_mut)
    {
        values.push(value);
    }
}

fn ink_bounds(
    bitmap: &[u8],
    size: [u32; 2],
    format: TextGlyphBitmapFormat,
    origin: [i32; 2],
    clip: &PixelRect,
    color_alpha: u8,
) -> (Option<[i64; 4]>, Option<[i64; 4]>) {
    let mut ink = None;
    let mut visible = None;
    let channels = match format {
        TextGlyphBitmapFormat::AlphaMask => 1,
        TextGlyphBitmapFormat::SubpixelMask | TextGlyphBitmapFormat::ColorRgba => 4,
    };
    let pixels = (size[0] as usize).saturating_mul(size[1] as usize);
    for (index, pixel) in bitmap.chunks_exact(channels).take(pixels).enumerate() {
        let painted = match format {
            TextGlyphBitmapFormat::AlphaMask => {
                u16::from(pixel[0]) * u16::from(color_alpha) / 255 != 0
            }
            TextGlyphBitmapFormat::SubpixelMask => {
                color_alpha != 0 && pixel[..3].iter().any(|value| *value != 0)
            }
            TextGlyphBitmapFormat::ColorRgba => {
                u16::from(pixel[3]) * u16::from(color_alpha) / 255 != 0
            }
        };
        if !painted || size[0] == 0 {
            continue;
        }
        let x = i64::from(origin[0]) + (index % size[0] as usize) as i64;
        let y = i64::from(origin[1]) + (index / size[0] as usize) as i64;
        extend(&mut ink, x, y);
        if x >= i64::from(clip.x0)
            && x < i64::from(clip.x1)
            && y >= i64::from(clip.y0)
            && y < i64::from(clip.y1)
        {
            extend(&mut visible, x, y);
        }
    }
    (ink, visible)
}

fn extend(bounds: &mut Option<[i64; 4]>, x: i64, y: i64) {
    *bounds = Some(match *bounds {
        Some([x0, y0, x1, y1]) => [x0.min(x), y0.min(y), x1.max(x + 1), y1.max(y + 1)],
        None => [x, y, x + 1, y + 1],
    });
}

fn bounds_json([x0, y0, x1, y1]: [i64; 4]) -> Value {
    json!({"x": x0, "y": y0, "width": x1 - x0, "height": y1 - y0})
}

fn hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn native_text_nodes(capture: &Capture) -> Vec<Value> {
    let mut grouped = BTreeMap::<String, Value>::new();
    for run in &capture.runs {
        let Some(owner) = run.get("owner").filter(|owner| owner.is_object()) else {
            continue;
        };
        let Some(source_path) = owner.get("sourcePath").and_then(Value::as_str) else {
            continue;
        };
        let Some(node_id) = owner.get("sourceNodeId").and_then(Value::as_str) else {
            continue;
        };
        let Some(instance_path) = owner.get("instancePath").and_then(Value::as_str) else {
            continue;
        };
        let key = format!("{source_path}\u{001f}{node_id}\u{001f}{instance_path}");
        let fonts = run_fonts(run, &capture.faces);
        let mut lines = run
            .get("measuredLines")
            .cloned()
            .unwrap_or_else(|| json!([]));
        if let Some(lines) = lines.as_array_mut() {
            for line in lines {
                for key in ["x", "y", "width", "height"] {
                    if let Some(value) = line["frame"][key].as_f64() {
                        line["frame"][key] = json!(value / f64::from(capture.dpi));
                    }
                }
            }
        }
        let text = run
            .get("sourceText")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if grouped.contains_key(&key) {
            let node = grouped.get_mut(&key).expect("existing text identity");
            node["text"] = Value::String(String::new());
            node["layout"]["lines"] = json!([]);
            continue;
        }
        let node = grouped.entry(key).or_insert_with(|| {
            json!({
                "nodeId": owner.get("nodeId").cloned().unwrap_or(Value::Null),
                "sourcePath": source_path,
                "sourceNodeId": node_id,
                "instancePath": instance_path,
                "controlId": owner.get("controlId").cloned().unwrap_or(Value::Null),
                "parentNodeId": owner.get("parentNodeId").cloned().unwrap_or(Value::Null),
                "parentSourcePath": owner.get("parentSourcePath").cloned().unwrap_or(Value::Null),
                "parentSourceNodeId": owner.get("parentSourceNodeId").cloned().unwrap_or(Value::Null),
                "parentInstancePath": owner.get("parentInstancePath").cloned().unwrap_or(Value::Null),
                "text": text,
                "layout": {
                    "font_size": run.get("fontSize").and_then(Value::as_f64)
                        .map(|value| json!(value / f64::from(capture.dpi)))
                        .unwrap_or_else(|| json!(0.0)),
                    "font_family": run.get("fontFamily").cloned().unwrap_or_else(|| json!("")),
                    "font_weight": run.get("fontWeight").cloned().unwrap_or_else(|| json!(0)),
                    "font_style": run.get("fontStyle").cloned().unwrap_or_else(|| json!("normal")),
                    "line_height": run.get("lineHeight").and_then(Value::as_f64)
                        .map(|value| json!(value / f64::from(capture.dpi)))
                        .unwrap_or_else(|| json!(0.0)),
                    "letter_spacing": 0.0,
                    "lines": lines,
                },
                "fonts": fonts,
            })
        });
        if run
            .get("errors")
            .and_then(Value::as_array)
            .is_some_and(|errors| !errors.is_empty())
        {
            node["text"] = Value::String(String::new());
            node["layout"]["lines"] = json!([]);
        }
    }
    grouped.into_values().collect()
}

fn run_fonts(run: &Value, faces: &BTreeMap<String, Value>) -> Vec<Value> {
    let mut counts = BTreeMap::<String, u64>::new();
    if let Some(glyphs) = run.get("glyphs").and_then(Value::as_array) {
        for glyph in glyphs {
            if let Some(face) = glyph.get("face").and_then(Value::as_str) {
                *counts.entry(face.to_owned()).or_default() += 1;
            }
        }
    }
    counts
        .into_iter()
        .filter_map(|(face_key, glyph_count)| {
            let face = faces.get(&face_key)?;
            Some(json!({
                "familyName": face.get("familyName")?,
                "postScriptName": face.get("postScriptName")?,
                "faceIndex": face.get("faceIndex")?,
                "glyphCount": glyph_count,
                "resourcePath": face.get("resourcePath")?,
                "sha256": face.get("sha256")?,
                "currentFileMatches": face.get("currentFileMatches")?,
            }))
        })
        .collect()
}

fn used_font_fingerprints(nodes: &[Value]) -> Vec<Value> {
    let mut pairs = BTreeMap::<String, String>::new();
    for node in nodes {
        if let Some(fonts) = node.get("fonts").and_then(Value::as_array) {
            for font in fonts {
                if let (Some(path), Some(sha)) = (
                    font.get("resourcePath").and_then(Value::as_str),
                    font.get("sha256").and_then(Value::as_str),
                ) {
                    pairs.insert(path.to_owned(), sha.to_owned());
                }
            }
        }
    }
    pairs
        .into_iter()
        .map(|(path, sha)| json!([path, sha]))
        .collect()
}

#[cfg(test)]
#[path = "tests/visual_evidence.rs"]
mod tests;

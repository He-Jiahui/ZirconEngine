use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, OnceLock};

use fontdb::{Database, Family, Query, Source, Stretch, Style, Weight};
use fontdue::{Font, FontSettings};
use zircon_runtime::ui::surface::measure_text_size;
use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextOverflow, UiTextRunPaintStyle, UiTextWrap,
};

use super::sync::lock_recovering_poison;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_text_preferences, HostTextPreferences,
};

mod metrics;

use self::metrics::{
    default_runtime_line_height, empty_runtime_text_width, measured_text_width,
    resolved_runtime_font_size, resolved_runtime_line_height, should_measure_runtime_text,
};

const FALLBACK_FONT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../zircon_runtime/assets/fonts/FiraMono-subset.ttf"
));
const UI_FONT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../zircon_runtime/assets/fonts/FiraSans-Regular.ttf"
));
const SYSTEM_UI_FALLBACK_FAMILIES: &[&str] =
    &["DengXian", "等线", "Microsoft YaHei UI", "Segoe UI"];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::ui::retained_host::host_contract) enum HostTextFontFace {
    Ui,
    UiStrong,
    Mono,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct HostTextFontRequest {
    pub face: HostTextFontFace,
    pub family: String,
    pub weight: u16,
}

pub(in crate::ui::retained_host::host_contract) fn font_face_for_paint_style(
    style: UiTextRunPaintStyle,
) -> HostTextFontFace {
    if style.code {
        HostTextFontFace::Mono
    } else if style.strong {
        HostTextFontFace::UiStrong
    } else {
        HostTextFontFace::Ui
    }
}

pub(in crate::ui::retained_host::host_contract) fn font_for_face(
    face: HostTextFontFace,
) -> Option<&'static Font> {
    host_font_for_face(face).font.as_ref()
}

pub(in crate::ui::retained_host::host_contract) fn font_bytes_for_face(
    face: HostTextFontFace,
) -> &'static [u8] {
    host_font_for_face(face).bytes.as_ref()
}

pub(in crate::ui::retained_host::host_contract) fn font_collection_index_for_face(
    face: HostTextFontFace,
) -> u32 {
    host_font_for_face(face).collection_index
}

pub(in crate::ui::retained_host::host_contract) fn font_cache_key_for_face(
    face: HostTextFontFace,
) -> u64 {
    host_font_for_face(face).cache_key
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) fn runtime_font_family_for_face(
    face: HostTextFontFace,
) -> &'static str {
    host_font_for_face(face).runtime_family.as_str()
}

pub(in crate::ui::retained_host::host_contract) fn font_request_for_face(
    face: HostTextFontFace,
) -> HostTextFontRequest {
    font_request_for_face_with_preferences(face, &current_host_text_preferences())
}

pub(in crate::ui::retained_host::host_contract) fn font_request_for_face_with_preferences(
    face: HostTextFontFace,
    preferences: &HostTextPreferences,
) -> HostTextFontRequest {
    match face {
        HostTextFontFace::Ui => HostTextFontRequest {
            face,
            family: preferences.ui_family.clone(),
            weight: preferences.ui_weight,
        },
        HostTextFontFace::UiStrong => HostTextFontRequest {
            face,
            family: preferences.ui_strong_family.clone(),
            weight: preferences.strong_weight,
        },
        HostTextFontFace::Mono => HostTextFontRequest {
            face,
            family: preferences.code_family.clone(),
            weight: preferences.code_weight,
        },
    }
}

pub(crate) fn measure_runtime_text_width(text: &str, font_size: f32) -> f32 {
    measure_runtime_text_width_with_style(text, font_size, UiTextRunPaintStyle::default())
}

pub(in crate::ui::retained_host::host_contract) fn measure_runtime_text_width_with_style(
    text: &str,
    font_size: f32,
    style: UiTextRunPaintStyle,
) -> f32 {
    if !should_measure_runtime_text(text, font_size) {
        return empty_runtime_text_width();
    }

    let font_face = font_face_for_paint_style(style);
    let style = runtime_text_style_for_face(
        font_face,
        font_size,
        default_runtime_line_height(font_size),
        UiTextWrap::None,
        UiTextOverflow::Clip,
    );
    measured_text_width(measure_text_size(text, &style).width)
}

pub(in crate::ui::retained_host::host_contract) fn runtime_text_style_for_face(
    face: HostTextFontFace,
    font_size: f32,
    line_height: f32,
    wrap: UiTextWrap,
    text_overflow: UiTextOverflow,
) -> UiResolvedStyle {
    let request = font_request_for_face(face);
    let runtime_family = cached_host_font(request.clone()).runtime_family.clone();
    let font_size = resolved_runtime_font_size(font_size);
    let line_height = resolved_runtime_line_height(font_size, line_height);
    UiResolvedStyle {
        font_family: Some(runtime_family),
        font_weight: request.weight,
        font_size,
        line_height,
        wrap,
        text_overflow,
        ..UiResolvedStyle::default()
    }
}

struct HostTextFont {
    font: Option<Font>,
    bytes: Box<[u8]>,
    runtime_family: String,
    collection_index: u32,
    cache_key: u64,
}

fn host_font_for_face(face: HostTextFontFace) -> &'static HostTextFont {
    let request = font_request_for_face(face);
    cached_host_font(request)
}

fn cached_host_font(request: HostTextFontRequest) -> &'static HostTextFont {
    static CACHE: OnceLock<Mutex<HashMap<HostTextFontRequest, &'static HostTextFont>>> =
        OnceLock::new();

    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(font) = lock_recovering_poison(cache).get(&request).copied() {
        return font;
    }

    let font = resolve_host_font(&request);
    let font = Box::leak(Box::new(font));
    lock_recovering_poison(cache).insert(request, font);
    font
}

fn resolve_host_font(request: &HostTextFontRequest) -> HostTextFont {
    load_system_font(request).unwrap_or_else(|| embedded_font_for_request(request))
}

fn load_system_font(request: &HostTextFontRequest) -> Option<HostTextFont> {
    let mut database = Database::new();
    database.load_system_fonts();
    let family = request.family.trim();
    let families = fontdb_families_for_request(family);
    let query = Query {
        families: families.as_slice(),
        weight: Weight(request.weight),
        stretch: Stretch::Normal,
        style: Style::Normal,
    };
    let face_id = database.query(&query)?;
    let face = database.face(face_id)?;
    let bytes = fontdb_source_bytes(&face.source)?;
    let collection_index = face.index;
    let runtime_family = face
        .families
        .first()
        .map(|family| family.0.clone())
        .filter(|family| !family.trim().is_empty())
        .unwrap_or_else(|| family.to_string());
    load_font_from_bytes(bytes, runtime_family, request, collection_index)
}

fn fontdb_families_for_request(family: &str) -> Vec<Family<'_>> {
    if is_system_ui_family(family) {
        return system_ui_fontdb_families();
    }

    match generic_font_family(family) {
        Some(family) => vec![family],
        None => vec![Family::Name(family)],
    }
}

fn is_system_ui_family(family: &str) -> bool {
    matches!(
        family.trim().to_ascii_lowercase().as_str(),
        "system-ui" | "ui-sans-serif"
    )
}

fn system_ui_fontdb_families() -> Vec<Family<'static>> {
    SYSTEM_UI_FALLBACK_FAMILIES
        .iter()
        .copied()
        .map(Family::Name)
        .chain(std::iter::once(Family::SansSerif))
        .collect()
}

fn generic_font_family(family: &str) -> Option<Family<'static>> {
    match family.trim().to_ascii_lowercase().as_str() {
        "sans-serif" => Some(Family::SansSerif),
        "monospace" | "ui-monospace" => Some(Family::Monospace),
        "serif" => Some(Family::Serif),
        "cursive" => Some(Family::Cursive),
        "fantasy" => Some(Family::Fantasy),
        _ => None,
    }
}

fn fontdb_source_bytes(source: &Source) -> Option<Vec<u8>> {
    match source {
        Source::File(path) | Source::SharedFile(path, _) => std::fs::read(path).ok(),
        Source::Binary(bytes) => Some(bytes.as_ref().as_ref().to_vec()),
    }
}

fn embedded_font_for_request(request: &HostTextFontRequest) -> HostTextFont {
    let bytes = match request.face {
        HostTextFontFace::Mono => FALLBACK_FONT_BYTES,
        HostTextFontFace::Ui | HostTextFontFace::UiStrong => UI_FONT_BYTES,
    };
    load_font_from_bytes(bytes.to_vec(), fallback_runtime_family(request), request, 0)
        .or_else(embedded_mono_font)
        .unwrap_or_else(|| unavailable_host_font(request))
}

fn embedded_mono_font() -> Option<HostTextFont> {
    let request = HostTextFontRequest {
        face: HostTextFontFace::Mono,
        family: EditorTypographyTokens::DEFAULT_CODE_FAMILY.to_string(),
        weight: 400,
    };
    load_font_from_bytes(
        FALLBACK_FONT_BYTES.to_vec(),
        request.family.clone(),
        &request,
        0,
    )
}

fn fallback_runtime_family(request: &HostTextFontRequest) -> String {
    let requested = request.family.trim();
    if !requested.is_empty() {
        return requested.to_string();
    }

    match request.face {
        HostTextFontFace::Mono => EditorTypographyTokens::DEFAULT_CODE_FAMILY.to_string(),
        HostTextFontFace::Ui | HostTextFontFace::UiStrong => {
            EditorTypographyTokens::DEFAULT_UI_FAMILY.to_string()
        }
    }
}

fn load_font_from_bytes(
    bytes: Vec<u8>,
    runtime_family: String,
    request: &HostTextFontRequest,
    collection_index: u32,
) -> Option<HostTextFont> {
    let font = Font::from_bytes(
        bytes.clone(),
        font_settings_for_collection_index(collection_index),
    )
    .ok()?;
    Some(HostTextFont {
        font: Some(font),
        cache_key: host_text_font_cache_key(
            request,
            runtime_family.as_str(),
            bytes.as_slice(),
            collection_index,
        ),
        bytes: bytes.into_boxed_slice(),
        runtime_family,
        collection_index,
    })
}

fn font_settings_for_collection_index(collection_index: u32) -> FontSettings {
    FontSettings {
        collection_index,
        ..FontSettings::default()
    }
}

fn unavailable_host_font(request: &HostTextFontRequest) -> HostTextFont {
    let runtime_family = fallback_runtime_family(request);
    HostTextFont {
        font: None,
        cache_key: host_text_font_cache_key(request, runtime_family.as_str(), &[], 0),
        bytes: Vec::<u8>::new().into_boxed_slice(),
        runtime_family,
        collection_index: 0,
    }
}

fn host_text_font_cache_key(
    request: &HostTextFontRequest,
    runtime_family: &str,
    bytes: &[u8],
    collection_index: u32,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    request.hash(&mut hasher);
    runtime_family.hash(&mut hasher);
    collection_index.hash(&mut hasher);
    bytes.len().hash(&mut hasher);
    bytes.first().copied().unwrap_or_default().hash(&mut hasher);
    bytes.last().copied().unwrap_or_default().hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests;

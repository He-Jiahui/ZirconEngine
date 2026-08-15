#[cfg(test)]
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock, Weak};

use fontdb::{Database, Family, Query, Source, Stretch, Style, Weight};
use fontdue::{Font, FontSettings};
use zircon_runtime::ui::surface::measure_text_size;
#[cfg(test)]
use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextOverflow, UiTextRunPaintStyle, UiTextWrap,
};

use super::sync::lock_recovering_poison;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_text_preferences, HostTextPreferences,
};

mod metrics;
mod runtime_artifact;

use self::metrics::{
    default_runtime_line_height, empty_runtime_text_width, measured_text_width,
    resolved_runtime_font_size, resolved_runtime_line_height, should_measure_runtime_text,
};
pub(in crate::ui::retained_host::host_contract) use runtime_artifact::host_runtime_artifact_font_snapshot;

const RUNTIME_FALLBACK_FONT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../zircon_runtime/assets/fonts/ZirconDefaultComposite-subset.ttc"
));
const RUNTIME_FALLBACK_FONT_FAMILY: &str = "Zircon Runtime Fallback Mono";
const HOST_FONT_SET_CACHE_CAPACITY: usize = 2;
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

pub(in crate::ui::retained_host::host_contract) fn runtime_font_family_for_face(
    face: HostTextFontFace,
) -> Arc<str> {
    host_font_snapshot_for_face(face).runtime_family()
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

pub(crate) fn runtime_text_metrics_generation() -> [u64; 3] {
    let fonts = current_host_font_set();
    [
        fonts.ui.cache_key,
        fonts.ui_strong.cache_key,
        fonts.mono.cache_key,
    ]
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
    let font = host_font_snapshot_for_face(face);
    let font_size = resolved_runtime_font_size(font_size);
    let line_height = resolved_runtime_line_height(font_size, line_height);
    UiResolvedStyle {
        font_family: Some(font.runtime_family().to_string()),
        font_weight: font.weight(),
        font_size,
        line_height,
        wrap,
        text_overflow,
        ..UiResolvedStyle::default()
    }
}

pub(in crate::ui::retained_host::host_contract) struct HostTextFontSnapshot {
    font: Arc<HostTextFont>,
}

impl HostTextFontSnapshot {
    pub(in crate::ui::retained_host::host_contract) fn font(&self) -> Option<&Font> {
        self.font.font.as_deref()
    }

    pub(in crate::ui::retained_host::host_contract) fn bytes(&self) -> &[u8] {
        self.font.bytes.as_ref()
    }

    pub(in crate::ui::retained_host::host_contract) fn collection_index(&self) -> u32 {
        self.font.collection_index
    }

    pub(in crate::ui::retained_host::host_contract) fn cache_key(&self) -> u64 {
        self.font.cache_key
    }

    pub(in crate::ui::retained_host::host_contract) fn runtime_family(&self) -> Arc<str> {
        Arc::clone(&self.font.runtime_family)
    }

    pub(in crate::ui::retained_host::host_contract) fn weight(&self) -> u16 {
        self.font.weight
    }
}

pub(in crate::ui::retained_host::host_contract) fn host_font_snapshot_for_face(
    face: HostTextFontFace,
) -> HostTextFontSnapshot {
    let fonts = current_host_font_set();
    HostTextFontSnapshot {
        font: Arc::clone(fonts.font(face)),
    }
}

#[derive(Clone)]
pub(in crate::ui::retained_host::host_contract) struct HostRuntimeTextFace {
    pub(in crate::ui::retained_host::host_contract) family: Arc<str>,
    pub(in crate::ui::retained_host::host_contract) weight: u16,
}

pub(in crate::ui::retained_host::host_contract) struct HostRuntimeTextFaces {
    ui: HostRuntimeTextFace,
    ui_strong: HostRuntimeTextFace,
    mono: HostRuntimeTextFace,
}

impl HostRuntimeTextFaces {
    pub(in crate::ui::retained_host::host_contract) fn face(
        &self,
        face: HostTextFontFace,
    ) -> &HostRuntimeTextFace {
        match face {
            HostTextFontFace::Ui => &self.ui,
            HostTextFontFace::UiStrong => &self.ui_strong,
            HostTextFontFace::Mono => &self.mono,
        }
    }
}

pub(in crate::ui::retained_host::host_contract) fn capture_runtime_text_faces(
) -> HostRuntimeTextFaces {
    #[cfg(test)]
    RUNTIME_TEXT_FACE_CAPTURE_COUNT.with(|count| count.set(count.get().saturating_add(1)));
    let fonts = current_host_font_set();
    HostRuntimeTextFaces {
        ui: fonts.ui.runtime_face(),
        ui_strong: fonts.ui_strong.runtime_face(),
        mono: fonts.mono.runtime_face(),
    }
}

#[cfg(test)]
thread_local! {
    static RUNTIME_TEXT_FACE_CAPTURE_COUNT: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) fn take_runtime_text_face_capture_count() -> usize {
    RUNTIME_TEXT_FACE_CAPTURE_COUNT.with(|count| count.replace(0))
}

struct HostTextFont {
    font: Option<Arc<Font>>,
    bytes: Arc<[u8]>,
    runtime_family: Arc<str>,
    weight: u16,
    collection_index: u32,
    cache_key: u64,
}

impl HostTextFont {
    fn runtime_face(&self) -> HostRuntimeTextFace {
        HostRuntimeTextFace {
            family: Arc::clone(&self.runtime_family),
            weight: self.weight,
        }
    }
}

struct HostTextFontSet {
    preferences: Arc<HostTextPreferences>,
    ui: Arc<HostTextFont>,
    ui_strong: Arc<HostTextFont>,
    mono: Arc<HostTextFont>,
}

impl HostTextFontSet {
    fn resolve(preferences: Arc<HostTextPreferences>) -> Self {
        let mut database = Database::new();
        database.load_system_fonts();
        Self {
            ui: Arc::new(resolve_host_font(
                &database,
                &font_request_for_face_with_preferences(HostTextFontFace::Ui, preferences.as_ref()),
            )),
            ui_strong: Arc::new(resolve_host_font(
                &database,
                &font_request_for_face_with_preferences(
                    HostTextFontFace::UiStrong,
                    preferences.as_ref(),
                ),
            )),
            mono: Arc::new(resolve_host_font(
                &database,
                &font_request_for_face_with_preferences(
                    HostTextFontFace::Mono,
                    preferences.as_ref(),
                ),
            )),
            preferences,
        }
    }

    fn font(&self, face: HostTextFontFace) -> &Arc<HostTextFont> {
        match face {
            HostTextFontFace::Ui => &self.ui,
            HostTextFontFace::UiStrong => &self.ui_strong,
            HostTextFontFace::Mono => &self.mono,
        }
    }
}

thread_local! {
    static ACTIVE_HOST_FONT_SET: RefCell<Weak<HostTextFontSet>> = const { RefCell::new(Weak::new()) };
}

fn current_host_font_set() -> Arc<HostTextFontSet> {
    let preferences = current_host_text_preferences();
    if let Some(fonts) = ACTIVE_HOST_FONT_SET.with(|active| {
        active.borrow().upgrade().filter(|fonts| {
            Arc::ptr_eq(&fonts.preferences, &preferences)
                || fonts.preferences.as_ref() == preferences.as_ref()
        })
    }) {
        return fonts;
    }

    let fonts = cached_host_font_set(preferences);
    ACTIVE_HOST_FONT_SET.with(|active| *active.borrow_mut() = Arc::downgrade(&fonts));
    fonts
}

fn cached_host_font_set(preferences: Arc<HostTextPreferences>) -> Arc<HostTextFontSet> {
    static CACHE: OnceLock<Mutex<VecDeque<Arc<HostTextFontSet>>>> = OnceLock::new();

    let cache = CACHE.get_or_init(|| Mutex::new(VecDeque::new()));
    {
        let cache = lock_recovering_poison(cache);
        if let Some(fonts) = cached_font_set(&cache, preferences.as_ref()) {
            return fonts;
        }
    }

    let fonts = Arc::new(HostTextFontSet::resolve(preferences));
    let mut cache = lock_recovering_poison(cache);
    if let Some(existing) = cached_font_set(&cache, fonts.preferences.as_ref()) {
        return existing;
    }
    cache.push_front(Arc::clone(&fonts));
    cache.truncate(HOST_FONT_SET_CACHE_CAPACITY);
    fonts
}

fn cached_font_set(
    cache: &VecDeque<Arc<HostTextFontSet>>,
    preferences: &HostTextPreferences,
) -> Option<Arc<HostTextFontSet>> {
    cache
        .iter()
        .find(|fonts| fonts.preferences.as_ref() == preferences)
        .map(Arc::clone)
}

fn resolve_host_font(database: &Database, request: &HostTextFontRequest) -> HostTextFont {
    load_system_font(database, request).unwrap_or_else(|| embedded_font_for_request(request))
}

fn load_system_font(database: &Database, request: &HostTextFontRequest) -> Option<HostTextFont> {
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
    load_font_from_bytes(Arc::from(bytes), runtime_family, request, collection_index)
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
    load_font_from_bytes(
        Arc::from(RUNTIME_FALLBACK_FONT_BYTES),
        RUNTIME_FALLBACK_FONT_FAMILY.to_string(),
        request,
        0,
    )
    .unwrap_or_else(|| unavailable_host_font(request))
}

fn load_font_from_bytes(
    bytes: Arc<[u8]>,
    runtime_family: String,
    request: &HostTextFontRequest,
    collection_index: u32,
) -> Option<HostTextFont> {
    let font = Font::from_bytes(
        Arc::clone(&bytes),
        font_settings_for_collection_index(collection_index),
    )
    .ok()?;
    Some(HostTextFont {
        font: Some(Arc::new(font)),
        cache_key: host_text_font_cache_key(
            request,
            runtime_family.as_str(),
            bytes.as_ref(),
            collection_index,
        ),
        bytes,
        runtime_family: Arc::from(runtime_family),
        weight: request.weight,
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
    let runtime_family = RUNTIME_FALLBACK_FONT_FAMILY;
    HostTextFont {
        font: None,
        cache_key: host_text_font_cache_key(request, runtime_family, &[], 0),
        bytes: Arc::from([]),
        runtime_family: Arc::from(runtime_family),
        weight: request.weight,
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
    bytes.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests;

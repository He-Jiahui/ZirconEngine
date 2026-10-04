use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use zircon_runtime_interface::ui::{
    layout::UiSize,
    window::{UiWindowMetrics, UiWindowPixelSize},
};

#[derive(Deserialize)]
pub(super) struct Catalog {
    pub entries: Vec<Entry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Entry {
    pub source_path: String,
    pub source_sha256: String,
    pub output_path: String,
    pub output_sha256: String,
    pub category: String,
    pub name: String,
    #[serde(default)]
    pub cases: Vec<Value>,
    #[serde(default)]
    pub dependency_fingerprints: Vec<Dependency>,
    #[serde(default)]
    pub dependency_sha256: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Dependency {
    pub source_path: String,
    pub sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Case {
    pub id: String,
    pub source_path: String,
    pub host: String,
    pub theme_source_path: Option<String>,
    pub review_host: Option<ReviewHost>,
    pub viewport: Viewport,
    pub dpi: f64,
    pub locale: String,
    pub state: String,
    pub data: Value,
}

#[derive(Deserialize)]
pub(super) struct ReviewHost {
    pub path: String,
    pub sha256: String,
}

pub(super) fn verify_review_host(
    root: &Path,
    entry: &Entry,
    case: &Case,
) -> Result<Option<PathBuf>, String> {
    let Some(host) = &case.review_host else {
        return Ok(None);
    };
    let file = if matches!(case.host.as_str(), "component" | "toolbar") {
        "component-host.zui"
    } else if case.host == "editor" {
        "editor-host.zui"
    } else {
        "theme-consumer.zui"
    };
    if host.path != format!("{}/{}/evidence/{file}", entry.category, entry.name) {
        return Err("review consumer host must belong to this catalog entry".into());
    }
    let path = source_path(root, &host.path)?;
    if file_hash(&path)? != host.sha256 {
        return Err("review consumer host SHA256 differs from catalog".into());
    }
    Ok(Some(path))
}

#[derive(Deserialize)]
pub(super) struct Viewport {
    pub width: u32,
    pub height: u32,
}

/// Product-facing Editor and plugin surfaces are rendered by the retained-host
/// capture binary.  Runtime WGPU evidence is reserved for WoC, runtime
/// fixtures, and runtime-owned components; keeping this split explicit avoids
/// recording a runtime screenshot as proof of an Editor host.
pub(super) fn requires_editor_renderer(entry: &Entry, case: &Case) -> bool {
    case.host != "fixture"
        && (entry.source_path.starts_with("zircon_editor/")
            || entry.source_path.starts_with("zircon_plugins/")
            || case.host == "plugin")
}

pub(super) fn window_metrics(case: &Case) -> Result<UiWindowMetrics, String> {
    if !case.dpi.is_finite() || case.dpi <= 0.0 {
        return Err("DPI must be positive and finite".into());
    }
    let physical_axis = |logical: u32| -> Result<u32, String> {
        let pixels = (logical as f64 * case.dpi).round();
        if logical == 0 || logical > 8192 || !(1.0..=8192.0).contains(&pixels) {
            return Err("logical and physical viewport dimensions must be in 1..=8192".into());
        }
        Ok(pixels as u32)
    };
    Ok(UiWindowMetrics::new(
        UiSize::new(case.viewport.width as f32, case.viewport.height as f32),
        UiWindowPixelSize::new(
            physical_axis(case.viewport.width)?,
            physical_axis(case.viewport.height)?,
        ),
        case.dpi,
    ))
}

pub(super) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn file_hash(path: &Path) -> Result<String, String> {
    use std::io::Read;
    let mut file =
        std::fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 65536];
    loop {
        let count = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

// Preserve the catalog's JSON numbers instead of round-tripping through f32 DTOs.
// JavaScript's lexical comparator orders keys by UTF-16 units.
pub(super) fn canonical_json(value: &Value) -> Result<String, String> {
    match value {
        Value::Object(map) => {
            let mut keys = map.keys().collect::<Vec<_>>();
            keys.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
            let mut fields = Vec::with_capacity(keys.len());
            for key in keys {
                fields.push(format!(
                    "{}:{}",
                    serde_json::to_string(key).map_err(|e| e.to_string())?,
                    canonical_json(&map[key])?
                ));
            }
            Ok(format!("{{{}}}", fields.join(",")))
        }
        Value::Array(items) => Ok(format!(
            "[{}]",
            items
                .iter()
                .map(canonical_json)
                .collect::<Result<Vec<_>, _>>()?
                .join(",")
        )),
        Value::Number(number) if number.is_f64() => {
            let number = number.as_f64().ok_or("invalid JSON number")?;
            if number.fract() == 0.0 && number.abs() <= 9_007_199_254_740_991.0 {
                Ok(if number == 0.0 {
                    "0".into()
                } else {
                    format!("{number:.0}")
                })
            } else {
                let text = number.to_string();
                if text.contains('e')
                    || (number != 0.0 && number.abs() < 0.000001)
                    || number.abs() >= 1e21
                {
                    return Err(
                        "exponential JSON numbers need the shared ECMAScript number serializer"
                            .into(),
                    );
                }
                Ok(text)
            }
        }
        _ => serde_json::to_string(value).map_err(|error| error.to_string()),
    }
}

pub(super) fn canonical_hash(value: &Value) -> Result<String, String> {
    Ok(sha256(canonical_json(value)?.as_bytes()))
}

pub(super) fn relative_path(value: &str) -> Result<&Path, String> {
    let path = Path::new(value);
    if value.is_empty()
        || value.contains('\\')
        || value.contains(':')
        || value
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(format!("expected contained relative path: {value}"));
    }
    Ok(path)
}

pub(super) fn source_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let candidate = root
        .join(relative_path(relative)?)
        .canonicalize()
        .map_err(|error| format!("{relative}: {error}"))?;
    if !candidate.starts_with(root.canonicalize().map_err(|e| e.to_string())?) {
        return Err(format!("path escapes repository: {relative}"));
    }
    Ok(candidate)
}

pub(super) fn case_directory(entry: &Entry, case: &Case) -> Result<PathBuf, String> {
    for segment in [&entry.category, &entry.name, &case.id] {
        if relative_path(segment)?.components().count() != 1 {
            return Err(format!(
                "output identity must be one path component: {segment}"
            ));
        }
    }
    Ok(Path::new(&entry.category)
        .join(&entry.name)
        .join("evidence"))
}

pub(super) fn verify_entry(
    root: &Path,
    catalog_root: &Path,
    entry: &Entry,
) -> Result<(String, String, Vec<PathBuf>), String> {
    let source_hash = file_hash(&source_path(root, &entry.source_path)?)?;
    if source_hash != entry.source_sha256 {
        return Err("source SHA256 differs from catalog; regenerate catalog".into());
    }
    if file_hash(&source_path(catalog_root, &entry.output_path)?)? != entry.output_sha256 {
        return Err("prepared output SHA256 differs from catalog".into());
    }
    let mut current = Vec::new();
    let mut paths = Vec::new();
    for dependency in &entry.dependency_fingerprints {
        let path = source_path(root, &dependency.source_path)?;
        let hash = file_hash(&path)?;
        if hash != dependency.sha256 {
            return Err(format!(
                "dependency SHA256 differs: {}",
                dependency.source_path
            ));
        }
        paths.push(path);
        current.push(Dependency {
            source_path: dependency.source_path.clone(),
            sha256: hash,
        });
    }
    current.sort_by(|a, b| {
        a.source_path
            .encode_utf16()
            .cmp(b.source_path.encode_utf16())
    });
    if current
        .windows(2)
        .any(|pair| pair[0].source_path == pair[1].source_path)
    {
        return Err("duplicate dependency fingerprint".into());
    }
    let dependency_hash =
        canonical_hash(&serde_json::to_value(current).map_err(|e| e.to_string())?)?;
    if dependency_hash != entry.dependency_sha256 {
        return Err("dependency set SHA256 differs from catalog".into());
    }
    Ok((source_hash, dependency_hash, paths))
}

pub(super) fn validate_case(case: &Case, entry: &Entry) -> Result<(), String> {
    if case.source_path != entry.source_path {
        return Err("case belongs to a different source".into());
    }
    window_metrics(case)?;
    if let Some(theme) = &case.theme_source_path {
        relative_path(theme)?;
        if !entry
            .dependency_fingerprints
            .iter()
            .any(|dependency| dependency.source_path == *theme)
        {
            return Err(format!(
                "review host theme is not a fingerprinted dependency: {theme}"
            ));
        }
        if case.review_host.is_some()
            && !matches!(case.host.as_str(), "component" | "toolbar")
            && theme != &case.source_path
        {
            return Err(
                "consumer host and external theme require an explicit precedence contract".into(),
            );
        }
    }
    if !super::state::supported(&case.state) {
        return Err(format!(
            "native state adapter unavailable for {}",
            case.state
        ));
    }
    if case.locale != "en-US"
        && !(case.locale == "zh-CN"
            && (case.data.get("componentInput").is_some()
                || case.data.get("collectionCase").is_some()
                || case.state == "long-zh"))
    {
        return Err(format!(
            "native locale adapter unavailable for {}",
            case.locale
        ));
    }
    super::data::validate(&case.data, &case.host)?;
    Ok(())
}

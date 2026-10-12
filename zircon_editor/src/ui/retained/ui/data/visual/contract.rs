use std::path::{Component, Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;
use zircon_runtime_interface::runtime_build_set::ZrRuntimeDigestV1;

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
    pub cases: Vec<Value>,
    pub dependency_sha256: String,
    pub dependency_fingerprints: Vec<Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReviewCase {
    pub id: String,
    pub source_path: String,
    pub host: String,
    pub viewport: Viewport,
    pub dpi: f64,
    pub locale: String,
    pub state: String,
    pub scroll_position: Option<ReviewScrollPosition>,
    pub data: Value,
    pub theme_source_path: Option<String>,
    pub review_host: Option<ReviewHost>,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(super) enum ReviewScrollPosition {
    Start,
    End,
}

#[derive(Deserialize)]
pub(super) struct ReviewHost {
    pub path: String,
    pub sha256: String,
}

#[derive(Clone, Copy, Deserialize)]
pub(super) struct Viewport {
    pub width: u32,
    pub height: u32,
}

impl ReviewCase {
    pub fn validate(&self, source: &str) -> Result<(), String> {
        if self.source_path != source
            || !matches!(
                self.host.as_str(),
                "editor" | "plugin" | "component" | "toolbar" | "theme"
            )
        {
            return Err("Editor case needs its exact source and a supported retained host".into());
        }
        self.physical_viewport()?;
        if matches!(
            (self.state.as_str(), self.scroll_position),
            ("scroll-before", Some(ReviewScrollPosition::End))
                | ("scroll-after", Some(ReviewScrollPosition::Start))
        ) {
            return Err("Editor case scrollPosition contradicts its state".into());
        }
        super::component_input::validate(&self.data)?;
        let has_component_input = self.data.get("componentInput").is_some();
        let has_workbench_state = self.data.get("workbenchState").is_some();
        let has_workbench_text_override = super::component_input::has_text_overrides(&self.data);
        let product_locale = self
            .data
            .get("workbenchPresentation")
            .map(|presentation| presentation.get("activeLocale").and_then(Value::as_str));
        let expected_product_locale = match self.locale.as_str() {
            "en-US" => Some("en"),
            "zh-CN" => Some("zh-CN"),
            _ => None,
        };
        if product_locale.is_some()
            && (self.host != "editor"
                || expected_product_locale.is_none()
                || product_locale.flatten() != expected_product_locale)
        {
            return Err("product workbench locale must match the Editor case locale".into());
        }
        if (self.locale != "en-US"
            && !(self.locale == "zh-CN"
                && (has_component_input
                    || has_workbench_text_override
                    || product_locale.is_some())))
            || !super::state::supported(&self.state)
            || (has_component_input && self.host != "component")
            || (has_workbench_state && self.host != "editor")
        {
            return Err(
                "Editor case needs an explicit locale, interaction or model-data adapter".into(),
            );
        }
        if !(1..=8192).contains(&self.viewport.width) || !(1..=8192).contains(&self.viewport.height)
        {
            return Err("Editor viewport must be within 1..=8192".into());
        }
        if self.id.is_empty()
            || !self
                .id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_'))
        {
            return Err("Editor case identifier contains path syntax".into());
        }
        Ok(())
    }

    pub(super) fn scroll_review_state(&self) -> &str {
        match self.scroll_position {
            Some(ReviewScrollPosition::Start) => "scroll-before",
            Some(ReviewScrollPosition::End) => "scroll-after",
            None => &self.state,
        }
    }

    pub fn physical_viewport(&self) -> Result<Viewport, String> {
        if !self.dpi.is_finite() || self.dpi <= 0.0 {
            return Err("Editor DPI must be positive and finite".into());
        }
        let physical_axis = |logical: u32| -> Result<u32, String> {
            let physical = (f64::from(logical) * self.dpi).round();
            if !(1.0..=8192.0).contains(&physical) {
                return Err("Editor physical viewport must be within 1..=8192".into());
            }
            Ok(physical as u32)
        };
        Ok(Viewport {
            width: physical_axis(self.viewport.width)?,
            height: physical_axis(self.viewport.height)?,
        })
    }
}

impl Entry {
    pub fn zui_dependencies(&self, repo: &Path) -> Result<Vec<PathBuf>, String> {
        let mut paths = Vec::new();
        for dependency in &self.dependency_fingerprints {
            let path = dependency["sourcePath"]
                .as_str()
                .ok_or("missing dependency path")?;
            if path.ends_with(".zui") {
                paths.push(contained(repo, path)?);
            }
        }
        Ok(paths)
    }

    pub fn verify(&self, repo: &Path, output: &Path) -> Result<(), String> {
        for (root, relative, expected) in [
            (repo, self.source_path.as_str(), self.source_sha256.as_str()),
            (
                output,
                self.output_path.as_str(),
                self.output_sha256.as_str(),
            ),
        ] {
            if hash_file(&contained(root, relative)?)? != expected {
                return Err(format!("stale catalog source: {relative}"));
            }
        }
        let mut previous = None;
        for dependency in &self.dependency_fingerprints {
            let path = dependency["sourcePath"]
                .as_str()
                .ok_or("missing dependency path")?;
            if previous
                .is_some_and(|prior: &str| prior.encode_utf16().cmp(path.encode_utf16()).is_ge())
            {
                return Err("dependency fingerprints must be unique and sorted".into());
            }
            previous = Some(path);
            if hash_file(&contained(repo, path)?)?
                != dependency["sha256"]
                    .as_str()
                    .ok_or("missing dependency hash")?
            {
                return Err(format!("stale catalog dependency: {path}"));
            }
        }
        if canonical_hash(&Value::Array(self.dependency_fingerprints.clone()))?
            != self.dependency_sha256
        {
            return Err("stale catalog dependency set".into());
        }
        Ok(())
    }
}

pub(super) fn verify_review_host(
    output: &Path,
    entry: &Entry,
    case: &ReviewCase,
) -> Result<Option<PathBuf>, String> {
    let Some(host) = &case.review_host else {
        return Ok(None);
    };
    let directory = format!("{}/{}/evidence/", entry.category, entry.name);
    let expected = match case.host.as_str() {
        "component" | "toolbar" => "component-host.zui",
        "theme" => "theme-consumer.zui",
        "editor" => "editor-host.zui",
        _ => return Err("Editor case cannot substitute an incompatible review host".into()),
    };
    if host.path != format!("{directory}{expected}") {
        return Err("Editor review host must belong to this catalog entry".into());
    }
    let path = contained(output, &host.path)?;
    if hash_file(&path)? != host.sha256 {
        return Err("Editor review host SHA256 differs from catalog".into());
    }
    Ok(Some(path))
}

pub(super) fn contained(root: &Path, relative: &str) -> Result<PathBuf, String> {
    if relative.is_empty()
        || relative.contains(['\\', ':'])
        || relative
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || Path::new(relative)
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("invalid relative evidence path: {relative}"));
    }
    let candidate = root.join(relative);
    let root = root.canonicalize().map_err(|error| error.to_string())?;
    let mut ancestor = candidate.as_path();
    while !ancestor.exists() {
        ancestor = ancestor.parent().ok_or("missing evidence parent")?;
    }
    if !ancestor
        .canonicalize()
        .map_err(|error| error.to_string())?
        .starts_with(root)
    {
        return Err(format!("evidence path escapes root: {relative}"));
    }
    Ok(candidate)
}

pub(super) fn hash_file(path: &Path) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    ZrRuntimeDigestV1::sha256_reader(file)
        .map(|digest| digest.as_str().into())
        .map_err(|error| error.to_string())
}

pub(super) fn canonical_hash(value: &Value) -> Result<String, String> {
    Ok(ZrRuntimeDigestV1::sha256(canonical(value)?).as_str().into())
}

fn canonical(value: &Value) -> Result<String, String> {
    match value {
        Value::Object(map) => {
            let mut keys = map.keys().collect::<Vec<_>>();
            keys.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
            Ok(format!(
                "{{{}}}",
                keys.into_iter()
                    .map(|key| Ok(format!(
                        "{}:{}",
                        serde_json::to_string(key).map_err(|error| error.to_string())?,
                        canonical(&map[key])?
                    )))
                    .collect::<Result<Vec<_>, String>>()?
                    .join(",")
            ))
        }
        Value::Array(items) => Ok(format!(
            "[{}]",
            items
                .iter()
                .map(canonical)
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
            } else if (number != 0.0 && number.abs() < 0.000001) || number.abs() >= 1e21 {
                Err("exponential JSON numbers require the shared ECMAScript serializer".into())
            } else {
                Ok(number.to_string())
            }
        }
        _ => serde_json::to_string(value).map_err(|error| error.to_string()),
    }
}

pub(super) fn read_json(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&std::fs::read(path).map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())
}

pub(super) fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    std::fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}

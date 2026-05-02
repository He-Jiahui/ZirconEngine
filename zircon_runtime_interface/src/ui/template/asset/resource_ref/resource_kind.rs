use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiResourceKind {
    Font,
    Image,
    Media,
    GenericAsset,
}

impl UiResourceKind {
    pub fn infer_from_path_and_uri(path: &str, uri: &str) -> Self {
        infer_from_path(path).unwrap_or_else(|| infer_from_uri_extension(uri))
    }
}

fn infer_from_path(path: &str) -> Option<UiResourceKind> {
    let normalized = path.to_ascii_lowercase();
    let segments: Vec<&str> = normalized
        .split(['.', '_', '-', '/', ':', '#', '[', ']'])
        .filter(|segment| !segment.is_empty())
        .collect();

    for index in (0..segments.len()).rev() {
        if index > 0 {
            if let Some(kind) =
                infer_from_path_name(&format!("{}_{}", segments[index - 1], segments[index]))
            {
                return Some(kind);
            }
        }
        if let Some(kind) = infer_from_path_name(segments[index]) {
            return Some(kind);
        }
    }

    None
}

fn infer_from_path_name(name: &str) -> Option<UiResourceKind> {
    match name {
        "font" | "font_asset" => Some(UiResourceKind::Font),
        "image" | "icon" | "background_image" => Some(UiResourceKind::Image),
        "media" | "video" | "audio" => Some(UiResourceKind::Media),
        "asset" | "resource" => Some(UiResourceKind::GenericAsset),
        _ => None,
    }
}

fn infer_from_uri_extension(uri: &str) -> UiResourceKind {
    let uri = uri.to_ascii_lowercase();
    let resource_path = uri
        .split(['#', '?'])
        .next()
        .unwrap_or(uri.as_str())
        .trim_end_matches('/');

    if resource_path.ends_with(".font.toml") {
        return UiResourceKind::Font;
    }

    match resource_path.rsplit_once('.') {
        Some((_, "ttf" | "otf" | "woff" | "woff2")) => UiResourceKind::Font,
        Some((_, "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tga" | "svg" | "ico")) => {
            UiResourceKind::Image
        }
        Some((_, "mp3" | "ogg" | "wav" | "flac" | "mp4" | "webm" | "mov")) => UiResourceKind::Media,
        _ => UiResourceKind::GenericAsset,
    }
}

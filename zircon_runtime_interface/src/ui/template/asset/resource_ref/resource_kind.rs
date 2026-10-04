use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 资源收集和回退验证使用的粗类型；GenericAsset 表示没有更具体的分类线索，不表示 URI 无效。
pub enum UiResourceKind {
    Font,
    Image,
    Media,
    GenericAsset,
}

impl UiResourceKind {
    /// 先按资源声明所在字段路径推断，找不到已知类别时才看 URI 后缀；这里只分类，不访问资源存储。
    pub fn infer_from_path_and_uri(path: &str, uri: &str) -> Self {
        infer_from_path(path).unwrap_or_else(|| infer_from_uri_extension(uri))
    }
}

// 字段路径从最靠近资源值的段向父级扫描；相邻组合先于单段匹配，保留 background-image 等语义。
fn infer_from_path(path: &str) -> Option<UiResourceKind> {
    let mut segments = path
        .rsplit(['.', '_', '-', '/', ':', '#', '[', ']'])
        .filter(|segment| !segment.is_empty());
    let mut current = segments.next()?;

    loop {
        let Some(previous) = segments.next() else {
            return infer_from_path_name(current);
        };
        if let Some(kind) = infer_from_path_pair(previous, current) {
            return Some(kind);
        }
        if let Some(kind) = infer_from_path_name(current) {
            return Some(kind);
        }
        current = previous;
    }
}

fn infer_from_path_name(name: &str) -> Option<UiResourceKind> {
    if name.eq_ignore_ascii_case("font") {
        Some(UiResourceKind::Font)
    } else if name.eq_ignore_ascii_case("image") || name.eq_ignore_ascii_case("icon") {
        Some(UiResourceKind::Image)
    } else if name.eq_ignore_ascii_case("media")
        || name.eq_ignore_ascii_case("video")
        || name.eq_ignore_ascii_case("audio")
    {
        Some(UiResourceKind::Media)
    } else if name.eq_ignore_ascii_case("asset") || name.eq_ignore_ascii_case("resource") {
        Some(UiResourceKind::GenericAsset)
    } else {
        None
    }
}

fn infer_from_path_pair(left: &str, right: &str) -> Option<UiResourceKind> {
    if left.eq_ignore_ascii_case("font") && right.eq_ignore_ascii_case("asset") {
        Some(UiResourceKind::Font)
    } else if left.eq_ignore_ascii_case("background") && right.eq_ignore_ascii_case("image") {
        Some(UiResourceKind::Image)
    } else {
        None
    }
}

// 路径未提供类别时用 URI 后缀兜底；先剥离 query/fragment 和尾斜线，避免 URL 元数据改变扩展名判断。
fn infer_from_uri_extension(uri: &str) -> UiResourceKind {
    let resource_path = uri
        .split(['#', '?'])
        .next()
        .unwrap_or(uri)
        .trim_end_matches('/');

    if ends_with_ascii_case(resource_path, ".font.toml") {
        return UiResourceKind::Font;
    }

    match resource_path.rsplit_once('.') {
        Some((_, extension))
            if extension.eq_ignore_ascii_case("ttf")
                || extension.eq_ignore_ascii_case("otf")
                || extension.eq_ignore_ascii_case("woff")
                || extension.eq_ignore_ascii_case("woff2") =>
        {
            UiResourceKind::Font
        }
        Some((_, extension))
            if extension.eq_ignore_ascii_case("png")
                || extension.eq_ignore_ascii_case("jpg")
                || extension.eq_ignore_ascii_case("jpeg")
                || extension.eq_ignore_ascii_case("webp")
                || extension.eq_ignore_ascii_case("bmp")
                || extension.eq_ignore_ascii_case("tga")
                || extension.eq_ignore_ascii_case("svg")
                || extension.eq_ignore_ascii_case("ico") =>
        {
            UiResourceKind::Image
        }
        Some((_, extension))
            if extension.eq_ignore_ascii_case("mp3")
                || extension.eq_ignore_ascii_case("ogg")
                || extension.eq_ignore_ascii_case("wav")
                || extension.eq_ignore_ascii_case("flac")
                || extension.eq_ignore_ascii_case("mp4")
                || extension.eq_ignore_ascii_case("webm")
                || extension.eq_ignore_ascii_case("mov") =>
        {
            UiResourceKind::Media
        }
        _ => UiResourceKind::GenericAsset,
    }
}

fn ends_with_ascii_case(value: &str, suffix: &str) -> bool {
    let value = value.as_bytes();
    value.len() >= suffix.len()
        && value[value.len() - suffix.len()..].eq_ignore_ascii_case(suffix.as_bytes())
}

#[cfg(test)]
#[path = "resource_kind/tests/performance_tests.rs"]
mod performance_tests;

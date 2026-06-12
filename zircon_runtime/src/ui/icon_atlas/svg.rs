use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
pub struct UiSvgIconDocument {
    pub viewport: UiSvgIconViewport,
    pub elements: Vec<UiSvgIconElement>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiSvgIconViewport {
    pub width: f32,
    pub height: f32,
    pub min_x: f32,
    pub min_y: f32,
    pub view_width: f32,
    pub view_height: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiSvgIconElement {
    Path {
        data: String,
        fill: Option<String>,
        stroke: Option<String>,
    },
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiSvgIconParseError {
    #[error("svg icon document is missing an <svg> root")]
    MissingSvgRoot,
    #[error("svg icon document has no supported shape elements")]
    MissingSupportedElements,
    #[error("svg icon {attribute} attribute is invalid: {value}")]
    InvalidNumber {
        attribute: &'static str,
        value: String,
    },
}

pub fn parse_ui_svg_icon(source: &str) -> Result<UiSvgIconDocument, UiSvgIconParseError> {
    let svg = find_svg_tag(source).ok_or(UiSvgIconParseError::MissingSvgRoot)?;
    let viewport = parse_viewport(svg)?;
    let elements = parse_path_elements(source);
    if elements.is_empty() {
        return Err(UiSvgIconParseError::MissingSupportedElements);
    }
    Ok(UiSvgIconDocument { viewport, elements })
}

fn find_svg_tag(source: &str) -> Option<&str> {
    let start = source.find("<svg")?;
    let rest = &source[start..];
    let end = rest.find('>')?;
    Some(&rest[..=end])
}

fn parse_viewport(svg_tag: &str) -> Result<UiSvgIconViewport, UiSvgIconParseError> {
    let view_box = attribute(svg_tag, "viewBox").and_then(parse_view_box);
    let width = match attribute(svg_tag, "width") {
        Some(width) => parse_svg_number("width", width)?,
        None => view_box.map(|(_, _, width, _)| width).unwrap_or(24.0),
    };
    let height = match attribute(svg_tag, "height") {
        Some(height) => parse_svg_number("height", height)?,
        None => view_box.map(|(_, _, _, height)| height).unwrap_or(24.0),
    };
    let (min_x, min_y, view_width, view_height) = view_box.unwrap_or((0.0, 0.0, width, height));
    Ok(UiSvgIconViewport {
        width,
        height,
        min_x,
        min_y,
        view_width,
        view_height,
    })
}

fn parse_path_elements(source: &str) -> Vec<UiSvgIconElement> {
    let mut elements = Vec::new();
    let mut remaining = source;
    while let Some(start) = remaining.find("<path") {
        remaining = &remaining[start..];
        let Some(end) = remaining.find('>') else {
            break;
        };
        let tag = &remaining[..=end];
        if let Some(data) = attribute(tag, "d") {
            if !data.trim().is_empty() {
                elements.push(UiSvgIconElement::Path {
                    data: data.to_string(),
                    fill: attribute(tag, "fill").map(str::to_string),
                    stroke: attribute(tag, "stroke").map(str::to_string),
                });
            }
        }
        remaining = &remaining[end + 1..];
    }
    elements
}

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let start = tag.find(name)?;
    let rest = tag[start + name.len()..].trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = &rest[quote.len_utf8()..];
    let end = rest.find(quote)?;
    Some(&rest[..end])
}

fn parse_view_box(raw: &str) -> Option<(f32, f32, f32, f32)> {
    let values = raw
        .split(|character: char| character.is_ascii_whitespace() || character == ',')
        .filter(|value| !value.is_empty())
        .filter_map(|value| value.parse::<f32>().ok())
        .collect::<Vec<_>>();
    (values.len() == 4).then(|| (values[0], values[1], values[2], values[3]))
}

fn parse_svg_number(attribute: &'static str, raw: &str) -> Result<f32, UiSvgIconParseError> {
    let value = raw
        .trim()
        .trim_end_matches("px")
        .parse::<f32>()
        .map_err(|_| UiSvgIconParseError::InvalidNumber {
            attribute,
            value: raw.to_string(),
        })?;
    if !value.is_finite() || value <= 0.0 {
        return Err(UiSvgIconParseError::InvalidNumber {
            attribute,
            value: raw.to_string(),
        });
    }
    Ok(value)
}

use glyphon::{cosmic_text::Align, Attrs, Buffer, Family, Metrics, Shaping, Style, Weight, Wrap};

use crate::core::framework::text::{
    TextDirection, TextFontRequest, TextRenderMode, TextShapeRequest,
};

use super::{
    fallback_spans_for_request, FontFaceId, FontFamilyName, FontQuery, FontStretch, FontStyle,
    FontWeight, TextRenderState,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum NativeTextWrap {
    None,
    #[default]
    Word,
    Glyph,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum NativeTextAlign {
    #[default]
    Left,
    Center,
    Right,
    Justified,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct NativeTextBufferRequest<'a> {
    pub(crate) text: &'a str,
    pub(crate) font_asset: Option<&'a str>,
    pub(crate) family: Option<&'a str>,
    pub(crate) language: Option<&'a str>,
    pub(crate) font_weight: u16,
    pub(crate) font_size: f32,
    pub(crate) line_height: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) direction: TextDirection,
    pub(crate) wrap: NativeTextWrap,
    pub(crate) align: NativeTextAlign,
    pub(crate) strong: bool,
    pub(crate) emphasis: bool,
    pub(crate) code: bool,
}

pub(crate) struct NativeTextBuffer {
    pub(crate) buffer: Buffer,
    pub(crate) primary_face: Option<FontFaceId>,
}

impl TextRenderState {
    pub(crate) fn shape_native_buffer(
        &mut self,
        request: NativeTextBufferRequest<'_>,
    ) -> NativeTextBuffer {
        self.with_native_text_backend(|font_system, font_database| {
            let font_query = native_font_query(request);
            let primary_face = font_database
                .match_face(&font_query)
                .map(|font_match| font_match.face);
            let attrs = native_attrs(request);
            let family_storage = request.family.map(|family| [family]);
            let families = family_storage
                .as_ref()
                .map_or(&[][..], |families| &families[..]);
            let font = TextFontRequest {
                families,
                asset: request.font_asset,
                size: request.font_size,
                weight: font_query.weight.0,
                stretch: 100,
                italic: matches!(font_query.style, FontStyle::Italic),
                render_mode: TextRenderMode::Auto,
            };
            let mut fallback_request = TextShapeRequest::new(request.text, font);
            fallback_request.language = request.language;
            fallback_request.direction = request.direction;
            fallback_request.line_height = request.line_height;
            let fallback_spans = fallback_spans_for_request(fallback_request, font_database);
            let mut buffer = Buffer::new(
                font_system,
                Metrics::new(request.font_size, request.line_height),
            );
            buffer.set_size(
                font_system,
                Some(request.width.max(1.0)),
                Some(request.height.max(1.0)),
            );
            buffer.set_wrap(
                font_system,
                match request.wrap {
                    NativeTextWrap::None => Wrap::None,
                    NativeTextWrap::Word => Wrap::Word,
                    NativeTextWrap::Glyph => Wrap::Glyph,
                },
            );
            let alignment = Some(match request.align {
                NativeTextAlign::Left => Align::Left,
                NativeTextAlign::Center => Align::Center,
                NativeTextAlign::Right => Align::Right,
                NativeTextAlign::Justified => Align::Justified,
            });
            if fallback_spans.is_empty() {
                buffer.set_text(
                    font_system,
                    request.text,
                    &attrs,
                    Shaping::Advanced,
                    alignment,
                );
            } else {
                buffer.set_rich_text(
                    font_system,
                    fallback_spans.iter().map(|span| {
                        let span_attrs = span
                            .family
                            .as_deref()
                            .map(|family| attrs.clone().family(Family::Name(family)))
                            .unwrap_or_else(|| attrs.clone());
                        (&request.text[span.range.clone()], span_attrs)
                    }),
                    &attrs,
                    Shaping::Advanced,
                    alignment,
                );
            }
            buffer.shape_until_scroll(font_system, false);
            NativeTextBuffer {
                buffer,
                primary_face,
            }
        })
    }
}

fn native_attrs(request: NativeTextBufferRequest<'_>) -> Attrs<'_> {
    let mut attrs = request
        .family
        .map(|family| Attrs::new().family(Family::Name(family)))
        .unwrap_or_else(|| {
            if request.code {
                Attrs::new().family(Family::Monospace)
            } else {
                Attrs::new()
            }
        });
    let query = native_font_query(request);
    attrs = attrs.weight(Weight(query.weight.0));
    if matches!(query.style, FontStyle::Italic) {
        attrs = attrs.style(Style::Italic);
    }
    attrs
}

fn native_font_query(request: NativeTextBufferRequest<'_>) -> FontQuery {
    let family = request.family.or(request.font_asset).unwrap_or_default();
    let requested_weight = FontWeight::clamped(request.font_weight);
    FontQuery {
        families: vec![FontFamilyName::from(family)],
        weight: if request.strong {
            requested_weight.max(FontWeight::BOLD)
        } else {
            requested_weight
        },
        style: if request.emphasis {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        },
        stretch: FontStretch::NORMAL,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request<'a>(family: Option<&'a str>) -> NativeTextBufferRequest<'a> {
        NativeTextBufferRequest {
            text: "Style",
            font_asset: None,
            family,
            language: None,
            font_weight: 500,
            font_size: 16.0,
            line_height: 20.0,
            width: 100.0,
            height: 20.0,
            direction: TextDirection::LeftToRight,
            wrap: NativeTextWrap::None,
            align: NativeTextAlign::Left,
            strong: false,
            emphasis: false,
            code: false,
        }
    }

    #[test]
    fn native_attrs_are_owned_by_text_cpu_preparation() {
        let mut rich = request(Some("Zircon Sans"));
        rich.font_weight = 650;
        rich.strong = true;
        rich.emphasis = true;
        let attrs = native_attrs(rich);
        let query = native_font_query(rich);
        assert_eq!(attrs.family, Family::Name("Zircon Sans"));
        assert_eq!(attrs.weight, Weight::BOLD);
        assert_eq!(attrs.style, Style::Italic);
        assert_eq!(query.families, vec![FontFamilyName::from("Zircon Sans")]);
        assert_eq!(query.weight, FontWeight::BOLD);
        assert_eq!(query.style, FontStyle::Italic);

        let medium = native_attrs(request(Some("Zircon Sans")));
        assert_eq!(medium.weight, Weight(500));

        let mut code = request(Some("Zircon Mono"));
        code.font_weight = 450;
        code.code = true;
        let attrs = native_attrs(code);
        let query = native_font_query(code);
        assert_eq!(attrs.family, Family::Name("Zircon Mono"));
        assert_eq!(attrs.weight, Weight(450));
        assert_eq!(query.families, vec![FontFamilyName::from("Zircon Mono")]);
        assert_eq!(query.weight, FontWeight(450));
        assert_eq!(query.style, FontStyle::Normal);
    }
}

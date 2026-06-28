use crate::graphics::text::layout::measure_line_width;
use zircon_runtime_interface::ui::surface::UiTextWrap;

use super::resolved_layout::{
    resolve_text_layout, UiTextLayoutRequest, UiTextLayoutResolution, UiTextStyleKey,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiWidthBucket(u32);

impl UiWidthBucket {
    pub(crate) fn from_request(request: &UiTextLayoutRequest<'_>) -> Self {
        if request.style.wrap == UiTextWrap::None {
            return Self(0);
        }

        let advance = measure_line_width("n", request.style)
            .max(request.style.font_size.max(1.0) * 0.25)
            .max(1.0);
        Self(
            (request.frame.width.max(advance) / advance)
                .floor()
                .max(1.0) as u32,
        )
    }

    pub(crate) const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiTextMeasureKey {
    pub content_hash: u64,
    pub width_bucket: UiWidthBucket,
    pub style: UiTextStyleKey,
}

impl UiTextMeasureKey {
    pub(crate) fn from_request(request: &UiTextLayoutRequest<'_>) -> Self {
        Self {
            content_hash: request.source_hash(),
            width_bucket: UiWidthBucket::from_request(request),
            style: request.style_key(),
        }
    }
}

#[derive(Clone, Debug)]
struct UiTextMeasureEntry {
    key: UiTextMeasureKey,
    resolution: UiTextLayoutResolution,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct UiTextMeasureCache {
    entries: Vec<UiTextMeasureEntry>,
    frame_shape_count: u64,
}

impl UiTextMeasureCache {
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.frame_shape_count = 0;
    }

    pub(crate) fn begin_frame(&mut self) {
        self.frame_shape_count = 0;
    }

    pub(crate) fn frame_shape_count(&self) -> u64 {
        self.frame_shape_count
    }

    pub(crate) fn resolve_or_shape(
        &mut self,
        request: &UiTextLayoutRequest<'_>,
    ) -> &UiTextLayoutResolution {
        let key = UiTextMeasureKey::from_request(request);
        if let Some(index) = self.entries.iter().position(|entry| entry.key == key) {
            return &self.entries[index].resolution;
        }

        let resolution = resolve_text_layout(request);
        self.entries.push(UiTextMeasureEntry { key, resolution });
        self.frame_shape_count += 1;
        &self
            .entries
            .last()
            .expect("entry was just pushed")
            .resolution
    }
}

use crate::core::framework::text::TextDirection;
use crate::text::shaping::{TextShapeRunProvider, TextShapingOutcome};
use crate::text::{TextRange, TextStyle, VerticalMode};

#[derive(Clone, Copy)]
pub(super) enum RichShapeProfilePhase {
    RangeIndex,
    Layout,
    UiItemProjection,
}

pub(super) struct RichShapeProfileProvider<'provider, P>
where
    P: TextShapeRunProvider + ?Sized,
{
    provider: &'provider mut P,
    phase: RichShapeProfilePhase,
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    shape_request_count: usize,
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    shape_input_byte_count: usize,
}

impl<'provider, P> RichShapeProfileProvider<'provider, P>
where
    P: TextShapeRunProvider + ?Sized,
{
    pub(super) fn new(provider: &'provider mut P, phase: RichShapeProfilePhase) -> Self {
        Self {
            provider,
            phase,
            #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
            shape_request_count: 0,
            #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
            shape_input_byte_count: 0,
        }
    }

    fn record_shape_request(&mut self, text: &str) {
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        {
            self.shape_request_count = self.shape_request_count.saturating_add(1);
            self.shape_input_byte_count = self.shape_input_byte_count.saturating_add(text.len());
        }
        #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
        let _ = text;
    }
}

impl<P> TextShapeRunProvider for RichShapeProfileProvider<'_, P>
where
    P: TextShapeRunProvider + ?Sized,
{
    fn shape_horizontal_range_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        include_kerning: bool,
    ) -> TextShapingOutcome {
        self.record_shape_request(text);
        self.provider.shape_horizontal_range_with_kerning(
            text,
            style,
            direction,
            source_range,
            include_kerning,
        )
    }

    fn shape_vertical_range_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        vertical_mode: VerticalMode,
        include_kerning: bool,
    ) -> TextShapingOutcome {
        self.record_shape_request(text);
        self.provider.shape_vertical_range_with_kerning(
            text,
            style,
            direction,
            source_range,
            vertical_mode,
            include_kerning,
        )
    }
}

impl<P> Drop for RichShapeProfileProvider<'_, P>
where
    P: TextShapeRunProvider + ?Sized,
{
    fn drop(&mut self) {
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        match self.phase {
            RichShapeProfilePhase::RangeIndex => {
                crate::profile_counter!(
                    "runtime",
                    "rich_range_index_shape_request_count",
                    self.shape_request_count
                );
                crate::profile_counter!(
                    "runtime",
                    "rich_range_index_shape_input_byte_count",
                    self.shape_input_byte_count
                );
            }
            RichShapeProfilePhase::Layout => {
                crate::profile_counter!(
                    "runtime",
                    "rich_layout_shape_request_count",
                    self.shape_request_count
                );
                crate::profile_counter!(
                    "runtime",
                    "rich_layout_shape_input_byte_count",
                    self.shape_input_byte_count
                );
            }
            RichShapeProfilePhase::UiItemProjection => {
                crate::profile_counter!(
                    "runtime",
                    "ui_rich_item_projection_shape_request_count",
                    self.shape_request_count
                );
                crate::profile_counter!(
                    "runtime",
                    "ui_rich_item_projection_shape_input_byte_count",
                    self.shape_input_byte_count
                );
            }
        }
        #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
        let _ = self.phase;
    }
}

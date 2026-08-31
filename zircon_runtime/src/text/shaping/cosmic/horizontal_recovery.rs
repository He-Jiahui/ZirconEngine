use crate::core::framework::text::TextLayoutError;
use crate::text::font::FontDatabase;
use crate::text::{ShapedGlyphRun, TextHorizontalCompositionReceipt, TextOrientation};

use super::super::failure_receipt::classify_direct_shape_failure;
use super::super::horizontal::{HorizontalPartialShape, compose_horizontal_partial};
use super::super::{TextShapingFailure, TextShapingFailureCode, TextShapingFailureReceipt};

pub(super) struct PendingHorizontalComposition {
    partial: HorizontalPartialShape,
    first_failure: TextShapingFailureReceipt,
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    profile: Option<super::direct_profile::DirectShapeProfileMetrics>,
}

impl PendingHorizontalComposition {
    pub(super) fn classify(
        partial: HorizontalPartialShape,
        orientation: TextOrientation,
    ) -> Result<Self, TextShapingFailure> {
        let mut first_alternate = None;
        for error in partial.errors() {
            let receipt = classify_direct_shape_failure(error, orientation);
            if !receipt.allows_alternate_backend() {
                #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
                super::direct_profile::discard();
                return Err(TextShapingFailure::with_receipt(
                    direct_failure_layout_error(receipt),
                    receipt,
                ));
            }
            first_alternate.get_or_insert(receipt);
        }
        let first_failure =
            first_alternate.expect("partial direct shape must retain at least one hole");
        Ok(Self {
            partial,
            first_failure,
            #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
            profile: super::direct_profile::detach(),
        })
    }

    pub(super) const fn first_failure(&self) -> TextShapingFailureReceipt {
        self.first_failure
    }

    pub(super) fn compose_or_retain_alternate(
        self,
        alternate: ShapedGlyphRun,
        database: &FontDatabase,
        font_size: f32,
        line_height: f32,
        _input_bytes: usize,
    ) -> ShapedGlyphRun {
        let _hole_count = self.partial.hole_count();
        let _direct_glyph_count = self.partial.direct_glyph_count();
        match compose_horizontal_partial(
            self.partial,
            alternate,
            database,
            font_size,
            line_height,
            self.first_failure,
        ) {
            Ok(composed) => {
                #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
                super::direct_profile::record_horizontal_composition(
                    self.profile,
                    _input_bytes,
                    _hole_count,
                    _direct_glyph_count,
                    composed.alternate_glyph_count,
                    false,
                );
                composed.shaped
            }
            Err((_, mut alternate)) => {
                #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
                super::direct_profile::record_horizontal_composition(
                    self.profile,
                    _input_bytes,
                    _hole_count,
                    _direct_glyph_count,
                    0,
                    true,
                );
                alternate.horizontal_composition_receipt =
                    Some(Box::new(TextHorizontalCompositionReceipt {
                        alternate_ranges: Vec::new(),
                        first_failure: self.first_failure,
                    }));
                alternate
            }
        }
    }
}

pub(super) fn direct_failure_layout_error(receipt: TextShapingFailureReceipt) -> TextLayoutError {
    if receipt.code == TextShapingFailureCode::BidiInvariant {
        TextLayoutError::BidiInvariant
    } else {
        TextLayoutError::ShapingFailed
    }
}

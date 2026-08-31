use super::{WindowLogicalExtent, WindowStateField, WindowStateValidationError};

/// Logical resize bounds accepted by a window-state contract. `None` means
/// intentionally unbounded; invalid or inverted bounds are rejected rather
/// than repaired by the host.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowStateResizeConstraints {
    minimum: WindowLogicalExtent,
    maximum: Option<WindowLogicalExtent>,
}

impl WindowStateResizeConstraints {
    pub fn new(
        minimum: WindowLogicalExtent,
        maximum: Option<WindowLogicalExtent>,
    ) -> Result<Self, WindowStateValidationError> {
        if let Some(maximum) = maximum {
            if maximum.width() < minimum.width() {
                return Err(WindowStateValidationError::MaximumBelowMinimum {
                    axis: WindowStateField::MaximumLogicalWidth,
                    minimum: minimum.width(),
                    maximum: maximum.width(),
                });
            }
            if maximum.height() < minimum.height() {
                return Err(WindowStateValidationError::MaximumBelowMinimum {
                    axis: WindowStateField::MaximumLogicalHeight,
                    minimum: minimum.height(),
                    maximum: maximum.height(),
                });
            }
        }
        Ok(Self { minimum, maximum })
    }

    pub const fn minimum(self) -> WindowLogicalExtent {
        self.minimum
    }

    pub const fn maximum(self) -> Option<WindowLogicalExtent> {
        self.maximum
    }
}

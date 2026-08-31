use super::{WindowStateField, WindowStateValidationError};

/// Strict physical extent for requested or observed window geometry. Zero is
/// rejected instead of being silently promoted to one pixel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowPhysicalExtent {
    width: u32,
    height: u32,
}

impl WindowPhysicalExtent {
    pub fn new(width: u32, height: u32) -> Result<Self, WindowStateValidationError> {
        if width == 0 {
            return Err(WindowStateValidationError::NonPositive {
                field: WindowStateField::PhysicalWidth,
                value: 0.0,
            });
        }
        if height == 0 {
            return Err(WindowStateValidationError::NonPositive {
                field: WindowStateField::PhysicalHeight,
                value: 0.0,
            });
        }
        Ok(Self { width, height })
    }

    pub const fn width(self) -> u32 {
        self.width
    }

    pub const fn height(self) -> u32 {
        self.height
    }
}

/// Strict logical extent for DPI-aware requested geometry and resize limits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowLogicalExtent {
    width: f64,
    height: f64,
}

impl WindowLogicalExtent {
    pub fn new(width: f64, height: f64) -> Result<Self, WindowStateValidationError> {
        validate_positive_finite(width, WindowStateField::LogicalWidth)?;
        validate_positive_finite(height, WindowStateField::LogicalHeight)?;
        Ok(Self { width, height })
    }

    pub const fn width(self) -> f64 {
        self.width
    }

    pub const fn height(self) -> f64 {
        self.height
    }
}

/// A finite logical desktop-space point. Placement does not coerce an invalid
/// value to automatic routing because doing so hides an invalid caller intent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowLogicalPosition {
    x: f64,
    y: f64,
}

impl WindowLogicalPosition {
    pub fn new(x: f64, y: f64) -> Result<Self, WindowStateValidationError> {
        validate_finite(x, WindowStateField::PositionX)?;
        validate_finite(y, WindowStateField::PositionY)?;
        Ok(Self { x, y })
    }

    pub const fn x(self) -> f64 {
        self.x
    }

    pub const fn y(self) -> f64 {
        self.y
    }
}

fn validate_positive_finite(
    value: f64,
    field: WindowStateField,
) -> Result<(), WindowStateValidationError> {
    validate_finite(value, field)?;
    if value <= 0.0 {
        return Err(WindowStateValidationError::NonPositive { field, value });
    }
    Ok(())
}

fn validate_finite(value: f64, field: WindowStateField) -> Result<(), WindowStateValidationError> {
    if !value.is_finite() {
        return Err(WindowStateValidationError::NonFinite { field, value });
    }
    Ok(())
}

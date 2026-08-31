use std::error::Error;
use std::fmt;

/// The exact requested-state field that failed strict validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowStateField {
    PhysicalWidth,
    PhysicalHeight,
    LogicalWidth,
    LogicalHeight,
    PositionX,
    PositionY,
    ScaleFactor,
    MaximumLogicalWidth,
    MaximumLogicalHeight,
}

impl fmt::Display for WindowStateField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let field = match self {
            Self::PhysicalWidth => "physical_width",
            Self::PhysicalHeight => "physical_height",
            Self::LogicalWidth => "logical_width",
            Self::LogicalHeight => "logical_height",
            Self::PositionX => "position_x",
            Self::PositionY => "position_y",
            Self::ScaleFactor => "scale_factor",
            Self::MaximumLogicalWidth => "maximum_logical_width",
            Self::MaximumLogicalHeight => "maximum_logical_height",
        };
        formatter.write_str(field)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WindowStateValidationError {
    NonFinite {
        field: WindowStateField,
        value: f64,
    },
    NonPositive {
        field: WindowStateField,
        value: f64,
    },
    MaximumBelowMinimum {
        axis: WindowStateField,
        minimum: f64,
        maximum: f64,
    },
}

impl fmt::Display for WindowStateValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite { field, value } => {
                write!(
                    formatter,
                    "window state field {field} is non-finite: {value}"
                )
            }
            Self::NonPositive { field, value } => {
                write!(
                    formatter,
                    "window state field {field} must be positive: {value}"
                )
            }
            Self::MaximumBelowMinimum {
                axis,
                minimum,
                maximum,
            } => write!(
                formatter,
                "window state field {axis} is below its minimum: {maximum} < {minimum}"
            ),
        }
    }
}

impl Error for WindowStateValidationError {}

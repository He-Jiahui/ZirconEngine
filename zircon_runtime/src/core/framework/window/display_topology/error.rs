use std::error::Error;
use std::fmt;

use super::DisplayId;

#[derive(Clone, Debug, PartialEq)]
pub enum DisplayTopologyError {
    NonFiniteLogicalGeometry,
    NonPositiveLogicalExtent {
        width: f64,
        height: f64,
    },
    NonFiniteSafeAreaInsets,
    NegativeSafeAreaInsets,
    SafeAreaExceedsUsableBounds {
        display: DisplayId,
    },
    NonFiniteScaleFactor {
        display: DisplayId,
        scale_factor: f64,
    },
    NonPositiveScaleFactor {
        display: DisplayId,
        scale_factor: f64,
    },
    DuplicateDisplay {
        display: DisplayId,
    },
    UnknownPrimaryDisplay {
        display: DisplayId,
    },
    CapacityExhausted,
}

impl fmt::Display for DisplayTopologyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteLogicalGeometry => {
                formatter.write_str("display logical geometry must be finite")
            }
            Self::NonPositiveLogicalExtent { width, height } => write!(
                formatter,
                "display logical extent must be positive, got {width}x{height}"
            ),
            Self::NonFiniteSafeAreaInsets => {
                formatter.write_str("display safe-area insets must be finite")
            }
            Self::NegativeSafeAreaInsets => {
                formatter.write_str("display safe-area insets must not be negative")
            }
            Self::SafeAreaExceedsUsableBounds { display } => write!(
                formatter,
                "display {display} reported safe-area insets larger than its usable logical bounds"
            ),
            Self::NonFiniteScaleFactor {
                display,
                scale_factor,
            } => write!(
                formatter,
                "display {display} reported non-finite scale factor {scale_factor}"
            ),
            Self::NonPositiveScaleFactor {
                display,
                scale_factor,
            } => write!(
                formatter,
                "display {display} reported non-positive scale factor {scale_factor}"
            ),
            Self::DuplicateDisplay { display } => {
                write!(
                    formatter,
                    "display topology contains duplicate display {display}"
                )
            }
            Self::UnknownPrimaryDisplay { display } => write!(
                formatter,
                "display topology primary display {display} is absent from the snapshot"
            ),
            Self::CapacityExhausted => {
                formatter.write_str("display topology index allocation exhausted capacity")
            }
        }
    }
}

impl Error for DisplayTopologyError {}

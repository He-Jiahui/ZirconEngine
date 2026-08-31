use zircon_runtime_interface::ZrByteSliceError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::dynamic_api) enum BoundedJsonError {
    Slice(ZrByteSliceError),
    Empty,
    EncodedBytes { observed: usize, limit: usize },
    Items { observed: usize, limit: usize },
    NestingDepth { observed: usize, limit: usize },
    ProcessingTime { limit_micros: u64 },
    Json(String),
}

impl BoundedJsonError {
    pub(in crate::dynamic_api) const fn is_limit_exceeded(&self) -> bool {
        matches!(
            self,
            Self::Slice(ZrByteSliceError::LengthExceedsLimit { .. })
                | Self::EncodedBytes { .. }
                | Self::Items { .. }
                | Self::NestingDepth { .. }
                | Self::ProcessingTime { .. }
        )
    }
}

impl std::fmt::Display for BoundedJsonError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Slice(error) => write!(formatter, "invalid byte slice: {error:?}"),
            Self::Empty => formatter.write_str("empty JSON payload is not allowed"),
            Self::EncodedBytes { observed, limit } => {
                write!(
                    formatter,
                    "JSON payload encoded {observed} bytes; maximum is {limit}"
                )
            }
            Self::Items { observed, limit } => {
                write!(
                    formatter,
                    "JSON payload contains {observed} items; maximum is {limit}"
                )
            }
            Self::NestingDepth { observed, limit } => {
                write!(
                    formatter,
                    "JSON payload nesting depth is {observed}; maximum is {limit}"
                )
            }
            Self::ProcessingTime { limit_micros } => {
                write!(
                    formatter,
                    "JSON processing exceeded {limit_micros} microseconds"
                )
            }
            Self::Json(message) => formatter.write_str(message),
        }
    }
}

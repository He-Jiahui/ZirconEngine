use std::fmt;

/// Failure from the exact retired-reference value walker or its caller-owned resolver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RetiredAssetRefMigrationError<E> {
    InvalidShape { message: String },
    Resolve(E),
}

impl<E: fmt::Display> fmt::Display for RetiredAssetRefMigrationError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidShape { message } => formatter.write_str(message),
            Self::Resolve(error) => write!(
                formatter,
                "retired asset reference resolution failed: {error}"
            ),
        }
    }
}

impl<E> std::error::Error for RetiredAssetRefMigrationError<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidShape { .. } => None,
            Self::Resolve(error) => Some(error),
        }
    }
}

use std::fmt;

/// Failure from the exact retired-reference value walker or its caller-owned resolver.
/// 区分旧引用形状或预算错误与调用方解析器错误，供迁移报告定位失败阶段。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RetiredAssetRefMigrationError<E> {
    InvalidShape {
        message: String,
    },
    ResourceLimitExceeded {
        resource: &'static str,
        max: usize,
        found: usize,
    },
    Resolve(E),
}

impl<E: fmt::Display> fmt::Display for RetiredAssetRefMigrationError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidShape { message } => formatter.write_str(message),
            Self::ResourceLimitExceeded {
                resource,
                max,
                found,
            } => write!(
                formatter,
                "retired asset reference migration {resource} limit {max} exceeded (found {found})"
            ),
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
            Self::InvalidShape { .. } | Self::ResourceLimitExceeded { .. } => None,
            Self::Resolve(error) => Some(error),
        }
    }
}

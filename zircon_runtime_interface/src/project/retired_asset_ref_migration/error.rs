use std::fmt;

/// Failure from the exact retired-reference value walker or its caller-owned resolver.
/// 区分旧引用形状或预算错误与调用方解析器错误，供迁移报告定位失败阶段。
///
/// # 变体
///
/// - `InvalidShape` 表示旧对象字段无法解析，或迁移结果无法序列化为 JSON。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RetiredAssetRefMigrationError<E> {
    InvalidShape {
        message: String,
    },
    /// 全树预算检查发现资源计数严格大于上限；该次检查尚未调用解析器。
    ResourceLimitExceeded {
        resource: &'static str,
        max: usize,
        found: usize,
    },
    /// 调用方解析器返回错误；此前已完成的解析器副作用不会由迁移器回滚。
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

use thiserror::Error;

/// Rejects malformed or non-unique persisted project GUID values.
/// 上述英文描述中的 non-unique 仅指 nil；跨项目 GUID 冲突由更高层身份检查负责。
/// 拒绝格式错误或 nil 的项目 GUID；此类型不判断不同项目之间的碰撞。
#[derive(Debug, Error)]
pub enum ProjectGuidParseError {
    #[error("project GUID is not a UUID: {source}")]
    InvalidUuid {
        #[source]
        source: uuid::Error,
    },
    #[error("project GUID must not be the nil UUID")]
    Nil,
}

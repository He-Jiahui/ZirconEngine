//! 通道校验错误中的数据位置标识；资产作者可据此区分键值与切线问题。
use std::fmt;

/// Location of channel data inside an animation key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationChannelDataRole {
    Value,
    InTangent,
    OutTangent,
}

impl fmt::Display for AnimationChannelDataRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Value => "value",
            Self::InTangent => "in tangent",
            Self::OutTangent => "out tangent",
        })
    }
}

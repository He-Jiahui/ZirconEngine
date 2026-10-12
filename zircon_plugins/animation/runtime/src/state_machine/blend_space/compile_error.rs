//! 混合空间输入拒绝原因；状态机编译端须阻止非有限坐标和退化样本进入帧采样。
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendSpaceCompileError {
    Empty,
    NonFinitePoint,
    DuplicatePoint,
    CollinearPoints,
    CapacityExceeded,
    TopologyFailure,
}

impl fmt::Display for BlendSpaceCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid blend space: {self:?}")
    }
}

impl Error for BlendSpaceCompileError {}

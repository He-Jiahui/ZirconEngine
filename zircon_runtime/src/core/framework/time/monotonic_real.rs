/// Monotonic frame-source marker. It is not UTC or calendar wall-clock time.
/// 零大小的真实时间域标记；不持有采样源，实际采样由 [FrameClock](crate::core::FrameClock) 完成。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MonotonicReal;

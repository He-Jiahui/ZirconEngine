//! 独立 IK 作业的输入失败类型；调用者在退化链或非有限输入时不得把结果写回骨架。
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationIkError {
    NonFiniteInput,
    DegenerateChain,
    DegenerateAxis,
    InvalidWeight,
}

impl fmt::Display for AnimationIkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid animation IK job: {self:?}")
    }
}

impl Error for AnimationIkError {}

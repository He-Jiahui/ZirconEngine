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

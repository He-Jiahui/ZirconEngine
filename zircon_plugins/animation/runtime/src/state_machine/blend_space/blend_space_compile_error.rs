use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendSpaceCompileError {
    Empty,
    NonFinitePoint,
    DuplicatePoint,
    CollinearPoints,
    CapacityExceeded,
}

impl fmt::Display for BlendSpaceCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid blend space: {self:?}")
    }
}

impl Error for BlendSpaceCompileError {}

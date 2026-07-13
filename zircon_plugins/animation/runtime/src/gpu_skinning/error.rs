use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SkinningPaletteError {
    TooManyJoints { joint_count: usize, limit: usize },
    MissingParent { bone: String, parent_index: u32 },
}

impl fmt::Display for SkinningPaletteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyJoints { joint_count, limit } => {
                write!(formatter, "skin has {joint_count} joints; limit is {limit}")
            }
            Self::MissingParent { bone, parent_index } => {
                write!(
                    formatter,
                    "bone '{bone}' references missing parent {parent_index}"
                )
            }
        }
    }
}

impl Error for SkinningPaletteError {}

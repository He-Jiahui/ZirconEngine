use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnimationGraphCompileError {
    DuplicateParameter { name: String },
    DuplicateNode { name: String },
    MissingParameter { name: String },
    MissingNode { name: String },
    MissingOutput,
    DuplicateOutput,
    UnexpectedOutputNode,
    NodeCapacityExceeded,
    ParameterCapacityExceeded,
    Cycle { name: String },
    InvalidMaskTarget { target: String },
    UnresolvedMaskTarget { target: String },
    AmbiguousMaskTarget { target: String },
}

impl Display for AnimationGraphCompileError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateParameter { name } => write!(formatter, "duplicate parameter `{name}`"),
            Self::DuplicateNode { name } => write!(formatter, "duplicate graph node `{name}`"),
            Self::MissingParameter { name } => write!(formatter, "missing parameter `{name}`"),
            Self::MissingNode { name } => write!(formatter, "missing graph node `{name}`"),
            Self::MissingOutput => formatter.write_str("animation graph has no output node"),
            Self::DuplicateOutput => formatter.write_str("animation graph has multiple outputs"),
            Self::UnexpectedOutputNode => {
                formatter.write_str("output node reached the graph data-node compiler")
            }
            Self::NodeCapacityExceeded => {
                formatter.write_str("animation graph node capacity exceeded")
            }
            Self::ParameterCapacityExceeded => {
                formatter.write_str("animation graph parameter capacity exceeded")
            }
            Self::Cycle { name } => write!(formatter, "animation graph cycle reaches `{name}`"),
            Self::InvalidMaskTarget { target } => {
                write!(formatter, "invalid mask target `{target}`")
            }
            Self::UnresolvedMaskTarget { target } => {
                write!(formatter, "unresolved mask target `{target}`")
            }
            Self::AmbiguousMaskTarget { target } => {
                write!(formatter, "ambiguous mask target `{target}`")
            }
        }
    }
}

impl Error for AnimationGraphCompileError {}

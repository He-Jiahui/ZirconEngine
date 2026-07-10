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

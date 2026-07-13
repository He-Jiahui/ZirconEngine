use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq)]
pub enum AvatarMaskError {
    Parse { message: String },
    InvalidId,
    InvalidTarget { target: String },
    UnresolvedTarget { target: String },
    AmbiguousTarget { target: String },
    InvalidWeight { target: Option<String>, weight: f32 },
}

impl Display for AvatarMaskError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse { message } => {
                write!(formatter, "avatar mask TOML parse failed: {message}")
            }
            Self::InvalidId => formatter.write_str("avatar mask id must not be empty"),
            Self::InvalidTarget { target } => write!(formatter, "invalid mask target `{target}`"),
            Self::UnresolvedTarget { target } => {
                write!(formatter, "unresolved mask target `{target}`")
            }
            Self::AmbiguousTarget { target } => {
                write!(formatter, "ambiguous mask target `{target}`")
            }
            Self::InvalidWeight { target, weight } => match target {
                Some(target) => write!(
                    formatter,
                    "invalid weight {weight} for mask target `{target}`"
                ),
                None => write!(formatter, "invalid default mask weight {weight}"),
            },
        }
    }
}

impl Error for AvatarMaskError {}

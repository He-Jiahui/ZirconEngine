use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationTransformChannel {
    Translation,
    Rotation,
    Scale,
}

impl fmt::Display for AnimationTransformChannel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Translation => "translation",
            Self::Rotation => "rotation",
            Self::Scale => "scale",
        })
    }
}

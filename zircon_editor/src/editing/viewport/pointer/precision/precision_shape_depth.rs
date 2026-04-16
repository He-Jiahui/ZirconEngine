use super::PrecisionShape;

impl PrecisionShape {
    pub(in crate::editing::viewport::pointer) fn depth(&self) -> f32 {
        match self {
            Self::Line { depth, .. } | Self::Circle { depth, .. } | Self::Ring { depth, .. } => {
                *depth
            }
        }
    }
}

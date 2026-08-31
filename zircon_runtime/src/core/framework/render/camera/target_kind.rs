#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderCameraTargetKind {
    #[default]
    PrimarySurface,
    Texture,
    Headless,
}

impl RenderCameraTargetKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::PrimarySurface => "primary_surface",
            Self::Texture => "texture",
            Self::Headless => "headless",
        }
    }
}

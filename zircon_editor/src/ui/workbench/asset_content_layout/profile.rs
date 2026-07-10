#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AssetContentSurfaceProfile {
    Activity,
    Browser,
}

impl AssetContentSurfaceProfile {
    pub(crate) fn from_surface_mode(surface_mode: &str) -> Option<Self> {
        match surface_mode {
            "activity" => Some(Self::Activity),
            "browser" => Some(Self::Browser),
            _ => None,
        }
    }
}

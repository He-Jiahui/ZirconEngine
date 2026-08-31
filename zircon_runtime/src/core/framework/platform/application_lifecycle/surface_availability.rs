#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ApplicationSurfaceAvailability {
    Unknown,
    Available,
    #[default]
    Unavailable,
}

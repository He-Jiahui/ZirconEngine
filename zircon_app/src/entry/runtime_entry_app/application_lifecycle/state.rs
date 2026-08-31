#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum ApplicationLifecycleState {
    #[default]
    Cold,
    AwaitingSurface,
    SurfaceActive,
    Suspended,
    Exiting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformHostThreadAffinity {
    MainThreadOnly,
    AnyThread,
}

/// Stable identities for every time source and derived time stream exposed by the engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ClockDomainId {
    MonotonicReal = 1,
    WallUtc = 2,
    WorldVirtual = 3,
    WorldFixed = 4,
    Input = 5,
    Render = 6,
    Audio = 7,
    Network = 8,
    Media = 9,
    EditorPreview = 10,
}

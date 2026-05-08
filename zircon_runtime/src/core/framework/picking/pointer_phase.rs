#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PointerPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

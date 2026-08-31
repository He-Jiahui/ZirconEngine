/// The scheduler only accepts monotonic host deadlines. Wall-clock time is
/// deliberately excluded so system clock adjustments cannot reorder work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventLoopClockDomain {
    Monotonic,
}

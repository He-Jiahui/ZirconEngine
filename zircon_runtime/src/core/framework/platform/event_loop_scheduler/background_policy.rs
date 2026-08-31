/// Host-owned background execution posture observed by the event-loop
/// scheduler. The policy owner supplies any corresponding wake deadline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventLoopBackgroundPolicy {
    Foreground,
    Throttled,
    Suspended,
}

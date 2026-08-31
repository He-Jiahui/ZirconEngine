/// Unit carried by a clock-domain value so duration and calendar values cannot be interchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClockDomainUnit {
    Duration,
    UnixTimestamp,
}

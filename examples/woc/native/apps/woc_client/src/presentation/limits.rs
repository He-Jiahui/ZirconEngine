use woc_protocol::limit;

pub const MAX_PENDING_COMMANDS: usize = limit::COMMANDS_PER_TICK as usize;

/// Large wall-clock jumps are treated as a suspend/discontinuity. Refusing
/// them keeps the accumulator bounded and lets the host submit an explicit
/// resume/cancellation frame instead of silently creating unbounded debt.
pub const MAX_FRAME_DELTA_NS: u64 = 250_000_000;

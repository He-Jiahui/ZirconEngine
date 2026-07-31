use woc_protocol::limit;

pub const MAX_PENDING_COMMANDS: usize = limit::COMMANDS_PER_TICK as usize;

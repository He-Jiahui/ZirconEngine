//! Level-owned script diagnostics; recording never mutates the World.
use crate::scene::EntityId;

pub(crate) const SCRIPT_BINDING_FAULT_CAPACITY: usize = 256;
pub(crate) const SCRIPT_FAULT_MESSAGE_MAX_BYTES: usize = 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScriptBindingFaultKind {
    Resolution,
    Guest,
    HostCall { module: String, function: String },
    InvalidBinding,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptBindingFault {
    pub entity: EntityId,
    /// Absent when malformed binding data has no valid identity.
    pub binding_key: Option<String>,
    pub package: String,
    pub module: String,
    pub export: String,
    pub kind: ScriptBindingFaultKind,
    pub message: String,
    pub host_fault_count: u64,
    pub outer_frame_index: u64,
}

pub(crate) fn bounded_script_fault_message(mut message: String) -> String {
    if message.len() > SCRIPT_FAULT_MESSAGE_MAX_BYTES {
        let mut end = SCRIPT_FAULT_MESSAGE_MAX_BYTES;
        while !message.is_char_boundary(end) {
            end -= 1;
        }
        message.truncate(end);
    }
    message
}

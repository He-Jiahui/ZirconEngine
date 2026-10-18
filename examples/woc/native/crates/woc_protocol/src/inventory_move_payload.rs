use crate::{validate_command_payload, ProtocolError, INVENTORY_MOVE_COMMAND_ID};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InventoryMovePayload {
    pub from: i32,
    pub to: i32,
}

impl InventoryMovePayload {
    pub fn encode(self) -> [u8; 8] {
        let mut bytes = [0; 8];
        bytes[..4].copy_from_slice(&self.from.to_le_bytes());
        bytes[4..].copy_from_slice(&self.to.to_le_bytes());
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(INVENTORY_MOVE_COMMAND_ID, bytes)?;
        Ok(Self {
            from: i32::from_le_bytes(
                bytes[..4]
                    .try_into()
                    .expect("validated inventory from-index has four bytes"),
            ),
            to: i32::from_le_bytes(
                bytes[4..]
                    .try_into()
                    .expect("validated inventory target-cell has four bytes"),
            ),
        })
    }
}

#[cfg(test)]
#[path = "tests/inventory_move_payload.rs"]
mod tests;

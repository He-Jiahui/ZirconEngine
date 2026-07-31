use crate::{
    require_finite, validate_command_payload, ProtocolError, AUTO_LOOT_COMMAND_ID,
    COLLECT_DELVE_CHEST_LOOT_COMMAND_ID, DELVE_INTERACT_COMMAND_ID, LOOT_COMMAND_ID,
    PICKUP_COMMAND_ID,
};

const WORLD_OBJECT_ID_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldObjectAction {
    Loot,
    Pickup,
    AutoLoot,
    DelveInteract,
    CollectDelveChestLoot,
}

impl WorldObjectAction {
    const fn command_id(self) -> u16 {
        match self {
            Self::Loot => LOOT_COMMAND_ID,
            Self::Pickup => PICKUP_COMMAND_ID,
            Self::AutoLoot => AUTO_LOOT_COMMAND_ID,
            Self::DelveInteract => DELVE_INTERACT_COMMAND_ID,
            Self::CollectDelveChestLoot => COLLECT_DELVE_CHEST_LOOT_COMMAND_ID,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorldObjectIdPayload {
    pub object_id: f64,
}

impl WorldObjectIdPayload {
    pub fn encode(
        self,
        action: WorldObjectAction,
    ) -> Result<[u8; WORLD_OBJECT_ID_BYTES], ProtocolError> {
        let object_id = canonical_finite_f64("WorldObjectIdPayload.object_id", self.object_id)?;
        let bytes = object_id.to_le_bytes();
        validate_command_payload(action.command_id(), &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8], action: WorldObjectAction) -> Result<Self, ProtocolError> {
        validate_command_payload(action.command_id(), bytes)?;
        Ok(Self {
            object_id: read_finite_f64(bytes, "WorldObjectIdPayload.object_id")?,
        })
    }
}

pub(crate) fn validate_world_object_id_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = read_finite_f64(bytes, "WorldObjectIdPayload.object_id")?;
    Ok(())
}

fn canonical_finite_f64(context: &'static str, value: f64) -> Result<f64, ProtocolError> {
    let value = require_finite(context, value)?;
    Ok(if value == 0.0 { 0.0 } else { value })
}

fn read_finite_f64(bytes: &[u8], context: &'static str) -> Result<f64, ProtocolError> {
    let value = f64::from_le_bytes(
        bytes[..WORLD_OBJECT_ID_BYTES]
            .try_into()
            .expect("validated world-object payload contains a complete f64"),
    );
    canonical_finite_f64(context, value)
}

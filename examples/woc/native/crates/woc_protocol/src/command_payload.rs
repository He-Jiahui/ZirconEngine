use crate::bank_payload::validate_bank_slot_optional_count_payload;
use crate::challenge_payload::validate_challenge_response_payload;
use crate::corpse_harvest_payload::validate_corpse_harvest_payload;
use crate::delve_rite_payload::validate_delve_rite_intensity_payload;
use crate::duel_arena_payload::{
    validate_arena_augment_payload, validate_arena_queue_payload, validate_duel_request_payload,
};
use crate::dungeon_difficulty_payload::validate_dungeon_difficulty_payload;
use crate::dungeon_finder_payload::{
    validate_dungeon_finder_activities_payload,
    validate_dungeon_finder_application_response_payload,
    validate_dungeon_finder_listing_id_payload, validate_dungeon_finder_listing_payload,
    validate_dungeon_finder_roles_payload,
};
use crate::equipment_payload::{validate_equip_item_payload, validate_unequip_item_payload};
use crate::event_skin_payload::validate_event_skin_payload;
use crate::linked_quest_payload::validate_linked_quest_acceptance_payload;
use crate::loot_roll_payload::validate_loot_roll_payload;
use crate::mail_payload::{validate_mail_id_payload, validate_mail_send_payload};
use crate::market_payload::{validate_market_listing_id_payload, validate_market_search_payload};
use crate::master_loot_assignment_payload::validate_master_loot_assignment_payload;
use crate::party_payload::{
    validate_party_loot_master_payload, validate_party_marker_clear_payload,
    validate_party_marker_payload,
};
use crate::save_loadout_payload::validate_save_loadout_payload;
use crate::telemetry_payload::validate_telemetry_payload;
use crate::town_focus_payload::validate_town_focus_payload;
use crate::trade_payload::{validate_trade_offer_payload, validate_trade_request_payload};
use crate::vale_cup_payload::{
    validate_vale_cup_bet_payload, validate_vale_cup_practice_payload,
    validate_vale_cup_queue_payload, validate_vale_cup_role_payload,
};
use crate::weapon_skin_payload::validate_weapon_skin_payload;
use crate::world_object_payload::validate_world_object_id_payload;
use crate::{
    command_payload_descriptor, require_finite, talent_option_id, talent_spec_id,
    CommandPayloadDescriptor, CommandPayloadKind, ProtocolError, ABANDON_QUEST_COMMAND_ID,
    ACCEPT_QUEST_COMMAND_ID, APPLY_TALENTS_COMMAND_ID, BUYBACK_COMMAND_ID, BUY_COMMAND_ID,
    CANCEL_AURA_COMMAND_ID, CARD_PLAY_COMMAND_ID, CAST_AT_COMMAND_ID, CAST_COMMAND_ID,
    CAST_SLOT_COMMAND_ID, CHANGE_SKIN_COMMAND_ID, CHAT_COMMAND_ID, COMPANION_UPGRADE_COMMAND_ID,
    CRAFT_ITEM_COMMAND_ID, DEED_SET_TITLE_COMMAND_ID, DELETE_LOADOUT_COMMAND_ID,
    DELVE_BUY_COMMAND_ID, DISCARD_ITEM_COMMAND_ID, ENTER_DELVE_COMMAND_ID,
    ENTER_DUNGEON_COMMAND_ID, EQUIP_BAG_COMMAND_ID, GUILD_EVENT_CREATE_COMMAND_ID,
    GUILD_EVENT_REMOVE_COMMAND_ID, HARVEST_NODE_COMMAND_ID, HEROIC_BUY_COMMAND_ID,
    LOCKPICK_ABORT_COMMAND_ID, LOCKPICK_ACTION_COMMAND_ID, LOCKPICK_ENGAGE_COMMAND_ID,
    MARKET_LIST_COMMAND_ID, PARTY_MOVE_RAID_COMMAND_ID, PET_AUTO_TAUNT_COMMAND_ID,
    PET_AUTO_WATER_JET_COMMAND_ID, PET_FEED_COMMAND_ID, PET_MODE_COMMAND_ID, PET_RENAME_COMMAND_ID,
    RELEASE_EMPOWERED_COMMAND_ID, RESURRECT_RESPOND_COMMAND_ID, SELECT_TALENT_ROW_COMMAND_ID,
    SELL_COMMAND_ID, SET_SPEC_COMMAND_ID, SWITCH_LOADOUT_COMMAND_ID, TARGET_COMMAND_ID,
    TURN_IN_QUEST_COMMAND_ID, UNEQUIP_BAG_COMMAND_ID, UNEQUIP_MECH_CHROMA_COMMAND_ID,
    USE_ITEM_COMMAND_ID,
};

const LENGTH_PREFIX_BYTES: usize = 4;
const OPTIONAL_U32_PRESENCE_BYTES: usize = 1;
const OPTIONAL_TARGET_PRESENCE_BYTES: usize = 1;
const OPTIONAL_GUILD_EVENT_HOUR_PRESENCE_BYTES: usize = 1;
const GROUND_POINT_BYTES: usize = 16;
const TALENT_ALLOCATION_ROW_COUNT: usize = 6;
const TALENT_LOADOUT_LIMIT: u32 = 10;
const CLASS_SKIN_INDEX_MAX: u8 = 7;
const GUILD_EVENT_DAY_MAX_UTF8_BYTES: usize = 10;
const GUILD_EVENT_TITLE_MAX_UTF8_BYTES: usize = 192;
const GUILD_EVENT_NOTE_MAX_UTF8_BYTES: usize = 640;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CastSlotCommandPayload {
    pub slot: i32,
}

impl CastSlotCommandPayload {
    pub fn encode(self) -> [u8; 4] {
        self.slot.to_le_bytes()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(CAST_SLOT_COMMAND_ID, bytes)?;
        Ok(Self {
            slot: i32::from_le_bytes(
                bytes
                    .try_into()
                    .expect("validated cast-slot command payload has four bytes"),
            ),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardPlayCommandPayload {
    pub card_value: i32,
}

impl CardPlayCommandPayload {
    pub fn encode(self) -> [u8; 4] {
        self.card_value.to_le_bytes()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(CARD_PLAY_COMMAND_ID, bytes)?;
        Ok(Self {
            card_value: i32::from_le_bytes(
                bytes
                    .try_into()
                    .expect("validated card-play command payload has four bytes"),
            ),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkinCatalog {
    Class,
    Mech,
}

impl SkinCatalog {
    pub const fn wire_code(self) -> u8 {
        match self {
            Self::Class => 0,
            Self::Mech => 1,
        }
    }

    fn from_wire_code(code: u8) -> Result<Self, ProtocolError> {
        match code {
            0 => Ok(Self::Class),
            1 => Ok(Self::Mech),
            _ => Err(ProtocolError::InvalidSkinCatalog(code)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChangeSkinCommandPayload {
    pub catalog: SkinCatalog,
    pub skin_index: u8,
}

impl ChangeSkinCommandPayload {
    pub fn encode(self) -> Result<[u8; 2], ProtocolError> {
        validate_cosmetic_skin(self.catalog.wire_code(), self.skin_index)?;
        let bytes = [self.catalog.wire_code(), self.skin_index];
        validate_command_payload(CHANGE_SKIN_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(CHANGE_SKIN_COMMAND_ID, bytes)?;
        Ok(Self {
            catalog: SkinCatalog::from_wire_code(bytes[0])?,
            skin_index: bytes[1],
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectTalentRowCommandPayload {
    pub level: u8,
    /// Zero is the canonical representation of the source `optionId: null` clear.
    pub option_code: u16,
}

impl SelectTalentRowCommandPayload {
    pub fn encode(self) -> Result<[u8; 3], ProtocolError> {
        validate_talent_row_selection(self.level, self.option_code)?;
        let mut bytes = [0; 3];
        bytes[0] = self.level;
        bytes[1..].copy_from_slice(&self.option_code.to_le_bytes());
        validate_command_payload(SELECT_TALENT_ROW_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(SELECT_TALENT_ROW_COMMAND_ID, bytes)?;
        Ok(Self {
            level: bytes[0],
            option_code: u16::from_le_bytes(
                bytes[1..]
                    .try_into()
                    .expect("validated talent-row command payload has two option bytes"),
            ),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetSpecCommandPayload {
    /// Zero is the canonical representation of the source `spec: null` clear.
    pub spec_code: u16,
}

impl SetSpecCommandPayload {
    pub fn encode(self) -> Result<[u8; 2], ProtocolError> {
        validate_talent_spec(self.spec_code)?;
        let bytes = self.spec_code.to_le_bytes();
        validate_command_payload(SET_SPEC_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(SET_SPEC_COMMAND_ID, bytes)?;
        Ok(Self {
            spec_code: u16::from_le_bytes(
                bytes
                    .try_into()
                    .expect("validated set-spec command payload has two bytes"),
            ),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApplyTalentsCommandPayload {
    /// Zero is the canonical representation of source `alloc.spec: null`.
    pub spec_code: u16,
    /// Each zero is the canonical representation of an absent source allocation row.
    pub row_option_codes: [u16; TALENT_ALLOCATION_ROW_COUNT],
}

impl ApplyTalentsCommandPayload {
    pub fn encode(self) -> Result<[u8; 14], ProtocolError> {
        validate_talent_allocation_codes(self.spec_code, &self.row_option_codes)?;
        let mut bytes = [0; 14];
        bytes[..2].copy_from_slice(&self.spec_code.to_le_bytes());
        for (index, option_code) in self.row_option_codes.iter().enumerate() {
            let offset = 2 + index * 2;
            bytes[offset..offset + 2].copy_from_slice(&option_code.to_le_bytes());
        }
        validate_command_payload(APPLY_TALENTS_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(APPLY_TALENTS_COMMAND_ID, bytes)?;
        let (spec_code, row_option_codes) = decode_talent_allocation_codes(bytes);
        Ok(Self {
            spec_code,
            row_option_codes,
        })
    }
}

/// A finite, canonicalized ground point that remains comparable in client inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroundTargetPoint {
    x_bits: u64,
    z_bits: u64,
}

impl GroundTargetPoint {
    pub fn new(x: f64, z: f64) -> Result<Self, ProtocolError> {
        Self::from_coordinates(x, z, "GroundTargetPoint.x", "GroundTargetPoint.z")
    }

    pub fn x(self) -> f64 {
        f64::from_bits(self.x_bits)
    }

    pub fn z(self) -> f64 {
        f64::from_bits(self.z_bits)
    }

    fn from_coordinates(
        x: f64,
        z: f64,
        x_context: &'static str,
        z_context: &'static str,
    ) -> Result<Self, ProtocolError> {
        Ok(Self {
            x_bits: canonical_finite_f64(x_context, x)?.to_bits(),
            z_bits: canonical_finite_f64(z_context, z)?.to_bits(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CastAtCommandPayload {
    pub ability_id: String,
    pub aim: GroundTargetPoint,
}

impl CastAtCommandPayload {
    pub fn new(ability_id: String, x: f64, z: f64) -> Result<Self, ProtocolError> {
        Ok(Self {
            ability_id,
            aim: GroundTargetPoint::from_coordinates(
                x,
                z,
                "CastAtCommandPayload.x",
                "CastAtCommandPayload.z",
            )?,
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let descriptor = descriptor(CAST_AT_COMMAND_ID)?;
        validate_utf8_length(descriptor, self.ability_id.len())?;
        let mut bytes =
            Vec::with_capacity(LENGTH_PREFIX_BYTES + self.ability_id.len() + GROUND_POINT_BYTES);
        bytes.extend_from_slice(
            &u32::try_from(self.ability_id.len())
                .expect("bounded command identifier length fits u32")
                .to_le_bytes(),
        );
        bytes.extend_from_slice(self.ability_id.as_bytes());
        bytes.extend_from_slice(&self.aim.x().to_le_bytes());
        bytes.extend_from_slice(&self.aim.z().to_le_bytes());
        validate_command_payload(CAST_AT_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(CAST_AT_COMMAND_ID, bytes)?;
        let (ability_id, consumed) = read_utf8_id(descriptor(CAST_AT_COMMAND_ID)?, bytes)?;
        let (x, consumed) = read_finite_f64(bytes, consumed, "CastAtCommandPayload.x")?;
        let (z, consumed) = read_finite_f64(bytes, consumed, "CastAtCommandPayload.z")?;
        reject_trailing(bytes, consumed)?;
        Ok(Self {
            ability_id: ability_id.to_owned(),
            aim: GroundTargetPoint::from_coordinates(
                x,
                z,
                "CastAtCommandPayload.x",
                "CastAtCommandPayload.z",
            )?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CastAbilityCommandPayload {
    pub ability_id: String,
    /// `None` is castAbility; `Some` is the explicit castAbilityOn override.
    /// Zero remains representable because the source forwards any numeric override
    /// and lets the authoritative target resolver reject an absent entity.
    pub target_id: Option<u64>,
}

impl CastAbilityCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let descriptor = descriptor(CAST_COMMAND_ID)?;
        let mut bytes = Vec::with_capacity(
            LENGTH_PREFIX_BYTES
                + self.ability_id.len()
                + OPTIONAL_TARGET_PRESENCE_BYTES
                + self.target_id.map(|_| 8).unwrap_or_default(),
        );
        write_utf8_id(descriptor, &self.ability_id, &mut bytes)?;
        match self.target_id {
            Some(target_id) => {
                bytes.push(1);
                bytes.extend_from_slice(&target_id.to_le_bytes());
            }
            None => bytes.push(0),
        }
        validate_command_payload(CAST_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(CAST_COMMAND_ID, bytes)?;
        let descriptor = descriptor(CAST_COMMAND_ID)?;
        let (ability_id, consumed) = read_utf8_id(descriptor, bytes)?;
        let (target_id, consumed) = read_optional_target_id(bytes, consumed)?;
        reject_trailing(bytes, consumed)?;
        Ok(Self {
            ability_id: ability_id.to_owned(),
            target_id,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CancelAuraCommandPayload {
    pub aura_id: String,
}

impl CancelAuraCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id(CANCEL_AURA_COMMAND_ID, &self.aura_id)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            aura_id: decode_utf8_id(CANCEL_AURA_COMMAND_ID, bytes)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReleaseEmpoweredCommandPayload {
    pub ability_id: String,
}

impl ReleaseEmpoweredCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id(RELEASE_EMPOWERED_COMMAND_ID, &self.ability_id)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            ability_id: decode_utf8_id(RELEASE_EMPOWERED_COMMAND_ID, bytes)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PetRenameCommandPayload {
    pub name: String,
}

impl PetRenameCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id(PET_RENAME_COMMAND_ID, &self.name)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            name: decode_utf8_id(PET_RENAME_COMMAND_ID, bytes)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PetFeedCommandPayload {
    pub item_id: String,
}

impl PetFeedCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id(PET_FEED_COMMAND_ID, &self.item_id)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            item_id: decode_utf8_id(PET_FEED_COMMAND_ID, bytes)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PetModeCommandPayload {
    pub mode: String,
}

impl PetModeCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id(PET_MODE_COMMAND_ID, &self.mode)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            mode: decode_utf8_id(PET_MODE_COMMAND_ID, bytes)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SocialNameCommandPayload {
    pub name: String,
}

impl SocialNameCommandPayload {
    pub fn encode(&self, command_id: u16) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id(command_id, &self.name)
    }

    pub fn decode(command_id: u16, bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            name: decode_utf8_id(command_id, bytes)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
// Preserve the source service's retained text prefix while keeping the wire bounded.
pub struct GuildEventCreateCommandPayload {
    pub day: String,
    pub hour: Option<f64>,
    pub title: String,
    pub note: String,
}

impl GuildEventCreateCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut bytes = Vec::with_capacity(
            LENGTH_PREFIX_BYTES * 3
                + OPTIONAL_GUILD_EVENT_HOUR_PRESENCE_BYTES
                + self.day.len()
                + self.title.len()
                + self.note.len()
                + self.hour.map(|_| 8).unwrap_or_default(),
        );
        write_bounded_utf8(
            &self.day,
            GUILD_EVENT_DAY_MAX_UTF8_BYTES,
            "GuildEventCreateCommandPayload.day",
            &mut bytes,
        )?;
        match self.hour {
            None => bytes.push(0),
            Some(hour) => {
                bytes.push(1);
                bytes.extend_from_slice(
                    &canonical_finite_f64("GuildEventCreateCommandPayload.hour", hour)?
                        .to_le_bytes(),
                );
            }
        }
        write_bounded_utf8(
            &self.title,
            GUILD_EVENT_TITLE_MAX_UTF8_BYTES,
            "GuildEventCreateCommandPayload.title",
            &mut bytes,
        )?;
        write_bounded_utf8(
            &self.note,
            GUILD_EVENT_NOTE_MAX_UTF8_BYTES,
            "GuildEventCreateCommandPayload.note",
            &mut bytes,
        )?;
        validate_command_payload(GUILD_EVENT_CREATE_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(GUILD_EVENT_CREATE_COMMAND_ID, bytes)?;
        let (day, consumed) = read_bounded_utf8_at(
            bytes,
            0,
            GUILD_EVENT_DAY_MAX_UTF8_BYTES,
            "GuildEventCreateCommandPayload.day",
        )?;
        let (hour, consumed) = read_optional_guild_event_hour(bytes, consumed)?;
        let (title, consumed) = read_bounded_utf8_at(
            bytes,
            consumed,
            GUILD_EVENT_TITLE_MAX_UTF8_BYTES,
            "GuildEventCreateCommandPayload.title",
        )?;
        let (note, consumed) = read_bounded_utf8_at(
            bytes,
            consumed,
            GUILD_EVENT_NOTE_MAX_UTF8_BYTES,
            "GuildEventCreateCommandPayload.note",
        )?;
        reject_trailing(bytes, consumed)?;
        Ok(Self {
            day: day.to_owned(),
            hour,
            title: title.to_owned(),
            note: note.to_owned(),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuildEventRemoveCommandPayload {
    pub event_id: u32,
}

impl GuildEventRemoveCommandPayload {
    pub fn encode(self) -> [u8; 4] {
        self.event_id.to_le_bytes()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(GUILD_EVENT_REMOVE_COMMAND_ID, bytes)?;
        Ok(Self {
            event_id: u32::from_le_bytes(
                bytes
                    .try_into()
                    .expect("validated guild-event-remove payload has four bytes"),
            ),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetCommandPayload {
    pub target_id: Option<u64>,
}

impl TargetCommandPayload {
    pub fn encode(self) -> Result<[u8; 8], ProtocolError> {
        let target_id = match self.target_id {
            Some(0) => {
                return Err(ProtocolError::InvalidEntityId {
                    context: "TargetCommandPayload.target_id",
                });
            }
            Some(target_id) => target_id,
            None => 0,
        };
        Ok(target_id.to_le_bytes())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(TARGET_COMMAND_ID, bytes)?;
        let target_id = u64::from_le_bytes(
            bytes
                .try_into()
                .expect("validated target command payload has eight bytes"),
        );
        Ok(Self {
            target_id: (target_id != 0).then_some(target_id),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartyMoveRaidCommandPayload {
    pub target_id: u64,
    pub subgroup: u8,
}

impl PartyMoveRaidCommandPayload {
    pub fn encode(self) -> Result<[u8; 9], ProtocolError> {
        if self.target_id == 0 {
            return Err(ProtocolError::InvalidEntityId {
                context: "PartyMoveRaidCommandPayload.target_id",
            });
        }
        validate_raid_subgroup(self.subgroup)?;
        let mut bytes = [0; 9];
        bytes[..8].copy_from_slice(&self.target_id.to_le_bytes());
        bytes[8] = self.subgroup;
        validate_command_payload(PARTY_MOVE_RAID_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(PARTY_MOVE_RAID_COMMAND_ID, bytes)?;
        let target_id = u64::from_le_bytes(
            bytes[..8]
                .try_into()
                .expect("validated party raid-move payload has eight target bytes"),
        );
        if target_id == 0 {
            return Err(ProtocolError::InvalidEntityId {
                context: "PartyMoveRaidCommandPayload.target_id",
            });
        }
        Ok(Self {
            target_id,
            subgroup: bytes[8],
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptQuestCommandPayload {
    pub quest_id: String,
    pub selection: Option<String>,
}

impl AcceptQuestCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let descriptor = descriptor(ACCEPT_QUEST_COMMAND_ID)?;
        let mut bytes = Vec::with_capacity(
            LENGTH_PREFIX_BYTES
                + self.quest_id.len()
                + 1
                + self
                    .selection
                    .as_ref()
                    .map(|selection| LENGTH_PREFIX_BYTES + selection.len())
                    .unwrap_or_default(),
        );
        write_utf8_id(descriptor, &self.quest_id, &mut bytes)?;
        write_optional_utf8_id(descriptor, self.selection.as_deref(), &mut bytes)?;
        validate_command_payload(ACCEPT_QUEST_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(ACCEPT_QUEST_COMMAND_ID, bytes)?;
        let descriptor = descriptor(ACCEPT_QUEST_COMMAND_ID)?;
        let (quest_id, consumed) = read_utf8_id(descriptor, bytes)?;
        let (selection, consumed) = read_optional_utf8_id_at(descriptor, bytes, consumed)?;
        reject_trailing(bytes, consumed)?;
        Ok(Self {
            quest_id: quest_id.to_owned(),
            selection: selection.map(str::to_owned),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AbandonQuestCommandPayload {
    pub quest_id: String,
}

impl AbandonQuestCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id(ABANDON_QUEST_COMMAND_ID, &self.quest_id)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            quest_id: decode_utf8_id(ABANDON_QUEST_COMMAND_ID, bytes)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TurnInQuestCommandPayload {
    pub quest_id: String,
}

impl TurnInQuestCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id(TURN_IN_QUEST_COMMAND_ID, &self.quest_id)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            quest_id: decode_utf8_id(TURN_IN_QUEST_COMMAND_ID, bytes)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UseItemCommandPayload {
    pub item_id: String,
}

impl UseItemCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id(USE_ITEM_COMMAND_ID, &self.item_id)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            item_id: decode_utf8_id(USE_ITEM_COMMAND_ID, bytes)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HarvestNodeCommandPayload {
    pub node_id: String,
}

impl HarvestNodeCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id(HARVEST_NODE_COMMAND_ID, &self.node_id)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            node_id: decode_utf8_id(HARVEST_NODE_COMMAND_ID, bytes)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnterDungeonCommandPayload {
    pub dungeon_id: String,
}

impl EnterDungeonCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id(ENTER_DUNGEON_COMMAND_ID, &self.dungeon_id)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            dungeon_id: decode_utf8_id(ENTER_DUNGEON_COMMAND_ID, bytes)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CraftItemCommandPayload {
    pub recipe_id: String,
}

impl CraftItemCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id(CRAFT_ITEM_COMMAND_ID, &self.recipe_id)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            recipe_id: decode_utf8_id(CRAFT_ITEM_COMMAND_ID, bytes)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeroicBuyCommandPayload {
    pub item_id: String,
}

impl HeroicBuyCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id(HEROIC_BUY_COMMAND_ID, &self.item_id)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            item_id: decode_utf8_id(HEROIC_BUY_COMMAND_ID, bytes)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DelveBuyCommandPayload {
    pub delve_id: String,
    pub item_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnterDelveCommandPayload {
    pub delve_id: String,
    pub tier_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompanionUpgradeCommandPayload {
    pub companion_id: String,
}

impl CompanionUpgradeCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id(COMPANION_UPGRADE_COMMAND_ID, &self.companion_id)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            companion_id: decode_utf8_id(COMPANION_UPGRADE_COMMAND_ID, bytes)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnequipMechChromaCommandPayload {
    pub chroma_id: String,
}

impl UnequipMechChromaCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id(UNEQUIP_MECH_CHROMA_COMMAND_ID, &self.chroma_id)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            chroma_id: decode_utf8_id(UNEQUIP_MECH_CHROMA_COMMAND_ID, bytes)?,
        })
    }
}

impl DelveBuyCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id_pair(DELVE_BUY_COMMAND_ID, &self.delve_id, &self.item_id)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let (delve_id, item_id) = decode_utf8_id_pair(DELVE_BUY_COMMAND_ID, bytes)?;
        Ok(Self { delve_id, item_id })
    }
}

impl EnterDelveCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id_pair(ENTER_DELVE_COMMAND_ID, &self.delve_id, &self.tier_id)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let (delve_id, tier_id) = decode_utf8_id_pair(ENTER_DELVE_COMMAND_ID, bytes)?;
        Ok(Self { delve_id, tier_id })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatCommandPayload {
    pub text: String,
}

impl ChatCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id(CHAT_COMMAND_ID, &self.text)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            text: decode_utf8_id(CHAT_COMMAND_ID, bytes)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeedSetTitleCommandPayload {
    pub deed_id: Option<String>,
}

impl DeedSetTitleCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let descriptor = descriptor(DEED_SET_TITLE_COMMAND_ID)?;
        let mut bytes = Vec::with_capacity(descriptor.max_byte_length);
        write_optional_utf8_id(descriptor, self.deed_id.as_deref(), &mut bytes)?;
        validate_command_payload(DEED_SET_TITLE_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(DEED_SET_TITLE_COMMAND_ID, bytes)?;
        let (deed_id, _) = read_optional_utf8_id(descriptor(DEED_SET_TITLE_COMMAND_ID)?, bytes)?;
        Ok(Self {
            deed_id: deed_id.map(str::to_owned),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscardItemCommandPayload {
    pub item_id: String,
    pub count: Option<u32>,
}

impl DiscardItemCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id_optional_u32(DISCARD_ITEM_COMMAND_ID, &self.item_id, self.count)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let (item_id, count) = decode_utf8_id_optional_u32(DISCARD_ITEM_COMMAND_ID, bytes)?;
        Ok(Self { item_id, count })
    }
}

// Source `buyItem(npcId, itemId)` has a numeric NPC identity and a string item
// identity. The binary command keeps the same information in canonical order:
// u32 UTF-8 item length, item bytes, required-presence byte, then u64 NPC id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuyItemCommandPayload {
    pub npc_id: u64,
    pub item_id: String,
}

impl BuyItemCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let descriptor = descriptor(BUY_COMMAND_ID)?;
        let mut bytes = Vec::with_capacity(
            LENGTH_PREFIX_BYTES + self.item_id.len() + OPTIONAL_TARGET_PRESENCE_BYTES + 8,
        );
        write_utf8_id(descriptor, &self.item_id, &mut bytes)?;
        bytes.push(1);
        bytes.extend_from_slice(&self.npc_id.to_le_bytes());
        validate_command_payload(BUY_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(BUY_COMMAND_ID, bytes)?;
        let descriptor = descriptor(BUY_COMMAND_ID)?;
        let (item_id, consumed) = read_utf8_id(descriptor, bytes)?;
        let (npc_id, consumed) = read_optional_target_id(bytes, consumed)?;
        reject_trailing(bytes, consumed)?;
        Ok(Self {
            npc_id: npc_id.ok_or(ProtocolError::InvalidEntityId {
                context: "BuyItemCommandPayload.npc_id",
            })?,
            item_id: item_id.to_owned(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SellItemCommandPayload {
    pub item_id: String,
    pub count: Option<u32>,
}

impl SellItemCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id_optional_u32(SELL_COMMAND_ID, &self.item_id, self.count)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let (item_id, count) = decode_utf8_id_optional_u32(SELL_COMMAND_ID, bytes)?;
        Ok(Self { item_id, count })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuybackItemCommandPayload {
    pub item_id: String,
}

impl BuybackItemCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id(BUYBACK_COMMAND_ID, &self.item_id)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            item_id: decode_utf8_id(BUYBACK_COMMAND_ID, bytes)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EquipBagCommandPayload {
    pub item_id: String,
    pub socket: Option<u32>,
}

impl EquipBagCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id_optional_u32(EQUIP_BAG_COMMAND_ID, &self.item_id, self.socket)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let (item_id, socket) = decode_utf8_id_optional_u32(EQUIP_BAG_COMMAND_ID, bytes)?;
        Ok(Self { item_id, socket })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnequipBagCommandPayload {
    pub socket: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwitchLoadoutCommandPayload {
    pub index: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeleteLoadoutCommandPayload {
    pub index: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResurrectRespondCommandPayload {
    pub accept: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PetAutoWaterJetCommandPayload {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PetAutoTauntCommandPayload {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockpickAction {
    HardSet,
    Set,
    Steady,
    Ease,
    Drop,
    Abort,
}

impl LockpickAction {
    pub const fn wire_code(self) -> u8 {
        match self {
            Self::HardSet => 0,
            Self::Set => 1,
            Self::Steady => 2,
            Self::Ease => 3,
            Self::Drop => 4,
            Self::Abort => 5,
        }
    }

    fn from_wire_code(value: u8) -> Result<Self, ProtocolError> {
        match value {
            0 => Ok(Self::HardSet),
            1 => Ok(Self::Set),
            2 => Ok(Self::Steady),
            3 => Ok(Self::Ease),
            4 => Ok(Self::Drop),
            5 => Ok(Self::Abort),
            _ => Err(ProtocolError::InvalidLockpickAction(value)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LockpickEngageCommandPayload {
    pub object_id: u64,
    pub ante: u8,
}

impl LockpickEngageCommandPayload {
    pub fn encode(self) -> Result<[u8; 9], ProtocolError> {
        validate_lockpick_ante(self.ante)?;
        let mut bytes = [0; 9];
        bytes[..8].copy_from_slice(&self.object_id.to_le_bytes());
        bytes[8] = self.ante;
        validate_command_payload(LOCKPICK_ENGAGE_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(LOCKPICK_ENGAGE_COMMAND_ID, bytes)?;
        let object_id = u64::from_le_bytes(
            bytes[..8]
                .try_into()
                .expect("validated lockpick-engage command payload has eight object-id bytes"),
        );
        let ante = bytes[8];
        validate_lockpick_ante(ante)?;
        Ok(Self { object_id, ante })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockpickActionCommandPayload {
    pub session_id: Option<String>,
    pub action: LockpickAction,
}

impl LockpickActionCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let descriptor = descriptor(LOCKPICK_ACTION_COMMAND_ID)?;
        let mut bytes = Vec::with_capacity(descriptor.max_byte_length);
        write_optional_utf8_id(descriptor, self.session_id.as_deref(), &mut bytes)?;
        bytes.push(self.action.wire_code());
        validate_command_payload(LOCKPICK_ACTION_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(LOCKPICK_ACTION_COMMAND_ID, bytes)?;
        let descriptor = descriptor(LOCKPICK_ACTION_COMMAND_ID)?;
        let (session_id, consumed) = read_optional_utf8_id(descriptor, bytes)?;
        let action = LockpickAction::from_wire_code(
            take(bytes, consumed, 1, "LockpickActionCommandPayload.action")?[0],
        )?;
        Ok(Self {
            session_id: session_id.map(str::to_owned),
            action,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockpickAbortCommandPayload {
    pub session_id: Option<String>,
}

impl LockpickAbortCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let descriptor = descriptor(LOCKPICK_ABORT_COMMAND_ID)?;
        let mut bytes = Vec::with_capacity(descriptor.max_byte_length);
        write_optional_utf8_id(descriptor, self.session_id.as_deref(), &mut bytes)?;
        validate_command_payload(LOCKPICK_ABORT_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(LOCKPICK_ABORT_COMMAND_ID, bytes)?;
        let (session_id, _) = read_optional_utf8_id(descriptor(LOCKPICK_ABORT_COMMAND_ID)?, bytes)?;
        Ok(Self {
            session_id: session_id.map(str::to_owned),
        })
    }
}

impl UnequipBagCommandPayload {
    pub fn encode(self) -> [u8; 4] {
        self.socket.to_le_bytes()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(UNEQUIP_BAG_COMMAND_ID, bytes)?;
        Ok(Self {
            socket: u32::from_le_bytes(
                bytes
                    .try_into()
                    .expect("validated unequip-bag command payload has four bytes"),
            ),
        })
    }
}

impl SwitchLoadoutCommandPayload {
    pub fn encode(self) -> Result<[u8; 4], ProtocolError> {
        encode_talent_loadout_index(SWITCH_LOADOUT_COMMAND_ID, self.index)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            index: decode_talent_loadout_index(SWITCH_LOADOUT_COMMAND_ID, bytes)?,
        })
    }
}

impl DeleteLoadoutCommandPayload {
    pub fn encode(self) -> Result<[u8; 4], ProtocolError> {
        encode_talent_loadout_index(DELETE_LOADOUT_COMMAND_ID, self.index)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            index: decode_talent_loadout_index(DELETE_LOADOUT_COMMAND_ID, bytes)?,
        })
    }
}

impl ResurrectRespondCommandPayload {
    pub fn encode(self) -> [u8; 1] {
        [u8::from(self.accept)]
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(RESURRECT_RESPOND_COMMAND_ID, bytes)?;
        Ok(Self {
            accept: match bytes[0] {
                0 => false,
                1 => true,
                value => return Err(ProtocolError::InvalidBoolean(value)),
            },
        })
    }
}

impl PetAutoWaterJetCommandPayload {
    pub fn encode(self) -> Result<[u8; 1], ProtocolError> {
        let bytes = [u8::from(self.enabled)];
        validate_command_payload(PET_AUTO_WATER_JET_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(PET_AUTO_WATER_JET_COMMAND_ID, bytes)?;
        Ok(Self {
            enabled: match bytes[0] {
                0 => false,
                1 => true,
                value => return Err(ProtocolError::InvalidBoolean(value)),
            },
        })
    }
}

impl PetAutoTauntCommandPayload {
    pub fn encode(self) -> Result<[u8; 1], ProtocolError> {
        let bytes = [u8::from(self.enabled)];
        validate_command_payload(PET_AUTO_TAUNT_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(PET_AUTO_TAUNT_COMMAND_ID, bytes)?;
        Ok(Self {
            enabled: match bytes[0] {
                0 => false,
                1 => true,
                value => return Err(ProtocolError::InvalidBoolean(value)),
            },
        })
    }
}

pub fn validate_command_payload(command_id: u16, bytes: &[u8]) -> Result<(), ProtocolError> {
    let descriptor = command_payload_descriptor(command_id)
        .ok_or(ProtocolError::UnsupportedCommandPayload(command_id))?;
    validate_payload_length(descriptor, bytes.len())?;
    match descriptor.kind {
        CommandPayloadKind::Empty
        | CommandPayloadKind::TargetEntity
        | CommandPayloadKind::SlotIndex
        | CommandPayloadKind::I32Value
        | CommandPayloadKind::I32Pair => Ok(()),
        CommandPayloadKind::EmoteId => match bytes[0] {
            1..=13 => Ok(()),
            value => Err(ProtocolError::InvalidEmoteId(value)),
        },
        CommandPayloadKind::TargetEntityRaidGroup => validate_raid_subgroup(bytes[8]),
        CommandPayloadKind::U32Index => {
            if descriptor.id == SWITCH_LOADOUT_COMMAND_ID
                || descriptor.id == DELETE_LOADOUT_COMMAND_ID
            {
                validate_talent_loadout_index(u32::from_le_bytes(
                    bytes
                        .try_into()
                        .expect("validated u32-index command payload has four bytes"),
                ))
            } else {
                Ok(())
            }
        }
        CommandPayloadKind::TalentRowSelection => {
            let option_code = u16::from_le_bytes(
                bytes[1..]
                    .try_into()
                    .expect("validated talent-row command payload has two option bytes"),
            );
            validate_talent_row_selection(bytes[0], option_code)
        }
        CommandPayloadKind::TalentSpec => {
            let spec_code = u16::from_le_bytes(
                bytes
                    .try_into()
                    .expect("validated set-spec command payload has two bytes"),
            );
            validate_talent_spec(spec_code)
        }
        CommandPayloadKind::TalentAllocation => {
            let (spec_code, row_option_codes) = decode_talent_allocation_codes(bytes);
            validate_talent_allocation_codes(spec_code, &row_option_codes)
        }
        CommandPayloadKind::SaveLoadout => validate_save_loadout_payload(bytes),
        CommandPayloadKind::CosmeticSkin => validate_cosmetic_skin(bytes[0], bytes[1]),
        CommandPayloadKind::Boolean => match bytes[0] {
            0 | 1 => Ok(()),
            value => Err(ProtocolError::InvalidBoolean(value)),
        },
        CommandPayloadKind::Utf8Id => {
            let (_, consumed) = read_utf8_id(descriptor, bytes)?;
            reject_trailing(bytes, consumed)
        }
        CommandPayloadKind::ChatText => {
            let (value, consumed) = read_utf8_id(descriptor, bytes)?;
            reject_trailing(bytes, consumed)?;
            let actual = value.encode_utf16().count();
            if actual > descriptor.max_utf16_code_units {
                Err(ProtocolError::CollectionTooLarge {
                    context: payload_context(descriptor.id),
                    actual,
                    maximum: descriptor.max_utf16_code_units,
                })
            } else {
                Ok(())
            }
        }
        CommandPayloadKind::Utf8IdOptionalUtf8Id => {
            let (_, consumed) = read_utf8_id(descriptor, bytes)?;
            let (_, consumed) = read_optional_utf8_id_at(descriptor, bytes, consumed)?;
            reject_trailing(bytes, consumed)
        }
        CommandPayloadKind::Utf8IdPair => {
            let (_, consumed) = read_utf8_id(descriptor, bytes)?;
            let (_, consumed) = read_utf8_id_at(descriptor, bytes, consumed)?;
            reject_trailing(bytes, consumed)
        }
        CommandPayloadKind::Utf8IdOptionalTargetEntity => {
            let (_, consumed) = read_utf8_id(descriptor, bytes)?;
            let (_, consumed) = read_optional_target_id(bytes, consumed)?;
            reject_trailing(bytes, consumed)
        }
        CommandPayloadKind::Utf8IdF64Pair => {
            let (_, consumed) = read_utf8_id(descriptor, bytes)?;
            let (_, consumed) =
                read_finite_f64(bytes, consumed, payload_f64_context(descriptor.id, true))?;
            let (_, consumed) =
                read_finite_f64(bytes, consumed, payload_f64_context(descriptor.id, false))?;
            reject_trailing(bytes, consumed)
        }
        CommandPayloadKind::MarketSearch => validate_market_search_payload(bytes),
        CommandPayloadKind::ChallengeResponse => validate_challenge_response_payload(bytes),
        CommandPayloadKind::Utf8IdOptionalU32 => {
            let (_, consumed) = read_utf8_id(descriptor, bytes)?;
            let consumed = read_optional_u32(bytes, consumed)?.1;
            reject_trailing(bytes, consumed)
        }
        CommandPayloadKind::LockpickEngage => {
            validate_lockpick_ante(take(bytes, 8, 1, "LockpickEngageCommandPayload.ante")?[0])
        }
        CommandPayloadKind::LockpickAction => {
            let (_, consumed) = read_optional_utf8_id(descriptor, bytes)?;
            let action = LockpickAction::from_wire_code(
                take(bytes, consumed, 1, "LockpickActionCommandPayload.action")?[0],
            )?;
            let consumed = consumed + 1;
            let _ = action;
            reject_trailing(bytes, consumed)
        }
        CommandPayloadKind::OptionalUtf8Id => {
            let (_, consumed) = read_optional_utf8_id(descriptor, bytes)?;
            reject_trailing(bytes, consumed)
        }
        CommandPayloadKind::GuildEventCreate => validate_guild_event_create_payload(bytes),
        CommandPayloadKind::PartyLootMaster => validate_party_loot_master_payload(bytes),
        CommandPayloadKind::MasterLootAssignment => validate_master_loot_assignment_payload(bytes),
        CommandPayloadKind::PartyMarker => validate_party_marker_payload(bytes),
        CommandPayloadKind::PartyMarkerClear => validate_party_marker_clear_payload(bytes),
        CommandPayloadKind::DuelRequest => validate_duel_request_payload(bytes),
        CommandPayloadKind::ArenaQueueFormat => validate_arena_queue_payload(bytes),
        CommandPayloadKind::ArenaAugment => validate_arena_augment_payload(bytes),
        CommandPayloadKind::TradeRequest => validate_trade_request_payload(bytes),
        CommandPayloadKind::TradeOffer => validate_trade_offer_payload(bytes),
        CommandPayloadKind::ValeCupQueue => validate_vale_cup_queue_payload(bytes),
        CommandPayloadKind::ValeCupRole => validate_vale_cup_role_payload(bytes),
        CommandPayloadKind::ValeCupBet => validate_vale_cup_bet_payload(bytes),
        CommandPayloadKind::ValeCupBracket => validate_vale_cup_practice_payload(bytes),
        CommandPayloadKind::MailId => validate_mail_id_payload(bytes),
        CommandPayloadKind::MailSend => validate_mail_send_payload(bytes),
        CommandPayloadKind::BankSlotOptionalCount => {
            validate_bank_slot_optional_count_payload(bytes)
        }
        CommandPayloadKind::DungeonFinderRoles => validate_dungeon_finder_roles_payload(bytes),
        CommandPayloadKind::DungeonFinderActivities => {
            validate_dungeon_finder_activities_payload(bytes)
        }
        CommandPayloadKind::DungeonFinderListing => validate_dungeon_finder_listing_payload(bytes),
        CommandPayloadKind::DungeonFinderListingId => {
            validate_dungeon_finder_listing_id_payload(bytes)
        }
        CommandPayloadKind::DungeonFinderApplicationResponse => {
            validate_dungeon_finder_application_response_payload(bytes)
        }
        CommandPayloadKind::WorldObjectId => validate_world_object_id_payload(bytes),
        CommandPayloadKind::MarketListingId => validate_market_listing_id_payload(bytes),
        CommandPayloadKind::DelveRiteIntensity => validate_delve_rite_intensity_payload(bytes),
        CommandPayloadKind::DungeonDifficulty => validate_dungeon_difficulty_payload(bytes),
        CommandPayloadKind::LootRoll => validate_loot_roll_payload(bytes),
        CommandPayloadKind::EventSkin => validate_event_skin_payload(bytes),
        CommandPayloadKind::LinkedQuestAcceptance => {
            validate_linked_quest_acceptance_payload(bytes)
        }
        CommandPayloadKind::EquipmentItemOptionalSlot => validate_equip_item_payload(bytes),
        CommandPayloadKind::EquipmentSlot => validate_unequip_item_payload(bytes),
        CommandPayloadKind::TelemetryNumericFields => validate_telemetry_payload(bytes),
        CommandPayloadKind::TownFocusAllocation => validate_town_focus_payload(bytes),
        CommandPayloadKind::WeaponSkinChange => validate_weapon_skin_payload(bytes),
        CommandPayloadKind::CorpseHarvest => validate_corpse_harvest_payload(bytes),
    }
}

fn validate_payload_length(
    descriptor: &CommandPayloadDescriptor,
    actual: usize,
) -> Result<(), ProtocolError> {
    if let Some(expected) = descriptor.fixed_byte_length() {
        if actual != expected {
            return Err(ProtocolError::InvalidCommandPayloadLength {
                command_id: descriptor.id,
                actual,
                expected,
            });
        }
    } else if actual < descriptor.min_byte_length || actual > descriptor.max_byte_length {
        return Err(ProtocolError::InvalidCommandPayloadLengthRange {
            command_id: descriptor.id,
            actual,
            minimum: descriptor.min_byte_length,
            maximum: descriptor.max_byte_length,
        });
    }
    Ok(())
}

fn encode_talent_loadout_index(command_id: u16, index: u32) -> Result<[u8; 4], ProtocolError> {
    validate_talent_loadout_index(index)?;
    let bytes = index.to_le_bytes();
    validate_command_payload(command_id, &bytes)?;
    Ok(bytes)
}

fn decode_talent_loadout_index(command_id: u16, bytes: &[u8]) -> Result<u32, ProtocolError> {
    validate_command_payload(command_id, bytes)?;
    Ok(u32::from_le_bytes(bytes.try_into().expect(
        "validated talent-loadout command payload has four bytes",
    )))
}

fn validate_talent_loadout_index(index: u32) -> Result<(), ProtocolError> {
    if index >= TALENT_LOADOUT_LIMIT {
        return Err(ProtocolError::InvalidTalentLoadoutIndex(index));
    }
    Ok(())
}

fn validate_cosmetic_skin(catalog: u8, skin_index: u8) -> Result<(), ProtocolError> {
    match SkinCatalog::from_wire_code(catalog)? {
        SkinCatalog::Class if skin_index > CLASS_SKIN_INDEX_MAX => {
            Err(ProtocolError::InvalidClassSkinIndex(skin_index))
        }
        SkinCatalog::Class | SkinCatalog::Mech => Ok(()),
    }
}

fn encode_utf8_id(command_id: u16, value: &str) -> Result<Vec<u8>, ProtocolError> {
    let descriptor = descriptor(command_id)?;
    let mut bytes = Vec::with_capacity(LENGTH_PREFIX_BYTES + value.len());
    write_utf8_id(descriptor, value, &mut bytes)?;
    validate_command_payload(command_id, &bytes)?;
    Ok(bytes)
}

fn decode_utf8_id(command_id: u16, bytes: &[u8]) -> Result<String, ProtocolError> {
    validate_command_payload(command_id, bytes)?;
    let (value, _) = read_utf8_id(descriptor(command_id)?, bytes)?;
    Ok(value.to_owned())
}

fn encode_utf8_id_pair(
    command_id: u16,
    first: &str,
    second: &str,
) -> Result<Vec<u8>, ProtocolError> {
    let descriptor = descriptor(command_id)?;
    let mut bytes = Vec::with_capacity(LENGTH_PREFIX_BYTES * 2 + first.len() + second.len());
    write_utf8_id(descriptor, first, &mut bytes)?;
    write_utf8_id(descriptor, second, &mut bytes)?;
    validate_command_payload(command_id, &bytes)?;
    Ok(bytes)
}

fn decode_utf8_id_pair(command_id: u16, bytes: &[u8]) -> Result<(String, String), ProtocolError> {
    validate_command_payload(command_id, bytes)?;
    let descriptor = descriptor(command_id)?;
    let (first, consumed) = read_utf8_id(descriptor, bytes)?;
    let (second, _) = read_utf8_id_at(descriptor, bytes, consumed)?;
    Ok((first.to_owned(), second.to_owned()))
}

pub(crate) fn encode_utf8_id_f64_pair(
    command_id: u16,
    value: &str,
    first: f64,
    second: f64,
    first_context: &'static str,
    second_context: &'static str,
) -> Result<Vec<u8>, ProtocolError> {
    let descriptor = descriptor(command_id)?;
    let mut bytes = Vec::with_capacity(LENGTH_PREFIX_BYTES + value.len() + GROUND_POINT_BYTES);
    write_utf8_id(descriptor, value, &mut bytes)?;
    bytes.extend_from_slice(&canonical_finite_f64(first_context, first)?.to_le_bytes());
    bytes.extend_from_slice(&canonical_finite_f64(second_context, second)?.to_le_bytes());
    validate_command_payload(command_id, &bytes)?;
    Ok(bytes)
}

pub(crate) fn decode_utf8_id_f64_pair(
    command_id: u16,
    bytes: &[u8],
    first_context: &'static str,
    second_context: &'static str,
) -> Result<(String, f64, f64), ProtocolError> {
    validate_command_payload(command_id, bytes)?;
    let descriptor = descriptor(command_id)?;
    let (value, consumed) = read_utf8_id(descriptor, bytes)?;
    let (first, consumed) = read_finite_f64(bytes, consumed, first_context)?;
    let (second, consumed) = read_finite_f64(bytes, consumed, second_context)?;
    reject_trailing(bytes, consumed)?;
    Ok((value.to_owned(), first, second))
}

fn write_optional_utf8_id(
    descriptor: &CommandPayloadDescriptor,
    value: Option<&str>,
    bytes: &mut Vec<u8>,
) -> Result<(), ProtocolError> {
    match value {
        None => bytes.push(0),
        Some(value) => {
            validate_utf8_length(descriptor, value.len())?;
            bytes.push(1);
            bytes.extend_from_slice(
                &u32::try_from(value.len())
                    .expect("bounded command identifier length fits u32")
                    .to_le_bytes(),
            );
            bytes.extend_from_slice(value.as_bytes());
        }
    }
    Ok(())
}

fn write_utf8_id(
    descriptor: &CommandPayloadDescriptor,
    value: &str,
    bytes: &mut Vec<u8>,
) -> Result<(), ProtocolError> {
    validate_utf8_length(descriptor, value.len())?;
    bytes.extend_from_slice(
        &u32::try_from(value.len())
            .expect("bounded command identifier length fits u32")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn encode_utf8_id_optional_u32(
    command_id: u16,
    value: &str,
    optional: Option<u32>,
) -> Result<Vec<u8>, ProtocolError> {
    let descriptor = descriptor(command_id)?;
    validate_utf8_length(descriptor, value.len())?;
    let mut bytes = Vec::with_capacity(
        LENGTH_PREFIX_BYTES
            + value.len()
            + OPTIONAL_U32_PRESENCE_BYTES
            + optional.map(|_| 4).unwrap_or_default(),
    );
    bytes.extend_from_slice(
        &u32::try_from(value.len())
            .expect("bounded command identifier length fits u32")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(value.as_bytes());
    match optional {
        Some(value) => {
            bytes.push(1);
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        None => bytes.push(0),
    }
    validate_command_payload(command_id, &bytes)?;
    Ok(bytes)
}

fn decode_utf8_id_optional_u32(
    command_id: u16,
    bytes: &[u8],
) -> Result<(String, Option<u32>), ProtocolError> {
    validate_command_payload(command_id, bytes)?;
    let (value, consumed) = read_utf8_id(descriptor(command_id)?, bytes)?;
    let (optional, _) = read_optional_u32(bytes, consumed)?;
    Ok((value.to_owned(), optional))
}

fn descriptor(command_id: u16) -> Result<&'static CommandPayloadDescriptor, ProtocolError> {
    command_payload_descriptor(command_id)
        .ok_or(ProtocolError::UnsupportedCommandPayload(command_id))
}

fn validate_utf8_length(
    descriptor: &CommandPayloadDescriptor,
    actual: usize,
) -> Result<(), ProtocolError> {
    if actual > descriptor.max_utf8_bytes {
        return Err(ProtocolError::CollectionTooLarge {
            context: payload_context(descriptor.id),
            actual,
            maximum: descriptor.max_utf8_bytes,
        });
    }
    Ok(())
}

fn canonical_finite_f64(context: &'static str, value: f64) -> Result<f64, ProtocolError> {
    let value = require_finite(context, value)?;
    Ok(if value == 0.0 { 0.0 } else { value })
}

fn write_bounded_utf8(
    value: &str,
    maximum: usize,
    context: &'static str,
    bytes: &mut Vec<u8>,
) -> Result<(), ProtocolError> {
    if value.len() > maximum {
        return Err(ProtocolError::CollectionTooLarge {
            context,
            actual: value.len(),
            maximum,
        });
    }
    bytes.extend_from_slice(
        &u32::try_from(value.len())
            .expect("bounded guild-event string length fits u32")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn read_bounded_utf8_at<'a>(
    bytes: &'a [u8],
    offset: usize,
    maximum: usize,
    context: &'static str,
) -> Result<(&'a str, usize), ProtocolError> {
    let length = u32::from_le_bytes(
        take(bytes, offset, LENGTH_PREFIX_BYTES, context)?
            .try_into()
            .expect("guild-event string length prefix has four bytes"),
    ) as usize;
    if length > maximum {
        return Err(ProtocolError::CollectionTooLarge {
            context,
            actual: length,
            maximum,
        });
    }
    let value_offset =
        offset
            .checked_add(LENGTH_PREFIX_BYTES)
            .ok_or(ProtocolError::CollectionTooLarge {
                context,
                actual: usize::MAX,
                maximum,
            })?;
    let end = value_offset
        .checked_add(length)
        .ok_or(ProtocolError::CollectionTooLarge {
            context,
            actual: usize::MAX,
            maximum,
        })?;
    let value = take(bytes, value_offset, length, context)?;
    let value = std::str::from_utf8(value).map_err(|_| ProtocolError::InvalidUtf8 { context })?;
    Ok((value, end))
}

fn read_optional_guild_event_hour(
    bytes: &[u8],
    offset: usize,
) -> Result<(Option<f64>, usize), ProtocolError> {
    match take(bytes, offset, 1, "GuildEventCreateCommandPayload.hour")?[0] {
        0 => Ok((None, offset + 1)),
        1 => {
            let (hour, consumed) =
                read_finite_f64(bytes, offset + 1, "GuildEventCreateCommandPayload.hour")?;
            Ok((Some(hour), consumed))
        }
        invalid => Err(ProtocolError::InvalidBoolean(invalid)),
    }
}

fn validate_guild_event_create_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let (_, consumed) = read_bounded_utf8_at(
        bytes,
        0,
        GUILD_EVENT_DAY_MAX_UTF8_BYTES,
        "GuildEventCreateCommandPayload.day",
    )?;
    let (_, consumed) = read_optional_guild_event_hour(bytes, consumed)?;
    let (_, consumed) = read_bounded_utf8_at(
        bytes,
        consumed,
        GUILD_EVENT_TITLE_MAX_UTF8_BYTES,
        "GuildEventCreateCommandPayload.title",
    )?;
    let (_, consumed) = read_bounded_utf8_at(
        bytes,
        consumed,
        GUILD_EVENT_NOTE_MAX_UTF8_BYTES,
        "GuildEventCreateCommandPayload.note",
    )?;
    reject_trailing(bytes, consumed)
}

fn read_utf8_id<'a>(
    descriptor: &CommandPayloadDescriptor,
    bytes: &'a [u8],
) -> Result<(&'a str, usize), ProtocolError> {
    read_utf8_id_at(descriptor, bytes, 0)
}

fn read_finite_f64(
    bytes: &[u8],
    offset: usize,
    context: &'static str,
) -> Result<(f64, usize), ProtocolError> {
    let value = f64::from_le_bytes(
        take(bytes, offset, 8, context)?
            .try_into()
            .expect("ground-target coordinate has eight bytes"),
    );
    Ok((canonical_finite_f64(context, value)?, offset + 8))
}

fn read_utf8_id_at<'a>(
    descriptor: &CommandPayloadDescriptor,
    bytes: &'a [u8],
    offset: usize,
) -> Result<(&'a str, usize), ProtocolError> {
    let context = payload_context(descriptor.id);
    let length_bytes = take(bytes, offset, LENGTH_PREFIX_BYTES, context)?;
    let length = u32::from_le_bytes(
        length_bytes
            .try_into()
            .expect("command identifier length prefix has four bytes"),
    ) as usize;
    validate_utf8_length(descriptor, length)?;
    let value_offset =
        offset
            .checked_add(LENGTH_PREFIX_BYTES)
            .ok_or(ProtocolError::CollectionTooLarge {
                context,
                actual: usize::MAX,
                maximum: descriptor.max_utf8_bytes,
            })?;
    let end = value_offset
        .checked_add(length)
        .ok_or(ProtocolError::CollectionTooLarge {
            context,
            actual: usize::MAX,
            maximum: descriptor.max_utf8_bytes,
        })?;
    let value = take(bytes, value_offset, length, context)?;
    let value = std::str::from_utf8(value).map_err(|_| ProtocolError::InvalidUtf8 { context })?;
    Ok((value, end))
}

fn read_optional_utf8_id<'a>(
    descriptor: &CommandPayloadDescriptor,
    bytes: &'a [u8],
) -> Result<(Option<&'a str>, usize), ProtocolError> {
    read_optional_utf8_id_at(descriptor, bytes, 0)
}

fn read_optional_utf8_id_at<'a>(
    descriptor: &CommandPayloadDescriptor,
    bytes: &'a [u8],
    offset: usize,
) -> Result<(Option<&'a str>, usize), ProtocolError> {
    let context = payload_context(descriptor.id);
    match take(bytes, offset, 1, context)?[0] {
        0 => Ok((None, offset + 1)),
        1 => {
            let (value, consumed) = read_utf8_id_at(descriptor, bytes, offset + 1)?;
            Ok((Some(value), consumed))
        }
        invalid => Err(ProtocolError::InvalidBoolean(invalid)),
    }
}

fn read_optional_u32(bytes: &[u8], offset: usize) -> Result<(Option<u32>, usize), ProtocolError> {
    let context = "command payload optional u32";
    let presence = take(bytes, offset, OPTIONAL_U32_PRESENCE_BYTES, context)?[0];
    match presence {
        0 => Ok((None, offset + OPTIONAL_U32_PRESENCE_BYTES)),
        1 => {
            let value_offset = offset + OPTIONAL_U32_PRESENCE_BYTES;
            let value = u32::from_le_bytes(
                take(bytes, value_offset, 4, context)?
                    .try_into()
                    .expect("optional u32 payload has four bytes"),
            );
            Ok((Some(value), value_offset + 4))
        }
        invalid => Err(ProtocolError::InvalidBoolean(invalid)),
    }
}

fn read_optional_target_id(
    bytes: &[u8],
    offset: usize,
) -> Result<(Option<u64>, usize), ProtocolError> {
    let context = "CastAbilityCommandPayload.target_id";
    let presence = take(bytes, offset, OPTIONAL_TARGET_PRESENCE_BYTES, context)?[0];
    match presence {
        0 => Ok((None, offset + OPTIONAL_TARGET_PRESENCE_BYTES)),
        1 => {
            let value_offset = offset + OPTIONAL_TARGET_PRESENCE_BYTES;
            let target_id = u64::from_le_bytes(
                take(bytes, value_offset, 8, context)?
                    .try_into()
                    .expect("optional cast target payload has eight bytes"),
            );
            Ok((Some(target_id), value_offset + 8))
        }
        invalid => Err(ProtocolError::InvalidBoolean(invalid)),
    }
}

fn take<'a>(
    bytes: &'a [u8],
    offset: usize,
    length: usize,
    context: &'static str,
) -> Result<&'a [u8], ProtocolError> {
    let remaining = bytes.len().saturating_sub(offset);
    if remaining < length {
        return Err(ProtocolError::TruncatedPayload {
            context,
            needed: length,
            remaining,
        });
    }
    Ok(&bytes[offset..offset + length])
}

fn reject_trailing(bytes: &[u8], consumed: usize) -> Result<(), ProtocolError> {
    let remaining = bytes.len().saturating_sub(consumed);
    if remaining == 0 {
        Ok(())
    } else {
        Err(ProtocolError::TrailingPayload { remaining })
    }
}

fn payload_context(command_id: u16) -> &'static str {
    match command_id {
        CAST_AT_COMMAND_ID => "CastAtCommandPayload.ability_id",
        CAST_COMMAND_ID => "CastAbilityCommandPayload.ability_id",
        CANCEL_AURA_COMMAND_ID => "CancelAuraCommandPayload.aura_id",
        ACCEPT_QUEST_COMMAND_ID => "AcceptQuestCommandPayload.quest_id",
        TURN_IN_QUEST_COMMAND_ID => "TurnInQuestCommandPayload.quest_id",
        ABANDON_QUEST_COMMAND_ID => "AbandonQuestCommandPayload.quest_id",
        USE_ITEM_COMMAND_ID => "UseItemCommandPayload.item_id",
        HARVEST_NODE_COMMAND_ID => "HarvestNodeCommandPayload.node_id",
        CRAFT_ITEM_COMMAND_ID => "CraftItemCommandPayload.recipe_id",
        HEROIC_BUY_COMMAND_ID => "HeroicBuyCommandPayload.item_id",
        DELVE_BUY_COMMAND_ID => "DelveBuyCommandPayload.id",
        MARKET_LIST_COMMAND_ID => "MarketListCommandPayload.item_id",
        COMPANION_UPGRADE_COMMAND_ID => "CompanionUpgradeCommandPayload.companion_id",
        UNEQUIP_MECH_CHROMA_COMMAND_ID => "UnequipMechChromaCommandPayload.chroma_id",
        CHAT_COMMAND_ID => "ChatCommandPayload.text",
        DEED_SET_TITLE_COMMAND_ID => "DeedSetTitleCommandPayload.deed_id",
        DISCARD_ITEM_COMMAND_ID => "DiscardItemCommandPayload.item_id",
        EQUIP_BAG_COMMAND_ID => "EquipBagCommandPayload.item_id",
        LOCKPICK_ACTION_COMMAND_ID => "LockpickActionCommandPayload.session_id",
        LOCKPICK_ABORT_COMMAND_ID => "LockpickAbortCommandPayload.session_id",
        _ => "command payload identifier",
    }
}

fn payload_f64_context(command_id: u16, first: bool) -> &'static str {
    match (command_id, first) {
        (CAST_AT_COMMAND_ID, true) => "CastAtCommandPayload.x",
        (CAST_AT_COMMAND_ID, false) => "CastAtCommandPayload.z",
        (MARKET_LIST_COMMAND_ID, true) => "MarketListCommandPayload.count",
        (MARKET_LIST_COMMAND_ID, false) => "MarketListCommandPayload.price",
        (_, true) => "command payload first number",
        (_, false) => "command payload second number",
    }
}

fn validate_lockpick_ante(ante: u8) -> Result<(), ProtocolError> {
    match ante {
        1..=3 => Ok(()),
        _ => Err(ProtocolError::InvalidLockpickAnte(ante)),
    }
}

fn validate_raid_subgroup(subgroup: u8) -> Result<(), ProtocolError> {
    match subgroup {
        1 | 2 => Ok(()),
        invalid => Err(ProtocolError::InvalidRaidSubgroup(invalid)),
    }
}

fn validate_talent_row_selection(level: u8, option_code: u16) -> Result<(), ProtocolError> {
    if !matches!(level, 5 | 8 | 11 | 14 | 17 | 20) {
        return Err(ProtocolError::InvalidTalentRowLevel(level));
    }
    if option_code != 0 && talent_option_id(option_code).is_none() {
        return Err(ProtocolError::InvalidTalentOptionCode(option_code));
    }
    Ok(())
}

fn validate_talent_spec(spec_code: u16) -> Result<(), ProtocolError> {
    if spec_code != 0 && talent_spec_id(spec_code).is_none() {
        return Err(ProtocolError::InvalidTalentSpecCode(spec_code));
    }
    Ok(())
}

fn validate_talent_allocation_codes(
    spec_code: u16,
    row_option_codes: &[u16; TALENT_ALLOCATION_ROW_COUNT],
) -> Result<(), ProtocolError> {
    validate_talent_spec(spec_code)?;
    for option_code in row_option_codes {
        if *option_code != 0 && talent_option_id(*option_code).is_none() {
            return Err(ProtocolError::InvalidTalentOptionCode(*option_code));
        }
    }
    Ok(())
}

fn decode_talent_allocation_codes(bytes: &[u8]) -> (u16, [u16; TALENT_ALLOCATION_ROW_COUNT]) {
    let spec_code = u16::from_le_bytes(
        bytes[..2]
            .try_into()
            .expect("validated talent-allocation command payload has two spec bytes"),
    );
    let mut row_option_codes = [0; TALENT_ALLOCATION_ROW_COUNT];
    for (index, option_code) in row_option_codes.iter_mut().enumerate() {
        let offset = 2 + index * 2;
        *option_code = u16::from_le_bytes(
            bytes[offset..offset + 2]
                .try_into()
                .expect("validated talent-allocation command payload has two row bytes"),
        );
    }
    (spec_code, row_option_codes)
}

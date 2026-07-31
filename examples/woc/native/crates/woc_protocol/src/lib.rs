#![forbid(unsafe_code)]

mod bank_payload;
mod codec;
mod command_payload;
mod command_value;
mod contracts;
mod delve_rite_payload;
mod digest;
mod duel_arena_payload;
mod dungeon_difficulty_payload;
mod dungeon_finder_payload;
mod equipment_payload;
mod error;
mod event_skin_payload;
mod generated;
mod generated_command_payloads;
mod generated_commands;
mod generated_talent_selection_catalog;
mod linked_quest_payload;
mod loot_roll_payload;
mod mail_payload;
mod market_payload;
mod master_loot_assignment_payload;
mod movement_input;
mod party_payload;
mod payload;
mod trade_payload;
mod vale_cup_payload;
mod world_object_payload;

pub use bank_payload::{BankAction, BankSlotCommandPayload};
pub use codec::{
    canonical_pairs, decode_frame, encode_frame, require_finite, DecodeLimits, Frame,
    FRAME_HEADER_BYTES,
};
pub use command_payload::{
    validate_command_payload, AbandonQuestCommandPayload, AcceptQuestCommandPayload,
    ApplyTalentsCommandPayload, BuyItemCommandPayload, BuybackItemCommandPayload,
    CancelAuraCommandPayload, CardPlayCommandPayload, CastAbilityCommandPayload,
    CastAtCommandPayload, CastSlotCommandPayload, ChangeSkinCommandPayload,
    DeleteLoadoutCommandPayload, DiscardItemCommandPayload, EquipBagCommandPayload,
    GroundTargetPoint, GuildEventCreateCommandPayload, GuildEventRemoveCommandPayload,
    LockpickAbortCommandPayload, LockpickAction, LockpickActionCommandPayload,
    LockpickEngageCommandPayload, PartyMoveRaidCommandPayload, PetAutoTauntCommandPayload,
    PetAutoWaterJetCommandPayload, PetFeedCommandPayload, PetModeCommandPayload,
    PetRenameCommandPayload, ReleaseEmpoweredCommandPayload, ResurrectRespondCommandPayload,
    SelectTalentRowCommandPayload, SellItemCommandPayload, SetSpecCommandPayload, SkinCatalog,
    SocialNameCommandPayload, SwitchLoadoutCommandPayload, TargetCommandPayload,
    TurnInQuestCommandPayload, UnequipBagCommandPayload, UseItemCommandPayload,
};
pub use command_value::{
    decode_command_value, encode_command_value, CommandValue, CommandValueLimits,
};
pub use contracts::{
    Command, EntityRef, Event, FixedTickInput, MessageKind, MovementFrame, NetworkEnvelope,
    OfflineSessionBootstrap, RlActionBatch, RlObservationBatch, SaveState, WocReferenceIdentity,
    WorldSnapshot,
};
pub use delve_rite_payload::{DelveRiteChoosePayload, DelveRiteIntensity};
pub use digest::{event_stream_digest, fnv1a_bytes, FNV1A_OFFSET};
pub use duel_arena_payload::{
    ArenaAugmentCommandPayload, ArenaFormat, ArenaQueueCommandPayload, DuelRequestCommandPayload,
};
pub use dungeon_difficulty_payload::{DungeonDifficulty, DungeonDifficultyPayload};
pub use dungeon_finder_payload::{
    DungeonFinderActivitiesPayload, DungeonFinderApplicationResponsePayload,
    DungeonFinderListingIdPayload, DungeonFinderListingPayload, DungeonFinderListingTag,
    DungeonFinderRole, DungeonFinderRolesPayload,
};
pub use equipment_payload::{EquipItemPayload, EquipmentSlot, UnequipItemPayload};
pub use error::ProtocolError;
pub use event_skin_payload::EventSkinPayload;
pub use generated::*;
pub use generated_command_payloads::*;
pub use generated_commands::*;
pub use generated_talent_selection_catalog::*;
pub use linked_quest_payload::LinkedQuestAcceptancePayload;
pub use loot_roll_payload::{LootRollChoice, LootRollPayload};
pub use mail_payload::{MailAction, MailIdCommandPayload};
pub use market_payload::{MarketAction, MarketListingIdPayload};
pub use master_loot_assignment_payload::{
    MasterLootAssignmentPayload, MAX_TARGET_PIDS as MAX_MASTER_LOOT_ASSIGNMENT_TARGETS,
};
pub use movement_input::{
    MovementFrameBatch, MovementFrameDisposition, MovementInputError, MovementInputFlags,
    MovementInputRelay, RetainedMovementInput, MAX_MOVEMENT_FACING_MAGNITUDE,
    MAX_MOVEMENT_FRAMES_PER_TICK, MOVEMENT_INPUT_STALE_AFTER_TICKS,
};
pub use party_payload::{
    MasterLootThreshold, PartyLootMasterCommandPayload, PartyMarkerClearCommandPayload,
    PartyMarkerCommandPayload, ReadyCheckRespondCommandPayload,
};
pub use trade_payload::TradeRequestCommandPayload;
pub use vale_cup_payload::{
    ValeCupBetCommandPayload, ValeCupBracket, ValeCupNation, ValeCupPracticeCommandPayload,
    ValeCupQueueCommandPayload, ValeCupRole, ValeCupRoleCommandPayload, ValeCupSide,
};
pub use world_object_payload::{WorldObjectAction, WorldObjectIdPayload};

pub const REFERENCE_COMMIT: &str = "5ef9f7cb21cd8875b6d2c49701015dfcd78de35a";
pub const SIMULATION_HZ: u32 = 20;
pub const PRESENTATION_HZ: u32 = 60;
pub const WORLD_STATE_FORMAT: &str = "WOS71";
pub const WORLD_STATE_SCHEMA_VERSION: u16 = 71;
pub const OFFLINE_SESSION_BOOTSTRAP_VERSION: u16 = 1;
pub const STANDARD_OFFLINE_WORLD_SEED: u32 = 20_061;

pub const REFERENCE_IDENTITY: WocReferenceIdentity = WocReferenceIdentity {
    source_commit: REFERENCE_COMMIT,
    contract_schema_fingerprint: SCHEMA_FINGERPRINT_HEX,
    command_catalog_sha256: COMMAND_CATALOG_SHA256,
    command_payload_schema_sha256: COMMAND_PAYLOAD_SCHEMA_SHA256,
    world_state_format: WORLD_STATE_FORMAT,
    world_state_schema_version: WORLD_STATE_SCHEMA_VERSION,
    simulation_hz: SIMULATION_HZ,
    presentation_hz: PRESENTATION_HZ,
};

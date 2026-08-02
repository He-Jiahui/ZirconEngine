#![forbid(unsafe_code)]

mod bank_payload;
mod challenge_payload;
mod codec;
mod command_payload;
mod command_value;
mod contracts;
mod corpse_harvest_contract;
mod corpse_harvest_payload;
mod delve_rite_payload;
mod digest;
mod duel_arena_payload;
mod dungeon_difficulty_payload;
mod dungeon_finder_payload;
mod emote_payload;
mod equipment_payload;
mod error;
mod event_skin_payload;
mod generated;
mod generated_command_payloads;
mod generated_commands;
mod generated_talent_selection_catalog;
mod inventory_move_payload;
mod linked_quest_payload;
mod loot_roll_payload;
mod mail_payload;
mod market_payload;
mod master_loot_assignment_payload;
mod movement_input;
mod party_payload;
mod payload;
mod save_loadout_payload;
mod telemetry_payload;
mod town_focus_payload;
mod trade_payload;
mod vale_cup_payload;
mod weapon_skin_contract;
mod weapon_skin_payload;
mod world_object_payload;

pub use bank_payload::{BankAction, BankSlotCommandPayload};
pub use challenge_payload::ChallengeResponseCommandPayload;
pub use codec::{
    canonical_pairs, decode_frame, encode_frame, require_finite, DecodeLimits, Frame,
    FRAME_HEADER_BYTES,
};
pub use command_payload::{
    validate_command_payload, AbandonQuestCommandPayload, AcceptQuestCommandPayload,
    ApplyTalentsCommandPayload, BuyItemCommandPayload, BuybackItemCommandPayload,
    CancelAuraCommandPayload, CardPlayCommandPayload, CastAbilityCommandPayload,
    CastAtCommandPayload, CastSlotCommandPayload, ChangeSkinCommandPayload, ChatCommandPayload,
    CompanionUpgradeCommandPayload, CraftItemCommandPayload, DeedSetTitleCommandPayload,
    DeleteLoadoutCommandPayload, DelveBuyCommandPayload, DiscardItemCommandPayload,
    EnterDelveCommandPayload, EnterDungeonCommandPayload, EquipBagCommandPayload,
    GroundTargetPoint, GuildEventCreateCommandPayload, GuildEventRemoveCommandPayload,
    HarvestNodeCommandPayload, HeroicBuyCommandPayload, LockpickAbortCommandPayload,
    LockpickAction, LockpickActionCommandPayload, LockpickEngageCommandPayload,
    PartyMoveRaidCommandPayload, PetAutoTauntCommandPayload, PetAutoWaterJetCommandPayload,
    PetFeedCommandPayload, PetModeCommandPayload, PetRenameCommandPayload,
    ReleaseEmpoweredCommandPayload, ResurrectRespondCommandPayload, SelectTalentRowCommandPayload,
    SellItemCommandPayload, SetSpecCommandPayload, SkinCatalog, SocialNameCommandPayload,
    SwitchLoadoutCommandPayload, TargetCommandPayload, TurnInQuestCommandPayload,
    UnequipBagCommandPayload, UnequipMechChromaCommandPayload, UseItemCommandPayload,
};
pub use command_value::{
    decode_command_value, encode_command_value, CommandValue, CommandValueLimits,
};
pub use contracts::{
    Command, EntityRef, Event, FixedTickInput, MessageKind, MovementFrame, NetworkEnvelope,
    OfflineSessionBootstrap, OfflineWeaponSkinAccount, RlActionBatch, RlObservationBatch,
    SaveState, WocReferenceIdentity, WorldSnapshot, OFFLINE_WEAPON_SKIN_COUNT,
    OFFLINE_WEAPON_SKIN_TYPE_COUNT,
};
pub use corpse_harvest_contract::corpse_harvest_component_code;
pub use corpse_harvest_payload::HarvestCorpseCommandPayload;
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
pub use emote_payload::{EmoteCommandPayload, EmoteId};
pub use equipment_payload::{EquipItemPayload, EquipmentSlot, UnequipItemPayload};
pub use error::ProtocolError;
pub use event_skin_payload::EventSkinPayload;
pub use generated::*;
pub use generated_command_payloads::*;
pub use generated_commands::*;
pub use generated_talent_selection_catalog::*;
pub use inventory_move_payload::InventoryMovePayload;
pub use linked_quest_payload::LinkedQuestAcceptancePayload;
pub use loot_roll_payload::{LootRollChoice, LootRollPayload};
pub use mail_payload::{
    MailAction, MailIdCommandPayload, MailSendAttachment, MailSendCommandPayload,
    MAIL_SEND_MAX_ATTACHMENTS, MAIL_SEND_MAX_PAYLOAD_BYTES,
};
pub use market_payload::{
    MarketAction, MarketListCommandPayload, MarketListingIdPayload, MarketSearchCommandPayload,
};
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
pub use save_loadout_payload::{
    SaveLoadoutCommandPayload, MAX_LOADOUT_ABILITY_ID_UTF8_BYTES, MAX_LOADOUT_ACTION_BAR_SLOTS,
    MAX_LOADOUT_NAME_UTF16_CODE_UNITS, MAX_LOADOUT_NAME_UTF8_BYTES,
};
pub use telemetry_payload::TelemetryPayload;
pub use town_focus_payload::{TownFocusAllocationEntry, TownFocusCommandPayload};
pub use trade_payload::{TradeOfferCommandPayload, TradeOfferItem, TradeRequestCommandPayload};
pub use vale_cup_payload::{
    ValeCupBetCommandPayload, ValeCupBracket, ValeCupNation, ValeCupPracticeCommandPayload,
    ValeCupQueueCommandPayload, ValeCupRole, ValeCupRoleCommandPayload, ValeCupSide,
};
pub use weapon_skin_payload::{ChangeWeaponSkinCommandPayload, WeaponSkinChange, WeaponSkinType};
pub use world_object_payload::{WorldObjectAction, WorldObjectIdPayload};

pub const REFERENCE_COMMIT: &str = "5ef9f7cb21cd8875b6d2c49701015dfcd78de35a";
pub const SIMULATION_HZ: u32 = 20;
pub const PRESENTATION_HZ: u32 = 60;
pub const WORLD_STATE_FORMAT: &str = "WOS83";
pub const WORLD_STATE_SCHEMA_VERSION: u16 = 83;
pub const OFFLINE_SESSION_BOOTSTRAP_VERSION: u16 = 2;
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

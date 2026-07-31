use woc_protocol::{
    command_descriptor, command_payload_descriptor, talent_option_code,
    talent_option_matches_class_row, talent_spec_code, validate_command_payload,
    AbandonQuestCommandPayload, AcceptQuestCommandPayload, ApplyTalentsCommandPayload,
    ArenaAugmentCommandPayload, ArenaFormat, ArenaQueueCommandPayload, BankAction,
    BankSlotCommandPayload, CancelAuraCommandPayload, CardPlayCommandPayload,
    CastAbilityCommandPayload, CastAtCommandPayload, CastSlotCommandPayload,
    ChangeSkinCommandPayload, Command, CommandKind, CommandPayloadKind,
    DeleteLoadoutCommandPayload, DelveRiteChoosePayload, DelveRiteIntensity,
    DiscardItemCommandPayload, DuelRequestCommandPayload, DungeonDifficulty,
    DungeonDifficultyPayload, DungeonFinderActivitiesPayload,
    DungeonFinderApplicationResponsePayload, DungeonFinderListingIdPayload,
    DungeonFinderListingPayload, DungeonFinderListingTag, DungeonFinderRole,
    DungeonFinderRolesPayload, EntityRef, EquipBagCommandPayload, EquipItemPayload, EquipmentSlot,
    EventSkinPayload, GroundTargetPoint, GuildEventCreateCommandPayload,
    GuildEventRemoveCommandPayload, LinkedQuestAcceptancePayload, LockpickAbortCommandPayload,
    LockpickAction, LockpickActionCommandPayload, LockpickEngageCommandPayload, LootRollChoice,
    LootRollPayload, MailAction, MailIdCommandPayload, MarketAction, MarketListingIdPayload,
    MasterLootAssignmentPayload, MasterLootThreshold, PartyLootMasterCommandPayload,
    PartyMarkerClearCommandPayload, PartyMarkerCommandPayload, PartyMoveRaidCommandPayload,
    PetAutoTauntCommandPayload, PetAutoWaterJetCommandPayload, PetFeedCommandPayload,
    PetModeCommandPayload, PetRenameCommandPayload, ProtocolError, ReadyCheckRespondCommandPayload,
    ReleaseEmpoweredCommandPayload, ResurrectRespondCommandPayload, SelectTalentRowCommandPayload,
    SetSpecCommandPayload, SkinCatalog, SocialNameCommandPayload, SwitchLoadoutCommandPayload,
    TargetCommandPayload, TradeRequestCommandPayload, TurnInQuestCommandPayload,
    UnequipBagCommandPayload, UnequipItemPayload, UseItemCommandPayload, ValeCupBetCommandPayload,
    ValeCupBracket, ValeCupNation, ValeCupPracticeCommandPayload, ValeCupQueueCommandPayload,
    ValeCupRole, ValeCupRoleCommandPayload, ValeCupSide, WorldObjectAction, WorldObjectIdPayload,
    ABANDON_QUEST_COMMAND_ID, ACCEPT_QUEST_COMMAND_ID, APPLY_TALENTS_COMMAND_ID,
    ARENA_AUGMENT_COMMAND_ID, ARENA_QUEUE_COMMAND_ID, ATTACK_COMMAND_ID, AUTO_LOOT_COMMAND_ID,
    BANK_DEPOSIT_COMMAND_ID, BANK_WITHDRAW_COMMAND_ID, BLOCK_ADD_COMMAND_ID,
    BLOCK_REMOVE_COMMAND_ID, CANCEL_AURA_COMMAND_ID, CARD_FORFEIT_COMMAND_ID, CARD_PLAY_COMMAND_ID,
    CARD_QUEUE_JOIN_COMMAND_ID, CARD_QUEUE_LEAVE_COMMAND_ID, CAST_AT_COMMAND_ID, CAST_COMMAND_ID,
    CAST_SLOT_COMMAND_ID, CHANGE_SKIN_COMMAND_ID, CLAIM_EVENT_SKIN_COMMAND_ID,
    COLLECT_DELVE_CHEST_LOOT_COMMAND_ID, COMMAND_PAYLOAD_CATALOG, DELETE_LOADOUT_COMMAND_ID,
    DELVE_INTERACT_COMMAND_ID, DELVE_RITE_CHOOSE_COMMAND_ID, DISCARD_ITEM_COMMAND_ID,
    DUEL_REQUEST_COMMAND_ID, DUNGEON_FINDER_APPLICATION_RESPONSE_COMMAND_ID,
    DUNGEON_FINDER_APPLY_COMMAND_ID, DUNGEON_FINDER_LIST_CREATE_COMMAND_ID,
    DUNGEON_FINDER_PROPOSAL_COMMAND_ID, DUNGEON_FINDER_QUEUE_COMMAND_ID,
    DUNGEON_FINDER_ROLES_COMMAND_ID, EQUIP_BAG_COMMAND_ID, EQUIP_ITEM_COMMAND_ID,
    FRIEND_ADD_COMMAND_ID, FRIEND_REMOVE_COMMAND_ID, GUILD_CREATE_COMMAND_ID,
    GUILD_DEMOTE_COMMAND_ID, GUILD_EVENT_CREATE_COMMAND_ID, GUILD_EVENT_REMOVE_COMMAND_ID,
    GUILD_INVITE_COMMAND_ID, GUILD_KICK_COMMAND_ID, GUILD_PROMOTE_COMMAND_ID,
    GUILD_TRANSFER_COMMAND_ID, IGNORE_ADD_COMMAND_ID, IGNORE_REMOVE_COMMAND_ID,
    INTERACT_COMMAND_ID, LINKED_QUEST_ACCEPT_COMMAND_ID, LOCKPICK_ABORT_COMMAND_ID,
    LOCKPICK_ACTION_COMMAND_ID, LOCKPICK_ENGAGE_COMMAND_ID, LOOT_COMMAND_ID, LOOT_ROLL_COMMAND_ID,
    MAIL_DELETE_COMMAND_ID, MAIL_READ_COMMAND_ID, MAIL_TAKE_COMMAND_ID, MARKET_BUY_COMMAND_ID,
    MARKET_CANCEL_COMMAND_ID, MASTER_ASSIGN_COMMAND_ID, PARTY_CLEAR_MARKER_COMMAND_ID,
    PARTY_INVITE_COMMAND_ID, PARTY_KICK_COMMAND_ID, PARTY_MOVE_RAID_COMMAND_ID,
    PARTY_PROMOTE_COMMAND_ID, PARTY_READY_RESPOND_COMMAND_ID, PARTY_SET_LOOT_MASTER_COMMAND_ID,
    PARTY_SET_MARKER_COMMAND_ID, PET_ABANDON_COMMAND_ID, PET_ATTACK_COMMAND_ID,
    PET_AUTO_TAUNT_COMMAND_ID, PET_AUTO_WATER_JET_COMMAND_ID, PET_FEED_COMMAND_ID,
    PET_HEAL_COMMAND_ID, PET_MODE_COMMAND_ID, PET_RENAME_COMMAND_ID, PET_REVIVE_COMMAND_ID,
    PET_TAUNT_COMMAND_ID, PET_WATER_JET_COMMAND_ID, PICKUP_COMMAND_ID,
    RELEASE_EMPOWERED_COMMAND_ID, RESURRECT_RESPOND_COMMAND_ID, SELECT_TALENT_ROW_COMMAND_ID,
    SET_DUNGEON_DIFFICULTY_COMMAND_ID, SET_SPEC_COMMAND_ID, STOP_ATTACK_COMMAND_ID,
    SWITCH_LOADOUT_COMMAND_ID, TARGET_COMMAND_ID, TARGET_NEAREST_FRIENDLY_COMMAND_ID,
    TRADE_REQUEST_COMMAND_ID, TURN_IN_QUEST_COMMAND_ID, UNEQUIP_BAG_COMMAND_ID,
    UNEQUIP_ITEM_COMMAND_ID, USE_ITEM_COMMAND_ID, VALE_CUP_BET_COMMAND_ID,
    VALE_CUP_PRACTICE_COMMAND_ID, VALE_CUP_QUEUE_COMMAND_ID, VALE_CUP_ROLE_COMMAND_ID,
    WEAPON_STOW_COMMAND_ID,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientInputDevice {
    KeyboardMouse,
    Gamepad,
    Touch,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClientGameplayIntent {
    CastAbilityAt {
        ability_id: String,
        aim: GroundTargetPoint,
    },
    CastAbility {
        ability_id: String,
        target_id: Option<u64>,
    },
    CancelAura {
        aura_id: String,
    },
    ChangeSkin {
        catalog: SkinCatalog,
        skin_index: u8,
    },
    ClaimEventSkin {
        skin: f64,
    },
    CastSlot {
        slot: i32,
    },
    SetTarget {
        target_id: Option<u64>,
    },
    CycleTarget {
        friendly: bool,
    },
    TargetNearestFriendly,
    Interact,
    SetAttacking {
        attacking: bool,
    },
    ToggleWeaponStow,
    JoinCardDuelQueue,
    LeaveCardDuelQueue,
    PlayCardInDuel {
        card_value: i32,
    },
    ForfeitCardDuel,
    ReleaseEmpoweredAbility {
        ability_id: String,
    },
    PetWaterJet,
    SetPetAutoWaterJet {
        enabled: bool,
    },
    AbandonPet,
    RenamePet {
        name: String,
    },
    RevivePet,
    PetAttack,
    PetTaunt,
    SetPetAutoTaunt {
        enabled: bool,
    },
    FeedPet {
        item_id: String,
    },
    HealPet,
    SetPetMode {
        mode: String,
    },
    AddFriend {
        name: String,
    },
    RemoveFriend {
        name: String,
    },
    AddBlock {
        name: String,
    },
    RemoveBlock {
        name: String,
    },
    CreateGuild {
        name: String,
    },
    InviteToGuild {
        name: String,
    },
    AcceptGuildInvite,
    DeclineGuildInvite,
    LeaveGuild,
    KickGuildMember {
        name: String,
    },
    PromoteGuildMember {
        name: String,
    },
    DemoteGuildMember {
        name: String,
    },
    TransferGuildLeadership {
        name: String,
    },
    DisbandGuild,
    CreateGuildEvent {
        day: String,
        hour: Option<f64>,
        title: String,
        note: String,
    },
    RemoveGuildEvent {
        event_id: u32,
    },
    AddIgnore {
        name: String,
    },
    RemoveIgnore {
        name: String,
    },
    AcceptQuest {
        quest_id: String,
    },
    AcceptLinkedQuest {
        quest_id: String,
        sharer_pid: f64,
    },
    TurnInQuest {
        quest_id: String,
    },
    AbandonQuest {
        quest_id: String,
    },
    EquipItem {
        item_id: String,
        slot: Option<EquipmentSlot>,
    },
    UnequipItem {
        slot: EquipmentSlot,
    },
    UseItem {
        item_id: String,
    },
    DiscardItem {
        item_id: String,
        count: Option<u32>,
    },
    EquipBag {
        item_id: String,
        socket: Option<u32>,
    },
    UnequipBag {
        socket: u32,
    },
    LockpickEngage {
        object_id: u64,
        ante: u8,
    },
    LockpickAction {
        session_id: Option<String>,
        action: LockpickAction,
    },
    LockpickAbort {
        session_id: Option<String>,
    },
    ApplyTalents {
        player_class_id: String,
        spec_id: Option<String>,
        row_option_ids: [Option<String>; 6],
    },
    Prestige,
    Respec,
    SelectTalentRow {
        level: u8,
        option_id: Option<String>,
    },
    SetTalentSpec {
        player_class_id: String,
        spec_id: Option<String>,
    },
    SwitchTalentLoadout {
        index: u32,
    },
    DeleteTalentLoadout {
        index: u32,
    },
    RespondToResurrection {
        accept: bool,
    },
    ReleaseSpirit,
    ResurrectAtCorpse,
    ResurrectAtSpiritHealer,
    PartyInvite {
        target_id: u64,
    },
    PartyAccept,
    PartyDecline,
    PartyLeave,
    PartyKick {
        target_id: u64,
    },
    PartyPromote {
        target_id: u64,
    },
    ConvertPartyToRaid,
    ConvertRaidToParty,
    MoveRaidMember {
        target_id: u64,
        subgroup: u8,
    },
    SetPartyLootMaster {
        enabled: bool,
        looter: f64,
        threshold: MasterLootThreshold,
    },
    AssignMasterLoot {
        roll_id: f64,
        target_pids: Vec<f64>,
    },
    SetPartyMarker {
        entity_id: f64,
        marker_id: f64,
    },
    ClearPartyMarker {
        entity_id: f64,
    },
    RespondToReadyCheck {
        ready: bool,
    },
    RequestDuel {
        target_id: f64,
    },
    AcceptDuel,
    DeclineDuel,
    JoinArenaQueue {
        format: ArenaFormat,
    },
    LeaveArenaQueue,
    PickArenaAugment {
        augment_id: String,
    },
    RequestTrade {
        target_id: f64,
    },
    AcceptTrade,
    ConfirmTrade,
    CancelTrade,
    JoinValeCupQueue {
        bracket: ValeCupBracket,
        nation: ValeCupNation,
        role: ValeCupRole,
        enter_as_guild: bool,
    },
    LeaveValeCupQueue,
    SetValeCupRole {
        role: ValeCupRole,
    },
    ReadyValeCup,
    PlaceValeCupBet {
        side: ValeCupSide,
        amount: f64,
    },
    StartValeCupPractice {
        bracket: ValeCupBracket,
    },
    TakeMail {
        mail_id: f64,
    },
    DeleteMail {
        mail_id: f64,
    },
    MarkMailRead {
        mail_id: f64,
    },
    DepositBank {
        slot: f64,
        count: Option<f64>,
    },
    WithdrawBank {
        slot: f64,
        count: Option<f64>,
    },
    BuyBankSlots,
    SetDungeonFinderRoles {
        roles: Vec<DungeonFinderRole>,
    },
    JoinDungeonFinderQueue {
        activities: Vec<String>,
    },
    LeaveDungeonFinderQueue,
    RespondDungeonFinderProposal {
        accept: bool,
    },
    CreateDungeonFinderListing {
        activity: String,
        tags: Vec<DungeonFinderListingTag>,
    },
    CloseDungeonFinderListing,
    ApplyToDungeonFinderListing {
        listing_id: f64,
    },
    CancelDungeonFinderApplication,
    RespondToDungeonFinderApplication {
        applicant_id: f64,
        accept: bool,
    },
    LootCorpse {
        object_id: f64,
    },
    SubmitLootRoll {
        roll_id: f64,
        choice: LootRollChoice,
    },
    PickUpObject {
        object_id: f64,
    },
    AutoLoot {
        object_id: f64,
    },
    InteractWithDelveObject {
        object_id: f64,
    },
    CollectDelveChestLoot {
        object_id: f64,
    },
    SellAllJunk,
    CollectMarketProceeds,
    LeaveDungeon,
    LeaveDelve,
    BuyMarketListing {
        listing_id: f64,
    },
    CancelMarketListing {
        listing_id: f64,
    },
    ChooseDelveRite {
        intensity: DelveRiteIntensity,
    },
    SetDungeonDifficulty {
        difficulty: DungeonDifficulty,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientInputEvent {
    pub device: ClientInputDevice,
    pub intent: ClientGameplayIntent,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClientInputMappingError {
    InvalidActor,
    SequenceExhausted,
    MissingCommandContract {
        name: &'static str,
    },
    InvalidTalentOptionId {
        option_id: String,
    },
    InvalidTalentSpecId {
        player_class_id: String,
        spec_id: String,
    },
    Protocol(ProtocolError),
}

pub struct ClientCommandMapper {
    actor: EntityRef,
    next_sequence: Option<u32>,
}

impl ClientCommandMapper {
    pub fn new(actor: EntityRef, next_sequence: u32) -> Result<Self, ClientInputMappingError> {
        if actor.id == 0 {
            return Err(ClientInputMappingError::InvalidActor);
        }
        Ok(Self {
            actor,
            next_sequence: Some(next_sequence),
        })
    }

    pub fn map(&mut self, event: ClientInputEvent) -> Result<Command, ClientInputMappingError> {
        self.map_intent(event.intent)
    }

    pub fn map_intent(
        &mut self,
        intent: ClientGameplayIntent,
    ) -> Result<Command, ClientInputMappingError> {
        let (command_id, payload) = encode_intent(intent)?;
        validate_command_payload(command_id, &payload)
            .map_err(ClientInputMappingError::Protocol)?;
        require_client_send(command_id)?;

        let sequence = self
            .next_sequence
            .ok_or(ClientInputMappingError::SequenceExhausted)?;
        let command = Command {
            command_id,
            actor: self.actor,
            sequence,
            payload,
        };
        self.next_sequence = sequence.checked_add(1);
        Ok(command)
    }

    pub fn actor(&self) -> EntityRef {
        self.actor
    }

    pub fn next_sequence(&self) -> Option<u32> {
        self.next_sequence
    }
}

fn encode_intent(intent: ClientGameplayIntent) -> Result<(u16, Vec<u8>), ClientInputMappingError> {
    match intent {
        ClientGameplayIntent::CastAbilityAt { ability_id, aim } => {
            CastAtCommandPayload { ability_id, aim }
                .encode()
                .map(|payload| (CAST_AT_COMMAND_ID, payload))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::CastAbility {
            ability_id,
            target_id,
        } => CastAbilityCommandPayload {
            ability_id,
            target_id,
        }
        .encode()
        .map(|payload| (CAST_COMMAND_ID, payload))
        .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::CancelAura { aura_id } => CancelAuraCommandPayload { aura_id }
            .encode()
            .map(|payload| (CANCEL_AURA_COMMAND_ID, payload))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::ChangeSkin {
            catalog,
            skin_index,
        } => ChangeSkinCommandPayload {
            catalog,
            skin_index,
        }
        .encode()
        .map(|payload| (CHANGE_SKIN_COMMAND_ID, payload.to_vec()))
        .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::ClaimEventSkin { skin } => EventSkinPayload { skin }
            .encode()
            .map(|payload| (CLAIM_EVENT_SKIN_COMMAND_ID, payload.to_vec()))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::CastSlot { slot } => Ok((
            CAST_SLOT_COMMAND_ID,
            CastSlotCommandPayload { slot }.encode().to_vec(),
        )),
        ClientGameplayIntent::SetTarget { target_id } => TargetCommandPayload { target_id }
            .encode()
            .map(|payload| (TARGET_COMMAND_ID, payload.to_vec()))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::CycleTarget { friendly } => {
            empty_client_command(if friendly { "tabFriendly" } else { "tab" })
        }
        ClientGameplayIntent::TargetNearestFriendly => {
            Ok((TARGET_NEAREST_FRIENDLY_COMMAND_ID, Vec::new()))
        }
        ClientGameplayIntent::Interact => Ok((INTERACT_COMMAND_ID, Vec::new())),
        ClientGameplayIntent::SetAttacking { attacking } => Ok((
            if attacking {
                ATTACK_COMMAND_ID
            } else {
                STOP_ATTACK_COMMAND_ID
            },
            Vec::new(),
        )),
        ClientGameplayIntent::ToggleWeaponStow => Ok((WEAPON_STOW_COMMAND_ID, Vec::new())),
        ClientGameplayIntent::JoinCardDuelQueue => Ok((CARD_QUEUE_JOIN_COMMAND_ID, Vec::new())),
        ClientGameplayIntent::LeaveCardDuelQueue => Ok((CARD_QUEUE_LEAVE_COMMAND_ID, Vec::new())),
        ClientGameplayIntent::PlayCardInDuel { card_value } => Ok((
            CARD_PLAY_COMMAND_ID,
            CardPlayCommandPayload { card_value }.encode().to_vec(),
        )),
        ClientGameplayIntent::ForfeitCardDuel => Ok((CARD_FORFEIT_COMMAND_ID, Vec::new())),
        ClientGameplayIntent::ReleaseEmpoweredAbility { ability_id } => {
            ReleaseEmpoweredCommandPayload { ability_id }
                .encode()
                .map(|payload| (RELEASE_EMPOWERED_COMMAND_ID, payload))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::PetWaterJet => Ok((PET_WATER_JET_COMMAND_ID, Vec::new())),
        ClientGameplayIntent::SetPetAutoWaterJet { enabled } => {
            PetAutoWaterJetCommandPayload { enabled }
                .encode()
                .map(|payload| (PET_AUTO_WATER_JET_COMMAND_ID, payload.to_vec()))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::AbandonPet => Ok((PET_ABANDON_COMMAND_ID, Vec::new())),
        ClientGameplayIntent::RenamePet { name } => PetRenameCommandPayload { name }
            .encode()
            .map(|payload| (PET_RENAME_COMMAND_ID, payload))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::RevivePet => Ok((PET_REVIVE_COMMAND_ID, Vec::new())),
        ClientGameplayIntent::PetAttack => Ok((PET_ATTACK_COMMAND_ID, Vec::new())),
        ClientGameplayIntent::PetTaunt => Ok((PET_TAUNT_COMMAND_ID, Vec::new())),
        ClientGameplayIntent::SetPetAutoTaunt { enabled } => PetAutoTauntCommandPayload { enabled }
            .encode()
            .map(|payload| (PET_AUTO_TAUNT_COMMAND_ID, payload.to_vec()))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::FeedPet { item_id } => PetFeedCommandPayload { item_id }
            .encode()
            .map(|payload| (PET_FEED_COMMAND_ID, payload))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::HealPet => Ok((PET_HEAL_COMMAND_ID, Vec::new())),
        ClientGameplayIntent::SetPetMode { mode } => PetModeCommandPayload { mode }
            .encode()
            .map(|payload| (PET_MODE_COMMAND_ID, payload))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::AddFriend { name } => {
            social_name_command(FRIEND_ADD_COMMAND_ID, name)
        }
        ClientGameplayIntent::RemoveFriend { name } => {
            social_name_command(FRIEND_REMOVE_COMMAND_ID, name)
        }
        ClientGameplayIntent::AddBlock { name } => social_name_command(BLOCK_ADD_COMMAND_ID, name),
        ClientGameplayIntent::RemoveBlock { name } => {
            social_name_command(BLOCK_REMOVE_COMMAND_ID, name)
        }
        ClientGameplayIntent::CreateGuild { name } => {
            social_name_command(GUILD_CREATE_COMMAND_ID, name)
        }
        ClientGameplayIntent::InviteToGuild { name } => {
            social_name_command(GUILD_INVITE_COMMAND_ID, name)
        }
        ClientGameplayIntent::AcceptGuildInvite => empty_client_command("guild_accept"),
        ClientGameplayIntent::DeclineGuildInvite => empty_client_command("guild_decline"),
        ClientGameplayIntent::LeaveGuild => empty_client_command("guild_leave"),
        ClientGameplayIntent::KickGuildMember { name } => {
            social_name_command(GUILD_KICK_COMMAND_ID, name)
        }
        ClientGameplayIntent::PromoteGuildMember { name } => {
            social_name_command(GUILD_PROMOTE_COMMAND_ID, name)
        }
        ClientGameplayIntent::DemoteGuildMember { name } => {
            social_name_command(GUILD_DEMOTE_COMMAND_ID, name)
        }
        ClientGameplayIntent::TransferGuildLeadership { name } => {
            social_name_command(GUILD_TRANSFER_COMMAND_ID, name)
        }
        ClientGameplayIntent::DisbandGuild => empty_client_command("guild_disband"),
        ClientGameplayIntent::CreateGuildEvent {
            day,
            hour,
            title,
            note,
        } => GuildEventCreateCommandPayload {
            day,
            hour,
            title,
            note,
        }
        .encode()
        .map(|payload| (GUILD_EVENT_CREATE_COMMAND_ID, payload))
        .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::RemoveGuildEvent { event_id } => Ok((
            GUILD_EVENT_REMOVE_COMMAND_ID,
            GuildEventRemoveCommandPayload { event_id }
                .encode()
                .to_vec(),
        )),
        ClientGameplayIntent::AddIgnore { name } => {
            social_name_command(IGNORE_ADD_COMMAND_ID, name)
        }
        ClientGameplayIntent::RemoveIgnore { name } => {
            social_name_command(IGNORE_REMOVE_COMMAND_ID, name)
        }
        ClientGameplayIntent::AcceptQuest { quest_id } => AcceptQuestCommandPayload {
            quest_id,
            selection: None,
        }
        .encode()
        .map(|payload| (ACCEPT_QUEST_COMMAND_ID, payload))
        .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::AcceptLinkedQuest {
            quest_id,
            sharer_pid,
        } => LinkedQuestAcceptancePayload {
            quest_id,
            sharer_pid,
        }
        .encode()
        .map(|payload| (LINKED_QUEST_ACCEPT_COMMAND_ID, payload))
        .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::TurnInQuest { quest_id } => TurnInQuestCommandPayload { quest_id }
            .encode()
            .map(|payload| (TURN_IN_QUEST_COMMAND_ID, payload))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::AbandonQuest { quest_id } => AbandonQuestCommandPayload { quest_id }
            .encode()
            .map(|payload| (ABANDON_QUEST_COMMAND_ID, payload))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::EquipItem { item_id, slot } => EquipItemPayload { item_id, slot }
            .encode()
            .map(|payload| (EQUIP_ITEM_COMMAND_ID, payload))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::UnequipItem { slot } => UnequipItemPayload { slot }
            .encode()
            .map(|payload| (UNEQUIP_ITEM_COMMAND_ID, payload.to_vec()))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::UseItem { item_id } => UseItemCommandPayload { item_id }
            .encode()
            .map(|payload| (USE_ITEM_COMMAND_ID, payload))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::DiscardItem { item_id, count } => {
            DiscardItemCommandPayload { item_id, count }
                .encode()
                .map(|payload| (DISCARD_ITEM_COMMAND_ID, payload))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::EquipBag { item_id, socket } => {
            EquipBagCommandPayload { item_id, socket }
                .encode()
                .map(|payload| (EQUIP_BAG_COMMAND_ID, payload))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::UnequipBag { socket } => Ok((
            UNEQUIP_BAG_COMMAND_ID,
            UnequipBagCommandPayload { socket }.encode().to_vec(),
        )),
        ClientGameplayIntent::SwitchTalentLoadout { index } => {
            SwitchLoadoutCommandPayload { index }
                .encode()
                .map(|payload| (SWITCH_LOADOUT_COMMAND_ID, payload.to_vec()))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::DeleteTalentLoadout { index } => {
            DeleteLoadoutCommandPayload { index }
                .encode()
                .map(|payload| (DELETE_LOADOUT_COMMAND_ID, payload.to_vec()))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::RespondToResurrection { accept } => Ok((
            RESURRECT_RESPOND_COMMAND_ID,
            ResurrectRespondCommandPayload { accept }.encode().to_vec(),
        )),
        ClientGameplayIntent::ReleaseSpirit => empty_client_command("release"),
        ClientGameplayIntent::ResurrectAtCorpse => empty_client_command("resurrect_corpse"),
        ClientGameplayIntent::ResurrectAtSpiritHealer => empty_client_command("resurrect_healer"),
        ClientGameplayIntent::PartyInvite { target_id } => {
            target_party_command(PARTY_INVITE_COMMAND_ID, target_id)
        }
        ClientGameplayIntent::PartyAccept => empty_client_command("paccept"),
        ClientGameplayIntent::PartyDecline => empty_client_command("pdecline"),
        ClientGameplayIntent::PartyLeave => empty_client_command("pleave"),
        ClientGameplayIntent::PartyKick { target_id } => {
            target_party_command(PARTY_KICK_COMMAND_ID, target_id)
        }
        ClientGameplayIntent::PartyPromote { target_id } => {
            target_party_command(PARTY_PROMOTE_COMMAND_ID, target_id)
        }
        ClientGameplayIntent::ConvertPartyToRaid => empty_client_command("praid"),
        ClientGameplayIntent::ConvertRaidToParty => empty_client_command("punraid"),
        ClientGameplayIntent::MoveRaidMember {
            target_id,
            subgroup,
        } => PartyMoveRaidCommandPayload {
            target_id,
            subgroup,
        }
        .encode()
        .map(|payload| (PARTY_MOVE_RAID_COMMAND_ID, payload.to_vec()))
        .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::SetPartyLootMaster {
            enabled,
            looter,
            threshold,
        } => PartyLootMasterCommandPayload {
            enabled,
            looter,
            threshold,
        }
        .encode()
        .map(|payload| (PARTY_SET_LOOT_MASTER_COMMAND_ID, payload))
        .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::AssignMasterLoot {
            roll_id,
            target_pids,
        } => MasterLootAssignmentPayload {
            roll_id,
            target_pids,
        }
        .encode()
        .map(|payload| (MASTER_ASSIGN_COMMAND_ID, payload))
        .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::SetPartyMarker {
            entity_id,
            marker_id,
        } => PartyMarkerCommandPayload {
            entity_id,
            marker_id,
        }
        .encode()
        .map(|payload| (PARTY_SET_MARKER_COMMAND_ID, payload))
        .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::ClearPartyMarker { entity_id } => {
            PartyMarkerClearCommandPayload { entity_id }
                .encode()
                .map(|payload| (PARTY_CLEAR_MARKER_COMMAND_ID, payload.to_vec()))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::RespondToReadyCheck { ready } => Ok((
            PARTY_READY_RESPOND_COMMAND_ID,
            ReadyCheckRespondCommandPayload { ready }.encode().to_vec(),
        )),
        ClientGameplayIntent::RequestDuel { target_id } => DuelRequestCommandPayload { target_id }
            .encode()
            .map(|payload| (DUEL_REQUEST_COMMAND_ID, payload.to_vec()))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::AcceptDuel => empty_client_command("duel_accept"),
        ClientGameplayIntent::DeclineDuel => empty_client_command("duel_decline"),
        ClientGameplayIntent::JoinArenaQueue { format } => ArenaQueueCommandPayload { format }
            .encode()
            .map(|payload| (ARENA_QUEUE_COMMAND_ID, payload.to_vec()))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::LeaveArenaQueue => empty_client_command("arena_leave"),
        ClientGameplayIntent::PickArenaAugment { augment_id } => {
            ArenaAugmentCommandPayload { augment_id }
                .encode()
                .map(|payload| (ARENA_AUGMENT_COMMAND_ID, payload))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::RequestTrade { target_id } => {
            TradeRequestCommandPayload { target_id }
                .encode()
                .map(|payload| (TRADE_REQUEST_COMMAND_ID, payload.to_vec()))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::AcceptTrade => empty_client_command("trade_accept"),
        ClientGameplayIntent::ConfirmTrade => empty_client_command("trade_confirm"),
        ClientGameplayIntent::CancelTrade => empty_client_command("trade_cancel"),
        ClientGameplayIntent::JoinValeCupQueue {
            bracket,
            nation,
            role,
            enter_as_guild,
        } => ValeCupQueueCommandPayload {
            bracket,
            nation,
            role,
            enter_as_guild,
        }
        .encode()
        .map(|payload| (VALE_CUP_QUEUE_COMMAND_ID, payload.to_vec()))
        .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::LeaveValeCupQueue => empty_client_command("vcup_leave"),
        ClientGameplayIntent::SetValeCupRole { role } => ValeCupRoleCommandPayload { role }
            .encode()
            .map(|payload| (VALE_CUP_ROLE_COMMAND_ID, payload.to_vec()))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::ReadyValeCup => empty_client_command("vcup_ready"),
        ClientGameplayIntent::PlaceValeCupBet { side, amount } => {
            ValeCupBetCommandPayload { side, amount }
                .encode()
                .map(|payload| (VALE_CUP_BET_COMMAND_ID, payload.to_vec()))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::StartValeCupPractice { bracket } => {
            ValeCupPracticeCommandPayload { bracket }
                .encode()
                .map(|payload| (VALE_CUP_PRACTICE_COMMAND_ID, payload.to_vec()))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::TakeMail { mail_id } => MailIdCommandPayload { mail_id }
            .encode(MailAction::Take)
            .map(|payload| (MAIL_TAKE_COMMAND_ID, payload.to_vec()))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::DeleteMail { mail_id } => MailIdCommandPayload { mail_id }
            .encode(MailAction::Delete)
            .map(|payload| (MAIL_DELETE_COMMAND_ID, payload.to_vec()))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::MarkMailRead { mail_id } => MailIdCommandPayload { mail_id }
            .encode(MailAction::MarkRead)
            .map(|payload| (MAIL_READ_COMMAND_ID, payload.to_vec()))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::DepositBank { slot, count } => BankSlotCommandPayload { slot, count }
            .encode(BankAction::Deposit)
            .map(|payload| (BANK_DEPOSIT_COMMAND_ID, payload))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::WithdrawBank { slot, count } => {
            BankSlotCommandPayload { slot, count }
                .encode(BankAction::Withdraw)
                .map(|payload| (BANK_WITHDRAW_COMMAND_ID, payload))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::BuyBankSlots => empty_client_command("bank_buy_slots"),
        ClientGameplayIntent::SetDungeonFinderRoles { roles } => {
            DungeonFinderRolesPayload { roles }
                .encode()
                .map(|payload| (DUNGEON_FINDER_ROLES_COMMAND_ID, payload))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::JoinDungeonFinderQueue { activities } => {
            DungeonFinderActivitiesPayload { activities }
                .encode()
                .map(|payload| (DUNGEON_FINDER_QUEUE_COMMAND_ID, payload))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::LeaveDungeonFinderQueue => empty_client_command("df_queue_leave"),
        ClientGameplayIntent::RespondDungeonFinderProposal { accept } => {
            let payload = [u8::from(accept)];
            validate_command_payload(DUNGEON_FINDER_PROPOSAL_COMMAND_ID, &payload)
                .map(|_| (DUNGEON_FINDER_PROPOSAL_COMMAND_ID, payload.to_vec()))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::CreateDungeonFinderListing { activity, tags } => {
            DungeonFinderListingPayload { activity, tags }
                .encode()
                .map(|payload| (DUNGEON_FINDER_LIST_CREATE_COMMAND_ID, payload))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::CloseDungeonFinderListing => empty_client_command("df_list_close"),
        ClientGameplayIntent::ApplyToDungeonFinderListing { listing_id } => {
            DungeonFinderListingIdPayload { listing_id }
                .encode()
                .map(|payload| (DUNGEON_FINDER_APPLY_COMMAND_ID, payload.to_vec()))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::CancelDungeonFinderApplication => {
            empty_client_command("df_apply_cancel")
        }
        ClientGameplayIntent::RespondToDungeonFinderApplication {
            applicant_id,
            accept,
        } => DungeonFinderApplicationResponsePayload {
            applicant_id,
            accept,
        }
        .encode()
        .map(|payload| {
            (
                DUNGEON_FINDER_APPLICATION_RESPONSE_COMMAND_ID,
                payload.to_vec(),
            )
        })
        .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::SubmitLootRoll { roll_id, choice } => {
            LootRollPayload { roll_id, choice }
                .encode()
                .map(|payload| (LOOT_ROLL_COMMAND_ID, payload.to_vec()))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::LootCorpse { object_id } => WorldObjectIdPayload { object_id }
            .encode(WorldObjectAction::Loot)
            .map(|payload| (LOOT_COMMAND_ID, payload.to_vec()))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::PickUpObject { object_id } => WorldObjectIdPayload { object_id }
            .encode(WorldObjectAction::Pickup)
            .map(|payload| (PICKUP_COMMAND_ID, payload.to_vec()))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::AutoLoot { object_id } => WorldObjectIdPayload { object_id }
            .encode(WorldObjectAction::AutoLoot)
            .map(|payload| (AUTO_LOOT_COMMAND_ID, payload.to_vec()))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::InteractWithDelveObject { object_id } => {
            WorldObjectIdPayload { object_id }
                .encode(WorldObjectAction::DelveInteract)
                .map(|payload| (DELVE_INTERACT_COMMAND_ID, payload.to_vec()))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::CollectDelveChestLoot { object_id } => {
            WorldObjectIdPayload { object_id }
                .encode(WorldObjectAction::CollectDelveChestLoot)
                .map(|payload| (COLLECT_DELVE_CHEST_LOOT_COMMAND_ID, payload.to_vec()))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::SellAllJunk => empty_client_command("sell_all_junk"),
        ClientGameplayIntent::CollectMarketProceeds => empty_client_command("market_collect"),
        ClientGameplayIntent::LeaveDungeon => empty_client_command("leave_dungeon"),
        ClientGameplayIntent::LeaveDelve => empty_client_command("leave_delve"),
        ClientGameplayIntent::BuyMarketListing { listing_id } => {
            MarketListingIdPayload { listing_id }
                .encode(MarketAction::Buy)
                .map(|payload| (MARKET_BUY_COMMAND_ID, payload.to_vec()))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::CancelMarketListing { listing_id } => {
            MarketListingIdPayload { listing_id }
                .encode(MarketAction::Cancel)
                .map(|payload| (MARKET_CANCEL_COMMAND_ID, payload.to_vec()))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::ChooseDelveRite { intensity } => DelveRiteChoosePayload { intensity }
            .encode()
            .map(|payload| (DELVE_RITE_CHOOSE_COMMAND_ID, payload.to_vec()))
            .map_err(ClientInputMappingError::Protocol),
        ClientGameplayIntent::SetDungeonDifficulty { difficulty } => {
            DungeonDifficultyPayload { difficulty }
                .encode()
                .map(|payload| (SET_DUNGEON_DIFFICULTY_COMMAND_ID, payload.to_vec()))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::LockpickEngage { object_id, ante } => {
            LockpickEngageCommandPayload { object_id, ante }
                .encode()
                .map(|payload| (LOCKPICK_ENGAGE_COMMAND_ID, payload.to_vec()))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::LockpickAction { session_id, action } => {
            LockpickActionCommandPayload { session_id, action }
                .encode()
                .map(|payload| (LOCKPICK_ACTION_COMMAND_ID, payload))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::LockpickAbort { session_id } => {
            LockpickAbortCommandPayload { session_id }
                .encode()
                .map(|payload| (LOCKPICK_ABORT_COMMAND_ID, payload))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::ApplyTalents {
            player_class_id,
            spec_id,
            row_option_ids,
        } => {
            let spec_code = match spec_id {
                None => 0,
                Some(spec_id) => talent_spec_code(&player_class_id, &spec_id).ok_or_else(|| {
                    ClientInputMappingError::InvalidTalentSpecId {
                        player_class_id: player_class_id.clone(),
                        spec_id,
                    }
                })?,
            };
            let row_levels = [5, 8, 11, 14, 17, 20];
            let mut row_option_codes = [0; 6];
            for (index, option_id) in row_option_ids.into_iter().enumerate() {
                let Some(option_id) = option_id else {
                    continue;
                };
                let option_code = talent_option_code(&option_id).ok_or_else(|| {
                    ClientInputMappingError::InvalidTalentOptionId {
                        option_id: option_id.clone(),
                    }
                })?;
                if !talent_option_matches_class_row(
                    &player_class_id,
                    row_levels[index],
                    option_code,
                ) {
                    return Err(ClientInputMappingError::InvalidTalentOptionId { option_id });
                }
                row_option_codes[index] = option_code;
            }
            ApplyTalentsCommandPayload {
                spec_code,
                row_option_codes,
            }
            .encode()
            .map(|payload| (APPLY_TALENTS_COMMAND_ID, payload.to_vec()))
            .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::Prestige => empty_client_command("prestige"),
        ClientGameplayIntent::Respec => empty_client_command("respec"),
        ClientGameplayIntent::SelectTalentRow { level, option_id } => {
            let option_code = match option_id {
                None => 0,
                Some(option_id) => talent_option_code(&option_id)
                    .ok_or(ClientInputMappingError::InvalidTalentOptionId { option_id })?,
            };
            SelectTalentRowCommandPayload { level, option_code }
                .encode()
                .map(|payload| (SELECT_TALENT_ROW_COMMAND_ID, payload.to_vec()))
                .map_err(ClientInputMappingError::Protocol)
        }
        ClientGameplayIntent::SetTalentSpec {
            player_class_id,
            spec_id,
        } => {
            let spec_code = match spec_id {
                None => 0,
                Some(spec_id) => talent_spec_code(&player_class_id, &spec_id).ok_or(
                    ClientInputMappingError::InvalidTalentSpecId {
                        player_class_id,
                        spec_id,
                    },
                )?,
            };
            SetSpecCommandPayload { spec_code }
                .encode()
                .map(|payload| (SET_SPEC_COMMAND_ID, payload.to_vec()))
                .map_err(ClientInputMappingError::Protocol)
        }
    }
}

fn target_party_command(
    command_id: u16,
    target_id: u64,
) -> Result<(u16, Vec<u8>), ClientInputMappingError> {
    TargetCommandPayload {
        target_id: Some(target_id),
    }
    .encode()
    .map(|payload| (command_id, payload.to_vec()))
    .map_err(ClientInputMappingError::Protocol)
}

fn social_name_command(
    command_id: u16,
    name: String,
) -> Result<(u16, Vec<u8>), ClientInputMappingError> {
    SocialNameCommandPayload { name }
        .encode(command_id)
        .map(|payload| (command_id, payload))
        .map_err(ClientInputMappingError::Protocol)
}

fn empty_client_command(name: &'static str) -> Result<(u16, Vec<u8>), ClientInputMappingError> {
    let payload = COMMAND_PAYLOAD_CATALOG
        .iter()
        .find(|entry| entry.name == name)
        .filter(|entry| {
            entry.kind == CommandPayloadKind::Empty && entry.fixed_byte_length() == Some(0)
        })
        .ok_or(ClientInputMappingError::MissingCommandContract { name })?;
    let command = command_descriptor(payload.id)
        .filter(|entry| entry.name == name && entry.kind == CommandKind::ClientSend)
        .ok_or(ClientInputMappingError::MissingCommandContract { name })?;
    Ok((command.id, Vec::new()))
}

fn require_client_send(command_id: u16) -> Result<(), ClientInputMappingError> {
    let payload = command_payload_descriptor(command_id)
        .ok_or(ClientInputMappingError::MissingCommandContract { name: "unknown" })?;
    command_descriptor(command_id)
        .filter(|entry| entry.name == payload.name && entry.kind == CommandKind::ClientSend)
        .map(|_| ())
        .ok_or(ClientInputMappingError::MissingCommandContract { name: payload.name })
}

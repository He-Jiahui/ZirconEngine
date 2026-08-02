use std::collections::BTreeMap;

use woc_client::{
    ClientCommandMapper, ClientGameplayIntent, ClientInputDevice, ClientInputEvent,
    ClientInputMappingError, ClientTalentAllocation,
};
use woc_protocol::{
    command_descriptor, talent_option_code, talent_spec_code, AbandonQuestCommandPayload,
    AcceptQuestCommandPayload, ApplyTalentsCommandPayload, ArenaAugmentCommandPayload, ArenaFormat,
    ArenaQueueCommandPayload, BankAction, BankSlotCommandPayload, CancelAuraCommandPayload,
    CardPlayCommandPayload, CastAbilityCommandPayload, CastAtCommandPayload,
    CastSlotCommandPayload, ChallengeResponseCommandPayload, ChangeSkinCommandPayload,
    ChangeWeaponSkinCommandPayload, ChatCommandPayload, CommandKind,
    CompanionUpgradeCommandPayload, CraftItemCommandPayload, DeedSetTitleCommandPayload,
    DeleteLoadoutCommandPayload, DelveBuyCommandPayload, DiscardItemCommandPayload,
    DuelRequestCommandPayload, DungeonFinderActivitiesPayload,
    DungeonFinderApplicationResponsePayload, DungeonFinderListingIdPayload,
    DungeonFinderListingPayload, DungeonFinderListingTag, DungeonFinderRole,
    DungeonFinderRolesPayload, EmoteCommandPayload, EmoteId, EnterDelveCommandPayload,
    EnterDungeonCommandPayload, EntityRef, EquipBagCommandPayload, EquipItemPayload, EquipmentSlot,
    GroundTargetPoint, GuildEventCreateCommandPayload, GuildEventRemoveCommandPayload,
    HarvestCorpseCommandPayload, HarvestNodeCommandPayload, HeroicBuyCommandPayload,
    InventoryMovePayload, LinkedQuestAcceptancePayload, LockpickAbortCommandPayload,
    LockpickAction, LockpickActionCommandPayload, LockpickEngageCommandPayload, MailAction,
    MailIdCommandPayload, MailSendAttachment, MailSendCommandPayload, MarketListCommandPayload,
    MarketSearchCommandPayload, MasterLootThreshold, PartyLootMasterCommandPayload,
    PartyMarkerClearCommandPayload, PartyMarkerCommandPayload, PartyMoveRaidCommandPayload,
    PetAutoTauntCommandPayload, PetAutoWaterJetCommandPayload, PetFeedCommandPayload,
    PetModeCommandPayload, PetRenameCommandPayload, ProtocolError, ReleaseEmpoweredCommandPayload,
    ResurrectRespondCommandPayload, SaveLoadoutCommandPayload, SelectTalentRowCommandPayload,
    SetSpecCommandPayload, SkinCatalog, SocialNameCommandPayload, SwitchLoadoutCommandPayload,
    TargetCommandPayload, TelemetryPayload, TownFocusAllocationEntry, TownFocusCommandPayload,
    TradeOfferCommandPayload, TradeOfferItem, TradeRequestCommandPayload,
    TurnInQuestCommandPayload, UnequipBagCommandPayload, UnequipItemPayload,
    UnequipMechChromaCommandPayload, UseItemCommandPayload, ValeCupBetCommandPayload,
    ValeCupBracket, ValeCupNation, ValeCupPracticeCommandPayload, ValeCupQueueCommandPayload,
    ValeCupRole, ValeCupRoleCommandPayload, ValeCupSide, WeaponSkinChange, WeaponSkinType,
    WorldObjectAction, WorldObjectIdPayload, ABANDON_QUEST_COMMAND_ID, ACCEPT_QUEST_COMMAND_ID,
    APPLY_TALENTS_COMMAND_ID, ARENA_AUGMENT_COMMAND_ID, ARENA_LEAVE_COMMAND_ID,
    ARENA_QUEUE_COMMAND_ID, ATTACK_COMMAND_ID, AUTO_LOOT_COMMAND_ID, BANK_BUY_SLOTS_COMMAND_ID,
    BANK_DEPOSIT_COMMAND_ID, BANK_WITHDRAW_COMMAND_ID, BLOCK_ADD_COMMAND_ID,
    BLOCK_REMOVE_COMMAND_ID, CANCEL_AURA_COMMAND_ID, CARD_FORFEIT_COMMAND_ID, CARD_PLAY_COMMAND_ID,
    CARD_QUEUE_JOIN_COMMAND_ID, CARD_QUEUE_LEAVE_COMMAND_ID, CAST_AT_COMMAND_ID, CAST_COMMAND_ID,
    CAST_SLOT_COMMAND_ID, CHALLENGE_RESPONSE_COMMAND_ID, CHANGE_SKIN_COMMAND_ID,
    CHANGE_WEAPON_SKIN_COMMAND_ID, CHAT_COMMAND_ID, COLLECT_DELVE_CHEST_LOOT_COMMAND_ID,
    COMPANION_UPGRADE_COMMAND_ID, CRAFT_ITEM_COMMAND_ID, DEED_SET_TITLE_COMMAND_ID,
    DELETE_LOADOUT_COMMAND_ID, DELVE_BUY_COMMAND_ID, DELVE_INTERACT_COMMAND_ID,
    DISCARD_ITEM_COMMAND_ID, DUEL_ACCEPT_COMMAND_ID, DUEL_DECLINE_COMMAND_ID,
    DUEL_REQUEST_COMMAND_ID, DUNGEON_FINDER_APPLICATION_RESPONSE_COMMAND_ID,
    DUNGEON_FINDER_APPLY_CANCEL_COMMAND_ID, DUNGEON_FINDER_APPLY_COMMAND_ID,
    DUNGEON_FINDER_LIST_CLOSE_COMMAND_ID, DUNGEON_FINDER_LIST_CREATE_COMMAND_ID,
    DUNGEON_FINDER_PROPOSAL_COMMAND_ID, DUNGEON_FINDER_QUEUE_COMMAND_ID,
    DUNGEON_FINDER_QUEUE_LEAVE_COMMAND_ID, DUNGEON_FINDER_ROLES_COMMAND_ID, EMOTE_COMMAND_ID,
    ENTER_DELVE_COMMAND_ID, ENTER_DUNGEON_COMMAND_ID, EQUIP_BAG_COMMAND_ID, EQUIP_ITEM_COMMAND_ID,
    FRIEND_ADD_COMMAND_ID, FRIEND_REMOVE_COMMAND_ID, GUILD_CREATE_COMMAND_ID,
    GUILD_DEMOTE_COMMAND_ID, GUILD_EVENT_CREATE_COMMAND_ID, GUILD_EVENT_REMOVE_COMMAND_ID,
    GUILD_INVITE_COMMAND_ID, GUILD_KICK_COMMAND_ID, GUILD_PROMOTE_COMMAND_ID,
    GUILD_TRANSFER_COMMAND_ID, HARVEST_CORPSE_COMMAND_ID, HARVEST_NODE_COMMAND_ID,
    HEROIC_BUY_COMMAND_ID, IGNORE_ADD_COMMAND_ID, IGNORE_REMOVE_COMMAND_ID, INTERACT_COMMAND_ID,
    INVENTORY_MOVE_COMMAND_ID, LEAVE_DELVE_COMMAND_ID, LEAVE_DUNGEON_COMMAND_ID,
    LINKED_QUEST_ACCEPT_COMMAND_ID, LOCKPICK_ABORT_COMMAND_ID, LOCKPICK_ACTION_COMMAND_ID,
    LOCKPICK_ENGAGE_COMMAND_ID, LOOT_COMMAND_ID, MAIL_DELETE_COMMAND_ID, MAIL_READ_COMMAND_ID,
    MAIL_SEND_COMMAND_ID, MAIL_TAKE_COMMAND_ID, MARKET_COLLECT_COMMAND_ID, MARKET_LIST_COMMAND_ID,
    MARKET_SEARCH_COMMAND_ID, PARTY_ACCEPT_COMMAND_ID, PARTY_CLEAR_MARKER_COMMAND_ID,
    PARTY_DECLINE_COMMAND_ID, PARTY_INVITE_COMMAND_ID, PARTY_KICK_COMMAND_ID,
    PARTY_LEAVE_COMMAND_ID, PARTY_MOVE_RAID_COMMAND_ID, PARTY_PROMOTE_COMMAND_ID,
    PARTY_RAID_COMMAND_ID, PARTY_READY_RESPOND_COMMAND_ID, PARTY_SET_LOOT_MASTER_COMMAND_ID,
    PARTY_SET_MARKER_COMMAND_ID, PARTY_UNRAID_COMMAND_ID, PET_ABANDON_COMMAND_ID,
    PET_ATTACK_COMMAND_ID, PET_AUTO_TAUNT_COMMAND_ID, PET_AUTO_WATER_JET_COMMAND_ID,
    PET_FEED_COMMAND_ID, PET_HEAL_COMMAND_ID, PET_MODE_COMMAND_ID, PET_RENAME_COMMAND_ID,
    PET_REVIVE_COMMAND_ID, PET_TAUNT_COMMAND_ID, PET_WATER_JET_COMMAND_ID, PICKUP_COMMAND_ID,
    RELEASE_COMMAND_ID, RELEASE_EMPOWERED_COMMAND_ID, RESPEC_COMMAND_ID,
    RESURRECT_CORPSE_COMMAND_ID, RESURRECT_HEALER_COMMAND_ID, RESURRECT_RESPOND_COMMAND_ID,
    SAVE_LOADOUT_COMMAND_ID, SELECT_TALENT_ROW_COMMAND_ID, SELL_ALL_JUNK_COMMAND_ID,
    SET_SPEC_COMMAND_ID, SET_TOWN_FOCUS_COMMAND_ID, STOP_ATTACK_COMMAND_ID,
    SWITCH_LOADOUT_COMMAND_ID, TARGET_COMMAND_ID, TARGET_NEAREST_FRIENDLY_COMMAND_ID,
    TELEMETRY_COMMAND_ID, TRADE_ACCEPT_COMMAND_ID, TRADE_CANCEL_COMMAND_ID,
    TRADE_CONFIRM_COMMAND_ID, TRADE_OFFER_COMMAND_ID, TRADE_REQUEST_COMMAND_ID,
    TURN_IN_QUEST_COMMAND_ID, UNEQUIP_BAG_COMMAND_ID, UNEQUIP_ITEM_COMMAND_ID,
    UNEQUIP_MECH_CHROMA_COMMAND_ID, USE_ITEM_COMMAND_ID, VALE_CUP_BET_COMMAND_ID,
    VALE_CUP_LEAVE_COMMAND_ID, VALE_CUP_PRACTICE_COMMAND_ID, VALE_CUP_QUEUE_COMMAND_ID,
    VALE_CUP_READY_COMMAND_ID, VALE_CUP_ROLE_COMMAND_ID, WEAPON_STOW_COMMAND_ID,
};

fn actor() -> EntityRef {
    EntityRef {
        id: 41,
        generation: 3,
    }
}

fn event(device: ClientInputDevice, intent: ClientGameplayIntent) -> ClientInputEvent {
    ClientInputEvent { device, intent }
}

#[test]
fn deed_title_intent_maps_select_and_clear() {
    let mut mapper = ClientCommandMapper::new(actor(), 313).expect("valid actor");
    for deed_id in [Some("prog_veteran".to_owned()), None] {
        let command = mapper
            .map_intent(ClientGameplayIntent::SetActiveDeedTitle {
                deed_id: deed_id.clone(),
            })
            .expect("deed title command");
        assert_eq!(command.command_id, DEED_SET_TITLE_COMMAND_ID);
        assert_eq!(
            DeedSetTitleCommandPayload::decode(&command.payload).expect("typed payload"),
            DeedSetTitleCommandPayload { deed_id }
        );
    }
    assert_eq!(mapper.next_sequence(), Some(315));
}

#[test]
fn town_focus_intent_maps_the_source_allocation_record() {
    let mut mapper = ClientCommandMapper::new(actor(), 315).expect("valid actor");
    let allocation = vec![
        TownFocusAllocationEntry {
            component: "hide".to_owned(),
            points: 6,
        },
        TownFocusAllocationEntry {
            component: "fang".to_owned(),
            points: 4,
        },
        TownFocusAllocationEntry {
            component: "herb".to_owned(),
            points: 0,
        },
    ];
    let command = mapper
        .map_intent(ClientGameplayIntent::SetTownFocus {
            allocation: allocation.clone(),
        })
        .expect("town focus command");

    assert_eq!(
        (command.command_id, command.sequence),
        (SET_TOWN_FOCUS_COMMAND_ID, 315)
    );
    assert_eq!(
        TownFocusCommandPayload::decode(&command.payload).expect("typed payload"),
        TownFocusCommandPayload { allocation }
    );
    assert_eq!(mapper.next_sequence(), Some(316));
}

#[test]
fn harvest_node_intent_maps_the_source_node_identity() {
    let mut mapper = ClientCommandMapper::new(actor(), 312).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::HarvestNode {
            node_id: "ore_eastbrook_1".to_owned(),
        })
        .expect("harvest node command");

    assert_eq!(
        (command.command_id, command.sequence),
        (HARVEST_NODE_COMMAND_ID, 312)
    );
    assert_eq!(
        HarvestNodeCommandPayload::decode(&command.payload).expect("typed payload"),
        HarvestNodeCommandPayload {
            node_id: "ore_eastbrook_1".to_owned(),
        }
    );
    assert_eq!(mapper.next_sequence(), Some(313));
}

#[test]
fn corpse_harvest_intent_canonicalizes_source_component_selection() {
    let mut mapper = ClientCommandMapper::new(actor(), 312).expect("valid actor");
    let focused = mapper
        .map_intent(ClientGameplayIntent::HarvestCorpse {
            target_id: 99,
            components: vec!["fang".to_owned(), "missing".to_owned()],
        })
        .expect("focused corpse harvest command");
    let spread = mapper
        .map_intent(ClientGameplayIntent::HarvestCorpse {
            target_id: 100,
            components: vec!["hide".to_owned(), "fang".to_owned(), "silk".to_owned()],
        })
        .expect("spread corpse harvest command");

    assert_eq!(focused.command_id, HARVEST_CORPSE_COMMAND_ID);
    assert_eq!(
        HarvestCorpseCommandPayload::decode(&focused.payload).expect("typed focused payload"),
        HarvestCorpseCommandPayload {
            target_id: 99,
            component_codes: vec![2, 0],
        }
    );
    assert_eq!(spread.command_id, HARVEST_CORPSE_COMMAND_ID);
    assert_eq!(
        HarvestCorpseCommandPayload::decode(&spread.payload).expect("typed spread payload"),
        HarvestCorpseCommandPayload {
            target_id: 100,
            component_codes: vec![0, 0, 0],
        }
    );
    assert_eq!(mapper.next_sequence(), Some(314));
}

#[test]
fn enter_dungeon_intent_preserves_unknown_ids_for_the_authoritative_lifecycle() {
    let mut mapper = ClientCommandMapper::new(actor(), 314).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::EnterDungeon {
            dungeon_id: "unlisted_authoritative_dungeon".to_owned(),
        })
        .expect("enter dungeon command");

    assert_eq!(
        (command.command_id, command.sequence),
        (ENTER_DUNGEON_COMMAND_ID, 314)
    );
    assert_eq!(
        EnterDungeonCommandPayload::decode(&command.payload).expect("typed payload"),
        EnterDungeonCommandPayload {
            dungeon_id: "unlisted_authoritative_dungeon".to_owned(),
        }
    );
    assert_eq!(mapper.next_sequence(), Some(315));
}

#[test]
fn craft_item_intent_maps_the_source_recipe_identity() {
    let mut mapper = ClientCommandMapper::new(actor(), 313).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::CraftItem {
            recipe_id: "recipe_minor_healing_potion".to_owned(),
        })
        .expect("craft item command");

    assert_eq!(
        (command.command_id, command.sequence),
        (CRAFT_ITEM_COMMAND_ID, 313)
    );
    assert_eq!(
        CraftItemCommandPayload::decode(&command.payload).expect("typed payload"),
        CraftItemCommandPayload {
            recipe_id: "recipe_minor_healing_potion".to_owned(),
        }
    );
    assert_eq!(mapper.next_sequence(), Some(314));
}

#[test]
fn heroic_buy_intent_maps_the_source_offer_identity() {
    let mut mapper = ClientCommandMapper::new(actor(), 314).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::BuyHeroicVendorItem {
            item_id: "seal_of_the_nine_oaths".to_owned(),
        })
        .expect("heroic-buy command");

    assert_eq!(
        (command.command_id, command.sequence),
        (HEROIC_BUY_COMMAND_ID, 314)
    );
    assert_eq!(
        HeroicBuyCommandPayload::decode(&command.payload).expect("typed heroic-buy payload"),
        HeroicBuyCommandPayload {
            item_id: "seal_of_the_nine_oaths".to_owned(),
        }
    );
    assert_eq!(mapper.next_sequence(), Some(315));
}

#[test]
fn delve_buy_intent_maps_both_source_identities() {
    let mut mapper = ClientCommandMapper::new(actor(), 314).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::BuyDelveShopItem {
            delve_id: "collapsed_reliquary".to_owned(),
            item_id: "reliquary_legs".to_owned(),
        })
        .expect("Delve-buy command");

    assert_eq!(
        (command.command_id, command.sequence),
        (DELVE_BUY_COMMAND_ID, 314)
    );
    assert_eq!(
        DelveBuyCommandPayload::decode(&command.payload).expect("typed Delve-buy payload"),
        DelveBuyCommandPayload {
            delve_id: "collapsed_reliquary".to_owned(),
            item_id: "reliquary_legs".to_owned(),
        }
    );
    assert_eq!(mapper.next_sequence(), Some(315));
}

#[test]
fn challenge_response_intent_preserves_raw_strings_without_signature_policy() {
    let mut mapper = ClientCommandMapper::new(actor(), 314).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::SendChallengeResponse {
            nonce: " nonce ".to_owned(),
            response: "42".to_owned(),
            signature: "sig:unknown".to_owned(),
        })
        .expect("challenge-response command");

    assert_eq!(
        (command.command_id, command.sequence),
        (CHALLENGE_RESPONSE_COMMAND_ID, 314)
    );
    assert_eq!(
        ChallengeResponseCommandPayload::decode(&command.payload)
            .expect("typed challenge-response payload"),
        ChallengeResponseCommandPayload {
            nonce: " nonce ".to_owned(),
            response: "42".to_owned(),
            signature: "sig:unknown".to_owned(),
        }
    );
    assert_eq!(mapper.next_sequence(), Some(315));
}

#[test]
fn enter_delve_intent_preserves_source_identities_without_admission_policy() {
    let mut mapper = ClientCommandMapper::new(actor(), 314).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::EnterDelve {
            delve_id: "drowned_litany".to_owned(),
            tier_id: "heroic".to_owned(),
        })
        .expect("enter-delve command");

    assert_eq!(
        (command.command_id, command.sequence),
        (ENTER_DELVE_COMMAND_ID, 314)
    );
    assert_eq!(
        EnterDelveCommandPayload::decode(&command.payload).expect("typed enter-delve payload"),
        EnterDelveCommandPayload {
            delve_id: "drowned_litany".to_owned(),
            tier_id: "heroic".to_owned(),
        }
    );
    assert_eq!(mapper.next_sequence(), Some(315));
}

#[test]
fn market_list_intent_preserves_source_values_without_market_policy() {
    let mut mapper = ClientCommandMapper::new(actor(), 314).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::ListMarketItem {
            item_id: " unknown reagent ".to_owned(),
            count: 2.75,
            price: -0.0,
        })
        .expect("market-list command");

    assert_eq!(
        (command.command_id, command.sequence),
        (MARKET_LIST_COMMAND_ID, 314)
    );
    assert_eq!(
        MarketListCommandPayload::decode(&command.payload).expect("typed market-list payload"),
        MarketListCommandPayload {
            item_id: " unknown reagent ".to_owned(),
            count: 2.75,
            price: 0.0,
        }
    );
    assert_eq!(mapper.next_sequence(), Some(315));
}

#[test]
fn market_search_intent_preserves_source_fields_without_normalization() {
    let mut mapper = ClientCommandMapper::new(actor(), 314).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::SearchMarket {
            search: "  Worn Blade  ".to_owned(),
            item_type: "unknown-kind".to_owned(),
            subtype: "?".to_owned(),
            rarity: "legendary".to_owned(),
            page: -2.75,
        })
        .expect("market-search command");

    assert_eq!(
        (command.command_id, command.sequence),
        (MARKET_SEARCH_COMMAND_ID, 314)
    );
    assert_eq!(
        MarketSearchCommandPayload::decode(&command.payload).expect("typed market-search payload"),
        MarketSearchCommandPayload {
            search: "  Worn Blade  ".to_owned(),
            item_type: "unknown-kind".to_owned(),
            subtype: "?".to_owned(),
            rarity: "legendary".to_owned(),
            page: -2.75,
        }
    );
    assert_eq!(mapper.next_sequence(), Some(315));
}

#[test]
fn trade_offer_intent_preserves_source_items_without_trade_policy() {
    let mut mapper = ClientCommandMapper::new(actor(), 314).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::SetTradeOffer {
            items: vec![
                TradeOfferItem {
                    item_id: " unknown ore ".to_owned(),
                    count: 2.75,
                },
                TradeOfferItem {
                    item_id: " unknown ore ".to_owned(),
                    count: -1.5,
                },
            ],
            copper: -10.25,
        })
        .expect("trade-offer command");

    assert_eq!(
        (command.command_id, command.sequence),
        (TRADE_OFFER_COMMAND_ID, 314)
    );
    assert_eq!(
        TradeOfferCommandPayload::decode(&command.payload).expect("typed trade-offer payload"),
        TradeOfferCommandPayload {
            items: vec![
                TradeOfferItem {
                    item_id: " unknown ore ".to_owned(),
                    count: 2.75,
                },
                TradeOfferItem {
                    item_id: " unknown ore ".to_owned(),
                    count: -1.5,
                },
            ],
            copper: -10.25,
        }
    );
    assert_eq!(mapper.next_sequence(), Some(315));
}

#[test]
fn companion_upgrade_intent_maps_the_source_identity() {
    let mut mapper = ClientCommandMapper::new(actor(), 314).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::UpgradeDelveCompanion {
            companion_id: "companion_tessa".to_owned(),
        })
        .expect("companion-upgrade command");

    assert_eq!(
        (command.command_id, command.sequence),
        (COMPANION_UPGRADE_COMMAND_ID, 314)
    );
    assert_eq!(
        CompanionUpgradeCommandPayload::decode(&command.payload)
            .expect("typed companion-upgrade payload"),
        CompanionUpgradeCommandPayload {
            companion_id: "companion_tessa".to_owned(),
        }
    );
    assert_eq!(mapper.next_sequence(), Some(315));
}

#[test]
fn unequip_mech_chroma_intent_maps_the_source_identity() {
    let mut mapper = ClientCommandMapper::new(actor(), 315).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::UnequipMechChroma {
            chroma_id: "vanguard_chrome".to_owned(),
        })
        .expect("unequip-mech-chroma command");

    assert_eq!(
        (command.command_id, command.sequence),
        (UNEQUIP_MECH_CHROMA_COMMAND_ID, 315)
    );
    assert_eq!(
        UnequipMechChromaCommandPayload::decode(&command.payload)
            .expect("typed unequip-mech-chroma payload"),
        UnequipMechChromaCommandPayload {
            chroma_id: "vanguard_chrome".to_owned(),
        }
    );
    assert_eq!(mapper.next_sequence(), Some(316));
}

#[test]
fn change_weapon_skin_intent_maps_apply_and_detach_without_rule_duplication() {
    let mut mapper = ClientCommandMapper::new(actor(), 316).expect("valid actor");
    let apply_change = WeaponSkinChange::Apply {
        skin_id: "guildmark_arming_sword".to_owned(),
    };
    let apply = mapper
        .map_intent(ClientGameplayIntent::ChangeWeaponSkin {
            change: apply_change.clone(),
        })
        .expect("weapon-skin apply command");
    let detach_change = WeaponSkinChange::Detach {
        weapon_type: WeaponSkinType::Sword,
    };
    let detach = mapper
        .map_intent(ClientGameplayIntent::ChangeWeaponSkin {
            change: detach_change.clone(),
        })
        .expect("weapon-skin detach command");

    assert_eq!(
        (apply.command_id, apply.sequence),
        (CHANGE_WEAPON_SKIN_COMMAND_ID, 316)
    );
    assert_eq!(
        (detach.command_id, detach.sequence),
        (CHANGE_WEAPON_SKIN_COMMAND_ID, 317)
    );
    assert_eq!(
        ChangeWeaponSkinCommandPayload::decode(&apply.payload)
            .expect("typed weapon-skin apply payload")
            .change,
        apply_change
    );
    assert_eq!(
        ChangeWeaponSkinCommandPayload::decode(&detach.payload)
            .expect("typed weapon-skin detach payload")
            .change,
        detach_change
    );
    assert_eq!(mapper.next_sequence(), Some(318));
}

#[test]
fn chat_intent_preserves_text_for_authoritative_routing() {
    let mut mapper = ClientCommandMapper::new(actor(), 314).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::Chat {
            text: "  /READYcheck  ".to_owned(),
        })
        .expect("chat command");

    assert_eq!(
        (command.command_id, command.sequence),
        (CHAT_COMMAND_ID, 314)
    );
    assert_eq!(
        ChatCommandPayload::decode(&command.payload).expect("typed chat payload"),
        ChatCommandPayload {
            text: "  /READYcheck  ".to_owned(),
        }
    );
    assert_eq!(mapper.next_sequence(), Some(315));
}

#[test]
fn save_loadout_intent_maps_source_name_bar_and_staged_allocation() {
    let mut mapper = ClientCommandMapper::new(actor(), 310).expect("valid actor");
    let mut action_bar = vec![Some("charge".to_owned()), None, Some("rend".to_owned())];
    action_bar.resize(23, None);
    let command = mapper
        .map_intent(ClientGameplayIntent::SaveTalentLoadout {
            name: "R".repeat(25),
            action_bar,
            player_class_id: "warrior".to_owned(),
            allocation: Some(ClientTalentAllocation {
                spec_id: Some("arms".to_owned()),
                row_option_ids: [
                    Some("war_row_double_charge".to_owned()),
                    None,
                    None,
                    None,
                    None,
                    None,
                ],
            }),
        })
        .expect("save loadout command");

    assert_eq!(
        (command.command_id, command.sequence),
        (SAVE_LOADOUT_COMMAND_ID, 310)
    );
    let decoded = SaveLoadoutCommandPayload::decode(&command.payload).expect("typed payload");
    assert_eq!(decoded.name, "R".repeat(24));
    assert_eq!(decoded.action_bar.len(), 22);
    assert_eq!(decoded.action_bar[0].as_deref(), Some("charge"));
    assert_eq!(decoded.action_bar[1], None);
    assert_eq!(decoded.action_bar[2].as_deref(), Some("rend"));
    assert!(decoded.allocation.is_some());
    assert_eq!(mapper.next_sequence(), Some(311));
}

#[test]
fn inventory_move_maps_dense_index_and_manual_target_cell() {
    let mut mapper = ClientCommandMapper::new(actor(), 950).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::MoveInventoryItem { from: 2, to: 17 })
        .expect("inventory move");

    assert_eq!(
        (command.command_id, command.sequence),
        (INVENTORY_MOVE_COMMAND_ID, 950)
    );
    assert_eq!(
        InventoryMovePayload::decode(&command.payload).expect("typed inventory move payload"),
        InventoryMovePayload { from: 2, to: 17 }
    );
    assert_eq!(mapper.next_sequence(), Some(951));
}

#[test]
fn emote_intent_maps_the_closed_source_id() {
    let mut mapper = ClientCommandMapper::new(actor(), 952).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::PlayEmote {
            emote: EmoteId::Salute,
        })
        .expect("emote");

    assert_eq!(
        (command.command_id, command.sequence),
        (EMOTE_COMMAND_ID, 952)
    );
    assert_eq!(
        EmoteCommandPayload::decode(&command.payload).expect("typed emote payload"),
        EmoteCommandPayload {
            emote: EmoteId::Salute,
        }
    );
    assert_eq!(mapper.next_sequence(), Some(953));
}

#[test]
fn keyboard_gamepad_and_touch_generate_the_same_authoritative_command() {
    let intent = ClientGameplayIntent::CastSlot { slot: -1 };
    let commands = [
        ClientInputDevice::KeyboardMouse,
        ClientInputDevice::Gamepad,
        ClientInputDevice::Touch,
    ]
    .map(|device| {
        ClientCommandMapper::new(actor(), 17)
            .expect("valid actor")
            .map(event(device, intent.clone()))
            .expect("mapped input")
    });

    assert_eq!(commands[0], commands[1]);
    assert_eq!(commands[1], commands[2]);
    assert_eq!(commands[0].command_id, CAST_SLOT_COMMAND_ID);
    assert_eq!(
        CastSlotCommandPayload::decode(&commands[0].payload).expect("typed slot payload"),
        CastSlotCommandPayload { slot: -1 }
    );
}

#[test]
fn cast_at_maps_the_authoritative_ground_point_without_consuming_a_rejected_point() {
    let mut mapper = ClientCommandMapper::new(actor(), 19).expect("valid actor");
    let aim = GroundTargetPoint::new(8.5, -3.25).expect("finite point");
    let command = mapper
        .map_intent(ClientGameplayIntent::CastAbilityAt {
            ability_id: "flame_strike".to_owned(),
            aim,
        })
        .expect("cast at");
    assert_eq!(
        (command.command_id, command.sequence),
        (CAST_AT_COMMAND_ID, 19)
    );
    assert_eq!(
        CastAtCommandPayload::decode(&command.payload).expect("typed cast-at payload"),
        CastAtCommandPayload::new("flame_strike".to_owned(), 8.5, -3.25).expect("finite point")
    );

    assert!(matches!(
        GroundTargetPoint::new(f64::INFINITY, 0.0),
        Err(ProtocolError::NonFinite {
            field: "GroundTargetPoint.x",
            ..
        })
    ));
    assert_eq!(mapper.next_sequence(), Some(20));
}

#[test]
fn cast_ability_uses_the_source_string_field_and_preserves_sequence_on_rejection() {
    let mut mapper = ClientCommandMapper::new(actor(), 21).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::CastAbility {
            ability_id: "frostbolt".to_owned(),
            target_id: None,
        })
        .expect("cast ability");
    assert_eq!(
        (command.command_id, command.sequence),
        (CAST_COMMAND_ID, 21)
    );
    assert_eq!(
        CastAbilityCommandPayload::decode(&command.payload).expect("typed cast payload"),
        CastAbilityCommandPayload {
            ability_id: "frostbolt".to_owned(),
            target_id: None,
        }
    );

    assert_eq!(
        mapper
            .map_intent(ClientGameplayIntent::CastAbility {
                ability_id: "x".repeat(257),
                target_id: None,
            })
            .expect_err("overlong ability id"),
        ClientInputMappingError::Protocol(ProtocolError::CollectionTooLarge {
            context: "CastAbilityCommandPayload.ability_id",
            actual: 257,
            maximum: 256,
        })
    );
    assert_eq!(mapper.next_sequence(), Some(22));
}

#[test]
fn cast_ability_on_preserves_the_source_mouseover_target_override() {
    let mut mapper = ClientCommandMapper::new(actor(), 22).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::CastAbility {
            ability_id: "temporal_reversal".to_owned(),
            target_id: Some(9_001),
        })
        .expect("cast ability on target");

    assert_eq!(
        (command.command_id, command.sequence),
        (CAST_COMMAND_ID, 22)
    );
    assert_eq!(
        CastAbilityCommandPayload::decode(&command.payload).expect("typed cast-on payload"),
        CastAbilityCommandPayload {
            ability_id: "temporal_reversal".to_owned(),
            target_id: Some(9_001),
        }
    );
    assert_eq!(mapper.next_sequence(), Some(23));
}

#[test]
fn cast_ability_keeps_explicit_zero_distinct_from_an_absent_target() {
    let explicit_zero = CastAbilityCommandPayload {
        ability_id: "temporal_reversal".to_owned(),
        target_id: Some(0),
    };
    let absent_target = CastAbilityCommandPayload {
        ability_id: "temporal_reversal".to_owned(),
        target_id: None,
    };

    let explicit_zero_bytes = explicit_zero.encode().expect("encode explicit zero target");
    let absent_target_bytes = absent_target.encode().expect("encode absent target");
    assert_ne!(explicit_zero_bytes, absent_target_bytes);
    assert_eq!(
        CastAbilityCommandPayload::decode(&explicit_zero_bytes).expect("decode explicit zero"),
        explicit_zero
    );
}

#[test]
fn party_commands_preserve_current_source_payload_shapes() {
    let mut mapper = ClientCommandMapper::new(actor(), 200).expect("valid actor");
    let invite = mapper
        .map_intent(ClientGameplayIntent::PartyInvite { target_id: 9_001 })
        .expect("party invite");
    let accept = mapper
        .map_intent(ClientGameplayIntent::PartyAccept)
        .expect("party accept");
    let decline = mapper
        .map_intent(ClientGameplayIntent::PartyDecline)
        .expect("party decline");
    let leave = mapper
        .map_intent(ClientGameplayIntent::PartyLeave)
        .expect("party leave");
    let kick = mapper
        .map_intent(ClientGameplayIntent::PartyKick { target_id: 9_002 })
        .expect("party kick");
    let promote = mapper
        .map_intent(ClientGameplayIntent::PartyPromote { target_id: 9_003 })
        .expect("party promote");
    let raid = mapper
        .map_intent(ClientGameplayIntent::ConvertPartyToRaid)
        .expect("party raid");
    let unraid = mapper
        .map_intent(ClientGameplayIntent::ConvertRaidToParty)
        .expect("party unraid");
    let move_raid = mapper
        .map_intent(ClientGameplayIntent::MoveRaidMember {
            target_id: 9_004,
            subgroup: 2,
        })
        .expect("move raid member");
    let loot_master = mapper
        .map_intent(ClientGameplayIntent::SetPartyLootMaster {
            enabled: true,
            looter: 9_001.0,
            threshold: MasterLootThreshold::Rare,
        })
        .expect("set party loot master");
    let marker = mapper
        .map_intent(ClientGameplayIntent::SetPartyMarker {
            entity_id: 9_004.0,
            marker_id: 7.0,
        })
        .expect("set party marker");
    let clear_marker = mapper
        .map_intent(ClientGameplayIntent::ClearPartyMarker { entity_id: 9_004.0 })
        .expect("clear party marker");
    let ready = mapper
        .map_intent(ClientGameplayIntent::RespondToReadyCheck { ready: false })
        .expect("respond to ready check");

    assert_eq!(
        (invite.command_id, invite.sequence, invite.payload),
        (
            PARTY_INVITE_COMMAND_ID,
            200,
            9_001_u64.to_le_bytes().to_vec()
        )
    );
    assert_eq!(
        (accept.command_id, accept.sequence, accept.payload),
        (PARTY_ACCEPT_COMMAND_ID, 201, Vec::new())
    );
    assert_eq!(
        (decline.command_id, decline.sequence, decline.payload),
        (PARTY_DECLINE_COMMAND_ID, 202, Vec::new())
    );
    assert_eq!(
        (leave.command_id, leave.sequence, leave.payload),
        (PARTY_LEAVE_COMMAND_ID, 203, Vec::new())
    );
    assert_eq!(
        (kick.command_id, kick.sequence, kick.payload),
        (PARTY_KICK_COMMAND_ID, 204, 9_002_u64.to_le_bytes().to_vec())
    );
    assert_eq!(
        (promote.command_id, promote.sequence, promote.payload),
        (
            PARTY_PROMOTE_COMMAND_ID,
            205,
            9_003_u64.to_le_bytes().to_vec()
        )
    );
    assert_eq!(
        (raid.command_id, raid.sequence, raid.payload),
        (PARTY_RAID_COMMAND_ID, 206, Vec::new())
    );
    assert_eq!(
        (unraid.command_id, unraid.sequence, unraid.payload),
        (PARTY_UNRAID_COMMAND_ID, 207, Vec::new())
    );
    assert_eq!(
        (move_raid.command_id, move_raid.sequence),
        (PARTY_MOVE_RAID_COMMAND_ID, 208)
    );
    assert_eq!(
        PartyMoveRaidCommandPayload::decode(&move_raid.payload).expect("typed raid move"),
        PartyMoveRaidCommandPayload {
            target_id: 9_004,
            subgroup: 2,
        }
    );
    assert_eq!(
        (loot_master.command_id, loot_master.sequence),
        (PARTY_SET_LOOT_MASTER_COMMAND_ID, 209)
    );
    assert_eq!(
        PartyLootMasterCommandPayload::decode(&loot_master.payload)
            .expect("typed party loot-master"),
        PartyLootMasterCommandPayload {
            enabled: true,
            looter: 9_001.0,
            threshold: MasterLootThreshold::Rare,
        }
    );
    assert_eq!(
        (marker.command_id, marker.sequence),
        (PARTY_SET_MARKER_COMMAND_ID, 210)
    );
    assert_eq!(
        PartyMarkerCommandPayload::decode(&marker.payload).expect("typed party marker"),
        PartyMarkerCommandPayload {
            entity_id: 9_004.0,
            marker_id: 7.0,
        }
    );
    assert_eq!(
        (clear_marker.command_id, clear_marker.sequence),
        (PARTY_CLEAR_MARKER_COMMAND_ID, 211)
    );
    assert_eq!(
        PartyMarkerClearCommandPayload::decode(&clear_marker.payload)
            .expect("typed clear party marker"),
        PartyMarkerClearCommandPayload { entity_id: 9_004.0 }
    );
    assert_eq!(
        (ready.command_id, ready.sequence, ready.payload),
        (PARTY_READY_RESPOND_COMMAND_ID, 212, vec![0])
    );
    assert_eq!(mapper.next_sequence(), Some(213));

    assert_eq!(
        mapper
            .map_intent(ClientGameplayIntent::MoveRaidMember {
                target_id: 9_005,
                subgroup: 3,
            })
            .expect_err("subgroup three is outside the source union"),
        ClientInputMappingError::Protocol(ProtocolError::InvalidRaidSubgroup(3))
    );
    assert_eq!(mapper.next_sequence(), Some(213));
}

#[test]
fn duel_arena_intents_preserve_source_commands_and_transport_bounds() {
    let mut mapper = ClientCommandMapper::new(actor(), 213).expect("valid actor");
    let request = mapper
        .map_intent(ClientGameplayIntent::RequestDuel { target_id: 9_001.5 })
        .expect("duel request");
    let accept = mapper
        .map_intent(ClientGameplayIntent::AcceptDuel)
        .expect("duel accept");
    let decline = mapper
        .map_intent(ClientGameplayIntent::DeclineDuel)
        .expect("duel decline");
    let queue = mapper
        .map_intent(ClientGameplayIntent::JoinArenaQueue {
            format: ArenaFormat::YumiThree,
        })
        .expect("arena queue");
    let leave = mapper
        .map_intent(ClientGameplayIntent::LeaveArenaQueue)
        .expect("arena leave");
    let augment = mapper
        .map_intent(ClientGameplayIntent::PickArenaAugment {
            augment_id: "fiesta_sprint".to_owned(),
        })
        .expect("arena augment");

    assert_eq!(
        (request.command_id, request.sequence),
        (DUEL_REQUEST_COMMAND_ID, 213)
    );
    assert_eq!(
        DuelRequestCommandPayload::decode(&request.payload).expect("typed duel request"),
        DuelRequestCommandPayload { target_id: 9_001.5 }
    );
    assert_eq!(
        (accept.command_id, accept.sequence, accept.payload),
        (DUEL_ACCEPT_COMMAND_ID, 214, Vec::new())
    );
    assert_eq!(
        (decline.command_id, decline.sequence, decline.payload),
        (DUEL_DECLINE_COMMAND_ID, 215, Vec::new())
    );
    assert_eq!(
        (queue.command_id, queue.sequence),
        (ARENA_QUEUE_COMMAND_ID, 216)
    );
    assert_eq!(
        ArenaQueueCommandPayload::decode(&queue.payload).expect("typed arena queue"),
        ArenaQueueCommandPayload {
            format: ArenaFormat::YumiThree,
        }
    );
    assert_eq!(
        (leave.command_id, leave.sequence, leave.payload),
        (ARENA_LEAVE_COMMAND_ID, 217, Vec::new())
    );
    assert_eq!(
        (augment.command_id, augment.sequence),
        (ARENA_AUGMENT_COMMAND_ID, 218)
    );
    assert_eq!(
        ArenaAugmentCommandPayload::decode(&augment.payload).expect("typed arena augment"),
        ArenaAugmentCommandPayload {
            augment_id: "fiesta_sprint".to_owned(),
        }
    );
    assert_eq!(mapper.next_sequence(), Some(219));

    assert_eq!(
        mapper
            .map_intent(ClientGameplayIntent::PickArenaAugment {
                augment_id: "x".repeat(65),
            })
            .expect_err("arena augment exceeds source UTF-16 bound"),
        ClientInputMappingError::Protocol(ProtocolError::CollectionTooLarge {
            context: "ArenaAugmentCommandPayload.augment_id_utf16_code_units",
            actual: 65,
            maximum: 64,
        })
    );
    assert_eq!(mapper.next_sequence(), Some(219));
}

#[test]
fn trade_intents_preserve_the_source_commands_without_trade_offer_policy() {
    let mut mapper = ClientCommandMapper::new(actor(), 219).expect("valid actor");
    let request = mapper
        .map_intent(ClientGameplayIntent::RequestTrade { target_id: 9_001.5 })
        .expect("trade request");
    let accept = mapper
        .map_intent(ClientGameplayIntent::AcceptTrade)
        .expect("trade accept");
    let confirm = mapper
        .map_intent(ClientGameplayIntent::ConfirmTrade)
        .expect("trade confirm");
    let cancel = mapper
        .map_intent(ClientGameplayIntent::CancelTrade)
        .expect("trade cancel");

    assert_eq!(
        (request.command_id, request.sequence),
        (TRADE_REQUEST_COMMAND_ID, 219)
    );
    assert_eq!(
        TradeRequestCommandPayload::decode(&request.payload).expect("typed trade request"),
        TradeRequestCommandPayload { target_id: 9_001.5 }
    );
    assert_eq!(
        (accept.command_id, accept.sequence, accept.payload),
        (TRADE_ACCEPT_COMMAND_ID, 220, Vec::new())
    );
    assert_eq!(
        (confirm.command_id, confirm.sequence, confirm.payload),
        (TRADE_CONFIRM_COMMAND_ID, 221, Vec::new())
    );
    assert_eq!(
        (cancel.command_id, cancel.sequence, cancel.payload),
        (TRADE_CANCEL_COMMAND_ID, 222, Vec::new())
    );
    assert_eq!(mapper.next_sequence(), Some(223));

    assert!(matches!(
        mapper.map_intent(ClientGameplayIntent::RequestTrade {
            target_id: f64::NAN,
        }),
        Err(ClientInputMappingError::Protocol(
            ProtocolError::NonFinite {
                field: "TradeRequestCommandPayload.target_id",
                ..
            }
        ))
    ));
    assert_eq!(mapper.next_sequence(), Some(223));
}

#[test]
fn vale_cup_intents_preserve_all_source_transport_fields() {
    let mut mapper = ClientCommandMapper::new(actor(), 230).expect("valid actor");
    let queue = mapper
        .map_intent(ClientGameplayIntent::JoinValeCupQueue {
            bracket: ValeCupBracket::Three,
            nation: ValeCupNation::Moon,
            role: ValeCupRole::Sweeper,
            enter_as_guild: true,
        })
        .expect("Vale Cup queue");
    let leave = mapper
        .map_intent(ClientGameplayIntent::LeaveValeCupQueue)
        .expect("Vale Cup queue leave");
    let role = mapper
        .map_intent(ClientGameplayIntent::SetValeCupRole {
            role: ValeCupRole::Keeper,
        })
        .expect("Vale Cup role");
    let ready = mapper
        .map_intent(ClientGameplayIntent::ReadyValeCup)
        .expect("Vale Cup ready");
    let bet = mapper
        .map_intent(ClientGameplayIntent::PlaceValeCupBet {
            side: ValeCupSide::B,
            amount: 44.25,
        })
        .expect("Vale Cup bet");
    let practice = mapper
        .map_intent(ClientGameplayIntent::StartValeCupPractice {
            bracket: ValeCupBracket::Four,
        })
        .expect("Vale Cup practice");

    assert_eq!(
        (queue.command_id, queue.sequence),
        (VALE_CUP_QUEUE_COMMAND_ID, 230)
    );
    assert_eq!(
        ValeCupQueueCommandPayload::decode(&queue.payload).expect("decode Vale Cup queue"),
        ValeCupQueueCommandPayload {
            bracket: ValeCupBracket::Three,
            nation: ValeCupNation::Moon,
            role: ValeCupRole::Sweeper,
            enter_as_guild: true,
        }
    );
    assert_eq!(
        (leave.command_id, leave.sequence, leave.payload),
        (VALE_CUP_LEAVE_COMMAND_ID, 231, Vec::new())
    );
    assert_eq!(
        ValeCupRoleCommandPayload::decode(&role.payload).expect("decode Vale Cup role"),
        ValeCupRoleCommandPayload {
            role: ValeCupRole::Keeper,
        }
    );
    assert_eq!(
        (role.command_id, role.sequence),
        (VALE_CUP_ROLE_COMMAND_ID, 232)
    );
    assert_eq!(
        (ready.command_id, ready.sequence, ready.payload),
        (VALE_CUP_READY_COMMAND_ID, 233, Vec::new())
    );
    assert_eq!(
        (bet.command_id, bet.sequence),
        (VALE_CUP_BET_COMMAND_ID, 234)
    );
    assert_eq!(
        ValeCupBetCommandPayload::decode(&bet.payload).expect("decode Vale Cup bet"),
        ValeCupBetCommandPayload {
            side: ValeCupSide::B,
            amount: 44.25,
        }
    );
    assert_eq!(
        (practice.command_id, practice.sequence),
        (VALE_CUP_PRACTICE_COMMAND_ID, 235)
    );
    assert_eq!(
        ValeCupPracticeCommandPayload::decode(&practice.payload).expect("decode Vale Cup practice"),
        ValeCupPracticeCommandPayload {
            bracket: ValeCupBracket::Four,
        }
    );
    assert_eq!(mapper.next_sequence(), Some(236));

    assert!(matches!(
        mapper.map_intent(ClientGameplayIntent::PlaceValeCupBet {
            side: ValeCupSide::A,
            amount: f64::INFINITY,
        }),
        Err(ClientInputMappingError::Protocol(
            ProtocolError::NonFinite {
                field: "ValeCupBetCommandPayload.amount",
                ..
            }
        ))
    ));
    assert_eq!(mapper.next_sequence(), Some(236));
}

#[test]
fn mail_intents_preserve_the_source_id_without_mail_policy() {
    let mut mapper = ClientCommandMapper::new(actor(), 240).expect("valid actor");
    let take = mapper
        .map_intent(ClientGameplayIntent::TakeMail { mail_id: 15.5 })
        .expect("take mail");
    let delete = mapper
        .map_intent(ClientGameplayIntent::DeleteMail { mail_id: -2.0 })
        .expect("delete mail");
    let read = mapper
        .map_intent(ClientGameplayIntent::MarkMailRead { mail_id: 0.0 })
        .expect("read mail");

    assert_eq!(
        (take.command_id, take.sequence),
        (MAIL_TAKE_COMMAND_ID, 240)
    );
    assert_eq!(
        MailIdCommandPayload::decode(&take.payload, MailAction::Take).expect("decode take"),
        MailIdCommandPayload { mail_id: 15.5 }
    );
    assert_eq!(
        (delete.command_id, delete.sequence),
        (MAIL_DELETE_COMMAND_ID, 241)
    );
    assert_eq!(
        MailIdCommandPayload::decode(&delete.payload, MailAction::Delete).expect("decode delete"),
        MailIdCommandPayload { mail_id: -2.0 }
    );
    assert_eq!(
        (read.command_id, read.sequence),
        (MAIL_READ_COMMAND_ID, 242)
    );
    assert_eq!(
        MailIdCommandPayload::decode(&read.payload, MailAction::MarkRead).expect("decode read"),
        MailIdCommandPayload { mail_id: 0.0 }
    );
    assert_eq!(mapper.next_sequence(), Some(243));

    assert!(matches!(
        mapper.map_intent(ClientGameplayIntent::TakeMail {
            mail_id: f64::NEG_INFINITY,
        }),
        Err(ClientInputMappingError::Protocol(
            ProtocolError::NonFinite {
                field: "MailIdCommandPayload.mail_id",
                ..
            }
        ))
    ));
    assert_eq!(mapper.next_sequence(), Some(243));
}

#[test]
fn mail_send_intent_preserves_source_values_without_local_escrow_policy() {
    let mut mapper = ClientCommandMapper::new(actor(), 243).expect("valid actor");
    let items = vec![
        MailSendAttachment {
            item_id: "wolf_fang".to_owned(),
            count: 1.75,
        },
        MailSendAttachment {
            item_id: "wolf_fang".to_owned(),
            count: -3.0,
        },
    ];
    let command = mapper
        .map_intent(ClientGameplayIntent::SendMail {
            to: "  Raven Receiver  ".to_owned(),
            subject: " subject ".to_owned(),
            body: " body ".to_owned(),
            copper: -2.75,
            items: items.clone(),
        })
        .expect("send mail");

    assert_eq!(
        (command.command_id, command.sequence),
        (MAIL_SEND_COMMAND_ID, 243)
    );
    assert_eq!(
        MailSendCommandPayload::decode(&command.payload).expect("decode send mail"),
        MailSendCommandPayload {
            to: "  Raven Receiver  ".to_owned(),
            subject: " subject ".to_owned(),
            body: " body ".to_owned(),
            copper: -2.75,
            items,
        }
    );
    assert_eq!(mapper.next_sequence(), Some(244));
}

#[test]
fn bank_intents_preserve_source_slot_and_optional_count_without_bank_policy() {
    let mut mapper = ClientCommandMapper::new(actor(), 243).expect("valid actor");
    let deposit = mapper
        .map_intent(ClientGameplayIntent::DepositBank {
            slot: 15.5,
            count: Some(-2.0),
        })
        .expect("bank deposit");
    let withdraw = mapper
        .map_intent(ClientGameplayIntent::WithdrawBank {
            slot: 0.0,
            count: None,
        })
        .expect("bank withdraw");
    let buy = mapper
        .map_intent(ClientGameplayIntent::BuyBankSlots)
        .expect("bank buy slots");

    assert_eq!(
        (deposit.command_id, deposit.sequence),
        (BANK_DEPOSIT_COMMAND_ID, 243)
    );
    assert_eq!(
        BankSlotCommandPayload::decode(&deposit.payload, BankAction::Deposit)
            .expect("decode deposit"),
        BankSlotCommandPayload {
            slot: 15.5,
            count: Some(-2.0),
        }
    );
    assert_eq!(
        (withdraw.command_id, withdraw.sequence),
        (BANK_WITHDRAW_COMMAND_ID, 244)
    );
    assert_eq!(
        BankSlotCommandPayload::decode(&withdraw.payload, BankAction::Withdraw)
            .expect("decode withdraw"),
        BankSlotCommandPayload {
            slot: 0.0,
            count: None,
        }
    );
    assert_eq!(
        (buy.command_id, buy.sequence),
        (BANK_BUY_SLOTS_COMMAND_ID, 245)
    );
    assert!(buy.payload.is_empty());
    assert_eq!(mapper.next_sequence(), Some(246));

    assert!(matches!(
        mapper.map_intent(ClientGameplayIntent::DepositBank {
            slot: 0.0,
            count: Some(f64::NAN),
        }),
        Err(ClientInputMappingError::Protocol(
            ProtocolError::NonFinite {
                field: "BankSlotCommandPayload.count",
                ..
            }
        ))
    ));
    assert_eq!(mapper.next_sequence(), Some(246));
}

#[test]
fn dungeon_finder_intents_preserve_all_current_source_transport_shapes() {
    let mut mapper = ClientCommandMapper::new(actor(), 500).expect("valid actor");
    let roles = mapper
        .map_intent(ClientGameplayIntent::SetDungeonFinderRoles {
            roles: vec![DungeonFinderRole::Tank, DungeonFinderRole::Dps],
        })
        .expect("finder roles");
    let queue = mapper
        .map_intent(ClientGameplayIntent::JoinDungeonFinderQueue {
            activities: vec!["hollow_crypt_normal".to_owned()],
        })
        .expect("finder queue");
    let leave = mapper
        .map_intent(ClientGameplayIntent::LeaveDungeonFinderQueue)
        .expect("finder queue leave");
    let proposal = mapper
        .map_intent(ClientGameplayIntent::RespondDungeonFinderProposal { accept: false })
        .expect("finder proposal");
    let listing = mapper
        .map_intent(ClientGameplayIntent::CreateDungeonFinderListing {
            activity: "hollow_crypt_normal".to_owned(),
            tags: vec![DungeonFinderListingTag::QuestRun],
        })
        .expect("finder listing");
    let close = mapper
        .map_intent(ClientGameplayIntent::CloseDungeonFinderListing)
        .expect("finder listing close");
    let apply = mapper
        .map_intent(ClientGameplayIntent::ApplyToDungeonFinderListing { listing_id: -7.5 })
        .expect("finder apply");
    let cancel = mapper
        .map_intent(ClientGameplayIntent::CancelDungeonFinderApplication)
        .expect("finder apply cancel");
    let response = mapper
        .map_intent(ClientGameplayIntent::RespondToDungeonFinderApplication {
            applicant_id: 9.5,
            accept: true,
        })
        .expect("finder application response");

    assert_eq!(
        (roles.command_id, roles.sequence),
        (DUNGEON_FINDER_ROLES_COMMAND_ID, 500)
    );
    assert_eq!(
        DungeonFinderRolesPayload::decode(&roles.payload).expect("decode finder roles"),
        DungeonFinderRolesPayload {
            roles: vec![DungeonFinderRole::Tank, DungeonFinderRole::Dps],
        }
    );
    assert_eq!(
        (queue.command_id, queue.sequence),
        (DUNGEON_FINDER_QUEUE_COMMAND_ID, 501)
    );
    assert_eq!(
        DungeonFinderActivitiesPayload::decode(&queue.payload).expect("decode finder queue"),
        DungeonFinderActivitiesPayload {
            activities: vec!["hollow_crypt_normal".to_owned()],
        }
    );
    assert_eq!(
        (leave.command_id, leave.sequence, leave.payload),
        (DUNGEON_FINDER_QUEUE_LEAVE_COMMAND_ID, 502, Vec::new())
    );
    assert_eq!(
        (proposal.command_id, proposal.sequence, proposal.payload),
        (DUNGEON_FINDER_PROPOSAL_COMMAND_ID, 503, vec![0])
    );
    assert_eq!(
        (listing.command_id, listing.sequence),
        (DUNGEON_FINDER_LIST_CREATE_COMMAND_ID, 504)
    );
    assert_eq!(
        DungeonFinderListingPayload::decode(&listing.payload).expect("decode finder listing"),
        DungeonFinderListingPayload {
            activity: "hollow_crypt_normal".to_owned(),
            tags: vec![DungeonFinderListingTag::QuestRun],
        }
    );
    assert_eq!(
        (close.command_id, close.sequence, close.payload),
        (DUNGEON_FINDER_LIST_CLOSE_COMMAND_ID, 505, Vec::new())
    );
    assert_eq!(
        (apply.command_id, apply.sequence),
        (DUNGEON_FINDER_APPLY_COMMAND_ID, 506)
    );
    assert_eq!(
        DungeonFinderListingIdPayload::decode(&apply.payload).expect("decode finder apply"),
        DungeonFinderListingIdPayload { listing_id: -7.5 }
    );
    assert_eq!(
        (cancel.command_id, cancel.sequence, cancel.payload),
        (DUNGEON_FINDER_APPLY_CANCEL_COMMAND_ID, 507, Vec::new())
    );
    assert_eq!(
        (response.command_id, response.sequence),
        (DUNGEON_FINDER_APPLICATION_RESPONSE_COMMAND_ID, 508)
    );
    assert_eq!(
        DungeonFinderApplicationResponsePayload::decode(&response.payload)
            .expect("decode finder response"),
        DungeonFinderApplicationResponsePayload {
            applicant_id: 9.5,
            accept: true,
        }
    );
    assert_eq!(mapper.next_sequence(), Some(509));

    assert!(matches!(
        mapper.map_intent(ClientGameplayIntent::JoinDungeonFinderQueue {
            activities: vec!["x".to_owned(); 17],
        }),
        Err(ClientInputMappingError::Protocol(
            ProtocolError::CollectionTooLarge {
                context: "DungeonFinderActivitiesPayload.activities",
                actual: 17,
                maximum: 16,
            }
        ))
    ));
    assert_eq!(mapper.next_sequence(), Some(509));
}

#[test]
fn card_duel_intents_preserve_source_commands_and_card_value() {
    let mut mapper = ClientCommandMapper::new(actor(), 300).expect("valid actor");
    let join = mapper
        .map_intent(ClientGameplayIntent::JoinCardDuelQueue)
        .expect("queue join");
    let leave = mapper
        .map_intent(ClientGameplayIntent::LeaveCardDuelQueue)
        .expect("queue leave");
    let play = mapper
        .map_intent(ClientGameplayIntent::PlayCardInDuel { card_value: 17 })
        .expect("play card");
    let forfeit = mapper
        .map_intent(ClientGameplayIntent::ForfeitCardDuel)
        .expect("forfeit card duel");

    assert_eq!(
        command_descriptor(join.command_id).map(|entry| (entry.name, entry.kind)),
        Some(("card_queue_join", CommandKind::ClientSend))
    );
    assert_eq!(
        command_descriptor(leave.command_id).map(|entry| (entry.name, entry.kind)),
        Some(("card_queue_leave", CommandKind::ClientSend))
    );
    assert_eq!(
        command_descriptor(play.command_id).map(|entry| (entry.name, entry.kind)),
        Some(("play_card", CommandKind::ClientSend))
    );
    assert_eq!(
        command_descriptor(forfeit.command_id).map(|entry| (entry.name, entry.kind)),
        Some(("card_forfeit", CommandKind::ClientSend))
    );
    assert_eq!(
        (join.command_id, join.sequence, join.payload),
        (CARD_QUEUE_JOIN_COMMAND_ID, 300, Vec::new())
    );
    assert_eq!(
        (leave.command_id, leave.sequence, leave.payload),
        (CARD_QUEUE_LEAVE_COMMAND_ID, 301, Vec::new())
    );
    assert_eq!(
        (play.command_id, play.sequence),
        (CARD_PLAY_COMMAND_ID, 302)
    );
    assert_eq!(
        CardPlayCommandPayload::decode(&play.payload).expect("typed card-play payload"),
        CardPlayCommandPayload { card_value: 17 }
    );
    assert_eq!(
        (forfeit.command_id, forfeit.sequence, forfeit.payload),
        (CARD_FORFEIT_COMMAND_ID, 303, Vec::new())
    );
    assert_eq!(mapper.next_sequence(), Some(304));
}

#[test]
fn release_empowered_preserves_the_source_ability_field() {
    let mut mapper = ClientCommandMapper::new(actor(), 304).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::ReleaseEmpoweredAbility {
            ability_id: "glacial_front".to_owned(),
        })
        .expect("release empowered ability");

    assert_eq!(
        command_descriptor(command.command_id).map(|entry| (entry.name, entry.kind)),
        Some(("releaseEmpowered", CommandKind::ClientSend))
    );
    assert_eq!(
        (command.command_id, command.sequence),
        (RELEASE_EMPOWERED_COMMAND_ID, 304)
    );
    assert_eq!(
        ReleaseEmpoweredCommandPayload::decode(&command.payload)
            .expect("typed release-empowered payload"),
        ReleaseEmpoweredCommandPayload {
            ability_id: "glacial_front".to_owned(),
        }
    );
    assert_eq!(mapper.next_sequence(), Some(305));
}

#[test]
fn water_jet_intents_preserve_empty_and_boolean_source_payloads() {
    let mut mapper = ClientCommandMapper::new(actor(), 305).expect("valid actor");
    let manual = mapper
        .map_intent(ClientGameplayIntent::PetWaterJet)
        .expect("manual water jet");
    let automatic_enabled = mapper
        .map_intent(ClientGameplayIntent::SetPetAutoWaterJet { enabled: true })
        .expect("enable automatic water jet");
    let automatic_disabled = mapper
        .map_intent(ClientGameplayIntent::SetPetAutoWaterJet { enabled: false })
        .expect("disable automatic water jet");

    assert_eq!(
        command_descriptor(manual.command_id).map(|entry| (entry.name, entry.kind)),
        Some(("pet_water_jet", CommandKind::ClientSend))
    );
    assert_eq!(
        command_descriptor(automatic_enabled.command_id).map(|entry| (entry.name, entry.kind)),
        Some(("pet_auto_water_jet", CommandKind::ClientSend))
    );
    assert_eq!(
        (manual.command_id, manual.sequence, manual.payload),
        (PET_WATER_JET_COMMAND_ID, 305, Vec::new())
    );
    assert_eq!(
        (automatic_enabled.command_id, automatic_enabled.sequence),
        (PET_AUTO_WATER_JET_COMMAND_ID, 306)
    );
    assert_eq!(
        PetAutoWaterJetCommandPayload::decode(&automatic_enabled.payload)
            .expect("typed automatic water-jet payload"),
        PetAutoWaterJetCommandPayload { enabled: true }
    );
    assert_eq!(
        PetAutoWaterJetCommandPayload::decode(&automatic_disabled.payload)
            .expect("typed automatic water-jet payload"),
        PetAutoWaterJetCommandPayload { enabled: false }
    );
    assert_eq!(mapper.next_sequence(), Some(308));
    assert_eq!(
        PetAutoWaterJetCommandPayload::decode(&[2]),
        Err(ProtocolError::InvalidBoolean(2))
    );
}

#[test]
fn remaining_pet_intents_preserve_current_source_payloads() {
    let mut mapper = ClientCommandMapper::new(actor(), 308).expect("valid actor");
    let abandon = mapper
        .map_intent(ClientGameplayIntent::AbandonPet)
        .expect("abandon pet");
    let rename = mapper
        .map_intent(ClientGameplayIntent::RenamePet {
            name: "Frosty".to_owned(),
        })
        .expect("rename pet");
    let revive = mapper
        .map_intent(ClientGameplayIntent::RevivePet)
        .expect("revive pet");
    let attack = mapper
        .map_intent(ClientGameplayIntent::PetAttack)
        .expect("pet attack");
    let taunt = mapper
        .map_intent(ClientGameplayIntent::PetTaunt)
        .expect("pet taunt");
    let auto_taunt = mapper
        .map_intent(ClientGameplayIntent::SetPetAutoTaunt { enabled: true })
        .expect("enable automatic taunt");
    let feed = mapper
        .map_intent(ClientGameplayIntent::FeedPet {
            item_id: "pet_treat".to_owned(),
        })
        .expect("feed pet");
    let heal = mapper
        .map_intent(ClientGameplayIntent::HealPet)
        .expect("heal pet");
    let mode = mapper
        .map_intent(ClientGameplayIntent::SetPetMode {
            mode: "defensive".to_owned(),
        })
        .expect("set pet mode");

    assert_eq!(
        (abandon.command_id, abandon.sequence, abandon.payload),
        (PET_ABANDON_COMMAND_ID, 308, Vec::new())
    );
    assert_eq!(
        PetRenameCommandPayload::decode(&rename.payload).expect("typed pet-name payload"),
        PetRenameCommandPayload {
            name: "Frosty".to_owned(),
        }
    );
    assert_eq!(
        (revive.command_id, revive.sequence, revive.payload),
        (PET_REVIVE_COMMAND_ID, 310, Vec::new())
    );
    assert_eq!(
        (attack.command_id, attack.sequence, attack.payload),
        (PET_ATTACK_COMMAND_ID, 311, Vec::new())
    );
    assert_eq!(
        (taunt.command_id, taunt.sequence, taunt.payload),
        (PET_TAUNT_COMMAND_ID, 312, Vec::new())
    );
    assert_eq!(
        PetAutoTauntCommandPayload::decode(&auto_taunt.payload)
            .expect("typed automatic-taunt payload"),
        PetAutoTauntCommandPayload { enabled: true }
    );
    assert_eq!(
        PetFeedCommandPayload::decode(&feed.payload).expect("typed pet-feed payload"),
        PetFeedCommandPayload {
            item_id: "pet_treat".to_owned(),
        }
    );
    assert_eq!(
        (heal.command_id, heal.sequence, heal.payload),
        (PET_HEAL_COMMAND_ID, 315, Vec::new())
    );
    assert_eq!(
        PetModeCommandPayload::decode(&mode.payload).expect("typed pet-mode payload"),
        PetModeCommandPayload {
            mode: "defensive".to_owned(),
        }
    );
    assert_eq!(
        (rename.command_id, rename.sequence),
        (PET_RENAME_COMMAND_ID, 309)
    );
    assert_eq!(
        (auto_taunt.command_id, auto_taunt.sequence),
        (PET_AUTO_TAUNT_COMMAND_ID, 313)
    );
    assert_eq!((feed.command_id, feed.sequence), (PET_FEED_COMMAND_ID, 314));
    assert_eq!((mode.command_id, mode.sequence), (PET_MODE_COMMAND_ID, 316));
    assert_eq!(mapper.next_sequence(), Some(317));
    assert_eq!(
        PetAutoTauntCommandPayload::decode(&[2]),
        Err(ProtocolError::InvalidBoolean(2))
    );
}

#[test]
fn social_intents_preserve_current_source_payloads() {
    let mut mapper = ClientCommandMapper::new(actor(), 317).expect("valid actor");
    let friend_add = mapper
        .map_intent(ClientGameplayIntent::AddFriend {
            name: "Ari".to_owned(),
        })
        .expect("add friend");
    let friend_remove = mapper
        .map_intent(ClientGameplayIntent::RemoveFriend {
            name: "Ari".to_owned(),
        })
        .expect("remove friend");
    let block_add = mapper
        .map_intent(ClientGameplayIntent::AddBlock {
            name: "Bex".to_owned(),
        })
        .expect("add block");
    let block_remove = mapper
        .map_intent(ClientGameplayIntent::RemoveBlock {
            name: "Bex".to_owned(),
        })
        .expect("remove block");
    let guild_create = mapper
        .map_intent(ClientGameplayIntent::CreateGuild {
            name: "Zircon".to_owned(),
        })
        .expect("create guild");
    let guild_invite = mapper
        .map_intent(ClientGameplayIntent::InviteToGuild {
            name: "Cyd".to_owned(),
        })
        .expect("invite to guild");
    let guild_accept = mapper
        .map_intent(ClientGameplayIntent::AcceptGuildInvite)
        .expect("accept guild invite");
    let guild_decline = mapper
        .map_intent(ClientGameplayIntent::DeclineGuildInvite)
        .expect("decline guild invite");
    let guild_leave = mapper
        .map_intent(ClientGameplayIntent::LeaveGuild)
        .expect("leave guild");
    let guild_kick = mapper
        .map_intent(ClientGameplayIntent::KickGuildMember {
            name: "Dee".to_owned(),
        })
        .expect("kick guild member");
    let guild_promote = mapper
        .map_intent(ClientGameplayIntent::PromoteGuildMember {
            name: "Eli".to_owned(),
        })
        .expect("promote guild member");
    let guild_demote = mapper
        .map_intent(ClientGameplayIntent::DemoteGuildMember {
            name: "Fay".to_owned(),
        })
        .expect("demote guild member");
    let guild_transfer = mapper
        .map_intent(ClientGameplayIntent::TransferGuildLeadership {
            name: "Gia".to_owned(),
        })
        .expect("transfer guild leadership");
    let guild_disband = mapper
        .map_intent(ClientGameplayIntent::DisbandGuild)
        .expect("disband guild");
    let guild_event_create = mapper
        .map_intent(ClientGameplayIntent::CreateGuildEvent {
            day: "2026-08-10".to_owned(),
            hour: Some(21.75),
            title: "Vault run".to_owned(),
            note: "Bring keys".to_owned(),
        })
        .expect("create guild event");
    let guild_event_remove = mapper
        .map_intent(ClientGameplayIntent::RemoveGuildEvent { event_id: 42 })
        .expect("remove guild event");
    let ignore_add = mapper
        .map_intent(ClientGameplayIntent::AddIgnore {
            name: "Hal".to_owned(),
        })
        .expect("add ignore");
    let ignore_remove = mapper
        .map_intent(ClientGameplayIntent::RemoveIgnore {
            name: "Hal".to_owned(),
        })
        .expect("remove ignore");

    assert_eq!(
        (friend_add.command_id, friend_add.sequence),
        (FRIEND_ADD_COMMAND_ID, 317)
    );
    assert_eq!(
        SocialNameCommandPayload::decode(FRIEND_ADD_COMMAND_ID, &friend_add.payload)
            .expect("typed friend name payload"),
        SocialNameCommandPayload {
            name: "Ari".to_owned(),
        }
    );
    assert_eq!(
        (friend_remove.command_id, friend_remove.sequence),
        (FRIEND_REMOVE_COMMAND_ID, 318)
    );
    assert_eq!(
        (block_add.command_id, block_add.sequence),
        (BLOCK_ADD_COMMAND_ID, 319)
    );
    assert_eq!(
        (block_remove.command_id, block_remove.sequence),
        (BLOCK_REMOVE_COMMAND_ID, 320)
    );
    assert_eq!(
        (guild_create.command_id, guild_create.sequence),
        (GUILD_CREATE_COMMAND_ID, 321)
    );
    assert_eq!(
        SocialNameCommandPayload::decode(GUILD_INVITE_COMMAND_ID, &guild_invite.payload)
            .expect("typed guild invite payload"),
        SocialNameCommandPayload {
            name: "Cyd".to_owned(),
        }
    );
    assert_eq!(
        (guild_invite.command_id, guild_invite.sequence),
        (GUILD_INVITE_COMMAND_ID, 322)
    );
    assert_eq!(
        (
            guild_accept.command_id,
            guild_accept.sequence,
            guild_accept.payload
        ),
        (79, 323, Vec::new())
    );
    assert_eq!(
        (
            guild_decline.command_id,
            guild_decline.sequence,
            guild_decline.payload
        ),
        (80, 324, Vec::new())
    );
    assert_eq!(
        (
            guild_leave.command_id,
            guild_leave.sequence,
            guild_leave.payload
        ),
        (81, 325, Vec::new())
    );
    assert_eq!(
        (guild_kick.command_id, guild_kick.sequence),
        (GUILD_KICK_COMMAND_ID, 326)
    );
    assert_eq!(
        (guild_promote.command_id, guild_promote.sequence),
        (GUILD_PROMOTE_COMMAND_ID, 327)
    );
    assert_eq!(
        (guild_demote.command_id, guild_demote.sequence),
        (GUILD_DEMOTE_COMMAND_ID, 328)
    );
    assert_eq!(
        (guild_transfer.command_id, guild_transfer.sequence),
        (GUILD_TRANSFER_COMMAND_ID, 329)
    );
    assert_eq!(
        (
            guild_disband.command_id,
            guild_disband.sequence,
            guild_disband.payload
        ),
        (86, 330, Vec::new())
    );
    assert_eq!(
        (guild_event_create.command_id, guild_event_create.sequence),
        (GUILD_EVENT_CREATE_COMMAND_ID, 331)
    );
    assert_eq!(
        GuildEventCreateCommandPayload::decode(&guild_event_create.payload)
            .expect("typed guild event payload"),
        GuildEventCreateCommandPayload {
            day: "2026-08-10".to_owned(),
            hour: Some(21.75),
            title: "Vault run".to_owned(),
            note: "Bring keys".to_owned(),
        }
    );
    assert_eq!(
        (guild_event_remove.command_id, guild_event_remove.sequence),
        (GUILD_EVENT_REMOVE_COMMAND_ID, 332)
    );
    assert_eq!(
        GuildEventRemoveCommandPayload::decode(&guild_event_remove.payload)
            .expect("typed guild event ID payload"),
        GuildEventRemoveCommandPayload { event_id: 42 }
    );
    assert_eq!(
        (ignore_add.command_id, ignore_add.sequence),
        (IGNORE_ADD_COMMAND_ID, 333)
    );
    assert_eq!(
        SocialNameCommandPayload::decode(IGNORE_REMOVE_COMMAND_ID, &ignore_remove.payload)
            .expect("typed ignore name payload"),
        SocialNameCommandPayload {
            name: "Hal".to_owned(),
        }
    );
    assert_eq!(
        (ignore_remove.command_id, ignore_remove.sequence),
        (IGNORE_REMOVE_COMMAND_ID, 334)
    );
    assert_eq!(mapper.next_sequence(), Some(335));
}

#[test]
fn cancel_aura_uses_the_source_aura_field_and_preserves_sequence_on_rejection() {
    let mut mapper = ClientCommandMapper::new(actor(), 23).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::CancelAura {
            aura_id: "ice_armor".to_owned(),
        })
        .expect("cancel aura");
    assert_eq!(
        (command.command_id, command.sequence),
        (CANCEL_AURA_COMMAND_ID, 23)
    );
    assert_eq!(
        CancelAuraCommandPayload::decode(&command.payload).expect("typed cancel-aura payload"),
        CancelAuraCommandPayload {
            aura_id: "ice_armor".to_owned(),
        }
    );

    assert_eq!(
        mapper
            .map_intent(ClientGameplayIntent::CancelAura {
                aura_id: "x".repeat(257),
            })
            .expect_err("overlong aura id"),
        ClientInputMappingError::Protocol(ProtocolError::CollectionTooLarge {
            context: "CancelAuraCommandPayload.aura_id",
            actual: 257,
            maximum: 256,
        })
    );
    assert_eq!(mapper.next_sequence(), Some(24));
}

#[test]
fn skin_intents_preserve_catalog_specific_source_bounds() {
    let mut mapper = ClientCommandMapper::new(actor(), 24).expect("valid actor");
    let class = mapper
        .map_intent(ClientGameplayIntent::ChangeSkin {
            catalog: SkinCatalog::Class,
            skin_index: 7,
        })
        .expect("class skin");
    let mech = mapper
        .map_intent(ClientGameplayIntent::ChangeSkin {
            catalog: SkinCatalog::Mech,
            skin_index: 0,
        })
        .expect("mech skin");

    assert_eq!(
        (class.command_id, class.sequence),
        (CHANGE_SKIN_COMMAND_ID, 24)
    );
    assert_eq!(
        (mech.command_id, mech.sequence),
        (CHANGE_SKIN_COMMAND_ID, 25)
    );
    assert_eq!(
        ChangeSkinCommandPayload::decode(&class.payload).expect("decode class skin"),
        ChangeSkinCommandPayload {
            catalog: SkinCatalog::Class,
            skin_index: 7,
        }
    );
    assert_eq!(
        ChangeSkinCommandPayload::decode(&mech.payload).expect("decode mech skin"),
        ChangeSkinCommandPayload {
            catalog: SkinCatalog::Mech,
            skin_index: 0,
        }
    );
    assert_eq!(mapper.next_sequence(), Some(26));

    assert_eq!(
        mapper
            .map_intent(ClientGameplayIntent::ChangeSkin {
                catalog: SkinCatalog::Class,
                skin_index: 8,
            })
            .expect_err("class skin eight is outside the source range"),
        ClientInputMappingError::Protocol(ProtocolError::InvalidClassSkinIndex(8))
    );
    assert_eq!(mapper.next_sequence(), Some(26));
}

#[test]
fn accept_quest_uses_the_source_quest_field_and_preserves_sequence_on_rejection() {
    let mut mapper = ClientCommandMapper::new(actor(), 25).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::AcceptQuest {
            quest_id: "eastbrook_welcome".to_owned(),
        })
        .expect("accept quest");
    assert_eq!(
        (command.command_id, command.sequence),
        (ACCEPT_QUEST_COMMAND_ID, 25)
    );
    assert_eq!(
        AcceptQuestCommandPayload::decode(&command.payload).expect("typed accept payload"),
        AcceptQuestCommandPayload {
            quest_id: "eastbrook_welcome".to_owned(),
            selection: None,
        }
    );

    assert_eq!(
        mapper
            .map_intent(ClientGameplayIntent::AcceptQuest {
                quest_id: "x".repeat(257),
            })
            .expect_err("overlong quest id"),
        ClientInputMappingError::Protocol(ProtocolError::CollectionTooLarge {
            context: "AcceptQuestCommandPayload.quest_id",
            actual: 257,
            maximum: 256,
        })
    );
    assert_eq!(mapper.next_sequence(), Some(26));
}

#[test]
fn linked_quest_acceptance_maps_source_fields_and_preserves_sequence_on_rejection() {
    let mut mapper = ClientCommandMapper::new(actor(), 26).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::AcceptLinkedQuest {
            quest_id: "q_wolves".to_owned(),
            sharer_pid: 2.0,
        })
        .expect("linked quest acceptance");
    assert_eq!(
        (command.command_id, command.sequence),
        (LINKED_QUEST_ACCEPT_COMMAND_ID, 26)
    );
    assert_eq!(
        LinkedQuestAcceptancePayload::decode(&command.payload).expect("typed linked quest payload"),
        LinkedQuestAcceptancePayload {
            quest_id: "q_wolves".to_owned(),
            sharer_pid: 2.0,
        }
    );

    assert_eq!(
        mapper
            .map_intent(ClientGameplayIntent::AcceptLinkedQuest {
                quest_id: "q_wolves".to_owned(),
                sharer_pid: 0.0,
            })
            .expect_err("zero sharer id"),
        ClientInputMappingError::Protocol(ProtocolError::InvalidEntityId {
            context: "LinkedQuestAcceptancePayload.sharer_pid",
        })
    );
    assert_eq!(mapper.next_sequence(), Some(27));
}

#[test]
fn equipment_intents_map_optional_slots_and_preserve_sequence_on_rejection() {
    let mut mapper = ClientCommandMapper::new(actor(), 27).expect("valid actor");
    let equip = mapper
        .map_intent(ClientGameplayIntent::EquipItem {
            item_id: "cryptbone_helm".to_owned(),
            slot: Some(EquipmentSlot::Helmet),
        })
        .expect("aimed equip");
    assert_eq!(
        (equip.command_id, equip.sequence),
        (EQUIP_ITEM_COMMAND_ID, 27)
    );
    assert_eq!(
        EquipItemPayload::decode(&equip.payload).expect("typed equip payload"),
        EquipItemPayload {
            item_id: "cryptbone_helm".to_owned(),
            slot: Some(EquipmentSlot::Helmet),
        }
    );

    let unequip = mapper
        .map_intent(ClientGameplayIntent::UnequipItem {
            slot: EquipmentSlot::Helmet,
        })
        .expect("unequip");
    assert_eq!(
        (unequip.command_id, unequip.sequence),
        (UNEQUIP_ITEM_COMMAND_ID, 28)
    );
    assert_eq!(
        UnequipItemPayload::decode(&unequip.payload).expect("typed unequip payload"),
        UnequipItemPayload {
            slot: EquipmentSlot::Helmet,
        }
    );

    assert_eq!(
        mapper
            .map_intent(ClientGameplayIntent::EquipItem {
                item_id: "x".repeat(257),
                slot: None,
            })
            .expect_err("overlong item id"),
        ClientInputMappingError::Protocol(ProtocolError::CollectionTooLarge {
            context: "EquipItemPayload.item_id",
            actual: 257,
            maximum: 256,
        })
    );
    assert_eq!(mapper.next_sequence(), Some(29));
}

#[test]
fn telemetry_intent_maps_numeric_fields_and_preserves_sequence_on_rejection() {
    let mut mapper = ClientCommandMapper::new(actor(), 29).expect("valid actor");
    let data = BTreeMap::from([("fps".to_owned(), 60.0), ("ping".to_owned(), 24.0)]);
    let command = mapper
        .map_intent(ClientGameplayIntent::ReportTelemetry {
            kind: "render".to_owned(),
            data: data.clone(),
        })
        .expect("telemetry command");
    assert_eq!(
        (command.command_id, command.sequence),
        (TELEMETRY_COMMAND_ID, 29)
    );
    assert_eq!(
        TelemetryPayload::decode(&command.payload).expect("typed telemetry payload"),
        TelemetryPayload {
            kind: "render".to_owned(),
            data,
        }
    );

    assert!(matches!(
        mapper.map_intent(ClientGameplayIntent::ReportTelemetry {
            kind: "render".to_owned(),
            data: BTreeMap::from([("fps".to_owned(), f64::INFINITY)]),
        }),
        Err(ClientInputMappingError::Protocol(
            ProtocolError::NonFinite { .. }
        ))
    ));
    assert_eq!(mapper.next_sequence(), Some(30));
}

#[test]
fn turn_in_quest_uses_the_source_quest_field_and_preserves_sequence_on_rejection() {
    let mut mapper = ClientCommandMapper::new(actor(), 27).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::TurnInQuest {
            quest_id: "eastbrook_welcome".to_owned(),
        })
        .expect("turn in quest");
    assert_eq!(
        (command.command_id, command.sequence),
        (TURN_IN_QUEST_COMMAND_ID, 27)
    );
    assert_eq!(
        TurnInQuestCommandPayload::decode(&command.payload).expect("typed turn-in payload"),
        TurnInQuestCommandPayload {
            quest_id: "eastbrook_welcome".to_owned(),
        }
    );

    assert_eq!(
        mapper
            .map_intent(ClientGameplayIntent::TurnInQuest {
                quest_id: "x".repeat(257),
            })
            .expect_err("overlong quest id"),
        ClientInputMappingError::Protocol(ProtocolError::CollectionTooLarge {
            context: "TurnInQuestCommandPayload.quest_id",
            actual: 257,
            maximum: 256,
        })
    );
    assert_eq!(mapper.next_sequence(), Some(28));
}

#[test]
fn target_selection_uses_the_typed_payload_and_invalid_ids_do_not_consume_sequence() {
    let mut mapper = ClientCommandMapper::new(actor(), 9).expect("valid actor");
    let invalid = mapper
        .map(event(
            ClientInputDevice::Touch,
            ClientGameplayIntent::SetTarget { target_id: Some(0) },
        ))
        .expect_err("entity zero is the clear sentinel, not a target");
    assert_eq!(
        invalid,
        ClientInputMappingError::Protocol(ProtocolError::InvalidEntityId {
            context: "TargetCommandPayload.target_id",
        })
    );
    assert_eq!(mapper.next_sequence(), Some(9));

    let set = mapper
        .map(event(
            ClientInputDevice::KeyboardMouse,
            ClientGameplayIntent::SetTarget {
                target_id: Some(9001),
            },
        ))
        .expect("set target");
    assert_eq!((set.command_id, set.sequence), (TARGET_COMMAND_ID, 9));
    assert_eq!(
        TargetCommandPayload::decode(&set.payload).expect("typed target payload"),
        TargetCommandPayload {
            target_id: Some(9001),
        }
    );

    let clear = mapper
        .map(event(
            ClientInputDevice::Gamepad,
            ClientGameplayIntent::SetTarget { target_id: None },
        ))
        .expect("clear target");
    assert_eq!(clear.sequence, 10);
    assert_eq!(
        TargetCommandPayload::decode(&clear.payload).expect("typed clear payload"),
        TargetCommandPayload { target_id: None }
    );
}

#[test]
fn cycle_target_maps_only_to_client_send_commands() {
    let mut mapper = ClientCommandMapper::new(actor(), 0).expect("valid actor");
    let hostile = mapper
        .map(event(
            ClientInputDevice::KeyboardMouse,
            ClientGameplayIntent::CycleTarget { friendly: false },
        ))
        .expect("hostile cycle");
    let friendly = mapper
        .map(event(
            ClientInputDevice::Gamepad,
            ClientGameplayIntent::CycleTarget { friendly: true },
        ))
        .expect("friendly cycle");

    let hostile_descriptor = command_descriptor(hostile.command_id).expect("tab command");
    let friendly_descriptor =
        command_descriptor(friendly.command_id).expect("friendly tab command");
    assert_eq!(
        (hostile_descriptor.name, hostile_descriptor.kind),
        ("tab", CommandKind::ClientSend)
    );
    assert_eq!(
        (friendly_descriptor.name, friendly_descriptor.kind),
        ("tabFriendly", CommandKind::ClientSend)
    );
    assert!(hostile.payload.is_empty());
    assert!(friendly.payload.is_empty());
    assert_ne!(hostile.command_id, 6, "targetNearest is dispatch-only");
}

#[test]
fn target_nearest_friendly_maps_to_the_source_empty_client_command() {
    let mut mapper = ClientCommandMapper::new(actor(), 81).expect("valid actor");

    let command = mapper
        .map(event(
            ClientInputDevice::Touch,
            ClientGameplayIntent::TargetNearestFriendly,
        ))
        .expect("nearest friendly command");

    let descriptor = command_descriptor(command.command_id).expect("nearest friendly command");
    assert_eq!(
        (descriptor.name, descriptor.kind, command.sequence),
        ("targetNearestFriendly", CommandKind::ClientSend, 81)
    );
    assert_eq!(command.command_id, TARGET_NEAREST_FRIENDLY_COMMAND_ID);
    assert!(command.payload.is_empty());
    assert_eq!(mapper.next_sequence(), Some(82));
}

#[test]
fn weapon_stow_maps_to_the_source_empty_client_command() {
    let mut mapper = ClientCommandMapper::new(actor(), 82).expect("valid actor");

    let command = mapper
        .map(event(
            ClientInputDevice::KeyboardMouse,
            ClientGameplayIntent::ToggleWeaponStow,
        ))
        .expect("weapon stow command");

    let descriptor = command_descriptor(command.command_id).expect("weapon stow command");
    assert_eq!(
        (descriptor.name, descriptor.kind, command.sequence),
        ("stow_weapon", CommandKind::ClientSend, 82)
    );
    assert_eq!(command.command_id, WEAPON_STOW_COMMAND_ID);
    assert!(command.payload.is_empty());
    assert_eq!(mapper.next_sequence(), Some(83));
}

#[test]
fn interact_maps_to_the_source_empty_client_command() {
    let mut mapper = ClientCommandMapper::new(actor(), 73).expect("valid actor");

    let command = mapper
        .map(event(
            ClientInputDevice::Touch,
            ClientGameplayIntent::Interact,
        ))
        .expect("interact command");

    let descriptor = command_descriptor(command.command_id).expect("interact command");
    assert_eq!(
        (descriptor.name, descriptor.kind, command.sequence),
        ("interact", CommandKind::ClientSend, 73)
    );
    assert_eq!(command.command_id, INTERACT_COMMAND_ID);
    assert!(command.payload.is_empty());
    assert_eq!(mapper.next_sequence(), Some(74));
}

#[test]
fn attack_edges_share_one_monotonic_sequence_across_devices() {
    let mut mapper = ClientCommandMapper::new(actor(), 71).expect("valid actor");
    let start = mapper
        .map(event(
            ClientInputDevice::Touch,
            ClientGameplayIntent::SetAttacking { attacking: true },
        ))
        .expect("start attack");
    let stop = mapper
        .map(event(
            ClientInputDevice::KeyboardMouse,
            ClientGameplayIntent::SetAttacking { attacking: false },
        ))
        .expect("stop attack");

    assert_eq!((start.command_id, start.sequence), (ATTACK_COMMAND_ID, 71));
    assert_eq!(
        (stop.command_id, stop.sequence),
        (STOP_ATTACK_COMMAND_ID, 72)
    );
    assert_eq!(start.actor, actor());
    assert_eq!(stop.actor, actor());
}

#[test]
fn actor_zero_and_sequence_wrap_are_rejected() {
    assert!(matches!(
        ClientCommandMapper::new(
            EntityRef {
                id: 0,
                generation: 0,
            },
            0,
        ),
        Err(ClientInputMappingError::InvalidActor)
    ));

    let mut mapper = ClientCommandMapper::new(actor(), u32::MAX).expect("valid actor");
    let last = mapper
        .map(event(
            ClientInputDevice::Gamepad,
            ClientGameplayIntent::SetAttacking { attacking: true },
        ))
        .expect("last sequence remains representable");
    assert_eq!(last.sequence, u32::MAX);
    assert_eq!(mapper.next_sequence(), None);
    assert_eq!(
        mapper
            .map(event(
                ClientInputDevice::Gamepad,
                ClientGameplayIntent::SetAttacking { attacking: false },
            ))
            .expect_err("sequence must never wrap"),
        ClientInputMappingError::SequenceExhausted
    );
}

#[test]
fn inventory_and_quest_intents_share_the_authoritative_sequence() {
    let mut mapper = ClientCommandMapper::new(actor(), 300).expect("valid actor");
    let abandon = mapper
        .map_intent(ClientGameplayIntent::AbandonQuest {
            quest_id: "q_boars".to_owned(),
        })
        .expect("abandon quest");
    let use_item = mapper
        .map_intent(ClientGameplayIntent::UseItem {
            item_id: "minor_healing_potion".to_owned(),
        })
        .expect("use item");
    let discard = mapper
        .map_intent(ClientGameplayIntent::DiscardItem {
            item_id: "wolf_fang".to_owned(),
            count: Some(2),
        })
        .expect("discard item");
    let equip_bag = mapper
        .map_intent(ClientGameplayIntent::EquipBag {
            item_id: "wolfhide_satchel".to_owned(),
            socket: None,
        })
        .expect("equip bag");
    let unequip_bag = mapper
        .map_intent(ClientGameplayIntent::UnequipBag { socket: 3 })
        .expect("unequip bag");

    assert_eq!(
        [
            abandon.command_id,
            use_item.command_id,
            discard.command_id,
            equip_bag.command_id,
            unequip_bag.command_id,
        ],
        [
            ABANDON_QUEST_COMMAND_ID,
            USE_ITEM_COMMAND_ID,
            DISCARD_ITEM_COMMAND_ID,
            EQUIP_BAG_COMMAND_ID,
            UNEQUIP_BAG_COMMAND_ID,
        ]
    );
    assert_eq!(
        [
            abandon.sequence,
            use_item.sequence,
            discard.sequence,
            equip_bag.sequence,
            unequip_bag.sequence,
        ],
        [300, 301, 302, 303, 304]
    );
    assert_eq!(
        AbandonQuestCommandPayload::decode(&abandon.payload).expect("typed abandon"),
        AbandonQuestCommandPayload {
            quest_id: "q_boars".to_owned(),
        }
    );
    assert_eq!(
        UseItemCommandPayload::decode(&use_item.payload).expect("typed use"),
        UseItemCommandPayload {
            item_id: "minor_healing_potion".to_owned(),
        }
    );
    assert_eq!(
        DiscardItemCommandPayload::decode(&discard.payload).expect("typed discard"),
        DiscardItemCommandPayload {
            item_id: "wolf_fang".to_owned(),
            count: Some(2),
        }
    );
    assert_eq!(
        EquipBagCommandPayload::decode(&equip_bag.payload).expect("typed equip bag"),
        EquipBagCommandPayload {
            item_id: "wolfhide_satchel".to_owned(),
            socket: None,
        }
    );
    assert_eq!(
        UnequipBagCommandPayload::decode(&unequip_bag.payload).expect("typed unequip bag"),
        UnequipBagCommandPayload { socket: 3 }
    );
}

#[test]
fn lockpick_intents_share_the_authoritative_sequence_and_typed_wire_contracts() {
    let mut mapper = ClientCommandMapper::new(actor(), 600).expect("valid actor");
    let engage = mapper
        .map_intent(ClientGameplayIntent::LockpickEngage {
            object_id: 700,
            ante: 3,
        })
        .expect("lockpick engage");
    let action = mapper
        .map_intent(ClientGameplayIntent::LockpickAction {
            session_id: Some("lk_7".to_owned()),
            action: LockpickAction::Steady,
        })
        .expect("lockpick action");
    let abort = mapper
        .map_intent(ClientGameplayIntent::LockpickAbort { session_id: None })
        .expect("lockpick abort");

    assert_eq!(
        [engage.command_id, action.command_id, abort.command_id],
        [
            LOCKPICK_ENGAGE_COMMAND_ID,
            LOCKPICK_ACTION_COMMAND_ID,
            LOCKPICK_ABORT_COMMAND_ID,
        ]
    );
    assert_eq!(
        [engage.sequence, action.sequence, abort.sequence],
        [600, 601, 602]
    );
    assert_eq!(
        LockpickEngageCommandPayload::decode(&engage.payload).expect("decode engage"),
        LockpickEngageCommandPayload {
            object_id: 700,
            ante: 3,
        }
    );
    assert_eq!(
        LockpickActionCommandPayload::decode(&action.payload).expect("decode action"),
        LockpickActionCommandPayload {
            session_id: Some("lk_7".to_owned()),
            action: LockpickAction::Steady,
        }
    );
    assert_eq!(
        LockpickAbortCommandPayload::decode(&abort.payload).expect("decode abort"),
        LockpickAbortCommandPayload { session_id: None }
    );
}

#[test]
fn invalid_identifier_does_not_consume_the_sequence() {
    let mut mapper = ClientCommandMapper::new(actor(), 91).expect("valid actor");
    assert_eq!(
        mapper
            .map_intent(ClientGameplayIntent::UseItem {
                item_id: "x".repeat(257),
            })
            .expect_err("overlong item id"),
        ClientInputMappingError::Protocol(ProtocolError::CollectionTooLarge {
            context: "UseItemCommandPayload.item_id",
            actual: 257,
            maximum: 256,
        })
    );
    assert_eq!(mapper.next_sequence(), Some(91));
}

#[test]
fn invalid_lockpick_ante_does_not_consume_the_sequence() {
    let mut mapper = ClientCommandMapper::new(actor(), 92).expect("valid actor");
    assert_eq!(
        mapper
            .map_intent(ClientGameplayIntent::LockpickEngage {
                object_id: 700,
                ante: 4,
            })
            .expect_err("ante four is not source-valid"),
        ClientInputMappingError::Protocol(ProtocolError::InvalidLockpickAnte(4))
    );
    assert_eq!(mapper.next_sequence(), Some(92));
}

#[test]
fn talent_row_intents_map_catalog_ids_and_preserve_the_clear_sentinel() {
    let mut mapper = ClientCommandMapper::new(actor(), 700).expect("valid actor");
    let selected = mapper
        .map_intent(ClientGameplayIntent::SelectTalentRow {
            level: 5,
            option_id: Some("war_row_double_charge".to_owned()),
        })
        .expect("select talent row");
    let cleared = mapper
        .map_intent(ClientGameplayIntent::SelectTalentRow {
            level: 5,
            option_id: None,
        })
        .expect("clear talent row");

    assert_eq!(
        (selected.command_id, selected.sequence),
        (SELECT_TALENT_ROW_COMMAND_ID, 700)
    );
    assert_eq!(
        SelectTalentRowCommandPayload::decode(&selected.payload).expect("decode selected row"),
        SelectTalentRowCommandPayload {
            level: 5,
            option_code: talent_option_code("war_row_double_charge").expect("option code"),
        }
    );
    assert_eq!(
        SelectTalentRowCommandPayload::decode(&cleared.payload).expect("decode clear row"),
        SelectTalentRowCommandPayload {
            level: 5,
            option_code: 0,
        }
    );
    assert_eq!(mapper.next_sequence(), Some(702));

    assert_eq!(
        mapper
            .map_intent(ClientGameplayIntent::SelectTalentRow {
                level: 5,
                option_id: Some("missing_talent".to_owned()),
            })
            .expect_err("unknown source option is rejected before transport"),
        ClientInputMappingError::InvalidTalentOptionId {
            option_id: "missing_talent".to_owned(),
        }
    );
    assert_eq!(mapper.next_sequence(), Some(702));
}

#[test]
fn respec_intent_maps_to_the_source_empty_client_command() {
    let mut mapper = ClientCommandMapper::new(actor(), 720).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::Respec)
        .expect("respec command");

    assert_eq!(
        (command.command_id, command.sequence),
        (RESPEC_COMMAND_ID, 720)
    );
    assert!(command.payload.is_empty());
    assert_eq!(mapper.next_sequence(), Some(721));
}

#[test]
fn prestige_intent_maps_to_the_source_empty_client_command() {
    let mut mapper = ClientCommandMapper::new(actor(), 725).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::Prestige)
        .expect("prestige command");

    assert_eq!((command.command_id, command.sequence), (94, 725));
    assert!(command.payload.is_empty());
    assert_eq!(mapper.next_sequence(), Some(726));
}

#[test]
fn apply_talents_intent_projects_the_strict_source_allocation_shape() {
    let mut mapper = ClientCommandMapper::new(actor(), 730).expect("valid actor");
    let command = mapper
        .map_intent(ClientGameplayIntent::ApplyTalents {
            player_class_id: "warrior".to_owned(),
            spec_id: Some("arms".to_owned()),
            row_option_ids: [
                Some("war_row_double_charge".to_owned()),
                None,
                None,
                None,
                None,
                None,
            ],
        })
        .expect("apply talent allocation");

    assert_eq!(
        (command.command_id, command.sequence),
        (APPLY_TALENTS_COMMAND_ID, 730)
    );
    assert_eq!(
        ApplyTalentsCommandPayload::decode(&command.payload).expect("decode allocation"),
        ApplyTalentsCommandPayload {
            spec_code: talent_spec_code("warrior", "arms").expect("spec code"),
            row_option_codes: [
                talent_option_code("war_row_double_charge").expect("option code"),
                0,
                0,
                0,
                0,
                0,
            ],
        }
    );
    assert_eq!(mapper.next_sequence(), Some(731));

    assert_eq!(
        mapper
            .map_intent(ClientGameplayIntent::ApplyTalents {
                player_class_id: "warrior".to_owned(),
                spec_id: None,
                row_option_ids: [
                    Some("mag_r5_ice_floes".to_owned()),
                    None,
                    None,
                    None,
                    None,
                    None,
                ],
            })
            .expect_err("foreign row option is rejected before transport"),
        ClientInputMappingError::InvalidTalentOptionId {
            option_id: "mag_r5_ice_floes".to_owned(),
        }
    );
    assert_eq!(mapper.next_sequence(), Some(731));
}

#[test]
fn talent_spec_intents_use_class_scoped_catalog_ids_and_preserve_the_clear_sentinel() {
    let mut mapper = ClientCommandMapper::new(actor(), 710).expect("valid actor");
    let selected = mapper
        .map_intent(ClientGameplayIntent::SetTalentSpec {
            player_class_id: "warrior".to_owned(),
            spec_id: Some("arms".to_owned()),
        })
        .expect("set talent spec");
    let cleared = mapper
        .map_intent(ClientGameplayIntent::SetTalentSpec {
            player_class_id: "warrior".to_owned(),
            spec_id: None,
        })
        .expect("clear talent spec");

    assert_eq!(
        (selected.command_id, selected.sequence),
        (SET_SPEC_COMMAND_ID, 710)
    );
    assert_eq!(
        SetSpecCommandPayload::decode(&selected.payload).expect("decode selected spec"),
        SetSpecCommandPayload {
            spec_code: talent_spec_code("warrior", "arms").expect("spec code"),
        }
    );
    assert_eq!(
        SetSpecCommandPayload::decode(&cleared.payload).expect("decode cleared spec"),
        SetSpecCommandPayload { spec_code: 0 }
    );
    assert_eq!(mapper.next_sequence(), Some(712));

    assert_eq!(
        mapper
            .map_intent(ClientGameplayIntent::SetTalentSpec {
                player_class_id: "warrior".to_owned(),
                spec_id: Some("arcane".to_owned()),
            })
            .expect_err("foreign source spec is rejected before transport"),
        ClientInputMappingError::InvalidTalentSpecId {
            player_class_id: "warrior".to_owned(),
            spec_id: "arcane".to_owned(),
        }
    );
    assert_eq!(mapper.next_sequence(), Some(712));
}

#[test]
fn talent_loadout_intents_encode_bounded_indices_without_consuming_rejections() {
    let mut mapper = ClientCommandMapper::new(actor(), 740).expect("valid actor");
    let switch = mapper
        .map_intent(ClientGameplayIntent::SwitchTalentLoadout { index: 2 })
        .expect("switch loadout");
    let delete = mapper
        .map_intent(ClientGameplayIntent::DeleteTalentLoadout { index: 9 })
        .expect("delete loadout");

    assert_eq!(
        (switch.command_id, switch.sequence),
        (SWITCH_LOADOUT_COMMAND_ID, 740)
    );
    assert_eq!(
        (delete.command_id, delete.sequence),
        (DELETE_LOADOUT_COMMAND_ID, 741)
    );
    assert_eq!(
        SwitchLoadoutCommandPayload::decode(&switch.payload).expect("decode switch"),
        SwitchLoadoutCommandPayload { index: 2 }
    );
    assert_eq!(
        DeleteLoadoutCommandPayload::decode(&delete.payload).expect("decode delete"),
        DeleteLoadoutCommandPayload { index: 9 }
    );
    assert_eq!(mapper.next_sequence(), Some(742));
    assert_eq!(
        mapper
            .map_intent(ClientGameplayIntent::SwitchTalentLoadout { index: 10 })
            .expect_err("the tenth slot is outside the source limit"),
        ClientInputMappingError::Protocol(ProtocolError::InvalidTalentLoadoutIndex(10))
    );
    assert_eq!(mapper.next_sequence(), Some(742));
}

#[test]
fn resurrection_response_intents_preserve_the_source_boolean_field() {
    let mut mapper = ClientCommandMapper::new(actor(), 760).expect("valid actor");
    let accept = mapper
        .map_intent(ClientGameplayIntent::RespondToResurrection { accept: true })
        .expect("accept response");
    let decline = mapper
        .map_intent(ClientGameplayIntent::RespondToResurrection { accept: false })
        .expect("decline response");

    assert_eq!(
        (accept.command_id, accept.sequence),
        (RESURRECT_RESPOND_COMMAND_ID, 760)
    );
    assert_eq!(
        (decline.command_id, decline.sequence),
        (RESURRECT_RESPOND_COMMAND_ID, 761)
    );
    assert_eq!(
        ResurrectRespondCommandPayload::decode(&accept.payload).expect("decode accept"),
        ResurrectRespondCommandPayload { accept: true }
    );
    assert_eq!(
        ResurrectRespondCommandPayload::decode(&decline.payload).expect("decode decline"),
        ResurrectRespondCommandPayload { accept: false }
    );
    assert_eq!(mapper.next_sequence(), Some(762));
}

#[test]
fn spirit_loop_intents_emit_the_source_empty_commands_in_order() {
    let mut mapper = ClientCommandMapper::new(actor(), 762).expect("valid actor");
    let release = mapper
        .map_intent(ClientGameplayIntent::ReleaseSpirit)
        .expect("release spirit");
    let corpse = mapper
        .map_intent(ClientGameplayIntent::ResurrectAtCorpse)
        .expect("resurrect at corpse");
    let healer = mapper
        .map_intent(ClientGameplayIntent::ResurrectAtSpiritHealer)
        .expect("resurrect at healer");

    assert_eq!(
        (release.command_id, release.sequence, release.payload),
        (RELEASE_COMMAND_ID, 762, Vec::new())
    );
    assert_eq!(
        (corpse.command_id, corpse.sequence, corpse.payload),
        (RESURRECT_CORPSE_COMMAND_ID, 763, Vec::new())
    );
    assert_eq!(
        (healer.command_id, healer.sequence, healer.payload),
        (RESURRECT_HEALER_COMMAND_ID, 764, Vec::new())
    );
}

#[test]
fn world_object_intents_preserve_source_number_fields_and_outcome_tokens() {
    let mut mapper = ClientCommandMapper::new(actor(), 900).expect("valid actor");
    let loot = mapper
        .map_intent(ClientGameplayIntent::LootCorpse { object_id: 11.5 })
        .expect("loot");
    let pickup = mapper
        .map_intent(ClientGameplayIntent::PickUpObject { object_id: -2.0 })
        .expect("pickup");
    let auto_loot = mapper
        .map_intent(ClientGameplayIntent::AutoLoot { object_id: 3.0 })
        .expect("auto loot");
    let delve_interact = mapper
        .map_intent(ClientGameplayIntent::InteractWithDelveObject { object_id: 4.0 })
        .expect("delve interact");
    let chest = mapper
        .map_intent(ClientGameplayIntent::CollectDelveChestLoot { object_id: 5.0 })
        .expect("collect chest");

    assert_eq!((loot.command_id, loot.sequence), (LOOT_COMMAND_ID, 900));
    assert_eq!(
        WorldObjectIdPayload::decode(&loot.payload, WorldObjectAction::Loot).expect("decode loot"),
        WorldObjectIdPayload { object_id: 11.5 }
    );
    assert_eq!(
        (pickup.command_id, pickup.sequence),
        (PICKUP_COMMAND_ID, 901)
    );
    assert_eq!(
        WorldObjectIdPayload::decode(&pickup.payload, WorldObjectAction::Pickup)
            .expect("decode pickup"),
        WorldObjectIdPayload { object_id: -2.0 }
    );
    assert_eq!(
        (auto_loot.command_id, auto_loot.sequence),
        (AUTO_LOOT_COMMAND_ID, 902)
    );
    assert_eq!(
        (delve_interact.command_id, delve_interact.sequence),
        (DELVE_INTERACT_COMMAND_ID, 903)
    );
    assert_eq!(
        (chest.command_id, chest.sequence),
        (COLLECT_DELVE_CHEST_LOOT_COMMAND_ID, 904)
    );
    assert_eq!(mapper.next_sequence(), Some(905));

    assert!(matches!(
        mapper.map_intent(ClientGameplayIntent::LootCorpse {
            object_id: f64::INFINITY,
        }),
        Err(ClientInputMappingError::Protocol(
            ProtocolError::NonFinite {
                field: "WorldObjectIdPayload.object_id",
                ..
            }
        ))
    ));
    assert_eq!(mapper.next_sequence(), Some(905));
}

#[test]
fn empty_action_intents_preserve_current_source_command_tokens() {
    let mut mapper = ClientCommandMapper::new(actor(), 910).expect("valid actor");
    let sell = mapper
        .map_intent(ClientGameplayIntent::SellAllJunk)
        .expect("sell all junk");
    let collect = mapper
        .map_intent(ClientGameplayIntent::CollectMarketProceeds)
        .expect("market collect");
    let dungeon = mapper
        .map_intent(ClientGameplayIntent::LeaveDungeon)
        .expect("leave dungeon");
    let delve = mapper
        .map_intent(ClientGameplayIntent::LeaveDelve)
        .expect("leave delve");

    assert_eq!(
        (sell.command_id, sell.sequence, sell.payload),
        (SELL_ALL_JUNK_COMMAND_ID, 910, Vec::new())
    );
    assert_eq!(
        (collect.command_id, collect.sequence, collect.payload),
        (MARKET_COLLECT_COMMAND_ID, 911, Vec::new())
    );
    assert_eq!(
        (dungeon.command_id, dungeon.sequence, dungeon.payload),
        (LEAVE_DUNGEON_COMMAND_ID, 912, Vec::new())
    );
    assert_eq!(
        (delve.command_id, delve.sequence, delve.payload),
        (LEAVE_DELVE_COMMAND_ID, 913, Vec::new())
    );
    assert_eq!(mapper.next_sequence(), Some(914));
}

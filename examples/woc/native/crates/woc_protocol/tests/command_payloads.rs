use woc_protocol::{
    command_payload_descriptor, talent_option_code, talent_spec_code, validate_command_payload,
    AbandonQuestCommandPayload, AcceptQuestCommandPayload, ApplyTalentsCommandPayload,
    ArenaAugmentCommandPayload, ArenaFormat, ArenaQueueCommandPayload, BankAction,
    BankSlotCommandPayload, BuyItemCommandPayload, BuybackItemCommandPayload,
    CancelAuraCommandPayload, CastAbilityCommandPayload, CastAtCommandPayload,
    CastSlotCommandPayload, ChangeSkinCommandPayload, CommandPayloadKind,
    DeleteLoadoutCommandPayload, DiscardItemCommandPayload, DuelRequestCommandPayload,
    DungeonFinderActivitiesPayload, DungeonFinderApplicationResponsePayload,
    DungeonFinderListingIdPayload, DungeonFinderListingPayload, DungeonFinderListingTag,
    DungeonFinderRole, DungeonFinderRolesPayload, EquipBagCommandPayload, EquipItemPayload,
    EquipmentSlot, GuildEventCreateCommandPayload, LinkedQuestAcceptancePayload,
    LockpickAbortCommandPayload, LockpickAction, LockpickActionCommandPayload,
    LockpickEngageCommandPayload, MailAction, MailIdCommandPayload, MasterLootThreshold,
    PartyLootMasterCommandPayload, PartyMarkerClearCommandPayload, PartyMarkerCommandPayload,
    ProtocolError, ReadyCheckRespondCommandPayload, ResurrectRespondCommandPayload,
    SelectTalentRowCommandPayload, SellItemCommandPayload, SetSpecCommandPayload, SkinCatalog,
    SwitchLoadoutCommandPayload, TargetCommandPayload, TradeRequestCommandPayload,
    TurnInQuestCommandPayload, UnequipBagCommandPayload, UnequipItemPayload, UseItemCommandPayload,
    ValeCupBetCommandPayload, ValeCupBracket, ValeCupNation, ValeCupPracticeCommandPayload,
    ValeCupQueueCommandPayload, ValeCupRole, ValeCupRoleCommandPayload, ValeCupSide,
    WorldObjectAction, WorldObjectIdPayload, ABANDON_QUEST_COMMAND_ID, ACCEPT_QUEST_COMMAND_ID,
    APPLY_TALENTS_COMMAND_ID, ARENA_AUGMENT_COMMAND_ID, ARENA_QUEUE_COMMAND_ID, ATTACK_COMMAND_ID,
    AUTO_LOOT_COMMAND_ID, BANK_DEPOSIT_COMMAND_ID, BANK_WITHDRAW_COMMAND_ID, BUYBACK_COMMAND_ID,
    BUY_COMMAND_ID, CANCEL_AURA_COMMAND_ID, CAST_AT_COMMAND_ID, CAST_COMMAND_ID,
    CAST_SLOT_COMMAND_ID, CHANGE_SKIN_COMMAND_ID, COLLECT_DELVE_CHEST_LOOT_COMMAND_ID,
    DELETE_LOADOUT_COMMAND_ID, DELVE_INTERACT_COMMAND_ID, DISCARD_ITEM_COMMAND_ID,
    DUEL_REQUEST_COMMAND_ID, DUNGEON_FINDER_APPLICATION_RESPONSE_COMMAND_ID,
    DUNGEON_FINDER_APPLY_COMMAND_ID, DUNGEON_FINDER_LIST_CREATE_COMMAND_ID,
    DUNGEON_FINDER_QUEUE_COMMAND_ID, DUNGEON_FINDER_ROLES_COMMAND_ID, EQUIP_BAG_COMMAND_ID,
    GUILD_EVENT_CREATE_COMMAND_ID, INTERACT_COMMAND_ID, LOCKPICK_ABORT_COMMAND_ID,
    LOCKPICK_ACTION_COMMAND_ID, LOCKPICK_ENGAGE_COMMAND_ID, LOOT_COMMAND_ID,
    MAIL_DELETE_COMMAND_ID, MAIL_READ_COMMAND_ID, MAIL_TAKE_COMMAND_ID,
    PARTY_CLEAR_MARKER_COMMAND_ID, PARTY_READY_RESPOND_COMMAND_ID,
    PARTY_SET_LOOT_MASTER_COMMAND_ID, PARTY_SET_MARKER_COMMAND_ID, PICKUP_COMMAND_ID,
    RESPEC_COMMAND_ID, RESURRECT_RESPOND_COMMAND_ID, SELECT_TALENT_ROW_COMMAND_ID, SELL_COMMAND_ID,
    SET_SPEC_COMMAND_ID, STOP_ATTACK_COMMAND_ID, SWITCH_LOADOUT_COMMAND_ID, TARGET_COMMAND_ID,
    TRADE_REQUEST_COMMAND_ID, TURN_IN_QUEST_COMMAND_ID, UNEQUIP_BAG_COMMAND_ID,
    USE_ITEM_COMMAND_ID, VALE_CUP_BET_COMMAND_ID, VALE_CUP_PRACTICE_COMMAND_ID,
    VALE_CUP_QUEUE_COMMAND_ID, VALE_CUP_ROLE_COMMAND_ID,
};

#[test]
fn target_payload_round_trips_a_target_and_the_clear_sentinel() {
    let target = TargetCommandPayload {
        target_id: Some(42),
    };
    let encoded = target.encode().expect("target payload must encode");
    assert_eq!(encoded, 42_u64.to_le_bytes());
    assert_eq!(
        TargetCommandPayload::decode(&encoded).expect("target payload must decode"),
        target
    );

    let clear = TargetCommandPayload { target_id: None };
    let encoded = clear.encode().expect("clear payload must encode");
    assert_eq!(encoded, 0_u64.to_le_bytes());
    assert_eq!(
        TargetCommandPayload::decode(&encoded).expect("clear payload must decode"),
        clear
    );
}

#[test]
fn vendor_payloads_preserve_source_item_and_npc_fields() {
    let buy = BuyItemCommandPayload {
        npc_id: 42,
        item_id: "baked_bread".to_owned(),
    };
    let mut expected_buy = (11_u32).to_le_bytes().to_vec();
    expected_buy.extend_from_slice(b"baked_bread");
    expected_buy.push(1);
    expected_buy.extend_from_slice(&42_u64.to_le_bytes());
    let encoded_buy = buy.encode().expect("buy payload");
    assert_eq!(encoded_buy, expected_buy);
    assert_eq!(
        BuyItemCommandPayload::decode(&encoded_buy).expect("decode buy"),
        buy
    );
    assert_eq!(
        validate_command_payload(BUY_COMMAND_ID, &encoded_buy),
        Ok(())
    );

    let sell = SellItemCommandPayload {
        item_id: "bandit_bandana".to_owned(),
        count: Some(2),
    };
    let mut expected_sell = (14_u32).to_le_bytes().to_vec();
    expected_sell.extend_from_slice(b"bandit_bandana");
    expected_sell.push(1);
    expected_sell.extend_from_slice(&2_u32.to_le_bytes());
    let encoded_sell = sell.encode().expect("sell payload");
    assert_eq!(encoded_sell, expected_sell);
    assert_eq!(
        SellItemCommandPayload::decode(&encoded_sell).expect("decode sell"),
        sell
    );
    assert_eq!(
        validate_command_payload(SELL_COMMAND_ID, &encoded_sell),
        Ok(())
    );

    let buyback = BuybackItemCommandPayload {
        item_id: "bandit_bandana".to_owned(),
    };
    let mut expected_buyback = (14_u32).to_le_bytes().to_vec();
    expected_buyback.extend_from_slice(b"bandit_bandana");
    let encoded_buyback = buyback.encode().expect("buyback payload");
    assert_eq!(encoded_buyback, expected_buyback);
    assert_eq!(
        BuybackItemCommandPayload::decode(&encoded_buyback).expect("decode buyback"),
        buyback
    );
    assert_eq!(
        validate_command_payload(BUYBACK_COMMAND_ID, &encoded_buyback),
        Ok(())
    );

    let mut missing_npc = (11_u32).to_le_bytes().to_vec();
    missing_npc.extend_from_slice(b"baked_bread");
    missing_npc.push(0);
    assert!(matches!(
        BuyItemCommandPayload::decode(&missing_npc),
        Err(ProtocolError::InvalidEntityId {
            context: "BuyItemCommandPayload.npc_id"
        })
    ));
}

#[test]
fn party_extension_payloads_preserve_json_transport_fields_without_sim_policy() {
    let loot_master = PartyLootMasterCommandPayload {
        enabled: true,
        looter: 42.5,
        threshold: MasterLootThreshold::Rare,
    };
    let encoded = loot_master
        .clone()
        .encode()
        .expect("party loot-master payload");
    let mut expected = vec![1];
    expected.extend_from_slice(&42.5_f64.to_le_bytes());
    expected.push(1);
    assert_eq!(encoded, expected);
    assert_eq!(
        PartyLootMasterCommandPayload::decode(&encoded).expect("decode party loot-master"),
        loot_master
    );

    let marker = PartyMarkerCommandPayload {
        entity_id: 9_001.0,
        marker_id: 24.5,
    };
    let encoded = marker.clone().encode().expect("party marker payload");
    assert_eq!(encoded.len(), 16);
    assert_eq!(
        PartyMarkerCommandPayload::decode(&encoded).expect("decode party marker"),
        marker
    );

    let clear = PartyMarkerClearCommandPayload { entity_id: 9_001.0 };
    let encoded = clear.clone().encode().expect("clear party marker payload");
    assert_eq!(encoded, 9_001.0_f64.to_le_bytes());
    assert_eq!(
        PartyMarkerClearCommandPayload::decode(&encoded).expect("decode clear party marker"),
        clear
    );

    for (ready, expected) in [(true, [1]), (false, [0])] {
        let payload = ReadyCheckRespondCommandPayload { ready };
        assert_eq!(payload.encode(), expected);
        assert_eq!(
            ReadyCheckRespondCommandPayload::decode(&expected).expect("decode ready response"),
            payload
        );
    }

    assert!(matches!(
        PartyLootMasterCommandPayload {
            enabled: true,
            looter: f64::NAN,
            threshold: MasterLootThreshold::Epic,
        }
        .encode(),
        Err(ProtocolError::NonFinite {
            field: "PartyLootMasterCommandPayload.looter",
            ..
        })
    ));

    let mut malformed_marker = marker.encode().expect("party marker payload");
    malformed_marker[8..].copy_from_slice(&f64::INFINITY.to_le_bytes());
    assert!(matches!(
        validate_command_payload(PARTY_SET_MARKER_COMMAND_ID, &malformed_marker),
        Err(ProtocolError::NonFinite {
            field: "PartyMarkerCommandPayload.marker_id",
            ..
        })
    ));
    assert_eq!(
        validate_command_payload(PARTY_SET_LOOT_MASTER_COMMAND_ID, &encoded),
        Err(ProtocolError::InvalidCommandPayloadLength {
            command_id: PARTY_SET_LOOT_MASTER_COMMAND_ID,
            actual: 8,
            expected: 10,
        })
    );
    assert_eq!(
        validate_command_payload(PARTY_CLEAR_MARKER_COMMAND_ID, &[0; 7]),
        Err(ProtocolError::InvalidCommandPayloadLength {
            command_id: PARTY_CLEAR_MARKER_COMMAND_ID,
            actual: 7,
            expected: 8,
        })
    );
    assert_eq!(
        validate_command_payload(PARTY_READY_RESPOND_COMMAND_ID, &[2]),
        Err(ProtocolError::InvalidBoolean(2))
    );
    let mut invalid_threshold = loot_master.encode().expect("party loot-master payload");
    invalid_threshold[9] = 3;
    assert_eq!(
        validate_command_payload(PARTY_SET_LOOT_MASTER_COMMAND_ID, &invalid_threshold),
        Err(ProtocolError::InvalidMasterLootThreshold(3))
    );
}

#[test]
fn duel_arena_payloads_preserve_source_transport_types_without_sim_policy() {
    let request = DuelRequestCommandPayload { target_id: 42.5 };
    let encoded = request.clone().encode().expect("duel request payload");
    assert_eq!(encoded, 42.5_f64.to_le_bytes());
    assert_eq!(
        DuelRequestCommandPayload::decode(&encoded).expect("decode duel request"),
        request
    );
    assert!(matches!(
        DuelRequestCommandPayload {
            target_id: f64::NEG_INFINITY,
        }
        .encode(),
        Err(ProtocolError::NonFinite {
            field: "DuelRequestCommandPayload.target_id",
            ..
        })
    ));

    for (format, expected) in [
        (ArenaFormat::OneVOne, [0]),
        (ArenaFormat::TwoVTwo, [1]),
        (ArenaFormat::Fiesta, [2]),
        (ArenaFormat::YumiThree, [3]),
        (ArenaFormat::YumiFive, [4]),
    ] {
        let payload = ArenaQueueCommandPayload { format };
        assert_eq!(payload.encode().expect("arena queue payload"), expected);
        assert_eq!(
            ArenaQueueCommandPayload::decode(&expected).expect("decode arena queue"),
            payload
        );
    }
    assert_eq!(
        validate_command_payload(ARENA_QUEUE_COMMAND_ID, &[5]),
        Err(ProtocolError::InvalidArenaFormat(5))
    );

    let augment = ArenaAugmentCommandPayload {
        augment_id: "fiesta_sprint".to_owned(),
    };
    let encoded = augment.clone().encode().expect("arena augment payload");
    assert_eq!(&encoded[..4], &(13_u32).to_le_bytes());
    assert_eq!(
        ArenaAugmentCommandPayload::decode(&encoded).expect("decode arena augment"),
        augment
    );
    let non_bmp = ArenaAugmentCommandPayload {
        augment_id: "\u{1f3c6}".repeat(32),
    };
    assert_eq!(
        ArenaAugmentCommandPayload::decode(&non_bmp.clone().encode().expect("non-BMP augment"))
            .expect("decode non-BMP augment"),
        non_bmp
    );
    assert_eq!(
        ArenaAugmentCommandPayload {
            augment_id: "x".repeat(65),
        }
        .encode()
        .expect_err("source UTF-16 bound"),
        ProtocolError::CollectionTooLarge {
            context: "ArenaAugmentCommandPayload.augment_id_utf16_code_units",
            actual: 65,
            maximum: 64,
        }
    );
    assert_eq!(
        validate_command_payload(ARENA_AUGMENT_COMMAND_ID, &[1, 0, 0, 0]),
        Err(ProtocolError::LengthMismatch {
            declared: 1,
            actual: 0,
        })
    );
    assert_eq!(
        validate_command_payload(ARENA_AUGMENT_COMMAND_ID, &[1, 0, 0, 0, 0xff]),
        Err(ProtocolError::InvalidUtf8 {
            context: "ArenaAugmentCommandPayload.augment_id",
        })
    );
    assert_eq!(
        validate_command_payload(DUEL_REQUEST_COMMAND_ID, &[0; 7]),
        Err(ProtocolError::InvalidCommandPayloadLength {
            command_id: DUEL_REQUEST_COMMAND_ID,
            actual: 7,
            expected: 8,
        })
    );
}

#[test]
fn trade_request_payload_preserves_the_source_number_without_trade_policy() {
    let request = TradeRequestCommandPayload { target_id: 42.5 };
    let encoded = request.clone().encode().expect("trade request payload");
    assert_eq!(encoded, 42.5_f64.to_le_bytes());
    assert_eq!(
        TradeRequestCommandPayload::decode(&encoded).expect("decode trade request"),
        request
    );
    assert!(matches!(
        TradeRequestCommandPayload {
            target_id: f64::INFINITY,
        }
        .encode(),
        Err(ProtocolError::NonFinite {
            field: "TradeRequestCommandPayload.target_id",
            ..
        })
    ));
    assert_eq!(
        validate_command_payload(TRADE_REQUEST_COMMAND_ID, &[0; 7]),
        Err(ProtocolError::InvalidCommandPayloadLength {
            command_id: TRADE_REQUEST_COMMAND_ID,
            actual: 7,
            expected: 8,
        })
    );
}

#[test]
fn cast_slot_payload_preserves_the_upstream_signed_wire_index() {
    for slot in [-1, 0, 7, i32::MAX, i32::MIN] {
        let payload = CastSlotCommandPayload { slot };
        let encoded = payload.encode();
        assert_eq!(encoded, slot.to_le_bytes());
        assert_eq!(
            CastSlotCommandPayload::decode(&encoded).expect("cast slot payload must decode"),
            payload
        );
    }
}

#[test]
fn change_skin_payload_preserves_catalog_and_source_class_bounds() {
    let class = ChangeSkinCommandPayload {
        catalog: SkinCatalog::Class,
        skin_index: 7,
    };
    assert_eq!(class.encode().expect("class skin payload"), [0, 7]);
    assert_eq!(
        ChangeSkinCommandPayload::decode(&[0, 7]).expect("decode class skin"),
        class
    );

    let mech = ChangeSkinCommandPayload {
        catalog: SkinCatalog::Mech,
        skin_index: u8::MAX,
    };
    assert_eq!(mech.encode().expect("mech skin payload"), [1, u8::MAX]);
    assert_eq!(
        ChangeSkinCommandPayload::decode(&[1, u8::MAX]).expect("decode mech skin"),
        mech
    );

    assert_eq!(
        ChangeSkinCommandPayload {
            catalog: SkinCatalog::Class,
            skin_index: 8,
        }
        .encode(),
        Err(ProtocolError::InvalidClassSkinIndex(8))
    );
    assert_eq!(
        validate_command_payload(CHANGE_SKIN_COMMAND_ID, &[2, 0]),
        Err(ProtocolError::InvalidSkinCatalog(2))
    );
}

#[test]
fn select_talent_row_payload_uses_a_catalog_option_code_and_zero_clear_sentinel() {
    let option_code = talent_option_code("war_row_double_charge").expect("current option code");
    let selected = SelectTalentRowCommandPayload {
        level: 5,
        option_code,
    };
    let encoded = selected.encode().expect("selected talent payload");
    assert_eq!(encoded, [5, option_code as u8, (option_code >> 8) as u8]);
    assert_eq!(
        SelectTalentRowCommandPayload::decode(&encoded).expect("decode selected talent"),
        selected
    );

    let cleared = SelectTalentRowCommandPayload {
        level: 5,
        option_code: 0,
    };
    assert_eq!(cleared.encode().expect("clear talent payload"), [5, 0, 0]);
    assert_eq!(
        SelectTalentRowCommandPayload::decode(&[5, 0, 0]).expect("decode clear talent"),
        cleared
    );
    assert_eq!(
        SelectTalentRowCommandPayload {
            level: 6,
            option_code: 0,
        }
        .encode(),
        Err(ProtocolError::InvalidTalentRowLevel(6))
    );
    assert_eq!(
        validate_command_payload(SELECT_TALENT_ROW_COMMAND_ID, &[5, 0xff, 0xff]),
        Err(ProtocolError::InvalidTalentOptionCode(u16::MAX))
    );
}

#[test]
fn set_spec_payload_uses_a_catalog_spec_code_and_zero_clear_sentinel() {
    let spec_code = talent_spec_code("warrior", "arms").expect("current spec code");
    let selected = SetSpecCommandPayload { spec_code };
    let encoded = selected.encode().expect("selected spec payload");
    assert_eq!(encoded, spec_code.to_le_bytes());
    assert_eq!(
        SetSpecCommandPayload::decode(&encoded).expect("decode selected spec"),
        selected
    );

    let cleared = SetSpecCommandPayload { spec_code: 0 };
    assert_eq!(cleared.encode().expect("clear spec payload"), [0, 0]);
    assert_eq!(
        SetSpecCommandPayload::decode(&[0, 0]).expect("decode clear spec"),
        cleared
    );
    assert_eq!(
        validate_command_payload(SET_SPEC_COMMAND_ID, &[0xff, 0xff]),
        Err(ProtocolError::InvalidTalentSpecCode(u16::MAX))
    );
}

#[test]
fn apply_talents_payload_preserves_all_six_canonical_row_codes() {
    let spec_code = talent_spec_code("warrior", "arms").expect("current spec code");
    let option_code = talent_option_code("war_row_double_charge").expect("current option code");
    let payload = ApplyTalentsCommandPayload {
        spec_code,
        row_option_codes: [option_code, 0, 0, 0, 0, 0],
    };
    let encoded = payload.encode().expect("talent allocation payload");
    assert_eq!(encoded.len(), 14);
    assert_eq!(&encoded[..2], &spec_code.to_le_bytes());
    assert_eq!(&encoded[2..4], &option_code.to_le_bytes());
    assert_eq!(
        ApplyTalentsCommandPayload::decode(&encoded).expect("decode allocation"),
        payload
    );

    let mut invalid = [0; 14];
    invalid[2..4].copy_from_slice(&u16::MAX.to_le_bytes());
    assert_eq!(
        validate_command_payload(APPLY_TALENTS_COMMAND_ID, &invalid),
        Err(ProtocolError::InvalidTalentOptionCode(u16::MAX))
    );
}

#[test]
fn cast_at_payload_preserves_finite_ground_coordinates_and_normalizes_negative_zero() {
    let payload = CastAtCommandPayload::new("flame_strike".to_owned(), 12.5, -0.0)
        .expect("finite ground point");
    assert_eq!(payload.aim.x(), 12.5);
    assert_eq!(payload.aim.z().to_bits(), 0.0_f64.to_bits());

    let encoded = payload.encode().expect("cast-at payload must encode");
    let mut expected = vec![12, 0, 0, 0];
    expected.extend_from_slice(b"flame_strike");
    expected.extend_from_slice(&12.5_f64.to_le_bytes());
    expected.extend_from_slice(&0.0_f64.to_le_bytes());
    assert_eq!(encoded, expected);
    assert_eq!(
        CastAtCommandPayload::decode(&encoded).expect("cast-at payload must decode"),
        payload
    );

    assert!(matches!(
        CastAtCommandPayload::new("flame_strike".to_owned(), f64::NAN, 0.0),
        Err(ProtocolError::NonFinite {
            field: "CastAtCommandPayload.x",
            ..
        })
    ));

    let mut nonfinite_wire = vec![12, 0, 0, 0];
    nonfinite_wire.extend_from_slice(b"flame_strike");
    nonfinite_wire.extend_from_slice(&f64::NAN.to_le_bytes());
    nonfinite_wire.extend_from_slice(&0.0_f64.to_le_bytes());
    assert!(matches!(
        CastAtCommandPayload::decode(&nonfinite_wire),
        Err(ProtocolError::NonFinite {
            field: "CastAtCommandPayload.x",
            ..
        })
    ));
}

#[test]
fn cast_ability_payload_preserves_the_upstream_ability_identifier() {
    let payload = CastAbilityCommandPayload {
        ability_id: "frostbolt".to_owned(),
        target_id: None,
    };
    let encoded = payload.encode().expect("cast ability payload must encode");
    assert_eq!(
        encoded,
        [9, 0, 0, 0, b'f', b'r', b'o', b's', b't', b'b', b'o', b'l', b't', 0]
    );
    assert_eq!(
        CastAbilityCommandPayload::decode(&encoded).expect("cast ability payload must decode"),
        payload
    );

    let targeted = CastAbilityCommandPayload {
        ability_id: "frostbolt".to_owned(),
        target_id: Some(42),
    };
    let mut targeted_wire = vec![
        9, 0, 0, 0, b'f', b'r', b'o', b's', b't', b'b', b'o', b'l', b't', 1,
    ];
    targeted_wire.extend_from_slice(&42_u64.to_le_bytes());
    assert_eq!(
        targeted.encode().expect("targeted cast ability payload"),
        targeted_wire
    );
    assert_eq!(
        CastAbilityCommandPayload::decode(&targeted_wire).expect("decode targeted cast ability"),
        targeted
    );
}

#[test]
fn cancel_aura_payload_preserves_the_upstream_aura_identifier() {
    let payload = CancelAuraCommandPayload {
        aura_id: "ice_armor".to_owned(),
    };
    let encoded = payload.encode().expect("cancel aura payload must encode");
    assert_eq!(
        encoded,
        [9, 0, 0, 0, b'i', b'c', b'e', b'_', b'a', b'r', b'm', b'o', b'r']
    );
    assert_eq!(
        CancelAuraCommandPayload::decode(&encoded).expect("cancel aura payload must decode"),
        payload
    );
}

#[test]
fn accept_quest_payload_preserves_the_upstream_quest_identifier() {
    let payload = AcceptQuestCommandPayload {
        quest_id: "eastbrook_welcome".to_owned(),
        selection: None,
    };
    let encoded = payload.encode().expect("accept quest payload must encode");
    assert_eq!(
        encoded,
        [
            17, 0, 0, 0, b'e', b'a', b's', b't', b'b', b'r', b'o', b'o', b'k', b'_', b'w', b'e',
            b'l', b'c', b'o', b'm', b'e', 0
        ]
    );
    assert_eq!(
        AcceptQuestCommandPayload::decode(&encoded).expect("accept quest payload must decode"),
        payload
    );
}

#[test]
fn accept_quest_payload_preserves_an_optional_source_selection() {
    let payload = AcceptQuestCommandPayload {
        quest_id: "q_wolves".to_owned(),
        selection: Some("wolf_a".to_owned()),
    };
    let encoded = payload
        .encode()
        .expect("selected accept quest payload must encode");
    assert_eq!(
        encoded,
        [
            8, 0, 0, 0, b'q', b'_', b'w', b'o', b'l', b'v', b'e', b's', 1, 6, 0, 0, 0, b'w', b'o',
            b'l', b'f', b'_', b'a'
        ]
    );
    assert_eq!(
        AcceptQuestCommandPayload::decode(&encoded)
            .expect("selected accept quest payload must decode"),
        payload
    );
}

#[test]
fn linked_quest_payload_preserves_the_source_quest_and_sharer_fields() {
    let payload = LinkedQuestAcceptancePayload {
        quest_id: "q_wolves".to_owned(),
        sharer_pid: 2.0,
    };
    let encoded = payload
        .clone()
        .encode()
        .expect("linked quest payload encodes");
    assert_eq!(
        LinkedQuestAcceptancePayload::decode(&encoded).expect("linked quest payload decodes"),
        payload
    );
    assert_eq!(
        &encoded[..12],
        &[8, 0, 0, 0, b'q', b'_', b'w', b'o', b'l', b'v', b'e', b's']
    );
    assert_eq!(&encoded[12..], &2.0f64.to_le_bytes());
}

#[test]
fn equipment_payloads_preserve_optional_and_required_source_slots() {
    let automatic = EquipItemPayload {
        item_id: "gnarled_staff".to_owned(),
        slot: None,
    };
    let encoded = automatic.clone().encode().expect("automatic equip encodes");
    assert_eq!(&encoded[..4], &13_u32.to_le_bytes());
    assert_eq!(&encoded[4..17], b"gnarled_staff");
    assert_eq!(encoded[17], 0);
    assert_eq!(EquipItemPayload::decode(&encoded), Ok(automatic));

    let aimed = EquipItemPayload {
        item_id: "cryptbone_helm".to_owned(),
        slot: Some(EquipmentSlot::Helmet),
    };
    let encoded = aimed.clone().encode().expect("aimed equip encodes");
    assert_eq!(encoded.last(), Some(&3));
    assert_eq!(EquipItemPayload::decode(&encoded), Ok(aimed));

    let unequip = UnequipItemPayload {
        slot: EquipmentSlot::Feet,
    };
    assert_eq!(unequip.encode(), Ok([10]));
    assert_eq!(UnequipItemPayload::decode(&[10]), Ok(unequip));
    assert_eq!(
        UnequipItemPayload::decode(&[0]),
        Err(ProtocolError::InvalidEquipmentSlot(0))
    );
}

#[test]
fn turn_in_quest_payload_preserves_the_upstream_quest_identifier() {
    let payload = TurnInQuestCommandPayload {
        quest_id: "eastbrook_welcome".to_owned(),
    };
    let encoded = payload.encode().expect("turn-in quest payload must encode");
    assert_eq!(
        encoded,
        [
            17, 0, 0, 0, b'e', b'a', b's', b't', b'b', b'r', b'o', b'o', b'k', b'_', b'w', b'e',
            b'l', b'c', b'o', b'm', b'e'
        ]
    );
    assert_eq!(
        TurnInQuestCommandPayload::decode(&encoded).expect("turn-in quest payload must decode"),
        payload
    );
}

#[test]
fn generated_payload_descriptors_pin_the_first_authoritative_commands() {
    assert_eq!(CAST_SLOT_COMMAND_ID, 0);
    assert_eq!(CAST_AT_COMMAND_ID, 1);
    assert_eq!(CAST_COMMAND_ID, 2);
    assert_eq!(CANCEL_AURA_COMMAND_ID, 3);
    assert_eq!(TARGET_COMMAND_ID, 4);
    assert_eq!(ATTACK_COMMAND_ID, 9);
    assert_eq!(STOP_ATTACK_COMMAND_ID, 10);
    assert_eq!(INTERACT_COMMAND_ID, 11);
    assert_eq!(ACCEPT_QUEST_COMMAND_ID, 16);
    assert_eq!(TURN_IN_QUEST_COMMAND_ID, 17);
    assert_eq!(ABANDON_QUEST_COMMAND_ID, 18);
    assert_eq!(USE_ITEM_COMMAND_ID, 23);
    assert_eq!(DISCARD_ITEM_COMMAND_ID, 24);
    assert_eq!(EQUIP_BAG_COMMAND_ID, 126);
    assert_eq!(UNEQUIP_BAG_COMMAND_ID, 127);
    assert_eq!(LOCKPICK_ENGAGE_COMMAND_ID, 120);
    assert_eq!(LOCKPICK_ACTION_COMMAND_ID, 121);
    assert_eq!(LOCKPICK_ABORT_COMMAND_ID, 122);
    assert_eq!(APPLY_TALENTS_COMMAND_ID, 95);
    assert_eq!(RESPEC_COMMAND_ID, 96);
    assert_eq!(SET_SPEC_COMMAND_ID, 97);
    assert_eq!(CHANGE_SKIN_COMMAND_ID, 31);
    assert_eq!(
        command_payload_descriptor(CAST_SLOT_COMMAND_ID).map(|entry| entry.kind),
        Some(CommandPayloadKind::I32Value)
    );
    let cast_at = command_payload_descriptor(CAST_AT_COMMAND_ID).expect("cast-at payload");
    assert_eq!(cast_at.kind, CommandPayloadKind::Utf8IdF64Pair);
    assert_eq!(
        (cast_at.min_byte_length, cast_at.max_byte_length),
        (20, 276)
    );
    assert_eq!(cast_at.max_utf8_bytes, 256);
    let cast = command_payload_descriptor(CAST_COMMAND_ID).expect("cast payload");
    assert_eq!(cast.kind, CommandPayloadKind::Utf8IdOptionalTargetEntity);
    assert_eq!((cast.min_byte_length, cast.max_byte_length), (5, 269));
    assert_eq!(cast.max_utf8_bytes, 256);
    let cancel_aura =
        command_payload_descriptor(CANCEL_AURA_COMMAND_ID).expect("cancel-aura payload");
    assert_eq!(cancel_aura.kind, CommandPayloadKind::Utf8Id);
    assert_eq!(
        (cancel_aura.min_byte_length, cancel_aura.max_byte_length),
        (4, 260)
    );
    assert_eq!(cancel_aura.max_utf8_bytes, 256);
    assert_eq!(
        command_payload_descriptor(TARGET_COMMAND_ID).map(|entry| entry.kind),
        Some(CommandPayloadKind::TargetEntity)
    );
    assert_eq!(
        command_payload_descriptor(ATTACK_COMMAND_ID).map(|entry| entry.kind),
        Some(CommandPayloadKind::Empty)
    );
    assert_eq!(
        command_payload_descriptor(STOP_ATTACK_COMMAND_ID).map(|entry| entry.kind),
        Some(CommandPayloadKind::Empty)
    );
    for (id, name) in [
        (5, "tab"),
        (6, "targetNearest"),
        (7, "tabFriendly"),
        (8, "targetNearestFriendly"),
        (INTERACT_COMMAND_ID, "interact"),
    ] {
        let descriptor = command_payload_descriptor(id).expect("targeting payload must exist");
        assert_eq!(descriptor.name, name);
        assert_eq!(descriptor.kind, CommandPayloadKind::Empty);
        assert_eq!(descriptor.fixed_byte_length(), Some(0));
        assert_eq!(validate_command_payload(id, &[]), Ok(()));
    }

    let abandon = command_payload_descriptor(ABANDON_QUEST_COMMAND_ID).expect("abandon payload");
    assert_eq!(abandon.kind, CommandPayloadKind::Utf8Id);
    assert_eq!((abandon.min_byte_length, abandon.max_byte_length), (4, 260));
    assert_eq!(abandon.fixed_byte_length(), None);

    let accept = command_payload_descriptor(ACCEPT_QUEST_COMMAND_ID).expect("accept payload");
    assert_eq!(accept.kind, CommandPayloadKind::Utf8IdOptionalUtf8Id);
    assert_eq!((accept.min_byte_length, accept.max_byte_length), (5, 521));
    assert_eq!(accept.max_utf8_bytes, 256);

    let turn_in = command_payload_descriptor(TURN_IN_QUEST_COMMAND_ID).expect("turn-in payload");
    assert_eq!(turn_in.kind, CommandPayloadKind::Utf8Id);
    assert_eq!((turn_in.min_byte_length, turn_in.max_byte_length), (4, 260));
    assert_eq!(turn_in.max_utf8_bytes, 256);

    let discard = command_payload_descriptor(DISCARD_ITEM_COMMAND_ID).expect("discard payload");
    assert_eq!(discard.kind, CommandPayloadKind::Utf8IdOptionalU32);
    assert_eq!((discard.min_byte_length, discard.max_byte_length), (5, 265));
    assert_eq!(discard.max_utf8_bytes, 256);

    let apply_talents =
        command_payload_descriptor(APPLY_TALENTS_COMMAND_ID).expect("apply-talents payload");
    assert_eq!(apply_talents.kind, CommandPayloadKind::TalentAllocation);
    assert_eq!(apply_talents.fixed_byte_length(), Some(14));

    let respec = command_payload_descriptor(RESPEC_COMMAND_ID).expect("respec payload");
    assert_eq!(respec.kind, CommandPayloadKind::Empty);
    assert_eq!(respec.fixed_byte_length(), Some(0));

    let set_spec = command_payload_descriptor(SET_SPEC_COMMAND_ID).expect("set-spec payload");
    assert_eq!(set_spec.kind, CommandPayloadKind::TalentSpec);
    assert_eq!(set_spec.fixed_byte_length(), Some(2));

    let change_skin =
        command_payload_descriptor(CHANGE_SKIN_COMMAND_ID).expect("change-skin payload");
    assert_eq!(change_skin.kind, CommandPayloadKind::CosmeticSkin);
    assert_eq!(change_skin.fixed_byte_length(), Some(2));

    let unequip = command_payload_descriptor(UNEQUIP_BAG_COMMAND_ID).expect("unequip-bag payload");
    assert_eq!(unequip.kind, CommandPayloadKind::U32Index);
    assert_eq!(unequip.fixed_byte_length(), Some(4));

    let engage =
        command_payload_descriptor(LOCKPICK_ENGAGE_COMMAND_ID).expect("lockpick-engage payload");
    assert_eq!(engage.kind, CommandPayloadKind::LockpickEngage);
    assert_eq!(engage.fixed_byte_length(), Some(9));

    let action =
        command_payload_descriptor(LOCKPICK_ACTION_COMMAND_ID).expect("lockpick-action payload");
    assert_eq!(action.kind, CommandPayloadKind::LockpickAction);
    assert_eq!((action.min_byte_length, action.max_byte_length), (2, 262));
    assert_eq!(action.max_utf8_bytes, 256);

    let abort =
        command_payload_descriptor(LOCKPICK_ABORT_COMMAND_ID).expect("lockpick-abort payload");
    assert_eq!(abort.kind, CommandPayloadKind::OptionalUtf8Id);
    assert_eq!((abort.min_byte_length, abort.max_byte_length), (1, 261));
    assert_eq!(abort.max_utf8_bytes, 256);

    let guild_event_create =
        command_payload_descriptor(GUILD_EVENT_CREATE_COMMAND_ID).expect("guild event payload");
    assert_eq!(
        guild_event_create.kind,
        CommandPayloadKind::GuildEventCreate
    );
    assert_eq!(
        (
            guild_event_create.min_byte_length,
            guild_event_create.max_byte_length,
            guild_event_create.max_utf8_bytes,
        ),
        (13, 863, 640)
    );
}

#[test]
fn guild_event_create_payload_preserves_structured_source_fields() {
    let payload = GuildEventCreateCommandPayload {
        day: "2026-08-10".to_owned(),
        hour: Some(21.75),
        title: "Vault run".to_owned(),
        note: "Bring keys".to_owned(),
    };
    let encoded = payload.encode().expect("encode guild event");
    assert_eq!(
        validate_command_payload(GUILD_EVENT_CREATE_COMMAND_ID, &encoded),
        Ok(())
    );
    assert_eq!(
        GuildEventCreateCommandPayload::decode(&encoded).expect("decode guild event"),
        payload
    );

    let all_day = GuildEventCreateCommandPayload {
        day: "2026-08-11".to_owned(),
        hour: None,
        title: "All day".to_owned(),
        note: String::new(),
    };
    let all_day_encoded = all_day.encode().expect("encode all-day event");
    assert_eq!(
        GuildEventCreateCommandPayload::decode(&all_day_encoded).expect("decode all-day event"),
        all_day
    );

    assert!(matches!(
        GuildEventCreateCommandPayload {
            day: "2026-08-10".to_owned(),
            hour: Some(f64::NAN),
            title: "Vault run".to_owned(),
            note: String::new(),
        }
        .encode(),
        Err(ProtocolError::NonFinite {
            field: "GuildEventCreateCommandPayload.hour",
            ..
        })
    ));
    assert_eq!(
        GuildEventCreateCommandPayload {
            day: "2026-08-10".to_owned(),
            hour: None,
            title: "x".repeat(193),
            note: String::new(),
        }
        .encode(),
        Err(ProtocolError::CollectionTooLarge {
            context: "GuildEventCreateCommandPayload.title",
            actual: 193,
            maximum: 192,
        })
    );

    let mut invalid_presence = all_day_encoded.clone();
    invalid_presence[14] = 2;
    assert_eq!(
        GuildEventCreateCommandPayload::decode(&invalid_presence),
        Err(ProtocolError::InvalidBoolean(2))
    );
    let mut trailing = encoded;
    trailing.push(0);
    assert_eq!(
        GuildEventCreateCommandPayload::decode(&trailing),
        Err(ProtocolError::TrailingPayload { remaining: 1 })
    );
}

#[test]
fn lockpick_payloads_preserve_source_command_shapes_without_a_raw_value_fallback() {
    let engage = LockpickEngageCommandPayload {
        object_id: 9_001,
        ante: 2,
    };
    let encoded = engage.encode().expect("lockpick engage payload");
    assert_eq!(encoded[..8], 9_001_u64.to_le_bytes());
    assert_eq!(encoded[8], 2);
    assert_eq!(
        LockpickEngageCommandPayload::decode(&encoded).expect("decode lockpick engage"),
        engage
    );

    for (session_id, action, expected) in [
        (None, LockpickAction::HardSet, vec![0, 0]),
        (
            Some("lk_42".to_owned()),
            LockpickAction::Ease,
            vec![1, 5, 0, 0, 0, b'l', b'k', b'_', b'4', b'2', 3],
        ),
        (Some("current".to_owned()), LockpickAction::Abort, {
            let mut bytes = vec![1, 7, 0, 0, 0];
            bytes.extend_from_slice(b"current");
            bytes.push(5);
            bytes
        }),
    ] {
        let payload = LockpickActionCommandPayload { session_id, action };
        let encoded = payload.encode().expect("lockpick action payload");
        assert_eq!(encoded, expected);
        assert_eq!(
            LockpickActionCommandPayload::decode(&encoded).expect("decode lockpick action"),
            payload
        );
    }

    for (session_id, expected) in [
        (None, vec![0]),
        (Some("lk_42".to_owned()), {
            let mut bytes = vec![1, 5, 0, 0, 0];
            bytes.extend_from_slice(b"lk_42");
            bytes
        }),
    ] {
        let payload = LockpickAbortCommandPayload { session_id };
        let encoded = payload.encode().expect("lockpick abort payload");
        assert_eq!(encoded, expected);
        assert_eq!(
            LockpickAbortCommandPayload::decode(&encoded).expect("decode lockpick abort"),
            payload
        );
    }
}

#[test]
fn inventory_and_quest_payloads_preserve_the_target_wire_fields() {
    let abandon = AbandonQuestCommandPayload {
        quest_id: "q_boars".to_owned(),
    };
    let encoded = abandon.encode().expect("abandon payload");
    assert_eq!(&encoded[..4], &7_u32.to_le_bytes());
    assert_eq!(&encoded[4..], b"q_boars");
    assert_eq!(
        AbandonQuestCommandPayload::decode(&encoded).expect("decode abandon"),
        abandon
    );

    let use_item = UseItemCommandPayload {
        item_id: "minor_healing_potion".to_owned(),
    };
    let encoded = use_item.encode().expect("use-item payload");
    assert_eq!(
        UseItemCommandPayload::decode(&encoded).expect("decode use-item"),
        use_item
    );

    for count in [None, Some(0), Some(3), Some(u32::MAX)] {
        let discard = DiscardItemCommandPayload {
            item_id: "wolf_fang".to_owned(),
            count,
        };
        let encoded = discard.encode().expect("discard payload");
        assert_eq!(encoded[4 + "wolf_fang".len()], u8::from(count.is_some()));
        assert_eq!(
            DiscardItemCommandPayload::decode(&encoded).expect("decode discard"),
            discard
        );
    }

    for socket in [None, Some(0), Some(3), Some(u32::MAX)] {
        let equip = EquipBagCommandPayload {
            item_id: "wolfhide_satchel".to_owned(),
            socket,
        };
        let encoded = equip.encode().expect("equip-bag payload");
        assert_eq!(
            EquipBagCommandPayload::decode(&encoded).expect("decode equip-bag"),
            equip
        );
    }

    let unequip = UnequipBagCommandPayload { socket: 3 };
    assert_eq!(
        UnequipBagCommandPayload::decode(&unequip.encode()).expect("decode unequip-bag"),
        unequip
    );
}

#[test]
fn variable_payload_validation_rejects_noncanonical_or_malformed_bytes() {
    assert_eq!(
        AbandonQuestCommandPayload {
            quest_id: "x".repeat(257),
        }
        .encode(),
        Err(ProtocolError::CollectionTooLarge {
            context: "AbandonQuestCommandPayload.quest_id",
            actual: 257,
            maximum: 256,
        })
    );
    assert_eq!(
        AbandonQuestCommandPayload::decode(&[1, 0, 0, 0, 0xff]),
        Err(ProtocolError::InvalidUtf8 {
            context: "AbandonQuestCommandPayload.quest_id",
        })
    );
    assert_eq!(
        AbandonQuestCommandPayload::decode(&[3, 0, 0, 0, b'a']),
        Err(ProtocolError::TruncatedPayload {
            context: "AbandonQuestCommandPayload.quest_id",
            needed: 3,
            remaining: 1,
        })
    );
    assert_eq!(
        AbandonQuestCommandPayload::decode(&[0, 0, 0, 0, 7]),
        Err(ProtocolError::TrailingPayload { remaining: 1 })
    );
    assert_eq!(
        DiscardItemCommandPayload::decode(&[0, 0, 0, 0, 2]),
        Err(ProtocolError::InvalidBoolean(2))
    );
    assert_eq!(
        DiscardItemCommandPayload::decode(&[0, 0, 0, 0, 1, 9]),
        Err(ProtocolError::TruncatedPayload {
            context: "command payload optional u32",
            needed: 4,
            remaining: 1,
        })
    );
    assert_eq!(
        validate_command_payload(DISCARD_ITEM_COMMAND_ID, &[0; 266]),
        Err(ProtocolError::InvalidCommandPayloadLengthRange {
            command_id: DISCARD_ITEM_COMMAND_ID,
            actual: 266,
            minimum: 5,
            maximum: 265,
        })
    );

    assert_eq!(
        LockpickEngageCommandPayload {
            object_id: 41,
            ante: 0,
        }
        .encode(),
        Err(ProtocolError::InvalidLockpickAnte(0))
    );
    assert_eq!(
        LockpickEngageCommandPayload::decode(&[0; 9]),
        Err(ProtocolError::InvalidLockpickAnte(0))
    );
    assert_eq!(
        LockpickActionCommandPayload::decode(&[0, 9]),
        Err(ProtocolError::InvalidLockpickAction(9))
    );
    assert_eq!(
        LockpickActionCommandPayload::decode(&[2, 0]),
        Err(ProtocolError::InvalidBoolean(2))
    );
    assert_eq!(
        LockpickActionCommandPayload::decode(&[1, 3, 0, 0, 0, b'x', 0]),
        Err(ProtocolError::TruncatedPayload {
            context: "LockpickActionCommandPayload.session_id",
            needed: 3,
            remaining: 2,
        })
    );
    assert_eq!(
        LockpickAbortCommandPayload::decode(&[0, 7]),
        Err(ProtocolError::TrailingPayload { remaining: 1 })
    );
    assert_eq!(
        LockpickAbortCommandPayload {
            session_id: Some("x".repeat(257)),
        }
        .encode(),
        Err(ProtocolError::CollectionTooLarge {
            context: "LockpickAbortCommandPayload.session_id",
            actual: 257,
            maximum: 256,
        })
    );
}

#[test]
fn payload_validation_rejects_noncanonical_or_unported_commands() {
    assert_eq!(
        CastSlotCommandPayload::decode(&[0, 1]),
        Err(ProtocolError::InvalidCommandPayloadLength {
            command_id: CAST_SLOT_COMMAND_ID,
            actual: 2,
            expected: 4,
        })
    );
    assert_eq!(validate_command_payload(ATTACK_COMMAND_ID, &[]), Ok(()));
    assert_eq!(
        validate_command_payload(ATTACK_COMMAND_ID, &[1]),
        Err(ProtocolError::InvalidCommandPayloadLength {
            command_id: ATTACK_COMMAND_ID,
            actual: 1,
            expected: 0,
        })
    );
    assert_eq!(
        TargetCommandPayload::decode(&[1, 2]),
        Err(ProtocolError::InvalidCommandPayloadLength {
            command_id: TARGET_COMMAND_ID,
            actual: 2,
            expected: 8,
        })
    );
    assert_eq!(
        TargetCommandPayload { target_id: Some(0) }.encode(),
        Err(ProtocolError::InvalidEntityId {
            context: "TargetCommandPayload.target_id",
        })
    );
    assert_eq!(validate_command_payload(INTERACT_COMMAND_ID, &[]), Ok(()));
    assert_eq!(
        validate_command_payload(13, &[]),
        Err(ProtocolError::UnsupportedCommandPayload(13))
    );
}

#[test]
fn talent_loadout_index_payloads_use_the_same_bounded_u32_contract() {
    let switch = SwitchLoadoutCommandPayload { index: 7 };
    let delete = DeleteLoadoutCommandPayload { index: 3 };

    assert_eq!(
        switch.encode().expect("switch payload"),
        7_u32.to_le_bytes()
    );
    assert_eq!(
        delete.encode().expect("delete payload"),
        3_u32.to_le_bytes()
    );
    assert_eq!(
        SwitchLoadoutCommandPayload::decode(&7_u32.to_le_bytes()).expect("decode switch"),
        switch
    );
    assert_eq!(
        DeleteLoadoutCommandPayload::decode(&3_u32.to_le_bytes()).expect("decode delete"),
        delete
    );
    assert_eq!(
        command_payload_descriptor(SWITCH_LOADOUT_COMMAND_ID).map(|descriptor| {
            (
                descriptor.name,
                descriptor.kind,
                descriptor.fixed_byte_length(),
            )
        }),
        Some(("switchLoadout", CommandPayloadKind::U32Index, Some(4)))
    );
    assert_eq!(
        command_payload_descriptor(DELETE_LOADOUT_COMMAND_ID).map(|descriptor| {
            (
                descriptor.name,
                descriptor.kind,
                descriptor.fixed_byte_length(),
            )
        }),
        Some(("deleteLoadout", CommandPayloadKind::U32Index, Some(4)))
    );
    assert_eq!(
        SwitchLoadoutCommandPayload { index: 10 }.encode(),
        Err(ProtocolError::InvalidTalentLoadoutIndex(10))
    );
    assert_eq!(
        validate_command_payload(DELETE_LOADOUT_COMMAND_ID, &10_u32.to_le_bytes()),
        Err(ProtocolError::InvalidTalentLoadoutIndex(10))
    );
}

#[test]
fn resurrection_response_payload_uses_the_source_boolean_field() {
    for (accept, wire) in [(false, 0), (true, 1)] {
        let payload = ResurrectRespondCommandPayload { accept };
        assert_eq!(payload.encode(), [wire]);
        assert_eq!(
            ResurrectRespondCommandPayload::decode(&[wire])
                .expect("resurrection response must decode"),
            payload
        );
    }
    assert_eq!(
        validate_command_payload(RESURRECT_RESPOND_COMMAND_ID, &[2]),
        Err(ProtocolError::InvalidBoolean(2))
    );
}

#[test]
fn vale_cup_payloads_preserve_all_closed_source_transport_values() {
    let queue = ValeCupQueueCommandPayload {
        bracket: ValeCupBracket::Five,
        nation: ValeCupNation::Copperdig,
        role: ValeCupRole::Keeper,
        enter_as_guild: true,
    };
    assert_eq!(
        queue.encode().expect("Vale Cup queue payload"),
        [5, 7, 3, 1]
    );
    assert_eq!(
        ValeCupQueueCommandPayload::decode(&[5, 7, 3, 1]).expect("decode Vale Cup queue"),
        queue
    );

    for (bracket, expected) in [
        (ValeCupBracket::One, [1]),
        (ValeCupBracket::Two, [2]),
        (ValeCupBracket::Three, [3]),
        (ValeCupBracket::Four, [4]),
        (ValeCupBracket::Five, [5]),
    ] {
        let payload = ValeCupPracticeCommandPayload { bracket };
        assert_eq!(
            payload.encode().expect("Vale Cup practice payload"),
            expected
        );
        assert_eq!(
            ValeCupPracticeCommandPayload::decode(&expected).expect("decode Vale Cup practice"),
            payload
        );
    }

    for (nation, code) in [
        (ValeCupNation::Vale, 0),
        (ValeCupNation::Mirefen, 1),
        (ValeCupNation::Thornpeak, 2),
        (ValeCupNation::Coliseum, 3),
        (ValeCupNation::Choir, 4),
        (ValeCupNation::Ogre, 5),
        (ValeCupNation::Moon, 6),
        (ValeCupNation::Copperdig, 7),
    ] {
        let payload = ValeCupQueueCommandPayload {
            bracket: ValeCupBracket::One,
            nation,
            role: ValeCupRole::Allrounder,
            enter_as_guild: false,
        };
        assert_eq!(
            payload.encode().expect("Vale Cup nation payload"),
            [1, code, 0, 0]
        );
    }

    for (role, expected) in [
        (ValeCupRole::Allrounder, [0]),
        (ValeCupRole::Striker, [1]),
        (ValeCupRole::Sweeper, [2]),
        (ValeCupRole::Keeper, [3]),
    ] {
        let payload = ValeCupRoleCommandPayload { role };
        assert_eq!(payload.encode().expect("Vale Cup role payload"), expected);
        assert_eq!(
            ValeCupRoleCommandPayload::decode(&expected).expect("decode Vale Cup role"),
            payload
        );
    }

    let bet = ValeCupBetCommandPayload {
        side: ValeCupSide::B,
        amount: 120.75,
    };
    let mut expected_bet = vec![1];
    expected_bet.extend_from_slice(&120.75_f64.to_le_bytes());
    assert_eq!(
        bet.clone().encode().expect("Vale Cup bet payload").to_vec(),
        expected_bet
    );
    assert_eq!(
        ValeCupBetCommandPayload::decode(&expected_bet).expect("decode Vale Cup bet"),
        bet
    );

    assert_eq!(
        validate_command_payload(VALE_CUP_QUEUE_COMMAND_ID, &[0, 0, 0, 0]),
        Err(ProtocolError::InvalidValeCupBracket(0))
    );
    assert_eq!(
        validate_command_payload(VALE_CUP_QUEUE_COMMAND_ID, &[1, 8, 0, 0]),
        Err(ProtocolError::InvalidValeCupNation(8))
    );
    assert_eq!(
        validate_command_payload(VALE_CUP_QUEUE_COMMAND_ID, &[1, 0, 4, 0]),
        Err(ProtocolError::InvalidValeCupRole(4))
    );
    assert_eq!(
        validate_command_payload(VALE_CUP_QUEUE_COMMAND_ID, &[1, 0, 0, 2]),
        Err(ProtocolError::InvalidBoolean(2))
    );
    assert_eq!(
        validate_command_payload(VALE_CUP_ROLE_COMMAND_ID, &[4]),
        Err(ProtocolError::InvalidValeCupRole(4))
    );
    assert_eq!(
        validate_command_payload(VALE_CUP_PRACTICE_COMMAND_ID, &[6]),
        Err(ProtocolError::InvalidValeCupBracket(6))
    );
    assert_eq!(
        validate_command_payload(VALE_CUP_BET_COMMAND_ID, &[2; 9]),
        Err(ProtocolError::InvalidValeCupSide(2))
    );
    assert!(matches!(
        ValeCupBetCommandPayload {
            side: ValeCupSide::A,
            amount: f64::NAN,
        }
        .encode(),
        Err(ProtocolError::NonFinite {
            field: "ValeCupBetCommandPayload.amount",
            ..
        })
    ));
}

#[test]
fn mail_id_payloads_preserve_the_source_number_without_mail_policy() {
    for (action, command_id, mail_id) in [
        (MailAction::Take, MAIL_TAKE_COMMAND_ID, 41.5),
        (MailAction::Delete, MAIL_DELETE_COMMAND_ID, -12.0),
        (MailAction::MarkRead, MAIL_READ_COMMAND_ID, 0.0),
    ] {
        let payload = MailIdCommandPayload { mail_id };
        let encoded = payload.clone().encode(action).expect("mail id payload");
        assert_eq!(encoded, mail_id.to_le_bytes());
        assert_eq!(
            MailIdCommandPayload::decode(&encoded, action).expect("decode mail id"),
            payload
        );
        assert_eq!(validate_command_payload(command_id, &encoded), Ok(()));
    }

    assert!(matches!(
        MailIdCommandPayload { mail_id: f64::NAN }.encode(MailAction::Take),
        Err(ProtocolError::NonFinite {
            field: "MailIdCommandPayload.mail_id",
            ..
        })
    ));
    assert!(matches!(
        validate_command_payload(MAIL_DELETE_COMMAND_ID, &f64::INFINITY.to_le_bytes()),
        Err(ProtocolError::NonFinite {
            field: "MailIdCommandPayload.mail_id",
            ..
        })
    ));
}

#[test]
fn bank_payloads_preserve_source_numbers_and_optional_count_without_bank_policy() {
    for (action, command_id, slot, count) in [
        (
            BankAction::Deposit,
            BANK_DEPOSIT_COMMAND_ID,
            7.5,
            Some(3.25),
        ),
        (BankAction::Withdraw, BANK_WITHDRAW_COMMAND_ID, -2.0, None),
    ] {
        let payload = BankSlotCommandPayload { slot, count };
        let encoded = payload.clone().encode(action).expect("bank payload");
        let mut expected = slot.to_le_bytes().to_vec();
        match count {
            None => expected.push(0),
            Some(count) => {
                expected.push(1);
                expected.extend_from_slice(&count.to_le_bytes());
            }
        }
        assert_eq!(encoded, expected);
        assert_eq!(
            BankSlotCommandPayload::decode(&encoded, action).expect("decode bank payload"),
            payload
        );
        assert_eq!(validate_command_payload(command_id, &encoded), Ok(()));
    }

    assert!(matches!(
        BankSlotCommandPayload {
            slot: f64::NAN,
            count: None,
        }
        .encode(BankAction::Deposit),
        Err(ProtocolError::NonFinite {
            field: "BankSlotCommandPayload.slot",
            ..
        })
    ));
    assert!(matches!(
        BankSlotCommandPayload {
            slot: 0.0,
            count: Some(f64::NEG_INFINITY),
        }
        .encode(BankAction::Withdraw),
        Err(ProtocolError::NonFinite {
            field: "BankSlotCommandPayload.count",
            ..
        })
    ));
    let mut invalid_presence = 0.0_f64.to_le_bytes().to_vec();
    invalid_presence.push(2);
    assert_eq!(
        validate_command_payload(BANK_DEPOSIT_COMMAND_ID, &invalid_presence),
        Err(ProtocolError::InvalidBoolean(2))
    );
}

#[test]
fn dungeon_finder_payloads_preserve_bounded_source_shapes_without_finder_policy() {
    let roles = DungeonFinderRolesPayload {
        roles: vec![
            DungeonFinderRole::Tank,
            DungeonFinderRole::Healer,
            DungeonFinderRole::Dps,
        ],
    };
    assert_eq!(roles.clone().encode().expect("roles payload"), [3, 0, 1, 2]);
    assert_eq!(
        DungeonFinderRolesPayload::decode(&[3, 0, 1, 2]).expect("decode roles"),
        roles
    );

    let activities = DungeonFinderActivitiesPayload {
        activities: vec![
            "hollow_crypt_normal".to_owned(),
            "nythraxis_raid_normal".to_owned(),
        ],
    };
    let encoded_activities = activities.clone().encode().expect("activities payload");
    assert_eq!(encoded_activities[0], 2);
    assert_eq!(
        DungeonFinderActivitiesPayload::decode(&encoded_activities).expect("decode activities"),
        activities
    );

    let listing = DungeonFinderListingPayload {
        activity: "hollow_crypt_normal".to_owned(),
        tags: vec![
            DungeonFinderListingTag::FirstRun,
            DungeonFinderListingTag::FastRun,
        ],
    };
    let encoded_listing = listing.clone().encode().expect("listing payload");
    assert_eq!(
        DungeonFinderListingPayload::decode(&encoded_listing).expect("decode listing"),
        listing
    );

    let listing_id = DungeonFinderListingIdPayload { listing_id: -17.5 };
    assert_eq!(
        listing_id.clone().encode().expect("listing id payload"),
        (-17.5_f64).to_le_bytes()
    );
    assert_eq!(
        DungeonFinderListingIdPayload::decode(&(-17.5_f64).to_le_bytes())
            .expect("decode listing id"),
        listing_id
    );

    let response = DungeonFinderApplicationResponsePayload {
        applicant_id: 3.25,
        accept: true,
    };
    let mut expected_response = 3.25_f64.to_le_bytes().to_vec();
    expected_response.push(1);
    assert_eq!(
        response
            .clone()
            .encode()
            .expect("application response payload")
            .to_vec(),
        expected_response
    );
    assert_eq!(
        DungeonFinderApplicationResponsePayload::decode(&expected_response)
            .expect("decode application response"),
        response
    );

    assert_eq!(
        validate_command_payload(DUNGEON_FINDER_ROLES_COMMAND_ID, &[1, 3]),
        Err(ProtocolError::InvalidDungeonFinderRole(3))
    );
    assert_eq!(
        validate_command_payload(DUNGEON_FINDER_LIST_CREATE_COMMAND_ID, &[0, 1, 5]),
        Err(ProtocolError::InvalidDungeonFinderListingTag(5))
    );
    assert_eq!(
        validate_command_payload(DUNGEON_FINDER_QUEUE_COMMAND_ID, &[17]),
        Err(ProtocolError::CollectionTooLarge {
            context: "DungeonFinderActivitiesPayload.activities",
            actual: 17,
            maximum: 16,
        })
    );
    assert!(matches!(
        DungeonFinderActivitiesPayload {
            activities: vec!["a".repeat(65)],
        }
        .encode(),
        Err(ProtocolError::CollectionTooLarge {
            context: "DungeonFinderActivitiesPayload.activity",
            actual: 65,
            maximum: 64,
        })
    ));
    assert!(matches!(
        DungeonFinderListingIdPayload {
            listing_id: f64::NAN,
        }
        .encode(),
        Err(ProtocolError::NonFinite {
            field: "DungeonFinderListingIdPayload.listing_id",
            ..
        })
    ));
    let mut invalid_response = 1.0_f64.to_le_bytes().to_vec();
    invalid_response.push(2);
    assert_eq!(
        validate_command_payload(
            DUNGEON_FINDER_APPLICATION_RESPONSE_COMMAND_ID,
            &invalid_response
        ),
        Err(ProtocolError::InvalidBoolean(2))
    );
    assert_eq!(
        validate_command_payload(DUNGEON_FINDER_APPLY_COMMAND_ID, &(-17.5_f64).to_le_bytes()),
        Ok(())
    );
}

#[test]
fn world_object_payloads_preserve_finite_source_number_shapes() {
    let payload = WorldObjectIdPayload { object_id: -17.5 };
    let encoded = payload
        .clone()
        .encode(WorldObjectAction::Loot)
        .expect("loot payload");
    assert_eq!(encoded, (-17.5_f64).to_le_bytes());
    assert_eq!(
        WorldObjectIdPayload::decode(&encoded, WorldObjectAction::Loot).expect("decode loot"),
        payload
    );

    for (command_id, action) in [
        (LOOT_COMMAND_ID, WorldObjectAction::Loot),
        (PICKUP_COMMAND_ID, WorldObjectAction::Pickup),
        (AUTO_LOOT_COMMAND_ID, WorldObjectAction::AutoLoot),
        (DELVE_INTERACT_COMMAND_ID, WorldObjectAction::DelveInteract),
        (
            COLLECT_DELVE_CHEST_LOOT_COMMAND_ID,
            WorldObjectAction::CollectDelveChestLoot,
        ),
    ] {
        let bytes = WorldObjectIdPayload { object_id: 0.0 }
            .encode(action)
            .expect("world-object payload");
        assert_eq!(bytes, 0.0_f64.to_le_bytes());
        assert_eq!(validate_command_payload(command_id, &bytes), Ok(()));
    }

    assert!(matches!(
        WorldObjectIdPayload {
            object_id: f64::NAN,
        }
        .encode(WorldObjectAction::Pickup),
        Err(ProtocolError::NonFinite {
            field: "WorldObjectIdPayload.object_id",
            ..
        })
    ));
}

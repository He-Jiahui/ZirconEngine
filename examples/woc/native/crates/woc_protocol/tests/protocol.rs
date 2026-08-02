use woc_protocol::{
    canonical_pairs, command_descriptor, decode_frame, encode_frame, require_finite,
    validate_command_payload, Command, CommandKind, DecodeLimits, EnterDungeonCommandPayload,
    EntityRef, Event, FixedTickInput, Frame, HarvestCorpseCommandPayload, MessageKind,
    MovementFrame, MovementInputFlags, NetworkEnvelope, OfflineSessionBootstrap,
    OfflineWeaponSkinAccount, ProtocolError, RlActionBatch, RlObservationBatch, SaveState,
    TownFocusAllocationEntry, TownFocusCommandPayload, WorldSnapshot, COMMAND_CATALOG,
    COMMAND_PAYLOAD_SCHEMA_SHA256, ENTER_DUNGEON_COMMAND_ID, FRAME_HEADER_BYTES,
    OFFLINE_SESSION_BOOTSTRAP_VERSION, OFFLINE_WEAPON_SKIN_COUNT, PROTOCOL_VERSION,
    SCHEMA_FINGERPRINT_BYTES, STANDARD_OFFLINE_WORLD_SEED, WORLD_STATE_FORMAT,
    WORLD_STATE_SCHEMA_VERSION,
};

#[test]
fn enter_dungeon_payload_preserves_authoritative_dungeon_identity() {
    let payload = EnterDungeonCommandPayload {
        dungeon_id: "unlisted_authoritative_dungeon".to_owned(),
    };
    let encoded = payload.clone().encode().expect("payload encodes");
    assert_eq!(
        encoded,
        [
            30, 0, 0, 0, b'u', b'n', b'l', b'i', b's', b't', b'e', b'd', b'_', b'a', b'u', b't',
            b'h', b'o', b'r', b'i', b't', b'a', b't', b'i', b'v', b'e', b'_', b'd', b'u', b'n',
            b'g', b'e', b'o', b'n',
        ]
    );
    assert_eq!(EnterDungeonCommandPayload::decode(&encoded), Ok(payload));

    let trailing = [4, 0, 0, 0, b't', b'e', b's', b't', 0];
    assert_eq!(
        EnterDungeonCommandPayload::decode(&trailing),
        Err(ProtocolError::TrailingPayload { remaining: 1 })
    );
    assert!(matches!(
        EnterDungeonCommandPayload {
            dungeon_id: "a".repeat(257),
        }
        .encode(),
        Err(ProtocolError::CollectionTooLarge { .. })
    ));
    assert_eq!(ENTER_DUNGEON_COMMAND_ID, 112);
}

#[test]
fn corpse_harvest_payload_roundtrips_and_rejects_noncanonical_component_codes() {
    let payload = HarvestCorpseCommandPayload {
        target_id: 0x0807_0605_0403_0201,
        component_codes: vec![1, 0, 3],
    };
    let encoded = payload.clone().encode().expect("payload encodes");
    assert_eq!(encoded, vec![1, 2, 3, 4, 5, 6, 7, 8, 3, 1, 0, 3]);
    assert_eq!(HarvestCorpseCommandPayload::decode(&encoded), Ok(payload));

    let invalid_code = HarvestCorpseCommandPayload {
        target_id: 1,
        component_codes: vec![9],
    };
    assert_eq!(
        invalid_code.encode(),
        Err(ProtocolError::InvalidCorpseHarvestComponentCode(9))
    );
    let too_many = HarvestCorpseCommandPayload {
        target_id: 1,
        component_codes: vec![1, 2, 3, 4],
    };
    assert!(matches!(
        too_many.encode(),
        Err(ProtocolError::CollectionTooLarge { .. })
    ));

    let invalid_count = [0, 0, 0, 0, 0, 0, 0, 0, 4, 1, 2, 3];
    assert_eq!(
        validate_command_payload(13, &invalid_count),
        Err(ProtocolError::InvalidCorpseHarvestComponentCount(4))
    );
    let trailing = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0];
    assert_eq!(
        HarvestCorpseCommandPayload::decode(&trailing),
        Err(ProtocolError::TrailingPayload { remaining: 1 })
    );
}

#[test]
fn town_focus_payload_roundtrips_integer_allocation_in_source_order() {
    let payload = TownFocusCommandPayload {
        allocation: vec![
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
            TownFocusAllocationEntry {
                component: "invalid".to_owned(),
                points: -1,
            },
        ],
    };
    let encoded = payload.clone().encode().expect("payload encodes");
    assert_eq!(&encoded[..2], &4u16.to_le_bytes());
    assert_eq!(TownFocusCommandPayload::decode(&encoded), Ok(payload));

    let empty = TownFocusCommandPayload {
        allocation: Vec::new(),
    };
    assert_eq!(empty.clone().encode(), Ok(0u16.to_le_bytes().to_vec()));
    assert_eq!(TownFocusCommandPayload::decode(&[0, 0]), Ok(empty));
}

#[test]
fn generated_command_catalog_is_contiguous_and_pins_transport_metadata() {
    assert_eq!(COMMAND_CATALOG.len(), 165);
    assert_eq!(command_descriptor(0).map(|row| row.name), Some("castSlot"));
    assert_eq!(
        command_descriptor(163).map(|row| row.name),
        Some("selectTalentRow")
    );
    assert_eq!(
        command_descriptor(164).map(|row| row.name),
        Some("resurrect_respond")
    );
    assert!(command_descriptor(165).is_none());
    assert_eq!(
        COMMAND_CATALOG
            .iter()
            .filter(|row| row.kind == CommandKind::DispatchOnly)
            .count(),
        9
    );
    assert_eq!(
        command_descriptor(42).and_then(|row| row.facet),
        Some("IWorldParty")
    );
    assert!(COMMAND_CATALOG
        .iter()
        .enumerate()
        .all(|(index, row)| usize::from(row.id) == index));
}

#[test]
fn current_catalog_exposes_the_typed_talent_cosmetic_and_spec_payloads() {
    assert_eq!(validate_command_payload(31, &[0, 7]), Ok(()));
    assert_eq!(validate_command_payload(95, &[0; 14]), Ok(()));
    assert_eq!(validate_command_payload(96, &[]), Ok(()));
    assert_eq!(validate_command_payload(97, &[1, 0]), Ok(()));
    assert_eq!(validate_command_payload(99, &[0; 4]), Ok(()));
    assert_eq!(validate_command_payload(100, &[9, 0, 0, 0]), Ok(()));
    assert_eq!(validate_command_payload(162, &[]), Ok(()));
    assert_eq!(
        validate_command_payload(162, &[1]),
        Err(ProtocolError::InvalidCommandPayloadLength {
            command_id: 162,
            expected: 0,
            actual: 1,
        })
    );
    assert_eq!(validate_command_payload(163, &[5, 0, 0]), Ok(()));
    assert_eq!(validate_command_payload(164, &[1]), Ok(()));
    assert_eq!(
        validate_command_payload(164, &[2]),
        Err(ProtocolError::InvalidBoolean(2))
    );
}

#[test]
fn reference_identity_pins_the_opaque_world_state_compatibility_key() {
    assert_eq!(WORLD_STATE_FORMAT, "WOS83");
    assert_eq!(WORLD_STATE_SCHEMA_VERSION, 83);
    assert_eq!(woc_protocol::REFERENCE_IDENTITY.world_state_format, "WOS83");
    assert_eq!(
        woc_protocol::REFERENCE_IDENTITY.world_state_schema_version,
        83
    );
    assert_eq!(
        woc_protocol::REFERENCE_IDENTITY.command_payload_schema_sha256,
        COMMAND_PAYLOAD_SCHEMA_SHA256
    );
}

#[test]
fn frame_round_trip_preserves_arbitrary_bytes() {
    let frame = Frame::new(
        MessageKind::FixedTickInput,
        vec![0, 0xff, 0x80, b'{', b'}', 0],
    );
    let encoded = encode_frame(&frame, DecodeLimits::default()).expect("frame must encode");
    let decoded = decode_frame(&encoded, DecodeLimits::default()).expect("frame must decode");
    assert_eq!(decoded, frame);
}

#[test]
fn fixed_tick_and_world_snapshot_payloads_round_trip_losslessly() {
    let mut weapon_skin_account = OfflineWeaponSkinAccount::default();
    weapon_skin_account.owned[0] = true;
    weapon_skin_account.loadout_codes[0] = 1;
    let input = FixedTickInput {
        tick: 42,
        commands: vec![Command {
            command_id: 7,
            actor: EntityRef {
                id: 0x0102_0304_0506_0708,
                generation: 9,
            },
            sequence: 11,
            payload: vec![0, 0xff, 0x80],
        }],
        wall_time_forbidden: true,
        committed_state: vec![0xaa, 0, 0xff],
        committed_state_digest: 0x1122_3344,
        generation: 3,
        movement_frames: vec![MovementFrame {
            actor: EntityRef {
                id: 0x1112_1314_1516_1718,
                generation: 19,
            },
            sequence: 20,
            flags: MovementInputFlags {
                forward: true,
                turn_left: true,
                jump: true,
                ..MovementInputFlags::default()
            },
            facing: Some(-0.75),
        }],
        offline_bootstrap: Some(OfflineSessionBootstrap {
            launch_version: OFFLINE_SESSION_BOOTSTRAP_VERSION,
            world_seed: STANDARD_OFFLINE_WORLD_SEED,
            player_class: 1,
            player_name: "Vale".to_string(),
            skin_variant: 2,
            weapon_skin_account,
        }),
    };
    let input_bytes = input.encode_payload().expect("tick input must encode");
    assert_eq!(
        FixedTickInput::decode_payload(&input_bytes).expect("tick input must decode"),
        input
    );

    let snapshot = WorldSnapshot {
        tick: 42,
        state_digest: 0x1234_5678,
        event_digest: 0x90ab_cdef,
        state: vec![0, 0xff, 1, 2, 3],
        events: vec![Event {
            event_id: 5,
            sequence: 12,
            payload: vec![0xfe, 0, 0xfd],
        }],
    };
    let snapshot_bytes = snapshot.encode_payload().expect("snapshot must encode");
    assert_eq!(
        WorldSnapshot::decode_payload(&snapshot_bytes).expect("snapshot must decode"),
        snapshot
    );
}

#[test]
fn fixed_tick_movement_frames_use_a_canonical_binary_contract() {
    let input = FixedTickInput {
        tick: 7,
        commands: vec![],
        wall_time_forbidden: true,
        committed_state: vec![],
        committed_state_digest: 0x1122_3344,
        generation: 9,
        movement_frames: vec![MovementFrame {
            actor: EntityRef {
                id: 0x0102_0304_0506_0708,
                generation: 3,
            },
            sequence: 4,
            flags: MovementInputFlags {
                forward: true,
                turn_right: true,
                strafe_left: true,
                jump: true,
                ..MovementInputFlags::default()
            },
            facing: Some(0.5),
        }],
        offline_bootstrap: None,
    };
    let bytes = input
        .encode_payload()
        .expect("movement tick input must encode");
    assert_eq!(bytes.len(), 69);
    assert_eq!(&bytes[29..33], &1_u32.to_le_bytes());
    assert_eq!(
        &bytes[33..65],
        &[
            0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x01, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xe0, 0x3f,
        ]
    );
    assert_eq!(&bytes[65..69], &0_u32.to_le_bytes());
    assert_eq!(
        FixedTickInput::decode_payload(&bytes).expect("movement tick input must decode"),
        input
    );

    let duplicate_actor = FixedTickInput {
        movement_frames: vec![
            MovementFrame {
                actor: EntityRef {
                    id: 9,
                    generation: 1,
                },
                sequence: 1,
                flags: MovementInputFlags::default(),
                facing: None,
            },
            MovementFrame {
                actor: EntityRef {
                    id: 9,
                    generation: 1,
                },
                sequence: 2,
                flags: MovementInputFlags::default(),
                facing: None,
            },
        ],
        ..input.clone()
    };
    assert!(matches!(
        duplicate_actor.encode_payload(),
        Err(ProtocolError::InvalidMovementInput(_))
    ));

    let mut invalid_boolean = bytes.clone();
    invalid_boolean[49] = 2;
    assert!(matches!(
        FixedTickInput::decode_payload(&invalid_boolean),
        Err(ProtocolError::InvalidBoolean(2))
    ));

    let mut absent_facing_with_value = bytes.clone();
    absent_facing_with_value[56] = 0;
    assert!(matches!(
        FixedTickInput::decode_payload(&absent_facing_with_value),
        Err(ProtocolError::InvalidMovementInput(_))
    ));

    let canonical_pair = FixedTickInput {
        movement_frames: vec![
            MovementFrame {
                actor: EntityRef {
                    id: 9,
                    generation: 1,
                },
                sequence: 1,
                flags: MovementInputFlags::default(),
                facing: None,
            },
            MovementFrame {
                actor: EntityRef {
                    id: 3,
                    generation: 2,
                },
                sequence: 2,
                flags: MovementInputFlags::default(),
                facing: None,
            },
        ],
        ..input
    };
    let mut noncanonical_pair = canonical_pair
        .encode_payload()
        .expect("pair must encode in canonical actor order");
    let first_frame = noncanonical_pair[33..65].to_vec();
    noncanonical_pair.copy_within(65..97, 33);
    noncanonical_pair[65..97].copy_from_slice(&first_frame);
    assert!(matches!(
        FixedTickInput::decode_payload(&noncanonical_pair),
        Err(ProtocolError::InvalidMovementInput(_))
    ));
}

#[test]
fn offline_bootstrap_rejects_nonstandard_seed_name_and_class_skin_mismatches() {
    let bootstrap = OfflineSessionBootstrap {
        launch_version: OFFLINE_SESSION_BOOTSTRAP_VERSION,
        world_seed: STANDARD_OFFLINE_WORLD_SEED + 1,
        player_class: 1,
        player_name: "Vale".to_string(),
        skin_variant: 0,
        weapon_skin_account: OfflineWeaponSkinAccount::default(),
    };
    assert!(matches!(
        bootstrap.encode_payload(),
        Err(ProtocolError::InvalidOfflineBootstrap(_))
    ));

    let invalid_name = OfflineSessionBootstrap {
        world_seed: STANDARD_OFFLINE_WORLD_SEED,
        player_name: "1Vale".to_string(),
        ..bootstrap
    };
    assert!(matches!(
        invalid_name.encode_payload(),
        Err(ProtocolError::InvalidOfflineBootstrap(_))
    ));

    let last_paladin_skin = OfflineSessionBootstrap {
        world_seed: STANDARD_OFFLINE_WORLD_SEED,
        player_class: 3,
        player_name: "Vale".to_string(),
        skin_variant: 7,
        ..bootstrap
    };
    assert!(last_paladin_skin.encode_payload().is_ok());

    let invalid_class_skin = OfflineSessionBootstrap {
        world_seed: STANDARD_OFFLINE_WORLD_SEED,
        player_class: 3,
        player_name: "Vale".to_string(),
        skin_variant: 8,
        ..bootstrap
    };
    assert!(matches!(
        invalid_class_skin.encode_payload(),
        Err(ProtocolError::InvalidOfflineBootstrap(_))
    ));

    let unowned_weapon_skin = OfflineSessionBootstrap {
        world_seed: STANDARD_OFFLINE_WORLD_SEED,
        player_class: 1,
        player_name: "Vale".to_string(),
        skin_variant: 0,
        weapon_skin_account: OfflineWeaponSkinAccount {
            owned: [false; 29],
            loadout_codes: [1, 0, 0, 0, 0, 0, 0, 0],
        },
        ..bootstrap
    };
    assert!(matches!(
        unowned_weapon_skin.encode_payload(),
        Err(ProtocolError::InvalidOfflineBootstrap(_))
    ));

    let mismatched_weapon_skin_type = OfflineSessionBootstrap {
        world_seed: STANDARD_OFFLINE_WORLD_SEED,
        player_class: 1,
        player_name: "Vale".to_string(),
        skin_variant: 0,
        weapon_skin_account: OfflineWeaponSkinAccount {
            owned: {
                let mut owned = [false; OFFLINE_WEAPON_SKIN_COUNT];
                owned[1] = true;
                owned
            },
            loadout_codes: [2, 0, 0, 0, 0, 0, 0, 0],
        },
        ..bootstrap
    };
    assert!(matches!(
        mismatched_weapon_skin_type.encode_payload(),
        Err(ProtocolError::InvalidOfflineBootstrap(_))
    ));

    let valid_axe_loadout = OfflineSessionBootstrap {
        world_seed: STANDARD_OFFLINE_WORLD_SEED,
        player_class: 1,
        player_name: "Vale".to_string(),
        skin_variant: 0,
        weapon_skin_account: OfflineWeaponSkinAccount {
            owned: {
                let mut owned = [false; OFFLINE_WEAPON_SKIN_COUNT];
                owned[1] = true;
                owned
            },
            loadout_codes: [0, 2, 0, 0, 0, 0, 0, 0],
        },
        ..bootstrap
    };
    let mut tampered_weapon_skin_type = valid_axe_loadout
        .encode_payload()
        .expect("matching axe skin loadout must encode");
    let loadout_offset = 2 + 4 + 1 + 4 + "Vale".len() + 2 + OFFLINE_WEAPON_SKIN_COUNT;
    tampered_weapon_skin_type[loadout_offset] = 2;
    tampered_weapon_skin_type[loadout_offset + 1] = 0;
    assert!(matches!(
        OfflineSessionBootstrap::decode_payload(&tampered_weapon_skin_type),
        Err(ProtocolError::InvalidOfflineBootstrap(_))
    ));
}

#[test]
fn offline_bootstrap_v2_carries_the_maximum_weapon_skin_account() {
    let mut weapon_skin_account = OfflineWeaponSkinAccount::default();
    weapon_skin_account.owned[0] = true;
    weapon_skin_account.loadout_codes[0] = 1;
    let bootstrap = OfflineSessionBootstrap {
        launch_version: OFFLINE_SESSION_BOOTSTRAP_VERSION,
        world_seed: STANDARD_OFFLINE_WORLD_SEED,
        player_class: 0,
        player_name: "Aaaaaaaaaaaaaaaa".to_string(),
        skin_variant: 0,
        weapon_skin_account,
    };
    let encoded_bootstrap = bootstrap
        .encode_payload()
        .expect("v2 bootstrap must encode");
    assert_eq!(encoded_bootstrap.len(), 66);
    assert_eq!(
        OfflineSessionBootstrap::decode_payload(&encoded_bootstrap)
            .expect("v2 bootstrap must decode"),
        bootstrap
    );

    let input = FixedTickInput {
        tick: 1,
        commands: Vec::new(),
        wall_time_forbidden: true,
        committed_state: Vec::new(),
        committed_state_digest: 0,
        generation: 0,
        movement_frames: Vec::new(),
        offline_bootstrap: Some(bootstrap),
    };
    assert_eq!(
        FixedTickInput::decode_payload(
            &input
                .encode_payload()
                .expect("v2 bootstrap reaches fixed tick input"),
        )
        .expect("fixed tick input decodes"),
        input
    );
}

#[test]
fn standalone_command_event_save_and_network_payloads_round_trip() {
    let command = Command {
        command_id: 42,
        actor: EntityRef {
            id: 99,
            generation: 4,
        },
        sequence: 12,
        payload: vec![0, 0xff, 8],
    };
    assert_eq!(
        Command::decode_payload(&command.encode_payload().expect("command must encode"))
            .expect("command must decode"),
        command
    );

    let event = Event {
        event_id: 9,
        sequence: 13,
        payload: vec![7, 0, 0xfe],
    };
    assert_eq!(
        Event::decode_payload(&event.encode_payload().expect("event must encode"))
            .expect("event must decode"),
        event
    );

    let save = SaveState {
        schema_fingerprint: SCHEMA_FINGERPRINT_BYTES,
        generation: 3,
        tick: 44,
        state: vec![0, 0xff, 1],
    };
    assert_eq!(
        SaveState::decode_payload(&save.encode_payload().expect("save must encode"))
            .expect("save must decode"),
        save
    );
    let mut mismatched_save = save.clone();
    mismatched_save.schema_fingerprint[0] ^= 0xff;
    assert!(matches!(
        mismatched_save.encode_payload(),
        Err(ProtocolError::SchemaMismatch { .. })
    ));

    let envelope = NetworkEnvelope {
        protocol_version: PROTOCOL_VERSION,
        kind: MessageKind::Command,
        sequence: 8,
        acknowledgement: 7,
        payload: vec![0, 0xff, 2],
    };
    assert_eq!(
        NetworkEnvelope::decode_payload(&envelope.encode_payload().expect("envelope must encode"))
            .expect("envelope must decode"),
        envelope
    );
    let mut mismatched_envelope = envelope;
    mismatched_envelope.protocol_version += 1;
    assert!(matches!(
        mismatched_envelope.encode_payload(),
        Err(ProtocolError::UnsupportedVersion { .. })
    ));
}

#[test]
fn rl_batches_round_trip_and_reject_non_partitioning_offsets() {
    let observations = RlObservationBatch {
        tick: 5,
        environment_ids: vec![10, 20],
        offsets: vec![0, 2, 5],
        observations: vec![1, 2, 3, 4, 5],
    };
    assert_eq!(
        RlObservationBatch::decode_payload(
            &observations
                .encode_payload()
                .expect("observations must encode")
        )
        .expect("observations must decode"),
        observations
    );

    let actions = RlActionBatch {
        tick: 6,
        environment_ids: vec![10, 20],
        offsets: vec![0, 1, 3],
        actions: vec![9, 8, 7],
    };
    assert_eq!(
        RlActionBatch::decode_payload(&actions.encode_payload().expect("actions must encode"))
            .expect("actions must decode"),
        actions
    );

    let invalid = RlActionBatch {
        offsets: vec![0, 3, 2],
        ..actions
    };
    assert!(matches!(
        invalid.encode_payload(),
        Err(ProtocolError::InvalidOffsets {
            context: "RlActionBatch"
        })
    ));
}

#[test]
fn typed_payload_decoder_rejects_truncation_invalid_boolean_and_trailing_bytes() {
    let input = FixedTickInput {
        tick: 1,
        commands: vec![],
        wall_time_forbidden: true,
        committed_state: vec![],
        committed_state_digest: 0,
        generation: 0,
        movement_frames: vec![],
        offline_bootstrap: None,
    };
    let encoded = input.encode_payload().expect("fixture must encode");
    assert!(matches!(
        FixedTickInput::decode_payload(&encoded[..encoded.len() - 1]),
        Err(ProtocolError::TruncatedPayload { .. })
    ));

    let mut invalid_boolean = encoded.clone();
    invalid_boolean[12] = 2;
    assert!(matches!(
        FixedTickInput::decode_payload(&invalid_boolean),
        Err(ProtocolError::InvalidBoolean(2))
    ));

    let mut trailing = encoded;
    trailing.push(0);
    assert!(matches!(
        FixedTickInput::decode_payload(&trailing),
        Err(ProtocolError::TrailingPayload { .. })
    ));
}

#[test]
fn fixed_tick_rejects_unknown_command_ids_on_encode_and_decode() {
    let invalid = FixedTickInput {
        tick: 1,
        commands: vec![Command {
            command_id: 165,
            actor: EntityRef {
                id: 1,
                generation: 1,
            },
            sequence: 1,
            payload: vec![],
        }],
        wall_time_forbidden: true,
        committed_state: vec![],
        committed_state_digest: 0,
        generation: 0,
        movement_frames: vec![],
        offline_bootstrap: None,
    };
    assert_eq!(
        invalid.encode_payload(),
        Err(ProtocolError::UnknownCommandId(165))
    );

    let mut encoded = FixedTickInput {
        tick: 1,
        commands: vec![Command {
            command_id: 7,
            actor: EntityRef {
                id: 1,
                generation: 1,
            },
            sequence: 1,
            payload: vec![],
        }],
        wall_time_forbidden: true,
        committed_state: vec![],
        committed_state_digest: 0,
        generation: 0,
        movement_frames: vec![],
        offline_bootstrap: None,
    }
    .encode_payload()
    .expect("known command fixture must encode");
    encoded[12..14].copy_from_slice(&165_u16.to_le_bytes());
    assert_eq!(
        FixedTickInput::decode_payload(&encoded),
        Err(ProtocolError::UnknownCommandId(165))
    );
}

#[test]
fn malformed_length_and_oversized_payload_are_rejected() {
    let limits = DecodeLimits {
        max_payload_bytes: 4,
    };
    let oversized = Frame::new(MessageKind::Command, vec![1, 2, 3, 4, 5]);
    assert!(matches!(
        encode_frame(&oversized, limits),
        Err(ProtocolError::PayloadTooLarge { .. })
    ));

    let valid = encode_frame(
        &Frame::new(MessageKind::Command, vec![1, 2, 3]),
        DecodeLimits::default(),
    )
    .expect("fixture must encode");
    let mut truncated = valid.clone();
    truncated.pop();
    assert!(matches!(
        decode_frame(&truncated, DecodeLimits::default()),
        Err(ProtocolError::LengthMismatch { .. })
    ));
    assert!(matches!(
        decode_frame(&valid[..FRAME_HEADER_BYTES - 1], DecodeLimits::default()),
        Err(ProtocolError::TruncatedHeader { .. })
    ));
}

#[test]
fn unknown_version_kind_and_schema_are_rejected() {
    let bytes = encode_frame(
        &Frame::new(MessageKind::Event, vec![]),
        DecodeLimits::default(),
    )
    .expect("fixture must encode");

    let mut version = bytes.clone();
    version[4..6].copy_from_slice(&2_u16.to_le_bytes());
    assert!(matches!(
        decode_frame(&version, DecodeLimits::default()),
        Err(ProtocolError::UnsupportedVersion { .. })
    ));

    let mut kind = bytes.clone();
    kind[6..8].copy_from_slice(&999_u16.to_le_bytes());
    assert!(matches!(
        decode_frame(&kind, DecodeLimits::default()),
        Err(ProtocolError::UnknownMessageKind(999))
    ));

    let mut schema = bytes;
    schema[8] ^= 0xff;
    assert!(matches!(
        decode_frame(&schema, DecodeLimits::default()),
        Err(ProtocolError::SchemaMismatch { .. })
    ));
}

#[test]
fn canonical_order_and_finite_policy_are_enforced() {
    let pairs = canonical_pairs(vec![("z", 1), ("a", 2), ("m", 3)]).expect("unique keys must sort");
    assert_eq!(pairs, vec![("a", 2), ("m", 3), ("z", 1)]);
    assert!(matches!(
        canonical_pairs(vec![("a", 1), ("a", 2)]),
        Err(ProtocolError::DuplicateCanonicalKey)
    ));
    assert_eq!(require_finite("position.x", 1.25), Ok(1.25));
    assert!(matches!(
        require_finite("position.x", f64::NAN),
        Err(ProtocolError::NonFinite { .. })
    ));
}

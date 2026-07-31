use woc_protocol::EntityRef;
use woc_runtime::{
    ActorAnimationInput, ActorAppearance, ActorPresentation, ActorTransform,
    BulkPresentationProjection, ClientPresentationProjection, ClientProjectionError,
    ClientWindowProjection, HudAction, HudActionId, HudCast, HudMeter, HudProjection,
    HudProjectionError, HudQuestObjective, HudQuestState, HudResource, HudResourceKind,
    HudTrackedQuest, HudUnit, HudUnitRole, InventoryWindowProjection, PresentationVec3,
    QuestLogEntryProjection, QuestLogObjectiveProjection, QuestLogWindowProjection,
    CLIENT_PRESENTATION_SCHEMA_VERSION,
};

fn entity(id: u64) -> EntityRef {
    EntityRef { id, generation: 3 }
}

fn actor(id: u64) -> ActorPresentation {
    ActorPresentation {
        entity: entity(id),
        template_id: format!("actor_{id}"),
        transform: ActorTransform {
            translation: PresentationVec3 {
                x: id as f32,
                y: 0.0,
                z: 0.0,
            },
            facing_radians: 0.0,
        },
        animation: ActorAnimationInput::default(),
        appearance: ActorAppearance::default(),
    }
}

fn unit(id: u64, name: &str) -> HudUnit {
    HudUnit {
        entity: entity(id),
        display_name: name.to_string(),
        title_id: None,
        level: 3,
        health: HudMeter {
            current: 120.0,
            maximum: 150.0,
        },
        resource: Some(HudResource {
            kind: HudResourceKind::Rage,
            meter: HudMeter {
                current: 20.0,
                maximum: 100.0,
            },
        }),
        absorb: 0.0,
        dead: false,
        hostile: false,
        elite: false,
        boss: false,
        cast: None,
    }
}

fn action(id: HudActionId) -> HudAction {
    HudAction {
        id,
        cooldown_remaining: 0.0,
        cooldown_total: 0.0,
        count: 1,
        usable: true,
        out_of_range: false,
        queued: false,
        proc_glow: false,
        empowered: false,
    }
}

fn valid_projection() -> ClientPresentationProjection {
    ClientPresentationProjection {
        schema_version: CLIENT_PRESENTATION_SCHEMA_VERSION,
        world: BulkPresentationProjection {
            viewer: entity(1),
            actors: vec![actor(1), actor(2)],
        },
        hud: HudProjection {
            player: unit(1, "Hero"),
            target: Some(HudUnit {
                hostile: true,
                ..unit(2, "Forest Wolf")
            }),
            target_of_target: Some(unit(1, "Hero")),
            combo_points: 0,
            actions: vec![
                action(HudActionId::Attack),
                action(HudActionId::Ability("heroic_strike".to_string())),
                action(HudActionId::Item("minor_healing_potion".to_string())),
            ],
            tracked_quests: vec![HudTrackedQuest {
                quest_id: "q_wolves".to_string(),
                acceptance_order: 1,
                state: HudQuestState::Active,
                objectives: vec![HudQuestObjective {
                    objective_index: 0,
                    current: 2,
                    required: 5,
                }],
            }],
        },
        windows: ClientWindowProjection {
            inventory: InventoryWindowProjection {
                backpack_slots: 16,
                capacity: 16,
                copper: 80,
                bags: vec![None, None, None, None],
                items: Vec::new(),
            },
            quest_log: QuestLogWindowProjection {
                completed_count: 0,
                quests: vec![QuestLogEntryProjection {
                    quest_id: "q_wolves".to_string(),
                    acceptance_order: 1,
                    state: HudQuestState::Active,
                    suggested_players: None,
                    objectives: vec![QuestLogObjectiveProjection {
                        objective_index: 0,
                        current: 2,
                        required: 5,
                    }],
                    xp_reward: 250,
                    copper_reward: 80,
                    reward_item_id: None,
                    turn_in_npc_id: "npc_marshal_arden".to_string(),
                }],
            },
        },
    }
}

#[test]
fn valid_bulk_hud_keeps_gameplay_values_raw_and_host_text_free() {
    let projection = valid_projection();
    projection.validate().expect("valid client projection");

    assert_eq!(projection.hud.player.health.current, 120.0);
    assert_eq!(
        projection.hud.actions[1].id,
        HudActionId::Ability("heroic_strike".into())
    );
    assert_eq!(projection.hud.tracked_quests[0].objectives[0].current, 2);
}

#[test]
fn projection_json_round_trip_validates_before_crossing_into_the_client() {
    let projection = valid_projection();
    let encoded = projection.encode_json().expect("encode valid projection");
    let decoded = ClientPresentationProjection::decode_json(&encoded).expect("decode projection");
    assert_eq!(decoded, projection);

    let mut unsupported = valid_projection();
    unsupported.schema_version += 1;
    let unsupported_json = serde_json::to_vec(&unsupported).expect("encode unsupported fixture");
    assert!(matches!(
        ClientPresentationProjection::decode_json(&unsupported_json),
        Err(woc_runtime::ClientProjectionCodecError::Projection(
            ClientProjectionError::UnsupportedSchemaVersion {
                actual: 3,
                expected: 2
            }
        ))
    ));

    let mut invalid = valid_projection();
    invalid.world.actors[0].transform.translation.x = f32::NAN;
    assert!(invalid.encode_json().is_err());
    assert!(ClientPresentationProjection::decode_json(br#"{"world":null}"#).is_err());
}

#[test]
fn hud_player_must_be_the_viewer_and_every_unit_must_exist_in_the_actor_bulk() {
    let mut projection = valid_projection();
    projection.hud.player.entity = entity(2);
    assert_eq!(
        projection.validate().expect_err("player/viewer mismatch"),
        ClientProjectionError::Hud(HudProjectionError::PlayerIsNotViewer {
            player: entity(2),
            viewer: entity(1),
        })
    );

    projection.hud.player.entity = entity(1);
    projection.hud.target = Some(unit(99, "Unknown"));
    assert_eq!(
        projection.validate().expect_err("target actor missing"),
        ClientProjectionError::Hud(HudProjectionError::UnitActorMissing {
            role: HudUnitRole::Target,
            entity: entity(99),
        })
    );

    projection.hud.target = None;
    projection.hud.target_of_target = Some(unit(1, "Hero"));
    assert_eq!(
        projection
            .validate()
            .expect_err("target-of-target requires target"),
        ClientProjectionError::Hud(HudProjectionError::TargetOfTargetWithoutTarget)
    );
}

#[test]
fn hud_rejects_invalid_numeric_values_and_noncanonical_actions() {
    let mut projection = valid_projection();
    projection.hud.player.cast = Some(HudCast {
        ability_id: "heroic_strike".to_string(),
        remaining: f32::NAN,
        total: 1.5,
        channeling: false,
    });
    assert_eq!(
        projection.validate().expect_err("non-finite cast"),
        ClientProjectionError::Hud(HudProjectionError::InvalidUnitField {
            role: HudUnitRole::Player,
            field: "cast.remaining",
        })
    );

    projection.hud.player.cast = None;
    projection.hud.actions.swap(1, 2);
    assert_eq!(
        projection
            .validate()
            .expect_err("actions must be canonical"),
        ClientProjectionError::Hud(HudProjectionError::ActionsNotStrictlySorted { index: 2 })
    );
}

#[test]
fn quest_and_objective_order_remain_explicit_authoritative_contracts() {
    let mut projection = valid_projection();
    projection.hud.tracked_quests[0].acceptance_order = 2;
    assert_eq!(
        projection
            .validate()
            .expect_err("acceptance order starts at one"),
        ClientProjectionError::Hud(HudProjectionError::QuestAcceptanceOrder {
            index: 0,
            actual: 2,
        })
    );

    projection.hud.tracked_quests[0].acceptance_order = 1;
    projection.hud.tracked_quests[0].objectives[0].objective_index = 1;
    assert_eq!(
        projection
            .validate()
            .expect_err("objective order starts at zero"),
        ClientProjectionError::Hud(HudProjectionError::QuestObjectiveOrder {
            quest_index: 0,
            objective_index: 0,
            actual: 1,
        })
    );
}

#[test]
fn hud_tracker_rejects_duplicate_active_quest_ids_before_window_projection() {
    let mut projection = valid_projection();
    let mut duplicate = projection.hud.tracked_quests[0].clone();
    duplicate.acceptance_order = 2;
    projection.hud.tracked_quests.push(duplicate);

    assert_eq!(
        projection
            .validate()
            .expect_err("tracker quest ids must be unique"),
        ClientProjectionError::Hud(HudProjectionError::DuplicateQuestId {
            index: 1,
            quest_id: "q_wolves".into(),
        })
    );
}

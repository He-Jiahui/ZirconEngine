use crate::generated::{
    command_field_id, event_field_id, fixed_tick_input_field_id, limit, network_envelope_field_id,
    offline_session_bootstrap_field_id, rl_action_batch_field_id, rl_observation_batch_field_id,
    save_state_field_id, world_snapshot_field_id,
};
use crate::weapon_skin_contract::weapon_skin_code_matches_loadout_type;
use crate::{
    command_descriptor, Command, EntityRef, Event, FixedTickInput, FixedTickInputRef, MessageKind,
    MovementFrame, MovementFrameBatch, MovementInputFlags, NetworkEnvelope,
    OfflineSessionBootstrap, OfflineWeaponSkinAccount, ProtocolError, RlActionBatch,
    RlObservationBatch, SaveState, WorldSnapshot, OFFLINE_SESSION_BOOTSTRAP_VERSION,
    OFFLINE_WEAPON_SKIN_COUNT, OFFLINE_WEAPON_SKIN_TYPE_COUNT, SCHEMA_FINGERPRINT_BYTES,
    STANDARD_OFFLINE_WORLD_SEED,
};

const FIXED_TICK_BASE_BYTES: usize = 8 + 4 + 1 + 4 + 4 + 8 + 4 + 4;
const OFFLINE_BOOTSTRAP_BASE_BYTES: usize =
    2 + 4 + 1 + 4 + 2 + OFFLINE_WEAPON_SKIN_COUNT + OFFLINE_WEAPON_SKIN_TYPE_COUNT;
const COMMAND_BASE_BYTES: usize = 2 + 8 + 4 + 4 + 4;
const MOVEMENT_FRAME_BYTES: usize = 8 + 4 + 4 + 7 + 1 + 8;
const WORLD_SNAPSHOT_BASE_BYTES: usize = 8 + 4 + 4 + 4 + 4;
const EVENT_BASE_BYTES: usize = 2 + 4 + 4;
const SAVE_STATE_BASE_BYTES: usize = 4 + 32 + 8 + 8 + 4;
const NETWORK_ENVELOPE_BASE_BYTES: usize = 2 + 2 + 8 + 8 + 4;
const RL_BATCH_BASE_BYTES: usize = 8 + 4 + 4 + 4;

impl FixedTickInput {
    pub fn encode_payload(&self) -> Result<Vec<u8>, ProtocolError> {
        FixedTickInputRef::from(self).encode_payload()
    }

    pub fn decode_payload(bytes: &[u8]) -> Result<Self, ProtocolError> {
        check_bound(
            "FixedTickInput.payload",
            bytes.len(),
            limit::FRAME_PAYLOAD_BYTES,
        )?;
        let mut reader = Reader::new(bytes);
        let tick = reader.read_u64("FixedTickInput.tick")?;
        let command_count = reader.read_length(
            "FixedTickInput.commands",
            fixed_tick_input_field_id::COMMANDS_MAX_LENGTH,
        )?;
        let mut commands = Vec::with_capacity(command_count);
        for _ in 0..command_count {
            commands.push(Command::decode_from(&mut reader)?);
        }
        let wall_time_forbidden = match reader.read_u8("FixedTickInput.wall_time_forbidden")? {
            0 => false,
            1 => true,
            invalid => return Err(ProtocolError::InvalidBoolean(invalid)),
        };
        let committed_state = reader.read_bytes(
            "FixedTickInput.committed_state",
            fixed_tick_input_field_id::COMMITTED_STATE_MAX_LENGTH,
        )?;
        let committed_state_digest = reader.read_u32("FixedTickInput.committed_state_digest")?;
        let generation = reader.read_u64("FixedTickInput.generation")?;
        let movement_count = reader.read_length(
            "FixedTickInput.movement_frames",
            fixed_tick_input_field_id::MOVEMENT_FRAMES_MAX_LENGTH,
        )?;
        let mut movement_frames = Vec::with_capacity(movement_count);
        for _ in 0..movement_count {
            movement_frames.push(decode_movement_frame(&mut reader)?);
        }
        let movement_frames = MovementFrameBatch::from_canonical(movement_frames)
            .map_err(|error| ProtocolError::InvalidMovementInput(error.to_string()))?
            .frames()
            .to_vec();
        let offline_bootstrap = reader.read_bytes(
            "FixedTickInput.offline_bootstrap",
            fixed_tick_input_field_id::OFFLINE_BOOTSTRAP_MAX_LENGTH,
        )?;
        let offline_bootstrap = if offline_bootstrap.is_empty() {
            None
        } else {
            Some(OfflineSessionBootstrap::decode_payload(&offline_bootstrap)?)
        };
        reader.finish()?;
        Ok(Self {
            tick,
            commands,
            wall_time_forbidden,
            committed_state,
            committed_state_digest,
            generation,
            movement_frames,
            offline_bootstrap,
        })
    }
}

impl FixedTickInputRef<'_> {
    pub fn encode_payload(&self) -> Result<Vec<u8>, ProtocolError> {
        check_bound(
            "FixedTickInput.commands",
            self.commands.len(),
            fixed_tick_input_field_id::COMMANDS_MAX_LENGTH,
        )?;
        check_bound(
            "FixedTickInput.committed_state",
            self.committed_state.len(),
            fixed_tick_input_field_id::COMMITTED_STATE_MAX_LENGTH,
        )?;
        let movement_frames = MovementFrameBatch::new(self.movement_frames.to_vec())
            .map_err(|error| ProtocolError::InvalidMovementInput(error.to_string()))?;
        check_bound(
            "FixedTickInput.movement_frames",
            movement_frames.frames().len(),
            fixed_tick_input_field_id::MOVEMENT_FRAMES_MAX_LENGTH,
        )?;
        let offline_bootstrap = self
            .offline_bootstrap
            .map(OfflineSessionBootstrap::encode_payload)
            .transpose()?;
        let offline_bootstrap_bytes = offline_bootstrap.as_deref().unwrap_or_default();
        check_bound(
            "FixedTickInput.offline_bootstrap",
            offline_bootstrap_bytes.len(),
            fixed_tick_input_field_id::OFFLINE_BOOTSTRAP_MAX_LENGTH,
        )?;
        let wire_length = checked_wire_length(
            "FixedTickInput.payload",
            FIXED_TICK_BASE_BYTES,
            std::iter::once(self.committed_state.len())
                .chain(
                    self.commands
                        .iter()
                        .map(|command| COMMAND_BASE_BYTES.saturating_add(command.payload.len())),
                )
                .chain(std::iter::once(
                    movement_frames
                        .frames()
                        .len()
                        .saturating_mul(MOVEMENT_FRAME_BYTES),
                ))
                .chain(std::iter::once(offline_bootstrap_bytes.len())),
            limit::FRAME_PAYLOAD_BYTES,
        )?;
        let mut output = Vec::with_capacity(wire_length);
        push_u64(&mut output, self.tick);
        push_length(&mut output, "FixedTickInput.commands", self.commands.len())?;
        for command in &self.commands {
            command.encode_into(&mut output)?;
        }
        output.push(u8::from(self.wall_time_forbidden));
        push_bytes(
            &mut output,
            "FixedTickInput.committed_state",
            self.committed_state,
        )?;
        push_u32(&mut output, self.committed_state_digest);
        push_u64(&mut output, self.generation);
        push_length(
            &mut output,
            "FixedTickInput.movement_frames",
            movement_frames.frames().len(),
        )?;
        for frame in movement_frames.frames() {
            encode_movement_frame(&mut output, *frame);
        }
        push_bytes(
            &mut output,
            "FixedTickInput.offline_bootstrap",
            offline_bootstrap_bytes,
        )?;
        Ok(output)
    }
}

impl OfflineSessionBootstrap {
    pub fn encode_payload(&self) -> Result<Vec<u8>, ProtocolError> {
        self.validate()?;
        let player_name = self.player_name.as_bytes();
        let mut output = Vec::with_capacity(OFFLINE_BOOTSTRAP_BASE_BYTES + player_name.len());
        push_u16(&mut output, self.launch_version);
        push_u32(&mut output, self.world_seed);
        output.push(self.player_class);
        push_bytes(
            &mut output,
            "OfflineSessionBootstrap.player_name",
            player_name,
        )?;
        push_u16(&mut output, self.skin_variant);
        for owned in &self.weapon_skin_account.owned {
            output.push(u8::from(*owned));
        }
        output.extend_from_slice(&self.weapon_skin_account.loadout_codes);
        Ok(output)
    }

    pub fn decode_payload(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let mut reader = Reader::new(bytes);
        let launch_version = reader.read_u16("OfflineSessionBootstrap.launch_version")?;
        let world_seed = reader.read_u32("OfflineSessionBootstrap.world_seed")?;
        let player_class = reader.read_u8("OfflineSessionBootstrap.player_class")?;
        let player_name = reader.read_bytes(
            "OfflineSessionBootstrap.player_name",
            offline_session_bootstrap_field_id::PLAYER_NAME_MAX_LENGTH,
        )?;
        let player_name =
            String::from_utf8(player_name).map_err(|_| ProtocolError::InvalidUtf8 {
                context: "OfflineSessionBootstrap.player_name",
            })?;
        let skin_variant = reader.read_u16("OfflineSessionBootstrap.skin_variant")?;
        let mut weapon_skin_owned = [false; OFFLINE_WEAPON_SKIN_COUNT];
        for owned in &mut weapon_skin_owned {
            *owned = reader.read_bool("OfflineSessionBootstrap.weapon_skin_owned")?;
        }
        let mut weapon_skin_loadout_codes = [0; OFFLINE_WEAPON_SKIN_TYPE_COUNT];
        for code in &mut weapon_skin_loadout_codes {
            *code = reader.read_u8("OfflineSessionBootstrap.weapon_skin_loadout_codes")?;
        }
        reader.finish()?;
        let bootstrap = Self {
            launch_version,
            world_seed,
            player_class,
            player_name,
            skin_variant,
            weapon_skin_account: OfflineWeaponSkinAccount {
                owned: weapon_skin_owned,
                loadout_codes: weapon_skin_loadout_codes,
            },
        };
        bootstrap.validate()?;
        Ok(bootstrap)
    }

    pub fn validate(&self) -> Result<(), ProtocolError> {
        if self.launch_version != OFFLINE_SESSION_BOOTSTRAP_VERSION {
            return Err(ProtocolError::InvalidOfflineBootstrap(format!(
                "launch version {} is unsupported",
                self.launch_version
            )));
        }
        if self.world_seed != STANDARD_OFFLINE_WORLD_SEED {
            return Err(ProtocolError::InvalidOfflineBootstrap(format!(
                "world seed {} is not the standard offline seed",
                self.world_seed
            )));
        }
        if self.player_class > 8 {
            return Err(ProtocolError::InvalidOfflineBootstrap(format!(
                "player class {} is outside the source catalog",
                self.player_class
            )));
        }
        let player_name = self.player_name.as_bytes();
        check_bound(
            "OfflineSessionBootstrap.player_name",
            player_name.len(),
            offline_session_bootstrap_field_id::PLAYER_NAME_MAX_LENGTH,
        )?;
        let Some((first, rest)) = player_name.split_first() else {
            return Err(ProtocolError::InvalidOfflineBootstrap(
                "player name is empty".to_string(),
            ));
        };
        if player_name.len() < 2
            || !first.is_ascii_alphabetic()
            || !rest.iter().all(valid_name_byte)
        {
            return Err(ProtocolError::InvalidOfflineBootstrap(
                "player name does not match the source offline rule".to_string(),
            ));
        }
        const CLASS_SKIN_VARIANT_LIMIT: u16 = 8;
        if self.skin_variant >= CLASS_SKIN_VARIANT_LIMIT {
            return Err(ProtocolError::InvalidOfflineBootstrap(format!(
                "skin variant {} is outside the {}-skin source class catalog",
                self.skin_variant, CLASS_SKIN_VARIANT_LIMIT
            )));
        }
        self.weapon_skin_account.validate()?;
        Ok(())
    }
}

impl OfflineWeaponSkinAccount {
    pub fn validate(&self) -> Result<(), ProtocolError> {
        for (loadout_type_index, code) in self.loadout_codes.iter().enumerate() {
            let code = *code;
            if usize::from(code) > OFFLINE_WEAPON_SKIN_COUNT {
                return Err(ProtocolError::InvalidOfflineBootstrap(format!(
                    "weapon skin loadout code {code} is outside the source catalog"
                )));
            }
            if code != 0 && !self.owned[usize::from(code) - 1] {
                return Err(ProtocolError::InvalidOfflineBootstrap(format!(
                    "weapon skin loadout code {code} is not owned"
                )));
            }
            if code != 0 && !weapon_skin_code_matches_loadout_type(code, loadout_type_index) {
                return Err(ProtocolError::InvalidOfflineBootstrap(format!(
                    "weapon skin loadout code {code} does not match loadout type {}",
                    loadout_type_index + 1
                )));
            }
        }
        Ok(())
    }
}

fn valid_name_byte(value: &u8) -> bool {
    value.is_ascii_alphabetic() || matches!(*value, b'\'' | b' ' | b'-')
}

fn encode_movement_frame(output: &mut Vec<u8>, frame: MovementFrame) {
    push_u64(output, frame.actor.id);
    push_u32(output, frame.actor.generation);
    push_u32(output, frame.sequence);
    output.push(u8::from(frame.flags.forward));
    output.push(u8::from(frame.flags.back));
    output.push(u8::from(frame.flags.turn_left));
    output.push(u8::from(frame.flags.turn_right));
    output.push(u8::from(frame.flags.strafe_left));
    output.push(u8::from(frame.flags.strafe_right));
    output.push(u8::from(frame.flags.jump));
    output.push(u8::from(frame.facing.is_some()));
    push_f64(output, frame.facing.unwrap_or(0.0));
}

fn decode_movement_frame(reader: &mut Reader<'_>) -> Result<MovementFrame, ProtocolError> {
    let actor = EntityRef {
        id: reader.read_u64("MovementFrame.actor.id")?,
        generation: reader.read_u32("MovementFrame.actor.generation")?,
    };
    let sequence = reader.read_u32("MovementFrame.sequence")?;
    let flags = MovementInputFlags {
        forward: reader.read_bool("MovementFrame.forward")?,
        back: reader.read_bool("MovementFrame.back")?,
        turn_left: reader.read_bool("MovementFrame.turn_left")?,
        turn_right: reader.read_bool("MovementFrame.turn_right")?,
        strafe_left: reader.read_bool("MovementFrame.strafe_left")?,
        strafe_right: reader.read_bool("MovementFrame.strafe_right")?,
        jump: reader.read_bool("MovementFrame.jump")?,
    };
    let has_facing = reader.read_bool("MovementFrame.has_facing")?;
    let encoded_facing = reader.read_f64("MovementFrame.facing")?;
    let facing = if has_facing {
        Some(encoded_facing)
    } else if encoded_facing == 0.0 {
        None
    } else {
        return Err(ProtocolError::InvalidMovementInput(
            "absent facing must use the canonical +0.0 payload".to_string(),
        ));
    };
    Ok(MovementFrame {
        actor,
        sequence,
        flags,
        facing,
    })
}

impl WorldSnapshot {
    pub fn encode_payload(&self) -> Result<Vec<u8>, ProtocolError> {
        check_bound(
            "WorldSnapshot.state",
            self.state.len(),
            world_snapshot_field_id::STATE_MAX_LENGTH,
        )?;
        check_bound(
            "WorldSnapshot.events",
            self.events.len(),
            world_snapshot_field_id::EVENTS_MAX_LENGTH,
        )?;
        let wire_length = checked_wire_length(
            "WorldSnapshot.payload",
            WORLD_SNAPSHOT_BASE_BYTES,
            std::iter::once(self.state.len()).chain(
                self.events
                    .iter()
                    .map(|event| EVENT_BASE_BYTES.saturating_add(event.payload.len())),
            ),
            limit::FRAME_PAYLOAD_BYTES,
        )?;
        let mut output = Vec::with_capacity(wire_length);
        push_u64(&mut output, self.tick);
        push_u32(&mut output, self.state_digest);
        push_u32(&mut output, self.event_digest);
        push_bytes(&mut output, "WorldSnapshot.state", &self.state)?;
        push_length(&mut output, "WorldSnapshot.events", self.events.len())?;
        for event in &self.events {
            event.encode_into(&mut output)?;
        }
        Ok(output)
    }

    pub fn decode_payload(bytes: &[u8]) -> Result<Self, ProtocolError> {
        check_bound(
            "WorldSnapshot.payload",
            bytes.len(),
            limit::FRAME_PAYLOAD_BYTES,
        )?;
        let mut reader = Reader::new(bytes);
        let tick = reader.read_u64("WorldSnapshot.tick")?;
        let state_digest = reader.read_u32("WorldSnapshot.state_digest")?;
        let event_digest = reader.read_u32("WorldSnapshot.event_digest")?;
        let state = reader.read_bytes(
            "WorldSnapshot.state",
            world_snapshot_field_id::STATE_MAX_LENGTH,
        )?;
        let event_count = reader.read_length(
            "WorldSnapshot.events",
            world_snapshot_field_id::EVENTS_MAX_LENGTH,
        )?;
        let mut events = Vec::with_capacity(event_count);
        for _ in 0..event_count {
            events.push(Event::decode_from(&mut reader)?);
        }
        reader.finish()?;
        Ok(Self {
            tick,
            state_digest,
            event_digest,
            state,
            events,
        })
    }
}

impl SaveState {
    pub fn encode_payload(&self) -> Result<Vec<u8>, ProtocolError> {
        require_schema_fingerprint(self.schema_fingerprint)?;
        check_bound(
            "SaveState.state",
            self.state.len(),
            save_state_field_id::STATE_MAX_LENGTH,
        )?;
        let wire_length = checked_wire_length(
            "SaveState.payload",
            SAVE_STATE_BASE_BYTES,
            [self.state.len()],
            limit::FRAME_PAYLOAD_BYTES,
        )?;
        let mut output = Vec::with_capacity(wire_length);
        push_bytes(
            &mut output,
            "SaveState.schema_fingerprint",
            &self.schema_fingerprint,
        )?;
        push_u64(&mut output, self.generation);
        push_u64(&mut output, self.tick);
        push_bytes(&mut output, "SaveState.state", &self.state)?;
        Ok(output)
    }

    pub fn decode_payload(bytes: &[u8]) -> Result<Self, ProtocolError> {
        check_bound("SaveState.payload", bytes.len(), limit::FRAME_PAYLOAD_BYTES)?;
        let mut reader = Reader::new(bytes);
        let schema_bytes = reader.read_bytes(
            "SaveState.schema_fingerprint",
            save_state_field_id::SCHEMA_FINGERPRINT_MAX_LENGTH,
        )?;
        let schema_fingerprint: [u8; 32] =
            schema_bytes
                .as_slice()
                .try_into()
                .map_err(|_| ProtocolError::SchemaMismatch {
                    actual: copy_schema_prefix(&schema_bytes),
                })?;
        require_schema_fingerprint(schema_fingerprint)?;
        let generation = reader.read_u64("SaveState.generation")?;
        let tick = reader.read_u64("SaveState.tick")?;
        let state = reader.read_bytes("SaveState.state", save_state_field_id::STATE_MAX_LENGTH)?;
        reader.finish()?;
        Ok(Self {
            schema_fingerprint,
            generation,
            tick,
            state,
        })
    }
}

impl NetworkEnvelope {
    pub fn encode_payload(&self) -> Result<Vec<u8>, ProtocolError> {
        require_protocol_version(self.protocol_version)?;
        check_bound(
            "NetworkEnvelope.payload",
            self.payload.len(),
            network_envelope_field_id::PAYLOAD_MAX_LENGTH,
        )?;
        let wire_length = checked_wire_length(
            "NetworkEnvelope.wire_payload",
            NETWORK_ENVELOPE_BASE_BYTES,
            [self.payload.len()],
            limit::FRAME_PAYLOAD_BYTES,
        )?;
        let mut output = Vec::with_capacity(wire_length);
        push_u16(&mut output, self.protocol_version);
        push_u16(&mut output, self.kind as u16);
        push_u64(&mut output, self.sequence);
        push_u64(&mut output, self.acknowledgement);
        push_bytes(&mut output, "NetworkEnvelope.payload", &self.payload)?;
        Ok(output)
    }

    pub fn decode_payload(bytes: &[u8]) -> Result<Self, ProtocolError> {
        check_bound(
            "NetworkEnvelope.wire_payload",
            bytes.len(),
            limit::FRAME_PAYLOAD_BYTES,
        )?;
        let mut reader = Reader::new(bytes);
        let protocol_version = reader.read_u16("NetworkEnvelope.protocol_version")?;
        require_protocol_version(protocol_version)?;
        let kind = MessageKind::try_from(reader.read_u16("NetworkEnvelope.kind")?)?;
        let sequence = reader.read_u64("NetworkEnvelope.sequence")?;
        let acknowledgement = reader.read_u64("NetworkEnvelope.acknowledgement")?;
        let payload = reader.read_bytes(
            "NetworkEnvelope.payload",
            network_envelope_field_id::PAYLOAD_MAX_LENGTH,
        )?;
        reader.finish()?;
        Ok(Self {
            protocol_version,
            kind,
            sequence,
            acknowledgement,
            payload,
        })
    }
}

impl RlObservationBatch {
    pub fn encode_payload(&self) -> Result<Vec<u8>, ProtocolError> {
        validate_rl_batch(
            "RlObservationBatch",
            &self.environment_ids,
            &self.offsets,
            self.observations.len(),
            rl_observation_batch_field_id::ENVIRONMENT_IDS_MAX_LENGTH,
            rl_observation_batch_field_id::OFFSETS_MAX_LENGTH,
            rl_observation_batch_field_id::OBSERVATIONS_MAX_LENGTH,
        )?;
        encode_rl_batch(
            "RlObservationBatch",
            self.tick,
            &self.environment_ids,
            &self.offsets,
            &self.observations,
        )
    }

    pub fn decode_payload(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let (tick, environment_ids, offsets, observations) = decode_rl_batch(
            "RlObservationBatch",
            bytes,
            rl_observation_batch_field_id::ENVIRONMENT_IDS_MAX_LENGTH,
            rl_observation_batch_field_id::OFFSETS_MAX_LENGTH,
            rl_observation_batch_field_id::OBSERVATIONS_MAX_LENGTH,
        )?;
        Ok(Self {
            tick,
            environment_ids,
            offsets,
            observations,
        })
    }
}

impl RlActionBatch {
    pub fn encode_payload(&self) -> Result<Vec<u8>, ProtocolError> {
        validate_rl_batch(
            "RlActionBatch",
            &self.environment_ids,
            &self.offsets,
            self.actions.len(),
            rl_action_batch_field_id::ENVIRONMENT_IDS_MAX_LENGTH,
            rl_action_batch_field_id::OFFSETS_MAX_LENGTH,
            rl_action_batch_field_id::ACTIONS_MAX_LENGTH,
        )?;
        encode_rl_batch(
            "RlActionBatch",
            self.tick,
            &self.environment_ids,
            &self.offsets,
            &self.actions,
        )
    }

    pub fn decode_payload(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let (tick, environment_ids, offsets, actions) = decode_rl_batch(
            "RlActionBatch",
            bytes,
            rl_action_batch_field_id::ENVIRONMENT_IDS_MAX_LENGTH,
            rl_action_batch_field_id::OFFSETS_MAX_LENGTH,
            rl_action_batch_field_id::ACTIONS_MAX_LENGTH,
        )?;
        Ok(Self {
            tick,
            environment_ids,
            offsets,
            actions,
        })
    }
}

impl Command {
    pub fn encode_payload(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut output = Vec::with_capacity(COMMAND_BASE_BYTES.saturating_add(self.payload.len()));
        self.encode_into(&mut output)?;
        Ok(output)
    }

    pub fn decode_payload(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let mut reader = Reader::new(bytes);
        let command = Self::decode_from(&mut reader)?;
        reader.finish()?;
        Ok(command)
    }

    fn encode_into(&self, output: &mut Vec<u8>) -> Result<(), ProtocolError> {
        require_known_command(self.command_id)?;
        check_bound(
            "Command.payload",
            self.payload.len(),
            command_field_id::PAYLOAD_MAX_LENGTH,
        )?;
        push_u16(output, self.command_id);
        push_u64(output, self.actor.id);
        push_u32(output, self.actor.generation);
        push_u32(output, self.sequence);
        push_bytes(output, "Command.payload", &self.payload)
    }

    fn decode_from(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let command_id = reader.read_u16("Command.command_id")?;
        require_known_command(command_id)?;
        Ok(Self {
            command_id,
            actor: EntityRef {
                id: reader.read_u64("Command.actor.id")?,
                generation: reader.read_u32("Command.actor.generation")?,
            },
            sequence: reader.read_u32("Command.sequence")?,
            payload: reader.read_bytes("Command.payload", command_field_id::PAYLOAD_MAX_LENGTH)?,
        })
    }
}

fn require_known_command(command_id: u16) -> Result<(), ProtocolError> {
    command_descriptor(command_id)
        .map(|_| ())
        .ok_or(ProtocolError::UnknownCommandId(command_id))
}

impl Event {
    pub fn encode_payload(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut output = Vec::with_capacity(EVENT_BASE_BYTES.saturating_add(self.payload.len()));
        self.encode_into(&mut output)?;
        Ok(output)
    }

    pub fn decode_payload(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let mut reader = Reader::new(bytes);
        let event = Self::decode_from(&mut reader)?;
        reader.finish()?;
        Ok(event)
    }

    fn encode_into(&self, output: &mut Vec<u8>) -> Result<(), ProtocolError> {
        check_bound(
            "Event.payload",
            self.payload.len(),
            event_field_id::PAYLOAD_MAX_LENGTH,
        )?;
        push_u16(output, self.event_id);
        push_u32(output, self.sequence);
        push_bytes(output, "Event.payload", &self.payload)
    }

    fn decode_from(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        Ok(Self {
            event_id: reader.read_u16("Event.event_id")?,
            sequence: reader.read_u32("Event.sequence")?,
            payload: reader.read_bytes("Event.payload", event_field_id::PAYLOAD_MAX_LENGTH)?,
        })
    }
}

fn require_schema_fingerprint(actual: [u8; 32]) -> Result<(), ProtocolError> {
    if actual == SCHEMA_FINGERPRINT_BYTES {
        Ok(())
    } else {
        Err(ProtocolError::SchemaMismatch { actual })
    }
}

fn copy_schema_prefix(bytes: &[u8]) -> [u8; 32] {
    let mut actual = [0_u8; 32];
    let length = bytes.len().min(actual.len());
    actual[..length].copy_from_slice(&bytes[..length]);
    actual
}

fn require_protocol_version(actual: u16) -> Result<(), ProtocolError> {
    if actual == crate::PROTOCOL_VERSION {
        Ok(())
    } else {
        Err(ProtocolError::UnsupportedVersion {
            actual,
            expected: crate::PROTOCOL_VERSION,
        })
    }
}

fn validate_rl_batch(
    context: &'static str,
    environment_ids: &[u32],
    offsets: &[u32],
    payload_length: usize,
    environment_limit: u64,
    offset_limit: u64,
    payload_limit: u64,
) -> Result<(), ProtocolError> {
    check_bound(context, environment_ids.len(), environment_limit)?;
    check_bound(context, offsets.len(), offset_limit)?;
    check_bound(context, payload_length, payload_limit)?;
    let expected_offsets = environment_ids
        .len()
        .checked_add(1)
        .ok_or(ProtocolError::InvalidOffsets { context })?;
    let partitions = offsets.len() == expected_offsets
        && offsets.first() == Some(&0)
        && offsets.windows(2).all(|pair| pair[0] <= pair[1])
        && offsets.last().copied().map(u64::from) == Some(payload_length as u64);
    if partitions {
        Ok(())
    } else {
        Err(ProtocolError::InvalidOffsets { context })
    }
}

fn encode_rl_batch(
    context: &'static str,
    tick: u64,
    environment_ids: &[u32],
    offsets: &[u32],
    payload: &[u8],
) -> Result<Vec<u8>, ProtocolError> {
    let wire_length = checked_wire_length(
        context,
        RL_BATCH_BASE_BYTES,
        [
            environment_ids.len().saturating_mul(4),
            offsets.len().saturating_mul(4),
            payload.len(),
        ],
        limit::FRAME_PAYLOAD_BYTES,
    )?;
    let mut output = Vec::with_capacity(wire_length);
    push_u64(&mut output, tick);
    push_u32_vector(&mut output, context, environment_ids)?;
    push_u32_vector(&mut output, context, offsets)?;
    push_bytes(&mut output, context, payload)?;
    Ok(output)
}

fn decode_rl_batch(
    context: &'static str,
    bytes: &[u8],
    environment_limit: u64,
    offset_limit: u64,
    payload_limit: u64,
) -> Result<(u64, Vec<u32>, Vec<u32>, Vec<u8>), ProtocolError> {
    check_bound(context, bytes.len(), limit::FRAME_PAYLOAD_BYTES)?;
    let mut reader = Reader::new(bytes);
    let tick = reader.read_u64(context)?;
    let environment_ids = reader.read_u32_vector(context, environment_limit)?;
    let offsets = reader.read_u32_vector(context, offset_limit)?;
    let payload = reader.read_bytes(context, payload_limit)?;
    reader.finish()?;
    validate_rl_batch(
        context,
        &environment_ids,
        &offsets,
        payload.len(),
        environment_limit,
        offset_limit,
        payload_limit,
    )?;
    Ok((tick, environment_ids, offsets, payload))
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn read_u8(&mut self, context: &'static str) -> Result<u8, ProtocolError> {
        Ok(self.take(context, 1)?[0])
    }

    fn read_bool(&mut self, context: &'static str) -> Result<bool, ProtocolError> {
        match self.read_u8(context)? {
            0 => Ok(false),
            1 => Ok(true),
            invalid => Err(ProtocolError::InvalidBoolean(invalid)),
        }
    }

    fn read_u16(&mut self, context: &'static str) -> Result<u16, ProtocolError> {
        Ok(u16::from_le_bytes(
            self.take(context, 2)?.try_into().expect("fixed u16 slice"),
        ))
    }

    fn read_u32(&mut self, context: &'static str) -> Result<u32, ProtocolError> {
        Ok(u32::from_le_bytes(
            self.take(context, 4)?.try_into().expect("fixed u32 slice"),
        ))
    }

    fn read_u64(&mut self, context: &'static str) -> Result<u64, ProtocolError> {
        Ok(u64::from_le_bytes(
            self.take(context, 8)?.try_into().expect("fixed u64 slice"),
        ))
    }

    fn read_f64(&mut self, context: &'static str) -> Result<f64, ProtocolError> {
        Ok(f64::from_le_bytes(
            self.take(context, 8)?.try_into().expect("fixed f64 slice"),
        ))
    }

    fn read_length(&mut self, context: &'static str, maximum: u64) -> Result<usize, ProtocolError> {
        let length = self.read_u32(context)? as usize;
        check_bound(context, length, maximum)?;
        Ok(length)
    }

    fn read_bytes(
        &mut self,
        context: &'static str,
        maximum: u64,
    ) -> Result<Vec<u8>, ProtocolError> {
        let length = self.read_length(context, maximum)?;
        Ok(self.take(context, length)?.to_vec())
    }

    fn read_u32_vector(
        &mut self,
        context: &'static str,
        maximum: u64,
    ) -> Result<Vec<u32>, ProtocolError> {
        let length = self.read_length(context, maximum)?;
        let mut values = Vec::with_capacity(length);
        for _ in 0..length {
            values.push(self.read_u32(context)?);
        }
        Ok(values)
    }

    fn take(&mut self, context: &'static str, length: usize) -> Result<&'a [u8], ProtocolError> {
        let remaining = self.bytes.len().saturating_sub(self.offset);
        if remaining < length {
            return Err(ProtocolError::TruncatedPayload {
                context,
                needed: length,
                remaining,
            });
        }
        let start = self.offset;
        self.offset += length;
        Ok(&self.bytes[start..self.offset])
    }

    fn finish(self) -> Result<(), ProtocolError> {
        let remaining = self.bytes.len() - self.offset;
        if remaining != 0 {
            return Err(ProtocolError::TrailingPayload { remaining });
        }
        Ok(())
    }
}

fn check_bound(context: &'static str, actual: usize, maximum: u64) -> Result<(), ProtocolError> {
    let maximum = usize::try_from(maximum).unwrap_or(usize::MAX);
    if actual > maximum {
        return Err(ProtocolError::CollectionTooLarge {
            context,
            actual,
            maximum,
        });
    }
    Ok(())
}

fn checked_wire_length(
    context: &'static str,
    base: usize,
    components: impl IntoIterator<Item = usize>,
    maximum: u64,
) -> Result<usize, ProtocolError> {
    let maximum = usize::try_from(maximum).unwrap_or(usize::MAX);
    let mut actual = base;
    for length in components {
        actual = actual
            .checked_add(length)
            .ok_or(ProtocolError::CollectionTooLarge {
                context,
                actual: usize::MAX,
                maximum,
            })?;
    }
    check_bound(context, actual, maximum as u64)?;
    Ok(actual)
}

fn push_length(
    output: &mut Vec<u8>,
    context: &'static str,
    length: usize,
) -> Result<(), ProtocolError> {
    let length = u32::try_from(length).map_err(|_| ProtocolError::CollectionTooLarge {
        context,
        actual: length,
        maximum: u32::MAX as usize,
    })?;
    push_u32(output, length);
    Ok(())
}

fn push_bytes(
    output: &mut Vec<u8>,
    context: &'static str,
    bytes: &[u8],
) -> Result<(), ProtocolError> {
    push_length(output, context, bytes.len())?;
    output.extend_from_slice(bytes);
    Ok(())
}

fn push_u32_vector(
    output: &mut Vec<u8>,
    context: &'static str,
    values: &[u32],
) -> Result<(), ProtocolError> {
    push_length(output, context, values.len())?;
    for value in values {
        push_u32(output, *value);
    }
    Ok(())
}

fn push_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn push_f64(output: &mut Vec<u8>, value: f64) {
    output.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_wire_length_rejects_limit_excess_and_saturating_overflow() {
        assert_eq!(checked_wire_length("fixture", 4, [3, 2], 9), Ok(9));
        assert_eq!(
            checked_wire_length("fixture", 4, [3, 3], 9),
            Err(ProtocolError::CollectionTooLarge {
                context: "fixture",
                actual: 10,
                maximum: 9,
            })
        );
        assert!(matches!(
            checked_wire_length("fixture", usize::MAX, [1], u64::MAX),
            Err(ProtocolError::CollectionTooLarge { actual, .. }) if actual == usize::MAX
        ));
    }
}

use woc_protocol::{Command, EntityRef, FixedTickInput, FixedTickInputRef, ProtocolError};

fn command(id: u16, payload: Vec<u8>) -> Command {
    Command {
        command_id: id,
        actor: EntityRef {
            id: 7,
            generation: 1,
        },
        sequence: 4,
        payload,
    }
}

fn input(command: Command) -> FixedTickInput {
    FixedTickInput {
        tick: 1,
        commands: vec![command],
        wall_time_forbidden: true,
        committed_state: vec![],
        committed_state_digest: 0,
        generation: 0,
        movement_frames: vec![],
        offline_bootstrap: None,
    }
}

// Construct untrusted bytes independently of the production encoders.
fn command_wire(command: &Command) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(command.command_id.to_le_bytes());
    bytes.extend(command.actor.id.to_le_bytes());
    bytes.extend(command.actor.generation.to_le_bytes());
    bytes.extend(command.sequence.to_le_bytes());
    bytes.extend(u32::try_from(command.payload.len()).unwrap().to_le_bytes());
    bytes.extend(&command.payload);
    bytes
}

fn input_wire(command: &Command) -> Vec<u8> {
    let mut bytes = 1_u64.to_le_bytes().to_vec();
    bytes.extend(1_u32.to_le_bytes());
    bytes.extend(command_wire(command));
    bytes.push(1);
    bytes.extend(0_u32.to_le_bytes()); // Committed state length.
    bytes.extend(0_u32.to_le_bytes()); // Committed state digest.
    bytes.extend(0_u64.to_le_bytes()); // Generation.
    bytes.extend(0_u32.to_le_bytes()); // Movement count.
    bytes.extend(0_u32.to_le_bytes()); // Bootstrap length.
    bytes
}

fn assert_rejected_everywhere(invalid: Command, expected: ProtocolError) {
    assert_eq!(invalid.validate(), Err(expected.clone()));
    assert_eq!(invalid.encode_payload(), Err(expected.clone()));
    assert_eq!(
        Command::decode_payload(&command_wire(&invalid)),
        Err(expected.clone())
    );
    assert_eq!(
        FixedTickInput::decode_payload(&input_wire(&invalid)),
        Err(expected.clone())
    );
    let nested = input(invalid);
    assert_eq!(nested.encode_payload(), Err(expected.clone()));
    assert_eq!(
        FixedTickInputRef::from(&nested).encode_payload(),
        Err(expected)
    );
}

#[test]
fn malformed_known_payloads_are_rejected_by_every_envelope_route() {
    for (invalid, expected) in [
        (
            command(5, vec![1]),
            ProtocolError::InvalidCommandPayloadLength {
                command_id: 5,
                expected: 0,
                actual: 1,
            },
        ),
        (command(164, vec![2]), ProtocolError::InvalidBoolean(2)),
        (
            command(112, vec![1, 0, 0, 0, 0xff]),
            ProtocolError::InvalidUtf8 {
                context: "command payload identifier",
            },
        ),
        (
            command(112, vec![1, 0, 0, 0, b'a', 0]),
            ProtocolError::TrailingPayload { remaining: 1 },
        ),
    ] {
        assert_rejected_everywhere(invalid, expected);
    }
}

#[test]
fn unknown_and_unsupported_commands_fail_closed() {
    assert_rejected_everywhere(
        command(u16::MAX, vec![]),
        ProtocolError::UnknownCommandId(u16::MAX),
    );
    for id in [76, 106, 107, 108, 109, 110, 111, 113] {
        assert_rejected_everywhere(
            command(id, vec![]),
            ProtocolError::UnsupportedCommandPayload(id),
        );
    }
}

#[test]
fn valid_command_wire_layout_is_unchanged() {
    for value in [
        command(5, vec![]),
        command(164, vec![1]),
        command(43, 23_u64.to_le_bytes().to_vec()),
    ] {
        let bytes = command_wire(&value);
        assert_eq!(value.encode_payload().unwrap(), bytes);
        assert_eq!(Command::decode_payload(&bytes).unwrap(), value);
        let bytes = input_wire(&value);
        let nested = input(value);
        assert_eq!(nested.encode_payload().unwrap(), bytes);
        assert_eq!(
            FixedTickInputRef::from(&nested).encode_payload().unwrap(),
            bytes
        );
        assert_eq!(FixedTickInput::decode_payload(&bytes).unwrap(), nested);
    }
}

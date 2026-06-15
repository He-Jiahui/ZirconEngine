pub const RELIABLE_UDP_FLAG_FRAGMENT: u8 = 0b0000_0001;
pub const RELIABLE_UDP_FLAG_LAST_FRAGMENT: u8 = 0b0000_0010;

const RELIABLE_UDP_BASE_HEADER_LEN: usize = 10;
const RELIABLE_UDP_FRAGMENT_HEADER_LEN: usize = 4;

/// Fixed Reliable UDP wire header described by the net plugin plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReliableUdpWireHeader {
    pub sequence: u16,
    pub ack: u16,
    pub ack_bits: u32,
    pub channel: u8,
    pub flags: u8,
    pub fragment: Option<ReliableUdpFragmentHeader>,
}

impl ReliableUdpWireHeader {
    pub fn new(sequence: u16, ack: u16, ack_bits: u32, channel: u8) -> Self {
        Self {
            sequence,
            ack,
            ack_bits,
            channel,
            flags: 0,
            fragment: None,
        }
    }

    pub fn with_flags(mut self, flags: u8) -> Self {
        self.flags = flags;
        self
    }

    pub fn with_fragment(mut self, fragment: ReliableUdpFragmentHeader) -> Self {
        self.flags |= RELIABLE_UDP_FLAG_FRAGMENT;
        self.fragment = Some(fragment);
        self
    }

    pub fn is_fragmented(&self) -> bool {
        self.flags & RELIABLE_UDP_FLAG_FRAGMENT != 0
    }

    pub fn acked_sequences(&self) -> Vec<u16> {
        let mut sequences = vec![self.ack];
        for bit in 0..32 {
            if self.ack_bits & (1_u32 << bit) != 0 {
                sequences.push(self.ack.wrapping_sub(bit + 1));
            }
        }
        sequences
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReliableUdpFragmentHeader {
    pub fragment_id: u16,
    pub fragment_index: u8,
    pub fragment_count: u8,
}

impl ReliableUdpFragmentHeader {
    pub fn new(fragment_id: u16, fragment_index: u8, fragment_count: u8) -> Self {
        Self {
            fragment_id,
            fragment_index,
            fragment_count,
        }
    }
}

/// Encoded Reliable UDP datagram with a fixed header and optional fragment header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReliableUdpWirePacket {
    pub header: ReliableUdpWireHeader,
    pub payload: Vec<u8>,
}

impl ReliableUdpWirePacket {
    pub fn new(header: ReliableUdpWireHeader, payload: impl Into<Vec<u8>>) -> Self {
        Self {
            header,
            payload: payload.into(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(
            RELIABLE_UDP_BASE_HEADER_LEN
                + self
                    .header
                    .fragment
                    .map_or(0, |_| RELIABLE_UDP_FRAGMENT_HEADER_LEN)
                + self.payload.len(),
        );
        bytes.extend_from_slice(&self.header.sequence.to_le_bytes());
        bytes.extend_from_slice(&self.header.ack.to_le_bytes());
        bytes.extend_from_slice(&self.header.ack_bits.to_le_bytes());
        bytes.push(self.header.channel);
        bytes.push(self.header.flags);
        if let Some(fragment) = self.header.fragment {
            bytes.extend_from_slice(&fragment.fragment_id.to_le_bytes());
            bytes.push(fragment.fragment_index);
            bytes.push(fragment.fragment_count);
        }
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ReliableUdpWirePacketError> {
        if bytes.len() < RELIABLE_UDP_BASE_HEADER_LEN {
            return Err(ReliableUdpWirePacketError::HeaderTooShort);
        }
        let sequence = u16::from_le_bytes([bytes[0], bytes[1]]);
        let ack = u16::from_le_bytes([bytes[2], bytes[3]]);
        let ack_bits = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let channel = bytes[8];
        let flags = bytes[9];
        let mut payload_offset = RELIABLE_UDP_BASE_HEADER_LEN;
        let fragment = if flags & RELIABLE_UDP_FLAG_FRAGMENT != 0 {
            if bytes.len() < RELIABLE_UDP_BASE_HEADER_LEN + RELIABLE_UDP_FRAGMENT_HEADER_LEN {
                return Err(ReliableUdpWirePacketError::FragmentHeaderMissing);
            }
            payload_offset += RELIABLE_UDP_FRAGMENT_HEADER_LEN;
            Some(ReliableUdpFragmentHeader {
                fragment_id: u16::from_le_bytes([bytes[10], bytes[11]]),
                fragment_index: bytes[12],
                fragment_count: bytes[13],
            })
        } else {
            None
        };

        Ok(Self {
            header: ReliableUdpWireHeader {
                sequence,
                ack,
                ack_bits,
                channel,
                flags,
                fragment,
            },
            payload: bytes[payload_offset..].to_vec(),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReliableUdpWirePacketError {
    HeaderTooShort,
    FragmentHeaderMissing,
}

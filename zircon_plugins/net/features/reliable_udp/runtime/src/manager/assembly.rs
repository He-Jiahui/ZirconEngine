use zircon_runtime::core::framework::net::{ReliableDatagramPacket, ReliableDatagramReceiveStatus};

// Keeps partial datagrams in fragment-index order so out-of-order delivery can be
// reassembled without leaking runtime-owned buffers through the public contract.
#[derive(Debug)]
pub(in crate::manager) struct InboundFragmentAssembly {
    channel: String,
    fragment_count: u16,
    fragments: Vec<Option<Vec<u8>>>,
}

impl InboundFragmentAssembly {
    pub(in crate::manager) fn new(packet: &ReliableDatagramPacket) -> Self {
        Self {
            channel: packet.channel.clone(),
            fragment_count: packet.fragment_count,
            fragments: vec![None; packet.fragment_count as usize],
        }
    }

    pub(in crate::manager) fn insert(
        &mut self,
        packet: &ReliableDatagramPacket,
    ) -> ReliableDatagramReceiveStatus {
        if packet.fragment_count != self.fragment_count
            || packet.channel != self.channel
            || packet.fragment_index >= self.fragment_count
        {
            return ReliableDatagramReceiveStatus::InvalidFragment;
        }
        let fragment = &mut self.fragments[packet.fragment_index as usize];
        if fragment.is_some() {
            return ReliableDatagramReceiveStatus::DuplicateFragment;
        }
        *fragment = Some(packet.payload.clone());
        if self.fragments.iter().all(Option::is_some) {
            ReliableDatagramReceiveStatus::Reassembled
        } else {
            ReliableDatagramReceiveStatus::AcceptedFragment
        }
    }

    pub(in crate::manager) fn payload(&self) -> Vec<u8> {
        self.fragments
            .iter()
            .flat_map(|fragment| fragment.as_deref().unwrap_or_default())
            .copied()
            .collect()
    }
}

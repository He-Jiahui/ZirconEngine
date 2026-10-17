//! App-owned connection incarnations, captured before forwarding/queueing any sample.
//! Native Gilrs queue provenance is a separate backend gate: this layer never relabels
//! an already captured sample after a reconnect.
use std::collections::BTreeMap;
use std::num::NonZeroU64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::entry::runtime_entry_app) struct CapturedGamepadStamp {
    pub(super) gamepad: u64,
    pub(super) generation: NonZeroU64,
}

#[derive(Default)]
pub(in crate::entry::runtime_entry_app) struct GamepadProducer {
    connections: BTreeMap<u64, (NonZeroU64, bool)>,
}

impl GamepadProducer {
    pub(super) fn connect(&mut self, gamepad: u64) -> Result<CapturedGamepadStamp, &'static str> {
        let generation = match self.connections.get(&gamepad) {
            None => NonZeroU64::MIN,
            Some(&(generation, true)) => generation,
            Some(&(generation, false)) => generation.get().checked_add(1)
                .and_then(NonZeroU64::new)
                .ok_or("runtime_gamepad_generation_exhausted")?,
        };
        self.connections.insert(gamepad, (generation, true));
        Ok(CapturedGamepadStamp { gamepad, generation })
    }

    pub(super) fn disconnect(&mut self, gamepad: u64) -> CapturedGamepadStamp {
        let generation = self.connections.get(&gamepad)
            .map(|(generation, _)| *generation).unwrap_or(NonZeroU64::MIN);
        self.connections.insert(gamepad, (generation, false));
        CapturedGamepadStamp { gamepad, generation }
    }

    pub(super) fn capture_sample(&self, gamepad: u64) -> Option<CapturedGamepadStamp> {
        self.connections.get(&gamepad).and_then(|&(generation, connected)| {
            connected.then_some(CapturedGamepadStamp { gamepad, generation })
        })
    }

    pub(super) fn is_connected(&self, gamepad: u64) -> bool {
        self.capture_sample(gamepad).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ZrRuntimeGamepadGenerationV2;

    #[test]
    fn producer_captured_sample_keeps_old_generation_across_reconnect() {
        let mut producer = GamepadProducer::default();
        assert_eq!(producer.capture_sample(4), None);
        let first = producer.connect(4).unwrap();
        let captured = producer.capture_sample(4).unwrap();
        assert_eq!(producer.disconnect(4), first);
        assert_eq!(producer.capture_sample(4), None);
        let second = producer.connect(4).unwrap();
        assert!(second.generation > first.generation);
        let delayed_payload = ZrRuntimeGamepadGenerationV2::encode_sample(captured.generation);
        assert_eq!(ZrRuntimeGamepadGenerationV2::decode_sample(&delayed_payload), Ok(first.generation));
        assert_ne!(captured.generation, producer.capture_sample(4).unwrap().generation);
    }

    #[test]
    fn producer_duplicate_inventory_and_connect_keep_generation() {
        let mut producer = GamepadProducer::default();
        let inventory = producer.connect(5).unwrap();
        assert_eq!(producer.connect(5).unwrap(), inventory);
        assert_eq!(producer.capture_sample(5), Some(inventory));
        assert_eq!(producer.disconnect(5), inventory);
        assert_eq!(producer.disconnect(5), inventory);
        let second = producer.connect(5).unwrap();
        assert_eq!(second.generation.get(), inventory.generation.get() + 1);
        let unknown_disconnect = producer.disconnect(6);
        assert!(!producer.is_connected(6));
        assert!(producer.connect(6).unwrap().generation > unknown_disconnect.generation);
    }

    #[test]
    fn producer_generation_overflow_fails_without_wrap() {
        let mut producer = GamepadProducer::default();
        let max = NonZeroU64::new(u64::MAX).unwrap();
        producer.connections.insert(7, (max, false));
        assert_eq!(producer.connect(7), Err("runtime_gamepad_generation_exhausted"));
        assert_eq!(producer.connections.get(&7), Some(&(max, false)));
        assert_eq!(producer.capture_sample(7), None);
    }
}

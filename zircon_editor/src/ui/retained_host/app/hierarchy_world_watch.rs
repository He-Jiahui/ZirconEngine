use zircon_runtime_interface::world_sync::WatchToken;

/// One retained hierarchy subscription, bound to the runtime session that issued it.
///
/// Runtime watch tokens are opaque and session-local. Retaining the gateway generation with the
/// token prevents a replacement session from receiving an unwatch for an unrelated token value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct HierarchyWorldWatch {
    token: WatchToken,
    gateway_generation: u64,
}

impl HierarchyWorldWatch {
    pub(super) const fn new(token: WatchToken, gateway_generation: u64) -> Self {
        Self {
            token,
            gateway_generation,
        }
    }

    pub(super) const fn token(self) -> WatchToken {
        self.token
    }

    pub(super) const fn belongs_to_gateway_generation(self, gateway_generation: u64) -> bool {
        self.gateway_generation == gateway_generation
    }
}

#[cfg(test)]
mod tests {
    use super::HierarchyWorldWatch;
    use zircon_runtime_interface::world_sync::WatchToken;

    #[test]
    fn hierarchy_watch_only_belongs_to_its_issuing_gateway_generation() {
        let watch = HierarchyWorldWatch::new(WatchToken::new(7), 3);

        assert_eq!(watch.token(), WatchToken::new(7));
        assert!(watch.belongs_to_gateway_generation(3));
        assert!(!watch.belongs_to_gateway_generation(4));
    }
}

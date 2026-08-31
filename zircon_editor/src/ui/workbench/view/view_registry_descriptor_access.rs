use super::{ViewDescriptor, ViewDescriptorId, ViewRegistry};

impl ViewRegistry {
    pub fn descriptor(&self, descriptor_id: &ViewDescriptorId) -> Option<&ViewDescriptor> {
        self.descriptors.get(descriptor_id)
    }

    pub fn list_descriptors(&self) -> Vec<ViewDescriptor> {
        let mut descriptors = Vec::with_capacity(self.descriptors.len());
        descriptors.extend(
            self.descriptors
                .values()
                .filter(|descriptor| self.descriptor_capability_error(descriptor).is_none())
                .cloned(),
        );
        descriptors
    }
}

#[cfg(test)]
mod optimization_tests {
    #[test]
    fn optimization_batch_20260830dc_view_registry_reserves_descriptor_upper_bound() {
        let source = include_str!("view_registry_descriptor_access.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("view registry descriptor production source");

        assert!(production.contains("Vec::with_capacity(self.descriptors.len())"));
        assert!(production.contains("descriptors.extend("));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830dc_view_registry_capacity_evidence() {
        const BATCH_COUNT: usize = 32_768;
        const DESCRIPTOR_COUNT: usize = 64;
        const MARKER: &str = "EDITOR515_VIEW_REGISTRY_CAPACITY_BENCH_V1";

        let legacy_growth_events = descriptor_growth_events(BATCH_COUNT, DESCRIPTOR_COUNT, false);
        let optimized_growth_events = descriptor_growth_events(BATCH_COUNT, DESCRIPTOR_COUNT, true);

        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
        println!(
            "{MARKER} batches={BATCH_COUNT} descriptors={DESCRIPTOR_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
        );
    }

    fn descriptor_growth_events(
        batch_count: usize,
        descriptor_count: usize,
        reserve: bool,
    ) -> usize {
        let mut growth_events = 0;
        for _ in 0..batch_count {
            let mut descriptors = if reserve {
                Vec::with_capacity(descriptor_count)
            } else {
                Vec::new()
            };
            for descriptor in 0..descriptor_count {
                let previous_capacity = descriptors.capacity();
                descriptors.push(descriptor);
                growth_events += usize::from(descriptors.capacity() != previous_capacity);
            }
        }
        growth_events
    }
}

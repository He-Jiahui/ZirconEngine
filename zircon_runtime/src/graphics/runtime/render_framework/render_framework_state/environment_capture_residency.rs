use std::collections::{HashMap, VecDeque};

use crate::core::framework::render::RenderEnvironmentCaptureHandle;
use crate::graphics::scene::EnvironmentCaptureResidentOutput;

pub(super) const MAX_RESIDENT_ENVIRONMENT_CAPTURES: usize = 64;

/// Bounded last-good GPU outputs keyed by the neutral capture identity.
///
/// Publication replaces one complete output atomically. Source cubemap and
/// depth scratch have already been dropped by `into_resident_output`.
pub(in crate::graphics::runtime::render_framework) struct EnvironmentCaptureResidency {
    outputs: HashMap<String, EnvironmentCaptureResidentOutput>,
    order: VecDeque<String>,
    observation_epoch: u64,
    last_published_handle: Option<RenderEnvironmentCaptureHandle>,
    last_published_output_generation: Option<u64>,
    resident_gpu_bytes: u64,
    eviction_count: u64,
}

impl Default for EnvironmentCaptureResidency {
    fn default() -> Self {
        Self {
            outputs: HashMap::with_capacity(MAX_RESIDENT_ENVIRONMENT_CAPTURES),
            order: VecDeque::with_capacity(MAX_RESIDENT_ENVIRONMENT_CAPTURES),
            observation_epoch: 0,
            last_published_handle: None,
            last_published_output_generation: None,
            resident_gpu_bytes: 0,
            eviction_count: 0,
        }
    }
}

impl EnvironmentCaptureResidency {
    pub(in crate::graphics::runtime::render_framework) fn publish(
        &mut self,
        output: EnvironmentCaptureResidentOutput,
    ) {
        self.observation_epoch = self.observation_epoch.saturating_add(1);
        self.last_published_handle = Some(output.handle());
        self.last_published_output_generation = Some(output.identity().output_generation());
        let capture_id = output.identity().capture_id().to_string();
        if let Some(previous) = self.outputs.remove(&capture_id) {
            self.resident_gpu_bytes = self.resident_gpu_bytes.saturating_sub(previous.gpu_bytes());
            if let Some(index) = self.order.iter().position(|known| known == &capture_id) {
                self.order.remove(index);
            }
        }
        self.resident_gpu_bytes = self.resident_gpu_bytes.saturating_add(output.gpu_bytes());
        self.order.push_back(capture_id.clone());
        self.outputs.insert(capture_id, output);

        while self.outputs.len() > MAX_RESIDENT_ENVIRONMENT_CAPTURES {
            let Some(evicted_id) = self.order.pop_front() else {
                break;
            };
            if let Some(evicted) = self.outputs.remove(&evicted_id) {
                self.resident_gpu_bytes =
                    self.resident_gpu_bytes.saturating_sub(evicted.gpu_bytes());
                self.eviction_count = self.eviction_count.saturating_add(1);
            }
        }
    }

    pub(in crate::graphics::runtime::render_framework) fn get(
        &self,
        capture_id: &str,
    ) -> Option<&EnvironmentCaptureResidentOutput> {
        self.outputs.get(capture_id)
    }

    pub(in crate::graphics::runtime::render_framework) fn len(&self) -> usize {
        self.outputs.len()
    }

    pub(in crate::graphics::runtime::render_framework) fn resident_gpu_bytes(&self) -> u64 {
        self.resident_gpu_bytes
    }

    pub(in crate::graphics::runtime::render_framework) fn observation_epoch(&self) -> u64 {
        self.observation_epoch
    }

    pub(in crate::graphics::runtime::render_framework) fn last_published_handle(
        &self,
    ) -> Option<RenderEnvironmentCaptureHandle> {
        self.last_published_handle
    }

    pub(in crate::graphics::runtime::render_framework) fn last_published_output_generation(
        &self,
    ) -> Option<u64> {
        self.last_published_output_generation
    }

    pub(in crate::graphics::runtime::render_framework) fn eviction_count(&self) -> u64 {
        self.eviction_count
    }
}

#[cfg(test)]
#[path = "tests/environment_capture_residency_source_contract_tests.rs"]
mod source_contract_tests;
